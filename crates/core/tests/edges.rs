//! MC-004, AC-1 to AC-6: row and column edge profiles locate strong lines.
//!
//! The pipeline's second stage (`docs/wiki/architecture.md`, "cropper-core",
//! step 2). A profile entry is the **mean absolute luma difference** between
//! one line and the next, taken over the other axis and clipped to a rect; a
//! **strong line** is a run of consecutive profile indices at or above
//! `Tuning::edge_threshold`, merged into one `Line { start, end }` with
//! inclusive indices into the profile.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: `edge_threshold = 24`. It is read out of `Tuning::default()`
//!   and pinned by its own test below; no number in this file was calibrated.
//! * **Mechanical**: the four exported shapes. Pinned exactly.
//! * **Measured**: AC-5's texture, and only that. MC-003's checkerboard art is
//!   the *opposite* of what AC-5 needs - its adjacent-row mean absolute
//!   difference measures 165.7, so it is a strong line at every row - so the
//!   generator gained a second art texture, [`common::soft_art`], whose
//!   measured maximum is 9.36 at the fixture size used here and 13.20 over 200
//!   seeds and five sizes. Both numbers are in the story's handoff.
//!
//! # Comparing `f32`
//!
//! Two disciplines, chosen per assertion and never widened to paper over the
//! other:
//!
//! * **Exact equality** wherever the true mean is an integer. Every such
//!   fixture here is built so the sum of the integer differences along a line
//!   is an exact multiple of the line's length, and every sum is far below
//!   `2^24`, so the sum is exact in `f32` and the quotient is the exactly
//!   representable integer it should be, under any summation order. A wrong
//!   answer - a sum instead of a mean, a truncation, a dropped term - differs
//!   by at least 1.0, so exactness costs nothing and rules out a whole class
//!   of near-miss implementations that a tolerance would accept.
//! * **An explicit `EPS` of 1e-5**, in the one test whose true mean (31/3) is
//!   not representable at all. The plausible wrong answers there are 10.0
//!   (integer truncation) and 31.0 (a sum), which are 0.33 and 20.67 away -
//!   four to six orders of magnitude outside the tolerance. `EPS` is about 30
//!   ulps at that magnitude, which absorbs a `f64`-then-cast implementation
//!   without absorbing any wrong answer.

mod common;

use common::{Recipe, soft_art};
use cropper_core::edges::{Line, col_profile, row_profile, strong_lines};
use cropper_core::{Luma, Rect, Tuning};

// --- Comparison discipline --------------------------------------------------

/// See the module header. Used exactly once, and never as a substitute for
/// exact equality where exact equality holds.
const EPS: f32 = 1e-5;

fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() <= EPS,
        "{what}: expected {expected} +/- {EPS}, got {actual}"
    );
}

// --- Fixtures ---------------------------------------------------------------

/// The rect covering the whole of `img`.
fn whole(img: &Luma) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    }
}

/// Rows `0..k` are all `a`, rows `k..h` are all `b`. AC-1's image: the only
/// adjacent row pair that differs is `(k - 1, k)`, and every column is
/// constant, so the true row profile is `|a - b|` at index `k - 1` and 0
/// everywhere else, and the true column profile is all 0.
fn two_bands(w: u32, h: u32, k: u32, a: u8, b: u8) -> Luma {
    let mut data = vec![0u8; w as usize * h as usize];
    for (y, row) in data.chunks_mut(w as usize).enumerate() {
        row.fill(if (y as u32) < k { a } else { b });
    }
    Luma {
        width: w,
        height: h,
        data,
    }
}

/// The transpose of [`two_bands`]: columns `0..k` are `a`, columns `k..w` are
/// `b`. Exists so a row/column swap in the implementation cannot hide behind
/// AC-1's all-zero column profile.
fn two_columns(w: u32, h: u32, k: u32, a: u8, b: u8) -> Luma {
    let mut data = vec![0u8; w as usize * h as usize];
    for row in data.chunks_mut(w as usize) {
        for (x, px) in row.iter_mut().enumerate() {
            *px = if (x as u32) < k { a } else { b };
        }
    }
    Luma {
        width: w,
        height: h,
        data,
    }
}

/// An image built from explicit rows, for the two mean-arithmetic tests.
fn from_rows(rows: &[&[u8]]) -> Luma {
    let w = rows[0].len();
    assert!(
        rows.iter().all(|r| r.len() == w),
        "every row must be the same length"
    );
    Luma {
        width: w as u32,
        height: rows.len() as u32,
        data: rows.iter().flat_map(|r| r.iter().copied()).collect(),
    }
}

/// AC-2's sub-rect scene. A 12x10 image that is `outside` everywhere except
/// [`SUB`], which holds a two-band pattern: rows 2..4 are 40 and rows 4..7 are
/// 200. Inside the rect the true row profile is `[0, 160, 0, 0]` and the true
/// column profile is `[0, 0, 0, 0, 0]`, whatever `outside` is.
const SUB: Rect = Rect {
    x: 3,
    y: 2,
    w: 6,
    h: 5,
};

fn banded_inside(outside: u8) -> Luma {
    let mut data = vec![outside; 12 * 10];
    for (y, row) in data.chunks_mut(12).enumerate().skip(2).take(5) {
        row[3..9].fill(if y < 4 { 40 } else { 200 });
    }
    Luma {
        width: 12,
        height: 10,
        data,
    }
}

/// AC-2's column scene, the mirror of [`banded_inside`]. A 6x6 image that is
/// `outside` everywhere except [`SUB_COLS`], whose columns 1..3 are 10 and
/// columns 3..5 are 30. Inside the rect the true column profile is
/// `[0, 20, 0]` - which is also the sum-versus-mean discriminator for
/// `col_profile`, since the rect is four rows tall and the sum would be 80.
const SUB_COLS: Rect = Rect {
    x: 1,
    y: 1,
    w: 4,
    h: 4,
};

fn columned_inside(outside: u8) -> Luma {
    let mut data = vec![outside; 6 * 6];
    for row in data.chunks_mut(6).skip(1).take(4) {
        row[1..3].fill(10);
        row[3..5].fill(30);
    }
    Luma {
        width: 6,
        height: 6,
        data,
    }
}

// --- The settled constant and the pinned shapes -----------------------------

/// `edge_threshold` is settled at 24 (`architecture.md`, the `Tuning` table;
/// the corpus story MC-019 may change it under `## Amendments`). Every
/// threshold in this file is 24 *because of this test*, not because 24 was
/// assumed - exactly as MC-003 pinned `uniform_tolerance`.
#[test]
fn the_default_edge_threshold_is_twenty_four() {
    assert_eq!(
        Tuning::default().edge_threshold,
        24,
        "the settled default edge threshold"
    );
}

#[test]
fn line_is_a_plain_comparable_copyable_value_type() {
    fn assert_value_type<T: std::fmt::Debug + Clone + Copy + PartialEq + Eq>(_: &T) {}

    let a = Line { start: 3, end: 7 };
    let same = a;
    assert_value_type(&a);
    assert_eq!(a, same, "Line is Copy, so `same` is a copy and still equal");
    assert_ne!(
        a,
        Line { start: 3, end: 8 },
        "two Lines with different ends are different lines"
    );
    assert_eq!(a.start, 3usize, "start is a usize index into the profile");
    assert_eq!(a.end, 7usize, "end is a usize index into the profile");
    assert!(
        format!("{a:?}").contains('7'),
        "Debug shows the fields, so a failure message names the indices: {a:?}"
    );
}

// --- AC-1: a two-band image ------------------------------------------------

#[test]
fn a_two_band_image_has_one_row_profile_entry_at_the_seam_and_zero_elsewhere() {
    let img = two_bands(9, 7, 3, 30, 190);

    assert_eq!(
        row_profile(&img, whole(&img)),
        vec![0.0, 0.0, 160.0, 0.0, 0.0, 0.0],
        "AC-1: H - 1 = 6 entries, |a - b| = 160 at index k - 1 = 2, 0 elsewhere"
    );
}

#[test]
fn a_two_band_image_has_an_all_zero_column_profile() {
    let img = two_bands(9, 7, 3, 30, 190);

    assert_eq!(
        col_profile(&img, whole(&img)),
        vec![0.0; 8],
        "AC-1: every row is constant, so W - 1 = 8 column entries are all 0"
    );
}

#[test]
fn a_two_column_image_has_the_seam_in_the_column_profile_and_an_empty_signal_in_the_row_profile() {
    let img = two_columns(9, 7, 4, 30, 190);

    assert_eq!(
        col_profile(&img, whole(&img)),
        vec![0.0, 0.0, 0.0, 160.0, 0.0, 0.0, 0.0, 0.0],
        "AC-1 transposed: 160 at column index k - 1 = 3"
    );
    assert_eq!(
        row_profile(&img, whole(&img)),
        vec![0.0; 6],
        "AC-1 transposed: every column is constant, so the row profile is all 0"
    );
}

// --- AC-2: only the rect contributes ---------------------------------------

#[test]
fn the_profiles_of_a_sub_rect_have_one_entry_per_adjacent_pair_inside_it() {
    let img = banded_inside(0);

    assert_eq!(
        row_profile(&img, SUB).len(),
        SUB.h as usize - 1,
        "AC-2: rect.h - 1 = 4 row entries, not the image's 9"
    );
    assert_eq!(
        col_profile(&img, SUB).len(),
        SUB.w as usize - 1,
        "AC-2: rect.w - 1 = 5 column entries, not the image's 11"
    );
}

#[test]
fn a_sub_rect_row_profile_sees_only_the_seam_inside_the_rect() {
    let img = banded_inside(0);

    assert_eq!(
        row_profile(&img, SUB),
        vec![0.0, 160.0, 0.0, 0.0],
        "AC-2: the 40-to-200 seam is at rect row index 1"
    );
}

#[test]
fn a_bright_band_outside_the_rect_changes_neither_profile() {
    let dark = banded_inside(0);
    let bright = banded_inside(255);

    assert_eq!(
        row_profile(&bright, SUB),
        row_profile(&dark, SUB),
        "AC-2: pixels outside the rect must not reach the row profile"
    );
    assert_eq!(
        col_profile(&bright, SUB),
        col_profile(&dark, SUB),
        "AC-2: pixels outside the rect must not reach the column profile"
    );
    assert_eq!(
        row_profile(&bright, SUB),
        vec![0.0, 160.0, 0.0, 0.0],
        "AC-2: and the shared answer is the one the rect's own pixels give, \
         not an all-zero profile that would make the comparison above vacuous"
    );
}

#[test]
fn a_sub_rect_column_profile_is_the_mean_over_the_rects_rows_only() {
    let dark = columned_inside(0);
    let bright = columned_inside(255);

    assert_eq!(
        col_profile(&dark, SUB_COLS),
        vec![0.0, 20.0, 0.0],
        "AC-2: the 10-to-30 seam is at rect column index 1, and 20 is the \
         *mean* over the rect's 4 rows - the sum would be 80"
    );
    assert_eq!(
        col_profile(&bright, SUB_COLS),
        col_profile(&dark, SUB_COLS),
        "AC-2: the surround does not reach a column profile either"
    );
}

// --- The profile entry is a mean --------------------------------------------

#[test]
fn a_row_profile_entry_is_the_mean_of_the_column_differences_not_their_sum() {
    let img = from_rows(&[&[10, 20, 30, 40], &[20, 40, 60, 80]]);

    assert_eq!(
        row_profile(&img, whole(&img)),
        vec![25.0],
        "differences 10, 20, 30, 40 over 4 columns: the mean is 25, the sum is 100"
    );
}

#[test]
fn a_row_profile_entry_keeps_the_fractional_part_of_the_mean() {
    let img = from_rows(&[&[10, 10, 10], &[20, 20, 21]]);
    let profile = row_profile(&img, whole(&img));

    assert_eq!(profile.len(), 1, "one adjacent row pair");
    assert_close(
        profile[0],
        31.0 / 3.0,
        "differences 10, 10, 11 over 3 columns: 31/3 = 10.333333, not 10 \
         (integer truncation) and not 31 (a sum)",
    );
}

// --- AC-3: runs merge, singletons stay --------------------------------------

#[test]
fn strong_lines_merges_a_run_of_adjacent_strong_indices_and_keeps_a_lone_one_separate() {
    let mut profile = vec![0.0f32; 45];
    // At or above the threshold at 10, 11, 12 and 40, and nowhere else. The
    // two near misses either side of a strong index are the sharp part: a
    // line must not grow into them.
    profile[9] = 23.5;
    profile[10] = 24.0;
    profile[11] = 30.0;
    profile[12] = 24.0;
    profile[13] = 23.9;
    profile[40] = 99.0;
    profile[41] = 23.0;

    assert_eq!(
        strong_lines(&profile, &Tuning::default()),
        vec![Line { start: 10, end: 12 }, Line { start: 40, end: 40 }],
        "AC-3: indices 10..=12 merge into one line; 40 stands alone; the \
         23.x neighbours are below the threshold and extend nothing"
    );
}

#[test]
fn a_run_of_strong_values_at_the_start_of_the_profile_becomes_one_line_from_zero() {
    assert_eq!(
        strong_lines(&[30.0, 30.0, 30.0, 0.0, 0.0], &Tuning::default()),
        vec![Line { start: 0, end: 2 }],
        "AC-3: a run touching index 0 starts at 0"
    );
}

#[test]
fn a_run_of_strong_values_at_the_end_of_the_profile_reaches_the_last_index() {
    assert_eq!(
        strong_lines(&[0.0, 0.0, 30.0, 30.0], &Tuning::default()),
        vec![Line { start: 2, end: 3 }],
        "AC-3: a run touching the last index is closed at it, not dropped"
    );
}

#[test]
fn two_runs_separated_by_a_single_weak_index_stay_two_lines() {
    assert_eq!(
        strong_lines(&[30.0, 30.0, 0.0, 30.0, 30.0], &Tuning::default()),
        vec![Line { start: 0, end: 1 }, Line { start: 3, end: 4 }],
        "AC-3: merging is by adjacency, so one weak index separates two lines"
    );
}

#[test]
fn a_single_strong_value_is_one_line_of_one_index() {
    assert_eq!(
        strong_lines(&[30.0], &Tuning::default()),
        vec![Line { start: 0, end: 0 }],
        "AC-3, the `one` case: a one-entry profile"
    );
}

#[test]
fn a_profile_with_every_value_below_the_threshold_has_no_strong_lines() {
    assert_eq!(
        strong_lines(&[23.0; 12], &Tuning::default()),
        vec![],
        "AC-3 control: 23 is below 24, so nothing here is a line"
    );
}

#[test]
fn the_edge_threshold_is_read_from_the_tuning_and_not_hard_coded() {
    let profile = [0.0, 0.0, 0.0, 0.0, 0.0, 24.0, 0.0];

    assert_eq!(
        strong_lines(&profile, &Tuning::default()),
        vec![Line { start: 5, end: 5 }],
        "at the settled threshold of 24, index 5 is a line"
    );
    assert_eq!(
        strong_lines(
            &profile,
            &Tuning {
                edge_threshold: 25,
                ..Default::default()
            }
        ),
        vec![],
        "the same profile with the threshold raised to 25 has no line, so \
         the threshold is read from the argument"
    );
}

// --- AC-4: the boundary, and the control on it ------------------------------

#[test]
fn a_seam_of_exactly_the_edge_threshold_is_one_strong_line() {
    let img = two_bands(9, 7, 3, 100, 124);
    let profile = row_profile(&img, whole(&img));

    assert_eq!(
        profile,
        vec![0.0, 0.0, 24.0, 0.0, 0.0, 0.0],
        "AC-4: |a - b| = 24 is exactly the threshold"
    );
    assert_eq!(
        strong_lines(&profile, &Tuning::default()),
        vec![Line { start: 2, end: 2 }],
        "AC-4: `at or above` includes the threshold itself, so one line at k - 1 = 2"
    );
}

#[test]
fn a_seam_one_below_the_edge_threshold_is_not_a_line() {
    let img = two_bands(9, 7, 3, 100, 123);
    let profile = row_profile(&img, whole(&img));

    assert_eq!(
        profile,
        vec![0.0, 0.0, 23.0, 0.0, 0.0, 0.0],
        "AC-4 control: the fixture's seam is 23, one below the threshold"
    );
    assert_eq!(
        strong_lines(&profile, &Tuning::default()),
        vec![],
        "AC-4 control on the number: 23 must produce no line at all"
    );
}

// --- AC-5: the art's own texture is not an edge -----------------------------

/// The generator's soft art texture, at eight seeds. Violations are collected
/// and asserted once rather than per seed, so a failure names every seed that
/// broke rather than only the first.
#[test]
fn the_soft_art_texture_produces_no_strong_lines_in_either_direction() {
    let tuning = Tuning::default();
    let mut lines = Vec::new();
    let mut worst = 0.0f32;
    let mut mildest_peak = f32::INFINITY;

    for seed in 1..=8u32 {
        let img = soft_art(48, 36, seed);
        for (axis, profile) in [
            ("row", row_profile(&img, whole(&img))),
            ("column", col_profile(&img, whole(&img))),
        ] {
            let found = strong_lines(&profile, &tuning);
            if !found.is_empty() {
                lines.push(format!("seed {seed} {axis}: {found:?}"));
            }
            let peak = profile.iter().copied().fold(0.0f32, f32::max);
            worst = worst.max(peak);
            mildest_peak = mildest_peak.min(peak);
        }
    }

    assert!(
        lines.is_empty(),
        "AC-5: the art's own texture must not read as an edge, but found {lines:?}"
    );
    assert!(
        worst < 16.0,
        "AC-5: the measured maximum adjacent-line mean absolute difference of \
         the soft art texture is 9.36112 (recorded in MC-004's handoff), which \
         must stay below the story's 16 target and so well below the threshold \
         of {}; measured {worst} here",
        tuning.edge_threshold
    );
    assert!(
        mildest_peak >= 4.0,
        "AC-5 control: this test would pass vacuously on a flat field, so the \
         fixture must have a real signal in every profile - the smallest peak \
         measured 7.64 in RED; measured {mildest_peak} here"
    );
}

#[test]
fn the_soft_art_texture_spans_far_more_than_the_uniform_tolerance() {
    let img = soft_art(48, 36, 1);
    let lo = img.data.iter().copied().min().expect("a non-empty fixture");
    let hi = img.data.iter().copied().max().expect("a non-empty fixture");

    assert!(
        hi - lo >= 64,
        "AC-5 control: `textured noise` means textured. The field must span \
         far more than uniform_tolerance ({}) or AC-5 is a statement about a \
         flat image; measured spread {} (RED measured 96)",
        Tuning::default().uniform_tolerance,
        hi - lo
    );
}

/// The paired negative control for AC-5, and the reason the test above is not
/// decoration: MC-003's checkerboard art is the *same* code path with the
/// opposite answer. Its adjacent-row mean absolute difference measures 157.6
/// to 162.6, so every index is strong and the whole profile merges into a
/// single line.
#[test]
fn the_high_contrast_checkerboard_art_is_one_strong_line_spanning_the_whole_profile() {
    let img = Recipe::new(48, 36).render();
    let rows = row_profile(&img, whole(&img));
    let cols = col_profile(&img, whole(&img));

    assert!(
        rows.iter().all(|&v| v >= 24.0) && cols.iter().all(|&v| v >= 24.0),
        "the checkerboard art is above the threshold everywhere: rows {rows:?}, columns {cols:?}"
    );
    assert_eq!(
        strong_lines(&rows, &Tuning::default()),
        vec![Line { start: 0, end: 34 }],
        "35 row entries, all strong, merge into one line covering 0..=34"
    );
    assert_eq!(
        strong_lines(&cols, &Tuning::default()),
        vec![Line { start: 0, end: 46 }],
        "47 column entries, all strong, merge into one line covering 0..=46"
    );
}

// --- AC-6: degenerate heights and widths ------------------------------------

#[test]
fn an_image_one_row_tall_has_an_empty_row_profile() {
    let img = from_rows(&[&[10, 10, 40, 40, 40]]);

    assert_eq!(
        row_profile(&img, whole(&img)),
        vec![],
        "AC-6: H = 1, so there is no adjacent row pair"
    );
    assert_eq!(
        col_profile(&img, whole(&img)),
        vec![0.0, 30.0, 0.0, 0.0],
        "AC-6: the column profile is still W - 1 = 4 entries over that one row"
    );
}

#[test]
fn a_rect_one_row_tall_has_an_empty_row_profile_inside_a_taller_image() {
    let img = two_bands(5, 8, 4, 50, 90);
    let one_row = Rect {
        x: 0,
        y: 2,
        w: 5,
        h: 1,
    };

    assert_eq!(
        row_profile(&img, one_row),
        vec![],
        "AC-6: rect.h = 1 inside an 8-row image is still no adjacent pair"
    );
    assert_eq!(
        col_profile(&img, one_row),
        vec![0.0; 4],
        "AC-6: rect.w - 1 = 4 column entries over that single constant row"
    );
}

#[test]
fn a_rect_one_column_wide_has_an_empty_column_profile() {
    let img = two_bands(5, 8, 4, 50, 90);
    let one_column = Rect {
        x: 2,
        y: 0,
        w: 1,
        h: 6,
    };

    assert_eq!(
        col_profile(&img, one_column),
        vec![],
        "AC-6 mirrored: rect.w = 1, so there is no adjacent column pair"
    );
    assert_eq!(
        row_profile(&img, one_column),
        vec![0.0, 0.0, 0.0, 40.0, 0.0],
        "AC-6 mirrored: rect.h - 1 = 5 row entries down that single column, \
         with the 50-to-90 seam at index 3"
    );
}

#[test]
fn an_image_with_no_rows_at_all_has_an_empty_row_profile() {
    let img = Luma {
        width: 4,
        height: 0,
        data: vec![],
    };

    assert_eq!(
        row_profile(&img, whole(&img)),
        vec![],
        "AC-6: H = 0 satisfies `H <= 1`; `h - 1` must not wrap round"
    );
}

#[test]
fn strong_lines_of_an_empty_profile_is_empty() {
    assert_eq!(
        strong_lines(&[], &Tuning::default()),
        vec![],
        "AC-6: no profile entries, no lines"
    );
}
