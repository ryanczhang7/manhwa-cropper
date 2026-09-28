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
//!   marks; "both margins" (`Tuning::default()` and `margin_px: 0`); AC-2's
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
    (
        "2025-08-05 00_11_13.webp",
        [950, 15, 646, 1425],
        [953, 18, 640, 1422],
    ),
    (
        "2025-08-05 00_11_27.webp",
        [1003, 15, 539, 1425],
        [1006, 18, 533, 1422],
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
    (
        "Screenshot (3538).png",
        [972, 137, 602, 1255],
        [975, 137, 596, 1255],
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

/// Both margins, as the story defines them.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning::default(),
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
            if t.margin_px == Tuning::default().margin_px {
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
    let (sides, failing, not_cropped) = page_background_let_in(&entries, &t, &tmp);
    let shown: Vec<String> = failing
        .iter()
        .map(|(name, side, cols)| format!("{name} {side:?}: page-background columns {cols:?}"))
        .collect();
    assert_eq!(
        entries.len(),
        26,
        "MC-053 (MC-055) AC-2 is over the 26 marked tuning entries"
    );
    assert!(
        not_cropped.is_empty(),
        "MC-053 (MC-055) AC-2: every marked tuning entry must be cropped; not cropped: {not_cropped:?}"
    );
    assert_eq!(sides, 52, "MC-053 (MC-055) AC-2 checks 52 sides");
    assert!(
        failing.is_empty(),
        "MC-053 (MC-055) AC-2: at margin_px 0 no column the crop keeps outside the mark may \
         be page background (share >= {PAGE_BACKGROUND_SHARE} within {} of the \
         column's median, over the mark's rows). {} of {sides} sides let it in:\n{}",
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

/// The stand-in, earned: on each of the 23 originals, MC-027's rule as frozen
/// here gives exactly the columns the shipped pipeline crops to at margin 0 -
/// on `c004d96`, and after GREEN too, because AC-3 holds the originals exactly.
/// The three are left out only because GREEN is required to move them; in RED
/// the stand-in matched them as well (26 of 26).
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
        if THE_THREE.contains(&entry.name().as_str()) {
            continue;
        }
        compared += 1;
        let stand_in = mc027_column(&luma(&entry.path), &t);
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
        .collect();
    assert_eq!(
        entries.len(),
        26,
        "the control is over the 26 marked tuning entries"
    );
    assert!(
        ok >= METRIC_CONTROL_REQUIRED && unexplained.is_empty(),
        "MC-053 (MC-055) AC-2's control on the metric: a mark's own first and last columns \
         are art, so the predicate must call both not page background on at least \
         {METRIC_CONTROL_REQUIRED} of {} entries (it did on {ok}), and may call a \
         mark edge background only on MC-049's seven mark errors (it also did on \
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
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_twenty_three_original_crops_do_not_move_by_a_pixel_at_either_margin() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut moved = Vec::new();
    let mut seen = Vec::new();
    for (entry, _) in marked() {
        let name = entry.name();
        if THE_THREE.contains(&name.as_str()) {
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
