//! MC-026, AC-1 to AC-5: the decision gates admit a real reader page.
//!
//! The oracle is one sentence - *given every corpus entry's marked rect,
//! `decide` returns `Crop`* - and `decide` takes an image rather than a rect,
//! so it is asked here as the three questions `decide` asks in order, each of
//! them of the public API:
//!
//! 1. **AC-3**, `NoBorderFound`: did anything get trimmed or peeled at all?
//! 2. **AC-2**, `Ambiguous`: was some edge strip a close call?
//! 3. **AC-1**, `LowContent`: is what survived big enough to be a page?
//!
//! Nothing else stands between a marked rect and `Crop`. AC-4 and AC-5 are the
//! two controls that stop the story from reaching those three by simply saying
//! yes to everything: the seven `"expect": "flag"` entries must stay flagged,
//! and no crop may cut a marked page.
//!
//! These are `#[ignore]`d and run by the **`integration`** gate,
//! `cargo test --workspace --release -- --ignored`. They decode 28 real
//! screenshots several times over and have no business in the `unit` gate's
//! loop. To run them by hand:
//!
//! ```text
//! cargo test -p cropper-engine --test corpus -- --ignored --nocapture
//! ```
//!
//! `tests/corpus_manifest.rs` is a different thing and is deliberately not
//! `#[ignore]`d: it checks the corpus is *well formed*. This file checks what
//! the detector does with it.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled elsewhere, read out and never re-derived here**: `margin_px`
//!   (3), `min_content_side` (64), `chrome_flat_fraction` (0.85),
//!   `min_line_spread` (8.0), `uniform_tolerance` (10), `edge_threshold` (24),
//!   `min_content_stddev` (12), `chrome_max_extent` (0.30); MC-007 AC-6's flag
//!   order; and AC-5's containment predicate. Every one of them is read out of
//!   the `Tuning` under test at its use site, never written as a literal at a
//!   comparison.
//! * **Oracle-free, and the two numbers this story chooses**:
//!   `Tuning::min_content_fraction`, **0.05**, and `Tuning::ambiguity_band`,
//!   **0.0025**. Both derivations are below. They are the only calibrated
//!   numbers in this story and nothing else in the `Tuning` table moves.
//! * **Mechanical**: AC-3, the per-file tables, and the six file names AC-4
//!   pins. Exact.
//!
//! # Comparison discipline
//!
//! Every assertion below accumulates its violations over the whole corpus and
//! asserts once, with the per-file table in the message and the first failing
//! entry named. A corpus test whose failure message is a bare count is one
//! nobody can act on, and a loop that asserts per item reports the first file
//! and hides the other twenty.
//!
//! # Deriving `min_content_fraction` = 0.05
//!
//! MC-025's derivation of `min_line_spread` is the standard: a floor from a
//! control that actually fires, a ceiling from a measurement, and a value
//! between them with the evidence written down.
//!
//! * **Ceiling 0.080186, measured.** The smallest marked page in the corpus is
//!   `Screenshot (93).jpg`, marked 246x1167 in a 2560x1440 frame; expanded by
//!   `margin_px` and clamped that is 252x1173, which is 0.080186 of the frame.
//!   A `min_content_fraction` above it rejects a page a person marked. The
//!   twenty-one marked rects run 0.080186 to 0.211156 after expansion, and
//!   **only two of them clear 0.20** - which is the whole reason this story
//!   exists. The full table is printed by AC-1 below.
//! * **Floor 0.024082, from AC-1's control, which fires.** The same rects with
//!   both sides divided by three - one ninth of the area - run 0.009426 to
//!   0.024082 of their frames, the largest being
//!   `2025-08-05 00_11_13.webp` at 216x411. A `min_content_fraction` at or
//!   below 0.024082 admits a rect one ninth the area of a real page, which is
//!   not a floor at all;
//!   `a_marked_page_with_both_sides_divided_by_three_is_rejected` is that
//!   control and it must fail on every one of the twenty-one.
//! * **Window `(0.024082, 0.080186]`, value 0.05.** It sits 2.08x above the
//!   floor and 1.60x below the ceiling, so neither end is close. It is
//!   deliberately *above* the window's geometric centre (0.0439): the defect a
//!   too-low value permits is a **clip** - a fragment of a page cropped as if
//!   it were the page - and the defect a too-high value permits is a **flag**,
//!   a file copied unchanged for a person to look at. MC-005 decision 13 and
//!   AC-5 both say a clip is the worse of the two, so the value leans away
//!   from it. Concretely, on a 500 px wide page 0.05 rejects anything shorter
//!   than about 370 px, which is the clip guard the floor is for.
//! * **It is also the value the headroom probe in the story's `## Notes`,
//!   finding 3, was run at**, so the "6 of 21 crops with 8 clips, and no
//!   configuration both clip-free and better than MC-025's" figure that sizes
//!   MC-027 and MC-028 is a measurement at this number rather than an
//!   extrapolation to it.
//! * **And it keeps an exact boundary fixture.** `decide` flags a rect whose
//!   area is *below* the fraction, so the rect sitting exactly on it must be
//!   cropped, and `crates/core/tests/decide.rs` pins that boundary in whole
//!   pixels: `8000f32 / 160000f32` is bit for bit `0.05f32`, as
//!   `32000f32 / 160000f32` was bit for bit `0.20f32`.
//!
//! # Deriving `ambiguity_band` = 0.0025
//!
//! * **Ceiling 0.0032497, measured by bisection.** The story's `## Notes`,
//!   finding 6, swept a grid and found every marked entry clear at 0.0025; a
//!   grid gives a window and not a margin, so the threshold was bisected per
//!   entry instead - the largest band at which `detect(img, t).ambiguous` is
//!   still false, which is `chrome_flat_fraction` minus the offending strip's
//!   flat fraction. The binding pair is `2026-01-05 13_45_59.png` and
//!   `2026-01-05 13_49_39.png`, both at **0.00324973**: their strip's flat
//!   fraction is 0.846750, and at any band from there up AC-2 fails on them.
//!   The next entry up is `2026-01-05 13_33_41.png` at 0.009935.
//! * **Floor 0.0016667, from a control that stops firing.** The flat fraction
//!   of a strip of `n` pixels can only take the values `k / n`, so a band
//!   narrower than `1 / n` cannot contain one and the `NearlyChrome` verdict
//!   is unreachable on that strip - which is exactly the defect
//!   `ambiguity_band = 0.0` has, arriving gradually instead of at once. The
//!   smallest strip the frozen suites judge is the 100x6 band in
//!   `crates/core/tests/content.rs`, so `1 / 600 = 0.0016667` is the floor: at
//!   or below it, AC-2's control - a strip inside the band that still reports
//!   `ambiguous: true` - cannot be built at that size at all.
//! * **Window `(0.0016667, 0.0032497)`, value 0.0025.** It is 1.50x the floor
//!   and 1.30x below the ceiling, and within a percent of the window's
//!   arithmetic centre (0.0024582). Its margin at the top is small in absolute
//!   terms because the quantity itself is small; what matters is that it is
//!   measured rather than assumed, and AC-2's control below drives the corpus
//!   at 0.005 to show the cliff is real and only 1.3x away.
//! * **It is the one value in the window whose foot is a ratio of whole
//!   pixels.** `chrome_flat_fraction - ambiguity_band` is 0.8475 = 339/400,
//!   and `678f32 / 800f32` is bit for bit `0.85f32 - 0.0025f32`, so
//!   `crates/core/tests/content.rs` can keep an exact boundary fixture for
//!   "the band is closed at its foot" - a 100x8 strip with exactly 678 of its
//!   800 pixels on the background - instead of a tolerance. At 0.002, 0.0015
//!   and 0.001 no strip size in either suite lands on the foot exactly.
//!
//! # Where AC-2's synthetic control lives
//!
//! AC-2's control - *the ambiguity path must still fire at
//! `Tuning::default()`* - is a synthetic scene and its home is the suite that
//! owns the fixture generator: `crates/core/tests/content.rs` builds the
//! four-point ladder 677, 678, 679 and 680 flat pixels of 800 (below the band,
//! exactly on its foot, inside it, and at `chrome_flat_fraction` itself), and
//! `crates/core/tests/decide.rs` carries the same pair through `detect`. This
//! file adds the control the corpus itself can give, which the synthetic one
//! cannot: that the cliff measured above is real, and 1.3x away.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use std::path::Path;

use corpus::{CorpusEntry, Expect};
use cropper_core::{Dimensions, FlagReason, Luma, Rect, Tuning, detect};
use cropper_engine::{Flag, Outcome, process_file};

// --- Constants read out of the story, never calibrated here -----------------

/// MC-026 AC-4. The six `"expect": "flag"` entries that are `Flagged` on
/// `main` at `a42f8e8`, all six by `Detector(NoBorderFound)`. The seventh flag
/// entry, `2025-02-27 22_46_15.png`, is cropped today and this story is not
/// required to fix it - that is MC-019's 90% - so it is named below as an
/// explicit exclusion rather than left to be inferred from a list of six.
const STILL_FLAGGED: [&str; 6] = [
    "2025-03-03 11_06_04.png",
    "2025-03-03 11_24_19.png",
    "2025-05-12 22_55_40.png",
    "2025-05-12 22_58_53.png",
    "Screenshot (3455).png",
    "Screenshot (3465).png",
];

/// The one `"expect": "flag"` entry AC-4 deliberately does not pin.
const NOT_THIS_STORYS_TO_FIX: &str = "2025-02-27 22_46_15.png";

/// AC-2's corpus-side control. The band at which the two entries closest to
/// `chrome_flat_fraction` become ambiguous again: their measured threshold is
/// 0.00324973, so 0.005 is past the cliff and 0.0025 is short of it. Not a
/// candidate value for anything - it is 1.3x the chosen band, and it is here
/// to show the cliff is real.
const BAND_PAST_THE_CLIFF: f32 = 0.005;

/// The two entries that go ambiguous again at [`BAND_PAST_THE_CLIFF`], and the
/// only two of the twenty-eight that do.
const AMBIGUOUS_PAST_THE_CLIFF: [&str; 2] = ["2026-01-05 13_45_59.png", "2026-01-05 13_49_39.png"];

/// AC-5's control: how far in each side a marked rect is pulled to build a
/// rect that genuinely clips it. Any positive number would do; ten pixels is
/// far larger than `margin_px` so no expansion can hide it.
const CLIP_INSET: u32 = 10;

// --- Harness ----------------------------------------------------------------

/// The corpus entries that carry a marked rect, with the rect, in manifest
/// order.
fn marked() -> Vec<(CorpusEntry, Rect)> {
    corpus::load()
        .into_iter()
        .filter_map(|entry| match entry.expect {
            Expect::Rect(rect) => Some((entry, rect)),
            Expect::Flag => None,
        })
        .collect()
}

/// The luma plane of `path`, through the engine's own decoder and the engine's
/// own luma conversion - so what is measured here is what `process_file` sees.
fn luma(path: &Path) -> Luma {
    let bytes =
        std::fs::read(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// The dimensions of `img`.
fn dims(img: &Luma) -> Dimensions {
    Dimensions {
        width: img.width,
        height: img.height,
    }
}

/// The rect the pipeline would judge if the locator were perfect: the marked
/// rect, expanded by `margin_px` and clamped to the image, which is exactly
/// what `detect` does to whatever its last stage returns.
fn expanded(rect: Rect, t: &Tuning, at: Dimensions) -> Rect {
    cropper_core::margin::expand(rect, t.margin_px, at)
}

/// A rect's area as a share of the image's, in `f32` and with both counts
/// widened first - the arithmetic `decide` does, for the reason its module
/// documentation gives.
fn area_fraction(rect: Rect, at: Dimensions) -> f32 {
    let area = u64::from(rect.w) * u64::from(rect.h);
    area as f32 / at.area() as f32
}

/// Whether `rect` clears the size gate `decide` applies: area at least
/// `min_content_fraction` of the image and both sides at least
/// `min_content_side`. Both limits are read from `t`.
fn clears_size_gate(rect: Rect, t: &Tuning, at: Dimensions) -> bool {
    area_fraction(rect, at) >= t.min_content_fraction
        && rect.w >= t.min_content_side
        && rect.h >= t.min_content_side
}

/// Whether `outer` contains `inner` entirely - AC-5's settled predicate, and
/// the definition of "not a clip".
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// A temp directory for `process_file`'s outputs, removed when the test ends.
fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temp dir")
}

/// Print a table under `--nocapture` and return it as one string for the
/// assertion message, so the same rows are visible whether the test passes or
/// fails.
fn table(title: &str, header: &str, rows: &[String]) -> String {
    let mut out = String::from(title);
    out.push('\n');
    out.push_str(header);
    out.push('\n');
    for row in rows {
        out.push_str(row);
        out.push('\n');
    }
    println!("{out}");
    out
}

// --- AC-1: the size gate admits every marked page ---------------------------

/// AC-1. Every corpus entry's marked rect, expanded by `margin_px` and clamped
/// to the image, clears both halves of `decide`'s size gate.
///
/// This is the criterion the story exists for. On `main` at `a42f8e8`,
/// `min_content_fraction` is 0.20 and nineteen of the twenty-one fail here;
/// the two that pass are `2025-08-05 00_11_13.webp` (0.21116) and
/// `Screenshot (2744).jpg` (0.20471).
///
/// `min_content_side` is settled and does not bind: the smallest marked side
/// in the corpus is 246 px against a 64 px limit. It is checked anyway because
/// AC-1 is "the size gate", not "the area half of the size gate", and a tuning
/// change that satisfied the area while breaking the side would still leave
/// `decide` answering `LowContent`.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn every_marked_page_clears_the_size_gate() {
    let t = Tuning::default();
    let mut rows = Vec::new();
    let mut failures = Vec::new();

    for (entry, rect) in marked() {
        let img = luma(&entry.path);
        let at = dims(&img);
        let grown = expanded(rect, &t, at);
        let frac = area_fraction(grown, at);
        let ok = clears_size_gate(grown, &t, at);
        rows.push(format!(
            "{:<30} {:>11} {:>12} {:>10.5} {:>9} {:>7}",
            entry.name(),
            format!("{}x{}", at.width, at.height),
            format!("{}x{}", grown.w, grown.h),
            frac,
            grown.w.min(grown.h),
            if ok { "admit" } else { "REJECT" }
        ));
        if !ok {
            failures.push(format!(
                "{}: {}x{} is {frac:.5} of {}x{}, against min_content_fraction {} and \
                 min_content_side {}",
                entry.name(),
                grown.w,
                grown.h,
                at.width,
                at.height,
                t.min_content_fraction,
                t.min_content_side
            ));
        }
    }

    let printed = table(
        "AC-1: every marked rect, expanded by margin_px and clamped",
        &format!(
            "{:<30} {:>11} {:>12} {:>10} {:>9} {:>7}",
            "file", "image", "expanded", "area frac", "min side", "verdict"
        ),
        &rows,
    );

    assert!(
        !rows.is_empty(),
        "AC-1 checked no entries at all; the corpus has no marked rect"
    );
    assert!(
        failures.is_empty(),
        "AC-1: the size gate must admit every page a person marked, or no locator \
         can ever be measured against this corpus. {} of {} entries were rejected; \
         the first is {}.\n\n{printed}\nall rejections:\n{}",
        failures.len(),
        rows.len(),
        failures[0],
        failures.join("\n")
    );
}

/// AC-1's control on the constant, and the floor of `min_content_fraction`'s
/// admissible window.
///
/// The same check, applied to every marked rect with **both sides divided by
/// three** - one ninth of the area, 0.009426 to 0.024082 of its image. Every
/// one of them must be rejected. A `min_content_fraction` low enough to admit
/// a rect one ninth the area of a real page is not a floor at all, and AC-1
/// on its own would be satisfied by setting the constant to zero.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn a_marked_page_with_both_sides_divided_by_three_is_rejected_by_the_size_gate() {
    let t = Tuning::default();
    let mut rows = Vec::new();
    let mut admitted = Vec::new();

    for (entry, rect) in marked() {
        let img = luma(&entry.path);
        let at = dims(&img);
        let ninth = expanded(
            Rect {
                x: rect.x,
                y: rect.y,
                w: rect.w / 3,
                h: rect.h / 3,
            },
            &t,
            at,
        );
        let frac = area_fraction(ninth, at);
        let ok = clears_size_gate(ninth, &t, at);
        rows.push(format!(
            "{:<30} {:>12} {:>10.5} {:>9} {:>7}",
            entry.name(),
            format!("{}x{}", ninth.w, ninth.h),
            frac,
            ninth.w.min(ninth.h),
            if ok { "ADMIT" } else { "reject" }
        ));
        if ok {
            admitted.push(format!(
                "{}: {}x{} is {frac:.5} of {}x{} and was admitted at \
                 min_content_fraction {}",
                entry.name(),
                ninth.w,
                ninth.h,
                at.width,
                at.height,
                t.min_content_fraction
            ));
        }
    }

    let printed = table(
        "AC-1's control: every marked rect with both sides divided by three",
        &format!(
            "{:<30} {:>12} {:>10} {:>9} {:>7}",
            "file", "one third", "area frac", "min side", "verdict"
        ),
        &rows,
    );

    assert!(!rows.is_empty(), "AC-1's control checked no entries at all");
    assert!(
        admitted.is_empty(),
        "AC-1's control: a rect one ninth the area of a real page must be rejected \
         by the size gate, or min_content_fraction is not a floor. {} of {} were \
         admitted; the first is {}.\n\n{printed}\nall admissions:\n{}",
        admitted.len(),
        rows.len(),
        admitted[0],
        admitted.join("\n")
    );
}

// --- AC-2: no marked page is ambiguous --------------------------------------

/// AC-2. `detect` at `Tuning::default()` reports `ambiguous: false` for every
/// corpus entry that carries a marked rect.
///
/// Eight are `true` on `main` at `a42f8e8`, listed in the story's `## Notes`,
/// finding 6: the three `2026-01-05` entries, `Screenshot (67)`,
/// `Screenshot (70)`, `Screenshot (75)`, `Screenshot (93)` and
/// `Screenshot (103)`. `decide` asks this question before it reaches the size
/// gate, so AC-1 alone would leave all eight flagged.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_marked_page_is_reported_ambiguous() {
    let t = Tuning::default();
    let mut rows = Vec::new();
    let mut ambiguous = Vec::new();

    for (entry, _) in marked() {
        let img = luma(&entry.path);
        let found = detect(&img, &t);
        let flag = found.as_ref().is_some_and(|d| d.ambiguous);
        rows.push(format!(
            "{:<30} {:>10} {:>9}",
            entry.name(),
            if found.is_some() {
                "detected"
            } else {
                "uniform"
            },
            if flag { "AMBIGUOUS" } else { "." }
        ));
        if flag {
            ambiguous.push(entry.name());
        }
    }

    let printed = table(
        &format!(
            "AC-2: detect(.., Tuning::default()).ambiguous, at ambiguity_band {}",
            t.ambiguity_band
        ),
        &format!("{:<30} {:>10} {:>9}", "file", "detect", "ambiguous"),
        &rows,
    );

    assert!(!rows.is_empty(), "AC-2 checked no entries at all");
    assert!(
        ambiguous.is_empty(),
        "AC-2: a page a person marked must not turn on a close call the detector is \
         not confident about - `decide` answers Flag(Ambiguous) before it ever \
         reaches the size gate. {} of {} are ambiguous at ambiguity_band {}; the \
         first is {}.\n\n{printed}\nall ambiguous entries: {:?}",
        ambiguous.len(),
        rows.len(),
        t.ambiguity_band,
        ambiguous[0],
        ambiguous
    );
}

/// AC-2's control on the constant, from the corpus rather than from a fixture.
///
/// An `ambiguity_band` of 0.0 satisfies AC-2 by deleting the feature.
/// `crates/core/tests/content.rs` and `crates/core/tests/decide.rs` are where
/// that is caught on a synthetic strip; what they cannot show is that the
/// chosen band is measured rather than merely small. This drives the whole
/// corpus at 0.005 - 1.3x the chosen band, and past the bisected cliff at
/// 0.00324973 - and pins that **exactly two** entries become ambiguous again
/// there, and which two.
///
/// So the band is not "somewhere below 0.05": it is immediately below a real
/// boundary on real files, and a band raised even to 0.005 breaks AC-2.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_two_pages_closest_to_chrome_are_ambiguous_again_one_step_above_the_band() {
    let t = Tuning::default();
    let past = Tuning {
        ambiguity_band: BAND_PAST_THE_CLIFF,
        ..Tuning::default()
    };
    assert!(
        t.ambiguity_band > 0.0,
        "AC-2's control: an ambiguity_band of 0.0 deletes the feature - the \
         NearlyChrome verdict becomes unreachable - and satisfies AC-2 by saying \
         nothing. The band must be strictly positive; it is {}",
        t.ambiguity_band
    );
    assert!(
        t.ambiguity_band < BAND_PAST_THE_CLIFF,
        "AC-2's control drives the corpus at {BAND_PAST_THE_CLIFF}, which must be \
         above the chosen band of {} for the comparison below to mean anything",
        t.ambiguity_band
    );

    let mut rows = Vec::new();
    let mut at_band = Vec::new();
    let mut past_band = Vec::new();

    for entry in corpus::load() {
        let img = luma(&entry.path);
        let now = detect(&img, &t).as_ref().is_some_and(|d| d.ambiguous);
        let then = detect(&img, &past).as_ref().is_some_and(|d| d.ambiguous);
        rows.push(format!(
            "{:<30} {:>10} {:>12}",
            entry.name(),
            if now { "AMBIGUOUS" } else { "." },
            if then { "AMBIGUOUS" } else { "." }
        ));
        if now {
            at_band.push(entry.name());
        }
        if then {
            past_band.push(entry.name());
        }
    }

    let printed = table(
        &format!(
            "AC-2's control: ambiguity at the chosen band {} against {BAND_PAST_THE_CLIFF}",
            t.ambiguity_band
        ),
        &format!("{:<30} {:>10} {:>12}", "file", "at default", "at 0.005"),
        &rows,
    );

    assert_eq!(
        past_band,
        AMBIGUOUS_PAST_THE_CLIFF.map(String::from).to_vec(),
        "AC-2's control: the ambiguity path must still fire on real files one step \
         above the chosen band - these two entries' offending strip sits 0.0032497 \
         below chrome_flat_fraction, which is the measured ceiling the band was \
         derived from. If this list is empty the feature is gone; if it is longer \
         the cliff has moved and the derivation needs re-measuring.\n\n{printed}"
    );
    assert!(
        at_band.is_empty(),
        "AC-2's control: and at the chosen band of {} nothing is ambiguous, so the \
         band sits strictly below the cliff. Still ambiguous: {at_band:?}\n\n{printed}",
        t.ambiguity_band
    );
}

// --- AC-3: no marked page is answered NoBorderFound -------------------------

/// AC-3, with its own negative control in the same test.
///
/// `decide` answers `NoBorderFound` when `!trimmed && removed.is_empty()` -
/// nothing was trimmed and no strip was peeled, so the rect is the whole image
/// and cropping to it would be a no-op. That question is asked **before** the
/// ambiguity and the size gate, so AC-1 and AC-2 are worth nothing without
/// this: a tuning change that stopped the pipeline trimming would satisfy both
/// of them and still answer `Flag`.
///
/// This holds on `main` at `a42f8e8` for all twenty-one, so it is a regression
/// guard and green on arrival. What earns it is the second half, which is the
/// negative control `reference/red-phase.md` asks for: the same predicate
/// **must** fire on the six entries AC-4 pins, which are flagged
/// `Detector(NoBorderFound)` today. One half without the other would prove
/// only that the predicate can be false.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_no_border_predicate_fires_on_every_flagged_page_and_on_no_marked_page() {
    let t = Tuning::default();
    let mut rows = Vec::new();
    let mut marked_without_a_border = Vec::new();
    let mut flagged_with_one = Vec::new();

    for entry in corpus::load() {
        let img = luma(&entry.path);
        let found = detect(&img, &t);
        let no_border = found
            .as_ref()
            .is_none_or(|d| !d.trimmed && d.removed.is_empty());
        let is_marked = matches!(entry.expect, Expect::Rect(_));
        rows.push(format!(
            "{:<30} {:>7} {:>9} {:>10} {:>14}",
            entry.name(),
            if is_marked { "crop" } else { "flag" },
            found.as_ref().is_some_and(|d| d.trimmed),
            found.as_ref().map_or(0, |d| d.removed.len()),
            if no_border { "NoBorderFound" } else { "." }
        ));
        if is_marked && no_border {
            marked_without_a_border.push(entry.name());
        }
        if !is_marked && STILL_FLAGGED.contains(&entry.name().as_str()) && !no_border {
            flagged_with_one.push(entry.name());
        }
    }

    let printed = table(
        "AC-3: decide's first question, `!trimmed && removed.is_empty()`",
        &format!(
            "{:<30} {:>7} {:>9} {:>10} {:>14}",
            "file", "expect", "trimmed", "removed", "first answer"
        ),
        &rows,
    );

    assert!(
        marked_without_a_border.is_empty(),
        "AC-3: `decide` asks NoBorderFound before anything else, so a marked page \
         that answers it is lost whatever AC-1 and AC-2 do. Entries: \
         {marked_without_a_border:?}\n\n{printed}"
    );
    assert!(
        flagged_with_one.is_empty(),
        "AC-3's negative control: the same predicate must still FIRE on the six \
         entries AC-4 pins, or the assertion above is passing because the predicate \
         never fires rather than because these pages have a border. Entries that \
         stopped answering NoBorderFound: {flagged_with_one:?}\n\n{printed}"
    );
}

// --- AC-4: the negative control, flagged pages stay flagged -----------------

/// AC-4. The six `"expect": "flag"` entries that are `Flagged` on `main` at
/// `a42f8e8` are still `Flagged`, and still for `Detector(NoBorderFound)`.
///
/// This is the control that stops the story reaching AC-1 to AC-3 by saying
/// yes to everything. It is pinned by name rather than by count, because a
/// count of six would survive one file starting to crop and another starting
/// to flag.
///
/// The seventh entry, `2025-02-27 22_46_15.png`, is **cropped** today - to
/// `0,3 553x853` - and this story is not required to fix it. AC-4 asserts
/// nothing about it, and this test says so in code so that nobody later reads
/// the list of six as an oversight.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_six_pages_flagged_today_are_still_flagged_for_no_border_found() {
    let t = Tuning::default();
    let tmp = scratch();
    let mut rows = Vec::new();
    let mut wrong = Vec::new();
    let mut seen = Vec::new();

    for entry in corpus::load() {
        if entry.expect != Expect::Flag {
            continue;
        }
        let output = tmp.path().join(entry.name());
        let result = process_file(&entry.path, &output, &t);
        let shown = match &result.outcome {
            Outcome::Cropped { rect, .. } => {
                format!("Cropped {},{} {}x{}", rect.x, rect.y, rect.w, rect.h)
            }
            Outcome::Flagged { reason, .. } => format!("Flagged {reason:?}"),
            Outcome::Failed { error } => format!("Failed {error}"),
        };
        let pinned = STILL_FLAGGED.contains(&entry.name().as_str());
        rows.push(format!(
            "{:<30} {:>7} {}",
            entry.name(),
            if pinned { "pinned" } else { "-" },
            shown
        ));
        if !pinned {
            continue;
        }
        seen.push(entry.name());
        let ok = matches!(
            &result.outcome,
            Outcome::Flagged {
                reason: Flag::Detector(FlagReason::NoBorderFound),
                ..
            }
        );
        if !ok {
            wrong.push(format!("{}: {shown}", entry.name()));
        }
    }

    let printed = table(
        "AC-4: process_file over the seven `expect: flag` entries",
        &format!("{:<30} {:>7} {}", "file", "AC-4", "outcome"),
        &rows,
    );

    assert_eq!(
        seen,
        STILL_FLAGGED.map(String::from).to_vec(),
        "AC-4 must reach all six entries it pins, in manifest order. A name that \
         has drifted turns this control into a test of five files or of \
         none.\n\n{printed}"
    );
    assert!(
        wrong.is_empty(),
        "AC-4: opening the two decision gates must not start cropping the pages the \
         corpus says to leave alone. {} of the six changed; the first is {}.\n\n\
         {printed}\nall changes:\n{}",
        wrong.len(),
        wrong[0],
        wrong.join("\n")
    );
    assert!(
        rows.iter()
            .any(|row| row.starts_with(NOT_THIS_STORYS_TO_FIX)),
        "AC-4 deliberately excludes {NOT_THIS_STORYS_TO_FIX}, which is cropped today \
         and is MC-019's 90% rather than this story's. The exclusion is only \
         meaningful while the entry is still in the corpus.\n\n{printed}"
    );
}

// --- AC-5: zero clips, still ------------------------------------------------

/// AC-5, with its own negative control in the same test.
///
/// No entry whose outcome is `Cropped` has a rect that fails to contain the
/// marked rect entirely. The count is **0 on `main` at `a42f8e8`**, and
/// opening a size gate is exactly the change that could turn a flag into a
/// clip: a rect that was discarded as too small is now cropped to, and if it
/// sits inside the page that is the worst defect the product has (MC-005
/// decision 13).
///
/// On `main` nothing in the corpus is cropped at all, so the first half of
/// this test passes **vacuously** and would pass against a predicate that
/// always says yes. The control is the second half: the same predicate,
/// applied to each marked rect pulled in by ten pixels on every side, must
/// report a clip on all twenty-one. The number of entries actually cropped is
/// printed either way, so the vacuity is visible rather than inferred.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_crop_clips_a_marked_page() {
    let t = Tuning::default();
    let tmp = scratch();
    let mut rows = Vec::new();
    let mut clips = Vec::new();
    let mut cropped = 0usize;
    let mut control_missed = Vec::new();

    for (entry, rect) in marked() {
        let output = tmp.path().join(entry.name());
        let result = process_file(&entry.path, &output, &t);
        let shown = match &result.outcome {
            Outcome::Cropped { rect: got, .. } => {
                cropped += 1;
                if contains(*got, rect) {
                    format!("Cropped {},{} {}x{}  contains", got.x, got.y, got.w, got.h)
                } else {
                    clips.push(format!(
                        "{}: cropped to {},{} {}x{} which does not contain the marked \
                         {},{} {}x{}",
                        entry.name(),
                        got.x,
                        got.y,
                        got.w,
                        got.h,
                        rect.x,
                        rect.y,
                        rect.w,
                        rect.h
                    ));
                    format!("Cropped {},{} {}x{}  CLIPS", got.x, got.y, got.w, got.h)
                }
            }
            Outcome::Flagged { reason, .. } => format!("Flagged {reason:?}"),
            Outcome::Failed { error } => format!("Failed {error}"),
        };
        rows.push(format!("{:<30} {shown}", entry.name()));

        // The control: a rect that really does clip this page must be reported
        // as clipping it. Without this the assertion above says nothing while
        // nothing is cropped.
        let inset = Rect {
            x: rect.x + CLIP_INSET,
            y: rect.y + CLIP_INSET,
            w: rect.w - 2 * CLIP_INSET,
            h: rect.h - 2 * CLIP_INSET,
        };
        if contains(inset, rect) {
            control_missed.push(entry.name());
        }
    }

    let printed = table(
        &format!("AC-5: process_file over the twenty-one marked entries ({cropped} cropped)"),
        &format!("{:<30} {}", "file", "outcome"),
        &rows,
    );

    assert!(
        control_missed.is_empty(),
        "AC-5's control: a marked rect pulled in by {CLIP_INSET} px on every side \
         clips the page by construction, so the containment predicate must say so \
         for all {} entries. It did not for {control_missed:?}, which means the \
         assertion below cannot detect a clip either",
        rows.len()
    );
    assert!(
        clips.is_empty(),
        "AC-5: a crop that does not contain the page a person marked is the worst \
         defect this product has, and opening the size gate is what could cause \
         one. {} of {} cropped entries clip; the first is {}.\n\n{printed}\n\
         all clips:\n{}",
        clips.len(),
        cropped,
        clips[0],
        clips.join("\n")
    );
}
