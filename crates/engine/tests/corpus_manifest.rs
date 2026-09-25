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
//!   tags and the 60 MB ceiling, both from MC-018's acceptance criteria; the
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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// MC-018 AC-1. Below this the corpus cannot support a "nine in ten" claim
/// with any resolution: at twenty entries one miss is five percent.
const MIN_ENTRIES: usize = 20;

/// MC-018 AC-4, so the repository stays clonable without LFS.
const MAX_BYTES: u64 = 60 * 1024 * 1024;

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
fn the_whole_corpus_fits_under_sixty_megabytes() {
    let dir = corpus::dir();
    let total: u64 = fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .map(|e| e.expect("a directory entry"))
        .filter(|e| e.path().is_file())
        .map(|e| e.metadata().expect("file metadata").len())
        .sum();

    assert!(
        total < MAX_BYTES,
        "AC-4: the corpus is {:.1} MB, over the {:.0} MB ceiling that keeps \
         this repository clonable without LFS",
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
const MIN_HELD_OUT_MARKED: usize = 20;

/// MC-037 AC-2. The detector has two jobs - crop the croppable and decline the
/// rest - and a held-out set of rectangles alone measures one of them.
const MIN_HELD_OUT_FLAGS: usize = 4;

/// MC-037 AC-4. Twenty held-out entries from a single reader would measure
/// almost nothing: a reader's furniture is pixel-identical across every
/// screenshot from it, so a rule can score well by learning that geometry
/// rather than learning what a page is.
const MIN_HELD_OUT_SITES: usize = 4;

/// MC-037 AC-4. The readers appearing *only* in the held-out set are the only
/// entries that can answer "does this generalise to an unseen reader?".
const MIN_UNSEEN_SITES: usize = 2;

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
fn the_held_out_set_carries_at_least_twenty_marked_entries() {
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
         twenty entries one miss is five percent, and below that the reported \
         number is decided by which screenshot happened to land here. The \
         whole-corpus floor is a different question and stays green while this \
         one fails - a floor that only fires when the *total* drops is not \
         measuring the split"
    );
}

// --- AC-2: and enough entries that should be left alone ---------------------

#[test]
fn the_held_out_set_carries_at_least_four_flag_entries() {
    let entries = corpus::load();
    let held = held_out(&entries);

    assert!(
        !held.is_empty(),
        "AC-2: no entry in the manifest is `held-out`, so this count is over \
         the empty set"
    );

    // Counted over `Expect::Flag` specifically, never over "held-out entries"
    // as a whole. A count of all held-out entries would be satisfied by a set
    // of twenty-four rectangles and zero flags - precisely the corpus this
    // criterion exists to reject - and it would read as passing.
    let flags = held
        .iter()
        .filter(|e| matches!(e.expect, Expect::Flag))
        .count();

    assert!(
        flags >= MIN_HELD_OUT_FLAGS,
        "AC-2: the held-out set carries {flags} entries expecting `flag` and \
         needs at least {MIN_HELD_OUT_FLAGS}. The detector has two jobs - crop \
         the croppable and decline the rest - and a held-out set of rectangles \
         alone measures one of them. MC-026's decision gates are scored \
         against exactly these. Held-out entries in total: {}",
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
         learning that geometry instead of learning what a page is. Readers \
         found: {sites:?}",
        sites.len()
    );
}

#[test]
fn at_least_two_held_out_readers_are_absent_from_the_pre_epic_07_set() {
    let entries = corpus::load();
    let sites = held_out_sites(&entries);
    let known = documented_pre_epic_07_readers();

    let unseen: Vec<&String> = sites.iter().filter(|s| !known.contains(*s)).collect();

    assert!(
        unseen.len() >= MIN_UNSEEN_SITES,
        "AC-4: {} held-out readers are absent from the pre-EPIC-07 set, and at \
         least {MIN_UNSEEN_SITES} are needed. These are the only entries that \
         can answer 'does this generalise to an unseen *reader*?' rather than \
         'an unseen *page*?'. Every other held-out entry shares a \
         pixel-identical navigation bar with an entry the detector was tuned \
         on, which flatters exactly the rules EPIC-07 story 3 proposes. This \
         is a strictly stronger claim than the reader count above, and the two \
         must be able to disagree: collapsing every held-out entry onto the \
         four pre-EPIC-07 readers satisfies that one and fails this. If they \
         cannot disagree, one of them is decorative. Held-out readers: \
         {sites:?}. Pre-EPIC-07 readers: {known:?}. Unseen: {unseen:?}",
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
const READER_BY_FILE: [(&str, &str); 30] = [
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
const READER_LABELS: [(&str, usize, usize, usize, usize); 7] = [
    ("toongod", 17, 13, 4, 8),
    ("w-network", 5, 2, 3, 5),
    ("demonicrevolution", 4, 4, 0, 1),
    ("rolia-scans", 2, 2, 0, 8),
    ("kunmanga", 2, 2, 0, 2),
    ("xbato", 0, 0, 0, 3),
    ("manhwaclan", 0, 0, 0, 2),
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
/// Written as `(file, x, y, w, h)` and rebuilt into a [`Rect`] where it is
/// read: a literal `Rect { .. }` per row is twenty-one rectangles rustfmt
/// explodes over nine lines each, and a pin nobody can scan in one screen is a
/// pin nobody re-reads.
const MARKED_TUNING_RECTS: [(&str, u32, u32, u32, u32); 23] = [
    ("2025-08-05 00_11_13.webp", 958, 114, 631, 1216),
    ("2025-08-05 00_11_27.webp", 1008, 118, 528, 1225),
    ("2025-10-14 23_29_06.png", 1008, 188, 528, 1138),
    ("2025-10-14 23_30_20.png", 1040, 171, 476, 1208),
    ("2025-10-20 15_37_25.png", 1008, 228, 528, 1074),
    ("2026-01-05 13_33_41.png", 1073, 233, 397, 1060),
    ("2026-01-05 13_45_59.png", 1040, 204, 466, 1167),
    ("2026-01-05 13_49_39.png", 1044, 286, 459, 1110),
    ("Screenshot (67).png", 1012, 290, 524, 1098),
    ("Screenshot (70).jpg", 1016, 298, 520, 1012),
    ("Screenshot (75).png", 1077, 171, 393, 1208),
    ("Screenshot (93).jpg", 1143, 212, 246, 1167),
    ("Screenshot (103).jpg", 1077, 302, 389, 844),
    ("Screenshot (1661).png", 1077, 192, 393, 1085),
    ("Screenshot (2582).jpg", 1008, 216, 524, 1008),
    ("Screenshot (2630).jpg", 958, 224, 635, 889),
    ("Screenshot (2698).jpg", 954, 343, 631, 742),
    ("Screenshot (2708).jpg", 1073, 220, 393, 1102),
    ("Screenshot (2744).jpg", 950, 134, 639, 1164),
    ("Screenshot (3187).png", 987, 142, 574, 1237),
    ("Screenshot (3538).png", 975, 171, 590, 1159),
    // MC-052: the two entries the user moved from `held-out` on 2026-09-24,
    // with the marks they carried there, unchanged.
    ("2025-03-06 01_22_45.png", 648, 118, 520, 1244),
    ("2025-03-07 00_58_06.png", 671, 362, 474, 928),
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
         labelled. `xbato`'s zero tuning entries is a recorded finding, not an \
         omission. Each row is \
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
