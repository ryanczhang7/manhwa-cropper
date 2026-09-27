//! MC-053, AC-7: the page column keeps dark, low-texture art beside a flat
//! page margin, and still lets no page margin in - seen through [`detect`].
//! This is what the required `unit` gate sees of the story; the corpus half
//! is `crates/engine/tests/corpus_page_column.rs` (AC-1 to AC-3) and the
//! existing corpus suites (AC-4, AC-6).
//!
//! # The bug these fixtures reproduce
//!
//! MC-027's page column (`flat::page_column`) keeps the widest run of columns
//! whose spread - mean absolute deviation over the central band of rows - is
//! at or above `Tuning::min_line_spread` (8.0). Dark, low-texture art has
//! outer columns whose spread is **below** 8.0 although they are plainly not
//! page background, so the locator trims them as page margin and the crop cuts
//! the art: `2025-07-17 14_41_58.png` (columns 928..935 at spread 1.9..4.2),
//! `2025-07-17 14_55_10.png` and `Screenshot (73).png`.
//!
//! # What "page background" means here
//!
//! Settled, MC-049's predicate, read out and not re-derived: a column is page
//! background when at least [`PAGE_BACKGROUND_SHARE`] of its pixels lie within
//! `uniform_tolerance` of that column's median luma (the upper median,
//! `sorted[n / 2]`). On the corpus it is measured over the mark's rows; on
//! these fixtures the art runs the whole height, so it is measured over every
//! row, and the premise test also checks it over the central band.
//!
//! # The fixtures: invented here, the story says no oracle exists
//!
//! One geometry, [`W`] x [`H`], mirrored left to right:
//!
//! - **the page margin**, [`MARGIN`] columns at each side, tone [`PAGE_TONE`];
//! - **the dark low-texture art**, [`LOW_W`] columns inside each margin: a
//!   flat dark tone [`ART_TONE`] with one pixel in ten lifted by
//!   [`LOW_SPECK`]. Spread about 2.5 - far below 8.0 - and share 0.90, so it
//!   is art by the predicate;
//! - **the textured art core** between them, [`ART_TONE`] plus and minus
//!   [`CORE_AMPLITUDE`] on alternate rows: spread 10, above the rule;
//! - **a panel seam** of two columns deep inside the core, whose step is far
//!   above `edge_threshold`, so `strong_lines` finds a column line and
//!   MC-025's `textured_box` stands down - exactly as on a real screenshot, and
//!   as MC-027's own fixture does.
//!
//! No adjacent-column step at either art edge comes near `edge_threshold`, so
//! `content_box` has no strip to peel there.
//!
//! The two cases differ **only in the page margin**:
//!
//! - *Case A* ([`Margin::Flat`]): a flat margin of a nearby tone, lifted by
//!   [`MARGIN_SPECK`] on one pixel in fifty so it is not uniform and reaches
//!   the locator instead of being trimmed by stage 1. Spread about 0.8, share
//!   0.98: page background;
//! - *Case B* ([`Margin::Noisy`]): the same margin carrying compression-like
//!   noise - plus and minus [`NOISE_AMPLITUDE`] in a checkerboard, with the
//!   same sparse lifts. Spread about 4.3, share 0.98: still page background,
//!   and the `Screenshot (2630).jpg` shape (share >= 0.95, spread up to about
//!   5; its column 954 reads 5.27).
//!
//! # The controls
//!
//! - **On `c004d96` the shipped rule fails case A and passes case B**: it
//!   keeps the core only, which cuts all [`LOW_W`] art columns on each side,
//!   and lets no margin in. Recorded in the story's handoff.
//! - **A `min_line_spread` lowered far enough to pass case A fails case B.**
//!   Asserted twice, both on **MC-027's rule frozen as a test-local stand-in**
//!   (so GREEN's change to `page_column` cannot move them): at
//!   [`LOWERED_SPREAD`]
//!   ([`a_min_line_spread_lowered_to_pass_case_a_lets_case_bs_noisy_margin_in`]),
//!   and over every threshold at once
//!   ([`no_single_min_line_spread_keeps_the_dark_art_and_keeps_out_the_noisy_margin`]).
//!   The art's flattest column is flatter than the noisy margin's roughest
//!   one, which is the corpus's own finding (`14_41_58` left at 1.9 against
//!   `(2630)` column 954 at 5.27).
//! - The premise test pins every spread and share the above leans on, so a
//!   fixture that drifted goes red rather than making a case vacuous.

use cropper_core::content::content_box;
use cropper_core::edges::{col_profile, strong_lines};
use cropper_core::flat::{central_band, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::{Luma, Rect, Tuning, detect};

// --- The settled constant ---------------------------------------------------

/// MC-049's page-background share, settled: a column is page background when
/// at least this share of its pixels lie within `uniform_tolerance` of its
/// median.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

// --- The fixture's geometry, invented here ----------------------------------

/// Columns of page margin at each side.
const MARGIN: u32 = 40;
/// Columns of dark low-texture art inside each margin: wider than the default
/// `margin_px` of 3, so the cut shows at margin 3 as well as at 0.
const LOW_W: u32 = 12;
/// The fixture's width.
const W: u32 = 300;
/// The fixture's height. Even, so alternate-row texture has an exact mean.
const H: u32 = 240;
/// The first column of the panel seam.
const SEAM_X: u32 = 149;
/// The seam's width.
const SEAM_W: u32 = 2;

/// The first art column on the left, and the last on the right: what the crop
/// must keep.
const ART_FIRST: u32 = MARGIN;
const ART_LAST: u32 = W - MARGIN - 1;
/// The textured core's first and last column: what the shipped rule keeps.
const CORE_FIRST: u32 = MARGIN + LOW_W;
const CORE_LAST: u32 = W - MARGIN - LOW_W - 1;

/// The dark art's base tone.
const ART_TONE: u8 = 40;
/// How far one pixel in ten of the low-texture art is lifted.
const LOW_SPECK: u8 = 14;
/// The textured core's amplitude about [`ART_TONE`]: spread exactly 10.
const CORE_AMPLITUDE: u8 = 10;
/// The seam's amplitude about [`ART_TONE`]: a 30-level step either side of the
/// core, over `edge_threshold` (24).
const SEAM_AMPLITUDE: u8 = 40;

/// The page margin's tone: near the art's, as on `14_41_58` (art 20, page 23).
const PAGE_TONE: u8 = 44;
/// How far one margin pixel in fifty is lifted, so no margin column is uniform.
const MARGIN_SPECK: u8 = 20;
/// Case B's noise amplitude about [`PAGE_TONE`].
const NOISE_AMPLITUDE: u8 = 4;

/// A `min_line_spread` lowered below the art's flattest column and above case
/// A's margin: the global fix AC-2 and AC-7 exist to rule out.
const LOWERED_SPREAD: f32 = 2.0;

/// Which page margin the fixture carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Margin {
    /// Case A: flat, of a nearby tone.
    Flat,
    /// Case B: page background carrying compression-like noise.
    Noisy,
}

/// Whether `x` is a page-margin column.
fn is_margin(x: u32) -> bool {
    !(ART_FIRST..=ART_LAST).contains(&x)
}

/// Whether `x` is a column of the dark low-texture art.
fn is_low_art(x: u32) -> bool {
    (ART_FIRST..CORE_FIRST).contains(&x) || (CORE_LAST + 1..=ART_LAST).contains(&x)
}

/// One pixel.
fn pixel(x: u32, y: u32, margin: Margin) -> u8 {
    if is_margin(x) {
        let lifted = (x + 3 * y).is_multiple_of(50);
        if lifted {
            return PAGE_TONE + MARGIN_SPECK;
        }
        return match margin {
            Margin::Flat => PAGE_TONE,
            Margin::Noisy if (x + y).is_multiple_of(2) => PAGE_TONE + NOISE_AMPLITUDE,
            Margin::Noisy => PAGE_TONE - NOISE_AMPLITUDE,
        };
    }
    if is_low_art(x) {
        return if (7 * x + y).is_multiple_of(10) {
            ART_TONE + LOW_SPECK
        } else {
            ART_TONE
        };
    }
    let amplitude = if (SEAM_X..SEAM_X + SEAM_W).contains(&x) {
        SEAM_AMPLITUDE
    } else {
        CORE_AMPLITUDE
    };
    if y.is_multiple_of(2) {
        ART_TONE + amplitude
    } else {
        ART_TONE - amplitude
    }
}

/// The fixture.
fn scene(margin: Margin) -> Luma {
    let mut data = Vec::with_capacity((W * H) as usize);
    for y in 0..H {
        for x in 0..W {
            data.push(pixel(x, y, margin));
        }
    }
    Luma {
        width: W,
        height: H,
        data,
    }
}

// --- Measuring a column, test-side ------------------------------------------

/// Column `x`'s pixels over `rows`.
fn column(img: &Luma, x: u32, rows: std::ops::Range<u32>) -> Vec<u8> {
    rows.map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

/// Mean absolute deviation about the mean: the statistic `min_line_spread` is
/// compared against.
fn spread(values: &[u8]) -> f64 {
    let n = values.len() as f64;
    let mean = values.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
    values
        .iter()
        .map(|&v| (f64::from(v) - mean).abs())
        .sum::<f64>()
        / n
}

/// MC-049's share: pixels within `tolerance` of the upper median.
fn share(values: &[u8], tolerance: u8) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let median = sorted[sorted.len() / 2];
    let near = sorted
        .iter()
        .filter(|&&v| v.abs_diff(median) <= tolerance)
        .count();
    near as f64 / sorted.len() as f64
}

/// The rows `page_column` measures a column's spread over.
fn band_rows(t: &Tuning) -> std::ops::Range<u32> {
    let band = central_band(
        Rect {
            x: 0,
            y: 0,
            w: W,
            h: H,
        },
        t,
    );
    band.y..band.y + band.h
}

/// The `(min, max)` of `f` over the columns `xs`.
fn range(xs: impl Iterator<Item = u32>, f: impl Fn(u32) -> f64) -> (f64, f64) {
    xs.map(f)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        })
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

/// `detect`'s crop, as `(first column, last column)`, both inclusive.
fn crop_columns(img: &Luma, t: &Tuning) -> (u32, u32) {
    let rect = detect(img, t)
        .expect("the fixture is not one flat colour")
        .rect;
    (rect.x, rect.x + rect.w - 1)
}

// --- The premise ------------------------------------------------------------

/// Every number the two cases lean on, measured off the fixtures rather than
/// assumed. If any of them drifts, case A or case B stops meaning what it
/// says, and this is the test that goes red first.
#[test]
fn the_fixtures_dark_art_is_below_the_rule_yet_not_page_background_and_both_margins_are() {
    let t = Tuning::default();
    let band = band_rows(&t);
    let tol = t.uniform_tolerance;
    let rule = f64::from(t.min_line_spread);
    let a = scene(Margin::Flat);
    let b = scene(Margin::Noisy);

    let low = || (0..W).filter(|&x| is_low_art(x));
    let core = || (CORE_FIRST..=CORE_LAST).filter(|x| !(SEAM_X..SEAM_X + SEAM_W).contains(x));
    let margins = || (0..W).filter(|&x| is_margin(x));

    let low_spread = range(low(), |x| spread(&column(&a, x, band.clone())));
    let low_share_all = range(low(), |x| share(&column(&a, x, 0..H), tol));
    let low_share_band = range(low(), |x| share(&column(&a, x, band.clone()), tol));
    let core_spread = range(core(), |x| spread(&column(&a, x, band.clone())));
    let core_share = range(core(), |x| share(&column(&a, x, 0..H), tol));
    let flat_spread = range(margins(), |x| spread(&column(&a, x, band.clone())));
    let flat_share = range(margins(), |x| share(&column(&a, x, 0..H), tol));
    let flat_share_band = range(margins(), |x| share(&column(&a, x, band.clone()), tol));
    let noisy_spread = range(margins(), |x| spread(&column(&b, x, band.clone())));
    let noisy_share = range(margins(), |x| share(&column(&b, x, 0..H), tol));
    let noisy_share_band = range(margins(), |x| share(&column(&b, x, band.clone()), tol));
    let uniform = |img: &Luma, x: u32| {
        let c = column(img, x, 0..H);
        c.iter().max().unwrap() - c.iter().min().unwrap() <= tol
    };

    let measured = format!(
        "low art spread {low_spread:.3?} share (all rows) {low_share_all:.3?} (band) \
         {low_share_band:.3?}; core spread {core_spread:.3?} share {core_share:.3?}; \
         flat margin spread {flat_spread:.3?} share {flat_share:.3?} (band) \
         {flat_share_band:.3?}; noisy margin spread {noisy_spread:.3?} share \
         {noisy_share:.3?} (band) {noisy_share_band:.3?}"
    );
    println!("{measured}");

    assert!(
        low_spread.1 < rule,
        "case A's premise: every dark art column must read below min_line_spread \
         {rule} over the central band - that is the bug. {measured}"
    );
    assert!(
        low_share_all.1 < PAGE_BACKGROUND_SHARE && low_share_band.1 < PAGE_BACKGROUND_SHARE,
        "case A's premise: and none may be page background by MC-049's predicate, \
         over every row or over the band. {measured}"
    );
    assert!(
        core_spread.0 >= rule && core_share.1 < PAGE_BACKGROUND_SHARE,
        "the core is textured art by both measures, so the shipped rule has a run \
         to keep. {measured}"
    );
    assert!(
        flat_share.0 >= PAGE_BACKGROUND_SHARE && flat_share_band.0 >= PAGE_BACKGROUND_SHARE,
        "case A's margin is page background by the predicate. {measured}"
    );
    assert!(
        noisy_share.0 >= PAGE_BACKGROUND_SHARE && noisy_share_band.0 >= PAGE_BACKGROUND_SHARE,
        "case B's margin is page background by the predicate too. {measured}"
    );
    assert!(
        noisy_spread.1 <= 5.5 && noisy_spread.0 > low_spread.1,
        "case B's margin is the (2630) shape - spread up to about 5 - and is rougher \
         than the dark art's roughest column, so no one threshold separates them. \
         {measured}"
    );
    assert!(
        flat_spread.1 < LOWERED_SPREAD.into() && f64::from(LOWERED_SPREAD) <= low_spread.0,
        "the lowered control threshold {LOWERED_SPREAD} sits above case A's margin \
         and at or below the dark art's flattest column. {measured}"
    );
    assert!(
        PAGE_TONE.abs_diff(ART_TONE) <= tol,
        "case A's margin is of a nearby tone to the art"
    );
    assert!(
        (0..W)
            .filter(|&x| is_margin(x))
            .all(|x| !uniform(&a, x) && !uniform(&b, x)),
        "no margin column is uniform, so stage 1 leaves the margins for the locator"
    );

    for img in [&a, &b] {
        let whole = Rect {
            x: 0,
            y: 0,
            w: W,
            h: H,
        };
        let lines = strong_lines(&col_profile(img, whole), &t);
        assert!(
            !lines.is_empty()
                && lines
                    .iter()
                    .all(|l| l.start + 1 >= SEAM_X as usize && l.end < (SEAM_X + SEAM_W) as usize),
            "the seam, and only the seam, is a strong column line: {lines:?}"
        );
        let first = trim_uniform(img, &t).expect("not uniform");
        let found = content_box(img, first, &t);
        let second = trim_within(img, found.rect, &t).expect("not uniform");
        let textured = textured_box(img, second, &t);
        assert!(
            first == whole && found.removed.is_empty() && !found.ambiguous,
            "nothing is trimmed or peeled before the locators: first {first:?}, \
             removed {:?}",
            found.removed
        );
        assert_eq!(
            (textured.x, textured.w),
            (0, W),
            "and MC-025's textured_box stands down on the column axis, so the page \
             column stage is the one that decides the columns"
        );
    }
}

// --- Case A -----------------------------------------------------------------

/// AC-7 case A: dark, low-texture art beside a flat page margin of a nearby
/// tone. At both margins the crop keeps every art column and takes no margin
/// column beyond the `margin_px` the pipeline adds - exactly the art, grown
/// by `margin_px`.
///
/// On `c004d96` it keeps the textured core only, columns 52..=247, and cuts
/// all twelve dark columns on each side.
#[test]
fn dark_low_texture_art_beside_a_flat_page_margin_is_kept_at_both_margins() {
    let img = scene(Margin::Flat);
    let mut wrong = Vec::new();
    for t in both_margins() {
        let got = crop_columns(&img, &t);
        let want = (ART_FIRST - t.margin_px, ART_LAST + t.margin_px);
        if got != want {
            wrong.push(format!(
                "margin_px {}: crop columns {}..={}, want {}..={} (the art {ART_FIRST}..={ART_LAST} \
                 grown by the margin; cut {} on the left and {} on the right)",
                t.margin_px,
                got.0,
                got.1,
                want.0,
                want.1,
                i64::from(got.0) - i64::from(want.0),
                i64::from(want.1) - i64::from(got.1),
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-7 case A: the page column locator must keep dark art whose spread is \
         below min_line_spread but which is not page background, and take in none \
         of the flat page margin beside it.\n{}",
        wrong.join("\n")
    );
}

// --- Case B -----------------------------------------------------------------

/// AC-7 case B: the same image, with a page margin that is page background by
/// the predicate but carries compression-like noise (spread about 4.3). At
/// both margins none of it is let in: the crop reaches no further than the
/// art grown by `margin_px`.
///
/// Passes on `c004d96`, which keeps the core only; this is the case a global
/// lowering of `min_line_spread` fails.
#[test]
fn a_noisy_page_margin_beside_dark_art_is_not_let_in_at_both_margins() {
    let img = scene(Margin::Noisy);
    let mut wrong = Vec::new();
    for t in both_margins() {
        let got = crop_columns(&img, &t);
        let (lo, hi) = (ART_FIRST - t.margin_px, ART_LAST + t.margin_px);
        if got.0 < lo || got.1 > hi {
            wrong.push(format!(
                "margin_px {}: crop columns {}..={}, which lets in {} margin columns on \
                 the left and {} on the right beyond {lo}..={hi}",
                t.margin_px,
                got.0,
                got.1,
                lo.saturating_sub(got.0),
                got.1.saturating_sub(hi),
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-7 case B: a page margin that is page background by MC-049's predicate \
         must stay out of the crop even when it carries noise above the dark art's \
         spread.\n{}",
        wrong.join("\n")
    );
}

// --- The controls: MC-027's rule, frozen as a test-local stand-in -----------
//
// The fix these controls rule out is "lower `min_line_spread` globally". They
// must go on ruling it out after GREEN has changed `flat::page_column`, so
// they do not run the shipped stage: they run **MC-027's rule as it stood on
// `c004d96`**, re-implemented here - each column's spread over the central
// band of the rect's rows, the widest run at or above a threshold, and the
// interior rule (a run reaching either end of the rect leaves the rect
// alone). MC-052's `either_side_flat` is the precedent.
//
// What earns the stand-in: in MC-053's RED, at 8.0, it gave exactly the
// shipped `detect`'s margin-0 columns on both fixtures (52..=247), and its
// corpus twin in `crates/engine/tests/corpus_page_column.rs` reproduced the
// shipped page column on all 26 marked `tuning` entries. Both are pasted in
// the story's handoff.

/// The mean absolute deviation of `values` about their own mean, as one exact
/// rational and then rounded to `f32` - the arithmetic MC-027's stage used.
fn mc027_spread(values: &[u8]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let n = values.len() as u64;
    let total: u64 = values.iter().map(|&v| u64::from(v)).sum();
    let deviations: u64 = values
        .iter()
        .map(|&v| (n * u64::from(v)).abs_diff(total))
        .sum();
    (deviations as f64 / (n * n) as f64) as f32
}

/// The widest run of `spread` at or above `threshold`, both bounds inclusive,
/// the first of equal width winning; `None` when no index reaches it.
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

/// MC-027's page column over the whole of `img` at `threshold`, as
/// `(first column, last column)`, both inclusive. The premise test shows no
/// earlier stage touches these fixtures' columns, so the whole image is the
/// rect the stage is handed.
fn mc027_page_columns(img: &Luma, threshold: f32) -> (u32, u32) {
    let band = band_rows(&Tuning::default());
    let spread: Vec<f32> = (0..img.width)
        .map(|x| mc027_spread(&column(img, x, band.clone())))
        .collect();
    match mc027_widest_run(&spread, threshold) {
        Some((first, last)) if first > 0 && last + 1 < spread.len() => (first as u32, last as u32),
        _ => (0, img.width - 1),
    }
}

/// AC-7's control: MC-027's rule with `min_line_spread` lowered to
/// [`LOWERED_SPREAD`] - below the dark art's flattest column - keeps exactly
/// case A's art and lets **all** of case B's noisy margin in, columns 0..=39
/// and 260..=299, because the widest run then reaches both ends of the image
/// and the interior rule gives the rect back whole. A global threshold is the
/// fix this story rules out, and this is it failing on a synthetic page.
#[test]
fn a_min_line_spread_lowered_to_pass_case_a_lets_case_bs_noisy_margin_in() {
    let got_a = mc027_page_columns(&scene(Margin::Flat), LOWERED_SPREAD);
    let got_b = mc027_page_columns(&scene(Margin::Noisy), LOWERED_SPREAD);
    let let_in: Vec<u32> = (got_b.0..=got_b.1).filter(|&x| is_margin(x)).collect();
    let margin_columns: Vec<u32> = (0..W).filter(|&x| is_margin(x)).collect();
    println!(
        "MC-027's rule at min_line_spread {LOWERED_SPREAD}: case A {got_a:?}, case B \
         {got_b:?}, {} margin columns let in",
        let_in.len()
    );
    assert_eq!(
        (got_a, got_b, let_in),
        ((ART_FIRST, ART_LAST), (0, W - 1), margin_columns),
        "AC-7's control: MC-027's rule at min_line_spread {LOWERED_SPREAD} must keep \
         exactly case A's art ({ART_FIRST}..={ART_LAST}) and give case B back whole, \
         every margin column let in. `(case A columns, case B columns, case B margin \
         columns let in)`"
    );
}

/// AC-7's control, over every threshold at once: on MC-027's rule **every**
/// `min_line_spread` that keeps all of case A's art and none of its margin
/// also takes in case B's margin. And there is at least one such threshold,
/// so the claim is not vacuous.
///
/// The thresholds tried are every column spread either fixture has over the
/// band - the only values at which the rule's answer can change.
#[test]
fn no_single_min_line_spread_keeps_the_dark_art_and_keeps_out_the_noisy_margin() {
    let band = band_rows(&Tuning::default());
    let a = scene(Margin::Flat);
    let b = scene(Margin::Noisy);
    let mut thresholds: Vec<f32> = [&a, &b]
        .iter()
        .flat_map(|img| (0..W).map(|x| mc027_spread(&column(img, x, band.clone()))))
        .collect();
    thresholds.sort_by(f32::total_cmp);
    thresholds.dedup();

    let mut passes_a = Vec::new();
    let mut separates = Vec::new();
    for &threshold in &thresholds {
        if mc027_page_columns(&a, threshold) == (ART_FIRST, ART_LAST) {
            passes_a.push(threshold);
            let (first, last) = mc027_page_columns(&b, threshold);
            if first >= ART_FIRST && last <= ART_LAST {
                separates.push(threshold);
            }
        }
    }
    println!("thresholds that keep exactly case A's art: {passes_a:?}");
    assert!(
        !passes_a.is_empty(),
        "the control is vacuous: no threshold keeps exactly case A's art"
    );
    assert!(
        separates.is_empty(),
        "AC-7's control: a global min_line_spread that keeps case A's dark art must \
         let case B's noisy margin in, and these thresholds did not: {separates:?}"
    );
}
