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
