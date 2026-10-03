//! MC-018, AC-1 to AC-6: the calibration corpus is well formed.
//!
//! Twenty-eight real screenshots in `fixtures/corpus/` with a hand-marked
//! expected crop for each. These tests check that the corpus is *usable* -
//! every file present and decodable, every rectangle inside its image, the tag
//! vocabulary covered, the whole set small enough to clone. They do **not**
//! check that a rectangle is the right rectangle; no code can, and MC-019's
//! guidance is explicit that a reported clip may be a mis-marked rect rather
//! than a detector fault.
//!
//! Not `#[ignore]`d: these run in the `unit` gate on every commit, because a
//! corpus that has quietly rotted - a file renamed, an image replaced with a
//! different size - turns MC-019 into a test that measures nothing and says so
//! in a way nobody can read.
//!
//! # What is settled, what is mechanical
//!
//! * **Settled elsewhere, read out rather than re-derived**: the nine required
//!   tags and the size ceiling, both from MC-018's acceptance criteria (the
//!   ceiling raised from 60 to 100 MiB by MC-062 and to 120 MiB by MC-068,
//!   both the user's requests); the
//!   twenty-entry floor; the WebP container layout, which AC-6 reads out of
//!   the file's own bytes.
//! * **Mechanical**: everything else. Exact counts, exact bounds, exact byte
//!   comparisons.
//!
//! # Why AC-6 reads the RIFF header itself
//!
//! Nothing in this workspace can *write* a lossy WebP (`image` 0.25 encodes
//! `VP8L` and nothing else), which is why MC-009's AC-3 was deferred to this
//! story: the corpus is where a real one arrives. "Is this lossy" therefore
//! has to be answered from the container, not from a decoder - a decoder
//! happily reads both and tells you nothing about which it read.

// Included directly rather than through `common/mod.rs`. The app crate reaches
// that module with `#[path]`, so declaring the loader there would compile it -
// and its `serde` dependency - into test binaries that have no use for either.
#[path = "common/corpus.rs"]
mod corpus;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// MC-018 AC-1. Below this the corpus cannot support a "nine in ten" claim
/// with any resolution: at twenty entries one miss is five percent.
const MIN_ENTRIES: usize = 20;

/// MC-018 AC-4, so the repository stays clonable without LFS.
///
/// **Raised from 60 MiB to 100 MiB by MC-062, at the user's request of
/// 2026-09-30** ("raise the corpus ceiling"). The corpus was 61,587,028 bytes
/// (58.7 MiB) before MC-062's fresh held-out draw, and the 25 drawn files add
/// 32,232,535 bytes, which brings it to about 89.5 MiB: over 60, under 100.
///
/// **Raised from 100 MiB to 120 MiB by MC-068, on the user's answer of
/// 2026-10-01** ("Raise it to 120 MiB"). The corpus was 93,824,337 bytes
/// (89.5 MiB) before MC-068's second fresh draw, and the 15 drawn files add
/// 18,611,679 bytes, which brings it to 112,436,016 bytes (107.2 MiB): over
/// 100, under 120.
///
/// **Raised from 120 MiB to 140 MiB by MC-077, on the user's answer of
/// 2026-10-03** ("Raise it to 140 MiB"). The corpus was 125,214,051 bytes
/// (119.4 MiB) before MC-077's third fresh draw, and the 10 drawn files add
/// 13,041,341 bytes, which brings it to about 131.9 MiB: over 120, under 140.
const MAX_BYTES: u64 = 140 * 1024 * 1024;

/// MC-018 AC-3. The union of every entry's tags must cover all nine; extra
/// tags are valid and deliberately not constrained.
const REQUIRED_TAGS: [&str; 9] = [
    "light-theme",
    "dark-theme",
    "white-gutter",
    "black-gutter",
    "all-art",
    "mostly-white",
    "png",
    "jpeg",
    "webp",
];

/// Tags that assert there is nothing to crop, so the entry must be `"flag"`.
const FLAG_ONLY_TAGS: [&str; 2] = ["all-art", "mostly-white"];

// --- Harness ----------------------------------------------------------------

/// The decoded dimensions of `path`, from the `image` crate - the same decoder
/// the engine uses, so "it decodes" here means "the engine can read it".
fn dimensions(path: &Path) -> (u32, u32) {
    let img = image::ImageReader::open(path)
        .unwrap_or_else(|err| panic!("opening {}: {err}", path.display()))
        .with_guessed_format()
        .unwrap_or_else(|err| panic!("sniffing {}: {err}", path.display()))
        .decode()
        .unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()));
    (img.width(), img.height())
}

/// The four-byte chunk FourCC after the `RIFF....WEBP` header, or `None` if
/// this is not a RIFF/WEBP file at all. `VP8 ` is lossy, `VP8L` lossless,
/// `VP8X` an extended container.
fn webp_chunk(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() < 16 {
        return None;
    }
    let tag = |at: usize| String::from_utf8_lossy(&bytes[at..at + 4]).into_owned();
    if tag(0) != "RIFF" || tag(8) != "WEBP" {
        return None;
    }
    Some(tag(12))
}

/// A temp directory to write outputs into, removed when the test ends.
fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temp dir")
}

// --- AC-1: it parses, it is big enough, every file is really there ----------

#[test]
fn the_manifest_parses_and_names_at_least_twenty_files_that_all_exist() {
    let entries = corpus::load();

    assert!(
        entries.len() >= MIN_ENTRIES,
        "AC-1: the corpus needs at least {MIN_ENTRIES} entries to support a \
         nine-in-ten claim with any resolution; the manifest has {}",
        entries.len()
    );

    let missing: Vec<String> = entries
        .iter()
        .filter(|e| !e.path.is_file())
        .map(CorpusEntry::name)
        .collect();
    assert_eq!(
        missing,
        Vec::<String>::new(),
        "AC-1: every file the manifest names must exist in {}. A manifest that \
         names a file nobody added is a corpus entry that silently does not \
         participate",
        corpus::dir().display()
    );
}

#[test]
fn every_corpus_file_decodes_with_the_engines_own_decoder() {
    let entries = corpus::load();

    let sizes: Vec<(String, bool)> = entries
        .iter()
        .map(|e| {
            let (w, h) = dimensions(&e.path);
            (e.name(), w >= 1 && h >= 1)
        })
        .collect();
    let expected: Vec<(String, bool)> = entries.iter().map(|e| (e.name(), true)).collect();

    assert_eq!(
        sizes, expected,
        "AC-1: every corpus file must decode to a real image with the `image` \
         crate - the decoder the engine itself uses. A file that only opens in \
         a viewer is a file MC-019 will report as `failed`. Each row is (name, \
         decoded to a non-empty image)"
    );
}

// --- AC-2: every rectangle lands inside its own image ------------------------

#[test]
fn every_expected_rect_is_non_empty_and_within_its_image() {
    let entries = corpus::load();
    let mut checked = 0;
    let mut violations: Vec<String> = Vec::new();

    for entry in &entries {
        let Expect::Rect(r) = entry.expect else {
            continue;
        };
        checked += 1;
        let (w, h) = dimensions(&entry.path);
        if r.w < 1 || r.h < 1 {
            violations.push(format!(
                "{}: w={} h={}, both must be >= 1",
                entry.name(),
                r.w,
                r.h
            ));
        }
        if r.x + r.w > w {
            violations.push(format!(
                "{}: x+w = {} but the image is {w} wide",
                entry.name(),
                r.x + r.w
            ));
        }
        if r.y + r.h > h {
            violations.push(format!(
                "{}: y+h = {} but the image is {h} tall",
                entry.name(),
                r.y + r.h
            ));
        }
    }

    assert_eq!(
        violations,
        Vec::<String>::new(),
        "AC-2: a rectangle that leaves its image cannot be a crop of it, and \
         MC-019 would report the miss against the detector rather than against \
         the manifest"
    );
    assert!(
        checked > 0,
        "AC-2 checked no rectangles at all. Every entry is `\"flag\"`, so this \
         test is passing vacuously and the corpus cannot measure a crop"
    );
}

// --- AC-3: the tag vocabulary, and the one tag pair that constrains expect --

#[test]
fn the_tags_across_the_corpus_cover_every_required_case() {
    let entries = corpus::load();

    let union: BTreeSet<&str> = entries
        .iter()
        .flat_map(|e| e.tags.iter().map(String::as_str))
        .collect();
    let missing: Vec<&str> = REQUIRED_TAGS
        .into_iter()
        .filter(|t| !union.contains(t))
        .collect();

    assert_eq!(
        missing,
        Vec::<&str>::new(),
        "AC-3: the corpus must exercise every one of these cases at least \
         once, or the detector is tuned against a world narrower than the one \
         it ships into. Tags present: {union:?}"
    );
}

#[test]
fn all_art_and_mostly_white_entries_expect_a_flag_and_not_a_rect() {
    let entries = corpus::load();

    let contradictions: Vec<String> = entries
        .iter()
        .filter(|e| FLAG_ONLY_TAGS.iter().any(|t| e.has_tag(t)))
        .filter(|e| e.expect != Expect::Flag)
        .map(|e| format!("{} is tagged {:?} but carries a rect", e.name(), e.tags))
        .collect();

    assert_eq!(
        contradictions,
        Vec::<String>::new(),
        "AC-3: `all-art` and `mostly-white` both say there is nothing to crop, \
         so the only expectation consistent with either is \"flag\". An entry \
         claiming both is one MC-019 cannot score either way"
    );
}

// --- AC-4: the corpus stays clonable ----------------------------------------

#[test]
fn the_whole_corpus_fits_under_one_hundred_and_forty_mebibytes() {
    let dir = corpus::dir();
    let total: u64 = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .map(|e| e.expect("a directory entry"))
        .filter(|e| e.path().is_file())
        .map(|e| e.metadata().expect("file metadata").len())
        .sum();

    assert!(
        total < MAX_BYTES,
        "AC-4: the corpus is {:.1} MiB, over the {:.0} MiB ceiling that keeps \
         this repository clonable without LFS (MC-062 raised it from 60 to 100 \
         MiB, the user's request of 2026-09-30; MC-068 from 100 to 120 MiB, the \
         user's answer of 2026-10-01; MC-077 from 120 to 140 MiB, the user's \
         answer of 2026-10-03, \"Raise it to 140 MiB\")",
        total as f64 / 1_048_576.0,
        MAX_BYTES as f64 / 1_048_576.0
    );
}

// --- AC-5: the loader ---------------------------------------------------------

#[test]
fn the_loader_returns_every_entry_in_manifest_order() {
    let text = fs::read_to_string(corpus::dir().join(corpus::MANIFEST_FILE))
        .expect("the manifest is readable");
    let entries = corpus::load();

    // The order the loader reports, checked against the order the names appear
    // in the file's own text rather than against a second parse of it - a
    // parser that reordered entries would reorder both.
    let mut cursor = 0usize;
    let mut out_of_order: Vec<String> = Vec::new();
    for entry in &entries {
        let needle = format!("\"{}\"", entry.name());
        match text[cursor..].find(&needle) {
            Some(at) => cursor += at + needle.len(),
            None => out_of_order.push(entry.name()),
        }
    }

    assert_eq!(
        out_of_order,
        Vec::<String>::new(),
        "AC-5: `load()` must return entries in manifest order. MC-019 prints a \
         per-file table, and a table whose rows move between runs is one \
         nobody can diff"
    );
    assert!(
        entries.iter().any(|e| matches!(e.expect, Expect::Rect(_))),
        "AC-5: the loader produced no Expect::Rect at all"
    );
    assert!(
        entries.iter().any(|e| e.expect == Expect::Flag),
        "AC-5: the loader produced no Expect::Flag at all"
    );
}

#[test]
fn every_loaded_path_stays_inside_the_corpus_directory() {
    let dir = corpus::dir();
    let escapes: Vec<PathBuf> = corpus::load()
        .into_iter()
        .map(|e| e.path)
        .filter(|p| p.parent() != Some(dir.as_path()))
        .collect();

    assert_eq!(
        escapes,
        Vec::<PathBuf>::new(),
        "AC-5: a manifest names a bare file and the corpus is one flat \
         directory, so every loaded path must sit directly in {}. A manifest \
         that could reach out of it could point anywhere on the machine \
         running these tests",
        dir.display()
    );
}

// --- AC-6: a real lossy WebP, cropped losslessly ----------------------------

#[test]
fn at_least_one_corpus_entry_is_a_genuinely_lossy_webp() {
    let entries = corpus::load();

    let webps: Vec<(String, Option<String>)> = entries
        .iter()
        .filter(|e| {
            e.path
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("webp"))
        })
        .map(|e| (e.name(), webp_chunk(&e.path)))
        .collect();
    let lossy: Vec<&(String, Option<String>)> = webps
        .iter()
        .filter(|(_, chunk)| chunk.as_deref() == Some("VP8 "))
        .collect();

    assert!(
        !lossy.is_empty(),
        "AC-6: the corpus must contain at least one genuinely lossy WebP - a \
         `VP8 ` chunk, not `VP8L`. This criterion is MC-009's AC-3, deferred \
         here because nothing in this workspace can *write* one, so the corpus \
         is the only place a real one can arrive. WebPs found: {webps:?}"
    );
}

#[test]
fn cropping_a_lossy_webp_reproduces_the_decoded_source_pixels_exactly() {
    let entries = corpus::load();
    let entry = entries
        .iter()
        .find(|e| webp_chunk(&e.path).as_deref() == Some("VP8 "))
        .expect("AC-6: a lossy WebP in the corpus (the test above says which)");

    let tmp = scratch();
    let output = tmp.path().join(entry.name());
    let result = process_file(&entry.path, &output, &Tuning::default());

    // WebP is re-encoded *lossless* (architecture.md, `codec`), so whatever
    // region was kept must come back byte-identical to the decoded source over
    // that region. Comparing against the decoded source and not the file's
    // bytes is the whole point: the source is lossy, so its bytes and its
    // pixels are different things.
    let source = image::ImageReader::open(&entry.path)
        .expect("the source opens")
        .with_guessed_format()
        .expect("the source sniffs")
        .decode()
        .expect("the source decodes");

    let kept: Rect = match &result.outcome {
        Outcome::Cropped { rect, .. } => *rect,
        Outcome::Flagged { .. } => Rect {
            x: 0,
            y: 0,
            w: source.width(),
            h: source.height(),
        },
        Outcome::Failed { error } => panic!(
            "AC-6: {} could not be processed at all: {error}",
            entry.name()
        ),
    };

    assert!(
        webp_chunk(&output).is_some(),
        "AC-6: the output of a WebP input must itself be a WebP; got {:?}",
        webp_chunk(&output)
    );

    let written = image::ImageReader::open(&output)
        .expect("the output opens")
        .with_guessed_format()
        .expect("the output sniffs")
        .decode()
        .expect("the output decodes");
    let expected = source.crop_imm(kept.x, kept.y, kept.w, kept.h);

    assert_eq!(
        (written.width(), written.height()),
        (expected.width(), expected.height()),
        "AC-6: the output's dimensions must be the kept rect's, {kept:?}"
    );
    assert!(
        written.to_rgba8().as_raw() == expected.to_rgba8().as_raw(),
        "AC-6: cropping {} kept {kept:?}, and WebP is re-encoded lossless, so \
         every pixel inside that rect must equal the *decoded* source pixel. A \
         difference here means the crop re-compressed a lossy source instead \
         of copying its decoded pixels",
        entry.name()
    );
}

// --- MC-033: the corpus's own rules are written down, and the manifest agrees

/// MC-033 AC-1. The entries a person confirmed have a **diagonal gutter**, on
/// 2026-09-17, by opening every marked entry and ruling on the marks they
/// drew (MC-032's `## Notes`).
///
/// This is a settled list read out here, never re-derived. MC-032 records the
/// instrument that would re-derive it - per column, the row of strongest
/// vertical gradient near the mark, then the slope and fit of those rows
/// against x - and records that it finds `2025-10-20 15_37_25`'s *top* edge
/// and nothing else: `Screenshot (93).jpg`, identified instantly by eye,
/// scores 14.9 rows per 100 columns at r2 0.12, and the user's diagonal on
/// `15_37_25` is its *bottom* edge, which the same instrument calls flat. Art
/// texture dominates the gradient. A third entry joins this list when a person
/// confirms one, and not before.
const DIAGONAL_GUTTER_ENTRIES: [&str; 2] = ["Screenshot (93).jpg", "2025-10-20 15_37_25.png"];

/// The tag under test above.
const DIAGONAL_GUTTER: &str = "diagonal-gutter";

/// MC-033 AC-3. The corpus's one written source of truth, relative to the
/// repository root: the marking rule, the diagonal-gutter tolerance, and the
/// tag vocabulary AC-2 checks against.
const CORPUS_PAGE: [&str; 3] = ["docs", "wiki", "corpus.md"];

/// The line that introduces the vocabulary table on that page.
const VOCABULARY_MARKER: &str = "<!-- tag-vocabulary -->";

/// The tag vocabulary, **read out of the page** rather than repeated here.
///
/// A `const KNOWN_TAGS` in this file would be a second copy of the list, and
/// two copies of the marking rule in two places going out of step with each
/// other is exactly the defect MC-033 exists to remove. So the page is the
/// source and this is the parser.
///
/// The vocabulary is the markdown table following [`VOCABULARY_MARKER`], and a
/// vocabulary entry is a row whose **first cell is a backticked tag**. The
/// header row (`| Tag |`) and the `|---|` separator are therefore excluded by
/// their shape, not by counting lines.
///
/// # Panics
///
/// If the page is missing or carries no marker. Both mean this test has
/// nothing to check against, which is a broken checkout rather than a failure
/// to report politely.
fn documented_tag_vocabulary() -> BTreeSet<String> {
    documented_vocabulary(VOCABULARY_MARKER, "AC-2")
}

/// The backticked first cells of the markdown table following `marker` on the
/// corpus page.
///
/// Extracted from `documented_tag_vocabulary` by MC-036, which needs the same
/// parse against a second marker. One parser, two callers: two copies of *this*
/// would be the same drift at one remove, and the point of reading a vocabulary
/// off the page is that there is exactly one copy of it anywhere.
///
/// A vocabulary entry is a row whose **first cell is a backticked value**. The
/// header row (`| Tag |`) and the `|---|` separator are therefore excluded by
/// their shape, not by counting lines.
///
/// `ac` is the criterion quoted in the failure messages, since the two callers
/// answer to different ones.
///
/// # Panics
///
/// If the page is missing, carries no `marker`, or the table after it yields
/// nothing. All three mean the caller has nothing to check against, which is a
/// broken checkout rather than a failure to report politely - and, crucially,
/// not a pass.
fn documented_vocabulary(marker: &str, ac: &str) -> BTreeSet<String> {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    for part in CORPUS_PAGE {
        path.push(part);
    }
    let text = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{ac}: reading {}: {err}. The vocabulary lives on that page and \
             nowhere else, so without it this test has nothing to check against",
            path.display()
        )
    });
    let after = text
        .split_once(marker)
        .unwrap_or_else(|| {
            panic!(
                "{ac}: {} carries no {marker} line, so the vocabulary table \
                 cannot be located on it",
                path.display()
            )
        })
        .1;

    let values: BTreeSet<String> = after
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| line.trim_start().starts_with('|'))
        .filter_map(|row| {
            let first = row.trim().trim_start_matches('|').split('|').next()?.trim();
            first
                .strip_prefix('`')?
                .strip_suffix('`')
                .map(str::to_owned)
        })
        .collect();

    assert!(
        !values.is_empty(),
        "{ac}: the table after {marker} in {} yielded nothing. A vocabulary \
         this test reads as empty would reject every value in the manifest, so \
         the parse is wrong rather than the corpus: a vocabulary row is a table \
         row whose first cell is a backticked value",
        path.display()
    );
    values
}

#[test]
fn exactly_the_two_confirmed_entries_carry_the_diagonal_gutter_tag() {
    let entries = corpus::load();
    let mut problems: Vec<String> = Vec::new();

    for name in DIAGONAL_GUTTER_ENTRIES {
        if !entries
            .iter()
            .any(|e| e.name() == name && e.has_tag(DIAGONAL_GUTTER))
        {
            problems.push(format!(
                "{name}: a person confirmed this gutter is diagonal on \
                 2026-09-17, but its manifest entry does not carry \
                 `{DIAGONAL_GUTTER}`"
            ));
        }
    }
    for entry in &entries {
        let name = entry.name();
        if entry.has_tag(DIAGONAL_GUTTER) && !DIAGONAL_GUTTER_ENTRIES.contains(&name.as_str()) {
            problems.push(format!(
                "{name}: carries `{DIAGONAL_GUTTER}`, which is not one of the \
                 entries a person confirmed. The instrument that would find a \
                 third cannot (MC-032's `## Notes`), so a third is a person's \
                 call and arrives with this list"
            ));
        }
    }

    assert_eq!(
        problems,
        Vec::<String>::new(),
        "AC-1: the entries tagged `{DIAGONAL_GUTTER}` must be exactly \
         {DIAGONAL_GUTTER_ENTRIES:?}. `Screenshot (93).jpg` is MC-019's sole \
         column miss, at +20 px on the right, and the diagonal is why: when the \
         gutter runs diagonally the page's left and right edges are \
         row-dependent, so no single pair of columns is right for the whole page"
    );
}

#[test]
fn every_tag_in_the_manifest_is_in_the_documented_vocabulary() {
    let vocabulary = documented_tag_vocabulary();
    let entries = corpus::load();

    // One direction only. A documented tag nothing carries is deliberate -
    // `overhang-text` is defined and unapplied, because the survey that would
    // settle which entries deserve it has not been done and a guessed tag is
    // worse than an absent one (MC-033, ## Out of scope).
    let strays: Vec<String> = entries
        .iter()
        .flat_map(|e| e.tags.iter().map(move |t| (e.name(), t.clone())))
        .filter(|(_, tag)| !vocabulary.contains(tag))
        .map(|(name, tag)| format!("{name}: tag {tag:?}"))
        .collect();

    assert_eq!(
        strays,
        Vec::<String>::new(),
        "AC-2: every tag in the manifest must appear in the vocabulary table on \
         docs/wiki/corpus.md. A misspelt tag - `diagonal-gutters` for \
         `{DIAGONAL_GUTTER}` - is invisible today: it belongs to no category, \
         nothing reads it, and the entry it was meant to classify is silently \
         untagged. Documented vocabulary: {vocabulary:?}"
    );
}

// --- MC-036: the tuning / held-out split ------------------------------------

/// MC-036 AC-2. The corpus as it stood at commit `a453aaa`, immediately before
/// EPIC-07 - the twenty-eight entries every v1 investigation was tuned against.
///
/// **This is a settled list, read out and never re-derived.** It is not
/// "whatever is in the manifest minus the new ones", because that definition
/// would silently absorb a future entry and quietly launder it into the tuning
/// set. It is the literal output of
/// `git show a453aaa:fixtures/corpus/manifest.json`, in manifest order.
///
/// Every one of these is `tuning` permanently and cannot become held out.
/// MC-025, MC-028, MC-031, MC-032, MC-034 and MC-035 all fitted thresholds
/// against them and their per-file tables have been read by the people and
/// agents planning v2; they are contaminated, and no later decision can
/// un-contaminate them. `docs/wiki/corpus.md` carries the reasoning.
const PRE_EPIC_07_ENTRIES: [&str; 28] = [
    "2025-02-27 22_46_15.png",
    "2025-03-03 11_06_04.png",
    "2025-03-03 11_24_19.png",
    "2025-05-12 22_55_40.png",
    "2025-05-12 22_58_53.png",
    "2025-08-05 00_11_13.webp",
    "2025-08-05 00_11_27.webp",
    "2025-10-14 23_29_06.png",
    "2025-10-14 23_30_20.png",
    "2025-10-20 15_37_25.png",
    "2026-01-05 13_33_41.png",
    "2026-01-05 13_45_59.png",
    "2026-01-05 13_49_39.png",
    "Screenshot (67).png",
    "Screenshot (70).jpg",
    "Screenshot (75).png",
    "Screenshot (93).jpg",
    "Screenshot (103).jpg",
    "Screenshot (1661).png",
    "Screenshot (2582).jpg",
    "Screenshot (2630).jpg",
    "Screenshot (2698).jpg",
    "Screenshot (2708).jpg",
    "Screenshot (2744).jpg",
    "Screenshot (3187).png",
    "Screenshot (3455).png",
    "Screenshot (3465).png",
    "Screenshot (3538).png",
];

/// The line that introduces the split table on the corpus page.
const SPLIT_VOCABULARY_MARKER: &str = "<!-- split-vocabulary -->";

/// The permitted `split` values, **read out of `docs/wiki/corpus.md`** rather
/// than repeated here - the same arrangement, and the same reason, as
/// [`documented_tag_vocabulary`].
///
/// # Panics
///
/// If the page is missing or carries no [`SPLIT_VOCABULARY_MARKER`]. Either
/// means this test has nothing to check against, which is a broken checkout
/// rather than a failure to report politely - and specifically *not* a pass.
fn documented_split_vocabulary() -> BTreeSet<String> {
    documented_vocabulary(SPLIT_VOCABULARY_MARKER, "AC-3")
}

/// The manifest's raw `"split"` strings, in manifest order, read out of the
/// file's own text.
///
/// Read from the JSON rather than from `load()` on purpose: these tests need to
/// compare what the manifest *says* against what the loader *reports*, and a
/// loader bug that mapped every value to `Tuning` would be invisible to any
/// check that asked the loader both times.
fn raw_splits() -> Vec<(String, String)> {
    let text = fs::read_to_string(corpus::dir().join(corpus::MANIFEST_FILE))
        .expect("the manifest is readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&text).expect("the manifest is valid JSON");
    parsed["entries"]
        .as_array()
        .expect("the manifest has an `entries` array")
        .iter()
        .map(|e| {
            let file = e["file"]
                .as_str()
                .expect("an entry names a file")
                .to_owned();
            let split = e["split"]
                .as_str()
                .unwrap_or_else(|| panic!("{file}: the manifest entry carries no `split` string"))
                .to_owned();
            (file, split)
        })
        .collect()
}

// --- AC-1: every entry carries a split, and it is the one the manifest says --

#[test]
fn every_entry_carries_the_split_its_manifest_entry_states() {
    // Reaching this line is already part of the criterion: `load()` panics on
    // an entry with no `split`, and on any value but the two documented ones.
    // That half of AC-1 is discharged by the probes in the story rather than
    // by an assertion here, because no assertion in this process can observe a
    // panic the loader raises before it returns.
    let entries = corpus::load();
    let raw = raw_splits();

    assert!(
        !entries.is_empty(),
        "AC-1: the loader returned no entries at all, so this test is checking \
         nothing"
    );

    let reported: Vec<(String, String)> = entries
        .iter()
        .map(|e| (e.name(), e.split.as_str().to_owned()))
        .collect();

    assert_eq!(
        reported, raw,
        "AC-1: every entry's `split` on `CorpusEntry` must be exactly the \
         string its manifest entry carries. Comparing the loader's answer \
         against a second, independent parse of the same file is what makes \
         this falsifiable: a loader that mapped every value to one variant \
         would satisfy any check that asked the loader twice. Each row is \
         (file, split)"
    );
}

// --- AC-2: the contamination guard ------------------------------------------

#[test]
fn every_pre_epic_07_entry_is_in_the_tuning_split() {
    let entries = corpus::load();

    assert!(
        !PRE_EPIC_07_ENTRIES.is_empty(),
        "AC-2: the settled list of pre-EPIC-07 entries is empty, so this test \
         measures nothing"
    );

    // An entry in the list that is not in the manifest would make this test
    // pass by checking fewer things than it claims to - the classic way a
    // guard stops guarding without anybody noticing.
    let absent: Vec<&str> = PRE_EPIC_07_ENTRIES
        .into_iter()
        .filter(|name| !entries.iter().any(|e| e.name() == *name))
        .collect();
    assert_eq!(
        absent,
        Vec::<&str>::new(),
        "AC-2: every name in PRE_EPIC_07_ENTRIES must still be present in the \
         manifest. A name that has been renamed or removed is one this guard \
         silently stops covering, and the corpus is the same twenty-eight \
         files before and after MC-036"
    );

    let contaminated_but_held_out: Vec<String> = entries
        .iter()
        .filter(|e| PRE_EPIC_07_ENTRIES.contains(&e.name().as_str()))
        .filter(|e| e.split != Split::Tuning)
        .map(|e| format!("{}: split {:?}", e.name(), e.split.as_str()))
        .collect();

    assert_eq!(
        contaminated_but_held_out,
        Vec::<String>::new(),
        "AC-2: all {} entries that existed at commit a453aaa are `tuning`, \
         permanently. Six v1 investigations fitted thresholds against them and \
         their per-file tables have been read, so they are contaminated and no \
         later decision can un-contaminate them. Moving one into the held-out \
         set would produce an accuracy number that looks rigorous and is not - \
         which is the exact failure MC-036 exists to prevent. New screenshots \
         are the only legitimate held-out set; MC-037 adds them",
        PRE_EPIC_07_ENTRIES.len()
    );
}

// --- AC-3: the two values are read off the page, not repeated in Rust -------

#[test]
fn every_split_in_the_manifest_is_in_the_documented_vocabulary() {
    let vocabulary = documented_split_vocabulary();

    let strays: Vec<String> = raw_splits()
        .into_iter()
        .filter(|(_, split)| !vocabulary.contains(split))
        .map(|(name, split)| format!("{name}: split {split:?}"))
        .collect();

    assert_eq!(
        strays,
        Vec::<String>::new(),
        "AC-3: every `split` in the manifest must appear in the table after \
         {SPLIT_VOCABULARY_MARKER} on docs/wiki/corpus.md. The page is the \
         source and this is the reader, deliberately: a second copy of the \
         vocabulary in Rust is the drift MC-033 removed for tags, and there is \
         no reason to reintroduce it here. Documented vocabulary: {vocabulary:?}"
    );

    // The other direction, which the tag vocabulary deliberately does not
    // assert. It is safe here and not there because this vocabulary is closed:
    // `Split` has exactly two variants, so a row on the page the loader cannot
    // produce is a page that documents a value no manifest can ever carry.
    let undeliverable: Vec<&String> = vocabulary
        .iter()
        .filter(|v| ![Split::Tuning.as_str(), Split::HeldOut.as_str()].contains(&v.as_str()))
        .collect();
    assert_eq!(
        undeliverable,
        Vec::<&String>::new(),
        "AC-3: the page documents a split value the loader cannot produce. \
         Unlike the tag vocabulary, this one is closed - `Split` has two \
         variants - so a third row is a promise to readers that nothing can keep"
    );
}

// --- AC-4: the split is additive --------------------------------------------

#[test]
fn the_loader_still_reports_path_expect_and_tags_exactly_as_the_manifest_gives_them() {
    let text = fs::read_to_string(corpus::dir().join(corpus::MANIFEST_FILE))
        .expect("the manifest is readable");
    let parsed: serde_json::Value =
        serde_json::from_str(&text).expect("the manifest is valid JSON");
    let raw = parsed["entries"]
        .as_array()
        .expect("the manifest has an `entries` array");
    let entries = corpus::load();

    assert_eq!(
        entries.len(),
        raw.len(),
        "AC-4: the loader must return one entry per manifest entry. Adding \
         `split` is additive: it does not drop, merge or reorder anything"
    );

    // Rendered as strings and compared in one shot so a failure prints the
    // whole picture - which entry, which field, and in which position - rather
    // than stopping at the first difference.
    let reported: Vec<String> = entries
        .iter()
        .map(|e| {
            let expect = match e.expect {
                Expect::Flag => "flag".to_owned(),
                Expect::Rect(r) => format!("{} {} {} {}", r.x, r.y, r.w, r.h),
            };
            format!("{} | {} | {}", e.name(), expect, e.tags.join(","))
        })
        .collect();
    let from_json: Vec<String> = raw
        .iter()
        .map(|e| {
            let file = e["file"].as_str().expect("an entry names a file");
            let expect = match &e["expect"] {
                serde_json::Value::String(s) => s.clone(),
                r => format!(
                    "{} {} {} {}",
                    r["x"].as_u64().expect("x"),
                    r["y"].as_u64().expect("y"),
                    r["w"].as_u64().expect("w"),
                    r["h"].as_u64().expect("h")
                ),
            };
            let tags: Vec<&str> = e["tags"]
                .as_array()
                .expect("an entry has tags")
                .iter()
                .map(|t| t.as_str().expect("a tag is a string"))
                .collect();
            format!("{file} | {expect} | {}", tags.join(","))
        })
        .collect();

    assert_eq!(
        reported, from_json,
        "AC-4: `load()` must still return every entry in manifest order with \
         `path`, `expect` and `tags` unchanged in shape and value. The twelve \
         MC-018 and MC-033 tests in this file are the other half of this \
         control: if any of them goes red alongside this, the split was not \
         added additively. Each row is `file | expect | tags`"
    );
}

// --- MC-037: the held-out set is filled, and its shape is pinned ------------

/// MC-037 AC-1. Twenty is [`MIN_ENTRIES`]'s own reasoning applied to the half
/// of the corpus that now carries the accuracy claim: below this the held-out
/// set cannot support a "nine in ten" number with any resolution, because one
/// miss moves it by five percent.
///
/// **Lowered to 19 by MC-053, on the user's ruling of 2026-09-24** (MC-053's
/// Open question 6, "the recommended default. `MIN_HELD_OUT_MARKED` drops to
/// 19, with the ruling recorded in `corpus.md` the same way
/// `MIN_UNSEEN_SITES` was lowered"). MC-052 moved two marked held-out entries
/// to `tuning` and MC-053 three more, every one of them for cause - each
/// reproduced a bug the story fixing it was designed while looking at - which
/// leaves 19. At 19 one miss is 5.3 %. The held-out set has shrunk by five
/// marked entries since MC-037, and new screenshots, not a lower floor, are
/// how it grows back; `docs/wiki/corpus.md` records the ruling.
///
/// **Lowered to 16 by MC-056, on the user's ruling of 2026-09-29** (MC-056's
/// Open question 1, "yes"). MC-051 took `EPIC-07`'s one held-out score and
/// read its three failing entries per file; being read, they move to `tuning`,
/// which leaves 16. At 16 one miss is 6.25 %. As before, new screenshots, not
/// a lower floor, are how the set grows back.
///
/// **Raised back to 20 by MC-062, on the user's rulings of 2026-09-30.** The
/// held-out set is now only MC-062's fresh draw: 25 whole-screen screenshots
/// picked blind from the user's own folders and marked by the user
/// (`marks-mc062`), all 25 of them art boxes. The 23 spent entries moved to
/// `tuning` ("move them to practice"). 20 is MC-037's own number again; at 25
/// present one miss is 4 %.
///
/// **Set to 15 by MC-068, on the user's ruling of 2026-10-01** ("15", the
/// recommended option). MC-062's draw has given its one scored run (MC-063)
/// and all 25 of it are `tuning` now, so the held-out set is only MC-068's
/// second fresh draw: 15 whole-screen screenshots the Lead PO drew blind,
/// marked by the user (`marks-mc068`), all 15 art boxes. The user cut the
/// draw from 20 to 15 the same day. The floor matches the set exactly, so
/// losing any one of the 15 is caught. At 15 one miss is 6.7 %.
///
/// **Lowered to 11 by MC-072, on the user's ruling of 2026-10-02** (MC-072's
/// Open question 1, put in these words: *"The test set drops from 15
/// screenshots to 11. Lower its minimum to 11?"*, recommended *"yes, 11,
/// matching what is left, as 15 matched the set when it was drawn"*; the
/// user's answer to all four questions was *"all recommended"*). MC-071 took
/// the second fresh draw's one scored run and read its four failing entries
/// per file (`n02`, `n05`, `n06`, `n13`); being read, they move to `tuning`,
/// which leaves 11. The 11 are spent and cannot give another real score, so
/// the floor only guards against losing one by accident; the next fresh draw
/// sets its own minimum. At 11 one miss is 9.1 %.
///
/// **Set to 10 by MC-077, on the user's ruling of 2026-10-03** ("10 boxes, 4
/// sites"). MC-071 took the second fresh draw's one scored run, so all 15 of
/// it are `tuning` now (MC-077 AC-4), and the held-out set is only MC-077's
/// third fresh draw: 10 whole-screen screenshots the Lead PO drew blind (the
/// user asked for 10, "and pick some from manhwa_panels"), marked by the user
/// (`marks-mc077`, "Yes, freeze them"), all 10 art boxes. The floor matches
/// the set exactly, so losing any one of the 10 is caught. At 10 one miss is
/// 10 points of accuracy.
const MIN_HELD_OUT_MARKED: usize = 10;

/// MC-037 AC-2. The detector has two jobs - crop the croppable and decline the
/// rest - and a held-out set of rectangles alone measures one of them.
///
/// **Lowered from 4 to 0 by MC-062, on the user's ruling of 2026-09-30**:
/// *"Drop that requirement"*. A blind draw of 25 whole-screen screenshots of
/// real use found no screenshot to leave alone, and `tuning`'s 14 flag
/// entries still carry the "decline the rest" job. `docs/wiki/corpus.md`
/// records the ruling.
///
/// A floor of 0 asserts nothing, so its test pins the count **exactly** at
/// this value rather than at least it: the ruling was made about a held-out
/// set with no flag entry, and a flag entry arriving in it is a change the
/// story that adds it must rule on, raising this constant as it does.
///
/// **Kept at 0 by MC-068, on the user's rulings of 2026-10-01** (its Open
/// question 2: the other three floors stand). MC-068's blind draw of 15 is
/// all art boxes, with no screenshot to leave alone, so the held-out set
/// still carries no flag entry, and `tuning`'s 14 still carry that job.
///
/// **Kept at 0 by MC-077, on the user's rulings of 2026-10-03** (its Open
/// question 2: `MIN_HELD_OUT_FLAGS` = 0 stands). MC-077's blind draw of 10 is
/// all art boxes - no flag entry was drawn - so the held-out set still carries
/// none, and `tuning`'s 14 still carry the "decline the rest" job.
const MIN_HELD_OUT_FLAGS: usize = 0;

/// MC-037 AC-4. Twenty held-out entries from a single reader would measure
/// almost nothing: a reader's furniture is pixel-identical across every
/// screenshot from it, so a rule can score well by learning that geometry
/// rather than learning what a page is.
///
/// **Kept at 4 by MC-062, on the user's rulings of 2026-09-30.** The fresh
/// draw spans 5 readers - `toongod` 14, `rolia-scans` 4, `xbato` 3,
/// `demonicrevolution` 2, `w-network` 2 - as the user marked them.
///
/// **Kept at 4 by MC-068, on the user's rulings of 2026-10-01** (its Open
/// question 2: the other three floors stand). The second fresh draw spans
/// exactly 4 readers - `toongod` 7, `rolia-scans` 4, `xbato` 3,
/// `demonicrevolution` 1 - as the user marked them, so losing a reader is
/// caught.
///
/// **Lowered to 3 by MC-072, on the user's ruling of 2026-10-02** (MC-072's
/// Open question 2, put in these words: *"After the move, the test set has
/// screenshots from three sites, not four. Lower that minimum to 3?"*,
/// recommended *"yes, 3. The set is spent, so the minimum only guards against
/// losing a site by accident from here on. The next fresh draw sets it
/// again."*; the user's answer was *"all recommended"*). `Screenshot
/// (2507).png` (`n13`) was the held-out set's only `demonicrevolution` entry
/// and is one of the four MC-071 read per file, so held-out spans `toongod` 4,
/// `rolia-scans` 4 and `xbato` 3.
///
/// **Set to 4 by MC-077, on the user's ruling of 2026-10-03** ("10 boxes, 4
/// sites"). The third fresh draw spans exactly 4 readers - `toongod` 5,
/// `rolia-scans` 2, `xbato` 2, `w-network` 1 - as the user marked them
/// (`marks-mc077`), so losing every screenshot from one reader is caught.
const MIN_HELD_OUT_SITES: usize = 4;

/// MC-037 AC-4. The readers appearing *only* in the held-out set are the only
/// entries that can answer "does this generalise to an unseen reader?".
///
/// **Lowered from 2 to 1 by MC-053, on the user's ruling of 2026-09-24**
/// (MC-053's Open question 2): *"unseen reader" means a reader with **no
/// `tuning` entries**.* MC-053 moved two of `xbato`'s three entries to
/// `tuning` and designs its fix on them, so `xbato` is tuned-on and the
/// unseen-reader claim rests on `manhwaclan` alone (2 entries). The test below
/// now counts a reader as unseen only when it is absent from the pre-EPIC-07
/// readers **and** from every `tuning` entry's `site:` tag; under that
/// definition the floor of 2 is not met, and the ruling lowers it to 1.
/// `docs/wiki/corpus.md` records it.
///
/// **Lowered from 1 to 0 by MC-062, on the user's ruling of 2026-09-30**:
/// *"Drop it; say so in the result"*. The eight folders the corpus is drawn
/// from hold only readers already in `tuning`, so the fresh held-out set has
/// no unseen reader, and a score on it speaks for crops on known readers only.
/// `docs/wiki/corpus.md` and the next score say so.
///
/// As with [`MIN_HELD_OUT_FLAGS`], a floor of 0 asserts nothing, so its test
/// pins the count **exactly** at this value: an unseen reader arriving in
/// held-out is a change the story that adds it must rule on.
///
/// **Kept at 0 by MC-068, on the user's rulings of 2026-10-01** (its Open
/// question 2: the other three floors stand). MC-068 drew from the same eight
/// folders, and its four readers all have `tuning` entries, so a score on the
/// second fresh set also speaks for crops on known readers only.
///
/// **Kept at 0 by MC-077, on the user's rulings of 2026-10-03** (its Open
/// question 2: `MIN_UNSEEN_SITES` = 0 stands). MC-077 drew from the same
/// eight folders, and its four readers all have `tuning` entries, so a score
/// on the third fresh set also speaks for crops on known readers only.
const MIN_UNSEEN_SITES: usize = 0;

/// The prefix marking a tag as naming the reader an entry was captured from.
const SITE_TAG_PREFIX: &str = "site:";

/// The line that introduces the pre-EPIC-07 reader table on the corpus page.
const PRE_EPIC_07_READERS_MARKER: &str = "<!-- pre-epic-07-readers -->";

/// The readers the pre-EPIC-07 corpus was captured from, **read out of
/// `docs/wiki/corpus.md`** rather than repeated here.
///
/// This is a person's ruling given as a *set*, not a derivation and not a
/// per-file attribution: the twenty-eight pre-EPIC-07 entries carry no `site:`
/// tag and are not required to. Attributing a reader to a screenshot captured
/// a year earlier is what manufactures guesses, and the corpus page rules that
/// a guessed tag is worse than an absent one.
///
/// Nothing here can check the ruling. What it can do is make "unseen reader" a
/// claim about a tracked file, so that widening the set is a visible edit
/// rather than a decision nobody wrote down.
fn documented_pre_epic_07_readers() -> BTreeSet<String> {
    documented_vocabulary(PRE_EPIC_07_READERS_MARKER, "AC-4")
}

/// Every entry whose split is `held-out`, in manifest order.
fn held_out(entries: &[CorpusEntry]) -> Vec<&CorpusEntry> {
    entries
        .iter()
        .filter(|e| e.split == Split::HeldOut)
        .collect()
}

/// The `site:` tags an entry carries, prefix stripped. Returns every one, not
/// the first: "exactly one" is an assertion below, and a helper that silently
/// took the first would make it unfalsifiable.
fn site_tags(entry: &CorpusEntry) -> Vec<&str> {
    entry
        .tags
        .iter()
        .filter_map(|t| t.strip_prefix(SITE_TAG_PREFIX))
        .collect()
}

/// The distinct readers named across the held-out set.
fn held_out_sites(entries: &[CorpusEntry]) -> BTreeSet<String> {
    held_out(entries)
        .iter()
        .flat_map(|e| site_tags(e))
        .map(str::to_owned)
        .collect()
}

// --- AC-1: the held-out set carries enough marked entries to mean something -

#[test]
fn the_held_out_set_carries_at_least_ten_marked_entries() {
    let entries = corpus::load();
    let held = held_out(&entries);

    // The vacuity guard. "At least twenty" over an empty set fails loudly, but
    // every *other* held-out count in this file would be quietly measuring
    // nothing, and that is the state MC-036 deliberately left behind.
    assert!(
        !held.is_empty(),
        "AC-1: no entry in the manifest is `held-out`, so every held-out count \
         in this file is measuring the empty set. MC-036 left it empty \
         deliberately and MC-037 is the story that fills it"
    );

    let marked = held
        .iter()
        .filter(|e| matches!(e.expect, Expect::Rect(_)))
        .count();

    assert!(
        marked >= MIN_HELD_OUT_MARKED,
        "AC-1: the held-out set carries {marked} marked entries and needs at \
         least {MIN_HELD_OUT_MARKED}. This is MIN_ENTRIES' reasoning applied \
         to the half of the corpus that now carries the accuracy claim: at \
         twenty entries one miss is five percent (5.3 % at the nineteen the \
         user ruled on 2026-09-24, MC-053; 6.25 % at the sixteen the user \
         ruled on 2026-09-29, MC-056; back to twenty on the user's rulings of \
         2026-09-30, MC-062, whose fresh draw carries 25; fifteen on the user's \
         ruling of 2026-10-01, MC-068, whose second fresh draw carries exactly \
         15, 6.7 % a miss; eleven on the user's ruling of 2026-10-02, MC-072, \
         \"all recommended\", which moved the four MC-071 read per file to \
         `tuning` and left 11, spent; ten on the user's ruling of 2026-10-03, \
         MC-077, \"10 boxes, 4 sites\", whose third fresh draw carries exactly \
         10, 10 points a miss), and below that the reported \
         number is decided by which screenshot happened to land here. The \
         whole-corpus floor is a different question and stays green while this \
         one fails - a floor that only fires when the *total* drops is not \
         measuring the split"
    );
}

// --- AC-2: and enough entries that should be left alone ---------------------

/// MC-037 AC-2, as MC-062 re-sets it on the user's ruling of 2026-09-30 (see
/// [`MIN_HELD_OUT_FLAGS`]). Until MC-062 this was "at least four"; a floor of
/// 0 cannot fail, so the count is pinned exactly at the ruled value.
#[test]
fn the_held_out_set_carries_exactly_the_zero_flag_entries_the_user_ruled_on() {
    let entries = corpus::load();
    let held = held_out(&entries);

    assert!(
        !held.is_empty(),
        "AC-2: no entry in the manifest is `held-out`, so this count is over \
         the empty set"
    );

    // Counted over `Expect::Flag` specifically, never over "held-out entries"
    // as a whole.
    let flags: Vec<String> = held
        .iter()
        .filter(|e| matches!(e.expect, Expect::Flag))
        .map(|e| e.name())
        .collect();

    assert_eq!(
        flags.len(),
        MIN_HELD_OUT_FLAGS,
        "MC-062 AC-5: the held-out set carries {} entries expecting `flag`, \
         and the user's ruling of 2026-09-30 (\"Drop that requirement\") was \
         made about a fresh draw with none: MIN_HELD_OUT_FLAGS is \
         {MIN_HELD_OUT_FLAGS}, pinned exactly because a floor of 0 cannot \
         fail; MC-068 kept it at 0 on the user's rulings of 2026-10-01, its \
         15 being art boxes too, and MC-077 on the user's rulings of \
         2026-10-03, its 10 being art boxes too. `tuning`'s flag entries carry \
         the \"decline the rest\" job. A flag entry in held-out is either a \
         spent entry MC-062, MC-068 or MC-077 moved to `tuning`, or a new one \
         the story adding it \
         must rule on and raise this constant for. Held-out flag entries: \
         {flags:?}. Held-out \
         entries in total: {}",
        flags.len(),
        held.len()
    );
}

// --- AC-3: MC-036's contamination guard, read in the other direction --------

#[test]
fn no_held_out_entry_is_a_pre_epic_07_file() {
    let entries = corpus::load();
    let held = held_out(&entries);

    // Without this the test passes on an empty held-out set - exactly the
    // state MC-036 left behind, and exactly why the same assertion was vacuous
    // there. It is the reason this criterion is worth restating from the other
    // side rather than trusting MC-036's version.
    assert!(
        !held.is_empty(),
        "AC-3: the held-out set is empty, so 'no held-out entry is \
         contaminated' is true and means nothing. This assertion only has \
         content once MC-037's screenshots are in"
    );

    let contaminated: Vec<String> = held
        .iter()
        .filter(|e| PRE_EPIC_07_ENTRIES.contains(&e.name().as_str()))
        .map(|e| e.name())
        .collect();

    assert_eq!(
        contaminated,
        Vec::<String>::new(),
        "AC-3: no held-out entry may be a file that existed at commit a453aaa. \
         Six v1 investigations fitted thresholds against those twenty-eight \
         and their per-file tables have been read; holding one out produces a \
         number that looks rigorous and is not. This is not hypothetical - on \
         MC-037's first collection attempt `2025-03-03 11_24_19.png` arrived \
         as a held-out entry, byte-identical to the tuning entry of the same \
         name, and was dropped rather than swapped per the story's \
         `## Out of scope`. A duplicate file name is the shape this mistake \
         takes in practice"
    );
}

// --- AC-4: every held-out entry names its reader ----------------------------

#[test]
fn every_held_out_entry_carries_exactly_one_site_tag() {
    let entries = corpus::load();
    let held = held_out(&entries);

    assert!(
        !held.is_empty(),
        "AC-4: the held-out set is empty, so 'every held-out entry names a \
         reader' is vacuously true"
    );

    // Both directions, and by name. Zero tags means the entry cannot count
    // toward reader coverage at all; two means one entry inflates the distinct
    // reader count on its own. Neither is visible from a total.
    let wrong: Vec<String> = held
        .iter()
        .filter_map(|e| {
            let sites = site_tags(e);
            (sites.len() != 1).then(|| format!("{}: {} site tags {sites:?}", e.name(), sites.len()))
        })
        .collect();

    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "AC-4: every held-out entry carries exactly one `{SITE_TAG_PREFIX}` tag \
         naming the reader it came from. The pre-EPIC-07 twenty-eight carry \
         none and are not required to - see `docs/wiki/corpus.md`. That the \
         tag is also in the documented vocabulary is a different test in this \
         file"
    );
}

#[test]
fn the_held_out_set_spans_at_least_four_readers() {
    let entries = corpus::load();
    let sites = held_out_sites(&entries);

    assert!(
        sites.len() >= MIN_HELD_OUT_SITES,
        "AC-4: the held-out set spans {} distinct readers and needs at least \
         {MIN_HELD_OUT_SITES}. A reader's furniture - navigation bar, header, \
         page margins - is pixel-identical across every screenshot from it, so \
         twenty entries from one reader would let a rule score well by \
         learning that geometry instead of learning what a page is. Four since \
         MC-077, the user's ruling of 2026-10-03 (\"10 boxes, 4 sites\"): the \
         third fresh draw spans `toongod`, `rolia-scans`, `xbato` and \
         `w-network`, exactly four (three under MC-072, 2026-10-02). Readers \
         found: {sites:?}",
        sites.len()
    );
}

/// MC-037 AC-4, as MC-053 re-defines "unseen" on the user's ruling of
/// 2026-09-24 (see [`MIN_UNSEEN_SITES`]): a held-out reader is unseen when it
/// is absent from the pre-EPIC-07 readers **and** no `tuning` entry carries
/// its `site:` tag. Until MC-053 only the first half was checked, so a reader
/// whose entries had moved to `tuning` - `xbato`, since MC-053 - still counted.
///
/// **MC-062, the user's ruling of 2026-09-30** ("Drop it; say so in the
/// result"): the floor is 0, and the count is pinned exactly at it, since a
/// floor of 0 cannot fail. The fresh draw's five readers all have `tuning`
/// entries. Until MC-062 this was "at least one".
#[test]
fn no_held_out_reader_is_absent_from_both_the_pre_epic_07_set_and_tuning() {
    let entries = corpus::load();
    let sites = held_out_sites(&entries);
    let known = documented_pre_epic_07_readers();
    let tuned_on: BTreeSet<&str> = entries
        .iter()
        .filter(|e| e.split == Split::Tuning)
        .flat_map(site_tags)
        .collect();

    let unseen: Vec<&String> = sites
        .iter()
        .filter(|s| !known.contains(*s) && !tuned_on.contains(s.as_str()))
        .collect();

    assert_eq!(
        unseen.len(),
        MIN_UNSEEN_SITES,
        "MC-062 AC-5: {} held-out readers are absent from the pre-EPIC-07 set \
         and from every `tuning` entry (an unseen reader has no `tuning` \
         entries, the user's ruling of 2026-09-24), and the user's ruling of \
         2026-09-30 (\"Drop it; say so in the result\") was made about a fresh \
         draw with none: MIN_UNSEEN_SITES is {MIN_UNSEEN_SITES}, pinned exactly \
         because a floor of 0 cannot fail. So a score on held-out speaks for \
         crops on known readers only, and `docs/wiki/corpus.md` says so; MC-068 \
         kept it at 0 on the user's rulings of 2026-10-01, its four readers all \
         being in `tuning`, and MC-077 on the user's rulings of 2026-10-03, for \
         the same reason. An unseen reader in held-out is either a spent entry \
         MC-062, MC-068 or MC-077 moved to `tuning` (`manhwaclan`, MC-062's), or a new \
         one the story adding it must rule on and raise this constant for. \
         Readers with `tuning` entries: \
         {tuned_on:?}. Held-out readers: {sites:?}. Pre-EPIC-07 readers: \
         {known:?}. Unseen: {unseen:?}",
        unseen.len()
    );
}

// --- MC-042: every entry names its reader, and the marks are pinned ---------

/// MC-042 AC-1. The reader the user attributed to each of the twenty-eight
/// `tuning` entries on **2026-09-21**, read back from the Corpus Reader
/// Labeller's own store, in manifest order.
///
/// Thirty since MC-052: the user moved two held-out `toongod` entries to
/// `tuning` on 2026-09-24, and they keep the `site:` tag they already had.
/// Thirty-three since MC-053, which moved two `xbato` entries and one
/// `kunmanga` entry the same way, by the user's ruling of the same day.
///
/// **This is the specification, not a derivation.** The labeller is a
/// throwaway tool outside this repository - the same arrangement as MC-018's
/// marking annotator - so MC-042's `## Context` table, under "The per-file
/// attribution, which is the part that cannot be derived", is the only record
/// of that session anywhere. No agent may revise a row of it, and nothing in
/// this repository can re-derive one: the counts do not determine the mapping,
/// and `docs/wiki/corpus.md` rules a guessed tag worse than an absent one.
/// [`PRE_EPIC_07_ENTRIES`] is the precedent - a list hard-coded here because
/// the ruling behind it lives in a person's head and a document, not in code.
///
/// Two readers in this list, `kunmanga` and `demonicrevolution`, were counted
/// *unseen* until 2026-09-21; `xbato` appears nowhere in it although the
/// recalled `PRE_EPIC_07_SITES` set named it. Both directions of that
/// falsification are AC-3's subject on the corpus page.
///
/// Fifty-nine since MC-062, which moved all 23 spent held-out entries the
/// same way, by the user's answer of 2026-09-30. Sixty-three since MC-064,
/// which moved the four fresh entries MC-063 read per file. Eighty-four since
/// MC-068, which moved the other 21 of MC-062's fresh draw once it was spent.
/// Ninety-eight since MC-069, which adds the 14 screenshots the app called
/// `Ambiguous` in the user's run, with the readers the user named on MC-069's
/// marking page. A hundred and two since MC-072, which moved the four of
/// MC-068's fresh draw that MC-071 read per file. A hundred and thirteen
/// since MC-077, which moved the other 11 of that draw once MC-071 had spent
/// it.
const READER_BY_FILE: [(&str, &str); 113] = [
    ("2025-02-27 22_46_15.png", "toongod"),
    ("2025-03-03 11_06_04.png", "toongod"),
    ("2025-03-03 11_24_19.png", "toongod"),
    ("2025-05-12 22_55_40.png", "w-network"),
    ("2025-05-12 22_58_53.png", "w-network"),
    ("2025-08-05 00_11_13.webp", "rolia-scans"),
    ("2025-08-05 00_11_27.webp", "rolia-scans"),
    ("2025-10-14 23_29_06.png", "demonicrevolution"),
    ("2025-10-14 23_30_20.png", "demonicrevolution"),
    ("2025-10-20 15_37_25.png", "toongod"),
    ("2026-01-05 13_33_41.png", "toongod"),
    ("2026-01-05 13_45_59.png", "demonicrevolution"),
    ("2026-01-05 13_49_39.png", "demonicrevolution"),
    ("Screenshot (67).png", "kunmanga"),
    ("Screenshot (70).jpg", "kunmanga"),
    ("Screenshot (75).png", "toongod"),
    ("Screenshot (93).jpg", "toongod"),
    ("Screenshot (103).jpg", "toongod"),
    ("Screenshot (1661).png", "toongod"),
    ("Screenshot (2582).jpg", "toongod"),
    ("Screenshot (2630).jpg", "toongod"),
    ("Screenshot (2698).jpg", "toongod"),
    ("Screenshot (2708).jpg", "toongod"),
    ("Screenshot (2744).jpg", "w-network"),
    ("Screenshot (3187).png", "w-network"),
    ("Screenshot (3455).png", "toongod"),
    ("Screenshot (3465).png", "w-network"),
    ("Screenshot (3538).png", "toongod"),
    // MC-052: moved from `held-out` to `tuning` by the user's ruling of
    // 2026-09-24. Their `site:` tag is the one they carried in held-out.
    ("2025-03-06 01_22_45.png", "toongod"),
    ("2025-03-07 00_58_06.png", "toongod"),
    // MC-053: moved from `held-out` to `tuning` by the user's ruling of
    // 2026-09-24, because the column-axis fix is designed while looking at
    // them. Their `site:` tag is the one they carried in held-out.
    ("2025-07-17 14_41_58.png", "xbato"),
    ("2025-07-17 14_55_10.png", "xbato"),
    ("Screenshot (73).png", "kunmanga"),
    // MC-056: moved from `held-out` to `tuning` by the user's request of
    // 2026-09-29, because MC-051 read all three per file. Their `site:` tag is
    // the one they carried in held-out.
    ("2025-07-17 14_20_23.png", "xbato"),
    ("2025-08-03 11_27_49.png", "rolia-scans"),
    ("Screenshot (68).png", "kunmanga"),
    // MC-062: the 23 spent held-out entries, moved to `tuning` by the user's
    // answer of 2026-09-30 ("move them to practice"), in manifest order. Their
    // `site:` tag is the one they carried in held-out, read out of the
    // manifest, not retyped.
    ("2024-09-09 23_30_01.png", "toongod"),
    ("2024-09-09 23_55_27.png", "toongod"),
    ("2025-03-03 11_00_13.png", "toongod"),
    ("2025-03-04 11_09_29.png", "toongod"),
    ("2025-03-07 00_41_10.png", "toongod"),
    ("2025-03-07 01_10_37.png", "toongod"),
    ("2025-05-12 10_37_44.png", "w-network"),
    ("2025-05-12 20_48_42.png", "w-network"),
    ("2025-05-13 00_21_02.png", "w-network"),
    ("2025-08-03 11_13_19.png", "rolia-scans"),
    ("2025-08-03 20_54_19.png", "rolia-scans"),
    ("2025-08-04 23_24_37.png", "rolia-scans"),
    ("2025-08-07 15_07_56.png", "rolia-scans"),
    ("2025-08-07 15_47_10.png", "rolia-scans"),
    ("2025-08-07 15_57_50.png", "rolia-scans"),
    ("2025-09-29 14_33_15.png", "w-network"),
    ("2025-10-05 01_34_27.png", "rolia-scans"),
    ("Screenshot (56).png", "manhwaclan"),
    ("Screenshot (59).png", "manhwaclan"),
    ("Screenshot (1720).png", "demonicrevolution"),
    ("Screenshot (3605).png", "toongod"),
    ("Screenshot (3606).png", "toongod"),
    ("Screenshot (3625).png", "w-network"),
    // MC-064: the four fresh entries MC-063 read per file, moved to `tuning`
    // by the user's ruling of 2026-09-30 ("Write up and file"), in manifest
    // order. Their `site:` tag is the one they carried in held-out, read out
    // of the manifest, not retyped.
    ("2025-03-06 12_48_06.png", "toongod"),
    ("2025-03-16 22_47_44.png", "toongod"),
    ("2025-08-07 01_13_55.png", "rolia-scans"),
    ("2025-12-08 17_22_50.png", "toongod"),
    // MC-068: the other 21 of MC-062's fresh draw, `held-out` until MC-068 and
    // spent by MC-063's one scored run, moved to `tuning` (MC-068 AC-4), in
    // manifest order. Their `site:` tag is the one they carried in held-out,
    // read out of the manifest at `345a9eb`, not retyped.
    ("2025-03-04 14_28_54.png", "toongod"),
    ("2025-03-06 02_01_06.png", "toongod"),
    ("2025-03-07 16_07_21.png", "toongod"),
    ("2025-03-18 12_37_27.png", "toongod"),
    ("2025-03-23 23_56_16.png", "toongod"),
    ("2025-03-24 22_44_31.png", "toongod"),
    ("2025-03-25 22_06_29.png", "toongod"),
    ("2025-07-17 23_45_48.png", "xbato"),
    ("2025-07-21 08_26_37.png", "xbato"),
    ("2025-07-21 17_47_22.png", "xbato"),
    ("2025-08-04 17_10_16.png", "rolia-scans"),
    ("2025-08-07 11_20_12.png", "rolia-scans"),
    ("2025-08-07 14_33_43.png", "rolia-scans"),
    ("2025-10-23 11_31_40.png", "w-network"),
    ("2025-11-12 17_43_44.png", "w-network"),
    ("2025-12-09 00_00_17.png", "toongod"),
    ("Screenshot (9).png", "toongod"),
    ("Screenshot (2368).png", "toongod"),
    ("Screenshot (2461).png", "demonicrevolution"),
    ("Screenshot (2486).png", "demonicrevolution"),
    ("Screenshot (2669).png", "toongod"),
    // MC-069: the 14 screenshots the user's run answered `Ambiguous` on,
    // added to `tuning` (MC-069 AC-1), with the readers the user named when
    // marking them (MC-069 `## Notes`, frozen 2026-10-01).
    ("Screenshot (14).png", "toongod"),
    ("Screenshot (19).png", "toongod"),
    ("Screenshot (20).png", "toongod"),
    ("Screenshot (23).png", "toongod"),
    ("Screenshot (42).png", "toongod"),
    ("Screenshot (48).png", "demonicrevolution"),
    ("Screenshot (49).png", "demonicrevolution"),
    ("Screenshot (50).png", "demonicrevolution"),
    ("Screenshot (51).png", "demonicrevolution"),
    ("Screenshot (52).png", "demonicrevolution"),
    ("Screenshot (53).png", "demonicrevolution"),
    ("Screenshot (57).png", "demonicrevolution"),
    ("Screenshot (58).png", "demonicrevolution"),
    ("Screenshot (2705).png", "toongod"),
    // MC-072: the four of MC-068's fresh draw that MC-071 read per file
    // (`n02`, `n06`, `n05`, `n13`), moved to `tuning` by the user's answer of
    // 2026-10-02 ("all recommended"), in manifest order. Their `site:` tag is
    // the one they carried in held-out, read out of the manifest at `0f9c579`,
    // and equal to MC068_DRAW's rows.
    ("2025-03-07 00_05_58.png", "toongod"),
    ("2025-03-13 12_01_01.png", "toongod"),
    ("2025-11-01 12_34_31.png", "toongod"),
    ("Screenshot (2507).png", "demonicrevolution"),
    // MC-077: the other 11 of MC-068's fresh draw, spent by MC-071's one
    // scored run and moved to `tuning` (MC-077 AC-4), in manifest order.
    // Their `site:` tag is the one they carried in held-out, read out of the
    // manifest at `4272252` (awk), and equal to MC068_DRAW's rows.
    ("2025-03-18 14_07_22.png", "toongod"),
    ("2025-04-16 17_00_49.png", "toongod"),
    ("2025-07-17 16_13_54.png", "xbato"),
    ("2025-07-18 00_21_31.png", "xbato"),
    ("2025-07-18 08_30_58.png", "xbato"),
    ("2025-08-04 08_22_11.png", "rolia-scans"),
    ("2025-08-05 08_44_44.png", "rolia-scans"),
    ("2025-08-05 11_01_27.png", "rolia-scans"),
    ("2025-08-07 00_24_27.png", "rolia-scans"),
    ("2025-11-20 23_55_15.png", "toongod"),
    ("Screenshot (1460).png", "toongod"),
];

/// MC-042 AC-1, the same labelling summarised: `(reader, tuning, of which
/// marked, of which flag, held-out)`, from the counts table in `## Context`.
///
/// **Strictly implied by [`READER_BY_FILE`] and kept anyway**, for two jobs
/// the per-file pin cannot do:
///
/// 1. *It is the message a person can read.* The per-file assertion fails with
///    twenty-eight rows; this one fails with seven, and says which reader
///    gained or lost entries. When both go red together, this is the one that
///    explains what happened.
/// 2. *It is the cross-check on `READER_BY_FILE` itself.* A hand-edit to one
///    row of the mapping - the realistic way a settled label gets quietly
///    revised - moves two counts here and contradicts the user's own summary.
///    A constant that is its own only source of truth checks nothing about
///    itself, and these two were transcribed from different tables.
///
/// The marked/flag columns carry the second job's weight. `toongod`'s fifteen
/// tuning entries are **eleven marked and four flag** and `w-network`'s five
/// are **two and three**; every accuracy number in EPIC-07 is measured over
/// *marked* entries, so a mapping edit that moved one flag entry between
/// readers while keeping both totals would still be caught here. MC-042's
/// `## Model guidance` names that split as the story's one trap.
///
/// The held-out columns are a fifth column this file has always been able to
/// check and never did: they must stay exactly as they are, which is AC-1's
/// "the 31 held-out entries keeping the tags they already have".
///
/// **MC-052, 2026-09-24.** The user moved two `toongod` entries from
/// `held-out` to `tuning` (`2025-03-06 01_22_45.png`, `2025-03-07
/// 00_58_06.png`), both marked. `toongod`'s row goes from `15, 11, 4, 10` to
/// `17, 13, 4, 8`; no other row moves, and no reader is re-attributed.
///
/// **MC-053, 2026-09-24.** The user moved three marked entries from
/// `held-out` to `tuning`: `2025-07-17 14_41_58.png` and `2025-07-17
/// 14_55_10.png` (`xbato`) and `Screenshot (73).png` (`kunmanga`). `xbato`'s
/// row goes from `0, 0, 0, 3` to `2, 2, 0, 1` and `kunmanga`'s from
/// `2, 2, 0, 2` to `3, 3, 0, 1`; no other row moves, and no reader is
/// re-attributed.
///
/// **MC-056, 2026-09-29.** The user moved the three held-out entries MC-051
/// read per file, all marked: `2025-07-17 14_20_23.png` (`xbato`), `2025-08-03
/// 11_27_49.png` (`rolia-scans`) and `Screenshot (68).png` (`kunmanga`).
/// `xbato` goes from `2, 2, 0, 1` to `3, 3, 0, 0`, `kunmanga` from `3, 3, 0, 1`
/// to `4, 4, 0, 0` and `rolia-scans` from `2, 2, 0, 8` to `3, 3, 0, 7`; no
/// other row moves, and no reader is re-attributed. `xbato` and `kunmanga`
/// leave the held-out set entirely.
///
/// **MC-062, 2026-09-30.** The user moved all 23 spent held-out entries to
/// `tuning` ("move them to practice") - 16 marked and 7 flag - and the
/// held-out set became MC-062's fresh draw of 25, all marked, with the
/// readers the user named on the marking page. Per reader, the 23 move
/// `toongod` 5 marked + 3 flag, `w-network` 2 + 3, `demonicrevolution` 1 + 0,
/// `rolia-scans` 6 + 1, `manhwaclan` 2 + 0; the 25 arrive as `toongod` 14,
/// `rolia-scans` 4, `xbato` 3, `demonicrevolution` 2, `w-network` 2.
/// `kunmanga` does not move. `manhwaclan` leaves the held-out set entirely and
/// gains its first `tuning` entries. No reader is re-attributed.
///
/// **MC-064, 2026-09-30.** The user moved the four fresh entries MC-063 read
/// per file ("Write up and file"), all marked: `2025-03-06 12_48_06.png`,
/// `2025-03-16 22_47_44.png` and `2025-12-08 17_22_50.png` (`toongod`) and
/// `2025-08-07 01_13_55.png` (`rolia-scans`). `toongod` goes from
/// `25, 18, 7, 14` to `28, 21, 7, 11` and `rolia-scans` from `10, 9, 1, 4` to
/// `11, 10, 1, 3`; no other row moves, and no reader is re-attributed.
///
/// **MC-068, 2026-10-01.** The other 21 of MC-062's fresh draw, all marked,
/// move to `tuning` once MC-063's run has spent them (`toongod` 11, `xbato` 3,
/// `rolia-scans` 3, `w-network` 2, `demonicrevolution` 2), and MC-068's
/// second fresh draw of 15, all marked, arrives in `held-out` with the readers
/// the user named on the marking page (`toongod` 7, `rolia-scans` 4, `xbato`
/// 3, `demonicrevolution` 1). `toongod` goes from `28, 21, 7, 11` to
/// `39, 32, 7, 7`, `w-network` from `10, 4, 6, 2` to `12, 6, 6, 0`,
/// `demonicrevolution` from `5, 5, 0, 2` to `7, 7, 0, 1`, `rolia-scans` from
/// `11, 10, 1, 3` to `14, 13, 1, 4` and `xbato` from `3, 3, 0, 3` to
/// `6, 6, 0, 3`. `kunmanga` and `manhwaclan` do not move; `w-network` leaves
/// the held-out set. No reader is re-attributed. Sums: `tuning` 84 (70 marked,
/// 14 flag), `held-out` 15.
///
/// **MC-069, 2026-10-01.** The 14 screenshots the app called `Ambiguous`
/// arrive in `tuning`, all marked: `toongod` 6 (`(14)`, `(19)`, `(20)`,
/// `(23)`, `(42)`, `(2705)`) and `demonicrevolution` 8 (`(48)` to `(58)`).
/// `toongod` goes from `39, 32, 7, 7` to `45, 38, 7, 7` and
/// `demonicrevolution` from `7, 7, 0, 1` to `15, 15, 0, 1`; no other row moves,
/// and no reader is re-attributed. Sums: `tuning` 98 (84 marked, 14 flag),
/// `held-out` 15.
///
/// **MC-072, 2026-10-02.** The user moved the four fresh entries MC-071 read
/// per file ("all recommended"), all marked: `2025-03-07 00_05_58.png`,
/// `2025-03-13 12_01_01.png` and `2025-11-01 12_34_31.png` (`toongod`) and
/// `Screenshot (2507).png` (`demonicrevolution`). `toongod` goes from
/// `45, 38, 7, 7` to `48, 41, 7, 4` and `demonicrevolution` from
/// `15, 15, 0, 1` to `16, 16, 0, 0`; no other row moves, and no reader is
/// re-attributed. `demonicrevolution` leaves the held-out set. Sums: `tuning`
/// 102 (88 marked, 14 flag), `held-out` 11.
///
/// **MC-077, 2026-10-03.** The other 11 of MC-068's fresh draw, all marked,
/// move to `tuning` once MC-071's run has spent them (`toongod` 4,
/// `rolia-scans` 4, `xbato` 3), and MC-077's third fresh draw of 10, all
/// marked, arrives in `held-out` with the readers the user named on the
/// marking page (`toongod` 5, `rolia-scans` 2, `xbato` 2, `w-network` 1).
/// `toongod` goes from `48, 41, 7, 4` to `52, 45, 7, 5`, `rolia-scans` from
/// `14, 13, 1, 4` to `18, 17, 1, 2`, `xbato` from `6, 6, 0, 3` to
/// `9, 9, 0, 2` and `w-network` from `12, 6, 6, 0` to `12, 6, 6, 1`.
/// `demonicrevolution`, `kunmanga` and `manhwaclan` do not move. No reader is
/// re-attributed. Sums: `tuning` 113 (99 marked, 14 flag), `held-out` 10.
const READER_LABELS: [(&str, usize, usize, usize, usize); 7] = [
    ("toongod", 52, 45, 7, 5),
    ("w-network", 12, 6, 6, 1),
    ("demonicrevolution", 16, 16, 0, 0),
    ("rolia-scans", 18, 17, 1, 2),
    ("kunmanga", 4, 4, 0, 0),
    ("xbato", 9, 9, 0, 2),
    ("manhwaclan", 2, 2, 0, 0),
];

/// MC-042 AC-2. The `expect` rectangle of every **marked `tuning`** entry, in
/// manifest order, as it stood at commit `f62dfb3` - read out of
/// `fixtures/corpus/manifest.json` itself rather than transcribed from a
/// document, because a rectangle retyped by eye is the defect this constant
/// exists to catch.
///
/// Nothing guarded these before MC-042, and **every number in EPIC-07 rests on
/// them**: MC-019's 20 of 21 on the column axis, MC-026's 8 of 21, MC-028's 4,
/// MC-031's 5, MC-034's 8, MC-035's re-score and MC-038's 8. MC-042 adds a
/// `site:` tag to all twenty-eight tuning entries by hand, and a fat-fingered
/// digit in an adjacent line is the realistic risk that carries - a `git diff`
/// nobody re-reads is not a control. [`PRE_EPIC_07_ENTRIES`] is the precedent.
///
/// This is a pin, not a judgement: no code can say a rectangle is *correct*,
/// and MC-042's `## Out of scope` forbids re-marking anything. A deliberate
/// re-mark changes this constant in the story that decides it, which is
/// exactly the visible edit the pin is here to force.
///
/// **MC-049 is the first such re-mark.** Seven marks contained 1-3 columns of
/// flat page background on one side (`corpus.md`'s marking rule excludes
/// them), and with `margin_px` going to 0 those columns would read as clips of
/// nothing but page. The user approved correcting them on 2026-09-23, on the
/// MC-027 precedent for `2026-01-05 13_45_59.png`; `Screenshot (2630).jpg`'s
/// dark column 1591 was ruled page background after a look at the image.
/// `docs/wiki/corpus.md`, "Corrected marks", records each old and new rect.
///
/// **MC-060 is the first on the row axis.** `Screenshot (67).png`'s top edge
/// held two rows of page white and the site header's closing line (rows
/// 290..292); the user ruled on 2026-09-30, after 8x zooms, that the art
/// starts on row 293. The mark goes from `1012,290 523x1098` to
/// `1012,293 523x1095`, bottom unchanged; "Corrected marks" records it.
///
/// Written as `(file, x, y, w, h)` and rebuilt into a [`Rect`] where it is
/// read: a literal `Rect { .. }` per row is twenty-one rectangles rustfmt
/// explodes over nine lines each, and a pin nobody can scan in one screen is a
/// pin nobody re-reads.
///
/// **MC-062 moves no mark.** It adds the 16 marked entries among the 23 spent
/// held-out entries, as they stood, and pins its own 25 fresh marks in
/// [`MC062_DRAW`] (`FRESH_HELD_OUT` until MC-068).
///
/// **MC-064 moves no mark.** It adds the four fresh entries MC-063 read per
/// file, with the marks [`MC062_DRAW`] freezes for them.
///
/// **MC-068 moves no mark.** It adds the other 21 of MC-062's fresh draw, with
/// the marks [`MC062_DRAW`] freezes for them, and pins its own 15 fresh marks
/// in [`MC068_DRAW`] (`FRESH_HELD_OUT` until MC-077).
///
/// **MC-069 moves no mark.** It adds its 14 reported screenshots with the
/// marks [`MC069_REPORTED`] freezes for them (`Screenshot (42).png` as
/// amended there).
///
/// **MC-072 moves no mark.** It adds the four fresh entries MC-071 read per
/// file, with the marks [`MC068_DRAW`] freezes for them.
///
/// **MC-073 moves one mark**, by the user's ruling of 2026-10-02, in their
/// words *"Box starts at 1007"*: `n05` (`2025-11-01 12_34_31.png`) from
/// `1006,167 532x1233` to `1007,167 531x1233`, as in [`MC068_DRAW`].
///
/// **MC-076 moves one mark**, by the user's ruling of 2026-10-02 (*"Fix all
/// three"*, MC-076 `## Notes`): `2025-08-05 00_11_13.webp`'s top row, 114, is
/// the browser toolbar's one-pixel bottom line, the same tone across page and
/// margin (zoom of rows 100..130, columns 930..1130, x8), and the art starts on
/// row 115. The mark goes from `958,114 631x1216` to `958,115 631x1215`,
/// bottom unchanged at 1330.
///
/// **MC-077 moves no mark.** It adds the other 11 of MC-068's fresh draw,
/// with the marks [`MC068_DRAW`] freezes for them, and pins its own 10 fresh
/// marks in [`FRESH_HELD_OUT`].
const MARKED_TUNING_RECTS: [(&str, u32, u32, u32, u32); 99] = [
    // MC-076: the top moves down 1 row by the user's ruling of 2026-10-02:
    // row 114 is browser bar. Bottom unchanged at 1330. Was `958,114 631x1216`.
    ("2025-08-05 00_11_13.webp", 958, 115, 631, 1215),
    ("2025-08-05 00_11_27.webp", 1008, 118, 528, 1225),
    ("2025-10-14 23_29_06.png", 1003, 188, 540, 1138),
    ("2025-10-14 23_30_20.png", 1040, 171, 473, 1208),
    ("2025-10-20 15_37_25.png", 1008, 228, 528, 1074),
    ("2026-01-05 13_33_41.png", 1074, 233, 396, 1060),
    ("2026-01-05 13_45_59.png", 1040, 204, 466, 1167),
    ("2026-01-05 13_49_39.png", 1044, 286, 459, 1110),
    // MC-060: the top moves down 3 rows by the user's ruling of 2026-09-30,
    // made on 8x zooms of rows 285..296: rows 290..292 are two rows of page
    // white and the site header's grey line, and the art starts on row 293.
    // Bottom unchanged at 1388. Was `1012,290 523x1098`; `corpus.md`,
    // "Corrected marks".
    ("Screenshot (67).png", 1012, 293, 523, 1095),
    ("Screenshot (70).jpg", 1016, 298, 519, 1012),
    ("Screenshot (75).png", 1077, 171, 393, 1208),
    ("Screenshot (93).jpg", 1143, 212, 246, 1167),
    ("Screenshot (103).jpg", 1077, 302, 389, 844),
    ("Screenshot (1661).png", 1077, 192, 393, 1085),
    ("Screenshot (2582).jpg", 1008, 216, 524, 1008),
    ("Screenshot (2630).jpg", 958, 224, 633, 889),
    ("Screenshot (2698).jpg", 954, 343, 631, 742),
    ("Screenshot (2708).jpg", 1074, 220, 392, 1102),
    ("Screenshot (2744).jpg", 950, 134, 639, 1164),
    ("Screenshot (3187).png", 987, 142, 573, 1237),
    ("Screenshot (3538).png", 975, 171, 590, 1159),
    // MC-052: the two entries the user moved from `held-out` on 2026-09-24,
    // with the marks they carried there, unchanged.
    ("2025-03-06 01_22_45.png", 648, 118, 520, 1244),
    ("2025-03-07 00_58_06.png", 671, 362, 474, 928),
    // MC-053: the three entries the user moved from `held-out` on 2026-09-24,
    // with the marks **widened** on all six side edges by the user's ruling of
    // 2026-09-25 (MC-053's Open question 3), made on renders of each edge:
    // the art runs to the page background on every one. Rows unchanged. Were
    // `931,273 681x1096`, `931,171 668x1224` and `1007,293 530x1086`.
    ("2025-07-17 14_41_58.png", 928, 273, 690, 1096),
    ("2025-07-17 14_55_10.png", 928, 171, 690, 1224),
    ("Screenshot (73).png", 1003, 293, 540, 1086),
    // MC-056: the three entries the user moved from `held-out` on 2026-09-29,
    // read per file by MC-051. `2025-08-03 11_27_49.png` and `Screenshot
    // (68).png` keep the marks they carried there. `2025-07-17 14_20_23.png`'s
    // left edge is **widened** from 967 to 962 by the user's ruling of
    // 2026-09-29 (MC-056's Open question 3): the page's grey ends at column
    // 961 and the art's own black background starts at 962. Was
    // `967,171 609x1211`.
    ("2025-07-17 14_20_23.png", 962, 171, 614, 1211),
    ("2025-08-03 11_27_49.png", 1013, 118, 524, 1267),
    ("Screenshot (68).png", 961, 392, 625, 944),
    // MC-062: the 16 marked entries among the 23 spent held-out entries the
    // user moved to `tuning` on 2026-09-30, with the marks they carried there,
    // unchanged, read out of the manifest at `7c36b5d`. The other seven are
    // flag entries and carry no rectangle.
    ("2025-03-04 11_09_29.png", 1013, 309, 514, 971),
    ("2025-03-07 00_41_10.png", 681, 118, 454, 1271),
    ("2025-03-07 01_10_37.png", 612, 148, 596, 1096),
    ("2025-08-03 11_13_19.png", 954, 118, 635, 1182),
    ("2025-08-03 20_54_19.png", 1010, 115, 527, 1280),
    ("2025-08-04 23_24_37.png", 1010, 118, 527, 1208),
    ("2025-08-07 15_07_56.png", 1076, 158, 395, 1171),
    ("2025-08-07 15_47_10.png", 974, 319, 599, 1030),
    ("2025-08-07 15_57_50.png", 994, 355, 556, 813),
    ("2025-09-29 14_33_15.png", 984, 171, 576, 1229),
    ("Screenshot (56).png", 1040, 171, 472, 1218),
    ("Screenshot (59).png", 1043, 224, 463, 1161),
    ("Screenshot (1720).png", 1013, 135, 520, 1247),
    ("Screenshot (3605).png", 1007, 174, 530, 1113),
    ("Screenshot (3606).png", 1010, 161, 527, 1175),
    ("Screenshot (3625).png", 948, 240, 645, 1126),
    // MC-064: the four fresh entries MC-063 read per file, moved to `tuning`
    // on 2026-09-30, with the marks they carried in held-out, unchanged, read
    // out of the manifest at `43e8e61` (they equal MC062_DRAW's rows).
    ("2025-03-06 12_48_06.png", 651, 115, 517, 1284),
    ("2025-03-16 22_47_44.png", 635, 115, 533, 1277),
    ("2025-08-07 01_13_55.png", 1022, 115, 500, 1285),
    ("2025-12-08 17_22_50.png", 1006, 167, 533, 1233),
    // MC-068: the other 21 of MC-062's fresh draw, spent by MC-063's run and
    // moved to `tuning`, with the marks they carried in held-out, unchanged,
    // read out of the manifest at `345a9eb` (they equal MC062_DRAW's rows), in
    // manifest order.
    ("2025-03-04 14_28_54.png", 1005, 115, 533, 1285),
    ("2025-03-06 02_01_06.png", 680, 115, 460, 1259),
    ("2025-03-07 16_07_21.png", 651, 115, 517, 1277),
    ("2025-03-18 12_37_27.png", 710, 115, 400, 1277),
    ("2025-03-23 23_56_16.png", 1172, 115, 200, 1281),
    ("2025-03-24 22_44_31.png", 1005, 115, 533, 1285),
    ("2025-03-25 22_06_29.png", 1072, 115, 400, 1285),
    ("2025-07-17 23_45_48.png", 1014, 167, 517, 1233),
    ("2025-07-21 08_26_37.png", 1044, 115, 459, 1285),
    ("2025-07-21 17_47_22.png", 997, 115, 552, 1285),
    ("2025-08-04 17_10_16.png", 1006, 115, 533, 1285),
    ("2025-08-07 11_20_12.png", 991, 115, 563, 1285),
    ("2025-08-07 14_33_43.png", 973, 115, 600, 1284),
    ("2025-10-23 11_31_40.png", 948, 167, 648, 1233),
    ("2025-11-12 17_43_44.png", 984, 167, 576, 1233),
    ("2025-12-09 00_00_17.png", 1073, 167, 400, 1233),
    ("Screenshot (9).png", 1073, 167, 400, 1225),
    ("Screenshot (2368).png", 1139, 133, 267, 1259),
    ("Screenshot (2461).png", 1010, 133, 534, 1259),
    ("Screenshot (2486).png", 1078, 133, 400, 1259),
    ("Screenshot (2669).png", 1073, 133, 400, 1259),
    // MC-069: the 14 screenshots the app called `Ambiguous`, with the marks
    // the user froze on 2026-10-01 (MC069_REPORTED's rows), `(42)` as amended
    // from `1073,166 400x1226`.
    ("Screenshot (14).png", 1139, 167, 267, 1225),
    ("Screenshot (19).png", 1139, 167, 267, 1225),
    ("Screenshot (20).png", 1139, 167, 267, 1225),
    ("Screenshot (23).png", 1139, 167, 267, 1225),
    ("Screenshot (42).png", 1073, 167, 400, 1225),
    ("Screenshot (48).png", 1039, 168, 467, 1224),
    ("Screenshot (49).png", 1039, 167, 467, 1225),
    ("Screenshot (50).png", 1039, 167, 467, 1225),
    ("Screenshot (51).png", 993, 167, 560, 1225),
    ("Screenshot (52).png", 1039, 167, 467, 1225),
    ("Screenshot (53).png", 1003, 167, 540, 1225),
    ("Screenshot (57).png", 1033, 167, 479, 1225),
    ("Screenshot (58).png", 1033, 167, 480, 1225),
    ("Screenshot (2705).png", 1073, 133, 399, 1259),
    // MC-070 `## Amendments` (the user, 2026-10-01, "Mark ends at 1471"):
    // `(2705)` was `1073,133 400x1259`; column 1472 is outside the mark.
    // MC-072: the four of MC-068's fresh draw that MC-071 read per file, moved
    // to `tuning` on 2026-10-02, with the marks they carried in held-out,
    // unchanged, read out of the manifest at `0f9c579` (they equal
    // MC068_DRAW's rows `n02`, `n06`, `n05`, `n13`), in manifest order.
    ("2025-03-07 00_05_58.png", 643, 115, 533, 1284),
    ("2025-03-13 12_01_01.png", 698, 115, 400, 1277),
    // MC-073 AC-1: `n05` re-marked by the user's ruling of 2026-10-02, in
    // their words "Box starts at 1007". It was `1006,167 532x1233`.
    ("2025-11-01 12_34_31.png", 1007, 167, 531, 1233),
    ("Screenshot (2507).png", 977, 133, 600, 1259),
    // MC-077: the other 11 of MC-068's fresh draw, spent by MC-071's one
    // scored run and moved to `tuning` on 2026-10-03, with the marks they
    // carried in held-out, unchanged, read out of the manifest at `4272252`
    // (awk; they equal MC068_DRAW's rows), in manifest order.
    ("2025-03-18 14_07_22.png", 643, 115, 533, 1277),
    ("2025-04-16 17_00_49.png", 1073, 115, 400, 1285),
    ("2025-07-17 16_13_54.png", 962, 167, 621, 1233),
    ("2025-07-18 00_21_31.png", 997, 167, 552, 1233),
    ("2025-07-18 08_30_58.png", 1043, 167, 460, 1233),
    ("2025-08-04 08_22_11.png", 1006, 115, 533, 1285),
    ("2025-08-05 08_44_44.png", 1139, 115, 267, 1285),
    ("2025-08-05 11_01_27.png", 1139, 115, 267, 1285),
    ("2025-08-07 00_24_27.png", 872, 115, 800, 1284),
    ("2025-11-20 23_55_15.png", 1073, 167, 400, 1233),
    ("Screenshot (1460).png", 953, 133, 639, 1259),
];

/// The entries in `split` carrying `site:<reader>`, in manifest order.
///
/// Built on [`site_tags`] rather than [`CorpusEntry::has_tag`] so that an
/// entry carrying two `site:` tags is counted once per tag it really has -
/// "exactly one" is a separate assertion below, and a helper that quietly
/// looked at the first tag would make this count agree with it no matter what
/// the manifest said.
fn by_reader<'a>(entries: &'a [CorpusEntry], reader: &str, split: Split) -> Vec<&'a CorpusEntry> {
    entries
        .iter()
        .filter(|e| e.split == split && site_tags(e).contains(&reader))
        .collect()
}

// --- AC-1: every entry names its reader -------------------------------------

#[test]
fn every_entry_in_the_manifest_names_exactly_one_reader() {
    let entries = corpus::load();

    assert!(
        !entries.is_empty(),
        "AC-1: the loader returned no entries at all, so 'every entry names a \
         reader' is vacuously true"
    );

    // Accumulated and asserted once, with the split alongside the name: the
    // interesting failure is twenty-eight files long and a per-file table is
    // the only form of it anyone can act on.
    let wrong: Vec<String> = entries
        .iter()
        .filter_map(|e| {
            let sites = site_tags(e);
            (sites.len() != 1).then(|| {
                format!(
                    "{} | {} | {} site tags {sites:?}",
                    e.name(),
                    e.split.as_str(),
                    sites.len()
                )
            })
        })
        .collect();

    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "AC-1: every one of the {} entries in the manifest carries exactly one \
         `{SITE_TAG_PREFIX}` tag naming the reader it came from. Until \
         2026-09-21 the twenty-eight `tuning` entries carried none and \
         `docs/wiki/corpus.md` said they were not required to, because \
         attributing a screenshot captured a year earlier manufactures \
         guesses; the user has now labelled all twenty-eight with the Corpus \
         Reader Labeller, so the exemption is spent. Exactly one and not 'at \
         least one': an entry with two inflates every distinct-reader count in \
         this file on its own, and site-disjoint evaluation is the only thing \
         that can tell a furniture rule which generalises from one that has \
         memorised `toongod`. Each row is `file | split | site tags`",
        entries.len()
    );
}

#[test]
fn every_tuning_entry_carries_the_reader_the_labeller_recorded_for_it() {
    let entries = corpus::load();
    let mut wrong: Vec<String> = Vec::new();

    for (name, reader) in READER_BY_FILE {
        match entries.iter().find(|e| e.name() == name) {
            // A pinned name absent from the manifest would make this test
            // check twenty-seven attributions while claiming twenty-eight.
            None => wrong.push(format!(
                "{name} | {reader} | no entry of that name is in the manifest"
            )),
            Some(entry) => {
                let sites = site_tags(entry);
                if sites.len() != 1 || sites[0] != reader {
                    wrong.push(format!("{name} | {reader} | {sites:?}"));
                }
                // MC-056: a pinned row names a `tuning` entry. Until MC-056
                // this was implied by the other direction below and never
                // checked, so a row naming a `held-out` entry passed.
                if entry.split != Split::Tuning {
                    wrong.push(format!(
                        "{name} | {reader} | split `{}`, not `tuning`",
                        entry.split.as_str()
                    ));
                }
            }
        }
    }

    // The other direction. A `tuning` entry the mapping does not name is one
    // whose reader nobody recorded, and no count in this file would notice.
    for entry in entries.iter().filter(|e| e.split == Split::Tuning) {
        let name = entry.name();
        if !READER_BY_FILE.iter().any(|(n, _)| *n == name) {
            wrong.push(format!(
                "{name} | - | a `tuning` entry the 2026-09-21 labelling does \
                 not attribute"
            ));
        }
    }

    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "AC-1: each of the {} `tuning` entries must carry exactly the `site:` \
         tag the user recorded for it on 2026-09-21 - the per-file table in \
         MC-042's `## Context`, which READER_BY_FILE reads out. This is the \
         criterion's actual contract: the counts do not determine the mapping, \
         so an attribution that gave `toongod`'s eleven marked slots to a \
         different eleven marked entries would satisfy every total in this \
         file and still be eleven wrong answers. Nothing here or anywhere else \
         in the repository can re-derive a row - the labeller is an Artifact \
         outside the tree - so a disagreement means the manifest is wrong, \
         never the constant. Each row is `file | expected | actual`",
        READER_BY_FILE.len()
    );
}

#[test]
fn the_reader_attribution_matches_the_counts_the_user_recorded() {
    let entries = corpus::load();

    // The readable half of AC-1, and the cross-check on READER_BY_FILE - see
    // that constant and this one's doc comment for why a test strictly implied
    // by another is worth its run time. Rendered as rows and compared in one
    // shot, so a failure prints the whole table - which reader, which column,
    // measured against recorded - rather than stopping at the first
    // disagreement.
    let mut measured: Vec<String> = Vec::new();
    let mut recorded: Vec<String> = Vec::new();
    for (reader, tuning_total, marked, flagged, held) in READER_LABELS {
        let tuning = by_reader(&entries, reader, Split::Tuning);
        measured.push(format!(
            "{reader} | tuning {} | marked {} | flag {} | held-out {}",
            tuning.len(),
            tuning
                .iter()
                .filter(|e| matches!(e.expect, Expect::Rect(_)))
                .count(),
            tuning.iter().filter(|e| e.expect == Expect::Flag).count(),
            by_reader(&entries, reader, Split::HeldOut).len(),
        ));
        recorded.push(format!(
            "{reader} | tuning {tuning_total} | marked {marked} | flag \
             {flagged} | held-out {held}"
        ));
    }

    // A reader the labelling never names is a row neither list would otherwise
    // carry, and it would leave the seven above still agreeing.
    let known: BTreeSet<&str> = READER_LABELS.iter().map(|(r, ..)| *r).collect();
    let strays: BTreeSet<String> = entries
        .iter()
        .flat_map(site_tags)
        .filter(|s| !known.contains(s))
        .map(str::to_owned)
        .collect();
    for stray in &strays {
        measured.push(format!(
            "{stray} | a reader slug the 2026-09-21 labelling does not name"
        ));
    }

    assert_eq!(
        measured, recorded,
        "AC-1: the reader attribution in the manifest must be exactly the one \
         the user recorded on 2026-09-21, which MC-042's `## Context` carries \
         and READER_LABELS reads out. The marked and flag columns are the \
         point: `toongod`'s fifteen tuning entries are eleven marked and four \
         flag, and every accuracy number in EPIC-07 is measured over marked \
         entries, so a manifest that got the fifteen right and the eleven \
         wrong would make AC-5's 'eleven of twenty-one' false while looking \
         labelled. `xbato` had zero tuning entries until MC-053 moved two of \
         its three there on 2026-09-24. Each row is \
         `reader | tuning | marked | flag | held-out`"
    );
}

// --- AC-2: the marked tuning rectangles cannot move -------------------------

#[test]
fn every_marked_tuning_entry_still_carries_the_rectangle_it_was_marked_with() {
    let entries = corpus::load();
    let mut drifted: Vec<String> = Vec::new();

    for (name, x, y, w, h) in MARKED_TUNING_RECTS {
        let pinned = Rect { x, y, w, h };
        // A pinned name that is no longer in the manifest is the classic way a
        // guard stops guarding without anybody noticing: it would check
        // twenty rectangles while claiming twenty-one.
        match entries.iter().find(|e| e.name() == name) {
            None => drifted.push(format!(
                "{name}: pinned {pinned:?}, but no entry of that name is in the \
                 manifest at all"
            )),
            // MC-056: a pinned row names a `tuning` entry, checked, not implied.
            Some(entry) if entry.split != Split::Tuning => drifted.push(format!(
                "{name}: pinned {pinned:?}, but the entry is `{}`, not `tuning`",
                entry.split.as_str()
            )),
            Some(entry) => match entry.expect {
                Expect::Rect(actual) if actual == pinned => {}
                Expect::Rect(actual) => {
                    drifted.push(format!("{name}: pinned {pinned:?}, manifest {actual:?}"))
                }
                Expect::Flag => {
                    drifted.push(format!("{name}: pinned {pinned:?}, manifest \"flag\""))
                }
            },
        }
    }

    // The other direction. Without it a marked tuning entry added or renamed
    // into the corpus would be unpinned and this test would not say so.
    for entry in entries
        .iter()
        .filter(|e| e.split == Split::Tuning && matches!(e.expect, Expect::Rect(_)))
    {
        let name = entry.name();
        if !MARKED_TUNING_RECTS.iter().any(|(n, ..)| *n == name) {
            drifted.push(format!(
                "{name}: a marked `tuning` entry that MARKED_TUNING_RECTS does \
                 not pin"
            ));
        }
    }

    assert_eq!(
        drifted,
        Vec::<String>::new(),
        "AC-2: all {} marked `tuning` rectangles must be byte-for-byte the \
         ones EPIC-07 measured against. MC-019's 20 of 21, MC-026's 8 of 21, \
         MC-028's 4, MC-031's 5, MC-034's 8, MC-035's re-score and MC-038's 8 \
         are all scored against these exact rectangles, and MC-042 edits every \
         one of the twenty-eight tuning entries by hand to add a `site:` tag. \
         A digit changed in an adjacent line would silently move a number in \
         seven prior documents. Re-marking is out of scope for MC-042: if a \
         mark is genuinely wrong, the story that rules on it changes this \
         constant and says so. Each row is `file | pinned | manifest`",
        MARKED_TUNING_RECTS.len()
    );
}

// --- MC-062: a fresh held-out set, drawn blind from the user's folders ------

/// What the user answered for "the gap between panels" on the marking page:
/// light, dark, or can't tell. It decides the one gutter tag, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gap {
    /// "light": the entry carries `white-gutter`.
    White,
    /// "dark": the entry carries `black-gutter`.
    Black,
    /// "can't tell": the entry carries neither.
    None,
}

impl Gap {
    /// The gutter tag this answer puts on the entry, if any.
    fn tag(self) -> Option<&'static str> {
        match self {
            Gap::White => Some("white-gutter"),
            Gap::Black => Some("black-gutter"),
            Gap::None => None,
        }
    }
}

/// One drawn screenshot - one of MC-062's 25 ([`MC062_DRAW`]), MC-068's 15
/// ([`MC068_DRAW`]) or MC-077's 10 ([`FRESH_HELD_OUT`]) - as the draw
/// recorded it and the user marked it.
struct Fresh {
    /// The file's own name, which is also its name in the corpus (AC-1: no
    /// collision, so none is renamed).
    file: &'static str,
    /// Its size in bytes when drawn, from the drawing story's `## Context`
    /// table.
    bytes: u64,
    /// The user's box as `(x, y, w, h)`, the manifest's convention.
    rect: (u32, u32, u32, u32),
    /// The reader the user picked on the marking page.
    site: &'static str,
    /// The user's gap answer.
    gap: Gap,
}

/// MC-062 AC-1 and AC-2. The 25 screenshots drawn blind on 2026-09-30, in the
/// draw's order (`f01`..`f25`): the bytes from `## Context`'s table, and the
/// user's marks, readers and gap answers from `## Notes`'s read-back of
/// `marks-mc062`, **frozen by the user the same day**. Every page was dark,
/// every one is a PNG, and every one is an art box - no flag.
///
/// **This is the oracle, not a derivation.** Nothing in the repository can
/// re-derive a row - the marking page is an Artifact outside the tree - and
/// MC-062's criteria forbid adjusting one. A change to a row is an Amendment.
///
/// The user ticked "diagonal" on `f07`, `f08`, `f14` and `f20`, and ruled
/// "No label" (2026-09-30): each box runs the full visible height, so the
/// diagonal gap lies inside it and moves no edge. None carries
/// `diagonal-gutter`, and [`DIAGONAL_GUTTER_ENTRIES`] is unchanged.
///
/// **Spent since MC-068.** This was `FRESH_HELD_OUT` until MC-068. MC-063
/// took the draw's one scored run; MC-064 moved the four it read per file to
/// `tuning`, and MC-068 the other 21 (its AC-4). All 25 are `tuning` now, and
/// keep these frozen marks, tags and files: the held-out set was
/// [`MC068_DRAW`], MC-068's second draw, until MC-077 drew [`FRESH_HELD_OUT`].
const MC062_DRAW: [Fresh; 25] = [
    Fresh {
        file: "Screenshot (2461).png",
        bytes: 1_249_203,
        rect: (1010, 133, 534, 1259),
        site: "demonicrevolution",
        gap: Gap::None,
    }, // f01
    Fresh {
        file: "Screenshot (2669).png",
        bytes: 1_097_496,
        rect: (1073, 133, 400, 1259),
        site: "toongod",
        gap: Gap::White,
    }, // f02
    Fresh {
        file: "2025-10-23 11_31_40.png",
        bytes: 1_645_269,
        rect: (948, 167, 648, 1233),
        site: "w-network",
        gap: Gap::White,
    }, // f03
    Fresh {
        file: "2025-12-09 00_00_17.png",
        bytes: 1_298_999,
        rect: (1073, 167, 400, 1233),
        site: "toongod",
        gap: Gap::None,
    }, // f04
    Fresh {
        file: "Screenshot (2486).png",
        bytes: 1_086_938,
        rect: (1078, 133, 400, 1259),
        site: "demonicrevolution",
        gap: Gap::None,
    }, // f05
    Fresh {
        file: "Screenshot (2368).png",
        bytes: 943_754,
        rect: (1139, 133, 267, 1259),
        site: "toongod",
        gap: Gap::None,
    }, // f06
    Fresh {
        file: "2025-11-12 17_43_44.png",
        bytes: 1_370_816,
        rect: (984, 167, 576, 1233),
        site: "w-network",
        gap: Gap::White,
    }, // f07
    Fresh {
        file: "Screenshot (9).png",
        bytes: 1_234_737,
        rect: (1073, 167, 400, 1225),
        site: "toongod",
        gap: Gap::White,
    }, // f08
    Fresh {
        file: "2025-12-08 17_22_50.png",
        bytes: 1_311_011,
        rect: (1006, 167, 533, 1233),
        site: "toongod",
        gap: Gap::White,
    }, // f09
    Fresh {
        file: "2025-03-04 14_28_54.png",
        bytes: 753_269,
        rect: (1005, 115, 533, 1285),
        site: "toongod",
        gap: Gap::White,
    }, // f10
    Fresh {
        file: "2025-03-06 02_01_06.png",
        bytes: 1_651_935,
        rect: (680, 115, 460, 1259),
        site: "toongod",
        gap: Gap::None,
    }, // f11
    Fresh {
        file: "2025-03-24 22_44_31.png",
        bytes: 1_698_975,
        rect: (1005, 115, 533, 1285),
        site: "toongod",
        gap: Gap::None,
    }, // f12
    Fresh {
        file: "2025-03-16 22_47_44.png",
        bytes: 2_048_469,
        rect: (635, 115, 533, 1277),
        site: "toongod",
        gap: Gap::None,
    }, // f13
    Fresh {
        file: "2025-03-07 16_07_21.png",
        bytes: 1_896_981,
        rect: (651, 115, 517, 1277),
        site: "toongod",
        gap: Gap::White,
    }, // f14
    Fresh {
        file: "2025-03-25 22_06_29.png",
        bytes: 1_347_328,
        rect: (1072, 115, 400, 1285),
        site: "toongod",
        gap: Gap::None,
    }, // f15
    Fresh {
        file: "2025-03-18 12_37_27.png",
        bytes: 1_070_656,
        rect: (710, 115, 400, 1277),
        site: "toongod",
        gap: Gap::None,
    }, // f16
    Fresh {
        file: "2025-03-23 23_56_16.png",
        bytes: 649_246,
        rect: (1172, 115, 200, 1281),
        site: "toongod",
        gap: Gap::Black,
    }, // f17
    Fresh {
        file: "2025-03-06 12_48_06.png",
        bytes: 1_505_174,
        rect: (651, 115, 517, 1284),
        site: "toongod",
        gap: Gap::White,
    }, // f18
    Fresh {
        file: "2025-08-07 11_20_12.png",
        bytes: 1_382_818,
        rect: (991, 115, 563, 1285),
        site: "rolia-scans",
        gap: Gap::None,
    }, // f19
    Fresh {
        file: "2025-08-07 01_13_55.png",
        bytes: 693_068,
        rect: (1022, 115, 500, 1285),
        site: "rolia-scans",
        gap: Gap::White,
    }, // f20
    Fresh {
        file: "2025-08-07 14_33_43.png",
        bytes: 1_309_775,
        rect: (973, 115, 600, 1284),
        site: "rolia-scans",
        gap: Gap::None,
    }, // f21
    Fresh {
        file: "2025-08-04 17_10_16.png",
        bytes: 1_106_488,
        rect: (1006, 115, 533, 1285),
        site: "rolia-scans",
        gap: Gap::None,
    }, // f22
    Fresh {
        file: "2025-07-21 17_47_22.png",
        bytes: 1_956_636,
        rect: (997, 115, 552, 1285),
        site: "xbato",
        gap: Gap::None,
    }, // f23
    Fresh {
        file: "2025-07-21 08_26_37.png",
        bytes: 1_201_442,
        rect: (1044, 115, 459, 1285),
        site: "xbato",
        gap: Gap::White,
    }, // f24
    Fresh {
        file: "2025-07-17 23_45_48.png",
        bytes: 722_052,
        rect: (1014, 167, 517, 1233),
        site: "xbato",
        gap: Gap::White,
    }, // f25
];

/// MC-068 AC-1 and AC-2. The 15 screenshots the Lead PO drew blind on
/// 2026-10-01, in the draw's order (`n01`..`n15`): the bytes from MC-068's
/// `## Context` table, and the user's marks, readers and gap answers from
/// `## Notes`'s read-back of `marks-mc068`, **frozen by the user the same day**
/// ("Yes, freeze them"). Every page was dark, every one is a PNG, and every
/// one is an art box - no flag. The page's "light" gap is `Gap::White`, its
/// "dark" `Gap::Black`, and its "can't tell" `Gap::None`.
///
/// **This is the oracle, not a derivation.** Nothing in the repository can
/// re-derive a row - the marking page is an Artifact outside the tree - and
/// MC-068's criteria forbid adjusting one. A change to a row is an Amendment.
///
/// The SHA-256 of each copy is checked against `## Context` when SCAFFOLD
/// copies it (no hashing crate is in this workspace); the byte count and the
/// header dimensions are what a test here can hold.
///
/// **MC-073 re-marks one row**, by the user's ruling of 2026-10-02, in their
/// words *"Box starts at 1007"*: `n05` (`2025-11-01 12_34_31.png`, `tuning`
/// since MC-072) from `1006,167 532x1233` to `1007,167 531x1233`. Column 1006
/// is the dark seam beside the art, as on `Screenshot (3605).png` and
/// `2025-10-20 15_37_25.png` (MC-073 `## Context`). MC-071's held-out score
/// was taken against the old mark and is not re-computed.
///
/// **Spent since MC-077.** This was `FRESH_HELD_OUT` until MC-077. MC-071
/// took the draw's one scored run; MC-072 moved the four it read per file to
/// `tuning`, and MC-077 the other 11 (its AC-4). All 15 are `tuning` now, and
/// keep these frozen marks, tags and files: the held-out set is
/// [`FRESH_HELD_OUT`], MC-077's third draw.
const MC068_DRAW: [Fresh; 15] = [
    Fresh {
        file: "Screenshot (1460).png",
        bytes: 1_159_792,
        rect: (953, 133, 639, 1259),
        site: "toongod",
        gap: Gap::White,
    }, // n01
    Fresh {
        file: "2025-03-07 00_05_58.png",
        bytes: 1_185_535,
        rect: (643, 115, 533, 1284),
        site: "toongod",
        gap: Gap::White,
    }, // n02
    Fresh {
        file: "2025-08-05 08_44_44.png",
        bytes: 795_381,
        rect: (1139, 115, 267, 1285),
        site: "rolia-scans",
        gap: Gap::White,
    }, // n03
    Fresh {
        file: "2025-07-18 08_30_58.png",
        bytes: 966_808,
        rect: (1043, 167, 460, 1233),
        site: "xbato",
        gap: Gap::White,
    }, // n04
    // MC-073 AC-1: `n05` re-marked by the user's ruling of 2026-10-02, in
    // their words "Box starts at 1007" - column 1006 is the dark seam beside
    // the art, as on `Screenshot (3605).png` and `2025-10-20 15_37_25.png`.
    // It was `(1006, 167, 532, 1233)`; the right edge and rows are unchanged.
    Fresh {
        file: "2025-11-01 12_34_31.png",
        bytes: 1_913_304,
        rect: (1007, 167, 531, 1233),
        site: "toongod",
        gap: Gap::White,
    }, // n05
    Fresh {
        file: "2025-03-13 12_01_01.png",
        bytes: 1_085_486,
        rect: (698, 115, 400, 1277),
        site: "toongod",
        gap: Gap::White,
    }, // n06
    Fresh {
        file: "2025-08-04 08_22_11.png",
        bytes: 1_485_470,
        rect: (1006, 115, 533, 1285),
        site: "rolia-scans",
        gap: Gap::White,
    }, // n07
    Fresh {
        file: "2025-07-17 16_13_54.png",
        bytes: 1_489_446,
        rect: (962, 167, 621, 1233),
        site: "xbato",
        gap: Gap::White,
    }, // n08
    Fresh {
        file: "2025-11-20 23_55_15.png",
        bytes: 1_453_856,
        rect: (1073, 167, 400, 1233),
        site: "toongod",
        gap: Gap::None,
    }, // n09
    Fresh {
        file: "2025-03-18 14_07_22.png",
        bytes: 1_871_471,
        rect: (643, 115, 533, 1277),
        site: "toongod",
        gap: Gap::None,
    }, // n10
    Fresh {
        file: "2025-08-05 11_01_27.png",
        bytes: 868_117,
        rect: (1139, 115, 267, 1285),
        site: "rolia-scans",
        gap: Gap::None,
    }, // n11
    Fresh {
        file: "2025-07-18 00_21_31.png",
        bytes: 1_363_458,
        rect: (997, 167, 552, 1233),
        site: "xbato",
        gap: Gap::None,
    }, // n12
    Fresh {
        file: "Screenshot (2507).png",
        bytes: 1_230_166,
        rect: (977, 133, 600, 1259),
        site: "demonicrevolution",
        gap: Gap::None,
    }, // n13
    Fresh {
        file: "2025-04-16 17_00_49.png",
        bytes: 823_637,
        rect: (1073, 115, 400, 1285),
        site: "toongod",
        gap: Gap::White,
    }, // n14
    Fresh {
        file: "2025-08-07 00_24_27.png",
        bytes: 919_752,
        rect: (872, 115, 800, 1284),
        site: "rolia-scans",
        gap: Gap::Black,
    }, // n15
];

/// MC-077 AC-1 and AC-2. The 10 screenshots the Lead PO drew blind on
/// 2026-10-03, in the draw's order (`n01`..`n10`): the bytes from MC-077's
/// `## Context` table, and the user's marks, readers and gap answers from
/// `## Notes`'s read-back of `marks-mc077`, **frozen by the user the same day**
/// ("Yes, freeze them"). Every page was dark, every one is a PNG, and every
/// one is an art box - no flag. The page's "light" gap is `Gap::White` and its
/// "can't tell" (read back as "none") `Gap::None`; no "dark" gap was given.
/// Three are from `mahwa panels` (`n01`, `n05`, `n09`), the user's request
/// ("pick some from manhwa_panels").
///
/// **This is the oracle, not a derivation.** Nothing in the repository can
/// re-derive a row - the marking page is an Artifact outside the tree - and
/// MC-077's criteria forbid adjusting one. A change to a row is an Amendment.
///
/// The SHA-256 of each copy is checked against `## Context` when SCAFFOLD
/// copies it (no hashing crate is in this workspace); the byte count and the
/// header dimensions are what a test here can hold.
const FRESH_HELD_OUT: [Fresh; 10] = [
    Fresh {
        file: "2025-10-08 13_14_05.png",
        bytes: 799_297,
        rect: (1073, 167, 399, 1233),
        site: "toongod",
        gap: Gap::White,
    }, // n01
    Fresh {
        file: "2025-03-21 11_19_03.png",
        bytes: 1_315_525,
        rect: (972, 115, 600, 1285),
        site: "toongod",
        gap: Gap::None,
    }, // n02
    Fresh {
        file: "2025-08-05 16_30_27.png",
        bytes: 1_290_884,
        rect: (1006, 115, 533, 1285),
        site: "rolia-scans",
        gap: Gap::None,
    }, // n03
    Fresh {
        file: "2025-07-18 16_28_07.png",
        bytes: 1_282_133,
        rect: (1014, 115, 517, 1285),
        site: "xbato",
        gap: Gap::White,
    }, // n04
    Fresh {
        file: "2025-10-26 12_13_16.png",
        bytes: 1_441_939,
        rect: (973, 167, 600, 1233),
        site: "toongod",
        gap: Gap::White,
    }, // n05
    Fresh {
        file: "2025-03-18 23_33_50.png",
        bytes: 1_124_799,
        rect: (710, 115, 400, 1277),
        site: "toongod",
        gap: Gap::None,
    }, // n06
    Fresh {
        file: "2025-08-05 12_21_01.png",
        bytes: 1_577_799,
        rect: (1006, 115, 533, 1285),
        site: "rolia-scans",
        gap: Gap::None,
    }, // n07
    Fresh {
        file: "2025-07-18 21_55_50.png",
        bytes: 1_324_777,
        rect: (997, 115, 552, 1285),
        site: "xbato",
        gap: Gap::White,
    }, // n08
    Fresh {
        file: "2025-12-04 22_40_23.png",
        bytes: 1_247_184,
        rect: (1002, 167, 540, 1233),
        site: "w-network",
        gap: Gap::White,
    }, // n09
    Fresh {
        file: "2025-03-12 23_13_21.png",
        bytes: 1_637_004,
        rect: (700, 115, 400, 1277),
        site: "toongod",
        gap: Gap::White,
    }, // n10
];

/// MC-062's, MC-068's and MC-077's draws took whole-screen captures only:
/// exactly this size, by header (each story's `## Context`, step 2). None of
/// the 50 is 1920x1080.
const FRESH_DIMENSIONS: (u32, u32) = (2560, 1440);

/// MC-077 AC-4. The 11 entries that were `held-out` before MC-077, at
/// `4272252`, in manifest order: MC-068's fresh draw of 15 less the four
/// MC-072 moved. **Spent** - MC-071 took their one scored run - and moved to
/// `tuning` by MC-077, whose request (the user, 2026-10-03) was a fresh
/// held-out set. Read out of the manifest at `4272252` (every `held-out`
/// entry in it). MC-068's own 21 spent entries stay `tuning`:
/// [`MC062_DRAW`]'s test pins all 25 of that draw as `tuning`.
const MC077_SPENT_HELD_OUT: [&str; 11] = [
    "2025-03-18 14_07_22.png",
    "2025-04-16 17_00_49.png",
    "2025-07-17 16_13_54.png",
    "2025-07-18 00_21_31.png",
    "2025-07-18 08_30_58.png",
    "2025-08-04 08_22_11.png",
    "2025-08-05 08_44_44.png",
    "2025-08-05 11_01_27.png",
    "2025-08-07 00_24_27.png",
    "2025-11-20 23_55_15.png",
    "Screenshot (1460).png",
];

/// MC-068 AC-4: after the move, `tuning` holds this many marked entries and
/// this many flag entries (49 + 14 before MC-068; 45 + 14 before MC-064).
///
/// **84 marked since MC-069**, whose 14 reported screenshots are all marked
/// `tuning` entries (MC-069 AC-1); 70 before it. The flags do not move.
///
/// **88 marked since MC-072**, which moved the four marked entries MC-071 read
/// per file (MC-072 AC-1). The flags do not move.
///
/// **99 marked since MC-077**, which moved the other 11 of MC-068's draw, all
/// marked (MC-077 AC-4). The flags do not move.
const TUNING_MARKED: usize = 99;
const TUNING_FLAGS: usize = 14;

/// MC-068 AC-2 and AC-5: after the move, `held-out` holds this many marked
/// entries, no flag entry ([`MIN_HELD_OUT_FLAGS`]), across this many readers
/// (21 marked across 5 readers before MC-068).
///
/// **11 across 3 since MC-072** (`toongod` 4, `rolia-scans` 4, `xbato` 3);
/// 15 across 4 before it (MC-072 AC-1).
///
/// **10 across 4 since MC-077** (`toongod` 5, `rolia-scans` 2, `xbato` 2,
/// `w-network` 1): MC-077's third fresh draw and nothing else (MC-077 AC-2,
/// AC-4).
const HELD_OUT_MARKED: usize = 10;
const HELD_OUT_READERS: usize = 4;

/// MC-077: the image files in the corpus directory, `manifest.json` aside,
/// one manifest entry each. 113 from MC-069 to MC-076 (MC-072 AC-1: "no file
/// is added to or removed from `fixtures/corpus/`"); 123 since MC-077 added
/// its 10.
const CORPUS_FILES: usize = 123;

/// The tags a drawn entry must carry, sorted: `dark-theme`, `png`,
/// `site:<site>`, and the gutter tag the user's gap answer gives (MC-062 AC-2,
/// MC-068 AC-2).
fn fresh_tags(fresh: &Fresh) -> Vec<String> {
    let mut tags: Vec<String> = ["dark-theme", "png"]
        .into_iter()
        .chain(fresh.gap.tag())
        .map(str::to_owned)
        .chain(std::iter::once(format!("{SITE_TAG_PREFIX}{}", fresh.site)))
        .collect();
    tags.sort();
    tags
}

/// Every way the files of `draw` differ from the draw: absent from the corpus
/// directory, a byte count other than the one drawn, or not
/// [`FRESH_DIMENSIONS`] by header. One row per difference, `file: what`.
fn drawn_files_wrong(draw: &[Fresh]) -> Vec<String> {
    files_wrong(draw.iter().map(|fresh| (fresh.file, fresh.bytes)))
}

/// [`drawn_files_wrong`]'s body, over `(file, bytes)` pairs, so MC-069's
/// reported screenshots ([`MC069_REPORTED`]) are held to exactly the same
/// three checks. Factored out by MC-069; the checks did not change.
fn files_wrong(files: impl IntoIterator<Item = (&'static str, u64)>) -> Vec<String> {
    let dir = corpus::dir();
    let mut wrong: Vec<String> = Vec::new();
    for (file, bytes) in files {
        let path = dir.join(file);
        let Ok(meta) = fs::metadata(&path) else {
            wrong.push(format!("{}: not in {}", file, dir.display()));
            continue;
        };
        if meta.len() != bytes {
            wrong.push(format!(
                "{}: {} bytes, drawn at {}",
                file,
                meta.len(),
                bytes
            ));
        }
        let dims = image::ImageReader::open(&path)
            .and_then(|reader| reader.with_guessed_format())
            .ok()
            .and_then(|reader| reader.into_dimensions().ok());
        if dims != Some(FRESH_DIMENSIONS) {
            wrong.push(format!(
                "{}: dimensions {dims:?} by header, drawn as {FRESH_DIMENSIONS:?}",
                file
            ));
        }
    }
    wrong
}

/// Every way the manifest entries of `draw` differ from the user's frozen
/// marks: not exactly one entry of the name, a `split` other than `split`, an
/// `expect` other than the user's box, or tags other than exactly
/// [`fresh_tags`]. One row per difference, `file: what`.
fn drawn_entries_wrong(entries: &[CorpusEntry], draw: &[Fresh], split: Split) -> Vec<String> {
    entries_wrong(
        entries,
        draw.iter()
            .map(|fresh| (fresh.file, fresh.rect, fresh_tags(fresh))),
        split,
    )
}

/// [`drawn_entries_wrong`]'s body, over `(file, (x, y, w, h), sorted tags)`,
/// so MC-069's reported screenshots, whose tags include `light-theme`, are
/// held to exactly the same checks. Factored out by MC-069; the checks did not
/// change.
fn entries_wrong(
    entries: &[CorpusEntry],
    draw: impl IntoIterator<Item = (&'static str, (u32, u32, u32, u32), Vec<String>)>,
    split: Split,
) -> Vec<String> {
    let mut wrong: Vec<String> = Vec::new();
    for (file, (x, y, w, h), want_tags) in draw {
        let pinned = Rect { x, y, w, h };
        let named: Vec<&CorpusEntry> = entries.iter().filter(|e| e.name() == file).collect();
        let [entry] = named.as_slice() else {
            wrong.push(format!(
                "{}: {} manifest entries of that name, needs exactly one",
                file,
                named.len()
            ));
            continue;
        };
        if entry.split != split {
            wrong.push(format!(
                "{}: split `{}`, needs `{}`",
                file,
                entry.split.as_str(),
                split.as_str()
            ));
        }
        match entry.expect {
            Expect::Rect(actual) if actual == pinned => {}
            Expect::Rect(actual) => wrong.push(format!(
                "{}: expect {actual:?}, the user marked {pinned:?}",
                file
            )),
            Expect::Flag => wrong.push(format!(
                "{}: expect \"flag\", the user marked {pinned:?}",
                file
            )),
        }
        let mut tags = entry.tags.clone();
        tags.sort();
        if tags != want_tags {
            wrong.push(format!(
                "{}: tags {tags:?}, needs exactly {want_tags:?}",
                file
            ));
        }
    }
    wrong
}

// --- MC-062 AC-1: the 25 files are in the corpus, as drawn ------------------

#[test]
fn every_mc062_drawn_screenshot_is_still_in_the_corpus_at_the_size_it_was_drawn_at() {
    assert_eq!(
        drawn_files_wrong(&MC062_DRAW),
        Vec::<String>::new(),
        "MC-062 AC-1: each of the 25 screenshots drawn blind on 2026-09-30 must \
         be in fixtures/corpus/ under its own name, byte for byte the size the \
         draw recorded and a {}x{} whole-screen capture. Spent and `tuning` \
         since MC-068, they are still corpus files. Each row is `file: what \
         differs`",
        FRESH_DIMENSIONS.0,
        FRESH_DIMENSIONS.1
    );
}

// --- MC-062 AC-2: one entry per drawn file, as the user marked it -----------

#[test]
fn every_mc062_drawn_screenshot_is_one_tuning_entry_with_the_users_mark_and_tags() {
    let entries = corpus::load();
    assert_eq!(
        drawn_entries_wrong(&entries, &MC062_DRAW, Split::Tuning),
        Vec::<String>::new(),
        "MC-062 AC-2, as MC-064 and MC-068 move it: each of MC-062's 25 drawn \
         screenshots must be exactly one manifest entry, `tuning` now - MC-063 \
         spent the draw, MC-064 moved the four it read per file and MC-068 the \
         other 21 - whose `expect` is still the box the user marked on \
         2026-09-30 (frozen; the read-back table in MC-062's `## Notes`) and \
         whose tags are still exactly `dark-theme`, `png`, its `site:` and the \
         gutter the user's gap answer gives (`white-gutter` for light, \
         `black-gutter` for dark, none for can't tell) - and no \
         `diagonal-gutter`, the user's ruling. Each row is `file: what differs`"
    );
}

// --- MC-068 AC-1: the 15 files are in the corpus, as drawn ------------------

#[test]
fn every_mc068_drawn_screenshot_is_still_in_the_corpus_at_the_size_it_was_drawn_at() {
    assert_eq!(
        drawn_files_wrong(&MC068_DRAW),
        Vec::<String>::new(),
        "MC-068 AC-1: each of the 15 screenshots the Lead PO drew blind on \
         2026-10-01 must be in fixtures/corpus/ under its own name, byte for \
         byte the size the draw recorded and a {}x{} whole-screen capture. Spent \
         and `tuning` since MC-077, they are still corpus files. Each row is \
         `file: what differs`",
        FRESH_DIMENSIONS.0,
        FRESH_DIMENSIONS.1
    );
}

// --- MC-068 AC-2: one tuning entry per drawn file, as the user marked it ----

#[test]
fn every_mc068_drawn_screenshot_is_one_tuning_entry_with_the_users_mark_and_tags() {
    let entries = corpus::load();
    assert_eq!(
        drawn_entries_wrong(&entries, &MC068_DRAW, Split::Tuning),
        Vec::<String>::new(),
        "MC-068 AC-2, as MC-072 and MC-077 move it: each of MC-068's 15 drawn \
         screenshots must be exactly one manifest entry, `tuning` now - MC-071 \
         spent the draw, MC-072 moved the four it read per file and MC-077 the \
         other 11 - whose `expect` is still the box the user marked on \
         2026-10-01 (frozen, \"Yes, freeze them\"; the read-back table in \
         MC-068's `## Notes`, `n05` as MC-073 re-marked it) and whose tags are \
         still exactly `dark-theme`, `png`, its `site:` and the gutter the \
         user's gap answer gives (`white-gutter` for light, `black-gutter` for \
         dark, none for can't tell). Each row is `file: what differs`"
    );
}

// --- MC-077 AC-1: the 10 files are in the corpus, as drawn ------------------

#[test]
fn every_freshly_drawn_screenshot_is_in_the_corpus_at_the_size_it_was_drawn_at() {
    assert_eq!(
        drawn_files_wrong(&FRESH_HELD_OUT),
        Vec::<String>::new(),
        "MC-077 AC-1: each of the 10 screenshots the Lead PO drew blind on \
         2026-10-03 must be in fixtures/corpus/ under its own name, byte for \
         byte the size the draw recorded and a {}x{} whole-screen capture. The \
         SHA-256 of each copy is checked against `## Context` when it is copied \
         (no hashing crate is in this workspace); the size is what this test \
         can hold. Each row is `file: what differs`",
        FRESH_DIMENSIONS.0,
        FRESH_DIMENSIONS.1
    );
}

// --- MC-077 AC-2: one held-out entry per drawn file, as the user marked it --

#[test]
fn every_freshly_drawn_screenshot_is_one_held_out_entry_with_the_users_mark_and_tags() {
    let entries = corpus::load();
    assert_eq!(
        drawn_entries_wrong(&entries, &FRESH_HELD_OUT, Split::HeldOut),
        Vec::<String>::new(),
        "MC-077 AC-2: each of the 10 drawn screenshots must be exactly one \
         manifest entry, `held-out`, whose `expect` is the box the user marked \
         on 2026-10-03 (frozen, \"Yes, freeze them\"; the read-back table in \
         MC-077's `## Notes`, `x,y wxh`) and whose tags are exactly \
         `dark-theme`, `png`, its `site:` and the gutter the user's gap answer \
         gives (`white-gutter` for white, none for none). Each row is `file: \
         what differs`"
    );
}

// --- MC-077 AC-4: the 11 spent entries are tuning; held-out is only fresh ---

#[test]
fn the_spent_held_out_entries_are_tuning_and_held_out_holds_only_the_fresh_draw() {
    let entries = corpus::load();
    let mut wrong: Vec<String> = Vec::new();

    for name in MC077_SPENT_HELD_OUT {
        match entries.iter().find(|e| e.name() == name) {
            None => wrong.push(format!(
                "{name}: a spent held-out entry no longer in the manifest"
            )),
            Some(entry) if entry.split != Split::Tuning => wrong.push(format!(
                "{name}: spent, and still `{}`, not `tuning`",
                entry.split.as_str()
            )),
            Some(_) => {}
        }
    }
    for entry in held_out(&entries) {
        let name = entry.name();
        if !FRESH_HELD_OUT.iter().any(|f| f.file == name) {
            wrong.push(format!(
                "{name}: `held-out`, and not one of MC-077's 10 fresh screenshots"
            ));
        }
    }

    assert_eq!(
        wrong,
        Vec::<String>::new(),
        "MC-077 AC-4: the 11 entries that were `held-out` before MC-077 have \
         each given their one score (MC-071), so they are `tuning` now, and \
         `held-out` means only the screenshots nothing has been run on: \
         MC-077's 10. Each row names an entry that breaks one or the other"
    );
}

// --- MC-077: the counts after the move ----------------------------------------

#[test]
fn tuning_holds_ninety_nine_marked_and_fourteen_flags_and_held_out_ten_marked_across_four_readers()
{
    let entries = corpus::load();

    // Held-out by reader, so the failure says which reader gained or lost.
    let mut by_site: BTreeMap<String, usize> = BTreeMap::new();
    for entry in held_out(&entries) {
        for site in site_tags(entry) {
            *by_site.entry(site.to_owned()).or_default() += 1;
        }
    }

    // The image files in the directory, the manifest aside.
    let dir = corpus::dir();
    let files = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .map(|e| e.expect("a directory entry").path())
        .filter(|p| p.is_file() && p.file_name().is_some_and(|n| n != corpus::MANIFEST_FILE))
        .count();

    let count = |split: Split, marked: bool| {
        entries
            .iter()
            .filter(|e| e.split == split && matches!(e.expect, Expect::Rect(_)) == marked)
            .count()
    };
    let measured = format!(
        "tuning {} marked + {} flag | held-out {} marked + {} flag across {} readers \
         {by_site:?} | corpus files {files} | manifest entries {}",
        count(Split::Tuning, true),
        count(Split::Tuning, false),
        count(Split::HeldOut, true),
        count(Split::HeldOut, false),
        held_out_sites(&entries).len(),
        entries.len(),
    );
    let recorded_sites: BTreeMap<String, usize> = [
        ("rolia-scans", 2),
        ("toongod", 5),
        ("w-network", 1),
        ("xbato", 2),
    ]
    .into_iter()
    .map(|(s, n)| (s.to_owned(), n))
    .collect();
    let recorded = format!(
        "tuning {TUNING_MARKED} marked + {TUNING_FLAGS} flag | held-out \
         {HELD_OUT_MARKED} marked + {MIN_HELD_OUT_FLAGS} flag across \
         {HELD_OUT_READERS} readers {recorded_sites:?} | corpus files \
         {CORPUS_FILES} | manifest entries {CORPUS_FILES}"
    );

    assert_eq!(
        measured, recorded,
        "MC-077 AC-2, AC-4 and AC-5: with MC-068's draw spent and all 15 of it \
         `tuning`, `tuning` holds 99 marked and 14 flag entries (88 + 14 until \
         MC-077 moved 11; 84 + 14 until MC-072 moved four; 70 + 14 until MC-069 \
         added 14; 49 + 14 until MC-068 moved 21), and `held-out` is MC-077's \
         10, all marked, no flag, across 4 readers - `toongod` 5, `rolia-scans` \
         2, `xbato` 2, `w-network` 1 (11 across 3 until MC-077). The corpus \
         holds 123 images, one manifest entry each (113 until MC-077 added its \
         10). Which entries, and with which marks, is the fresh-set and \
         spent-set tests' job; this is the summary a person reads first"
    );
}

// --- MC-069 AC-1: the 14 screenshots the app called Ambiguous ----------------

/// The page's theme, as the user answered it on MC-069's marking page
/// (*Ambiguous Shots Marks*, `marks-mc069`): the one tag [`Fresh`] cannot
/// carry, because every screenshot MC-062 and MC-068 drew was dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    /// "dark": `dark-theme`.
    Dark,
    /// "light": `light-theme`.
    Light,
}

impl Theme {
    fn tag(self) -> &'static str {
        match self {
            Theme::Dark => "dark-theme",
            Theme::Light => "light-theme",
        }
    }
}

/// One of MC-069's reported screenshots, as the user marked it.
struct Reported {
    /// The original's name, unchanged (AC-1: "under their original names").
    file: &'static str,
    /// The original's size in bytes, from MC-069's `## Notes` table. The
    /// SHA-256 prefix beside it there was checked when the file was copied
    /// (`sha256sum`, and `cmp` against the original); no hashing crate is in
    /// this workspace, so the byte count and the header are what a test here
    /// can hold - MC-068's arrangement.
    bytes: u64,
    /// The user's frozen box as `(x, y, w, h)`.
    rect: (u32, u32, u32, u32),
    /// The reader the user named.
    site: &'static str,
    /// The page's theme.
    theme: Theme,
    /// The user's gap answer.
    gap: Gap,
}

/// MC-069 AC-1. The 14 screenshots the user's run of the app answered
/// `Ambiguous` on (2026-10-01), `a01`..`a14`, with the marks the user froze
/// the same day ("the MC-069 marks are right, freeze them") as **amended
/// once**: `Screenshot (42).png` is `1073,167 400x1225`, was
/// `1073,166 400x1226` (MC-069 `## Amendments`, the user's "Yes, start at
/// 167": row 166 is the browser bar's flat luma 59).
///
/// **This is the oracle, not a derivation.** The marking page is an Artifact
/// outside the tree; a change to a row is an Amendment.
///
/// All 14 are `tuning`, never `held-out`: they were chosen by the app failing
/// on them (MC-069 `## Context`).
const MC069_REPORTED: [Reported; 14] = [
    Reported {
        file: "Screenshot (14).png",
        bytes: 929_358,
        rect: (1139, 167, 267, 1225),
        site: "toongod",
        theme: Theme::Dark,
        gap: Gap::None,
    }, // a01
    Reported {
        file: "Screenshot (19).png",
        bytes: 743_364,
        rect: (1139, 167, 267, 1225),
        site: "toongod",
        theme: Theme::Dark,
        gap: Gap::None,
    }, // a02
    Reported {
        file: "Screenshot (20).png",
        bytes: 926_112,
        rect: (1139, 167, 267, 1225),
        site: "toongod",
        theme: Theme::Dark,
        gap: Gap::None,
    }, // a03
    Reported {
        file: "Screenshot (23).png",
        bytes: 922_508,
        rect: (1139, 167, 267, 1225),
        site: "toongod",
        theme: Theme::Dark,
        gap: Gap::None,
    }, // a04
    Reported {
        file: "Screenshot (42).png",
        bytes: 1_083_580,
        rect: (1073, 167, 400, 1225),
        site: "toongod",
        theme: Theme::Dark,
        gap: Gap::None,
    }, // a05, as amended
    Reported {
        file: "Screenshot (48).png",
        bytes: 537_942,
        rect: (1039, 168, 467, 1224),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::None,
    }, // a06
    Reported {
        file: "Screenshot (49).png",
        bytes: 1_183_580,
        rect: (1039, 167, 467, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::None,
    }, // a07
    Reported {
        file: "Screenshot (50).png",
        bytes: 1_158_118,
        rect: (1039, 167, 467, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::None,
    }, // a08
    Reported {
        file: "Screenshot (51).png",
        bytes: 867_237,
        rect: (993, 167, 560, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::White,
    }, // a09
    Reported {
        file: "Screenshot (52).png",
        bytes: 745_332,
        rect: (1039, 167, 467, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::None,
    }, // a10
    Reported {
        file: "Screenshot (53).png",
        bytes: 696_432,
        rect: (1003, 167, 540, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::None,
    }, // a11
    Reported {
        file: "Screenshot (57).png",
        bytes: 889_716,
        rect: (1033, 167, 479, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::Black,
    }, // a12
    Reported {
        file: "Screenshot (58).png",
        bytes: 884_276,
        rect: (1033, 167, 480, 1225),
        site: "demonicrevolution",
        theme: Theme::Light,
        gap: Gap::White,
    }, // a13
    Reported {
        file: "Screenshot (2705).png",
        bytes: 1_204_928,
        // MC-070 `## Amendments`: was (1073, 133, 400, 1259), "Mark ends at 1471".
        rect: (1073, 133, 399, 1259),
        site: "toongod",
        theme: Theme::Dark,
        gap: Gap::White,
    }, // a14
];

/// The tags a reported entry must carry, sorted: its theme, `png`,
/// `site:<site>`, and the gutter tag its gap answer gives - [`fresh_tags`]'s
/// rule with the theme the user answered rather than `dark-theme` always.
fn reported_tags(reported: &Reported) -> Vec<String> {
    let mut tags: Vec<String> = [reported.theme.tag(), "png"]
        .into_iter()
        .chain(reported.gap.tag())
        .map(str::to_owned)
        .chain(std::iter::once(format!(
            "{SITE_TAG_PREFIX}{}",
            reported.site
        )))
        .collect();
    tags.sort();
    tags
}

#[test]
fn every_screenshot_the_app_called_ambiguous_is_in_the_corpus_at_its_original_size() {
    assert_eq!(
        files_wrong(MC069_REPORTED.iter().map(|r| (r.file, r.bytes))),
        Vec::<String>::new(),
        "MC-069 AC-1: each of the 14 screenshots the user's run of the app \
         answered `Ambiguous` on must be in fixtures/corpus/ under its original \
         name, byte for byte the size of the original (SHA-256 checked against \
         MC-069's `## Notes` when copied) and a {}x{} whole-screen capture. \
         Each row is `file: what differs`",
        FRESH_DIMENSIONS.0,
        FRESH_DIMENSIONS.1
    );
}

#[test]
fn every_screenshot_the_app_called_ambiguous_is_one_tuning_entry_with_the_users_mark_and_tags() {
    let entries = corpus::load();
    assert_eq!(
        entries_wrong(
            &entries,
            MC069_REPORTED
                .iter()
                .map(|r| (r.file, r.rect, reported_tags(r))),
            Split::Tuning,
        ),
        Vec::<String>::new(),
        "MC-069 AC-1: each of the 14 must be exactly one `tuning` manifest entry \
         (never `held-out`: the app's failure chose them) whose `expect` is the \
         box the user froze on 2026-10-01, with `Screenshot (42).png` as amended \
         to 1073,167 400x1225, and whose tags are exactly its theme \
         (`dark-theme` for toongod's five and (2705), `light-theme` for \
         demonicrevolution's eight), `png`, its `site:` and the gutter its gap \
         answer gives. Each row is `file: what differs`"
    );
}
