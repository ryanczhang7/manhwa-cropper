//! MC-053 AC-1 to AC-3, carried unchanged into MC-055 (the fix; MC-053
//! became a spike): on the corpus, the page column keeps dark,
//! low-texture art, lets no page background back in, and moves nothing else.
//!
//! The page column locator (`flat::page_column`, MC-027) trimmed the outer
//! columns of dark, low-texture art as if they were page margin, because their
//! spread is below `min_line_spread` (8.0). Found on three held-out entries by
//! MC-049's held-out zero-clip check; the user moved them to `tuning` on
//! 2026-09-24, and widened their marks on 2026-09-25 on renders of every side
//! edge (MC-053's Open question 3; `docs/wiki/corpus.md`).
//!
//! # What is settled, what is mechanical
//!
//! * **Settled, read out and never re-derived**: MC-049's page-background
//!   predicate ([`PAGE_BACKGROUND_SHARE`], `uniform_tolerance` from the
//!   `Tuning` under test, over the mark's rows, the upper median); the user's
//!   marks; "both margins" (`margin_px: 3` and `margin_px: 0`, explicitly since
//!   MC-049 moved the default to 0); AC-2's
//!   floor of 19 on the metric control ([`METRIC_CONTROL_REQUIRED`]) and MC-049's
//!   seven mark errors ([`MC049_MARK_ERRORS`]).
//! * **Mechanical, pinned exactly**: AC-3's 23 x 2 rects
//!   ([`ORIGINALS_AT_BOTH_MARGINS`]), measured in RED on `c004d96`
//!   (post-MC-052 `main`); the lowered `min_line_spread` values AC-2's control
//!   names ([`LOWERED_SPREADS`]) and the columns they let in. That control
//!   runs **MC-027's rule frozen as a test-local stand-in**
//!   ([`mc027_page_column`]), not the shipped stage, so GREEN's fix cannot
//!   move it; the stand-in is earned against the shipped stage on the 23
//!   originals.
//!
//! # What the existing suites already say, and are not restated here
//!
//! AC-1's margin-3 half is `corpus.rs::no_crop_clips_a_marked_page`,
//! `corpus.rs::no_crop_clips_a_marked_page_on_the_column_axis` and
//! `corpus_accuracy.rs::no_corpus_crop_cuts_into_the_artwork_its_manifest_entry_marked`,
//! which fail on exactly the three and nothing else once they are `tuning`.
//! AC-4 is those and `corpus_viewport.rs::no_marked_tuning_crop_clips_its_mark_at_either_margin`,
//! over all 26. AC-6 is `corpus.rs`, `corpus_accuracy.rs` and the viewport
//! suites as they stand.
//!
//! # Running it
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it. It
//! sorts after `corpus.rs`, so the gate's floor - read off `corpus.rs` - does
//! not move.
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_page_column -- --ignored --nocapture
//! ```
//!
//! **Held-out discipline.** Every test here reads `tuning` entries only.
//! AC-5's held-out run is the orchestrator's, counts only, at GATES.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use std::path::Path;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::content::content_box;
use cropper_core::flat::textured_box;
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::{Luma, Rect, Tuning};
use cropper_engine::{Outcome, process_file};

// --- Constants read out of the story, never calibrated here -----------------

/// MC-049's page-background share: a column is page background when at least
/// this share of its pixels over the mark's rows lie within
/// `uniform_tolerance` of the column's median luma.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

/// The three entries MC-053 moved from `held-out` to `tuning`, whose dark,
/// low-texture art the locator cut.
const THE_THREE: [&str; 3] = [
    "2025-07-17 14_41_58.png",
    "2025-07-17 14_55_10.png",
    "Screenshot (73).png",
];

/// MC-056's three: the held-out entries MC-051 read per file, moved to
/// `tuning` by the user on 2026-09-29. They are **not** originals and join no
/// originals list here ([`ORIGINALS_AT_BOTH_MARGINS`] stays the 23), for the
/// reason MC-053's three do not: they were never part of the set AC-3 holds.
/// They do join every test over all marked `tuning` entries.
const MC056_THREE: [&str; 3] = [
    "2025-07-17 14_20_23.png",
    "2025-08-03 11_27_49.png",
    "Screenshot (68).png",
];

/// MC-062's sixteen: the marked entries among the 23 spent held-out entries
/// the user moved to `tuning` on 2026-09-30 ("move them to practice"), in
/// manifest order. Like MC-053's and MC-056's three they are **not**
/// originals and join no originals list here; they join every test over all
/// marked `tuning` entries.
const MC062_SIXTEEN: [&str; 16] = [
    "2025-03-04 11_09_29.png",
    "2025-03-07 00_41_10.png",
    "2025-03-07 01_10_37.png",
    "2025-08-03 11_13_19.png",
    "2025-08-03 20_54_19.png",
    "2025-08-04 23_24_37.png",
    "2025-08-07 15_07_56.png",
    "2025-08-07 15_47_10.png",
    "2025-08-07 15_57_50.png",
    "2025-09-29 14_33_15.png",
    "Screenshot (56).png",
    "Screenshot (59).png",
    "Screenshot (1720).png",
    "Screenshot (3605).png",
    "Screenshot (3606).png",
    "Screenshot (3625).png",
];

/// MC-064's four: the fresh entries MC-063 read per file, moved to `tuning` by
/// the user's ruling of 2026-09-30 ("Write up and file"), in manifest order.
/// Like the others above they are **not** originals and join no originals list
/// here; they join every test over all marked `tuning` entries.
const MC064_FOUR: [&str; 4] = [
    "2025-03-06 12_48_06.png",
    "2025-03-16 22_47_44.png",
    "2025-08-07 01_13_55.png",
    "2025-12-08 17_22_50.png",
];

/// MC-068's 21: the rest of MC-062's fresh draw, spent by MC-063's one scored
/// run and moved to `tuning` by MC-068 (its AC-4), in manifest order. Like
/// the others above they are **not** originals and join no originals list
/// here; they join every test over all marked `tuning` entries.
const MC068_TWENTY_ONE: [&str; 21] = [
    "2025-03-04 14_28_54.png",
    "2025-03-06 02_01_06.png",
    "2025-03-07 16_07_21.png",
    "2025-03-18 12_37_27.png",
    "2025-03-23 23_56_16.png",
    "2025-03-24 22_44_31.png",
    "2025-03-25 22_06_29.png",
    "2025-07-17 23_45_48.png",
    "2025-07-21 08_26_37.png",
    "2025-07-21 17_47_22.png",
    "2025-08-04 17_10_16.png",
    "2025-08-07 11_20_12.png",
    "2025-08-07 14_33_43.png",
    "2025-10-23 11_31_40.png",
    "2025-11-12 17_43_44.png",
    "2025-12-09 00_00_17.png",
    "Screenshot (9).png",
    "Screenshot (2368).png",
    "Screenshot (2461).png",
    "Screenshot (2486).png",
    "Screenshot (2669).png",
];

/// MC-069's 14: the screenshots the user's run of the app answered
/// `Ambiguous` on, added to `tuning` by MC-069 (its AC-1), in manifest order.
/// Like the others above they are **not** originals and join no originals
/// list here; they join every test over all marked `tuning` entries.
const MC069_FOURTEEN: [&str; 14] = [
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
    "Screenshot (2705).png",
];

/// MC-072's four: the fresh entries MC-071 read per file (`n02`, `n06`,
/// `n05`, `n13`), moved to `tuning` by the user's answer of 2026-10-02 ("all
/// recommended"), in manifest order. Like the others above they are **not**
/// originals and join no originals list here; they join every test over all
/// marked `tuning` entries.
const MC072_FOUR: [&str; 4] = [
    "2025-03-07 00_05_58.png",
    "2025-03-13 12_01_01.png",
    "2025-11-01 12_34_31.png",
    "Screenshot (2507).png",
];

/// MC-077's 11: the rest of MC-068's fresh draw, spent by MC-071's one scored
/// run and moved to `tuning` by MC-077 (its AC-4), in manifest order. Like
/// the others above they are **not** originals and join no originals list
/// here; they join every test over all marked `tuning` entries. On a scratch
/// copy with the move applied (`4272252`, release, MC-077's RED) every one is
/// cropped at margin 0 to exactly its mark, no side lets page background in,
/// and every mark edge reads as art (largest share 0.867, `Screenshot
/// (1460).png` on both sides, under 0.95), so no list here gains a name.
const MC077_ELEVEN: [&str; 11] = [
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

/// MC-079's one: `n05` of MC-077's fresh draw, `2025-10-26 12_13_16.png`,
/// the one entry MC-078 read per file, moved to `tuning` by MC-079 (its
/// AC-1). Like the others above it is **not** an original and joins no
/// originals list here; it joins every test over all marked `tuning` entries.
/// On a scratch copy with the move applied (`6343029`, release, MC-079's RED)
/// its margin-0 crop `974,167 599x1233` keeps no column outside its mark
/// `973,167 600x1233` (it clips one on the left: a known clip in
/// `tests/corpus.rs` and elsewhere), so no side lets page background in; and
/// both mark edges read as art - first column 973 share 0.922, last column
/// 1572 share 0.281, under 0.95 - so no list here gains its name, and no
/// background-edge list like [`MC072_BACKGROUND_EDGES`] is needed for it.
///
/// MC-080 re-marked it to `974,167 599x1233`, by the user's ruling of
/// 2026-10-04 (*"Box starts at 974"*: column 973 is the dark seam beside the
/// art). Its crop at margin 0 now equals the mark, and its new first column,
/// 974, is art (share 0.161 within 10 of its own median over the mark's rows,
/// MC-080 `## Context`), so it still joins no background-edge list here and
/// no `RULED_ART_EDGES`. It stays on this list, which records where it came
/// from, not an exception.
const MC079_ONE: [&str; 1] = ["2025-10-26 12_13_16.png"];

/// MC-081's seven: the Eleceed screenshots where the app cropped the wrong
/// window, `e01`..`e07`, added to `tuning` by MC-081 (its AC-1, AC-2), in
/// manifest order. Like the others above they are **not** originals and join
/// no originals list here; they join every test over all marked `tuning`
/// entries. On a scratch copy with the seven added (`ebdedd7`, release,
/// MC-081's RED) all seven crops clip their marks (known clips in
/// `tests/corpus.rs` and elsewhere): six keep only the right-hand (YouTube)
/// window, wholly right of their marks, and none of those columns reads as
/// page background over the mark's rows; `e03` starts at column 727 against
/// a mark that starts at 610 and ends where the mark ends, so it keeps no
/// column outside its mark. AC-2 here passes with all seven read and
/// unlisted, and [`KNOWN_BACKGROUND_SIDES`] stays empty. All fourteen
/// mark edges read as art (shares 0.024 to 0.679, under 0.95), so no
/// background-edge list like [`MC072_BACKGROUND_EDGES`] is needed for them.
///
/// MC-082 (the six) and MC-083 (`e03`) moved their crops to the marks; on
/// MC-083's scratch trial this suite passes unchanged with all seven read. The
/// list records where they came from, not an exception, and stays.
const MC081_SEVEN: [&str; 7] = [
    "2025-03-16 22_56_00.png",
    "2025-03-07 00_20_37.png",
    "2025-03-07 01_02_31.png",
    "2025-03-16 22_48_01.png",
    "2025-03-16 22_51_37.png",
    "2025-03-16 22_51_49.png",
    "2025-03-16 22_54_27.png",
];

/// MC-084's nine: the rest of MC-077's fresh draw, spent by MC-078's one
/// scored run and moved to `tuning` by MC-084 (its AC-4), in manifest order.
/// Like the others above they are **not** originals and join no originals
/// list here; they join every test over all marked `tuning` entries. On a
/// scratch copy with the move applied (`864e717`, release, MC-084's RED) no
/// side lets page background in, and all eighteen mark edges read as art
/// (largest share 0.925, `2025-07-18 21_55_50.png`'s first column 997, under
/// 0.95), so no list here gains a name.
const MC084_NINE: [&str; 9] = [
    "2025-10-08 13_14_05.png",
    "2025-03-21 11_19_03.png",
    "2025-08-05 16_30_27.png",
    "2025-07-18 16_28_07.png",
    "2025-03-18 23_33_50.png",
    "2025-08-05 12_21_01.png",
    "2025-07-18 21_55_50.png",
    "2025-12-04 22_40_23.png",
    "2025-03-12 23_13_21.png",
];

/// Whether `name` is one of the marked `tuning` entries added after the 23
/// originals: MC-053's three, MC-056's three, MC-062's sixteen, MC-064's
/// four, MC-068's 21, MC-069's 14, MC-072's four, MC-077's 11, MC-079's one,
/// MC-081's seven or MC-084's nine.
fn is_not_an_original(name: &str) -> bool {
    THE_THREE.contains(&name)
        || MC056_THREE.contains(&name)
        || MC062_SIXTEEN.contains(&name)
        || MC064_FOUR.contains(&name)
        || MC068_TWENTY_ONE.contains(&name)
        || MC069_FOURTEEN.contains(&name)
        || MC072_FOUR.contains(&name)
        || MC077_ELEVEN.contains(&name)
        || MC079_ONE.contains(&name)
        || MC081_SEVEN.contains(&name)
        || MC084_NINE.contains(&name)
}

/// MC-064, the user's ruling of 2026-09-30 (its Open question 1, *"List them
/// as known"*): AC-2's known page-background sides at margin 0, `(file, side,
/// [x, y, w, h] of the crop at margin 0)`. `2025-03-16 22_47_44.png`'s crop
/// keeps the browser's scrollbar, running to column 2556 against a mark that
/// ends at 1167, and columns it keeps beyond the mark on the right read as
/// page background. The crop is **measured, not chosen**: one run of
/// `process_file` on `43e8e61` (release; crates unchanged since `d2876f5`) in
/// MC-064's RED, on a scratch copy with only the four `split` values changed.
/// **Exact in both directions**: AC-2 fails if any other side lets page
/// background in, if this one stops doing so, or if its crop is not the pin.
///
/// MC-066 took `f13` off (its AC-2 and AC-3): its crop ends inside the
/// reader's window, and AC-2 judges its sides like every other entry's. The
/// list is empty, and stays exact in both directions.
const KNOWN_BACKGROUND_SIDES: [(&str, Side, [u32; 4]); 0] = [];

/// MC-056, the user's ruling of 2026-09-29 (its Open question 3): the mark
/// edges the user ruled **art** after a close-up although the predicate reads
/// them as page background, `(file, side)`. `2025-07-17 14_20_23.png`'s mark
/// was widened to column 962, where the picture's own flat black background
/// starts (the page's grey ends at 961); flat black is uniform, so the
/// predicate reads it as background (share 1.000 in MC-056's RED). **Exact in
/// both directions**: the metric control fails if any other edge outside
/// MC-049's seven reads as background, and fails if this one stops reading so.
///
/// MC-065 adds `2025-08-07 01_13_55.png`'s right edge (`f20`), by the user's
/// ruling of 2026-09-30 (*"agreed, box stands"*, MC-065 `## Notes`): column
/// 1521 is the page's own white paper, one flat shade (share 0.988 in MC-064's
/// RED), and the site's dark background starts at 1522. It was MC-064's
/// `MC064_BACKGROUND_EDGES`, the list for an edge nobody had looked at; that
/// list held only this edge, and MC-065 AC-2 deletes it.
///
/// MC-074 adds both edges of `2025-03-07 00_05_58.png` (`n02`), by the user's
/// ruling of 2026-10-02 (*"Box stands"*, MC-074 `## Context`), as for `f20`:
/// its drawn panels sit on flat white page paper (band median 255, share
/// 1.000), and the mark's first column 643 and last column 1175 are that
/// paper, the page's own, with the site's background (11) starting at 642
/// and 1176. They come off [`MC072_BACKGROUND_EDGES`]. In manifest order.
const RULED_ART_EDGES: [(&str, Side); 4] = [
    ("2025-07-17 14_20_23.png", Side::Left),
    ("2025-08-07 01_13_55.png", Side::Right),
    ("2025-03-07 00_05_58.png", Side::Left),
    ("2025-03-07 00_05_58.png", Side::Right),
];

/// MC-072: the mark edges of the four fresh entries MC-071 read per file that
/// the predicate reads as page background, `(file, side)`, in manifest order -
/// MC-064's `MC064_BACKGROUND_EDGES` shape, **the list for an edge nobody has
/// looked at**. Measured in MC-072's RED on `0f9c579` (release), on a scratch
/// copy with only the four `split` values changed: `n02`
/// (`2025-03-07 00_05_58.png`) first column 643 and last column 1175 both
/// share 1.000; `n05` (`2025-11-01 12_34_31.png`) first column 1006 share
/// 0.982 (its last column, 1537, reads 0.277). `n06` and `n13` read as art on
/// both edges. These are **not** ruled art: whether each is the art's own flat
/// edge or a mark error is the user's question, put by the fix story that
/// owns the entry (MC-072 `## Out of scope`: "MC-073 owns that question" for
/// `n05`'s column 1006). **Exact in both directions**: the metric control fails
/// if any other edge reads as background, and fails if one of these stops
/// reading so - so the story that rules on one moves it to
/// [`RULED_ART_EDGES`] or corrects the mark, and cannot leave it here.
///
/// MC-073 took `n05`'s left edge off (its AC-2) by correcting the mark: the
/// user ruled on 2026-10-02 *"Box starts at 1007"* - column 1006 is the dark
/// seam beside the art, as on `Screenshot (3605).png` and
/// `2025-10-20 15_37_25.png`, not art - so the mark is `1007,167 531x1233`.
/// Its new first column, 1007, is art (share 0.376 within 10 of its own
/// median over the mark's rows, MC-073 `## Context`), so it is **not** added
/// to [`RULED_ART_EDGES`] either: the predicate reads it as art, like every
/// other edge.
///
/// MC-074 took `n02`'s two edges off (its AC-1) to [`RULED_ART_EDGES`]: the
/// user ruled on 2026-10-02 *"Box stands"* - its flat white page paper is
/// page, so the mark stands and its edges, which read as page background, are
/// ruled page. The list is empty: every edge it held has been ruled on, and
/// it stays exact in both directions.
const MC072_BACKGROUND_EDGES: [(&str, Side); 0] = [];

/// MC-056, AC-4 as amended on 2026-09-29 (the user's ruling on Open question
/// 2): the one marked `tuning` entry that is not cropped - the detector flags
/// it `Ambiguous` - named as AC-2's only known exception. **Exact in both
/// directions**: AC-2 fails if any other marked `tuning` entry is not cropped,
/// and fails if this one is cropped, so the story that fixes it has to empty
/// this list.
///
/// **MC-069 swaps it** (its AC-3 and AC-4, the user's answers of 2026-10-01):
/// `2025-07-17 14_20_23.png` crops now - its close call is the browser
/// scrollbar, outside its crop - and is judged like every other entry, and
/// `Screenshot (2705).png`, whose close call lies inside its crop, is the
/// known exception until MC-070.
///
/// **MC-070 empties it** (its AC-1): `(2705)` crops, to its mark at margin 0,
/// and AC-2 judges its sides like every other entry's. Every marked `tuning`
/// entry must be cropped; the list stays exact in both directions.
const KNOWN_NOT_CROPPED: [&str; 0] = [];

/// AC-2's control on the metric: on at least this many of the 26 marked
/// `tuning` entries the predicate calls the mark's own first **and** last
/// columns not page background. The story's number: 26 less MC-049's seven.
const METRIC_CONTROL_REQUIRED: usize = 19;

/// MC-049's seven mark errors (MC-049 `## Context`): the marks whose own
/// outermost column is flat page background, `(file, side)`. MC-049 corrects
/// them; until it does, they are the only sides AC-2's metric control may
/// call page background.
const MC049_MARK_ERRORS: [(&str, Side); 7] = [
    ("2025-10-14 23_30_20.png", Side::Right),
    ("Screenshot (2630).jpg", Side::Right),
    ("Screenshot (67).png", Side::Right),
    ("Screenshot (70).jpg", Side::Right),
    ("Screenshot (3187).png", Side::Right),
    ("2026-01-05 13_33_41.png", Side::Left),
    ("Screenshot (2708).jpg", Side::Left),
];

/// AC-2's control, the fix it exists to rule out: `min_line_spread` lowered
/// globally, and the page-background columns each lets in outside the mark on
/// `Screenshot (2630).jpg`'s left, at margin 0. The Lead PO's probe measured
/// 954 at 5.0 and 953..=954 at 1.8 on `26eddcb`; RED re-measured both on
/// `c004d96`, identical. 1.8 is the value that clears all four clipped sides
/// of the three under the old marks.
const LOWERED_SPREADS: [(f32, &[u32]); 2] = [(5.0, &[954]), (1.8, &[953, 954])];

/// The entry and side [`LOWERED_SPREADS`] names.
const LET_IN_ON: (&str, Side) = ("Screenshot (2630).jpg", Side::Left);

/// AC-3: the whole crop of each of the 23 originals - the marked `tuning`
/// entries on post-MC-052 `main` - `(file, [x, y, w, h] at margin 3, [x, y,
/// w, h] at margin 0)`, as `process_file` produces it on `c004d96` (release),
/// measured in MC-053's RED. The 21 are MC-052's `ORIGINALS_BEFORE`
/// unchanged; the last two are MC-052's split-screen shots.
const ORIGINALS_AT_BOTH_MARGINS: [(&str, [u32; 4], [u32; 4]); 23] = [
    // MC-076 re-pins both WebPs (its AC-2, the user's ruling of 2026-10-02;
    // MC-048 AC-3 reversed): the crop starts at row 115, below the browser
    // chrome, columns kept exactly, bottom 1400 as the Lead PO's scratch trial
    // measured it (mechanical; `corpus_tuning_crops_unmoved.rs` says why).
    // They were `[950, 15, 646, 1425]` / `[953, 18, 640, 1422]` and
    // `[1003, 15, 539, 1425]` / `[1006, 18, 533, 1422]`.
    (
        "2025-08-05 00_11_13.webp",
        [950, 115, 646, 1285],
        [953, 115, 640, 1285],
    ),
    (
        "2025-08-05 00_11_27.webp",
        [1003, 115, 539, 1285],
        [1006, 115, 533, 1285],
    ),
    (
        "2025-10-14 23_29_06.png",
        [1000, 167, 546, 1233],
        [1003, 167, 540, 1233],
    ),
    (
        "2025-10-14 23_30_20.png",
        [1030, 167, 486, 1233],
        [1033, 167, 480, 1233],
    ),
    (
        "2025-10-20 15_37_25.png",
        [1004, 167, 538, 1233],
        [1007, 167, 532, 1233],
    ),
    (
        "2026-01-05 13_33_41.png",
        [1071, 167, 404, 1233],
        [1074, 167, 398, 1233],
    ),
    (
        "2026-01-05 13_45_59.png",
        [1036, 167, 473, 1233],
        [1039, 167, 467, 1233],
    ),
    (
        "2026-01-05 13_49_39.png",
        [1036, 167, 473, 1233],
        [1039, 167, 467, 1233],
    ),
    (
        "Screenshot (67).png",
        [1007, 167, 531, 1225],
        [1010, 167, 525, 1225],
    ),
    (
        "Screenshot (70).jpg",
        [1007, 167, 531, 1225],
        [1010, 167, 525, 1225],
    ),
    (
        "Screenshot (75).png",
        [1070, 167, 406, 1225],
        [1073, 167, 400, 1225],
    ),
    (
        "Screenshot (93).jpg",
        [1136, 167, 273, 1225],
        [1139, 167, 267, 1225],
    ),
    (
        "Screenshot (103).jpg",
        [1070, 167, 406, 1225],
        [1073, 167, 400, 1225],
    ),
    (
        "Screenshot (1661).png",
        [1070, 133, 406, 1259],
        [1073, 133, 400, 1259],
    ),
    (
        "Screenshot (2582).jpg",
        [1003, 133, 539, 1259],
        [1006, 133, 533, 1259],
    ),
    (
        "Screenshot (2630).jpg",
        [952, 133, 642, 1259],
        [955, 133, 636, 1259],
    ),
    (
        "Screenshot (2698).jpg",
        [950, 133, 645, 1259],
        [953, 133, 639, 1259],
    ),
    (
        "Screenshot (2708).jpg",
        [1071, 133, 405, 1259],
        [1074, 133, 399, 1259],
    ),
    (
        "Screenshot (2744).jpg",
        [945, 133, 654, 1259],
        [948, 133, 648, 1259],
    ),
    (
        "Screenshot (3187).png",
        [981, 137, 582, 1255],
        [984, 137, 576, 1255],
    ),
    // MC-067 AC-2, the user's ruling of 2026-10-01 ("Let 3538 grow"): the
    // crop grows by its 2-column dark fringe on each side, reversing MC-053's
    // preference for 3538 (its Open question 5). It was `[972, 137, 602,
    // 1255]` / `[975, 137, 596, 1255]`. Ruled, not measured.
    (
        "Screenshot (3538).png",
        [970, 137, 606, 1255],
        [973, 137, 600, 1255],
    ),
    (
        "2025-03-06 01_22_45.png",
        [640, 115, 539, 1259],
        [643, 115, 533, 1259],
    ),
    (
        "2025-03-07 00_58_06.png",
        [667, 115, 486, 1284],
        [670, 115, 480, 1284],
    ),
];

/// A side of a crop or a mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

// --- Harness ----------------------------------------------------------------

/// The marked `tuning` entries, with their marks, in manifest order. **Never
/// `held-out`**: `docs/wiki/corpus.md` rules that a held-out entry is scored
/// once, and AC-5's run is the orchestrator's.
fn marked() -> Vec<(CorpusEntry, Rect)> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .filter_map(|entry| match entry.expect {
            Expect::Rect(rect) => Some((entry, rect)),
            Expect::Flag => None,
        })
        .collect()
}

/// Both margins, as the story defines them: 3 and 0. The 3 is written out
/// because MC-049 moved `Tuning::default().margin_px` to 0, and the pins were
/// measured at 3.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
        Tuning {
            margin_px: 0,
            ..Tuning::default()
        },
    ]
}

/// The luma plane `process_file` sees: the engine's decoder and conversion.
fn luma(path: &Path) -> Luma {
    let bytes =
        std::fs::read(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// The rect `process_file` cropped `entry` to at `t`, or the outcome as text.
fn crop_at(entry: &CorpusEntry, tmp: &tempfile::TempDir, t: &Tuning) -> Result<Rect, String> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Ok(rect),
        Outcome::Flagged { reason, .. } => Err(format!("Flagged {reason:?}")),
        Outcome::Failed { error } => Err(format!("Failed {error}")),
    }
}

/// MC-049's share: of column `x`'s pixels over `mark`'s rows, the share
/// within `tolerance` of the column's upper median.
fn background_share(img: &Luma, x: u32, mark: Rect, tolerance: u8) -> f64 {
    let mut values: Vec<u8> = (mark.y..mark.y + mark.h)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect();
    values.sort_unstable();
    let median = values[values.len() / 2];
    let near = values
        .iter()
        .filter(|&&v| v.abs_diff(median) <= tolerance)
        .count();
    near as f64 / values.len() as f64
}

/// MC-049's page-background predicate.
fn is_page_background(img: &Luma, x: u32, mark: Rect, t: &Tuning) -> bool {
    background_share(img, x, mark, t.uniform_tolerance) >= PAGE_BACKGROUND_SHARE
}

/// The last column of `r`, inclusive.
fn right(r: Rect) -> u32 {
    r.x + r.w - 1
}

/// The crop's page-background columns outside the mark, on the left and on
/// the right.
fn background_outside(img: &Luma, crop: Rect, mark: Rect, t: &Tuning) -> (Vec<u32>, Vec<u32>) {
    let left = (crop.x..mark.x.min(right(crop) + 1))
        .filter(|&x| is_page_background(img, x, mark, t))
        .collect();
    let right_side = ((right(mark) + 1).max(crop.x)..=right(crop))
        .filter(|&x| is_page_background(img, x, mark, t))
        .collect();
    (left, right_side)
}

/// One side of one crop that lets page background in: `(file, side, the
/// page-background columns outside the mark)`.
type LetIn = (String, Side, Vec<u32>);

/// AC-2 over every marked `tuning` entry at `t`: `(sides checked, failing
/// sides, entries not cropped)`.
fn page_background_let_in(
    entries: &[(CorpusEntry, Rect)],
    t: &Tuning,
    tmp: &tempfile::TempDir,
) -> (usize, Vec<LetIn>, Vec<String>) {
    let mut sides = 0usize;
    let mut failing = Vec::new();
    let mut not_cropped = Vec::new();
    for (entry, mark) in entries {
        match crop_at(entry, tmp, t) {
            Ok(crop) => {
                let img = luma(&entry.path);
                let (left, right_side) = background_outside(&img, crop, *mark, t);
                sides += 2;
                if !left.is_empty() {
                    failing.push((entry.name(), Side::Left, left));
                }
                if !right_side.is_empty() {
                    failing.push((entry.name(), Side::Right, right_side));
                }
            }
            Err(what) => not_cropped.push(format!("{}: {what}", entry.name())),
        }
    }
    (sides, failing, not_cropped)
}

/// A table printed under `--nocapture` and returned for the assertion message.
fn table(title: &str, rows: &[String]) -> String {
    let out = format!("{title}\n{}\n", rows.join("\n"));
    println!("{out}");
    out
}

// --- AC-1: the bug, reproduced ----------------------------------------------

/// AC-1. On the three, at both margins, the crop contains its mark on the
/// left and on the right.
///
/// On `c004d96`, against the widened marks, it cuts - margin 3 / margin 0 -
/// `14_41_58` left 6 / 9 and right 11 / 14, `14_55_10` right 21 / 24, and
/// `(73)` left 10 / 13: four sides of three entries at each margin. Against
/// the marks as they were before the widening it was 3 / 6, 5 / 8, 2 / 5 and
/// 6 / 9, the story's `26eddcb` figures exactly.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn on_the_three_dark_art_entries_the_crop_keeps_the_art_on_the_left_and_right_at_both_margins() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let entries = marked();
    let mut rows = Vec::new();
    let mut cut = Vec::new();
    let mut seen = Vec::new();
    for t in both_margins() {
        for (entry, mark) in entries
            .iter()
            .filter(|(entry, _)| THE_THREE.contains(&entry.name().as_str()))
        {
            if t.margin_px == both_margins()[0].margin_px {
                seen.push(entry.name());
            }
            match crop_at(entry, &tmp, &t) {
                Ok(crop) => {
                    let left = i64::from(crop.x) - i64::from(mark.x);
                    let right_cut = i64::from(right(*mark)) - i64::from(right(crop));
                    rows.push(format!(
                        "{:<26} m{} crop {}..={}  mark {}..={}  cut left {} right {}",
                        entry.name(),
                        t.margin_px,
                        crop.x,
                        right(crop),
                        mark.x,
                        right(*mark),
                        left.max(0),
                        right_cut.max(0)
                    ));
                    if left > 0 {
                        cut.push(format!(
                            "{} at margin_px {}: cuts {left} columns of art on the left",
                            entry.name(),
                            t.margin_px
                        ));
                    }
                    if right_cut > 0 {
                        cut.push(format!(
                            "{} at margin_px {}: cuts {right_cut} columns of art on the right",
                            entry.name(),
                            t.margin_px
                        ));
                    }
                }
                Err(what) => {
                    cut.push(format!(
                        "{} at margin_px {}: {what}",
                        entry.name(),
                        t.margin_px
                    ));
                    rows.push(format!("{:<26} m{} {what}", entry.name(), t.margin_px));
                }
            }
        }
    }
    let printed = table("MC-053 (MC-055) AC-1: the three, left and right", &rows);
    assert_eq!(
        seen,
        THE_THREE.map(String::from).to_vec(),
        "AC-1 must reach all three as marked tuning entries, in manifest order"
    );
    assert!(
        cut.is_empty(),
        "MC-053 (MC-055) AC-1: the page column locator must keep the dark, low-texture art \
         the user marked, on the left and on the right, at margin_px 3 and 0.\n{}\n\n{printed}",
        cut.join("\n")
    );
}

// --- AC-2: no page background is let back in -------------------------------

/// AC-2. Over all 26 marked `tuning` entries at margin 0, no crop column that
/// lies outside the mark is page background, on either side: 0 of 52 sides.
///
/// Holds on `c004d96` (0 of 52, re-measured in RED over MC-052's two as well).
/// It is the guard on the fix: its two controls are the next two tests.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn at_margin_0_no_crop_column_outside_the_mark_is_page_background() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let t = Tuning {
        margin_px: 0,
        ..Tuning::default()
    };
    let entries = marked();
    let (sides, all_failing, not_cropped) = page_background_let_in(&entries, &t, &tmp);
    // MC-064: the known page-background sides are set apart, exactly, below.
    let is_known = |name: &str, side: Side| {
        KNOWN_BACKGROUND_SIDES
            .iter()
            .any(|(file, s, _)| *file == name && *s == side)
    };
    let known_read: Vec<(String, Side)> = all_failing
        .iter()
        .filter(|(name, side, _)| is_known(name, *side))
        .map(|(name, side, _)| (name.clone(), *side))
        .collect();
    let failing: Vec<&LetIn> = all_failing
        .iter()
        .filter(|(name, side, _)| !is_known(name, *side))
        .collect();
    let shown: Vec<String> = failing
        .iter()
        .map(|(name, side, cols)| format!("{name} {side:?}: page-background columns {cols:?}"))
        .collect();
    assert_eq!(
        entries.len(),
        116,
        "MC-053 (MC-055) AC-2 is over the 116 marked tuning entries (26 until MC-056 moved \
         three, 29 until MC-062 moved sixteen, 45 until MC-064 moved four, 49 until MC-068 \
         moved 21, 70 until MC-069 added 14, 84 until MC-072 moved four, 88 until MC-077 \
         moved 11, 99 until MC-079 moved one, 100 until MC-081 added seven, 107 until \
         MC-084 moved nine)"
    );
    assert_eq!(
        known_read,
        KNOWN_BACKGROUND_SIDES
            .map(|(name, side, _)| (name.to_string(), side))
            .to_vec(),
        "MC-064: every known page-background side (KNOWN_BACKGROUND_SIDES, the user's \
         ruling of 2026-09-30) must still let page background in at margin_px 0 - if a \
         fix clears one, take it off the list. `left` is measured"
    );
    let moved: Vec<String> = KNOWN_BACKGROUND_SIDES
        .iter()
        .filter_map(|&(file, _, [x, y, w, h])| {
            let pinned = Rect { x, y, w, h };
            let entry = entries.iter().find(|(entry, _)| entry.name() == file);
            match entry.map(|(entry, _)| crop_at(entry, &tmp, &t)) {
                Some(Ok(crop)) if crop == pinned => None,
                got => Some(format!("{file}: {got:?}, pinned {pinned:?}")),
            }
        })
        .collect();
    assert!(
        moved.is_empty(),
        "MC-064: each entry with a known page-background side must still be cropped to \
         exactly its pin in KNOWN_BACKGROUND_SIDES at margin_px 0:\n{}",
        moved.join("\n")
    );
    // MC-056: exact against KNOWN_NOT_CROPPED, by name. `not_cropped` rows are
    // `name: outcome`; the name is everything before the first ": ".
    let not_cropped_names: Vec<&str> = not_cropped
        .iter()
        .map(|row| row.split(": ").next().unwrap_or(row))
        .collect();
    assert_eq!(
        not_cropped_names,
        KNOWN_NOT_CROPPED.to_vec(),
        "MC-053 (MC-055) AC-2: every marked tuning entry must be cropped - \
         KNOWN_NOT_CROPPED is empty since MC-070 cropped (2705). Not cropped: \
         {not_cropped:?}"
    );
    assert_eq!(
        sides,
        2 * (entries.len() - KNOWN_NOT_CROPPED.len()),
        "MC-053 (MC-055) AC-2 checks both sides of every cropped entry"
    );
    assert_eq!(
        sides, 232,
        "MC-053 (MC-055) AC-2 checks 232 sides: both sides of all 116 entries (88 until \
         MC-064 moved four, 96 until MC-068 moved 21, 138 until MC-069 added 14 and \
         swapped MC-056's exception for its own, 166 until MC-070 cropped (2705), 168 \
         until MC-072 moved four, 176 until MC-077 moved 11, 198 until MC-079 moved one, \
         200 until MC-081 added seven, 214 until MC-084 moved nine)"
    );
    assert!(
        failing.is_empty(),
        "MC-053 (MC-055) AC-2: at margin_px 0 no column the crop keeps outside the mark may \
         be page background (share >= {PAGE_BACKGROUND_SHARE} within {} of the \
         column's median, over the mark's rows), except MC-064's known sides \
         (KNOWN_BACKGROUND_SIDES). {} of {sides} sides let it in:\n{}",
        t.uniform_tolerance,
        failing.len(),
        shown.join("\n")
    );
}

// --- MC-027's rule, frozen as a test-local stand-in -------------------------
//
// AC-2's control rules out one fix, "lower `min_line_spread` globally", and it
// must go on ruling it out after GREEN has changed `flat::page_column`. So it
// does not run the shipped stage. It runs **MC-027's rule as it stood on
// `c004d96`**, re-implemented here: each column's spread over the central band
// of the rect's rows, the widest run at or above `min_line_spread`, and the
// interior rule (a run reaching either end of the rect leaves it alone). The
// stages before it are the shipped ones, `trim_uniform`, `content_box`,
// `trim_within` and `textured_box`, composed as `decide.rs` composes them;
// MC-053 does not change them. At margin 0 the crop's columns are the page
// column's, because the viewport stage moves rows only. MC-052's
// `either_side_flat` is the precedent.
//
// What earns it: `mc027s_rule_frozen_here_reproduces_the_shipped_page_column_on_the_originals`
// below, which must hold before GREEN and after it. In RED it also reproduced
// the three, 26 of 26 (the story's handoff).

/// The mean absolute deviation of `values` about their own mean, as one exact
/// rational rounded to `f32`: MC-027's arithmetic (`edges::spread_within`).
fn mc027_spread(values: impl Iterator<Item = u8> + Clone) -> f32 {
    let n = values.clone().count() as u64;
    if n == 0 {
        return 0.0;
    }
    let total: u64 = values.clone().map(u64::from).sum();
    let deviations: u64 = values.map(|v| (n * u64::from(v)).abs_diff(total)).sum();
    (deviations as f64 / (n * n) as f64) as f32
}

/// The widest run of `spread` at or above `threshold`, bounds inclusive, the
/// first of equal width winning.
fn mc027_widest_run(spread: &[f32], threshold: f32) -> Option<(usize, usize)> {
    let mut widest: Option<(usize, usize)> = None;
    let mut open: Option<usize> = None;
    let close = |start: usize, end: usize, widest: &mut Option<(usize, usize)>| {
        if widest.is_none_or(|(a, b)| end - start > b - a) {
            *widest = Some((start, end));
        }
    };
    for (i, &value) in spread.iter().enumerate() {
        match (value >= threshold, open) {
            (true, None) => open = Some(i),
            (false, Some(start)) => {
                close(start, i - 1, &mut widest);
                open = None;
            }
            _ => {}
        }
    }
    if let Some(start) = open {
        close(start, spread.len() - 1, &mut widest);
    }
    widest
}

/// MC-027's page column: `within`, narrowed on the columns to the widest run
/// at or above `t.min_line_spread` over the central `t.central_band_fraction`
/// of its rows, or `within` unchanged where there is no run or it reaches an
/// end.
fn mc027_page_column(img: &Luma, within: Rect, t: &Tuning) -> Rect {
    let rows = (f64::from(within.h) * f64::from(t.central_band_fraction)).floor() as u32;
    let rows = rows.clamp(u32::from(within.h > 0), within.h);
    let top = within.y + (within.h - rows) / 2;
    let spread: Vec<f32> = (within.x..within.x + within.w)
        .map(|x| mc027_spread((top..top + rows).map(|y| img.data[(y * img.width + x) as usize])))
        .collect();
    match mc027_widest_run(&spread, t.min_line_spread) {
        Some((first, last)) if first > 0 && last + 1 < spread.len() => Rect {
            x: within.x + first as u32,
            w: (last - first + 1) as u32,
            ..within
        },
        _ => within,
    }
}

/// The page column the pipeline would locate with MC-027's rule in place of
/// the shipped `page_column`, at `t`.
fn mc027_column(img: &Luma, t: &Tuning) -> Rect {
    let first = trim_uniform(img, t).expect("a corpus page is not one flat colour");
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t).expect("the content box is not one flat colour");
    mc027_page_column(img, textured_box(img, second, t), t)
}

/// A page column's `(x, w)`.
type XW = (u32, u32);

/// MC-067 AC-2, the user's ruling of 2026-10-01 ("Let 3538 grow"): the one
/// original whose shipped page column is **not** MC-027's widest run, `(file,
/// the stand-in's (x, w), the shipped (x, w))` at margin 0. MC-027's rule ends
/// `Screenshot (3538).png`'s page column on the last textured column each
/// side, 975..1570; the shipped crop keeps the 2-column dark fringe beyond it
/// on each side, 973..1572, as the user ruled. **Exact in both directions**:
/// the stand-in must still give its value here and the pipeline must crop to
/// exactly the ruled one, so this is 3538's re-pin and not a loosening.
const STAND_IN_DIFFERS_ON: [(&str, XW, XW); 1] =
    [("Screenshot (3538).png", (975, 596), (973, 600))];

/// The stand-in, earned: on each of the 23 originals, MC-027's rule as frozen
/// here gives exactly the columns the shipped pipeline crops to at margin 0 -
/// on `c004d96`, and after GREEN too, because AC-3 holds the originals exactly.
/// The three are left out only because GREEN is required to move them; in RED
/// the stand-in matched them as well (26 of 26).
///
/// MC-067: on `Screenshot (3538).png` the two must differ by exactly the
/// ruled fringe ([`STAND_IN_DIFFERS_ON`]); on the other 22 they must agree.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn mc027s_rule_frozen_here_reproduces_the_shipped_page_column_on_the_originals() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let t = Tuning {
        margin_px: 0,
        ..Tuning::default()
    };
    let mut differ = Vec::new();
    let mut compared = 0usize;
    for (entry, _) in marked() {
        // MC-056's three, MC-062's sixteen and MC-064's four are not originals either.
        if is_not_an_original(&entry.name()) {
            continue;
        }
        compared += 1;
        let stand_in = mc027_column(&luma(&entry.path), &t);
        if let Some(&(_, want_stand_in, want_shipped)) = STAND_IN_DIFFERS_ON
            .iter()
            .find(|(file, _, _)| *file == entry.name())
        {
            let shipped = crop_at(&entry, &tmp, &t).map(|crop| (crop.x, crop.w));
            if (stand_in.x, stand_in.w) != want_stand_in || shipped != Ok(want_shipped) {
                differ.push(format!(
                    "{}: MC-067 AC-2 - the stand-in must give x,w {want_stand_in:?} (got \
                     {:?}) and the shipped crop the ruled {want_shipped:?} (got {shipped:?})",
                    entry.name(),
                    (stand_in.x, stand_in.w)
                ));
            }
            continue;
        }
        match crop_at(&entry, &tmp, &t) {
            Ok(crop) if (crop.x, crop.w) == (stand_in.x, stand_in.w) => {}
            Ok(crop) => differ.push(format!(
                "{}: shipped x {} w {}, stand-in x {} w {}",
                entry.name(),
                crop.x,
                crop.w,
                stand_in.x,
                stand_in.w
            )),
            Err(what) => differ.push(format!("{}: {what}", entry.name())),
        }
    }
    assert_eq!(compared, 23, "the stand-in is earned on the 23 originals");
    assert!(
        differ.is_empty(),
        "MC-027's rule as frozen in this file must reproduce the shipped page column \
         on every original at margin_px 0, or AC-2's control is measuring a rule \
         that never shipped:\n{}",
        differ.join("\n")
    );
}

/// AC-2's control, the fix it exists to rule out. MC-027's rule (the stand-in
/// above) with `min_line_spread` lowered globally fails AC-2: at 5.0 it lets
/// in column 954 of `Screenshot (2630).jpg` on the left, and at 1.8 columns
/// 953 and 954.
///
/// Also the control that a predicate calling **nothing** page background
/// would fail: these columns must read as page background.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn a_min_line_spread_lowered_globally_lets_page_background_into_screenshot_2630() {
    let entries = marked();
    let mut got = Vec::new();
    let mut shown = Vec::new();
    let images: Vec<Luma> = entries.iter().map(|(entry, _)| luma(&entry.path)).collect();
    for (spread, _) in LOWERED_SPREADS {
        let t = Tuning {
            margin_px: 0,
            min_line_spread: spread,
            ..Tuning::default()
        };
        let mut failing: Vec<LetIn> = Vec::new();
        for ((entry, mark), img) in entries.iter().zip(&images) {
            let column = mc027_column(img, &t);
            let (left, right_side) = background_outside(img, column, *mark, &t);
            if !left.is_empty() {
                failing.push((entry.name(), Side::Left, left));
            }
            if !right_side.is_empty() {
                failing.push((entry.name(), Side::Right, right_side));
            }
        }
        for (name, side, cols) in &failing {
            shown.push(format!(
                "min_line_spread {spread}: {name} {side:?} {cols:?}"
            ));
        }
        let on_2630 = failing
            .iter()
            .find(|(name, side, _)| (name.as_str(), *side) == LET_IN_ON)
            .map(|(_, _, cols)| cols.clone());
        got.push((spread, on_2630));
    }
    println!("{}", shown.join("\n"));
    let want: Vec<(f32, Option<Vec<u32>>)> = LOWERED_SPREADS
        .iter()
        .map(|&(spread, cols)| (spread, Some(cols.to_vec())))
        .collect();
    assert_eq!(
        got,
        want,
        "MC-053 (MC-055) AC-2's control: MC-027's rule with min_line_spread lowered \
         globally must let page \
         background into {} on the {:?}, at margin_px 0 - the columns the story \
         measured. `(min_line_spread, page-background columns let in)`. Every side \
         that failed:\n{}",
        LET_IN_ON.0,
        LET_IN_ON.1,
        shown.join("\n")
    );
}

/// AC-2's control on the metric. The predicate calls each mark's own first
/// and last columns **not** page background on at least
/// [`METRIC_CONTROL_REQUIRED`] of the 26, and the only sides it calls page
/// background are among MC-049's seven mark errors. A predicate that called
/// everything background would fail the count; one that called nothing would
/// fail the previous test.
///
/// On `c004d96`: 19 of 26, the seven exceptions exactly MC-049's (14 of the 21
/// originals, as MC-049 measured; MC-052's two and the three all pass).
///
/// MC-056 adds its three, over 29, and one edge the user ruled art
/// ([`RULED_ART_EDGES`]): `2025-07-17 14_20_23.png`'s widened left edge, column
/// 962, flat black (share 1.000). The other two's edges read as art (0.327 /
/// 0.237 and 0.841 / 0.930). Measured in MC-056's RED on `c3fee28` with the
/// move applied: 28 of 29, the one exception exactly [`RULED_ART_EDGES`] (none
/// of MC-049's seven reads as background any longer).
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_predicate_calls_each_marks_own_edge_columns_art_except_on_mc049s_mark_errors() {
    let t = Tuning::default();
    let entries = marked();
    let mut ok = 0usize;
    let mut exceptions = Vec::new();
    let mut rows = Vec::new();
    for (entry, mark) in &entries {
        let img = luma(&entry.path);
        let first = background_share(&img, mark.x, *mark, t.uniform_tolerance);
        let last = background_share(&img, right(*mark), *mark, t.uniform_tolerance);
        let (first_bg, last_bg) = (
            first >= PAGE_BACKGROUND_SHARE,
            last >= PAGE_BACKGROUND_SHARE,
        );
        if !first_bg && !last_bg {
            ok += 1;
        }
        if first_bg {
            exceptions.push((entry.name(), Side::Left));
        }
        if last_bg {
            exceptions.push((entry.name(), Side::Right));
        }
        rows.push(format!(
            "{:<26} first column {} share {first:.3}  last column {} share {last:.3}",
            entry.name(),
            mark.x,
            right(*mark)
        ));
    }
    let printed = table("MC-053 (MC-055) AC-2's metric control", &rows);
    let unexplained: Vec<&(String, Side)> = exceptions
        .iter()
        .filter(|(name, side)| !MC049_MARK_ERRORS.contains(&(name.as_str(), *side)))
        .filter(|(name, side)| !RULED_ART_EDGES.contains(&(name.as_str(), *side)))
        .filter(|(name, side)| !MC072_BACKGROUND_EDGES.contains(&(name.as_str(), *side)))
        .collect();
    // MC-056: the other direction for the edges the user ruled art. Each must
    // still read as page background; one that stops is a stale exception.
    let ruled_art_read: Vec<(&str, Side)> = exceptions
        .iter()
        .map(|(name, side)| (name.as_str(), *side))
        .filter(|edge| RULED_ART_EDGES.contains(edge))
        .collect();
    // MC-072: the same other direction for the four's unlooked-at edges.
    let mc072_edges_read: Vec<(&str, Side)> = exceptions
        .iter()
        .map(|(name, side)| (name.as_str(), *side))
        .filter(|edge| MC072_BACKGROUND_EDGES.contains(edge))
        .collect();
    assert_eq!(
        entries.len(),
        116,
        "the control is over the 116 marked tuning entries (26 until MC-056 moved three, \
         29 until MC-062 moved sixteen, 45 until MC-064 moved four, 49 until MC-068 moved 21, \
         70 until MC-069 added 14, 84 until MC-072 moved four, 88 until MC-077 moved 11, \
         99 until MC-079 moved one, 100 until MC-081 added seven, 107 until MC-084 moved \
         nine)"
    );
    assert_eq!(
        ruled_art_read,
        RULED_ART_EDGES.to_vec(),
        "MC-056: every mark edge the user ruled art (RULED_ART_EDGES) must still read \
         as page background here - if one no longer does, the exception is stale and \
         must be removed. `left` is measured.\n\n{printed}"
    );
    assert_eq!(
        mc072_edges_read,
        MC072_BACKGROUND_EDGES.to_vec(),
        "MC-072: every unlooked-at mark edge of the four MC-071 read per file \
         (MC072_BACKGROUND_EDGES) must still read as page background here - if one \
         no longer does, or the user rules on it, it comes off the list. `left` is \
         measured.\n\n{printed}"
    );
    assert!(
        ok >= METRIC_CONTROL_REQUIRED && unexplained.is_empty(),
        "MC-053 (MC-055) AC-2's control on the metric: a mark's own first and last columns \
         are art, so the predicate must call both not page background on at least \
         {METRIC_CONTROL_REQUIRED} of {} entries (it did on {ok}), and may call a \
         mark edge background only on MC-049's seven mark errors, the edges the \
         user ruled art (RULED_ART_EDGES, MC-056, MC-065 and MC-074) and MC-072's \
         unlooked-at edges (MC072_BACKGROUND_EDGES) (it also did on \
         {unexplained:?}).\n\n{printed}",
        entries.len()
    );
}

// --- AC-3: the 23 originals do not move -------------------------------------

/// AC-3. Each of the 23 originals is cropped to exactly the rect `c004d96`
/// produces, at both margins ([`ORIGINALS_AT_BOTH_MARGINS`]).
///
/// Green on arrival by construction; earned in RED by a probe recorded in the
/// story's `## Handoff`: the variant "extend outward through every column
/// that is not page background, measured over the central band", in an
/// ignored copy of the tree, moves `Screenshot (3538).png` by one column on
/// each side, and this test goes red naming it.
///
/// MC-067 re-pinned 3538 to the user's ruling of 2026-10-01 (it grows by its
/// 2-column fringe on each side). The pin is still exact, so this test still
/// goes red on any move of 3538, including back to its old crop.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_twenty_three_original_crops_do_not_move_by_a_pixel_at_either_margin() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut moved = Vec::new();
    let mut seen = Vec::new();
    for (entry, _) in marked() {
        let name = entry.name();
        // MC-056's three, MC-062's sixteen and MC-064's four are not originals either.
        if is_not_an_original(&name) {
            continue;
        }
        seen.push(name.clone());
        let pinned = ORIGINALS_AT_BOTH_MARGINS
            .iter()
            .find(|(file, _, _)| *file == name)
            .map(|&(_, m3, m0)| [m3, m0]);
        for (i, t) in both_margins().iter().enumerate() {
            let want = pinned.map(|p| {
                let [x, y, w, h] = p[i];
                Rect { x, y, w, h }
            });
            match crop_at(&entry, &tmp, t) {
                Ok(rect) if Some(rect) == want => {}
                Ok(rect) => moved.push(format!(
                    "{name} at margin_px {}: {rect:?}, pinned {want:?}",
                    t.margin_px
                )),
                Err(what) => moved.push(format!("{name} at margin_px {}: {what}", t.margin_px)),
            }
        }
    }
    assert_eq!(
        seen,
        ORIGINALS_AT_BOTH_MARGINS
            .map(|(name, _, _)| name.to_string())
            .to_vec(),
        "MC-053 (MC-055) AC-3 must reach all 23 originals, in manifest order"
    );
    assert!(
        moved.is_empty(),
        "MC-053 (MC-055) AC-3: the fix for dark art must leave every original crop exactly \
         as post-MC-052 main produces it, x, y, w and h, at both margins. Moved:\n{}",
        moved.join("\n")
    );
}
