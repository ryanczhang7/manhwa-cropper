//! MC-005, AC-1 to AC-7: chrome strips outside the strong edges are peeled
//! away.
//!
//! The pipeline's third stage (`docs/wiki/architecture.md`, "cropper-core",
//! step 3, and decision 13). On each side of the current rect the *outermost
//! strip* is the band between that edge and the nearest strong line (MC-004)
//! parallel to it. The strip is **chrome-like** when all three hold:
//!
//! 1. its extent is at most `chrome_max_extent` of the **image's** dimension
//!    on that axis - height for a horizontal strip, width for a vertical one;
//! 2. its **flat fraction** - the share of its pixels within
//!    `uniform_tolerance` of the strip's median luma - is at least
//!    `chrome_flat_fraction`;
//! 3. what is left after removing it still has luma standard deviation at or
//!    above `min_content_stddev`.
//!
//! A chrome-like strip is removed; the sides are visited `Top, Bottom, Left,
//! Right` and the pass repeats until one removes nothing. A strip that is
//! *otherwise* a chrome candidate and misses on flatness alone, by less than
//! `ambiguity_band`, is kept and marks the result ambiguous - the qualified
//! reading settled in MC-005's `## Notes`, which is why two tests below check
//! that a strip failing the extent or the stddev condition does **not** flag
//! ambiguity however flat it is.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: `chrome_flat_fraction = 0.85`, `chrome_max_extent = 0.30`,
//!   `ambiguity_band = 0.05`, `min_content_stddev = 12`,
//!   `uniform_tolerance = 10`, `edge_threshold = 24`. Every one is read out of
//!   `Tuning::default()` and pinned by its own test; not one number in this
//!   file was calibrated, and no threshold is ever written as a literal at a
//!   comparison site.
//! * **Mechanical**: `content_box`, `ContentBox` and `Side`. Pinned exactly.
//! * **Measured**: only the fixtures. The chrome band generator
//!   ([`common::chrome_band`]) asserts its own flat fraction on every call, and
//!   the tests below re-measure it, the art backdrops' standard deviation and
//!   flat fraction, and the seam between them, so a fixture that drifts fails
//!   loudly instead of quietly moving a threshold. Every measured value is in
//!   the story's handoff.
//!
//! # Two numeric notes for whoever implements this
//!
//! * The flat fraction and the extent share are compared against `f32` fields
//!   of `Tuning`, and three tests below sit **exactly** on a boundary (flat
//!   fraction 0.85, flat fraction 0.80 = `0.85 - 0.05`, extent 30 of 100).
//!   Do the arithmetic in `f32`, as the fields are: `510f32 / 600f32` is bit
//!   for bit `0.85f32`, so `>=` holds, while the same quotient in `f64` is one
//!   ulp *below* `f64::from(0.85f32)` and the boundary silently moves.
//! * `stddev` is a population standard deviation here, but every fixture is
//!   far from `min_content_stddev` (18.06 or 30.39 against 12 where it must
//!   pass, 5.00 against 12 where it must fail), so the sample/population
//!   choice is not pinned and is yours.

mod common;

use common::{chrome_band, flat_band, soft_art};
use cropper_core::content::{ContentBox, Side, content_box};
use cropper_core::edges::{col_profile, row_profile, strong_lines};
use cropper_core::{Luma, Rect, Tuning};

// --- Fixture constants ------------------------------------------------------
//
// None of these is a threshold. Thresholds are read from `Tuning::default()`
// at every use; these are the sizes and colours of the synthetic screenshots.

/// Every scene is 100x100, so a percentage in an acceptance criterion is a
/// pixel count: AC-1's "6% of H" is 6 rows, AC-2's "15% of W" is 15 columns,
/// AC-4's 31% and 29% are 31 and 29 rows.
const W: u32 = 100;
const H: u32 = 100;

/// The chrome background: a light toolbar.
const BAND_BG: u8 = 200;
/// A slightly lighter sidebar, so the sidebar/band seam is not zero either.
const SIDEBAR_BG: u8 = 230;

/// How far a band's "text" pixels sit from its background. Twice
/// `uniform_tolerance` (10), so those pixels are unambiguously *not* flat,
/// and small enough that no run of them ever reaches `edge_threshold` (24):
/// the sharpest edge inside any band in this file measures 20 across adjacent
/// columns and 8.4 across adjacent rows.
const DEVIATION: u8 = 20;

/// Seed for [`common::soft_art`]. Fixed, because every expectation in this
/// file is a rect, not a pixel value.
const SEED: u32 = 7;

/// The "art" of AC-7, which is nearly flat: a checkerboard of 40 and
/// `40 + FLAT_ART_SPREAD`, whose standard deviation is exactly half the
/// spread - 5.0, comfortably below `min_content_stddev`.
const FLAT_ART_LOW: u8 = 40;
const FLAT_ART_SPREAD: u8 = 10;

// --- Fixture builders -------------------------------------------------------

/// The rect covering the whole of `img`.
fn whole(img: &Luma) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    }
}

/// Blocks of equal width, stacked top to bottom.
fn stack(blocks: &[Luma]) -> Luma {
    let width = blocks[0].width;
    let mut data = Vec::new();
    let mut height = 0;
    for block in blocks {
        assert_eq!(block.width, width, "stacked blocks must share a width");
        data.extend_from_slice(&block.data);
        height += block.height;
    }
    Luma {
        width,
        height,
        data,
    }
}

/// Blocks of equal height, placed left to right.
fn beside(blocks: &[Luma]) -> Luma {
    let height = blocks[0].height;
    let width: u32 = blocks.iter().map(|block| block.width).sum();
    let mut data = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height as usize {
        for block in blocks {
            assert_eq!(block.height, height, "adjacent blocks must share a height");
            let w = block.width as usize;
            data.extend_from_slice(&block.data[y * w..(y + 1) * w]);
        }
    }
    Luma {
        width,
        height,
        data,
    }
}

/// A chrome band at the settled `uniform_tolerance`, which is the tolerance
/// its flat fraction is defined against. Read out, never written down.
fn band(width: u32, height: u32, background: u8, flat_fraction: f64) -> Luma {
    chrome_band(
        width,
        height,
        background,
        DEVIATION,
        Tuning::default().uniform_tolerance,
        flat_fraction,
    )
}

/// Textured art whose own texture is neither an edge nor chrome: measured
/// below at stddev 18.06 and flat fraction 0.41.
fn art(width: u32, height: u32) -> Luma {
    soft_art(width, height, SEED)
}

/// AC-1's scene, and AC-3's and AC-4's with the band's shape varied: a chrome
/// band across the top of the image, textured art filling the rest.
fn top_band_scene(band_h: u32, flat_fraction: f64) -> Luma {
    stack(&[band(W, band_h, BAND_BG, flat_fraction), art(W, H - band_h)])
}

/// The art rect of [`top_band_scene`]: everything below the band.
fn below_band(band_h: u32) -> Rect {
    Rect {
        x: 0,
        y: band_h,
        w: W,
        h: H - band_h,
    }
}

/// A one-line seam that is itself textured: 130 and 170 alternating, so the
/// difference to a band on one side and to the art on the other are both past
/// `edge_threshold` and the strong run covers **two** profile indices rather
/// than one. Textured on purpose - a flat seam line would be a one-pixel
/// chrome strip of its own on the next pass, which is a different behaviour
/// from the one these tests are about.
fn textured_seam(width: u32, height: u32) -> Luma {
    flat_band(width, height, 130, 40)
}

/// A band whose "text" is 10% of its columns, dark, in the same columns on
/// every row: median 200, mean 186. The two are further apart than
/// `uniform_tolerance`, which is what makes the flat fraction about the median
/// (0.90) and about the mean (0.00) disagree.
fn dark_dashed_band(width: u32, height: u32) -> Luma {
    let mut data = vec![BAND_BG; width as usize * height as usize];
    for (i, px) in data.iter_mut().enumerate() {
        if (i % width as usize).is_multiple_of(10) {
            *px = 60;
        }
    }
    Luma {
        width,
        height,
        data,
    }
}

/// AC-2's scene: AC-1's, plus a chrome sidebar down the left of the art.
fn top_band_and_sidebar(sidebar_flat: f64) -> Luma {
    stack(&[
        band(W, 6, BAND_BG, 0.90),
        beside(&[band(15, 94, SIDEBAR_BG, sidebar_flat), art(W - 15, 94)]),
    ])
}

// --- Measurement, so a drifting fixture fails loudly ------------------------

/// The pixels of `rect`, row by row.
fn pixels(img: &Luma, rect: Rect) -> Vec<u8> {
    let stride = img.width as usize;
    let mut out = Vec::with_capacity(rect.w as usize * rect.h as usize);
    for dy in 0..rect.h as usize {
        let start = (rect.y as usize + dy) * stride + rect.x as usize;
        out.extend_from_slice(&img.data[start..start + rect.w as usize]);
    }
    out
}

/// The share of `rect`'s pixels within `tolerance` of its median luma - the
/// definition `chrome_flat_fraction` is measured against, written out here so
/// the tests can check what a fixture actually is rather than what it was
/// asked to be.
fn flat_fraction(img: &Luma, rect: Rect, tolerance: u8) -> f64 {
    let mut values = pixels(img, rect);
    values.sort_unstable();
    let median = values[values.len() / 2];
    let flat = values
        .iter()
        .filter(|&&px| px.abs_diff(median) <= tolerance)
        .count();
    flat as f64 / values.len() as f64
}

/// Population standard deviation of `rect`'s luma.
fn stddev(img: &Luma, rect: Rect) -> f64 {
    let values = pixels(img, rect);
    let n = values.len() as f64;
    let mean = values.iter().map(|&px| f64::from(px)).sum::<f64>() / n;
    let variance = values
        .iter()
        .map(|&px| (f64::from(px) - mean).powi(2))
        .sum::<f64>()
        / n;
    variance.sqrt()
}

/// The mean luma of a single row of `img`.
fn row_mean(img: &Luma, y: u32) -> f64 {
    let row = pixels(
        img,
        Rect {
            x: 0,
            y,
            w: img.width,
            h: 1,
        },
    );
    row.iter().map(|&px| f64::from(px)).sum::<f64>() / row.len() as f64
}

// --- The settled constants --------------------------------------------------

#[test]
fn the_default_chrome_flat_fraction_is_zero_point_eight_five() {
    assert_eq!(Tuning::default().chrome_flat_fraction, 0.85);
}

#[test]
fn the_default_chrome_max_extent_is_zero_point_three() {
    assert_eq!(Tuning::default().chrome_max_extent, 0.30);
}

#[test]
fn the_default_ambiguity_band_is_zero_point_zero_five() {
    assert_eq!(Tuning::default().ambiguity_band, 0.05);
}

#[test]
fn the_default_min_content_stddev_is_twelve() {
    assert_eq!(Tuning::default().min_content_stddev, 12.0);
}

#[test]
fn the_default_uniform_tolerance_is_ten() {
    assert_eq!(Tuning::default().uniform_tolerance, 10);
}

#[test]
fn the_default_edge_threshold_is_twenty_four() {
    assert_eq!(Tuning::default().edge_threshold, 24);
}

// --- The exported shapes ----------------------------------------------------

#[test]
fn side_is_a_plain_comparable_copyable_value_type() {
    let top = Side::Top;
    // Copy: `top` is still usable after being moved into the comparison.
    let same = top;
    assert_eq!(top, same);
    // Clone, Debug, and four distinct variants.
    #[allow(clippy::clone_on_copy)]
    let cloned = top.clone();
    assert_eq!(cloned, Side::Top);
    let all = [Side::Top, Side::Bottom, Side::Left, Side::Right];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(
                a == b,
                i == j,
                "{a:?} and {b:?} must compare equal only to themselves"
            );
        }
    }
}

#[test]
fn a_content_box_carries_a_rect_a_removal_order_and_an_ambiguity_flag() {
    let scene = top_band_scene(6, 0.90);
    let ContentBox {
        rect,
        removed,
        ambiguous,
    } = content_box(&scene, whole(&scene), &Tuning::default());
    let _: Rect = rect;
    let _: Vec<Side> = removed;
    let _: bool = ambiguous;
    // Debug, so a failing assertion elsewhere prints something readable.
    let printed = format!(
        "{:?}",
        content_box(&scene, whole(&scene), &Tuning::default())
    );
    assert!(!printed.is_empty());
}

// --- The fixtures are what they claim to be ---------------------------------

#[test]
fn the_chrome_band_generator_hits_every_requested_flat_fraction() {
    let tolerance = Tuning::default().uniform_tolerance;
    let mut drifted = Vec::new();
    for &requested in &[0.50f64, 0.79, 0.80, 0.84, 0.85, 0.86, 0.90, 0.95] {
        for &(w, h) in &[(W, 6u32), (W, 29), (W, 30), (W, 31), (15, 94)] {
            let fixture = band(w, h, BAND_BG, requested);
            let rendered = flat_fraction(&fixture, whole(&fixture), tolerance);
            if (rendered - requested).abs() > common::CHROME_BAND_FLAT_TOLERANCE {
                drifted.push(format!("{w}x{h} asked {requested}, rendered {rendered}"));
            }
        }
    }
    assert!(
        drifted.is_empty(),
        "the chrome band generator missed its requested flat fraction: {drifted:?}"
    );
}

#[test]
fn a_chrome_band_carries_no_strong_line_of_its_own() {
    let t = Tuning::default();
    let mut split = Vec::new();
    for &requested in &[0.50f64, 0.79, 0.80, 0.84, 0.85, 0.86, 0.90, 0.95] {
        for &(w, h) in &[(W, 6u32), (W, 31), (15, 94)] {
            let fixture = band(w, h, BAND_BG, requested);
            let rows = strong_lines(&row_profile(&fixture, whole(&fixture)), &t);
            let cols = strong_lines(&col_profile(&fixture, whole(&fixture)), &t);
            if !rows.is_empty() || !cols.is_empty() {
                split.push(format!("{w}x{h} at {requested}: {rows:?} {cols:?}"));
            }
        }
    }
    assert!(
        split.is_empty(),
        "a band holding a strong line is no longer the strip these tests build: {split:?}"
    );
}

#[test]
fn the_textured_art_backdrop_is_neither_flat_enough_nor_uniform_enough_to_be_chrome() {
    let t = Tuning::default();
    let backdrop = art(W, 94);
    let rect = whole(&backdrop);
    let measured_stddev = stddev(&backdrop, rect);
    let measured_flat = flat_fraction(&backdrop, rect, t.uniform_tolerance);
    assert!(
        measured_stddev >= f64::from(t.min_content_stddev),
        "AC-1 needs art with stddev >= min_content_stddev; measured {measured_stddev}"
    );
    assert!(
        measured_flat <= 0.5,
        "AC-1 needs art with flat fraction <= 0.5; measured {measured_flat}"
    );
    assert!(
        strong_lines(&row_profile(&backdrop, rect), &t).is_empty()
            && strong_lines(&col_profile(&backdrop, rect), &t).is_empty(),
        "the art backdrop must contribute no strong line of its own"
    );
}

#[test]
fn the_band_and_the_art_are_separated_by_exactly_one_strong_line_at_the_seam() {
    let t = Tuning::default();
    let scene = top_band_scene(6, 0.90);
    let difference = (row_mean(&scene, 5) - row_mean(&scene, 6)).abs();
    assert!(
        difference >= f64::from(t.edge_threshold),
        "AC-1 needs the band's mean luma to differ from the art's first row by at \
         least edge_threshold; measured {difference}"
    );
    let lines = strong_lines(&row_profile(&scene, whole(&scene)), &t);
    assert_eq!(
        lines.len(),
        1,
        "the seam must be the only strong row line, so the strip's outer bound is \
         unambiguous; got {lines:?}"
    );
    assert_eq!(
        (lines[0].start, lines[0].end),
        (5, 5),
        "the run must be the single profile index between the band's last row and \
         the art's first"
    );
}

#[test]
fn the_nearly_flat_art_of_the_last_criterion_has_too_little_content_to_keep() {
    let t = Tuning::default();
    let nearly_flat = flat_band(W, 94, FLAT_ART_LOW, FLAT_ART_SPREAD);
    let measured = stddev(&nearly_flat, whole(&nearly_flat));
    assert!(
        measured < f64::from(t.min_content_stddev),
        "AC-7 needs art below min_content_stddev; measured {measured}"
    );
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn a_flat_top_band_is_peeled_and_the_top_edge_lands_on_the_bands_last_row_plus_one() {
    let scene = top_band_scene(6, 0.90);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        below_band(6),
        "the content box must be exactly the art rows"
    );
    assert_eq!(found.removed, vec![Side::Top]);
    assert!(!found.ambiguous);
}

// --- AC-2 -------------------------------------------------------------------

#[test]
fn a_top_band_and_a_left_sidebar_are_both_peeled_in_the_order_they_were_visited() {
    let scene = top_band_and_sidebar(0.90);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 15,
            y: 6,
            w: 85,
            h: 94
        },
        "the content box must be exactly the art rect"
    );
    assert_eq!(found.removed, vec![Side::Top, Side::Left]);
    assert!(!found.ambiguous);
}

#[test]
fn a_pass_visits_top_then_bottom_then_left_then_right() {
    let scene = stack(&[
        band(W, 6, BAND_BG, 0.90),
        beside(&[
            band(12, 88, SIDEBAR_BG, 0.90),
            art(76, 88),
            band(12, 88, 220, 0.90),
        ]),
        band(W, 6, BAND_BG, 0.90),
    ]);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 12,
            y: 6,
            w: 76,
            h: 88
        }
    );
    assert_eq!(
        found.removed,
        vec![Side::Top, Side::Bottom, Side::Left, Side::Right],
        "all four strips qualify, so `removed` is the visiting order itself"
    );
}

#[test]
fn a_flat_bottom_band_is_peeled_from_the_bottom_edge() {
    let scene = stack(&[art(W, 94), band(W, 6, BAND_BG, 0.90)]);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 0,
            y: 0,
            w: W,
            h: 94
        },
        "the bottom edge must land on the band's first row, taking the run's outer \
         bound on the side nearest the bottom"
    );
    assert_eq!(found.removed, vec![Side::Bottom]);
}

#[test]
fn a_flat_right_sidebar_is_peeled_from_the_right_edge() {
    let scene = beside(&[art(85, H), band(15, H, SIDEBAR_BG, 0.90)]);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 0,
            y: 0,
            w: 85,
            h: H
        }
    );
    assert_eq!(found.removed, vec![Side::Right]);
}

#[test]
fn a_horizontal_seam_two_indices_wide_is_cut_at_the_runs_outer_bound() {
    // A textured line between each band and the art makes each seam a run of
    // two strong profile indices. The cut takes the run's outer bound on the
    // side nearest the image edge - `start` at the top, `end` at the bottom -
    // so the seam lines themselves stay with the content.
    let scene = stack(&[
        band(W, 6, BAND_BG, 0.90),
        textured_seam(W, 1),
        art(W, 86),
        textured_seam(W, 1),
        band(W, 6, BAND_BG, 0.90),
    ]);
    let t = Tuning::default();
    assert_eq!(
        strong_lines(&row_profile(&scene, whole(&scene)), &t)
            .iter()
            .map(|line| (line.start, line.end))
            .collect::<Vec<_>>(),
        vec![(5, 6), (92, 93)],
        "each seam must be a run of two indices for this test to mean anything"
    );
    let found = content_box(&scene, whole(&scene), &t);
    assert_eq!(
        found.rect,
        Rect {
            x: 0,
            y: 6,
            w: W,
            h: 88
        },
        "the top strip ends at the run's first index and the bottom strip starts \
         after its last, so both seam lines are kept"
    );
    assert_eq!(found.removed, vec![Side::Top, Side::Bottom]);
}

#[test]
fn a_vertical_seam_two_indices_wide_is_cut_at_the_runs_outer_bound() {
    let scene = beside(&[
        band(6, H, BAND_BG, 0.90),
        textured_seam(1, H),
        art(86, H),
        textured_seam(1, H),
        band(6, H, BAND_BG, 0.90),
    ]);
    let t = Tuning::default();
    assert_eq!(
        strong_lines(&col_profile(&scene, whole(&scene)), &t)
            .iter()
            .map(|line| (line.start, line.end))
            .collect::<Vec<_>>(),
        vec![(5, 6), (92, 93)]
    );
    let found = content_box(&scene, whole(&scene), &t);
    assert_eq!(
        found.rect,
        Rect {
            x: 6,
            y: 0,
            w: 88,
            h: H
        }
    );
    assert_eq!(found.removed, vec![Side::Left, Side::Right]);
}

// --- AC-3: the flat fraction threshold and its controls ---------------------

#[test]
fn a_top_strip_that_is_only_half_flat_is_art_and_is_left_where_it_is() {
    let scene = top_band_scene(6, 0.50);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        whole(&scene),
        "a strip at flat fraction 0.50 is content, not chrome"
    );
    assert_eq!(found.removed, Vec::new());
    assert!(!found.ambiguous, "0.50 is nowhere near the ambiguity band");
}

#[test]
fn a_top_strip_flatter_than_the_threshold_is_peeled() {
    let scene = top_band_scene(6, 0.86);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, below_band(6));
    assert_eq!(found.removed, vec![Side::Top]);
    assert!(!found.ambiguous);
}

#[test]
fn a_top_strip_just_under_the_threshold_is_kept_and_marks_the_result_ambiguous() {
    let scene = top_band_scene(6, 0.84);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, whole(&scene), "0.84 is below the threshold");
    assert_eq!(found.removed, Vec::new());
    assert!(
        found.ambiguous,
        "0.84 is inside [chrome_flat_fraction - ambiguity_band, chrome_flat_fraction)"
    );
}

#[test]
fn a_top_strip_below_the_ambiguity_band_is_kept_without_marking_the_result_ambiguous() {
    let scene = top_band_scene(6, 0.79);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, whole(&scene));
    assert_eq!(found.removed, Vec::new());
    assert!(
        !found.ambiguous,
        "0.79 is below chrome_flat_fraction - ambiguity_band, so it is plain content"
    );
}

#[test]
fn a_top_strip_exactly_at_the_flat_fraction_threshold_is_peeled() {
    let scene = top_band_scene(6, 0.85);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        below_band(6),
        "\"at least chrome_flat_fraction\" includes the threshold itself"
    );
    assert_eq!(found.removed, vec![Side::Top]);
    assert!(!found.ambiguous);
}

#[test]
fn a_top_strip_exactly_at_the_foot_of_the_ambiguity_band_is_ambiguous() {
    let scene = top_band_scene(6, 0.80);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, whole(&scene));
    assert!(
        found.ambiguous,
        "the band is closed at chrome_flat_fraction - ambiguity_band"
    );
}

// --- AC-4: the extent threshold and its controls ----------------------------

#[test]
fn a_top_strip_taller_than_the_chrome_max_extent_is_left_where_it_is() {
    let scene = top_band_scene(31, 0.95);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        whole(&scene),
        "31 rows of 100 is past chrome_max_extent, however flat the strip is"
    );
    assert_eq!(found.removed, Vec::new());
    assert!(
        !found.ambiguous,
        "a strip too tall to be chrome is not nearly chrome-like either"
    );
}

#[test]
fn a_top_strip_inside_the_chrome_max_extent_is_peeled() {
    let scene = top_band_scene(29, 0.95);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, below_band(29));
    assert_eq!(found.removed, vec![Side::Top]);
}

#[test]
fn a_top_strip_of_exactly_the_chrome_max_extent_is_peeled() {
    let scene = top_band_scene(30, 0.95);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        below_band(30),
        "\"at most chrome_max_extent\" includes the extent itself"
    );
    assert_eq!(found.removed, vec![Side::Top]);
}

#[test]
fn the_chrome_max_extent_is_a_share_of_the_image_not_of_the_rect() {
    let scene = top_band_scene(10, 0.90);
    let within = Rect {
        x: 0,
        y: 0,
        w: W,
        h: 30,
    };
    let found = content_box(&scene, within, &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 0,
            y: 10,
            w: W,
            h: 20
        },
        "10 rows is 33% of this rect but 10% of the image, and the image is what \
         chrome_max_extent is a share of"
    );
    assert_eq!(found.removed, vec![Side::Top]);
}

// --- AC-5 -------------------------------------------------------------------

#[test]
fn two_stacked_chrome_bands_are_peeled_one_per_pass() {
    let scene = stack(&[band(W, 8, BAND_BG, 0.90), band(W, 5, 150, 0.90), art(W, 87)]);
    let t = Tuning::default();
    assert_eq!(
        strong_lines(&row_profile(&scene, whole(&scene)), &t).len(),
        2,
        "each band must be bounded by its own strong line"
    );
    let found = content_box(&scene, whole(&scene), &t);
    assert_eq!(
        found.rect,
        Rect {
            x: 0,
            y: 13,
            w: W,
            h: 87
        },
        "the box must start below the second band"
    );
    assert_eq!(
        found.removed,
        vec![Side::Top, Side::Top],
        "one strip per side per pass, so two bands take two passes"
    );
    assert!(!found.ambiguous);
}

#[test]
fn each_pass_takes_at_most_one_strip_from_each_side() {
    // Two stacked bands at the top and a sidebar down the left. Visiting one
    // strip per side per pass peels A, then the sidebar, then B; draining a
    // side greedily would peel A and B before ever reaching the sidebar, and
    // `removed` is where the difference shows.
    let scene = stack(&[
        band(W, 8, BAND_BG, 0.90),
        band(W, 5, 150, 0.90),
        beside(&[band(15, 87, SIDEBAR_BG, 0.95), art(85, 87)]),
    ]);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 15,
            y: 13,
            w: 85,
            h: 87
        }
    );
    assert_eq!(
        found.removed,
        vec![Side::Top, Side::Left, Side::Top],
        "the second top band waits for the next pass, so the sidebar is peeled \
         between the two"
    );
}

// --- AC-6 -------------------------------------------------------------------

#[test]
fn a_rect_with_no_strong_lines_keeps_every_pixel() {
    let scene = art(W, H);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, whole(&scene));
    assert_eq!(found.removed, Vec::new());
    assert!(!found.ambiguous);
}

#[test]
fn only_strong_lines_inside_the_rect_are_looked_at() {
    let scene = top_band_scene(6, 0.90);
    let within = below_band(6);
    let found = content_box(&scene, within, &Tuning::default());
    assert_eq!(
        found.rect, within,
        "the seam above the rect is outside it, so there is no strip to peel"
    );
    assert_eq!(found.removed, Vec::new());
}

#[test]
fn the_search_starts_from_the_rect_it_was_given() {
    let scene = top_band_and_sidebar(0.90);
    let within = below_band(6);
    let found = content_box(&scene, within, &Tuning::default());
    assert_eq!(
        found.rect,
        Rect {
            x: 15,
            y: 6,
            w: 85,
            h: 94
        }
    );
    assert_eq!(
        found.removed,
        vec![Side::Left],
        "the band above `within` was never in play, so only the sidebar is peeled"
    );
}

// --- AC-7 -------------------------------------------------------------------

#[test]
fn chrome_is_never_peeled_off_an_image_with_no_content_left_to_keep() {
    let scene = stack(&[
        band(W, 6, BAND_BG, 0.90),
        flat_band(W, 94, FLAT_ART_LOW, FLAT_ART_SPREAD),
    ]);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        whole(&scene),
        "removing the strip would leave a region below min_content_stddev"
    );
    assert_eq!(found.removed, Vec::new());
    assert!(
        !found.ambiguous,
        "a strip whose removal leaves nothing is not nearly chrome-like"
    );
}

// --- The qualified reading of ambiguity (MC-005 `## Notes`) -----------------

#[test]
fn a_strip_too_tall_to_be_chrome_does_not_mark_the_result_ambiguous() {
    let scene = top_band_scene(31, 0.82);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, whole(&scene));
    assert!(
        !found.ambiguous,
        "flat fraction 0.82 is inside the band, but a strip covering 31% of the \
         image is not a chrome candidate, so it is ordinary art"
    );
}

#[test]
fn a_strip_whose_removal_would_leave_no_content_does_not_mark_the_result_ambiguous() {
    let scene = stack(&[
        band(W, 6, BAND_BG, 0.82),
        flat_band(W, 94, FLAT_ART_LOW, FLAT_ART_SPREAD),
    ]);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(found.rect, whole(&scene));
    assert!(
        !found.ambiguous,
        "flat fraction 0.82 is inside the band, but there is no content to keep, \
         so the strip is not nearly chrome-like"
    );
}

#[test]
fn a_peeled_side_and_an_ambiguous_side_both_show_in_one_result() {
    let scene = top_band_and_sidebar(0.84);
    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.rect,
        below_band(6),
        "the band goes, the sidebar stays"
    );
    assert_eq!(found.removed, vec![Side::Top]);
    assert!(
        found.ambiguous,
        "the sidebar missed the flat fraction threshold by less than ambiguity_band"
    );
}

// --- Out of scope, pinned because it is cheap -------------------------------

#[test]
fn an_interior_flat_panel_that_touches_no_edge_is_never_peeled() {
    let scene = stack(&[art(W, 20), band(W, 6, BAND_BG, 0.90), art(W, 74)]);
    let t = Tuning::default();
    assert_eq!(
        strong_lines(&row_profile(&scene, whole(&scene)), &t).len(),
        2,
        "the interior band must be bounded by two strong lines"
    );
    let found = content_box(&scene, whole(&scene), &t);
    assert_eq!(
        found.rect,
        whole(&scene),
        "the flat band is bounded by strong lines but touches no edge, so it is a \
         panel border and stays"
    );
    assert_eq!(found.removed, Vec::new());
    assert!(!found.ambiguous);
}

// --- Every threshold is read from the Tuning, never written down ------------

#[test]
fn the_flat_fraction_threshold_is_read_from_the_tuning() {
    let scene = top_band_scene(6, 0.84);
    let default = content_box(&scene, whole(&scene), &Tuning::default());
    let lowered = content_box(
        &scene,
        whole(&scene),
        &Tuning {
            chrome_flat_fraction: 0.84,
            ..Default::default()
        },
    );
    assert_eq!(default.removed, Vec::new());
    assert_eq!(
        lowered.removed,
        vec![Side::Top],
        "the same strip becomes chrome when the threshold drops to meet it"
    );
    assert_eq!(lowered.rect, below_band(6));
}

#[test]
fn the_chrome_max_extent_is_read_from_the_tuning() {
    let scene = top_band_scene(31, 0.95);
    let default = content_box(&scene, whole(&scene), &Tuning::default());
    let widened = content_box(
        &scene,
        whole(&scene),
        &Tuning {
            chrome_max_extent: 0.35,
            ..Default::default()
        },
    );
    assert_eq!(default.removed, Vec::new());
    assert_eq!(widened.removed, vec![Side::Top]);
    assert_eq!(widened.rect, below_band(31));
}

#[test]
fn the_min_content_stddev_is_read_from_the_tuning() {
    let scene = stack(&[
        band(W, 6, BAND_BG, 0.90),
        flat_band(W, 94, FLAT_ART_LOW, FLAT_ART_SPREAD),
    ]);
    let default = content_box(&scene, whole(&scene), &Tuning::default());
    let lowered = content_box(
        &scene,
        whole(&scene),
        &Tuning {
            min_content_stddev: 4.0,
            ..Default::default()
        },
    );
    assert_eq!(default.removed, Vec::new());
    assert_eq!(
        lowered.removed,
        vec![Side::Top],
        "the nearly flat art is content once the floor drops below its stddev of 5"
    );
    assert_eq!(lowered.rect, below_band(6));
}

#[test]
fn the_ambiguity_band_is_read_from_the_tuning() {
    let scene = top_band_scene(6, 0.79);
    let default = content_box(&scene, whole(&scene), &Tuning::default());
    let widened = content_box(
        &scene,
        whole(&scene),
        &Tuning {
            ambiguity_band: 0.10,
            ..Default::default()
        },
    );
    assert!(!default.ambiguous);
    assert!(
        widened.ambiguous,
        "0.79 falls inside a band of 0.10 below the threshold"
    );
    assert_eq!(
        widened.removed,
        Vec::new(),
        "widening the band never removes anything"
    );
}

#[test]
fn the_edge_threshold_is_read_from_the_tuning() {
    let scene = top_band_scene(6, 0.90);
    let raised = content_box(
        &scene,
        whole(&scene),
        &Tuning {
            edge_threshold: 200,
            ..Default::default()
        },
    );
    assert_eq!(
        raised.rect,
        whole(&scene),
        "with no strong line at the seam there is no strip to peel"
    );
    assert_eq!(raised.removed, Vec::new());
}

#[test]
fn the_flat_fraction_is_measured_about_the_strips_median_not_its_mean() {
    let scene = stack(&[dark_dashed_band(W, 6), art(W, 94)]);
    let strip = Rect {
        x: 0,
        y: 0,
        w: W,
        h: 6,
    };
    let tolerance = Tuning::default().uniform_tolerance;
    let mut values = pixels(&scene, strip);
    values.sort_unstable();
    let median = values[values.len() / 2];
    let mean = (values.iter().map(|&px| u32::from(px)).sum::<u32>() / values.len() as u32) as u8;
    assert!(
        median.abs_diff(mean) > tolerance,
        "the fixture only discriminates if the strip's median and mean are further \
         apart than uniform_tolerance; measured {median} and {mean}"
    );
    let about = |centre: u8| {
        values
            .iter()
            .filter(|&&px| px.abs_diff(centre) <= tolerance)
            .count() as f64
            / values.len() as f64
    };
    let t = Tuning::default();
    assert!(
        about(median) >= f64::from(t.chrome_flat_fraction)
            && about(mean) < f64::from(t.chrome_flat_fraction - t.ambiguity_band),
        "about the median this strip is chrome and about the mean it is not even \
         ambiguous: {} and {}",
        about(median),
        about(mean)
    );

    let found = content_box(&scene, whole(&scene), &Tuning::default());
    assert_eq!(
        found.removed,
        vec![Side::Top],
        "the flat fraction is a share of pixels near the median, so the strip is chrome"
    );
    assert_eq!(found.rect, below_band(6));
}

#[test]
fn a_pixel_exactly_uniform_tolerance_from_the_median_counts_as_flat() {
    let tolerance = Tuning::default().uniform_tolerance;
    // Half this band's pixels sit `tolerance` away from its median and half
    // sit on it, so its flat fraction is 1.0 if "within uniform_tolerance" is
    // inclusive and 0.50 if it is not.
    let inclusive = stack(&[
        chrome_band(W, 6, BAND_BG, tolerance, tolerance, 0.50),
        art(W, 94),
    ]);
    let found = content_box(&inclusive, whole(&inclusive), &Tuning::default());
    assert_eq!(
        found.removed,
        vec![Side::Top],
        "every pixel is within uniform_tolerance of the median, so the strip is \
         wholly flat and is chrome"
    );
    assert_eq!(found.rect, below_band(6));

    // The control, one level further out: now half the pixels are not flat.
    let exclusive = stack(&[
        chrome_band(W, 6, BAND_BG, tolerance + 1, tolerance, 0.50),
        art(W, 94),
    ]);
    let control = content_box(&exclusive, whole(&exclusive), &Tuning::default());
    assert_eq!(
        control.rect,
        whole(&exclusive),
        "one level further out and the same band is only half flat, so it stays"
    );
    assert_eq!(control.removed, Vec::new());
}
