//! MC-025, AC-1, AC-2, AC-4 and AC-5: panel edges found by flatness where the
//! gradient goes blind.
//!
//! `edges` locates a boundary by asking *how much did this line change from
//! the last one* and calling the answer a strong line when it reaches
//! `Tuning::edge_threshold`. MC-019 measured seven real edges where that
//! question has no answer: a dark panel bleeding into a dark gutter over tens
//! of pixels, peak adjacent-line step 0.59 to 7.2 against a threshold of 24
//! (the table is in the story's `## Context`, and colour was measured and
//! ruled out there - do not revisit it). This story adds the other question -
//! **is this line flat?** - and nothing else. It retunes no constant.
//!
//! Two functions and one enum, all additive:
//!
//! * `spread_profile(&Luma, Axis) -> Vec<f32>`, the mean absolute deviation of
//!   each line about its own mean, **one entry per line**;
//! * `textured_span(&[f32], &Tuning) -> Option<(usize, usize)>`, the first and
//!   last index of that profile at or above `Tuning::min_line_spread`, both
//!   inclusive. That pair is "the flatness locator" everywhere below.
//!
//! Nothing here says where in the pipeline either is called. That is
//! deliberate and it is load-bearing: a flatness fallback wired naively into
//! `content::strip_depth` would locate the 90%-flat band in
//! `tests/content.rs::the_edge_threshold_is_read_from_the_tuning` and peel it,
//! breaking a frozen assertion and with it AC-4. Finding a wiring that
//! satisfies AC-3 *and* AC-4 is GREEN's problem, and these tests leave it the
//! room to solve it. AC-3 is the one criterion that goes through `decide` and
//! it lives in `tests/decide.rs`, which still compiles today and so can
//! actually be watched to fail.
//!
//! # A per-line statistic has one entry per line
//!
//! AC-1 asks for "the same orientation and length convention as the existing
//! `edges` profiles". That means the same `Vec<f32>`, the same axis
//! orientation and the same top-to-bottom / left-to-right index order, over
//! the same lines. It does **not** mean `row_profile`'s `h - 1`: that count is
//! a property of *adjacent pairs* and there is no such thing as the spread of
//! a pair. A plane of `h` rows has `h` row spreads and `w` column spreads, and
//! `the_spread_profile_has_one_entry_per_line_not_one_per_adjacent_pair` pins
//! both counts beside `row_profile`'s, so the contrast is stated rather than
//! left to be discovered.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled, read out, never calibrated here**: `edge_threshold = 24`,
//!   `uniform_tolerance = 10`, `chrome_flat_fraction = 0.85`,
//!   `margin_px = 3`. Every one of them is read from the `Tuning` under test
//!   at its use site. Retuning is MC-019's job.
//! * **Mechanical**: AC-1's two counts and every value in them, AC-4's
//!   gradient report, AC-5's agreement. Pinned exactly.
//! * **Oracle-free, and the one number this story chooses**:
//!   `Tuning::min_line_spread`, defaulting to **8.0**. Its derivation is
//!   immediately below, and it is the only calibrated number in this file.
//!
//! # Deriving `min_line_spread`
//!
//! Three constraints, two of them hard and taken from constants this story
//! does not own:
//!
//! 1. **Floor 5.0, from `uniform_tolerance`.** A line is uniform by MC-003's
//!    settled rule iff `max - min <= uniform_tolerance`, and the largest mean
//!    absolute deviation such a line can have is `uniform_tolerance / 2` -
//!    half its pixels at each end of the band. A threshold at or below that
//!    would let the flatness locator call "art" a line stage 1 calls uniform
//!    and trims. `a_line_that_is_uniform_by_the_settled_rule_never_reads_as_art`
//!    builds that extreme line, measures 5.0, and pins the inequality.
//! 2. **Ceiling 8.26, from `common::soft_art`.** That is the art in every
//!    scene `tests/content.rs` and `tests/edges.rs` build, and AC-4 says this
//!    story must not change what they report. Measured over `soft_art(100,
//!    94, 7)`: row spreads 11.73 to 13.46, **column spreads 8.26 to 10.32**.
//!    A threshold above 8.26 reads that art as flat on the column axis.
//!    `the_art_the_existing_fixtures_use_stays_textured_on_both_axes` pins
//!    that headroom, which is 0.26 and thinner than anything else in this
//!    file - so if a later story wants a larger threshold, that test is where
//!    it will find out, loudly, instead of in a corpus score.
//! 3. **8, from the corpus.** The story's `## Context` records that
//!    thresholding per-row spread at 8 locates the art boundary at 0 px offset
//!    for 15 of 19 marked edges and within 5 px for 17 of 19. It is the one
//!    value inside `(5.0, 8.26)` with a measurement behind it.
//!
//! So **8.0**. Two negative controls say what it excludes, because a threshold
//! with no negative control means nothing:
//!
//! * the **flat gutter** this story is about (`common::flat_gutter_only`) -
//!   row spread 0.8, column spread 0.67 to 0.99, and *not* uniform, so it
//!   reaches the locator rather than being trimmed away first;
//! * the **chrome band at the settled flat fraction** - `chrome_band(100, 6,
//!   200, 20, 10, 0.90)`, which is exactly the band
//!   `tests/content.rs::top_band_scene(6, 0.90)` stacks. Row spread exactly
//!   2.0, column spread exactly 0.0.
//!
//! Neither may fire, and both are four to ten times below the threshold.
//! Note the other direction for whoever tunes this next: a high-contrast
//! toolbar - black text at 15% coverage on white - has a row spread around 15
//! and reads as *art* to this locator. That is the safe way round (it declines
//! to cut rather than cutting into a page), and it is why this locator is an
//! addition to the chrome peel and not a replacement for it.
//!
//! # Comparing `f32`
//!
//! The two disciplines `tests/edges.rs` established, chosen per assertion and
//! never widened to paper over the other:
//!
//! * **Exact equality** wherever the true mean absolute deviation is an
//!   integer. Every fixture here that is compared exactly is built so that
//!   both the line's mean and its mean absolute deviation are integers reached
//!   as an exact integer sum over an exact count, far below `2^24`, so the
//!   quotient is the exactly representable integer under any summation order.
//!   The plausible wrong answers - a deviation about the *median* (10.0 where
//!   the mean gives 15.0), a standard deviation (17.32), a sum instead of a
//!   mean - are all at least 1.0 away, so exactness costs nothing and rules
//!   them out.
//! * **An explicit `EPS` of 1e-5**, in the two places the true value is not
//!   representable: `[0, 0, 1]`, whose spread is 4/9, and the fade fixture's
//!   gutter rows at 0.8. At 4/9 the `f32` arithmetic lands 3e-8 from the true
//!   value while the nearest wrong answer (deviation about the median, 1/3) is
//!   0.111 away - four orders of magnitude outside the tolerance.

mod common;

use common::{
    CHROME_DEVIATION, FADE_GUTTER, FADE_H, FADE_PEAK, FADE_W, HARD_GUTTER, HARD_H, HARD_W, PAGE_H,
    PAGE_SIXTH, PAGE_W, WAIST_BOTTOM, WAIST_INSET, WAIST_TOP, chrome_band, fade_amplitude,
    fade_first_textured_line, fade_gutter_spread, fade_last_textured_line, fade_row_spread,
    fade_to_gutter, flat_gutter_only, hard_edge_art_rows, hard_edge_to_gutter,
    page_first_textured_column, page_in_margins, page_in_margins_with_blank_bands,
    page_in_margins_with_chrome_bands, page_last_textured_column, page_with_a_waist, soft_art,
    waist_inner_columns, waist_page_columns,
};
use cropper_core::edges::{
    Axis, Line, row_profile, spread_profile, strong_lines, textured_span, widest_textured_run,
};
use cropper_core::flat::{central_band, page_column};
use cropper_core::{Luma, Rect, Tuning};

// --- Comparison discipline --------------------------------------------------

/// See the module header. Used in exactly the two places the true value is not
/// an integer, and never as a substitute for exact equality.
const EPS: f32 = 1e-5;

fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() <= EPS,
        "{what}: expected {expected} +/- {EPS}, got {actual}"
    );
}

/// AC-2's tolerance, whole: "within 4 px of the first line whose spread
/// exceeds the flatness threshold". AC-5, where both locators can see the
/// edge, is exact and does not use this.
const AC2_TOLERANCE_PX: u32 = 4;

// --- Fixture helpers --------------------------------------------------------

/// The rect covering the whole of `img`.
fn whole(img: &Luma) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    }
}

/// A plane built from literal rows, so the expected spread of every line can
/// be worked out by hand and written down.
fn plane(width: u32, height: u32, data: Vec<u8>) -> Luma {
    assert_eq!(
        data.len(),
        (width * height) as usize,
        "a plane holds exactly width * height samples"
    );
    Luma {
        width,
        height,
        data,
    }
}

/// AC-1's hand-built plane. Every row spread and every column spread is an
/// exact integer, and the two multisets are disjoint apart from nothing, so a
/// transposed implementation cannot pass:
///
/// ```text
///          col 0  col 1  col 2  col 3     row mean   row spread
/// row 0       10     20     30     40           25           10
/// row 1        0      0      0     40           10           15
/// row 2        8     16     24     40           22           10
/// col mean     6     12     18     40
/// col spread   4      8     12      0
/// ```
///
/// Row 1 is the discriminator that matters: its deviation about the row's own
/// **mean** is 15, about its median 10, and its standard deviation 17.32.
fn hand_plane() -> Luma {
    plane(4, 3, vec![10, 20, 30, 40, 0, 0, 0, 40, 8, 16, 24, 40])
}

/// The largest value in a profile.
fn peak(profile: &[f32]) -> f32 {
    profile.iter().copied().fold(f32::MIN, f32::max)
}

/// The smallest value in a profile.
fn trough(profile: &[f32]) -> f32 {
    profile.iter().copied().fold(f32::MAX, f32::min)
}

/// Every column spread of `img` **inside `rect`**, through the public API.
///
/// `spread_profile` measures a whole plane, which is MC-025 AC-1 verbatim, and
/// the rect-taking form of it is crate-private. Cropping the plane and asking
/// for the whole of the crop is the same question, and it is a question a test
/// can ask - so a test can compute the profile MC-027's locator works over
/// without depending on an internal.
fn column_spreads(img: &Luma, rect: Rect) -> Vec<f32> {
    let stride = img.width as usize;
    let (x, y) = (rect.x as usize, rect.y as usize);
    let (w, h) = (rect.w as usize, rect.h as usize);
    let mut data = vec![0u8; w * h];
    for dy in 0..h {
        let from = (y + dy) * stride + x;
        data[dy * w..(dy + 1) * w].copy_from_slice(&img.data[from..from + w]);
    }
    spread_profile(
        &Luma {
            width: rect.w,
            height: rect.h,
            data,
        },
        Axis::Columns,
    )
}

// --- The one number this story chooses --------------------------------------

#[test]
fn the_default_min_line_spread_is_eight() {
    assert_eq!(
        Tuning::default().min_line_spread,
        8.0,
        "the module header derives this; it is not a free parameter"
    );
}

/// Constraint 1 of the derivation, as behaviour rather than as arithmetic.
///
/// A line is uniform by MC-003's settled rule iff `max - min <=
/// uniform_tolerance`, and this is the most deviating line that still is:
/// half its pixels at each end of a band exactly `uniform_tolerance` wide.
/// Its spread is `uniform_tolerance / 2`. If `min_line_spread` ever slipped to
/// or below that, the flatness locator would call art a line stage 1 trims as
/// a uniform border, and the two stages would contradict each other.
#[test]
fn a_line_that_is_uniform_by_the_settled_rule_never_reads_as_art() {
    let t = Tuning::default();
    let low = 100u8;
    let high = low + t.uniform_tolerance;
    let data = (0..200u32)
        .map(|i| if i.is_multiple_of(2) { low } else { high })
        .collect();
    let line = plane(200, 1, data);

    let rows = spread_profile(&line, Axis::Rows);
    assert_eq!(
        rows,
        vec![f32::from(t.uniform_tolerance) / 2.0],
        "the most deviating uniform line has spread uniform_tolerance / 2"
    );
    assert!(
        t.min_line_spread > rows[0],
        "min_line_spread ({}) must sit strictly above the largest spread a line the \
         settled uniform rule accepts ({}), or the flatness locator contradicts \
         trim_uniform",
        t.min_line_spread,
        rows[0]
    );
    assert_eq!(
        textured_span(&rows, &t),
        None,
        "and so the locator must not fire on it"
    );
}

/// The first negative control: the flat gutter this whole story is about.
///
/// It is flat in the mean-absolute-deviation sense the locator measures and
/// **not** uniform in the `max - min` sense stage 1 measures, which is what
/// makes it a control rather than a curiosity - a perfectly flat gutter would
/// be trimmed away before any locator saw it, and the corpus's gutters are
/// not perfectly flat. RED measured: row spread 0.8 throughout, column spread
/// 0.6666667 to 0.9916715, every row's and every column's `max - min` = 40.
#[test]
fn a_flat_gutter_never_reads_as_art_on_either_axis() {
    let t = Tuning::default();
    let img = flat_gutter_only(FADE_W, 120);

    let rows = spread_profile(&img, Axis::Rows);
    let cols = spread_profile(&img, Axis::Columns);
    assert!(
        peak(&rows) < t.min_line_spread,
        "no row of a flat gutter may reach min_line_spread {}; the largest measures {}",
        t.min_line_spread,
        peak(&rows)
    );
    assert!(
        peak(&cols) < t.min_line_spread,
        "no column of a flat gutter may reach min_line_spread {}; the largest measures {}",
        t.min_line_spread,
        peak(&cols)
    );
    assert_eq!(
        textured_span(&rows, &t),
        None,
        "a flatness locator that fires on a flat gutter is firing on noise, and every \
         threshold in this file would then mean nothing"
    );
    assert_eq!(textured_span(&cols, &t), None, "the same on the other axis");

    // The control only controls anything if the gutter reaches the locator at
    // all, so pin that stage 1 will not take it first.
    let widest_uniform_row = (0..img.height)
        .map(|y| {
            let row = &img.data[(y * img.width) as usize..((y + 1) * img.width) as usize];
            row.iter().max().unwrap() - row.iter().min().unwrap()
        })
        .min()
        .unwrap();
    assert!(
        widest_uniform_row > t.uniform_tolerance,
        "every gutter row must span more than uniform_tolerance {} or trim_uniform \
         removes this fixture before the flatness locator is asked; the flattest row \
         spans {widest_uniform_row}",
        t.uniform_tolerance
    );
}

/// The second negative control, and the one that ties this story's threshold
/// to a settled constant it must not disturb.
///
/// This is exactly the band `tests/content.rs::top_band_scene(6, 0.90)` stacks
/// on top of its art - the band that
/// `the_edge_threshold_is_read_from_the_tuning` requires `content_box` to
/// leave alone. It is *chrome*, and the flatness locator must agree: every row
/// of it measures exactly 2.0 and every column exactly 0.0, a quarter of
/// `min_line_spread` and below.
#[test]
fn a_chrome_band_at_the_settled_flat_fraction_never_reads_as_art() {
    let t = Tuning::default();
    let band = chrome_band(100, 6, 200, CHROME_DEVIATION, t.uniform_tolerance, 0.90);

    let rows = spread_profile(&band, Axis::Rows);
    let cols = spread_profile(&band, Axis::Columns);
    assert_eq!(
        rows,
        vec![2.0; band.height as usize],
        "10 of every 100 pixels moved by {CHROME_DEVIATION}, symmetrically, is a row \
         spread of exactly 2.0"
    );
    assert_eq!(
        cols,
        vec![0.0; band.width as usize],
        "the band's pattern repeats row for row, so every column of it is constant"
    );
    assert_eq!(
        textured_span(&rows, &t),
        None,
        "a chrome band at chrome_flat_fraction must stay chrome to the flatness locator"
    );
    assert_eq!(textured_span(&cols, &t), None, "the same on the other axis");
}

// --- AC-1: the spread profile -----------------------------------------------

/// AC-1's length convention, stated beside the gradient profile's so the
/// contrast is explicit. A per-line statistic has one entry per line; `h - 1`
/// is a property of adjacent *pairs* and there is no spread of a pair.
#[test]
fn the_spread_profile_has_one_entry_per_line_not_one_per_adjacent_pair() {
    let img = hand_plane();
    assert_eq!(
        spread_profile(&img, Axis::Rows).len(),
        img.height as usize,
        "one spread per row: a {}x{} plane has {} row spreads",
        img.width,
        img.height,
        img.height
    );
    assert_eq!(
        spread_profile(&img, Axis::Columns).len(),
        img.width as usize,
        "one spread per column: a {}x{} plane has {} column spreads",
        img.width,
        img.height,
        img.width
    );
    assert_eq!(
        row_profile(&img, whole(&img)).len(),
        img.height as usize - 1,
        "the gradient profile is unchanged and still has one entry per adjacent pair"
    );
}

/// AC-1's value, exactly: the mean absolute deviation of each line about **its
/// own mean**, in top-to-bottom and left-to-right order.
///
/// The row multiset `{10, 15, 10}` and the column multiset `{4, 8, 12, 0}`
/// share no arrangement, so a transposed implementation fails here; row 1's
/// 15 is not what a deviation about the median (10) or a standard deviation
/// (17.32) would give; and no entry is what a *sum* would give.
#[test]
fn each_entry_is_the_mean_absolute_deviation_of_its_line_about_its_own_mean() {
    let img = hand_plane();
    assert_eq!(
        spread_profile(&img, Axis::Rows),
        vec![10.0, 15.0, 10.0],
        "rows, top to bottom"
    );
    assert_eq!(
        spread_profile(&img, Axis::Columns),
        vec![4.0, 8.0, 12.0, 0.0],
        "columns, left to right"
    );
}

/// The true mean here is 1/3 and the true spread 4/9, neither representable.
/// The wrong answers are far away: a deviation about the median is 1/3
/// (0.111 off), an integer truncation 0, a sum 4/3.
#[test]
fn a_line_whose_true_spread_is_not_representable_is_not_rounded_away() {
    let img = plane(3, 1, vec![0, 0, 1]);
    let rows = spread_profile(&img, Axis::Rows);
    assert_eq!(rows.len(), 1);
    assert_close(rows[0], 4.0 / 9.0, "the spread of [0, 0, 1]");
}

/// Empty, one, many: the first two.
#[test]
fn an_empty_plane_has_no_entries_and_a_single_pixel_has_one_of_zero() {
    let t = Tuning::default();
    let empty = Luma {
        width: 0,
        height: 0,
        data: Vec::new(),
    };
    assert!(
        spread_profile(&empty, Axis::Rows).is_empty(),
        "no rows, no row spreads"
    );
    assert!(
        spread_profile(&empty, Axis::Columns).is_empty(),
        "no columns, no column spreads"
    );
    assert_eq!(
        textured_span(&[], &t),
        None,
        "an empty profile has no textured span"
    );

    let one = plane(1, 1, vec![200]);
    assert_eq!(spread_profile(&one, Axis::Rows), vec![0.0]);
    assert_eq!(spread_profile(&one, Axis::Columns), vec![0.0]);
}

/// AC-1 over the fade fixture, row by row against the recipe rather than
/// against a re-derivation of the statistic.
///
/// The art rows are compared **exactly** - the recipe puts every pixel of an
/// art row exactly `a` away from a mean that is exactly `FADE_TONE` - and the
/// gutter rows within `EPS`, their true spread being 0.8.
#[test]
fn the_spread_profile_of_the_fade_fixture_is_what_the_recipe_drew() {
    let img = fade_to_gutter();
    let rows = spread_profile(&img, Axis::Rows);
    assert_eq!(rows.len(), FADE_H as usize);

    let off_recipe: Vec<(u32, f32, f32)> = (0..FADE_H)
        .map(|y| (y, rows[y as usize], fade_row_spread(y)))
        .filter(|&(_, actual, expected)| (actual - expected).abs() > EPS)
        .collect();
    assert!(
        off_recipe.is_empty(),
        "every row's spread must be the one the recipe drew; (row, measured, recipe) \
         for each that is not: {off_recipe:?}"
    );

    let inexact_art: Vec<(u32, f32)> = (FADE_GUTTER..FADE_H - FADE_GUTTER)
        .map(|y| (y, rows[y as usize]))
        .filter(|&(y, actual)| actual != fade_amplitude(y) as f32)
        .collect();
    assert!(
        inexact_art.is_empty(),
        "an art row's spread is an exact integer - its amplitude - and must compare \
         exactly; (row, measured) for each that does not: {inexact_art:?}"
    );
    assert_close(rows[0], fade_gutter_spread(), "the gutter's spread");
}

// --- AC-2: one fixture, both locators, opposite results ---------------------

/// AC-2, whole. The contrast *is* the story, so both halves are one test: a
/// fixture that stopped failing the gradient half would make the flatness half
/// meaningless, and a reader should not have to find that out in two places.
///
/// RED measured on this fixture, outside the test framework: peak
/// adjacent-row step **1.8** over the whole image (and the same 1.8 inside the
/// fades), zero steps at or above `edge_threshold`; row spread 0.8 through the
/// gutters, climbing by exactly one per row through the fade, 50.0 through the
/// core; first row at or above 8.0 is **47**, last is **212**.
#[test]
fn the_gradient_locator_is_blind_to_the_fade_the_flatness_locator_finds() {
    let t = Tuning::default();
    let img = fade_to_gutter();

    // Half one. There is no step here for any step-detector to find.
    let gradient = row_profile(&img, whole(&img));
    assert!(
        peak(&gradient) < f32::from(t.edge_threshold),
        "AC-2's precondition: no adjacent-row step in the fade may reach edge_threshold \
         {}; the largest anywhere in the fixture measures {}",
        t.edge_threshold,
        peak(&gradient)
    );
    assert_eq!(
        strong_lines(&gradient, &t),
        Vec::new(),
        "AC-2: the gradient locator must find no strong line at all in this fixture"
    );

    // Half two. The flatness locator finds the boundary the gradient cannot.
    let spread = spread_profile(&img, Axis::Rows);
    let (first, last) = textured_span(&spread, &t)
        .expect("AC-2: the flatness locator must find the art the gradient locator cannot see");
    let expected_first = fade_first_textured_line(t.min_line_spread)
        .expect("the recipe has a first row at or above the threshold");
    let expected_last = fade_last_textured_line(t.min_line_spread)
        .expect("the recipe has a last row at or above the threshold");
    assert!(
        (first as u32).abs_diff(expected_first) <= AC2_TOLERANCE_PX,
        "AC-2: the top boundary must land within {AC2_TOLERANCE_PX} px of row \
         {expected_first}, the first line the recipe draws at or above {}; it landed on \
         row {first}",
        t.min_line_spread
    );
    assert!(
        (last as u32).abs_diff(expected_last) <= AC2_TOLERANCE_PX,
        "AC-2: the bottom boundary must land within {AC2_TOLERANCE_PX} px of row \
         {expected_last}, the last line the recipe draws at or above {}; it landed on \
         row {last}",
        t.min_line_spread
    );
}

/// The threshold is read from the `Tuning` argument, not written at the
/// comparison site - the same treatment `edge_threshold` gets in
/// `tests/edges.rs`. Same fixture, threshold moved, answer moves with it, by
/// far more than AC-2's tolerance so the move cannot be noise.
///
/// RED measured: at 8.0 the recipe's span is rows 47..=212, at 30.0 it is
/// 69..=190.
#[test]
fn the_flatness_threshold_is_read_from_the_tuning() {
    let img = fade_to_gutter();
    let spread = spread_profile(&img, Axis::Rows);
    let span_at = |threshold: f32| {
        let t = Tuning {
            min_line_spread: threshold,
            ..Default::default()
        };
        textured_span(&spread, &t)
            .map(|(first, last)| (first as u32, last as u32))
            .unwrap_or_else(|| panic!("the fade fixture has a textured span at {threshold}"))
    };

    let settled = Tuning::default().min_line_spread;
    let raised = 30.0f32;
    let (low_first, low_last) = span_at(settled);
    let (high_first, high_last) = span_at(raised);

    for (threshold, first, last) in [
        (settled, low_first, low_last),
        (raised, high_first, high_last),
    ] {
        let expected_first = fade_first_textured_line(threshold).unwrap();
        let expected_last = fade_last_textured_line(threshold).unwrap();
        assert!(
            first.abs_diff(expected_first) <= AC2_TOLERANCE_PX
                && last.abs_diff(expected_last) <= AC2_TOLERANCE_PX,
            "at min_line_spread {threshold} the recipe's span is {expected_first}..={expected_last} \
             and the locator returned {first}..={last}, outside the {AC2_TOLERANCE_PX} px \
             AC-2 allows"
        );
    }
    assert!(
        high_first > low_first + AC2_TOLERANCE_PX && high_last + AC2_TOLERANCE_PX < low_last,
        "raising min_line_spread from {settled} to {raised} must pull the span further \
         into the art than AC-2's {AC2_TOLERANCE_PX} px tolerance, or the threshold is \
         not being read from the argument: {low_first}..={low_last} became \
         {high_first}..={high_last}"
    );
}

/// The inclusivity of the comparison, matching `strong_lines`' "at or above".
#[test]
fn a_line_exactly_at_the_threshold_is_textured_and_one_below_it_is_not() {
    let t = Tuning::default();
    assert_eq!(
        textured_span(&[t.min_line_spread], &t),
        Some((0, 0)),
        "at or above is >=, as it is for edge_threshold"
    );
    assert_eq!(
        textured_span(&[t.min_line_spread - 1.0], &t),
        None,
        "and strictly below is flat"
    );
}

/// The span reaches the **outermost** textured lines. A flat run *between* two
/// textured ones is a panel gutter, and MC-005's decision 13 is that a flat
/// band in the middle of a multi-panel page keeps its panels rather than being
/// cut at. A locator that stopped at the first flat line would cut there.
#[test]
fn the_span_reaches_the_outermost_textured_lines_across_a_flat_gap_between_them() {
    let t = Tuning::default();
    let profile = [0.0, 20.0, 0.0, 0.0, 30.0, 0.0];
    assert_eq!(
        textured_span(&profile, &t),
        Some((1, 4)),
        "a flat gutter between two panels must not end the span"
    );
}

// --- AC-5: where both locators can see, they agree --------------------------

/// AC-5. The same art as the fade fixture, at full amplitude, on a flat gutter
/// with no fade: an adjacent-row step of 98 at each seam, four times
/// `edge_threshold`. Both locators can see this edge and both must put the art
/// in the same place - exactly, not within AC-2's tolerance, because there is
/// nothing here for either of them to be uncertain about.
///
/// The index conversion is the one `content::strip_depth` uses: a gradient
/// profile index `i` sits between lines `i` and `i + 1`, so a run starting at
/// `i` puts the art's first line at `i + 1`, and a run ending at `i` puts the
/// art's last line at `i`. A spread profile index *is* a line index.
#[test]
fn on_a_hard_edge_both_locators_place_the_boundary_on_the_same_line() {
    let t = Tuning::default();
    let img = hard_edge_to_gutter();
    let (art_first, art_last) = hard_edge_art_rows();

    let spread = spread_profile(&img, Axis::Rows);
    assert_eq!(
        spread[art_first as usize], FADE_PEAK as f32,
        "the fixture's premise: an art row's spread is its amplitude"
    );
    assert_eq!(
        spread[0], 0.0,
        "the fixture's premise: this gutter is perfectly flat"
    );

    let gradient = strong_lines(&row_profile(&img, whole(&img)), &t);
    let gradient_first = gradient
        .first()
        .expect("AC-5's fixture must carry a strong line the gradient locator can find")
        .start as u32
        + 1;
    let gradient_last = gradient
        .last()
        .expect("AC-5's fixture must carry a strong line at each seam")
        .end as u32;
    assert_eq!(
        (gradient_first, gradient_last),
        (art_first, art_last),
        "the fixture's premise: the gradient locator places the art at rows \
         {art_first}..={art_last}"
    );

    let (flat_first, flat_last) = textured_span(&spread, &t)
        .expect("AC-5: the flatness locator must find an edge the gradient locator can");
    assert_eq!(
        (flat_first as u32, flat_last as u32),
        (gradient_first, gradient_last),
        "AC-5: where both locators can see the edge, they must place the boundary on \
         the same line"
    );
}

// --- AC-4: the gradient locator's report is unchanged -----------------------

/// AC-4's real oracle is the rest of `crates/core/tests`, which this story
/// leaves untouched and which must still pass. This pins the specific claim
/// the criterion makes - that adding a locator does not change what the
/// gradient one reports - on a fixture whose gradient profile is exactly
/// computable end to end.
#[test]
fn adding_the_flatness_locator_does_not_change_what_the_gradient_locator_reports() {
    let t = Tuning::default();
    let img = hard_edge_to_gutter();
    let profile = row_profile(&img, whole(&img));

    assert_eq!(
        profile.len(),
        (HARD_H - 1) as usize,
        "one entry per adjacent pair, unchanged"
    );
    let seams = [
        HARD_GUTTER as usize - 1,
        (HARD_H - HARD_GUTTER - 1) as usize,
    ];
    let off: Vec<(usize, f32)> = profile
        .iter()
        .enumerate()
        .map(|(i, &v)| (i, v))
        .filter(|&(i, v)| v != if seams.contains(&i) { 98.0 } else { 0.0 })
        .collect();
    assert!(
        off.is_empty(),
        "the gradient profile is 98.0 at each seam ({seams:?}) and 0.0 everywhere else, \
         because every art row of this fixture is identical; (index, measured) for each \
         that is not: {off:?}"
    );
    assert_eq!(
        strong_lines(&profile, &t),
        vec![
            Line {
                start: seams[0],
                end: seams[0]
            },
            Line {
                start: seams[1],
                end: seams[1]
            },
        ],
        "and strong_lines still merges by adjacency alone, one line per seam"
    );
    assert_eq!(
        img.width, HARD_W,
        "the mean is over the fixture's own width, so this is the one it was measured at"
    );
}

/// Constraint 2 of the derivation, and the tightest margin in this file.
///
/// `common::soft_art` is the art in every scene `tests/content.rs` and
/// `tests/edges.rs` build. If `min_line_spread` were raised above that art's
/// smallest **column** spread, the flatness locator would read it as gutter,
/// and any wiring of the locator into the pipeline would start cutting into
/// the art those suites pin - which is AC-4, broken. RED measured on
/// `soft_art(100, 94, 7)`: row spreads 11.731198 to 13.456203, column spreads
/// 8.261655 to 10.319375.
#[test]
fn the_art_the_existing_fixtures_use_stays_textured_on_both_axes() {
    let t = Tuning::default();
    let img = soft_art(100, 94, 7);
    let rows = spread_profile(&img, Axis::Rows);
    let cols = spread_profile(&img, Axis::Columns);

    assert!(
        trough(&rows) > t.min_line_spread,
        "every row of the existing art fixture must stay textured at min_line_spread \
         {}; the flattest measures {}",
        t.min_line_spread,
        trough(&rows)
    );
    assert!(
        trough(&cols) > t.min_line_spread,
        "every column of the existing art fixture must stay textured at min_line_spread \
         {}; the flattest measures {}, which is the ceiling on this threshold and the \
         thinnest margin in this file",
        t.min_line_spread,
        trough(&cols)
    );
    assert_eq!(
        textured_span(&rows, &t),
        Some((0, rows.len() - 1)),
        "so the locator finds the whole of it on the row axis"
    );
    assert_eq!(
        textured_span(&cols, &t),
        Some((0, cols.len() - 1)),
        "and the whole of it on the column axis"
    );
}

// --- The exported shapes ----------------------------------------------------

#[test]
fn axis_is_a_plain_comparable_copyable_value_type() {
    let rows = Axis::Rows;
    // Copy: `rows` is still usable after being moved into the comparison.
    let same = rows;
    assert_eq!(rows, same);
    #[allow(clippy::clone_on_copy)]
    let cloned = rows.clone();
    assert_eq!(cloned, Axis::Rows);
    assert_ne!(Axis::Rows, Axis::Columns, "two distinct variants");
    assert!(
        format!("{:?}", Axis::Columns).contains("Columns"),
        "Debug, so a failing assertion names the axis"
    );
}

// ============================================================================
// MC-027: the page column, located by its flat page margins
// ============================================================================
//
// MC-025 added the other question - *is this line flat?* - and one rule over
// the answer: the **outermost** textured pair, on whichever axis the gradient
// locator is blind. MC-026 measured what that rule reaches on the corpus and
// found the column half of the problem solved by a different rule and the row
// half not solved at all. This story ships the column half. The row axis stays
// exactly where MC-025 left it, and MC-028 owns it.
//
// Three additions, all of them additive, and `textured_span` is not one of
// them - it keeps returning the outermost pair, which is how MC-005's decision
// 13 is honoured and which AC-2 asserts in the same breath as the new rule:
//
// * `edges::widest_textured_run(&[f32], &Tuning) -> Option<(usize, usize)>`,
//   the **widest** run of consecutive textured indices, both bounds inclusive.
//   MC-026 finding 1 measured it at 19 of 21 against `textured_span`'s 15 on
//   the column axis, and identical to it on the row axis;
// * `Tuning::central_band_fraction`, defaulting to **0.6**, and
//   `flat::central_band(Rect, &Tuning) -> Rect`, the middle share of a rect's
//   rows;
// * `flat::page_column(&Luma, Rect, &Tuning) -> Rect`, which is the two put
//   together: measure each column's spread over the central band of the rect's
//   rows, take the widest textured run, and narrow the rect to it.
//
// # What is settled, what is mechanical, what was measured
//
// * **Settled, read out, never calibrated here**: `min_line_spread` = 8.0 and
//   both ends of MC-025's derivation of it, `uniform_tolerance` = 10,
//   `edge_threshold` = 24, `margin_px` = 3. This story adds a row restriction
//   and a selector; it retunes nothing.
// * **Mechanical**: `central_band`'s geometry, the selector's contract against
//   `textured_span`'s, and the counts. Pinned exactly.
// * **Oracle-free, and the one number this story chooses**:
//   `Tuning::central_band_fraction`. Its derivation is immediately below.
//
// # Deriving `central_band_fraction = 0.6`
//
// MC-025's derivation of `min_line_spread` is the standard: a floor from a
// control that fires, a ceiling from a control that fires, and a value between
// them with a corpus measurement behind it. Both ends of this one are fixtures
// in this file, so the **`unit`** gate is what tells a later story it has left
// the window - not a corpus score, and not the optional `integration` gate.
//
// 1. **Ceiling 2/3, from the chrome rows.** `common::page_in_margins_with_
//    chrome_bands` puts full-width texture in the outer sixth of the rect at
//    the top and at the bottom, which is the browser chrome and the taskbar
//    that survive inside the rect on a real screenshot. A central band of a
//    fraction at or below 2/3 misses both sixths entirely and locates the page
//    at columns 47..252; a fraction above 2/3 reaches into them, every column
//    of the band reads textured, and the locator gives the page back whole.
//    Measured on the fixture: at 2/3 the band is rows 40..199 and the answer
//    is 47..252, identical to the blanked variant; at 0.7 it is 46..253
//    against the blanked variant's 48..251; at 0.8 the answer is the whole
//    rect and the page is gone. The corpus agrees and is looser - MC-026
//    finding 7 scores 20 of 21 at 0.8 and 2 of 21 at 1.0 - so the fixture is
//    the binding end and that is deliberate.
//    `a_band_that_reaches_the_chrome_rows_gives_the_page_back_whole` is that
//    control.
// 2. **Floor 0.5, from the waist.** `common::page_with_a_waist` is
//    `Screenshot (2708).jpg` synthetically: a page whose outer 60 columns
//    carry art only outside the central half of the rows. A band at a fraction
//    of 0.5 lies exactly inside that waist, reads those columns as gutter and
//    locates the inner panel alone - 100..199 against the page's true 40..259,
//    a 60 px error on each side. A band above 0.5 sees rows where the outer
//    panels are art and locates the page. Measured on the fixture: the outer
//    panel column's spread over the band is 0.9917 at 0.5 and 9.1377 at 0.6,
//    against a `min_line_spread` of 8.0. The corpus agrees: MC-026 finding 7
//    records `Screenshot (2708).jpg` at `+1` on the left at 0.6 and at `+85`
//    at 0.5, and the whole-corpus score falls from 20 of 21 to 19 there.
//    `a_page_whose_outer_panels_miss_the_middle_needs_a_band_wider_than_half`
//    is that control.
// 3. **0.6, from the corpus.** Over the twenty-one marked corpus entries, the
//    left and right edges land inside MC-019's window on 17 at a fraction of
//    0.25, 19 at 0.4, 0.5 and 0.55, **20 at 0.6, 2/3, 0.7 and 0.8**, 6 at 0.9
//    and 2 at 1.0. 0.6 is the smallest fraction that reaches the maximum, it
//    sits 0.1 above the floor and 0.0667 below the ceiling, and it is within
//    3% of the window's arithmetic centre (0.5833).
//
// So **0.6**, in the window `(0.5, 2/3]`.
// `the_default_central_band_fraction_sits_inside_the_window_its_controls_measure`
// states that as an inequality over the two fixture constants, so a later
// story that moves the constant finds out here rather than in a corpus score.
//
// Two negative controls say what the locator must *not* do, because a locator
// with no negative control means nothing:
//
// * `common::soft_art(100, 94, 7)` - the art every scene in `tests/content.rs`
//   and `tests/edges.rs` builds - fills its rect edge to edge. Its flattest
//   column over the central band measures 8.2526 against a `min_line_spread`
//   of 8.0, so every column of it is textured, the widest run is the whole
//   profile, and `page_column` must return the rect untouched. This is the
//   same 0.26 of headroom MC-025 measured over the whole plane (8.2617), and
//   it is still the thinnest margin in this file;
// * `common::flat_gutter_only` - nothing but gutter - has no textured column
//   at all, so there is no run to select and the rect is untouched again.
//
// # Why the rect is untouched where the widest run reaches an end
//
// `page_column` narrows the rect only where the widest run has a flat column
// on **both** sides of it. That is not a tuning choice, it is what a page
// *margin* is: the flat browser background to the left of the page and to the
// right of it. Where the widest run reaches column 0 or the rect's last
// column, the rect has no page margin on that side and there is nothing for
// this stage to remove - it is looking at a bare artwork file, not at a
// screenshot of a reader. That rule is what
// `a_run_that_reaches_an_end_of_the_rect_is_not_a_page_column_and_is_left_alone`
// pins here; MC-026's corpus controls in `crates/engine/tests/corpus.rs` are
// what pin its consequence, and the story's `## Test plan` records that it
// separates the twenty-one marked entries from the six flagged ones exactly.

/// The one number this story chooses. See the derivation above.
#[test]
fn the_default_central_band_fraction_is_three_fifths() {
    assert_eq!(
        Tuning::default().central_band_fraction,
        0.6,
        "the central-band fraction is settled at 0.6 by the derivation in this \
         file's header; moving it is a corpus story's decision and not an \
         implementation detail"
    );
}

/// The derivation, as an inequality over the two fixtures that measure its
/// ends rather than as a comment.
///
/// A later story that raises the fraction past 2/3 breaks
/// `a_band_that_reaches_the_chrome_rows_gives_the_page_back_whole`; one that
/// lowers it to 0.5 or below breaks
/// `a_page_whose_outer_panels_miss_the_middle_needs_a_band_wider_than_half`.
/// This says so in one place, loudly, before either of those fails obscurely.
#[test]
fn the_default_central_band_fraction_sits_inside_the_window_its_controls_measure() {
    let t = Tuning::default();

    // The floor: the waist is exactly the central half of the fixture's rows,
    // so a band at or below that fraction fits inside it.
    let waist_fraction = (WAIST_BOTTOM - WAIST_TOP) as f32 / PAGE_H as f32;
    assert_eq!(waist_fraction, 0.5, "the waist is the central half exactly");
    assert!(
        t.central_band_fraction > waist_fraction,
        "central_band_fraction {} must be strictly above the waist's {waist_fraction}, \
         or the band fits inside a waist and the page's outer panels are missed by \
         {WAIST_INSET} px on each side",
        t.central_band_fraction
    );

    // The ceiling: the chrome bands occupy the outer sixth at each end, so a
    // band above two thirds of the rect reaches them.
    let clear_of_the_chrome = (PAGE_H - 2 * PAGE_SIXTH) as f32 / PAGE_H as f32;
    assert!(
        t.central_band_fraction <= clear_of_the_chrome,
        "central_band_fraction {} must be at or below {clear_of_the_chrome}, the share \
         of the rect that clears the chrome rows in the outer sixths, or every column \
         of the band reads textured and the page is given back whole",
        t.central_band_fraction
    );
}

// --- AC-1: the central band -------------------------------------------------

/// AC-1's geometry, exactly: the middle `central_band_fraction` of the rect's
/// rows, centred, with the columns untouched.
///
/// The arithmetic is pinned rather than described because it is not free.
/// `0.6f32` is `0.60000002384185791015625`, so the product is taken in `f64`
/// and truncated: at every height below the true product is never reached from
/// beneath and the floor is the mathematical one. A rect one row tall keeps
/// its row - a band of no rows would make every column's spread zero and the
/// locator silently inert - and a rect of no rows keeps none.
#[test]
fn the_central_band_is_the_middle_share_of_the_rects_rows_and_nothing_else() {
    let t = Tuning::default();
    let at = |h: u32| {
        central_band(
            Rect {
                x: 7,
                y: 11,
                w: 5,
                h,
            },
            &t,
        )
    };

    let wrong: Vec<(u32, u32, u32)> = [
        (0u32, 11u32, 0u32),
        (1, 11, 1),
        (2, 11, 1),
        (3, 12, 1),
        (10, 13, 6),
        (100, 31, 60),
        (101, 31, 60),
        (170, 45, 102),
        (240, 59, 144),
    ]
    .into_iter()
    .filter(|&(h, y, height)| at(h).y != y || at(h).h != height)
    .map(|(h, _, _)| (h, at(h).y, at(h).h))
    .collect();
    assert!(
        wrong.is_empty(),
        "central_band at {} must be floor(h * fraction) rows starting at \
         y + (h - that) / 2; (h, got y, got h) for each that is not: {wrong:?}",
        t.central_band_fraction
    );

    let full = Rect {
        x: 7,
        y: 11,
        w: 5,
        h: 240,
    };
    assert_eq!(
        (central_band(full, &t).x, central_band(full, &t).w),
        (full.x, full.w),
        "the band restricts rows and only rows: the columns are the rect's own"
    );
    assert_eq!(
        central_band(
            full,
            &Tuning {
                central_band_fraction: 1.0,
                ..Tuning::default()
            }
        ),
        full,
        "and a fraction of 1.0 is the rect itself, which is what makes the \
         restriction visible as a restriction"
    );
}

/// AC-1, whole, and the reason the restriction exists.
///
/// Two fixtures identical inside the middle two thirds of the rect and
/// different everywhere else: one with full-width texture in the outer sixths,
/// one with those sixths blanked to gutter. At the settled fraction the band
/// touches neither sixth, so the pixels that differ cannot reach the
/// statistic, and the locator must give the same answer on both - the answer
/// being the page, columns 47..252 at a `min_line_spread` of 8.0.
///
/// RED measured outside the test framework, on both fixtures: at 0.5, 0.6 and
/// 2/3 both give 47..252; at 0.7 they give 46..253 and 48..251; at 0.8, 0..299
/// and 49..250.
#[test]
fn a_row_outside_the_central_band_cannot_change_where_the_page_column_is() {
    let t = Tuning::default();
    let chrome = page_in_margins_with_chrome_bands();
    let blank = page_in_margins_with_blank_bands();
    let rect = whole(&chrome);

    let first = page_first_textured_column(t.min_line_spread);
    let last = page_last_textured_column(t.min_line_spread);
    let expected = Rect {
        x: first,
        w: last - first + 1,
        ..rect
    };

    assert_eq!(
        page_column(&chrome, rect, &t),
        expected,
        "AC-1: the page is columns {first}..={last} at min_line_spread {}, and the \
         chrome rows in the outer sixths of the rect must not reach the statistic",
        t.min_line_spread
    );
    assert_eq!(
        page_column(&blank, rect, &t),
        page_column(&chrome, rect, &t),
        "AC-1: the two fixtures differ only in rows the central band excludes, so \
         the locator must not be able to tell them apart"
    );

    // The premise, so that a fixture that stopped differing outside the band
    // could not make the assertion above vacuous.
    let differing: Vec<u32> = (0..PAGE_H)
        .filter(|&y| {
            (0..PAGE_W).any(|x| {
                chrome.data[(y * PAGE_W + x) as usize] != blank.data[(y * PAGE_W + x) as usize]
            })
        })
        .collect();
    assert_eq!(
        (differing.first().copied(), differing.last().copied()),
        (Some(0), Some(PAGE_H - 1)),
        "the fixtures must differ in the outer sixths, or this test compares a \
         fixture with itself"
    );
    assert!(
        differing
            .iter()
            .all(|&y| !(PAGE_SIXTH..PAGE_H - PAGE_SIXTH).contains(&y)),
        "and must differ nowhere else; rows that differ: {differing:?}"
    );
}

// --- AC-2: the widest run, against the outermost pair -----------------------

/// AC-2, whole. Both rules on one profile, in one test, because the difference
/// between them is the entire reason this story exists: on the column axis of
/// the corpus the widest run reaches 19 of 21 where the outermost pair reaches
/// 15 (MC-026 finding 1).
///
/// `textured_span` is **required to keep returning the outermost pair**. A
/// flat run between two textured ones is a panel gutter and MC-005's decision
/// 13 keeps the panels rather than cutting there; changing what it means is
/// this story's `## Out of scope`, and this assertion is what says so.
#[test]
fn the_widest_textured_run_is_not_the_outermost_textured_pair() {
    let t = Tuning::default();
    // One narrow run, a flat gap, one wide run. Every value is far from the
    // threshold, so this is about the selection and not about the comparison.
    let profile = [0.0, 20.0, 0.0, 0.0, 30.0, 30.0, 30.0, 0.0];

    assert_eq!(
        widest_textured_run(&profile, &t),
        Some((4, 6)),
        "AC-2: the widest run is indices 4..=6, three long, and not the one-long \
         run at index 1"
    );
    assert_eq!(
        textured_span(&profile, &t),
        Some((1, 6)),
        "AC-2: and `textured_span` still spans the outermost textured pair on the \
         same profile - MC-005's decision 13, unchanged by this story"
    );
}

/// Zero, one, many - and the tie, which is a decision rather than an accident.
///
/// A tie is broken by taking the **first** run. The corpus case that turns on
/// the selection is `Screenshot (1661).png`, which carries a second textured
/// region to the right of the page; there the page is strictly wider and no
/// tie arises, so nothing is riding on which way this falls. It is pinned so
/// that the answer is the same on every machine and in every later story.
#[test]
fn zero_one_and_many_runs_and_the_tie_between_two_of_the_same_width() {
    let t = Tuning::default();
    assert_eq!(
        widest_textured_run(&[], &t),
        None,
        "an empty profile has no run"
    );
    assert_eq!(
        widest_textured_run(&[0.0, 1.0, 2.0], &t),
        None,
        "and a profile with no textured index has none either"
    );
    assert_eq!(
        widest_textured_run(&[0.0, 20.0, 0.0], &t),
        Some((1, 1)),
        "a single textured index is a run one long, both bounds inclusive"
    );
    assert_eq!(
        widest_textured_run(&[20.0, 0.0, 30.0], &t),
        Some((0, 0)),
        "two runs of the same width: the first wins"
    );
    assert_eq!(
        widest_textured_run(&[20.0, 20.0, 0.0, 30.0], &t),
        Some((0, 1)),
        "a run still open when the profile ends at its start is closed at its own end"
    );
    assert_eq!(
        widest_textured_run(&[20.0, 0.0, 30.0, 30.0], &t),
        Some((2, 3)),
        "and a run still open when the profile ends is closed at the last index"
    );
}

/// The threshold is read from the `Tuning` argument at the one comparison
/// site, and "at or above" is `>=` - the same treatment `edge_threshold` gets
/// in `strong_lines` and `min_line_spread` gets in `textured_span`.
#[test]
fn the_widest_run_reads_its_threshold_from_the_tuning_like_the_span_does() {
    let t = Tuning::default();
    assert_eq!(
        widest_textured_run(&[t.min_line_spread], &t),
        Some((0, 0)),
        "at or above is >=, as it is for textured_span"
    );
    assert_eq!(
        widest_textured_run(&[t.min_line_spread - 1.0], &t),
        None,
        "and strictly below is flat"
    );

    // Same profile, threshold moved, answer moves with it: at 8.0 the wide run
    // is indices 3..=5 and the narrow one is index 1; at 25.0 only index 4
    // survives at all.
    let profile = [0.0, 20.0, 0.0, 10.0, 30.0, 10.0];
    assert_eq!(widest_textured_run(&profile, &t), Some((3, 5)));
    assert_eq!(
        widest_textured_run(
            &profile,
            &Tuning {
                min_line_spread: 25.0,
                ..Tuning::default()
            }
        ),
        Some((4, 4)),
        "the threshold is the argument's, never a literal at the comparison"
    );
}

// --- The derivation's two ends, as controls that fire -----------------------

/// The **floor** under `central_band_fraction`, and a control that fires below
/// it.
///
/// `page_with_a_waist` is a page whose outer `WAIST_INSET` columns carry art
/// only outside the central half of the rows. At a fraction of 0.5 the band is
/// exactly that waist: those columns read as gutter, the widest textured run
/// is the inner panel alone, and the locator is `WAIST_INSET` px wrong on each
/// side. At the settled fraction it sees rows where the outer panels are art
/// and locates the page.
///
/// RED measured outside the test framework: the outer panel column's mean
/// absolute deviation over the band is 0.991667 at 0.5 and 9.137731 at 0.6,
/// against a `min_line_spread` of 8.0.
#[test]
fn a_page_whose_outer_panels_miss_the_middle_needs_a_band_wider_than_half() {
    let t = Tuning::default();
    let img = page_with_a_waist();
    let rect = whole(&img);
    let (page_first, page_last) = waist_page_columns();
    let (inner_first, inner_last) = waist_inner_columns();

    assert_eq!(
        page_column(&img, rect, &t),
        Rect {
            x: page_first,
            w: page_last - page_first + 1,
            ..rect
        },
        "at the settled central_band_fraction {} the band reaches past the waist and \
         the whole page is located",
        t.central_band_fraction
    );

    let at_the_floor = Tuning {
        central_band_fraction: 0.5,
        ..Tuning::default()
    };
    assert_eq!(
        page_column(&img, rect, &at_the_floor),
        Rect {
            x: inner_first,
            w: inner_last - inner_first + 1,
            ..rect
        },
        "and at a fraction of 0.5 the band fits inside the waist, so the locator \
         finds the inner panel alone and is {WAIST_INSET} px wrong on each side. \
         That is the floor, and it fires."
    );
}

/// The **ceiling** on `central_band_fraction`, and a control that fires above
/// it.
///
/// `page_in_margins_with_chrome_bands` carries full-width texture in the outer
/// sixth of the rect at each end - the browser chrome and the taskbar that
/// survive inside the rect on a real screenshot, and MC-025's own diagnosis of
/// why the column axis scores 0 of 21 when the whole rect is measured. A band
/// that reaches those rows reads every column as textured, so there is no page
/// margin anywhere and the rect comes back whole.
///
/// RED measured outside the test framework: column 0's mean absolute deviation
/// over the band is 0.75 at 2/3, 3.0887 at 0.7 and 8.9583 at 0.8, against a
/// `min_line_spread` of 8.0.
#[test]
fn a_band_that_reaches_the_chrome_rows_gives_the_page_back_whole() {
    let t = Tuning::default();
    let img = page_in_margins_with_chrome_bands();
    let rect = whole(&img);
    let first = page_first_textured_column(t.min_line_spread);
    let last = page_last_textured_column(t.min_line_spread);

    assert_eq!(
        page_column(&img, rect, &t),
        Rect {
            x: first,
            w: last - first + 1,
            ..rect
        },
        "at the settled central_band_fraction {} the band clears both chrome rows \
         and the page is located at columns {first}..={last}",
        t.central_band_fraction
    );

    let past_the_ceiling = Tuning {
        central_band_fraction: 0.8,
        ..Tuning::default()
    };
    assert_eq!(
        page_column(&img, rect, &past_the_ceiling),
        rect,
        "and at a fraction of 0.8 the band reaches the chrome rows, every column of \
         it reads textured, and the page is given back whole - which is MC-026 \
         finding 7's 2 of 21 at a fraction of 1.0, on one fixture. That is the \
         ceiling, and it fires."
    );
}

// --- The negative controls --------------------------------------------------

/// The first negative control: a rect that is art edge to edge has no page
/// margin, so the locator must leave it exactly alone.
///
/// `common::soft_art(100, 94, 7)` is the art every scene in `tests/content.rs`
/// and `tests/edges.rs` builds. RED measured its flattest column over the
/// central band at **8.252551** against a `min_line_spread` of 8.0 - the same
/// 0.26 of headroom MC-025 measured over the whole plane (8.261657), and still
/// the thinnest margin in this file. If a later story raised `min_line_spread`
/// above it, this locator would start reading that art as page margin and
/// cutting into the scenes those suites pin.
#[test]
fn the_locator_leaves_a_rect_that_is_art_edge_to_edge_alone() {
    let t = Tuning::default();
    let art = soft_art(100, 94, 7);
    let band = central_band(whole(&art), &t);
    let spread = column_spreads(&art, band);

    assert!(
        trough(&spread) > t.min_line_spread,
        "every column of the existing art fixture must stay textured over the \
         central band at min_line_spread {}; the flattest measures {}, which is the \
         ceiling on that threshold restated over the band",
        t.min_line_spread,
        trough(&spread)
    );
    assert_eq!(
        widest_textured_run(&spread, &t),
        Some((0, spread.len() - 1)),
        "so the widest run is the whole profile"
    );
    assert_eq!(
        page_column(&art, whole(&art), &t),
        whole(&art),
        "and the rect comes back untouched: there is no page margin to remove"
    );
}

/// The second negative control: nothing but gutter has no textured column at
/// all, so there is no run to select and the rect is untouched again.
///
/// `common::flat_gutter_only` is flat in the mean-absolute-deviation sense the
/// locator measures and *not* uniform in the `max - min` sense stage 1
/// measures, which is what makes it a control rather than a curiosity - it
/// reaches the locator instead of being trimmed away first. RED measured its
/// column spreads over the central band at 0.555556 to 1.095679.
#[test]
fn the_locator_finds_no_page_column_in_a_rect_that_is_all_gutter() {
    let t = Tuning::default();
    let img = flat_gutter_only(200, 120);
    let band = central_band(whole(&img), &t);
    let spread = column_spreads(&img, band);

    assert!(
        peak(&spread) < t.min_line_spread,
        "a flat gutter must not read as art on the column axis at min_line_spread \
         {}; the widest column measures {}",
        t.min_line_spread,
        peak(&spread)
    );
    assert_eq!(
        widest_textured_run(&spread, &t),
        None,
        "so there is no textured run to select"
    );
    assert_eq!(
        page_column(&img, whole(&img), &t),
        whole(&img),
        "and a rect with no page column in it is left exactly as it was"
    );
}

/// The rule that makes the two controls above more than a coincidence: a page
/// column is a textured run with a flat page margin on **both** sides of it.
///
/// Where the widest run reaches column 0 or the rect's last column, that side
/// has no page margin, there is nothing for this stage to remove, and the rect
/// is returned untouched. The same fixture answers both halves: given the
/// whole plane the locator finds the page inside its margins; given a rect
/// that starts and ends on the page itself, the run reaches both ends and the
/// rect comes back as it went in.
#[test]
fn a_run_that_reaches_an_end_of_the_rect_is_not_a_page_column_and_is_left_alone() {
    let t = Tuning::default();
    let img = page_in_margins();
    let first = page_first_textured_column(t.min_line_spread);
    let last = page_last_textured_column(t.min_line_spread);

    let inside_the_page = Rect {
        x: first,
        y: 0,
        w: last - first + 1,
        h: PAGE_H,
    };
    assert_eq!(
        page_column(&img, inside_the_page, &t),
        inside_the_page,
        "a rect whose own outer columns are the page has no page margin on either \
         side, so there is nothing to remove"
    );

    let one_margin_only = Rect {
        x: 0,
        y: 0,
        w: last + 1,
        h: PAGE_H,
    };
    assert_eq!(
        page_column(&img, one_margin_only, &t),
        one_margin_only,
        "and a rect with a margin on the left but none on the right is the same \
         answer: this stage removes page margins in pairs or not at all, which is \
         what keeps it off a bare artwork file that happens to start with a \
         blank column"
    );

    let both_margins = whole(&img);
    assert_eq!(
        page_column(&img, both_margins, &t),
        Rect {
            x: first,
            w: last - first + 1,
            ..both_margins
        },
        "while the same fixture with both its margins is located at columns \
         {first}..={last}"
    );
}
