//! MC-049, AC-1, AC-2 and AC-4: the crop's side edges carry no page background,
//! still never clip, and the rows move only by the margin.
//!
//! The user, 2026-09-23: *"The cropping from the side isnt tight enough and I
//! still see the page on the right and left side."* MC-027's column locator
//! already stops on the boundary between flat page background and textured
//! art; `margin::expand` then adds `Tuning::margin_px` columns of that page
//! back on every side. MC-049 rules the margin to 0 on all four sides and
//! corrects the seven marks that contained flat page columns (`corpus.md`,
//! "Corrected marks").
//!
//! # Why this is a new file and not more of `tests/corpus.rs`
//!
//! `tests/corpus.rs` (MC-026, MC-027) and `tests/corpus_accuracy.rs` (MC-019)
//! are other stories' criteria, and MC-048 is editing `corpus.rs` in parallel.
//! This target shares their loader (`common/corpus.rs`) by the same `#[path]`
//! include and nothing else. It sorts after `corpus.rs`, so the `integration`
//! gate's floor - read off `corpus.rs`, the first engine target with ignored
//! tests - is not moved by it.
//!
//! The criteria this file does **not** restate, because an existing test
//! already says them and must keep saying them: AC-3 is
//! `corpus.rs::the_six_pages_flagged_today_are_still_flagged_for_no_border_found`;
//! AC-5 is `corpus.rs::the_left_and_right_edges_of_every_marked_page_land_inside_the_window`
//! and `corpus_accuracy.rs::at_least_nine_corpus_screenshots_in_ten_are_right_on_the_column_axis`;
//! AC-2's "the existing inset-10 controls stay" is `corpus.rs`'s two
//! `no_crop_clips_*` tests and `corpus_accuracy.rs`'s zero-clip test, all of
//! which now read the corrected marks.
//!
//! # Running it
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_sides -- --ignored --nocapture
//! ```
//!
//! # What is settled, what is invented, what is mechanical
//!
//! * **Settled, read out**: `uniform_tolerance` (10), from the `Tuning` under
//!   test; the containment predicate (`corpus.rs`'s); the marking rule and the
//!   user's rulings on the corrected marks, which live in the manifest.
//! * **Invented here, with two controls - the page-background predicate.** A
//!   column is page background when at least [`PAGE_BACKGROUND_SHARE`] of its
//!   pixels over the mark's rows lie within `uniform_tolerance` of the
//!   column's median luma (the upper median, `sorted[n / 2]`). The story's
//!   definition verbatim. The control on the metric - the mark's own first and
//!   last columns are **not** background - and the control on the number - a
//!   crop one column wider **is** caught - are both in AC-1's test.
//! * **Mechanical**: the margin before this story ([`MARGIN_BEFORE_MC049`]),
//!   which AC-4 measures the row move against.
//!
//! # Measured in RED (story `## Handoff`)
//!
//! At `cb2deef` + the corrected marks, margin 3: AC-1 fails on 42 of 42 sides;
//! AC-2's narrowed-crop control clips 0 entries. Against a candidate at
//! margin 0 (a scratch copy outside the repository): AC-1 0 of 42; the
//! widened control fires on 19 of 21 left sides and 20 of 21 right; the metric
//! control holds on 21 of 21; AC-2 0 clips and the narrowed control clips 9.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use std::path::Path;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::viewport::locate;
use cropper_core::{Luma, Rect, Tuning, detect};
use cropper_engine::{Outcome, process_file};

// --- Constants ----------------------------------------------------------------

/// The page-background predicate's share: the story's definition.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

/// AC-1's control on the metric: on at least this many of the 21 entries the
/// mark's own first **and** last columns must read as *not* page background.
/// The story's number. Measured 21 of 21 on the corrected marks, and 14 of 21
/// on the uncorrected ones - the seven corrected entries each had a flat
/// outermost column, which is exactly what this control exists to notice.
const METRIC_CONTROL_REQUIRED: usize = 19;

/// AC-1's control on the number: widening the crop by one column on one side
/// must make that side fail, on at least this many of the 21 entries **per
/// side**.
///
/// The story predicted 21 of 21, "because the next column out is page
/// background on every one". RED measured that premise false on three sides
/// under the story's own 0.95 predicate: the column just outside the locator
/// is a near-flat transition column on `2025-10-20 15_37_25.png` left (share
/// 0.94) and on `Screenshot (3538).png` both sides (0.86 left, 0.89 right).
/// 19 is the measured left count and one below the measured right count
/// (20). **Escalated in the story's `## Handoff`**: this number, or the
/// predicate's share, needs the product owner's ruling before GREEN.
///
/// MC-067 grows 3538's crop by those transition columns (the user's ruling of
/// 2026-10-01), so the next column out on both of its sides is the site's
/// single value, and the control fires there too: 3538 adds to both counts.
const WIDENED_CONTROL_REQUIRED_PER_SIDE: usize = 19;

/// AC-2's control: the crop narrowed by one column on each side must clip on
/// at least this many entries. The story's number; RED measured 9 against the
/// margin-0 candidate (the seven corrected marks, `Screenshot (3538).png`'s
/// left and `2026-01-05 13_45_59.png`'s right, whose MC-027 correction already
/// put its mark on the locator's edge).
///
/// MC-067 grows 3538's crop 2 columns past its mark's left edge, so 3538 no
/// longer counts here, and `f09`, cropped to exactly its mark, now does.
const NARROWED_CLIPS_REQUIRED: usize = 8;

/// `Tuning::margin_px` before MC-049, which AC-4 measures the row move from.
/// Mechanical: the value `architecture.md` decision 5 carried until this story.
const MARGIN_BEFORE_MC049: u32 = 3;

/// MC-056, the user's ruling of 2026-09-29 (its Open question 2): the one
/// marked `tuning` entry that is not cropped - the detector flags it
/// `Ambiguous` - named as AC-1's only known exception. **Exact in both
/// directions**: AC-1 fails if any other marked `tuning` entry is not cropped,
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
/// and AC-1 judges its sides like every other entry's. Every marked `tuning`
/// entry must be cropped; the list stays exact in both directions.
const KNOWN_NOT_CROPPED: [&str; 0] = [];

/// MC-064: the crop `process_file` makes at `Tuning::default()` (margin_px 0)
/// of each of the four fresh entries MC-063 read per file, `(file, [x, y, w,
/// h])`, in manifest order. **Measured, not chosen**: read out of one run on
/// `43e8e61` (release; crates unchanged since `d2876f5`) in MC-064's RED, on a
/// scratch copy with only the four `split` values changed. Every MC-064
/// exception in this file holds its entry to exactly this crop.
///
/// MC-065 took `f20` (`2025-08-07 01_13_55.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-2): with its right edge fixed, AC-1 and AC-2 here
/// judge it like every other entry, so its crop may carry no page background
/// either. `f09` (`2025-12-08 17_22_50.png`) stays at its MC-064 pin for
/// MC-067 (MC-065 `## Amendments`, 2026-10-01).
///
/// MC-066 took `f18` (`2025-03-06 12_48_06.png`) and `f13` (`2025-03-16
/// 22_47_44.png`) off this list, off [`KNOWN_CLIPS`] and off
/// [`KNOWN_BACKGROUND_SIDES`] (its AC-1 to AC-3): their crops are the reader's
/// window now, and AC-1 and AC-2 here judge them like every other entry.
///
/// MC-067 took `f09` off this list and off [`KNOWN_CLIPS`] (its AC-1): it
/// keeps its art's near-black right edge, and AC-1 and AC-2 here judge it like
/// every other entry. Its pin was `[1006, 167, 531, 1233]`. The list was
/// empty, and stayed exact.
///
/// **MC-072 adds two** (the user's answer of 2026-10-02, *"all
/// recommended"*): `n02` and `n05`, the two fresh entries MC-071 read per file
/// whose crop clips the mark, in manifest order. **Measured, not chosen**:
/// read out of one run on `0f9c579` (release) in MC-072's RED, on a scratch
/// copy with only the four `split` values changed; they equal MC-071's
/// recorded crops. They are held here for [`KNOWN_CLIPS`] only. Neither has a
/// page-background side (`n02`'s columns lie inside its mark, `n05`'s right
/// edge is the mark's), so [`KNOWN_BACKGROUND_SIDES`] stays empty.
///
/// MC-073 took `n05` (`2025-11-01 12_34_31.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-2): the user re-marked it, ruling on 2026-10-02
/// *"Box starts at 1007"* - column 1006 is the dark seam beside the art, as
/// on `Screenshot (3605).png` and `2025-10-20 15_37_25.png`. Its mark is
/// `1007,167 531x1233`, which its crop at margin 0 already equals, so AC-1
/// and AC-2 here judge it like every other entry. Its pin was
/// `[1007, 167, 531, 1233]`.
///
/// MC-074 took `n02` (`2025-03-07 00_05_58.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-1): its drawn panels sit on flat white page paper,
/// which the user ruled on 2026-10-02 is page (*"Box stands"*), so its crop
/// at margin 0 is to equal its mark `643,115 533x1284` and AC-1 and AC-2 here
/// judge it like every other entry. Its pin was `[703, 0, 398, 1440]`. The
/// list is empty, and stays exact. Equal to its mark, its crop keeps no column
/// outside it, so it has no page-background side either and
/// [`KNOWN_BACKGROUND_SIDES`] stays empty.
///
/// **MC-079 adds one** (its AC-3): `n05` of MC-077's fresh draw,
/// `2025-10-26 12_13_16.png`, the one entry MC-078 read per file, whose crop
/// clips its mark `973,167 600x1233` by one column on the left. **Measured,
/// not chosen**: read out of one run on `6343029` (release) in MC-079's RED,
/// on a scratch copy with only its `split` value changed; it equals MC-078's
/// recorded crop, columns 974..1573, rows 167..1400. It is held here for
/// [`KNOWN_CLIPS`] only. Its crop keeps no column outside the mark (its right
/// edge is the mark's), so it has no page-background side and
/// [`KNOWN_BACKGROUND_SIDES`] stays empty.
///
/// MC-080 took `n05` (`2025-10-26 12_13_16.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-2): the user re-marked it, ruling on 2026-10-04
/// *"Box starts at 974"* - column 973 is the dark seam beside the art, as on
/// MC-073's `2025-11-01 12_34_31.png`, `Screenshot (3605).png` and
/// `2025-10-20 15_37_25.png`. Its mark is `974,167 599x1233`, which its crop
/// at margin 0 already equals, so AC-1 and AC-2 here judge it like every
/// other entry. Its pin was `[974, 167, 599, 1233]`. The list is empty, and
/// stays exact. Equal to its mark, its crop keeps no column outside it, so it
/// has no page-background side either and [`KNOWN_BACKGROUND_SIDES`] stays
/// empty.
///
/// **MC-081 adds all seven** of its Eleceed screenshots (its AC-4), in manifest
/// order, each the crop that does not contain its mark: six of the right-hand
/// (YouTube) window, and `e03` (`2025-03-07 01_02_31.png`) of the manhwa's
/// window at full height, starting at column 727 against a page that starts at
/// 610. **Measured, not chosen**: read out of one run on `ebdedd7` (release,
/// margin_px 0) in MC-081's RED, on a scratch copy with the seven files and
/// entries added; they equal the crops the Lead PO measured on `main`. They are
/// held here for [`KNOWN_CLIPS`] only: AC-1 passes with all seven read and
/// unlisted, so [`KNOWN_BACKGROUND_SIDES`] stays empty. Four of them clip the
/// mark's rows too, which [`KNOWN_ROW_CLIPS`] names at both margins. MC-082
/// and MC-083 take them off.
const MC064_CROPS: [(&str, [u32; 4]); 7] = [
    ("2025-03-16 22_56_00.png", [1820, 121, 402, 1138]), // e01
    ("2025-03-07 00_20_37.png", [1936, 0, 609, 1440]),   // e02
    ("2025-03-07 01_02_31.png", [727, 0, 483, 1440]),    // e03
    ("2025-03-16 22_48_01.png", [1820, 121, 402, 1138]), // e04
    ("2025-03-16 22_51_37.png", [1820, 0, 402, 1400]),   // e05
    ("2025-03-16 22_51_49.png", [1820, 179, 402, 1080]), // e06
    ("2025-03-16 22_54_27.png", [1820, 173, 402, 1086]), // e07
];

/// MC-064, the user's ruling of 2026-09-30 (its Open question 1, *"List them
/// as known"*): AC-2's known clips, the two of [`MC064_CROPS`] whose crop does
/// not contain the mark. **Exact in both directions**: AC-2 fails if any
/// other crop clips, and fails if a listed entry's crop is not its pin.
/// MC-065 took `f20` off, MC-066 took `f18` off and MC-067 took `f09` off.
/// The list was empty: no known clip was left.
///
/// **MC-072 adds two**, by the user's answer of 2026-10-02 (*"all
/// recommended"*): `n02` (`2025-03-07 00_05_58.png`) and `n05` (`2025-11-01
/// 12_34_31.png`), each held to its measured pin in [`MC064_CROPS`]. The fix
/// stories take them off.
///
/// MC-073 took `n05` off (its AC-2): the user re-marked it to
/// `1007,167 531x1233` (*"Box starts at 1007"*, 2026-10-02), which its crop
/// already equals, so it no longer clips. The list stays exact both ways.
///
/// MC-074 took `n02` off (its AC-1): the user ruled its flat white page paper
/// is page (*"Box stands"*, 2026-10-02), so its crop is to equal its mark
/// `643,115 533x1284` and it no longer clips. The list was empty, and stayed
/// exact both ways: no known clip was left.
///
/// **MC-079 adds one** (its AC-3): `n05` (`2025-10-26 12_13_16.png`), the one
/// entry of MC-077's fresh draw MC-078 read per file, whose crop clips its
/// mark by one column on the left, held to its measured pin in
/// [`MC064_CROPS`]. MC-080 takes it off.
///
/// MC-080 took `n05` off (its AC-2): the user re-marked it to
/// `974,167 599x1233` (*"Box starts at 974"*, 2026-10-04), which its crop
/// already equals, so it no longer clips. The list is empty, and stays exact
/// both ways: no known clip is left.
///
/// **MC-081 adds all seven** of its Eleceed screenshots (its AC-4), each held
/// to its measured pin in [`MC064_CROPS`]: six cropped to the right-hand
/// (YouTube) window, and `e03` cut 117 columns short on the left. MC-082 and
/// MC-083 take them off.
const KNOWN_CLIPS: [&str; 7] = [
    "2025-03-16 22_56_00.png",
    "2025-03-07 00_20_37.png",
    "2025-03-07 01_02_31.png",
    "2025-03-16 22_48_01.png",
    "2025-03-16 22_51_37.png",
    "2025-03-16 22_51_49.png",
    "2025-03-16 22_54_27.png",
];

/// MC-081 AC-4: AC-4's known row clips here,
/// `(file, [x, y, w, h] at margin_px 3, [x, y, w, h] at margin_px 0)`, in
/// manifest order. Four of MC-081's seven Eleceed screenshots are cropped to
/// the right-hand (YouTube) window, where the viewport stage, handed that
/// window's column, locates rows that end at 1259, above the marked page's
/// bottom row 1391: `e01` and `e04` 121..1259, `e06` 179..1259, `e07`
/// 173..1259 - so at margin 0 the crop's rows do not contain the mark's
/// 115..1392. The other three (`e02`, `e03`, `e05`) keep rows 0..1440 or
/// 0..1400 there, which do contain the mark's rows, so they are not here.
/// **Measured, not chosen**: `detect`'s rect at margins 3 and 0, read out of
/// one run on `ebdedd7` (release) in MC-081's RED, on a scratch copy with the
/// seven files and entries added; they equal `process_file`'s crops there.
/// **Exact in both directions**: AC-4 fails if any other entry's rows do not
/// contain its mark, if a listed entry's rows come to contain it, or if a
/// listed entry's rect is not its pin at either margin. MC-082 takes them off.
const KNOWN_ROW_CLIPS: [(&str, [u32; 4], [u32; 4]); 4] = [
    (
        "2025-03-16 22_56_00.png",
        [1817, 121, 408, 1138],
        [1820, 121, 402, 1138],
    ), // e01
    (
        "2025-03-16 22_48_01.png",
        [1817, 121, 408, 1138],
        [1820, 121, 402, 1138],
    ), // e04
    (
        "2025-03-16 22_51_49.png",
        [1817, 179, 408, 1080],
        [1820, 179, 402, 1080],
    ), // e06
    (
        "2025-03-16 22_54_27.png",
        [1817, 173, 408, 1086],
        [1820, 173, 402, 1086],
    ), // e07
];

/// MC-064, the same ruling: AC-1's known page-background sides, `(file,
/// side)`. `2025-03-16 22_47_44.png`'s crop runs to column 2556 at margin 0,
/// keeping the browser's scrollbar, while the mark ends at 1167; columns the
/// crop keeps beyond the mark on its right read as page background - measured
/// in MC-064's RED, 1 of 98 sides. **Exact in both directions**:
/// AC-1 fails if any other side carries page background, if this one stops
/// doing so, or if the entry's crop is not its pin in [`MC064_CROPS`].
///
/// MC-066 took `f13` off (its AC-2 and AC-3): its crop ends inside the
/// reader's window, and AC-1 judges its sides like every other entry's. The
/// list is empty, and stays exact in both directions.
const KNOWN_BACKGROUND_SIDES: [(&str, &str); 0] = [];

/// MC-064: every one of `names` whose crop in `got` is not exactly its pin in
/// [`MC064_CROPS`], as a row naming both. `got` is `(file, crop or None)` for
/// every marked entry the test reached.
fn mc064_crops_moved(got: &[(String, Option<Rect>)], names: &[&str]) -> Vec<String> {
    MC064_CROPS
        .iter()
        .filter(|(file, _)| names.contains(file))
        .filter_map(|&(file, [x, y, w, h])| {
            let pinned = Some(Rect { x, y, w, h });
            match got.iter().find(|(name, _)| name == file) {
                None => Some(format!("{file}: not a marked `tuning` entry here")),
                Some((_, crop)) if *crop == pinned => None,
                Some((_, crop)) => Some(format!("{file}: crop {crop:?}, pinned {pinned:?}")),
            }
        })
        .collect()
}

// --- Harness ------------------------------------------------------------------

/// The marked `tuning` entries, in manifest order. Tuning only - see
/// `corpus.rs::tuning_only` for why no accuracy suite reads held-out entries.
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

/// The luma plane `process_file` sees: the engine's decoder and conversion.
fn luma(path: &Path) -> Luma {
    let bytes =
        std::fs::read(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// The rect `process_file` cropped `entry` to at `t`, or the outcome as text.
fn cropped(entry: &CorpusEntry, t: &Tuning, tmp: &tempfile::TempDir) -> Result<Rect, String> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Ok(rect),
        Outcome::Flagged { reason, .. } => Err(format!("Flagged {reason:?}")),
        Outcome::Failed { error } => Err(format!("Failed {error}")),
    }
}

/// The share of column `x`'s pixels over `mark`'s rows lying within
/// `tolerance` of the column's median luma (upper median).
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

/// The story's page-background predicate.
fn is_page_background(img: &Luma, x: u32, mark: Rect, t: &Tuning) -> bool {
    background_share(img, x, mark, t.uniform_tolerance) >= PAGE_BACKGROUND_SHARE
}

/// The last column of `r`, inclusive.
fn right(r: Rect) -> u32 {
    r.x + r.w - 1
}

/// The crop's columns that lie outside the mark on the left and on the right.
fn outside_columns(crop: Rect, mark: Rect) -> (Vec<u32>, Vec<u32>) {
    let left = (crop.x..mark.x.min(right(crop) + 1)).collect();
    let right_side = ((right(mark) + 1).max(crop.x)..=right(crop)).collect();
    (left, right_side)
}

/// Whether `outer` contains `inner` on all four sides - `corpus.rs`'s AC-5
/// predicate.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// A table printed under `--nocapture` and returned for the assertion message.
fn table(title: &str, rows: &[String]) -> String {
    let out = format!("{title}\n{}\n", rows.join("\n"));
    println!("{out}");
    out
}

// --- AC-1 ---------------------------------------------------------------------

/// AC-1, with both of its controls.
///
/// For each of the 21 marked `tuning` entries, `process_file` at
/// `Tuning::default()`: no column of the crop lying outside the mark may be
/// page background, on either side. Before MC-049 every side carries
/// `margin_px` columns of flat page - 42 of 42 sides fail.
///
/// * **Control on the metric**: the mark's own first and last columns are
///   art, and the predicate must say so on at least
///   [`METRIC_CONTROL_REQUIRED`] entries - a predicate that calls everything
///   background fails here.
/// * **Control on the number**: the crop widened by one column on one side
///   must fail that side on at least [`WIDENED_CONTROL_REQUIRED_PER_SIDE`]
///   entries per side - a predicate that calls nothing background, or an
///   assertion blind to a single column, fails here.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_column_the_crop_keeps_outside_the_mark_is_page_background() {
    let t = Tuning::default();
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut failing_sides = Vec::new();
    let mut metric_ok = 0usize;
    let mut metric_missed = Vec::new();
    let (mut widened_left, mut widened_right) = (0usize, 0usize);
    let mut widened_missed = Vec::new();
    let mut entries = 0usize;
    let mut not_cropped = Vec::new();
    // MC-064: every crop, and every `(file, side)` carrying page background,
    // compared with KNOWN_BACKGROUND_SIDES below, exactly.
    let mut got_all: Vec<(String, Option<Rect>)> = Vec::new();
    let mut background_sides: Vec<(String, &str)> = Vec::new();

    for (entry, mark) in marked() {
        entries += 1;
        let img = luma(&entry.path);
        let bg = |x: u32| is_page_background(&img, x, mark, &t);

        // The control on the metric does not need a crop.
        let (first_bg, last_bg) = (bg(mark.x), bg(right(mark)));
        if !first_bg && !last_bg {
            metric_ok += 1;
        } else {
            metric_missed.push(format!(
                "{}: mark column {} background {first_bg}, column {} background {last_bg}",
                entry.name(),
                mark.x,
                right(mark)
            ));
        }

        let crop = match cropped(&entry, &t, &tmp) {
            Ok(rect) => rect,
            Err(outcome) => {
                rows.push(format!("{:<30} {outcome}", entry.name()));
                // MC-056: every entry not cropped is collected by name and
                // compared with KNOWN_NOT_CROPPED below, exactly. Before MC-056
                // it went into `failing_sides` twice; any entry other than the
                // known exception still fails AC-1, now on that comparison.
                not_cropped.push(entry.name());
                got_all.push((entry.name(), None));
                continue;
            }
        };
        got_all.push((entry.name(), Some(crop)));

        let (left_cols, right_cols) = outside_columns(crop, mark);
        let left_bg: Vec<u32> = left_cols.iter().copied().filter(|&x| bg(x)).collect();
        let right_bg: Vec<u32> = right_cols.iter().copied().filter(|&x| bg(x)).collect();
        let known = |side: &str| KNOWN_BACKGROUND_SIDES.contains(&(entry.name().as_str(), side));
        if !left_bg.is_empty() {
            background_sides.push((entry.name(), "left"));
        }
        if !right_bg.is_empty() {
            background_sides.push((entry.name(), "right"));
        }
        if !left_bg.is_empty() && !known("left") {
            failing_sides.push(format!(
                "{} left: page-background columns {left_bg:?} outside the mark's {}",
                entry.name(),
                mark.x
            ));
        }
        if !right_bg.is_empty() && !known("right") {
            failing_sides.push(format!(
                "{} right: page-background columns {right_bg:?} outside the mark's {}",
                entry.name(),
                right(mark)
            ));
        }

        // The control on the number: one more column on each side, in turn.
        let next_left = crop.x.checked_sub(1).filter(|&x| x < mark.x);
        let next_right = Some(right(crop) + 1).filter(|&x| x < img.width && x > right(mark));
        let fired_left = next_left.is_some_and(bg);
        let fired_right = next_right.is_some_and(bg);
        widened_left += usize::from(fired_left);
        widened_right += usize::from(fired_right);
        for (side, col, fired) in [
            ("left", next_left, fired_left),
            ("right", next_right, fired_right),
        ] {
            if !fired {
                let share = col.map(|x| background_share(&img, x, mark, t.uniform_tolerance));
                widened_missed.push(format!(
                    "{} {side}: next column {col:?} share {share:.2?}",
                    entry.name()
                ));
            }
        }

        rows.push(format!(
            "{:<30} crop {:>4}..{:<4} mark {:>4}..{:<4} outside L {:>2} (bg {:>2})  R {:>2} (bg {:>2})  widened L {} R {}",
            entry.name(),
            crop.x,
            right(crop),
            mark.x,
            right(mark),
            left_cols.len(),
            left_bg.len(),
            right_cols.len(),
            right_bg.len(),
            if fired_left { "fires" } else { "-" },
            if fired_right { "fires" } else { "-" },
        ));
    }

    let printed = table(
        &format!(
            "AC-1: page-background columns outside the mark, margin_px {}, share >= {PAGE_BACKGROUND_SHARE} within {} of the column median",
            t.margin_px, t.uniform_tolerance
        ),
        &rows,
    );
    // The controls' measured counts, for a story's controls table
    // (`--nocapture`).
    println!(
        "AC-1 controls: metric {metric_ok} of {entries}; widened fires on {widened_left} \
         left and {widened_right} right"
    );

    assert!(
        entries >= METRIC_CONTROL_REQUIRED,
        "AC-1 read only {entries} marked tuning entries"
    );
    assert!(
        metric_ok >= METRIC_CONTROL_REQUIRED,
        "AC-1's control on the metric: the mark's own first and last columns are \
         art, so the predicate must call them not page background on at least \
         {METRIC_CONTROL_REQUIRED} of {entries} entries; it did on {metric_ok}. \
         Either the predicate calls art background, or a mark still contains \
         flat page columns:\n{}\n\n{printed}",
        metric_missed.join("\n")
    );
    assert!(
        widened_left >= WIDENED_CONTROL_REQUIRED_PER_SIDE
            && widened_right >= WIDENED_CONTROL_REQUIRED_PER_SIDE,
        "AC-1's control on the number: a crop widened by one column must be caught \
         carrying page background on that side, on at least \
         {WIDENED_CONTROL_REQUIRED_PER_SIDE} of {entries} entries per side; it was \
         on {widened_left} left and {widened_right} right. Misses:\n{}\n\n{printed}",
        widened_missed.join("\n")
    );
    assert_eq!(
        not_cropped,
        KNOWN_NOT_CROPPED.map(String::from).to_vec(),
        "AC-1: every marked tuning entry must be cropped - KNOWN_NOT_CROPPED is \
         empty since MC-070 cropped (2705). `left` is the entries not \
         cropped.\n\n{printed}"
    );
    // MC-064: the known page-background sides, exact in both directions, and
    // their entries held to their measured crops.
    let known_read: Vec<(String, &str)> = background_sides
        .iter()
        .filter(|(name, side)| KNOWN_BACKGROUND_SIDES.contains(&(name.as_str(), *side)))
        .cloned()
        .collect();
    assert_eq!(
        known_read,
        KNOWN_BACKGROUND_SIDES
            .map(|(name, side)| (name.to_string(), side))
            .to_vec(),
        "MC-064: every known page-background side (KNOWN_BACKGROUND_SIDES, the user's \
         ruling of 2026-09-30) must still carry page background - if a fix clears one, \
         take it off the list. `left` is measured.\n\n{printed}"
    );
    let known_names: Vec<&str> = KNOWN_BACKGROUND_SIDES
        .iter()
        .map(|(name, _)| *name)
        .collect();
    let moved = mc064_crops_moved(&got_all, &known_names);
    assert!(
        moved.is_empty(),
        "MC-064: each entry with a known page-background side must still be cropped to \
         exactly its pin in MC064_CROPS:\n{}\n\n{printed}",
        moved.join("\n")
    );
    assert!(
        failing_sides.is_empty(),
        "AC-1: the crop must carry no page background outside the mark, on either \
         side, except MC-064's known sides (KNOWN_BACKGROUND_SIDES). {} of {} sides \
         do (margin_px is {}); the first is {}.\n\nAll:\n{}\n\n{printed}",
        failing_sides.len(),
        2 * entries,
        t.margin_px,
        failing_sides.first().map_or("-", String::as_str),
        failing_sides.join("\n")
    );
}

// --- AC-2 ---------------------------------------------------------------------

/// AC-2, with its control.
///
/// Against the corrected marks, every `Cropped` rect contains its mark on all
/// four sides - 0 clips. With the margin at 0 the crop's side edges rest on
/// the locator alone, so the control is what shows the crop is *tight*: the
/// same crop narrowed by one column on each side must clip on at least
/// [`NARROWED_CLIPS_REQUIRED`] entries. Before MC-049 it clips on none - three
/// columns of margin hide every one.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn every_crop_contains_its_corrected_mark_and_one_column_narrower_clips() {
    let t = Tuning::default();
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut clips = Vec::new();
    let mut narrowed_clips = Vec::new();
    let mut cropped_count = 0usize;
    let mut got_all: Vec<(String, Option<Rect>)> = Vec::new();

    for (entry, mark) in marked() {
        let crop = match cropped(&entry, &t, &tmp) {
            Ok(rect) => rect,
            Err(outcome) => {
                rows.push(format!("{:<30} {outcome}", entry.name()));
                got_all.push((entry.name(), None));
                continue;
            }
        };
        got_all.push((entry.name(), Some(crop)));
        cropped_count += 1;
        let holds = contains(crop, mark);
        // MC-064: a known clip is held to its pinned crop below instead.
        if !holds && !KNOWN_CLIPS.contains(&entry.name().as_str()) {
            clips.push(format!(
                "{}: crop {},{} {}x{} does not contain the mark {},{} {}x{}",
                entry.name(),
                crop.x,
                crop.y,
                crop.w,
                crop.h,
                mark.x,
                mark.y,
                mark.w,
                mark.h
            ));
        }
        let narrowed = Rect {
            x: crop.x + 1,
            w: crop.w - 2,
            ..crop
        };
        let narrowed_holds = contains(narrowed, mark);
        // MC-064: a known clip clips narrowed or not, so it would make the
        // control easier to meet; it is left out of the count.
        if !narrowed_holds && !KNOWN_CLIPS.contains(&entry.name().as_str()) {
            narrowed_clips.push(entry.name());
        }
        rows.push(format!(
            "{:<30} crop {:>4}..{:<4} mark {:>4}..{:<4} slack L {:>+3} R {:>+3}  {}  narrowed {}",
            entry.name(),
            crop.x,
            right(crop),
            mark.x,
            right(mark),
            i64::from(mark.x) - i64::from(crop.x),
            i64::from(right(crop)) - i64::from(right(mark)),
            if holds { "holds" } else { "CLIPS" },
            if narrowed_holds { "holds" } else { "clips" },
        ));
    }

    let printed = table(
        &format!(
            "AC-2: containment of the corrected marks, margin_px {} ({cropped_count} cropped)",
            t.margin_px
        ),
        &rows,
    );
    // The control's measured count, for a story's controls table
    // (`--nocapture`).
    println!(
        "AC-2 control: narrowed clips on {} ({narrowed_clips:?})",
        narrowed_clips.len()
    );

    assert!(
        cropped_count > 0,
        "AC-2 compared nothing: no marked entry was cropped\n\n{printed}"
    );
    let moved = mc064_crops_moved(&got_all, &KNOWN_CLIPS);
    assert!(
        moved.is_empty(),
        "MC-064, MC-072, MC-079, MC-081: each known clip (KNOWN_CLIPS; the user's rulings of 2026-09-30 and 2026-10-02, MC-079 AC-3, MC-081 AC-4) must \
         still be cropped to exactly its pin in MC064_CROPS - if a fix moves one, take \
         it off the list and let AC-2 judge it:\n{}\n\n{printed}",
        moved.join("\n")
    );
    assert!(
        clips.is_empty(),
        "AC-2: a crop that cuts into the page a person marked is the worst defect \
         this product has, and with margin_px {} nothing but the locator stands \
         between the art and the cut. {} of {cropped_count} clip besides MC-064's, MC-072's, MC-079's and MC-081's \
         known clips (KNOWN_CLIPS); the first is {}.\n\n\
         All:\n{}\n\n{printed}",
        t.margin_px,
        clips.len(),
        clips[0],
        clips.join("\n")
    );
    assert!(
        narrowed_clips.len() >= NARROWED_CLIPS_REQUIRED,
        "AC-2's control: the crop narrowed by one column on each side must clip on \
         at least {NARROWED_CLIPS_REQUIRED} entries, or the crop still carries \
         slack beside the art and 'zero clips' is measured through a margin. It \
         clipped on {} ({narrowed_clips:?}); margin_px is {}.\n\n{printed}",
        narrowed_clips.len(),
        t.margin_px
    );
}

// --- AC-4 ---------------------------------------------------------------------

/// Sides on which the margin must be seen to reach: a side MC-048's viewport
/// stage did not cut, with room for the whole margin before the image edge.
/// Without this floor, a corpus on which the stage cut every side would pass
/// AC-4 with the margin never exercised at all.
///
/// Until MC-076 it was a count, 2: the two WebPs' tops (`2025-08-05
/// 00_11_13.webp` and `00_11_27.webp`), where the stage declined and the
/// located top was row 18. MC-076 AC-2 has the stage cut both WebPs' tops at
/// row 115, and AC-1 `n13`'s top and bottom (the other two sides the margin
/// reached on `main`, `1d7a921`, where the stage declined on it too). So the
/// control is **moved, not loosened** (MC-076 AC-3), to the two sides that
/// still exercise it and that no story moves: the **bottoms** of
/// `Screenshot (3605).png` and `Screenshot (3606).png`, where the stage cut
/// the top at 137 but the page column ends at row 1379 and 1385, above the
/// viewport's 1392, so the margin grows the bottom by 3 (1382, 1388).
/// Mechanical: read off this test's own table on `1d7a921`.
///
/// **Named, and exact in both directions**, where it was a floor: the test
/// fails if either side stops being reached, and fails if any other side is
/// reached in full - which on this corpus means the stage declined, or cut
/// nothing, where it should have cut. `(file, side)` in manifest order.
///
/// **MC-081 adds one** (its AC-4): the bottom of `e05`,
/// `2025-03-16 22_51_37.png`, one of its seven Eleceed screenshots. Its crop is
/// the right-hand (YouTube) window's column, rows 0..1400, and beside that
/// column the viewport stage declines, so the margin grows the bottom by 3
/// (1403 at margin 3). It is exactly that: the stage declining where it should
/// have cut, beside the wrong window. **Measured, not chosen**: this test's
/// own table on `ebdedd7` (release) in MC-081's RED, on a scratch copy with the
/// seven added (`column 0..1400 viewport None cut -- m3 (0, 1403) m0 (0,
/// 1400)`). `e02` and `e03`, where the stage declines too, have columns 0..1440,
/// the whole height, so no side has room. MC-082 takes `e05` off.
const MARGIN_REACHED_SIDES: [(&str, &str); 3] = [
    ("Screenshot (3605).png", "bottom"),
    ("Screenshot (3606).png", "bottom"),
    ("2025-03-16 22_51_37.png", "bottom"), // MC-081's e05
];

/// The page column `detect` locates before the viewport stage and before the
/// margin: its first five stages, composed exactly as `decide.rs` composes
/// them, as `corpus_viewport_stage.rs` does. None of them reads `margin_px`.
fn page_column_of(img: &Luma, t: &Tuning) -> Rect {
    let first = trim_uniform(img, t).expect("a marked page is not one flat colour");
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t).expect("the content box is not one flat colour");
    page_column(img, textured_box(img, second, t), t)
}

/// The rows MC-048's viewport stage keeps beside `column`, as
/// `(first row, one past the last)`, or `None` where the stage declines. A
/// viewport sharing no row with the column is a decline too, because that is
/// how `detect` treats it.
fn viewport_rows(img: &Luma, column: Rect, t: &Tuning) -> Option<(u32, u32)> {
    let view = locate(img, column, t)?;
    (view.top.max(column.y) < view.bottom.min(column.y + column.h))
        .then_some((view.top, view.bottom))
}

/// One entry's row edges by AC-4 as amended: which sides the viewport stage
/// cut, and where each side must be at a given margin.
struct RowEdges {
    column: Rect,
    view: Option<(u32, u32)>,
    height: u32,
}

impl RowEdges {
    /// Whether the viewport stage cut the top: the viewport's first row is at
    /// or below the page column's.
    ///
    /// "At" counts. On eight entries (`Screenshot (1661).png` to `(3538).png`)
    /// the earlier stages already end the page column on the viewport's
    /// bottom row, 1392, so the stage moves nothing there, but that row is
    /// the browser window's edge and the margin stops at it. That is the
    /// user's wording of the amendment: *"except where they stop at the image
    /// edge or the browser window's edge"*.
    fn top_cut(&self) -> bool {
        self.view.is_some_and(|(top, _)| top >= self.column.y)
    }

    /// Whether the viewport stage cut the bottom: the viewport ends at or
    /// above the page column's end. See [`RowEdges::top_cut`] for why "at"
    /// counts.
    fn bottom_cut(&self) -> bool {
        self.view
            .is_some_and(|(_, bottom)| bottom <= self.column.y + self.column.h)
    }

    /// The crop's rows at margin `by`, `(first row, one past the last)`.
    /// Test-side arithmetic, deliberately not `margin::expand` or the stage's
    /// clamp, so a change to either cannot move both sides of the comparison.
    ///
    /// On a side the stage cut: the viewport's edge, at every margin - the
    /// margin never puts a row of chrome or taskbar back. On a side it did
    /// not cut: the page column's edge moved out by exactly `by`, clamped at
    /// the image edge.
    fn at(&self, by: u32) -> (u32, u32) {
        let top = match self.view {
            Some((top, _)) if self.top_cut() => top,
            _ => self.column.y.saturating_sub(by),
        };
        let bottom = match self.view {
            Some((_, bottom)) if self.bottom_cut() => bottom,
            _ => (self.column.y + self.column.h + by).min(self.height),
        };
        (top, bottom)
    }

    /// The sides the margin `by` reaches in full: sides the stage did not
    /// cut, with room for all of `by` before the image edge.
    fn reached_in_full(&self, by: u32) -> Vec<&'static str> {
        let top = !self.top_cut() && self.column.y >= by;
        let bottom = !self.bottom_cut() && self.column.y + self.column.h + by <= self.height;
        [(top, "top"), (bottom, "bottom")]
            .into_iter()
            .filter_map(|(reached, side)| reached.then_some(side))
            .collect()
    }
}

/// AC-4, as amended on 2026-09-28: each crop's top and bottom edge moves by
/// exactly the change in `margin_px` on that side and no more - clamped at the
/// image edge, or at the browser viewport's edge where MC-048's viewport stage
/// cut that side - and still contains the mark.
///
/// Whether the stage cut a side is read from the stage itself, not inferred
/// from the crop: [`page_column_of`] is the rect the stage is handed, and
/// [`viewport_rows`] is what `viewport::locate` finds beside it. A side is cut
/// where the viewport's edge lies inside the page column's rows or on its
/// edge ([`RowEdges::top_cut`]). Where the stage declines no side is cut (the
/// two WebPs and `n13` on `main`, `1d7a921`; none after MC-076).
///
/// Three detections per entry: `margin_px` 0, the margin before MC-049
/// ([`MARGIN_BEFORE_MC049`]), and `Tuning::default()`. Each must have exactly
/// [`RowEdges::at`] its own margin. So a margin that grows an uncut side by
/// more (or less) than the change fails, as does one that grows a cut side
/// past the viewport. The margin 0 leg also checks the premise: the stage's
/// cut, read here, is the one the pipeline made.
///
/// MC-081: the mark's rows are excused only for [`KNOWN_ROW_CLIPS`], each held
/// to its rect at margins 3 and 0, exact in both directions.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_top_and_bottom_edges_move_by_exactly_the_margin_change() {
    let t = Tuning::default();
    let bare = Tuning {
        margin_px: 0,
        ..Tuning::default()
    };
    let before = Tuning {
        margin_px: MARGIN_BEFORE_MC049,
        ..Tuning::default()
    };
    let mut rows = Vec::new();
    let mut wrong = Vec::new();
    let mut cut = 0usize;
    let mut reached: Vec<(String, &str)> = Vec::new();
    let mut known_row_clips_seen: Vec<String> = Vec::new();

    for (entry, mark) in marked() {
        let img = luma(&entry.path);
        let at = |tuning: &Tuning| {
            let r = detect(&img, tuning)
                .unwrap_or_else(|| panic!("{} is not a uniform image", entry.name()))
                .rect;
            (r.y, r.y + r.h)
        };
        let column = page_column_of(&img, &t);
        let edges = RowEdges {
            column,
            view: viewport_rows(&img, column, &t),
            height: img.height,
        };
        cut += usize::from(edges.top_cut()) + usize::from(edges.bottom_cut());
        reached.extend(
            edges
                .reached_in_full(MARGIN_BEFORE_MC049)
                .into_iter()
                .map(|side| (entry.name(), side)),
        );

        let mut ok = true;
        for (margin, tuning) in [
            (0, &bare),
            (MARGIN_BEFORE_MC049, &before),
            (t.margin_px, &t),
        ] {
            let (got, want) = (at(tuning), edges.at(margin));
            if got != want {
                ok = false;
                wrong.push(format!(
                    "{}: at margin {margin} rows {got:?}, want {want:?} (page column rows \
                     {}..{}, viewport {:?}, cut top {} bottom {})",
                    entry.name(),
                    column.y,
                    column.y + column.h,
                    edges.view,
                    edges.top_cut(),
                    edges.bottom_cut()
                ));
            }
        }
        let (top, bottom) = at(&t);
        let contains_rows = top <= mark.y && bottom >= mark.y + mark.h;
        // MC-081: a known row clip is excused only at exactly its pins, and
        // only while it still clips.
        let rect_at = |tuning: &Tuning| {
            let r = detect(&img, tuning)
                .unwrap_or_else(|| panic!("{} is not a uniform image", entry.name()))
                .rect;
            [r.x, r.y, r.w, r.h]
        };
        if let Some(&(_, m3, m0)) = KNOWN_ROW_CLIPS
            .iter()
            .find(|(file, _, _)| *file == entry.name())
        {
            known_row_clips_seen.push(entry.name());
            let got = (rect_at(&before), rect_at(&bare));
            if contains_rows {
                ok = false;
                wrong.push(format!(
                    "{}: a known row clip (KNOWN_ROW_CLIPS) whose rows {top}..{bottom} now \
                     contain the mark's {}..{} - take it off the list",
                    entry.name(),
                    mark.y,
                    mark.y + mark.h
                ));
            } else if got != (m3, m0) {
                ok = false;
                wrong.push(format!(
                    "{}: a known row clip (KNOWN_ROW_CLIPS) at margins \
                     {MARGIN_BEFORE_MC049} and 0 is {got:?}, pinned {:?}",
                    entry.name(),
                    (m3, m0)
                ));
            }
        } else if !contains_rows {
            ok = false;
            wrong.push(format!(
                "{}: at margin {} rows {top}..{bottom} do not contain the mark's {}..{}",
                entry.name(),
                t.margin_px,
                mark.y,
                mark.y + mark.h
            ));
        }
        rows.push(format!(
            "{:<30} column {:>4}..{:<4} viewport {:<14} cut {}{}  m{MARGIN_BEFORE_MC049} {:?}  m{} {:?}  {}",
            entry.name(),
            column.y,
            column.y + column.h,
            format!("{:?}", edges.view),
            if edges.top_cut() { "T" } else { "-" },
            if edges.bottom_cut() { "B" } else { "-" },
            at(&before),
            t.margin_px,
            at(&t),
            if ok { "ok" } else { "MOVED" }
        ));
    }
    for (file, _, _) in KNOWN_ROW_CLIPS {
        if !known_row_clips_seen.iter().any(|seen| seen == file) {
            wrong.push(format!(
                "{file}: a known row clip (KNOWN_ROW_CLIPS) that is not a marked tuning \
                 entry here"
            ));
        }
    }

    let printed = table(
        &format!(
            "AC-4: rows at margin {MARGIN_BEFORE_MC049} and {}; {cut} sides cut by the viewport \
             stage, {} sides the margin {MARGIN_BEFORE_MC049} reaches in full: {reached:?}",
            t.margin_px,
            reached.len()
        ),
        &rows,
    );
    assert!(
        wrong.is_empty(),
        "AC-4: on each side the rows may move by the change in margin_px, clamped at \
         the image edge or at the viewport's edge where the viewport stage cut that \
         side, and nothing else, and the crop must contain the mark - besides \
         MC-081's KNOWN_ROW_CLIPS, each held to its pins. {} failures:\n{}\n\n{printed}",
        wrong.len(),
        wrong.join("\n")
    );
    let named: Vec<(String, &str)> = MARGIN_REACHED_SIDES
        .iter()
        .map(|&(file, side)| (file.to_string(), side))
        .collect();
    assert_eq!(
        reached, named,
        "AC-4's control (moved by MC-076 AC-3 from the WebPs' tops): the margin \
         {MARGIN_BEFORE_MC049} must reach in full on exactly MARGIN_REACHED_SIDES - \
         the bottoms of (3605) and (3606) - or the test above never sees a row move; \
         any other side reached is a side the viewport stage did not cut. \
         `(file, side)`, `left` measured.\n\n{printed}"
    );
}
