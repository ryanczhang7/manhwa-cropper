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
//!
//!   **Re-measured by MC-062, recorded on the user's ruling of 2026-09-30
//!   ("Record it").** MC-062 moved 23 spent held-out entries into `tuning`,
//!   and one of them binds lower: `Screenshot (56).png` at **0.00265363**
//!   (`Screenshot (59).png` is at 0.00349161; the MC-026 pair re-measures at
//!   0.00324973). So the ceiling is now **0.0026536**. `ambiguity_band` is
//!   unchanged at 0.0025, which is still below it.
//! * **Floor 0.0016667, from a control that stops firing.** The flat fraction
//!   of a strip of `n` pixels can only take the values `k / n`, so a band
//!   narrower than `1 / n` cannot contain one and the `NearlyChrome` verdict
//!   is unreachable on that strip - which is exactly the defect
//!   `ambiguity_band = 0.0` has, arriving gradually instead of at once. The
//!   smallest strip the frozen suites judge is the 100x6 band in
//!   `crates/core/tests/content.rs`, so `1 / 600 = 0.0016667` is the floor: at
//!   or below it, AC-2's control - a strip inside the band that still reports
//!   `ambiguous: true` - cannot be built at that size at all.
//! * **Window `(0.0016667, 0.0026536)`, value 0.0025** (MC-062; it was
//!   `(0.0016667, 0.0032497)` over the twenty-eight). It is 1.50x the floor
//!   and **1.06x** below the ceiling (it was 1.30x), no longer near the
//!   window's centre. Its margin at the top is small in absolute terms because
//!   the quantity itself is small; what matters is that it is measured rather
//!   than assumed, and AC-2's control below drives the corpus at 0.005 to show
//!   the cliff is real and only 1.06x away.
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
//! cannot: that the cliff measured above is real, and 1.06x away (1.3x
//! before MC-062).

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use std::path::Path;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::content::content_box;
use cropper_core::trim::trim_uniform;
use cropper_core::{CropDecision, Dimensions, FlagReason, Luma, Rect, Tuning, decide, detect};
use cropper_engine::{Flag, Outcome, process_file};

// --- Constants read out of the story, never calibrated here -----------------

/// MC-026 AC-4. The six `"expect": "flag"` entries that are `Flagged` on
/// `main` at `a42f8e8`, all six by `Detector(NoBorderFound)`. The seventh flag
/// entry, `2025-02-27 22_46_15.png`, is cropped today and this story is not
/// required to fix it - that is MC-019's 90% - so it is named below as an
/// explicit exclusion rather than left to be inferred from a list of six.
///
/// **MC-062 adds five**, in manifest order: the flag entries among the 23
/// spent held-out entries the user moved to `tuning` on 2026-09-30 that are
/// `Flagged` by `Detector(NoBorderFound)` on `7c36b5d` (crates unchanged since
/// `d2876f5`), read out of one run in MC-062's RED on a scratch copy with the
/// move applied. The other two of the seven moved flag entries are **cropped**
/// there; they are named in [`KNOWN_CROPPED_FLAGS`].
const STILL_FLAGGED: [&str; 11] = [
    "2025-03-03 11_06_04.png",
    "2025-03-03 11_24_19.png",
    "2025-05-12 22_55_40.png",
    "2025-05-12 22_58_53.png",
    "Screenshot (3455).png",
    "Screenshot (3465).png",
    "2024-09-09 23_30_01.png",
    "2024-09-09 23_55_27.png",
    "2025-05-12 10_37_44.png",
    "2025-05-13 00_21_02.png",
    "2025-10-05 01_34_27.png",
];

/// The `"expect": "flag"` entries AC-4 deliberately does not pin, because the
/// app **crops** them. **Exact in both directions**: AC-4 fails if any other
/// `tuning` flag entry is neither here nor in [`STILL_FLAGGED`], and fails if
/// one named here is `Flagged` after all, so the story that fixes one has to
/// move it into [`STILL_FLAGGED`].
///
/// * `2025-02-27 22_46_15.png`: MC-026's own exclusion, cropped since before
///   EPIC-07 (MC-019's 90 %, not MC-026's to fix).
/// * `2025-03-03 11_00_13.png` and `2025-05-12 20_48_42.png`: MC-062, the
///   user's ruling of 2026-09-30 ("Known misses"). Both are `all-art` flag
///   entries among the 23 spent held-out entries moved to `tuning`, and both
///   are cropped (`0,1 542x1201` and `0,5 1892x4679` at margin 0). MC-051:
///   they are the "Cropped 2" of its 7 held-out flag entries at `d2876f5`
///   ("flags: of 7, Flagged 5, Cropped 2"; crates unchanged since). To be
///   fixed later, not here.
const KNOWN_CROPPED_FLAGS: [&str; 3] = [
    "2025-02-27 22_46_15.png",
    "2025-03-03 11_00_13.png",
    "2025-05-12 20_48_42.png",
];

/// AC-2's corpus-side control. The band at which the entries closest to
/// `chrome_flat_fraction` become ambiguous again: the lowest measured
/// threshold is 0.00265363 (`Screenshot (56).png`, since MC-062; 0.00324973
/// before), so 0.005 is past the cliff and 0.0025 is short of it. Not a
/// candidate value for anything - it is here to show the cliff is real.
const BAND_PAST_THE_CLIFF: f32 = 0.005;

/// The entries that go ambiguous again at [`BAND_PAST_THE_CLIFF`], in manifest
/// order. MC-026's two, and since MC-062 two of the 23 spent held-out entries
/// moved to `tuning`, `Screenshot (56).png` (threshold 0.00265363, now the
/// binding entry) and `Screenshot (59).png` (0.00349161) - recorded on the
/// user's ruling of 2026-09-30 ("Record it"). `ambiguity_band` is unchanged.
///
/// **MC-068 adds one**, `Screenshot (9).png`, one of the 21 spent fresh
/// entries MC-068 moved to `tuning`: it turns ambiguous at the band
/// 0.00474861, bisected in MC-068's RED on `345a9eb` with the move applied by
/// the method that reproduces the three recorded values to the eighth place
/// (`(56)` 0.00265363, `(59)` 0.00349161, `13_45_59` 0.00324973). That is
/// above `(56)`'s, so the binding entry, the window and the 1.06x are
/// unchanged; this is a read-out of a newly-`tuning` entry, not a move of the
/// cliff. None of the other 20 turns ambiguous at 0.005.
const AMBIGUOUS_PAST_THE_CLIFF: [&str; 5] = [
    "2026-01-05 13_45_59.png",
    "2026-01-05 13_49_39.png",
    "Screenshot (56).png",
    "Screenshot (59).png",
    "Screenshot (9).png",
];

/// The one marked `tuning` entry `decide` answers `Flag(Ambiguous)` at
/// `Tuning::default()`, so it crops nothing.
///
/// **MC-069, the user's answer of 2026-10-01 ("Its own story").**
/// `Screenshot (2705).png`: its close call is the page's own bottom rows,
/// `0,1366 2545x26` (flat fraction 0.848708), which lie inside the crop, so
/// MC-069's rule leaves it flagged. MC-070 fixes it.
///
/// It was `2025-07-17 14_20_23.png` from MC-056 (the user's ruling of
/// 2026-09-29) until MC-069: that entry's close call is the browser
/// scrollbar, `x 2545..2560` (0.849273), which its crop lies wholly left of,
/// so under MC-069's rule it crops and it left this list.
///
/// **Exact in both directions.** The tests naming it fail if any *other*
/// marked `tuning` entry is flagged `Ambiguous`, and fail if this one stops
/// being - so the story that fixes it has to empty this list, and cannot leave
/// a stale exception behind.
///
/// **MC-070 empties it** (its AC-1): `(2705)`'s bottom strip is the page's own
/// white gap between two dark margins - its tone over the page column is 255
/// against its median of 11 - so it is the page's rows and not a close call,
/// and `(2705)` crops. No marked `tuning` entry is flagged `Ambiguous` now,
/// and the list stays exact: any entry flagged `Ambiguous` fails.
const KNOWN_AMBIGUOUS: [&str; 0] = [];

/// MC-069 AC-4: the `tuning` entries whose **strip-level** judgement,
/// `content_box(img, trim_uniform(img, t)?, t).ambiguous`, is true at the
/// default band, in manifest order. This is where the band and its cliff live,
/// and MC-069 does not change it: only what `decide` makes of a close call
/// narrows.
///
/// Measured on `main` at `afeaf3b` with MC-069's 14 added: MC-056's
/// `2025-07-17 14_20_23.png` (the scrollbar, 0.849273) and all 14 of MC-069's
/// (the scrollbar on the 13, flat fractions 0.847765 to 0.849860; the page's
/// bottom rows on `(2705)`, 0.848708) - MC-069 `## Amendments`, read back by
/// RED. Until MC-069 this list and [`KNOWN_AMBIGUOUS`] were the same list,
/// because `decide` flagged every close call; they are not any more.
///
/// **MC-070 takes `(2705)` off** (its AC-1 and AC-3): the rule MC-070 ships is
/// a statement about the strip - a Top or Bottom strip whose tone over the
/// page column differs from its own median by more than `uniform_tolerance`
/// is the page's, "not chrome, and not nearly chrome" (MC-070 `## Notes`) -
/// so `content_box` no longer calls `(2705)`'s bottom rows a close call. Every
/// other entry here is the scrollbar, a Right strip, which the rule does not
/// touch. 14 names.
const STRIP_AMBIGUOUS_AT_THE_BAND: [&str; 14] = [
    "2025-07-17 14_20_23.png",
    "Screenshot (14).png",
    "Screenshot (19).png",
    "Screenshot (20).png",
    "Screenshot (23).png",
    "Screenshot (42).png",
    "Screenshot (48).png",
    "Screenshot (49).png",
    "Screenshot (50).png",
    "Screenshot (51).png",
    "Screenshot (52).png",
    "Screenshot (53).png",
    "Screenshot (57).png",
    "Screenshot (58).png",
];

/// MC-064, the user's ruling of 2026-09-30 (its Open question 1, *"List them
/// as known"*): the three fresh entries MC-063 read per file whose crop clips
/// the mark, moved to `tuning` by MC-064, `(file, [x, y, w, h])` at
/// `Tuning::default()` (margin_px 0) - the only margin the two zero-clip tests
/// here run at. In manifest order. **Measured, not chosen**: read out of one
/// run of `process_file` on `43e8e61` (release; crates unchanged since
/// `d2876f5`) in MC-064's RED, on a scratch copy with only the four `split`
/// values changed. They are MC-063's recorded crops: `f18` columns 1828..2545
/// rows 0..1440, `f20` 1022..1516 rows 115..1400, `f09` 1006..1537 rows
/// 167..1400 (end exclusive). MC-065 fixes them.
///
/// **Exact in both directions**: each test naming this list fails if any
/// *other* marked `tuning` entry clips, and fails if a listed entry's crop is
/// anything but its pin - so the story that fixes one has to take it off, and
/// cannot leave a stale exception behind.
///
/// MC-065 took `f20` (`2025-08-07 01_13_55.png`) off (its AC-2): its right
/// edge is fixed there, and these tests judge it like every other entry.
///
/// MC-066 took `f18` (`2025-03-06 12_48_06.png`) off (its AC-1): its crop is
/// the reader's window now, and these tests judge it like every other entry.
///
/// MC-067 took `f09` (`2025-12-08 17_22_50.png`) off (its AC-1): it keeps its
/// art's near-black right edge, and these tests judge it like every other
/// entry. Its pin was `[1006, 167, 531, 1233]`. The list was empty, and
/// stayed exact in both directions: no known clip was left.
///
/// **MC-072 adds two**, by the user's answer of 2026-10-02 to its Open
/// question 3 (*"list them by name as known problems"*, recommended; the
/// answer was *"all recommended"*): the two fresh entries MC-071 read per file
/// whose crop clips the mark, moved to `tuning` by MC-072, in manifest order.
/// **Measured, not chosen**: read out of this suite's own table, run on
/// `0f9c579` (release) in MC-072's RED on a scratch copy with only the four
/// `split` values changed. They equal MC-071's recorded crops: `n02` columns
/// 703..1101 rows 0..1440, `n05` 1007..1538 rows 167..1400 (end exclusive).
/// MC-073 to MC-076 fix them, and each takes its own entry off.
///
/// MC-073 took `n05` (`2025-11-01 12_34_31.png`) off (its AC-2): the user
/// re-marked it, ruling on 2026-10-02 *"Box starts at 1007"* - column 1006 is
/// the dark seam beside the art, as on `Screenshot (3605).png` and
/// `2025-10-20 15_37_25.png`, not art. Its mark is `1007,167 531x1233` now,
/// which its crop at margin 0 already equals, so these tests judge it like
/// every other entry. Its pin was `[1007, 167, 531, 1233]`. The list stays
/// exact in both directions.
const KNOWN_CLIPS: [(&str, [u32; 4]); 1] = [("2025-03-07 00_05_58.png", [703, 0, 398, 1440])];

/// The pinned crop of a [`KNOWN_CLIPS`] entry, if `name` is one.
fn known_clip(name: &str) -> Option<Rect> {
    KNOWN_CLIPS
        .iter()
        .find(|(file, _)| *file == name)
        .map(|&(_, [x, y, w, h])| Rect { x, y, w, h })
}

/// MC-064: every [`KNOWN_CLIPS`] entry whose measured crop is not exactly its
/// pin, as a row naming both. `got` is `(file, crop or None)` for every marked
/// entry the test reached. A listed entry not reached at all is reported too.
fn known_clips_moved(got: &[(String, Option<Rect>)]) -> Vec<String> {
    KNOWN_CLIPS
        .iter()
        .filter_map(|&(file, _)| {
            let pinned = known_clip(file);
            match got.iter().find(|(name, _)| name == file) {
                None => Some(format!(
                    "{file}: a known clip that is not a marked tuning entry here"
                )),
                Some((_, crop)) if *crop == pinned => None,
                Some((_, crop)) => Some(format!("{file}: crop {crop:?}, pinned {pinned:?}")),
            }
        })
        .collect()
}

/// AC-5's control: how far in each side a marked rect is pulled to build a
/// rect that genuinely clips it. Any positive number would do; ten pixels is
/// far larger than `margin_px` so no expansion can hide it.
const CLIP_INSET: u32 = 10;

// --- Harness ----------------------------------------------------------------

/// The `tuning` half of the corpus, in manifest order. **Every accuracy
/// measurement in this file runs over this and never over `corpus::load()`.**
///
/// MC-037 added thirty-one `held-out` entries, and `docs/wiki/corpus.md` is
/// explicit that a held-out entry is scored once, at the end of a v2 attempt,
/// and never before. These suites are not that attempt: they are v1's recorded
/// behaviour, and running them across held-out entries would spend the set on
/// a measurement nobody asked for, every time the `integration` gate runs.
///
/// It also keeps the existing numbers meaning what they meant. MC-019's,
/// MC-026's and MC-032's figures were recorded against the twenty-eight
/// pre-EPIC-07 entries; without this filter their denominator would have
/// silently become fifty-nine and every one of those claims would quietly
/// describe a different corpus than the one it was written about.
///
/// Whichever EPIC-07 story earns the first held-out score will select
/// `Split::HeldOut` deliberately, once, and say so in its own file.
fn tuning_only() -> Vec<CorpusEntry> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .collect()
}

/// The corpus entries that carry a marked rect, with the rect, in manifest
/// order. Tuning only, via [`tuning_only`].
fn marked() -> Vec<(CorpusEntry, Rect)> {
    tuning_only()
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

/// MC-069: `content_box`'s strip-level judgement over the rect `detect` hands
/// it - `trim_uniform` first, exactly as `detect` composes the two - which is
/// where `ambiguity_band` acts. `false` for an image `trim_uniform` drains,
/// as `detect` returns `None` there. MC-069 does not change this; it changes
/// only what `decide` makes of it.
fn strip_level_ambiguous(img: &Luma, t: &Tuning) -> bool {
    trim_uniform(img, t).is_some_and(|first| content_box(img, first, t).ambiguous)
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

/// AC-2. `decide` at `Tuning::default()` does not answer `Flag(Ambiguous)`
/// for any corpus entry that carries a marked rect, but the known exception.
///
/// Eight were ambiguous on `main` at `a42f8e8`, listed in the story's
/// `## Notes`, finding 6: the three `2026-01-05` entries, `Screenshot (67)`,
/// `Screenshot (70)`, `Screenshot (75)`, `Screenshot (93)` and
/// `Screenshot (103)`. `decide` asks this question before it reaches the size
/// gate, so AC-1 alone would leave all eight flagged.
///
/// **MC-069 re-instruments it on `decide`.** It read `detect(..).ambiguous`
/// until MC-069, which was the same question while every close call flagged.
/// MC-069's rule narrows the *decision* only - a close call on a strip the
/// crop leaves out no longer flags - so the question this test asks, "is a
/// marked page flagged `Ambiguous`", is now asked of the decision itself, and
/// whether `Detection::ambiguous` narrows with it is left to the implementer.
/// On `main` at `afeaf3b` this fails on `2025-07-17 14_20_23.png` and the 13
/// of MC-069 (their close call is the browser scrollbar, outside the crop),
/// and holds on `(2705)`, the new [`KNOWN_AMBIGUOUS`].
///
/// **MC-070 empties [`KNOWN_AMBIGUOUS`]**: no marked `tuning` entry may be
/// flagged `Ambiguous`. On `main` at `eb54767` this fails on `(2705)` alone.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_marked_page_is_reported_ambiguous() {
    let t = Tuning::default();
    let mut rows = Vec::new();
    let mut ambiguous = Vec::new();

    for (entry, _) in marked() {
        let img = luma(&entry.path);
        let decision = decide(&img, &t);
        let flag = decision == CropDecision::Flag(FlagReason::Ambiguous);
        rows.push(format!(
            "{:<30} {:>9}  {decision:?}",
            entry.name(),
            if flag { "AMBIGUOUS" } else { "." }
        ));
        if flag {
            ambiguous.push(entry.name());
        }
    }

    let printed = table(
        &format!(
            "AC-2: decide(.., Tuning::default()) == Flag(Ambiguous), at ambiguity_band {}",
            t.ambiguity_band
        ),
        &format!("{:<30} {:>9}  {}", "file", "ambiguous", "decision"),
        &rows,
    );

    assert!(!rows.is_empty(), "AC-2 checked no entries at all");
    // MC-056: exact in both directions against KNOWN_AMBIGUOUS. Any other
    // marked page that is flagged Ambiguous fails this exactly as it did
    // before MC-056; the known exception going unflagged fails it too, until
    // the list is emptied.
    assert_eq!(
        ambiguous,
        KNOWN_AMBIGUOUS.map(String::from).to_vec(),
        "AC-2: a page a person marked must not turn on a close call the detector is \
         not confident about - `decide` answers Flag(Ambiguous) before it ever \
         reaches the size gate. No marked page is allowed to be flagged \
         Ambiguous: MC-070 emptied KNOWN_AMBIGUOUS, whose last entry, MC-069's \
         (2705), had its close call on the page's own white bottom rows. Since \
         MC-069 a close call on a strip the crop lies wholly outside of does not \
         flag; since MC-070 a Top or Bottom strip whose tone over the page column \
         is not its own median is the page's, not a close call. {} of {} are \
         flagged Ambiguous at ambiguity_band {}. `left` is measured, `right` is \
         KNOWN_AMBIGUOUS.\n\n{printed}",
        ambiguous.len(),
        rows.len(),
        t.ambiguity_band
    );
}

/// AC-2's control on the constant, from the corpus rather than from a fixture.
///
/// An `ambiguity_band` of 0.0 satisfies AC-2 by deleting the feature.
/// `crates/core/tests/content.rs` and `crates/core/tests/decide.rs` are where
/// that is caught on a synthetic strip; what they cannot show is that the
/// chosen band is measured rather than merely small. This drives the whole
/// corpus at 0.005 - past the bisected cliff, 0.00265363 since MC-062
/// (0.00324973 before) - and pins exactly which entries become ambiguous again
/// there: [`AMBIGUOUS_PAST_THE_CLIFF`], four since MC-062 (the user's ruling of
/// 2026-09-30, "Record it"; renamed from `the_two_pages_...`), five since
/// MC-068 (`Screenshot (9).png`, above the binding threshold).
///
/// So the band is not "somewhere below 0.05": it is immediately below a real
/// boundary on real files, and a band raised even to 0.005 breaks AC-2.
///
/// **MC-069 measures it at the strip judgement**, `content_box` over
/// `trim_uniform`'s rect - exactly as `detect` composes them - where it read
/// `detect(..).ambiguous` until MC-069. The band and its cliff live in
/// `content_box`, and MC-069 changes only what `decide` makes of a close call
/// (MC-069 AC-4): read through the decision, every entry whose near strip
/// lies outside its crop would vanish from both columns and the cliff would
/// look like it had moved when nothing in the band had. So the turned list is
/// still exactly [`AMBIGUOUS_PAST_THE_CLIFF`] - MC-069's 14 are already close
/// calls at the default band, so none of them turns - and the at-band list is
/// its own exact pin, [`STRIP_AMBIGUOUS_AT_THE_BAND`], no longer
/// [`KNOWN_AMBIGUOUS`]. The decision-level list is AC-2's, above.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_pages_closest_to_chrome_are_ambiguous_again_one_step_above_the_band() {
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

    for entry in tuning_only() {
        let img = luma(&entry.path);
        let now = strip_level_ambiguous(&img, &t);
        let then = strip_level_ambiguous(&img, &past);
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
            "AC-2's control: content_box's strip-level ambiguity at the chosen band {} \
             against {BAND_PAST_THE_CLIFF}",
            t.ambiguity_band
        ),
        &format!("{:<30} {:>10} {:>12}", "file", "at default", "at 0.005"),
        &rows,
    );

    // The entries that *turn* ambiguous past the band: ambiguous at 0.005 and
    // not at the default. Before MC-056 nothing was ambiguous at the default,
    // so this was `past_band` itself; the entries already ambiguous at the
    // default (STRIP_AMBIGUOUS_AT_THE_BAND since MC-069) are ambiguous at both
    // bands and are accounted for by the two assertions after this one.
    let turned: Vec<String> = past_band
        .iter()
        .filter(|name| !at_band.contains(name))
        .cloned()
        .collect();
    assert_eq!(
        turned,
        AMBIGUOUS_PAST_THE_CLIFF.map(String::from).to_vec(),
        "AC-2's control: the ambiguity path must still fire on real files one step \
         above the chosen band. The lowest of these entries' offending strips sits \
         0.00265363 below chrome_flat_fraction (Screenshot (56).png, recorded by \
         MC-062 on the user's ruling of 2026-09-30; 0.0032497 before), which is \
         the measured ceiling the band sits under, 1.06x above it; MC-068's \
         Screenshot (9).png turns at 0.00474861, above it. If this list is \
         empty the feature is gone; if it changes the cliff has moved and the \
         derivation needs re-measuring. Measured at content_box's strip-level \
         judgement since MC-069, which does not change it. `left` is \
         measured.\n\n{printed}"
    );
    assert_eq!(
        at_band,
        STRIP_AMBIGUOUS_AT_THE_BAND.map(String::from).to_vec(),
        "AC-2's control, as MC-069 AC-4 measures it: at the chosen band of {} \
         content_box calls a strip a close call on exactly \
         STRIP_AMBIGUOUS_AT_THE_BAND - MC-056's 2025-07-17 14_20_23.png and \
         MC-069's 13 scrollbars; MC-070 took (2705)'s bottom rows off, as the \
         page's and not nearly chrome - and on nothing else, so the band sits strictly below the \
         cliff. This is the strip judgement, which MC-069 leaves alone; which of \
         these `decide` flags is AC-2's question (KNOWN_AMBIGUOUS). `left` is \
         measured.\n\n{printed}",
        t.ambiguity_band
    );
    let missing_past: Vec<&String> = at_band
        .iter()
        .filter(|name| !past_band.contains(name))
        .collect();
    assert!(
        missing_past.is_empty(),
        "AC-2's control: an entry ambiguous at the chosen band must still be \
         ambiguous at the wider {BAND_PAST_THE_CLIFF}; not: {missing_past:?}\n\n{printed}"
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

    for entry in tuning_only() {
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
        "AC-3's negative control: the same predicate must still FIRE on the \
         entries AC-4 pins (STILL_FLAGGED, eleven since MC-062), or the assertion above is passing because the predicate \
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
///
/// **MC-062** moves seven more flag entries into `tuning` and pins the five
/// that are flagged. Before MC-062 a `tuning` flag entry that was neither
/// pinned nor the named exclusion was silently skipped, which was harmless
/// while there were none; now every `tuning` flag entry must be one or the
/// other, so an entry the corpus says to leave alone and the app crops is
/// named rather than skipped. Two of the seven are cropped, and the user ruled
/// them known exceptions on 2026-09-30 ("Known misses"): the exclusion became
/// the list [`KNOWN_CROPPED_FLAGS`], exact in both directions.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn every_flag_entry_but_the_named_known_exceptions_is_still_flagged_for_no_border_found() {
    let t = Tuning::default();
    let tmp = scratch();
    let mut rows = Vec::new();
    let mut wrong = Vec::new();
    let mut seen = Vec::new();
    let mut unpinned = Vec::new();
    let mut still_cropped = Vec::new();

    for entry in tuning_only() {
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
            if KNOWN_CROPPED_FLAGS.contains(&entry.name().as_str()) {
                // The other direction: a known exception counts only while it
                // is still cropped.
                if matches!(result.outcome, Outcome::Cropped { .. }) {
                    still_cropped.push(entry.name());
                }
            } else {
                unpinned.push(format!("{}: {shown}", entry.name()));
            }
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
        "AC-4: process_file over the `expect: flag` tuning entries",
        &format!("{:<30} {:>7} {}", "file", "AC-4", "outcome"),
        &rows,
    );

    assert_eq!(
        seen,
        STILL_FLAGGED.map(String::from).to_vec(),
        "AC-4 must reach all {} entries it pins, in manifest order. A name that \
         has drifted turns this control into a test of fewer files or of \
         none.\n\n{printed}",
        STILL_FLAGGED.len()
    );
    assert!(
        wrong.is_empty(),
        "AC-4: opening the two decision gates must not start cropping the pages the \
         corpus says to leave alone. {} of the {} pinned changed; the first is {}.\n\n\
         {printed}\nall changes:\n{}",
        wrong.len(),
        STILL_FLAGGED.len(),
        wrong[0],
        wrong.join("\n")
    );
    assert!(
        unpinned.is_empty(),
        "AC-4, as MC-062 extends it: every `tuning` entry expecting `flag` is either \
         pinned as flagged (STILL_FLAGGED) or is a named known exception \
         (KNOWN_CROPPED_FLAGS, {KNOWN_CROPPED_FLAGS:?}). These are neither - the \
         corpus says to leave them alone, and this is what the app does with them. \
         An exception is the user's ruling, never this file's:\n{}\n\n{printed}",
        unpinned.join("\n")
    );
    assert_eq!(
        still_cropped,
        KNOWN_CROPPED_FLAGS.map(String::from).to_vec(),
        "AC-4 deliberately excludes KNOWN_CROPPED_FLAGS because the app crops them \
         (2025-02-27 22_46_15.png since MC-026; the other two by the user's ruling \
         of 2026-09-30, MC-062). Exact in both directions: each must still be in \
         the corpus as a `tuning` flag entry and still be cropped. One missing from \
         `left` is flagged now, or gone: move it into STILL_FLAGGED, or drop it. \
         `left` is measured.\n\n{printed}"
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
///
/// Twenty-six marked tuning entries since MC-053, which moved three whose dark,
/// low-texture art the page column locator cut: on post-MC-052 `main` this
/// fails on exactly those three, and it is one of MC-053's AC-1 regression
/// tests (the margin-3 half; `tests/corpus_page_column.rs` has margin 0).
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_crop_clips_a_marked_page() {
    let t = Tuning::default();
    let tmp = scratch();
    let mut rows = Vec::new();
    let mut clips = Vec::new();
    let mut cropped = 0usize;
    let mut control_missed = Vec::new();
    let mut got_all: Vec<(String, Option<Rect>)> = Vec::new();

    for (entry, rect) in marked() {
        let output = tmp.path().join(entry.name());
        let result = process_file(&entry.path, &output, &t);
        got_all.push((
            entry.name(),
            match &result.outcome {
                Outcome::Cropped { rect: got, .. } => Some(*got),
                _ => None,
            },
        ));
        let shown = match &result.outcome {
            Outcome::Cropped { rect: got, .. } => {
                cropped += 1;
                if contains(*got, rect) {
                    format!("Cropped {},{} {}x{}  contains", got.x, got.y, got.w, got.h)
                } else if known_clip(&entry.name()).is_some() {
                    // MC-064: a known clip, held to its pinned crop below.
                    format!(
                        "Cropped {},{} {}x{}  CLIPS (KNOWN)",
                        got.x, got.y, got.w, got.h
                    )
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
        &format!(
            "AC-5: process_file over the {} marked tuning entries ({cropped} cropped)",
            rows.len()
        ),
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
    let moved = known_clips_moved(&got_all);
    assert!(
        moved.is_empty(),
        "MC-064, MC-072: each known clip (KNOWN_CLIPS, the user's rulings of 2026-09-30 and 2026-10-02) must \
         still be cropped to exactly its pinned rect - if a fix moves one, take it \
         off the list and let this test judge it:\n{}\n\n{printed}",
        moved.join("\n")
    );
    assert!(
        clips.is_empty(),
        "AC-5: a crop that does not contain the page a person marked is the worst \
         defect this product has, and opening the size gate is what could cause \
         one. {} of {} cropped entries clip besides the {} known clips (MC-064, MC-072); the \
         first is {}.\n\n{printed}\n\
         all clips:\n{}",
        clips.len(),
        cropped,
        KNOWN_CLIPS.len(),
        clips[0],
        clips.join("\n")
    );
}

// ============================================================================
// MC-027 AC-4 and AC-5: the corpus, on the column axis only
// ============================================================================
//
// MC-027 locates the page's **left and right** edges by its flat page margins.
// The rows are explicitly not this story's - after it lands most entries are
// still cropped to something far too tall, and MC-028 owns that - so every
// number below is a column number and the two tests here say so in their
// names.
//
// # Reading MC-019's window off the produced rect
//
// AC-4 states the window as `[e.x - 8, e.x + 3]` on the low edge and
// `[e.x + e.w - 1 - 3, e.x + e.w - 1 + 8]` on the high one, and asks it of the
// rect `process_file` produced. Those two statements are in different
// coordinates and MC-019 is what reconciles them. MC-019 AC-1 requires the
// produced rect to **contain** the marked one and AC-2 requires it to lie
// inside the marked one **expanded by `margin_px + 8`**, so in produced
// coordinates the window is `[e.x - 11, e.x]` - and `detect` expands whatever
// its last stage returns by `margin_px` on every side, so the located edge it
// was computed from lies in `[e.x - 8, e.x + 3]`, which is AC-4's bracket
// exactly and is the convention MC-026 finding 7's per-file table is printed
// in.
//
// Both readings are the same arithmetic said twice, and the way to say it once
// is to compare the produced rect against the marked rect **expanded by
// `margin_px`** - which is what `expanded()` above already does for AC-1, and
// what [`column_offsets`] does here. Applying AC-4's bracket to the *raw* mark
// instead would be a window 3 px tighter on each side than MC-019's; RED
// measured 11 of 21 under that reading and 20 of 21 under this one, and the
// story's `## Handoff` records both so the choice is visible rather than
// assumed.
//
// # What is settled and what is measured
//
// * **Settled elsewhere, read out here**: MC-019's window and `margin_px`,
//   both above; `min_line_spread` (8.0) and `central_band_fraction` (0.6),
//   read from the `Tuning` under test and derived in
//   `crates/core/tests/flatness.rs`. Nothing is calibrated in this file.
// * **Measured in RED, against the candidate implementation, outside the
//   repository**: 20 of 21 entries hit at the settled fraction, the only miss
//   being `Screenshot (93).jpg` at `-4,+17`; 21 of 21 entries fail the control;
//   21 entries cropped and 0 of them clipping. The per-file tables below print
//   the same numbers on every run.

/// MC-019's window on the **low** (left) edge, in produced coordinates against
/// the mark expanded by `margin_px`. See the module note above.
const LOW_WINDOW: std::ops::RangeInclusive<i64> = -8..=3;

/// The same on the **high** (right) edge.
const HIGH_WINDOW: std::ops::RangeInclusive<i64> = -3..=8;

/// AC-4's floor: how many of the twenty-one must land inside the window.
const COLUMN_HITS_REQUIRED: usize = 19;

/// AC-4's control on the metric: how far the expected left edge is shifted
/// **inward** before the comparison is redone.
///
/// Far larger than the largest real offset - RED measured every one of the
/// twenty-one inside 7 px - so a metric that still reports a hit after the
/// shift is a metric that is not reading the left edge at all.
const LEFT_SHIFT_INWARD: u32 = 40;

/// AC-4's control floor: how many entries the shifted comparison must fail on.
const CONTROL_FAILURES_REQUIRED: usize = 15;

/// AC-5's control: how far in the left and right edges of a marked rect are
/// pulled to build a rect that genuinely clips it on the column axis.
const COLUMN_CLIP_INSET: u32 = 10;

/// The produced rect's left and right offsets from `reference`'s, both in
/// pixels and both signed: negative is outward (more page kept), positive is
/// inward (page lost) on the low edge and the mirror on the high one.
fn column_offsets(produced: Rect, reference: Rect) -> (i64, i64) {
    let low = i64::from(produced.x) - i64::from(reference.x);
    let high = (i64::from(produced.x) + i64::from(produced.w) - 1)
        - (i64::from(reference.x) + i64::from(reference.w) - 1);
    (low, high)
}

/// Whether a pair of offsets lands inside MC-019's window.
fn inside_the_window((low, high): (i64, i64)) -> bool {
    LOW_WINDOW.contains(&low) && HIGH_WINDOW.contains(&high)
}

/// The rect `process_file` produced for `entry`, or `None` where it was
/// flagged or failed.
fn produced(entry: &CorpusEntry, t: &Tuning, tmp: &tempfile::TempDir) -> Option<Rect> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Some(rect),
        _ => None,
    }
}

// --- AC-4: the left and right edges land inside MC-019's window -------------

/// AC-4, with its control on the metric in the same test.
///
/// Every corpus entry that carries a marked rect, through `process_file` at
/// `Tuning::default()`: the produced rect's left and right edges must land
/// inside MC-019's window of the marked rect's, on at least
/// [`COLUMN_HITS_REQUIRED`] of the twenty-one. The rows are not measured here
/// at all.
///
/// On `main` before this story the column locator does not exist, so the
/// produced rect spans whatever the chrome peel left - the story's `## Handoff`
/// records the "before" count. The control is the second half: with the
/// expected left edge shifted [`LEFT_SHIFT_INWARD`] px inward, the same check
/// must **fail** on at least [`CONTROL_FAILURES_REQUIRED`] entries. Without it
/// this test would be satisfied by a window wide enough to admit anything, and
/// a metric that passes whatever the locator returns is the failure mode a
/// corpus criterion has.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_left_and_right_edges_of_every_marked_page_land_inside_the_window() {
    let t = Tuning::default();
    let tmp = scratch();
    let mut rows = Vec::new();
    let mut hits = Vec::new();
    let mut misses = Vec::new();
    let mut control_hits = Vec::new();

    for (entry, mark) in marked() {
        let img = luma(&entry.path);
        let at = dims(&img);
        let grown = expanded(mark, &t, at);
        let Some(rect) = produced(&entry, &t, &tmp) else {
            rows.push(format!(
                "{:<30} {:>13} {:>13} {:>7} {:>7} {:>6}",
                entry.name(),
                "not cropped",
                format!("{}..{}", grown.x, grown.x + grown.w - 1),
                "-",
                "-",
                "MISS"
            ));
            misses.push(format!("{}: not cropped at all", entry.name()));
            continue;
        };
        let offsets = column_offsets(rect, grown);
        let ok = inside_the_window(offsets);
        rows.push(format!(
            "{:<30} {:>13} {:>13} {:>+7} {:>+7} {:>6}",
            entry.name(),
            format!("{}..{}", rect.x, rect.x + rect.w - 1),
            format!("{}..{}", grown.x, grown.x + grown.w - 1),
            offsets.0,
            offsets.1,
            if ok { "hit" } else { "MISS" }
        ));
        if ok {
            hits.push(entry.name());
        } else {
            misses.push(format!(
                "{}: produced {}..{} against expected {}..{}, offsets {:+},{:+}, \
                 against the windows {LOW_WINDOW:?} and {HIGH_WINDOW:?}",
                entry.name(),
                rect.x,
                rect.x + rect.w - 1,
                grown.x,
                grown.x + grown.w - 1,
                offsets.0,
                offsets.1
            ));
        }

        // The control: the same comparison against an expected left edge
        // shifted inward, which no correct locator can match.
        let shifted = Rect {
            x: grown.x + LEFT_SHIFT_INWARD,
            w: grown.w - LEFT_SHIFT_INWARD,
            ..grown
        };
        if inside_the_window(column_offsets(rect, shifted)) {
            control_hits.push(entry.name());
        }
    }

    let printed = table(
        &format!(
            "AC-4: left and right offsets from the marked rect expanded by margin_px \
             ({}), at min_line_spread {}",
            t.margin_px, t.min_line_spread
        ),
        &format!(
            "{:<30} {:>13} {:>13} {:>7} {:>7} {:>6}",
            "file", "produced", "expected", "low", "high", "window"
        ),
        &rows,
    );

    assert!(
        !rows.is_empty(),
        "AC-4 checked no entries at all; the corpus has no marked rect"
    );
    assert!(
        rows.len() - control_hits.len() >= CONTROL_FAILURES_REQUIRED,
        "AC-4's control: with the expected left edge shifted {LEFT_SHIFT_INWARD} px \
         inward, the window must reject at least {CONTROL_FAILURES_REQUIRED} of the \
         {} entries, or the window is wide enough to admit whatever the locator \
         returns and the assertion below means nothing. It still accepted {} of \
         them: {control_hits:?}\n\n{printed}",
        rows.len(),
        control_hits.len()
    );
    assert!(
        hits.len() >= COLUMN_HITS_REQUIRED,
        "AC-4: the page's left and right edges must land inside MC-019's window on \
         at least {COLUMN_HITS_REQUIRED} of the {} marked entries; {} did. The \
         misses are:\n{}\n\n{printed}",
        rows.len(),
        hits.len(),
        misses.join("\n")
    );
}

// --- AC-5: zero clips on the column axis ------------------------------------

/// AC-5, on the axis this story moves, with its control in the same test.
///
/// MC-026's `no_crop_clips_a_marked_page` above is the whole-rect statement
/// and it stays exactly as it is. This is the column-axis restatement, and it
/// is here because MC-027 is the story that moves those two edges and because
/// a clip is the worst defect this product has (MC-005 decision 13). It prints
/// the **slack** on each side - how many pixels of margin the crop has over
/// the mark - so a crop that is one pixel from clipping is visible before it
/// clips.
///
/// One entry was known to be at risk when the story was written and it was two
/// pixels: `2026-01-05 13_45_59.png`, located `1039..1505` against a manifest
/// mark of `1040..1510`. The user was shown the image and the measurement and
/// chose to **correct the manifest**: columns 1506..1510 are exactly 255 on
/// every one of the 1167 marked rows, so the mark contained five columns of
/// blank page margin against MC-018's own rule, and its `w` is now 466. The
/// story's `## Handoff` carries the column table that decided it. There is no
/// special case for that file here and no margin was widened.
///
/// The control is the second half: the same predicate, applied to each marked
/// rect pulled in by [`COLUMN_CLIP_INSET`] px on the left and the right, must
/// report a clip on every one of the twenty-one. Without it this test would
/// pass against a predicate that always says no.
///
/// MC-053: over twenty-six marked tuning entries now, and one of that story's
/// AC-1 regression tests at margin 3 - on post-MC-052 `main` it fails on
/// exactly its three.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_crop_clips_a_marked_page_on_the_column_axis() {
    let t = Tuning::default();
    let tmp = scratch();
    let mut rows = Vec::new();
    let mut clips = Vec::new();
    let mut control_missed = Vec::new();
    let mut cropped = 0usize;
    let mut got_all: Vec<(String, Option<Rect>)> = Vec::new();

    for (entry, mark) in marked() {
        let held = |rect: Rect| rect.x <= mark.x && rect.x + rect.w >= mark.x + mark.w;
        let got = produced(&entry, &t, &tmp);
        got_all.push((entry.name(), got));
        match got {
            Some(rect) => {
                cropped += 1;
                let ok = held(rect);
                // MC-064: a known clip is held to its pinned crop below instead.
                let known = known_clip(&entry.name()).is_some();
                rows.push(format!(
                    "{:<30} {:>13} {:>13} {:>7} {:>7} {:>7}",
                    entry.name(),
                    format!("{}..{}", rect.x, rect.x + rect.w - 1),
                    format!("{}..{}", mark.x, mark.x + mark.w - 1),
                    i64::from(mark.x) - i64::from(rect.x),
                    (i64::from(rect.x) + i64::from(rect.w))
                        - (i64::from(mark.x) + i64::from(mark.w)),
                    match (ok, known) {
                        (true, _) => "holds",
                        (false, true) => "KNOWN",
                        (false, false) => "CLIPS",
                    }
                ));
                if !ok && !known {
                    clips.push(format!(
                        "{}: cropped to columns {}..{}, which does not contain the \
                         marked {}..{}",
                        entry.name(),
                        rect.x,
                        rect.x + rect.w - 1,
                        mark.x,
                        mark.x + mark.w - 1
                    ));
                }
            }
            None => rows.push(format!(
                "{:<30} {:>13} {:>13} {:>7} {:>7} {:>7}",
                entry.name(),
                "not cropped",
                format!("{}..{}", mark.x, mark.x + mark.w - 1),
                "-",
                "-",
                "n/a"
            )),
        }

        let inset = Rect {
            x: mark.x + COLUMN_CLIP_INSET,
            w: mark.w - 2 * COLUMN_CLIP_INSET,
            ..mark
        };
        if held(inset) {
            control_missed.push(entry.name());
        }
    }

    let printed = table(
        &format!(
            "AC-5: column-axis containment over the {} marked tuning entries \
             ({cropped} cropped)",
            rows.len()
        ),
        &format!(
            "{:<30} {:>13} {:>13} {:>7} {:>7} {:>7}",
            "file", "produced", "marked", "left", "right", "verdict"
        ),
        &rows,
    );

    assert!(
        !rows.is_empty(),
        "AC-5 checked no entries at all; the corpus has no marked rect"
    );
    assert!(
        control_missed.is_empty(),
        "AC-5's control: a marked rect pulled in by {COLUMN_CLIP_INSET} px on the \
         left and the right clips the page by construction, so the containment \
         predicate must say so for all {} entries. It did not for \
         {control_missed:?}, which means the assertion below cannot detect a clip \
         either",
        rows.len()
    );
    let moved = known_clips_moved(&got_all);
    assert!(
        moved.is_empty(),
        "MC-064, MC-072: each known clip (KNOWN_CLIPS, the user's rulings of 2026-09-30 and 2026-10-02) must \
         still be cropped to exactly its pinned rect - if a fix moves one, take it \
         off the list and let this test judge it:\n{}\n\n{printed}",
        moved.join("\n")
    );
    assert!(
        clips.is_empty(),
        "AC-5: a crop whose left or right edge cuts into the page a person marked \
         is the worst defect this product has, and locating those two edges is \
         exactly what this story does. {} of {cropped} cropped entries clip \
         besides the {} known clips (MC-064, MC-072); the first is {}.\n\n{printed}\nall clips:\n{}",
        clips.len(),
        KNOWN_CLIPS.len(),
        clips[0],
        clips.join("\n")
    );
}
