//! MC-049 AC-6: on a synthetic page, the crop's left and right columns are the
//! art's own first and last columns - no page background on either side.
//!
//! The user, 2026-09-23: *"The cropping from the side isnt tight enough and I
//! still see the page on the right and left side."* The column locator
//! already stops on the boundary between flat page margin and art; what the
//! user saw was `margin::expand` adding `Tuning::margin_px` columns of flat
//! page back on each side afterwards. MC-049 rules that margin to 0, and this
//! file is the required `unit` gate's half of that ruling (the corpus half is
//! `crates/engine/tests/corpus_sides.rs`, `integration` only).
//!
//! # Which rule decides the boundary, read out of the source and not invented
//!
//! On a reader page the side boundary is decided by stage 3c,
//! [`cropper_core::flat::page_column`]. Since MC-055 (2026-09-28,
//! `architecture.md` stage 3, "Then the page column") the widest run of
//! columns whose spread reaches [`Tuning::min_line_spread`] only *finds* the
//! page. The page column is that run widened outward one column at a time,
//! stopping at the first column that is page margin, judged in this order:
//!
//! 1. its median over the central band is within `uniform_tolerance` of the
//!    page background tone and not of the column just inside it: margin;
//! 2. otherwise, it is **not page background over the band** - under 95 % of
//!    its pixels within `uniform_tolerance` of its own (upper) median,
//!    `flat.rs`'s `PAGE_BACKGROUND_PERCENT`, which is MC-049's own predicate:
//!    page;
//! 3. it is page background over the band: margin, unless it carries on the
//!    tone of the column just inside it *and* is not page background over the
//!    viewport's rows.
//!
//! This file's boundary is **step 2**, and the constants that decide it are
//! the settled 95 % and `uniform_tolerance` (10). Nothing here introduces a
//! new rule or a new number. `PAGE_BACKGROUND_PERCENT` is private to the
//! crate, so the fixture is derived from its documented value
//! ([`PAGE_BACKGROUND_PERCENT_DOCUMENTED`]) and a premise test measures the
//! share it produces rather than trusting the arithmetic.
//!
//! So the art's outermost column is a **low-texture** column - below
//! `min_line_spread`, so the run does not reach it and the widening must
//! decide it - that is flat apart from a few pixels just outside
//! `uniform_tolerance` of its median. On the kept fixture there is exactly one
//! such pixel too many for it to be page background (share 136/144, just
//! under 95 %): step 2 keeps it, and the crop edge is that column. Its twin
//! restores one of those pixels (137/144, just at or over 95 %): step 2 no
//! longer speaks, and step 3 trims it, because its tone does not carry on the
//! textured art's. Neither column's tone is the margin's, so step 1 decides
//! neither. The column outside it, the speckled page margin, is stopped by
//! step 1 in both.
//!
//! # The fixture
//!
//! MC-027's page-in-margins geometry (`common::PAGE_*`):
//!
//! * the page margins are the **speckled** gutter (`common::SPECKLE_*`), flat
//!   in the spread sense and *not* uniform in the `max - min` sense, so
//!   `trim_uniform` leaves them for the column locator rather than trimming
//!   them first. Their median, and so the page background tone, is
//!   `FADE_TONE`;
//! * the art's outermost column on each side ([`ART_FIRST`], [`ART_LAST`]) is
//!   the low-texture column above, at [`edge_tone`];
//! * inside it the textured art ramps up from `min_line_spread` by one
//!   amplitude step per column, so the widest run is exactly the textured
//!   columns and there is no adjacent-column step for `content_box` to peel a
//!   strip at;
//! * a two-column **panel seam** deep inside the page is a strong column line,
//!   so MC-025's `textured_box` stands down on the column axis and stage 3c is
//!   the stage that answers, exactly as on a real screenshot.
//!
//! The textured art is `FADE_TONE + a` on even rows and `FADE_TONE - a` on odd
//! ones, so its upper median over the band is `FADE_TONE + a`.

mod common;

use common::{
    FADE_PEAK, FADE_TONE, PAGE_H, PAGE_MARGIN, PAGE_SEAM_AMPLITUDE, PAGE_SEAM_W, PAGE_SEAM_X,
    PAGE_W, SPECKLE_DEVIATION, SPECKLE_PERIOD, is_seam,
};
use cropper_core::edges::{col_profile, strong_lines};
use cropper_core::flat::central_band;
use cropper_core::{Luma, Rect, Tuning, detect};

// --- The fixture ------------------------------------------------------------

/// MC-049's page-background share, as a whole percentage: `flat.rs`'s
/// `PAGE_BACKGROUND_PERCENT`, which is crate-private, copied from its
/// documentation and from `architecture.md` stage 3 ("under 0.95 of its
/// pixels within `uniform_tolerance` of its median"). Settled; the premise
/// test measures the fixture against it.
const PAGE_BACKGROUND_PERCENT_DOCUMENTED: u32 = 95;

/// The art's first column, its low-texture outermost one: the first column
/// right of the left page margin.
const ART_FIRST: u32 = PAGE_MARGIN;

/// The art's last column, its low-texture outermost one: the last column left
/// of the right page margin.
const ART_LAST: u32 = PAGE_W - PAGE_MARGIN - 1;

/// The whole fixture as a rect.
fn whole_page() -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: PAGE_W,
        h: PAGE_H,
    }
}

/// The rows stage 3c measures a column over: the central band of the whole
/// fixture, which is the rect `page_column` is handed here (the premise test
/// checks that nothing earlier moves it).
fn band() -> Rect {
    central_band(whole_page(), &Tuning::default())
}

/// How many of the outermost column's band pixels sit off its median on the
/// kept fixture: the fewest that put its share **under**
/// [`PAGE_BACKGROUND_PERCENT_DOCUMENTED`]. The twin has one fewer, which is
/// the most that leaves the share **at or over** it. On the 144-row band:
/// 8 (136/144 = 0.944) and 7 (137/144 = 0.951).
fn off_median_kept() -> u32 {
    let n = band().h;
    (0..=n)
        .find(|k| (n - k) * 100 < n * PAGE_BACKGROUND_PERCENT_DOCUMENTED)
        .expect("some count is under the share")
}

/// The rows of the outermost column that sit off its median, spread evenly
/// through the band: `count` of them.
fn off_median_rows(count: u32) -> Vec<u32> {
    let band = band();
    let step = band.h / off_median_kept();
    (0..count).map(|i| band.y + 1 + i * step).collect()
}

/// The outermost column's tone. More than `uniform_tolerance` + 1 from the
/// page background tone (`FADE_TONE`), so step 1 never calls it margin, and
/// further again from the textured art's median beside it
/// (`FADE_TONE + min_line_spread`), so step 3 never calls it the page's tone
/// carried on. Both hold at `uniform_tolerance` + 1 too, which is what the
/// tolerance test below needs.
fn edge_tone() -> u8 {
    FADE_TONE - (Tuning::default().uniform_tolerance + 3)
}

/// The value of the outermost column's off-median pixels: one grey level
/// outside `uniform_tolerance` of [`edge_tone`].
fn off_median_tone() -> u8 {
    edge_tone() + Tuning::default().uniform_tolerance + 1
}

/// The textured art's outermost amplitude: `Tuning::default().min_line_spread`,
/// read out rather than written as 8, and required to be a whole number so
/// that the run of textured columns is exactly the textured art.
fn edge_amplitude() -> u32 {
    let spread = Tuning::default().min_line_spread;
    assert!(
        spread.fract() == 0.0 && spread >= 1.0 && spread < FADE_PEAK as f32,
        "the fixture's premise: min_line_spread ({spread}) is a whole amplitude \
         below the art's peak"
    );
    spread as u32
}

/// The textured art's amplitude at column `x`: 0 outside it, then
/// `edge_amplitude()` next to [`ART_FIRST`] and [`ART_LAST`], climbing by one
/// per column towards the middle and capped at [`FADE_PEAK`].
fn amplitude(x: u32) -> u32 {
    if !(ART_FIRST + 1..ART_LAST).contains(&x) {
        return 0;
    }
    let from_edge = (x - ART_FIRST - 1).min(ART_LAST - 1 - x);
    (edge_amplitude() + from_edge).min(FADE_PEAK)
}

/// One pixel of textured art at `amplitude`, `+a` on even rows and `-a` on odd
/// ones.
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

/// The page: margin, low-texture outermost column, textured art ramping up
/// from the spread rule, seam, art, outermost column, margin.
///
/// With `trimmed`, each outermost column has one off-median pixel fewer,
/// which puts its share over the band at or over 95 %: page background.
fn page(trimmed: bool) -> Luma {
    let count = off_median_kept() - u32::from(trimmed);
    let off = off_median_rows(count);
    let mut data = vec![0u8; (PAGE_W * PAGE_H) as usize];
    for y in 0..PAGE_H {
        for x in 0..PAGE_W {
            let a = amplitude(x);
            let px = if is_seam(x) {
                art(PAGE_SEAM_AMPLITUDE, y)
            } else if x == ART_FIRST || x == ART_LAST {
                if off.contains(&y) {
                    off_median_tone()
                } else {
                    edge_tone()
                }
            } else if a == 0 {
                page_margin(x, y)
            } else {
                art(a, y)
            };
            data[(y * PAGE_W + x) as usize] = px;
        }
    }
    Luma {
        width: PAGE_W,
        height: PAGE_H,
        data,
    }
}

/// The first and last column of `detect`'s rect, both inclusive.
fn columns(img: &Luma, t: &Tuning) -> (u32, u32) {
    let rect = detect(img, t)
        .expect("the page fixture is not a uniform image")
        .rect;
    (rect.x, rect.x + rect.w - 1)
}

/// Column `x`'s pixels over `rows`.
fn column(img: &Luma, x: u32, rows: Rect) -> Vec<u8> {
    (rows.y..rows.y + rows.h)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

/// The upper median, `sorted[n / 2]`: the median `flat.rs` documents.
fn upper_median(mut values: Vec<u8>) -> u8 {
    values.sort_unstable();
    values[values.len() / 2]
}

/// Column `x`'s upper median over `rows`, and the share of its pixels there
/// within `tolerance` of it. Computed here from MC-049's definition, not
/// through the crate, so a premise about the fixture does not lean on the
/// code under test.
fn median_and_share(img: &Luma, x: u32, rows: Rect, tolerance: u8) -> (u8, f64) {
    let values = column(img, x, rows);
    let median = upper_median(values.clone());
    let near = values
        .iter()
        .filter(|&&v| v.abs_diff(median) <= tolerance)
        .count();
    (median, near as f64 / values.len() as f64)
}

/// Column `x`'s mean absolute deviation about its own mean over `rows`, as an
/// exact rational in `f64` - the spread `min_line_spread` is compared with.
fn spread(img: &Luma, x: u32, rows: Rect) -> f64 {
    let samples: Vec<u64> = column(img, x, rows).into_iter().map(u64::from).collect();
    let n = samples.len() as u64;
    let total: u64 = samples.iter().sum();
    let deviations: u64 = samples.iter().map(|&s| (n * s).abs_diff(total)).sum();
    deviations as f64 / (n * n) as f64
}

// --- The fixture is what it claims to be ------------------------------------

/// The premises the AC-6 tests stand on, measured rather than assumed:
///
/// * the kept fixture's outermost columns are **just under** 95 % page
///   background over the band, and the trimmed twin's **at or just over** it,
///   one pixel apart;
/// * neither outermost column is within `uniform_tolerance` of the page
///   background tone or of the textured art's median beside it, so steps 1
///   and 3 turn on nothing else;
/// * the outermost columns are below `min_line_spread` and every textured
///   column is at or above it, so the widest run is exactly the textured art
///   and the widening decides the outermost columns;
/// * every page-margin column has the page background tone for its median,
///   and so does every row outside the textured art, which is where the tone
///   is read;
/// * the seam is the only strong column line and nothing is peeled, so stage
///   3c, and no earlier stage, decides the side edges.
#[test]
fn the_fixtures_outermost_art_column_sits_one_pixel_either_side_of_the_page_background_share() {
    let t = Tuning::default();
    let tol = t.uniform_tolerance;
    let rule = f64::from(PAGE_BACKGROUND_PERCENT_DOCUMENTED) / 100.0;
    let threshold = f64::from(t.min_line_spread);
    let kept = page(false);
    let trimmed = page(true);
    let band = band();
    let textured_median = FADE_TONE + edge_amplitude() as u8;

    for x in [ART_FIRST, ART_LAST] {
        let (median, share) = median_and_share(&kept, x, band, tol);
        let (twin_median, twin_share) = median_and_share(&trimmed, x, band, tol);
        let pixel = 1.0 / f64::from(band.h);
        assert!(
            share < rule && share + pixel >= rule,
            "column {x} of the kept fixture must be one pixel under the page-background \
             share {rule}; it measures {share}"
        );
        assert!(
            twin_share >= rule && twin_share - pixel < rule,
            "column {x} of the trimmed twin must be at or over the share {rule}, and one \
             pixel from under it; it measures {twin_share}"
        );
        for (m, which) in [(median, "kept"), (twin_median, "twin")] {
            assert_eq!(m, edge_tone(), "column {x} ({which}) has the edge tone");
            for tolerance in [tol, tol + 1] {
                assert!(
                    m.abs_diff(FADE_TONE) > tolerance && m.abs_diff(textured_median) > tolerance,
                    "column {x} ({which}): median {m} must be more than {tolerance} from the \
                     page tone {FADE_TONE} and from the textured art's {textured_median}"
                );
            }
        }
        for img in [&kept, &trimmed] {
            let s = spread(img, x, band);
            assert!(
                s < threshold,
                "column {x} must be below min_line_spread; measures {s}"
            );
        }
    }
    for x in ART_FIRST + 1..ART_LAST {
        let s = spread(&kept, x, band);
        assert!(
            s >= threshold,
            "textured art column {x} must be at or above min_line_spread; measures {s}"
        );
    }
    for x in (0..ART_FIRST).chain(ART_LAST + 1..PAGE_W) {
        let (median, _) = median_and_share(&kept, x, band, tol);
        let s = spread(&kept, x, band);
        assert!(
            median == FADE_TONE && s < threshold,
            "page-margin column {x} must have the page tone and be below the spread \
             rule; median {median}, spread {s}"
        );
    }
    for img in [&kept, &trimmed] {
        for y in 0..PAGE_H {
            let beside: Vec<u8> = (0..=ART_FIRST)
                .chain(ART_LAST..PAGE_W)
                .map(|x| img.data[(y * PAGE_W + x) as usize])
                .collect();
            assert_eq!(
                upper_median(beside),
                FADE_TONE,
                "row {y}'s median beside the textured art is the page background tone"
            );
        }
    }

    for img in [&kept, &trimmed] {
        let lines = strong_lines(&col_profile(img, whole_page()), &t);
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
        assert_eq!(
            (found.rect.y, found.rect.h),
            (0, PAGE_H),
            "nothing moves the rows, so the band stage 3c reads is the one measured here"
        );
    }
}

// --- AC-6 -------------------------------------------------------------------

/// AC-6, first sentence and the "kept" half of the second: at
/// `Tuning::default()` the crop's left and right columns are exactly the art's
/// first and last columns. The outermost art column, just under the
/// page-background share, is kept by step 2 of the rule, and not one column of
/// page margin comes with it.
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

/// AC-6, the "trimmed" half: the same page with one off-median pixel restored
/// in each outermost column, which puts its share over the band at or over
/// 95 % - on the page-margin side of the rule the locator already applies -
/// loses exactly those two columns, and the crop edge lands on the next
/// column in.
#[test]
fn an_outermost_column_at_the_page_background_share_is_trimmed_as_page_margin() {
    let t = Tuning::default();
    assert_eq!(
        columns(&page(true), &t),
        (ART_FIRST + 1, ART_LAST - 1),
        "columns {ART_FIRST} and {ART_LAST} are page background over the band by \
         MC-049's predicate ({PAGE_BACKGROUND_PERCENT_DOCUMENTED} % within \
         uniform_tolerance {} of their median), and do not carry the art's tone on, so \
         they are page margin and the crop must start at {} and end at {}; margin_px is {}",
        t.uniform_tolerance,
        ART_FIRST + 1,
        ART_LAST - 1,
        t.margin_px
    );
}

/// AC-6's boundary reads the *settled* tolerance from the `Tuning`: the kept
/// fixture's off-median pixels sit one grey level outside `uniform_tolerance`
/// of the column's median, so at `uniform_tolerance + 1` they are within it,
/// the column is page background over the band, and it is trimmed. A
/// predicate with the tolerance written in as 10 keeps it.
#[test]
fn the_side_boundary_reads_uniform_tolerance_from_the_tuning() {
    let t = Tuning::default();
    let wider = Tuning {
        uniform_tolerance: t.uniform_tolerance + 1,
        ..Tuning::default()
    };
    assert_eq!(
        columns(&page(false), &wider),
        (ART_FIRST + 1, ART_LAST - 1),
        "at uniform_tolerance {} the edge columns' off-median pixels ({} from the \
         median) are within tolerance, the columns are page background and must be \
         trimmed; margin_px is {}",
        wider.uniform_tolerance,
        off_median_tone() - edge_tone(),
        wider.margin_px
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
