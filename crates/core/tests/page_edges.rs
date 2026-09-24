//! MC-049 AC-6: on a synthetic page, the crop's left and right columns are the
//! art's own first and last columns - no page background on either side.
//!
//! The user, 2026-09-23: *"The cropping from the side isnt tight enough and I
//! still see the page on the right and left side."* The column locator MC-027
//! built already stops on the boundary between flat page margin and textured
//! art; what the user sees is `margin::expand` adding `Tuning::margin_px`
//! columns of flat page back on each side afterwards. MC-049 rules that margin
//! to 0, and this file is the required `unit` gate's half of that ruling (the
//! corpus half is `crates/engine/tests/corpus_sides.rs`, `integration` only).
//!
//! # Which rule decides the boundary, read out of the source and not invented
//!
//! On a reader page the side boundary is decided by stage 3c,
//! [`cropper_core::flat::page_column`]: it takes the **widest run of columns
//! whose spread** - mean absolute deviation about the column's own mean,
//! measured over [`cropper_core::flat::central_band`] of the rect's rows -
//! **is at or above [`Tuning::min_line_spread`]** (`>=`, read in
//! `edges::widest_textured_run`), with a flat column on both sides of it.
//! A column below that threshold is page margin; at or above it, art. The
//! constant is the settled `min_line_spread = 8.0` (MC-025). Nothing here
//! introduces a new rule or a new number.
//!
//! So the boundary fixture puts the art's outermost column **exactly on** that
//! rule - spread 8.0, bit for bit - and its twin **one pixel below** it,
//! spread `8 - 1/144`. The first must be kept and the second trimmed, at
//! `Tuning::default()`, with the crop edge on the art column itself.
//!
//! # The fixture
//!
//! MC-027's page-in-margins geometry (`common::PAGE_*`), with one difference:
//! the art ramps up from `min_line_spread` at its outermost column instead of
//! from 1, so there is a single, exact "first column of art" to pin. The
//! pieces are there for the same reasons MC-027 gives:
//!
//! * the page margins are the **speckled** gutter (`common::SPECKLE_*`), flat
//!   in the spread sense and *not* uniform in the `max - min` sense, so
//!   `trim_uniform` leaves them for the column locator rather than trimming
//!   them first;
//! * the art climbs by one amplitude step per column, so there is no
//!   adjacent-column step at the edge for `content_box` to peel a strip at;
//! * a two-column **panel seam** deep inside the page is a strong column line,
//!   so MC-025's `textured_box` stands down on the column axis and stage 3c is
//!   the stage that answers, exactly as on a real screenshot.
//!
//! The art is a horizontal-edge texture of constant phase: `FADE_TONE + a` on
//! even rows and `FADE_TONE - a` on odd ones. Over an even number of rows a
//! column's mean is exactly `FADE_TONE` and its spread exactly `a`. The
//! central band of the full 240-row rect is 144 rows starting at row 48 -
//! both even - which is what makes "exactly 8.0" exact; a premise test below
//! measures it rather than trusting the arithmetic.

mod common;

use common::{
    FADE_PEAK, FADE_TONE, PAGE_H, PAGE_MARGIN, PAGE_SEAM_AMPLITUDE, PAGE_SEAM_W, PAGE_SEAM_X,
    PAGE_W, SPECKLE_DEVIATION, SPECKLE_PERIOD, is_seam,
};
use cropper_core::edges::{col_profile, strong_lines};
use cropper_core::flat::central_band;
use cropper_core::{Luma, Rect, Tuning, detect};

// --- The fixture ------------------------------------------------------------

/// The art's first column: the first column right of the left page margin.
const ART_FIRST: u32 = PAGE_MARGIN;

/// The art's last column: the last column left of the right page margin.
const ART_LAST: u32 = PAGE_W - PAGE_MARGIN - 1;

/// The row whose pixel the "just below the rule" twin nudges, in both edge
/// columns. The fixture's middle row: inside the central band of any rect the
/// pipeline could hand stage 3c here. Even, so its art pixel is the `+a` one.
const NUDGED_ROW: u32 = PAGE_H / 2;

/// The art's outermost amplitude: `Tuning::default().min_line_spread`, read out
/// rather than written as 8, and required to be a whole number so that a
/// column of it has spread exactly equal to the threshold.
fn edge_amplitude() -> u32 {
    let spread = Tuning::default().min_line_spread;
    assert!(
        spread.fract() == 0.0 && spread >= 1.0 && spread < FADE_PEAK as f32,
        "the fixture's premise: min_line_spread ({spread}) is a whole amplitude \
         below the art's peak, so one column of art can sit exactly on it"
    );
    spread as u32
}

/// The art's amplitude at column `x`: 0 in the page margins, then
/// `edge_amplitude()` at [`ART_FIRST`] and [`ART_LAST`], climbing by one per
/// column towards the middle and capped at [`FADE_PEAK`].
fn amplitude(x: u32) -> u32 {
    if !(ART_FIRST..=ART_LAST).contains(&x) {
        return 0;
    }
    let from_edge = (x - ART_FIRST).min(ART_LAST - x);
    (edge_amplitude() + from_edge).min(FADE_PEAK)
}

/// One pixel of art at `amplitude`, `+a` on even rows and `-a` on odd ones.
fn art(amplitude: u32, y: u32) -> u8 {
    let a = u8::try_from(amplitude).expect("amplitudes here are at most 108");
    if y.is_multiple_of(2) {
        FADE_TONE + a
    } else {
        FADE_TONE - a
    }
}

/// One pixel of speckled page margin: `common`'s gutter rule, restated from
/// its public constants because the helper itself is private to `common`.
fn page_margin(x: u32, y: u32) -> u8 {
    let t = x + 3 * y;
    if !t.is_multiple_of(SPECKLE_PERIOD) {
        FADE_TONE
    } else if (t / SPECKLE_PERIOD).is_multiple_of(2) {
        FADE_TONE + SPECKLE_DEVIATION
    } else {
        FADE_TONE - SPECKLE_DEVIATION
    }
}

/// The page: margin, art ramping up from the rule, seam, art, margin.
///
/// With `nudged`, the art's two outermost columns each have their pixel at
/// [`NUDGED_ROW`] lowered by one grey level, which puts their spread one
/// pixel's worth below `min_line_spread`.
fn page(nudged: bool) -> Luma {
    let mut data = vec![0u8; (PAGE_W * PAGE_H) as usize];
    for y in 0..PAGE_H {
        for x in 0..PAGE_W {
            let a = amplitude(x);
            let mut px = if is_seam(x) {
                art(PAGE_SEAM_AMPLITUDE, y)
            } else if a == 0 {
                page_margin(x, y)
            } else {
                art(a, y)
            };
            if nudged && y == NUDGED_ROW && (x == ART_FIRST || x == ART_LAST) {
                px -= 1;
            }
            data[(y * PAGE_W + x) as usize] = px;
        }
    }
    Luma {
        width: PAGE_W,
        height: PAGE_H,
        data,
    }
}

fn whole(img: &Luma) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    }
}

/// The first and last column of `detect`'s rect, both inclusive.
fn columns(img: &Luma, t: &Tuning) -> (u32, u32) {
    let rect = detect(img, t)
        .expect("the page fixture is not a uniform image")
        .rect;
    (rect.x, rect.x + rect.w - 1)
}

/// Column `x`'s mean absolute deviation about its own mean over `rows`,
/// computed here from the definition as an exact rational in `f64` - not
/// through the crate, so a premise about the fixture does not lean on the
/// code under test.
fn spread(img: &Luma, x: u32, rows: Rect) -> f64 {
    let samples: Vec<u64> = (rows.y..rows.y + rows.h)
        .map(|y| u64::from(img.data[(y * img.width + x) as usize]))
        .collect();
    let n = samples.len() as u64;
    let total: u64 = samples.iter().sum();
    let deviations: u64 = samples.iter().map(|&s| (n * s).abs_diff(total)).sum();
    deviations as f64 / (n * n) as f64
}

// --- The fixture is what it claims to be ------------------------------------

/// The premises the three AC-6 tests stand on, measured rather than assumed:
/// the kept twin's outermost art columns sit **exactly** on `min_line_spread`,
/// the nudged twin's sit **just** below it, every other art column is above
/// it, every page-margin column is below it, and the only strong column line
/// is the seam - so stage 3c, and no earlier stage, decides the side edges.
#[test]
fn the_fixtures_outermost_art_column_sits_exactly_on_the_rule_and_its_twin_just_below_it() {
    let t = Tuning::default();
    let threshold = f64::from(t.min_line_spread);
    let kept = page(false);
    let nudged = page(true);
    let band = central_band(whole(&kept), &t);

    assert!(
        band.contains_row(NUDGED_ROW) && band.h.is_multiple_of(2) && band.y.is_multiple_of(2),
        "the central band ({band:?}) must hold the nudged row and start on an even \
         row with an even count, or the 'exactly on the rule' column is not exact"
    );
    for x in [ART_FIRST, ART_LAST] {
        assert_eq!(
            spread(&kept, x, band),
            threshold,
            "column {x} of the kept fixture must sit exactly on min_line_spread"
        );
        let below = spread(&nudged, x, band);
        assert!(
            below < threshold && below > threshold - 0.01,
            "column {x} of the nudged fixture must sit just below min_line_spread \
             ({threshold}); it measures {below}"
        );
    }
    for x in ART_FIRST + 1..ART_LAST {
        let s = spread(&kept, x, band);
        assert!(
            s > threshold,
            "art column {x} must be above the rule; measures {s}"
        );
    }
    for x in (0..ART_FIRST).chain(ART_LAST + 1..PAGE_W) {
        let s = spread(&kept, x, band);
        assert!(
            s < threshold,
            "page-margin column {x} must be below the rule; measures {s}"
        );
    }

    for img in [&kept, &nudged] {
        let lines = strong_lines(&col_profile(img, whole(img)), &t);
        assert!(
            !lines.is_empty()
                && lines.iter().all(|l| {
                    l.start + 1 >= PAGE_SEAM_X as usize
                        && l.end < (PAGE_SEAM_X + PAGE_SEAM_W) as usize
                }),
            "the seam must be the only strong column line, so `textured_box` stands \
             down and nothing peels a strip at the page edge. Lines: {lines:?}"
        );
        let found = detect(img, &t).expect("not uniform");
        assert!(
            found.removed.is_empty() && !found.ambiguous,
            "no edge strip is peeled or ambiguous on the page fixture: {found:?}"
        );
    }
}

/// Small extension so the premise above reads as a sentence.
trait ContainsRow {
    fn contains_row(&self, y: u32) -> bool;
}

impl ContainsRow for Rect {
    fn contains_row(&self, y: u32) -> bool {
        (self.y..self.y + self.h).contains(&y)
    }
}

// --- AC-6 -------------------------------------------------------------------

/// AC-6, first sentence and the "kept" half of the second: at
/// `Tuning::default()` the crop's left and right columns are exactly the art's
/// first and last columns - the outermost art column, sitting exactly on
/// `min_line_spread`, is kept, and not one column of page margin comes with it.
///
/// Before MC-049 this is `(37, 262)`: the locator finds `(40, 259)` and
/// `margin_px = 3` adds three columns of flat page back on each side.
#[test]
fn at_the_default_tuning_the_crop_edges_are_the_arts_own_first_and_last_columns() {
    let t = Tuning::default();
    assert_eq!(
        columns(&page(false), &t),
        (ART_FIRST, ART_LAST),
        "the crop must start on the art's first column ({ART_FIRST}) and end on its \
         last ({ART_LAST}), with no page background on either side; margin_px is {}",
        t.margin_px
    );
}

/// AC-6, the "trimmed" half: the same page with its outermost art columns one
/// pixel's worth below `min_line_spread` - on the page-margin side of the rule
/// the locator already applies - loses exactly those two columns, and the crop
/// edge lands on the next column in.
///
/// Before MC-049 this is `(38, 261)`.
#[test]
fn an_outermost_column_just_below_the_rule_is_trimmed_as_page_margin() {
    let t = Tuning::default();
    assert_eq!(
        columns(&page(true), &t),
        (ART_FIRST + 1, ART_LAST - 1),
        "columns {ART_FIRST} and {ART_LAST} sit just below min_line_spread ({}), so \
         they are page margin by the locator's own rule and the crop must start at \
         {} and end at {}; margin_px is {}",
        t.min_line_spread,
        ART_FIRST + 1,
        ART_LAST - 1,
        t.margin_px
    );
}

/// AC-6's boundary is the *settled* constant, read from the `Tuning`: the kept
/// fixture, whose edge columns sit exactly on `min_line_spread`, loses them the
/// moment the threshold rises by one `f32` step. That is `>=` at the one
/// comparison site, and it is `min_line_spread` and nothing else deciding it.
#[test]
fn the_side_boundary_is_min_line_spread_read_from_the_tuning() {
    let t = Tuning::default();
    let raised = Tuning {
        min_line_spread: t.min_line_spread.next_up(),
        ..Tuning::default()
    };
    assert_eq!(
        columns(&page(false), &raised),
        (ART_FIRST + 1, ART_LAST - 1),
        "at min_line_spread {} the edge columns (spread exactly {}) are below the \
         rule and must be trimmed; margin_px is {}",
        raised.min_line_spread,
        t.min_line_spread,
        raised.margin_px
    );
}

/// AC-6's control: the first test's assertion is tight to one column. At
/// `margin_px` = 1 the same fixture comes back one column wider on **each**
/// side, so an implementation that left a one-column margin would fail the
/// first test by exactly that.
#[test]
fn at_a_one_pixel_margin_the_crop_is_one_column_wide_of_the_art_on_each_side() {
    let one = Tuning {
        margin_px: 1,
        ..Tuning::default()
    };
    let got = columns(&page(false), &one);
    assert_eq!(
        got,
        (ART_FIRST - 1, ART_LAST + 1),
        "at margin_px 1 the crop must carry exactly one column of page margin on \
         each side"
    );
    assert_ne!(
        got,
        (ART_FIRST, ART_LAST),
        "and so it must fail the default-tuning assertion"
    );
}
