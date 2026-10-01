//! MC-065 AC-4: a page whose outermost columns are a flat white strip of its
//! own paper, beside a **single-value site background**, keeps them - seen
//! through [`detect`], on generated images. The corpus half is
//! `crates/engine/tests/corpus_right_edge.rs` (AC-2); this file is what the
//! required `unit` gate sees of the story.
//!
//! `f09`'s shape (a dark, row-varying art edge beside a site of similar
//! brightness), MC-065's former second case, moved to MC-067 with `f09`
//! (MC-065 `## Amendments`, 2026-10-01). Its cause test, control and premise
//! were deleted here; their record is in MC-065's `## Handoff`, and MC-067's
//! RED rebuilt them in `page_column_dark_fringe.rs`.
//!
//! # The bug, and where it lives
//!
//! A fresh corpus screenshot loses its right page edge (MC-063): `f20`
//! (`2025-08-07 01_13_55.png`) by 6 columns. Per column, luma BT.601 over the
//! mark's rows (re-measured in MC-065's RED; mean / mean absolute deviation
//! about the mean):
//!
//! ```text
//! f20  1515 250.0 / 9.2 | 1516..1521 253.1..253.7 / 2.4..3.6 | 1522 25.0 / 0.0 (one value)
//! ```
//!
//! Read out of the stages (MC-065's RED, `## Handoff`): every stage up to
//! `textured_box` keeps the whole width, and **stage 3c, `flat::page_column`,
//! cuts the edge**. Its widest textured run ends at the last column whose
//! spread over the central band reaches `min_line_spread` (8.0), 1515, and the
//! widening outward (`extend_to_the_margin`, `Margin::belongs`) then stops on
//! the very next column, in branch 3. Column 1516 has band median 255, the
//! page edge's own tone, so it carries the page on; but it is page background
//! over the band (share 0.980 within `uniform_tolerance` of its median) *and*
//! over the viewport's rows (0.979), because the page's white paper is one
//! flat shade. Branch 3 calls a column flat over the viewport margin, so the
//! white strip 1516..1521 is cut although the site's background (25) only
//! starts at 1522.
//!
//! The site's background is a column of **one exact value**, and the page's
//! edge is not. Luma is the only signal used, here and in the fix: colour is
//! out of scope (MC-065 `## Out of scope`, `architecture.md`).
//!
//! # The fixtures: invented here, the cause turned on and off
//!
//! One geometry, [`W`] x [`H`]: a band of textured **browser chrome** across
//! the top [`CHROME_H`] rows and the whole width, then the **site** - one
//! exact value on every row below the chrome - on both sides of a **page**
//! column [`PAGE_FIRST`] ..= [`PAGE_LAST`]. The chrome is what keeps a
//! single-value site column from being uniform over the whole image, so stage
//! 1 leaves it for stage 3c, exactly as the browser chrome and taskbar do on a
//! real screenshot. Each site is wider than `chrome_max_extent` (0.30) of the
//! width, so `content_box` cannot peel it as a strip.
//!
//! The page's left edge is plain textured art against the site, the case that
//! works today; only the **right** edge carries the cause, as on the
//! screenshot.
//!
//! - **`f20`'s shape** ([`white_page`]): white textured art (median 255), then
//!   [`STRIP_W`] columns of flat white page, then a site of [`F20_SITE`].
//!   - *cause on* ([`Strip::Background`]): the strip is 255 with one pixel in
//!     [`STRIP_PERIOD`] darker - page background by MC-049's predicate (share
//!     about 0.98) and flat by `min_line_spread` (spread about 5). Before
//!     MC-065's fix the crop ended before the strip.
//!   - *cause off* ([`Strip::NotBackground`]): the same strip with one pixel in
//!     [`CONTROL_PERIOD`] [`CONTROL_OFFSET`] levels down - still flat by
//!     `min_line_spread` (spread about 1.3) but **not** page background (share
//!     about 0.94). Before the fix the crop already kept the strip, so what cut
//!     the strip was its being page background, and nothing else about it.
//!
//! The premise test measures every statistic above on the fixtures, from the
//! definitions, without the crate, so a fixture that drifts goes red rather
//! than making a case vacuous.

use cropper_core::flat::central_band;
use cropper_core::{Luma, Rect, Tuning, detect};

// --- Settled constants, read out --------------------------------------------

/// MC-049's page-background share: a column is page background when at least
/// this share of its pixels lie within `uniform_tolerance` of its upper
/// median. `flat.rs`'s `PAGE_BACKGROUND_PERCENT` is crate-private, so it is
/// copied from its documentation, as `page_edges.rs` does.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

// --- The geometry, invented here --------------------------------------------

/// The fixture's width.
const W: u32 = 400;
/// The fixture's height.
const H: u32 = 240;
/// Rows of textured browser chrome across the top.
const CHROME_H: u32 = 24;
/// The page's first column. The left site is `0 .. PAGE_FIRST`: 130 of 400
/// columns, 0.325 of the width, above `chrome_max_extent`.
const PAGE_FIRST: u32 = 130;
/// The page's last column, inclusive; the right site is `PAGE_LAST + 1 .. W`,
/// also 130 columns.
const PAGE_LAST: u32 = 269;

/// `f20`'s flat white strip: its width (the 6 columns `f20` loses).
const STRIP_W: u32 = 6;
/// `f20`'s site value (`2025-08-07 01_13_55.png` column 1522).
const F20_SITE: u8 = 25;
/// The cause-on strip: one pixel in this many is [`STRIP_DARK`].
const STRIP_PERIOD: u32 = 50;
/// The cause-on strip's rare dark pixel: the line art reaching the paper's
/// edge (`f20`'s 1516..1521 reach 1 to 9 on a few rows).
const STRIP_DARK: u8 = 120;
/// The control strip: one pixel in this many is [`CONTROL_OFFSET`] below
/// white, so its share drops under [`PAGE_BACKGROUND_SHARE`] while its spread
/// stays far below `min_line_spread`.
const CONTROL_PERIOD: u32 = 16;
/// How far below 255 the control strip's off pixels are: one level outside
/// `uniform_tolerance` (10).
const CONTROL_OFFSET: u8 = 11;
/// The white art's line-art value, on roughly three pixels in ten.
const INK: u8 = 120;

// --- The fixtures -----------------------------------------------------------

/// A well-mixed hash of a pixel position, so texture has no column or row
/// structure for `content_box` to read as a band.
fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

/// One pixel of browser chrome: textured grey, 60 ..= 200.
fn chrome(x: u32, y: u32) -> u8 {
    60 + (hash(x, y) % 141) as u8
}

/// `f20`'s strip, cause on or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strip {
    /// Flat white page, page background by MC-049's predicate: the cause.
    Background,
    /// Flat white page, not page background: the control.
    NotBackground,
}

/// The first column of `f20`'s strip; the strip runs to [`PAGE_LAST`].
const STRIP_FIRST: u32 = PAGE_LAST + 1 - STRIP_W;

/// `f20`'s shape: white textured art, a flat white strip at its right edge,
/// and a site of one value.
fn white_page(strip: Strip) -> Luma {
    render(|x, y| {
        if !(PAGE_FIRST..=PAGE_LAST).contains(&x) {
            F20_SITE
        } else if x >= STRIP_FIRST {
            // Phase per column, so the off rows differ between strip columns.
            let k = y + 7 * x;
            match strip {
                Strip::Background if k.is_multiple_of(STRIP_PERIOD) => STRIP_DARK,
                Strip::NotBackground if k.is_multiple_of(CONTROL_PERIOD) => 255 - CONTROL_OFFSET,
                _ => 255,
            }
        } else if hash(x, y) % 10 < 3 {
            INK
        } else {
            255
        }
    })
}

/// The fixture: chrome over the top [`CHROME_H`] rows, `body` below.
fn render(body: impl Fn(u32, u32) -> u8) -> Luma {
    let mut data = Vec::with_capacity((W * H) as usize);
    for y in 0..H {
        for x in 0..W {
            data.push(if y < CHROME_H {
                chrome(x, y)
            } else {
                body(x, y)
            });
        }
    }
    Luma {
        width: W,
        height: H,
        data,
    }
}

// --- Measuring, test-side ---------------------------------------------------

/// The central band of the whole fixture: the rows stage 3c measures a column
/// over, when nothing earlier moves the rect's rows (the premise test checks
/// the band holds no chrome row).
fn band() -> Rect {
    central_band(
        Rect {
            x: 0,
            y: 0,
            w: W,
            h: H,
        },
        &Tuning::default(),
    )
}

/// Column `x`'s pixels over the band.
fn column(img: &Luma, x: u32) -> Vec<u8> {
    let band = band();
    (band.y..band.y + band.h)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

/// The upper median, `sorted[n / 2]`.
fn median(values: &[u8]) -> u8 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

/// The share of `values` within `tolerance` of their upper median: MC-049's
/// predicate, from its definition.
fn share(values: &[u8], tolerance: u8) -> f64 {
    let m = median(values);
    values
        .iter()
        .filter(|&&v| v.abs_diff(m) <= tolerance)
        .count() as f64
        / values.len() as f64
}

/// Mean absolute deviation about the mean: the spread `min_line_spread` is
/// compared with.
fn spread(values: &[u8]) -> f64 {
    let n = values.len() as f64;
    let mean = values.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
    values
        .iter()
        .map(|&v| (f64::from(v) - mean).abs())
        .sum::<f64>()
        / n
}

/// Both margins the corpus suites test: 0 (the default) and 3.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning::default(),
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
    ]
}

/// `detect`'s first and last column, both inclusive.
fn crop_columns(img: &Luma, t: &Tuning) -> (u32, u32) {
    let rect = detect(img, t)
        .expect("the fixture is not one flat colour")
        .rect;
    (rect.x, rect.x + rect.w - 1)
}

/// The columns the crop must have at `t`: exactly the page, plus the margin.
fn page_columns(t: &Tuning) -> (u32, u32) {
    (PAGE_FIRST - t.margin_px, PAGE_LAST + t.margin_px)
}

/// Every margin at which `img`'s crop is not exactly [`page_columns`], as a
/// line naming what it got.
fn wrong_columns(img: &Luma) -> Vec<String> {
    both_margins()
        .iter()
        .filter_map(|t| {
            let (got, want) = (crop_columns(img, t), page_columns(t));
            (got != want).then(|| {
                format!(
                    "margin_px {}: crop columns {}..={}, the page is {}..={} (right edge \
                     short by {})",
                    t.margin_px,
                    got.0,
                    got.1,
                    want.0,
                    want.1,
                    i64::from(want.1) - i64::from(got.1)
                )
            })
        })
        .collect()
}

// --- The fixtures are what they claim to be ---------------------------------

/// The premises every case below stands on, measured on the fixtures from the
/// definitions and not through the crate.
#[test]
fn the_fixtures_edge_columns_and_sites_measure_as_the_causes_claim() {
    let t = Tuning::default();
    let tol = t.uniform_tolerance;
    let rule = f64::from(t.min_line_spread);
    let band = band();
    assert!(
        band.y >= CHROME_H,
        "the central band {band:?} must hold no chrome row (< {CHROME_H})"
    );

    // The site: one exact value on every row below the chrome, with the
    // cause on and off.
    for img in [
        white_page(Strip::Background),
        white_page(Strip::NotBackground),
    ] {
        for x in [0, PAGE_FIRST - 1, PAGE_LAST + 1, W - 1] {
            let values: Vec<u8> = (CHROME_H..H)
                .map(|y| img.data[(y * W + x) as usize])
                .collect();
            assert!(
                values.iter().all(|&v| v == F20_SITE),
                "site column {x} must be the single value {F20_SITE} below the chrome"
            );
        }
    }

    // f20's strip: flat by min_line_spread either way; page background only
    // with the cause on. Its median carries the page's white on.
    let on = white_page(Strip::Background);
    let off = white_page(Strip::NotBackground);
    for x in STRIP_FIRST..=PAGE_LAST {
        let (a, b) = (column(&on, x), column(&off, x));
        assert!(
            spread(&a) < rule && spread(&b) < rule,
            "strip column {x}: spread {:.2} (cause on) and {:.2} (control) must be below \
             min_line_spread {rule}",
            spread(&a),
            spread(&b)
        );
        assert!(
            share(&a, tol) >= PAGE_BACKGROUND_SHARE,
            "strip column {x} (cause on) must be page background: share {:.3}",
            share(&a, tol)
        );
        assert!(
            share(&b, tol) < PAGE_BACKGROUND_SHARE,
            "strip column {x} (control) must not be page background: share {:.3}",
            share(&b, tol)
        );
        assert_eq!(
            (median(&a), median(&b)),
            (255, 255),
            "strip column {x}'s median"
        );
    }
    let core_edge = column(&on, STRIP_FIRST - 1);
    assert!(
        spread(&core_edge) >= rule && median(&core_edge) == 255,
        "the white art's last column is textured (spread {:.2}) with median 255 ({}), so \
         the widest run ends there and the strip carries its tone on",
        spread(&core_edge),
        median(&core_edge)
    );
}

// --- f20's shape ------------------------------------------------------------

/// AC-4, `f20`'s cause: a flat white strip of the page's own paper at the
/// art's edge, beside a single-value site, is page, not margin. Fails today:
/// the crop ends [`STRIP_W`] columns short.
#[test]
fn a_flat_white_strip_of_page_beside_a_single_value_site_is_kept_at_both_margins() {
    let wrong = wrong_columns(&white_page(Strip::Background));
    assert!(
        wrong.is_empty(),
        "MC-065 AC-4 (f20's shape): the page's flat white paper at its right edge \
         ({STRIP_W} columns, {STRIP_FIRST}..={PAGE_LAST}) is page, and the site's \
         background ({F20_SITE}, one value) starts at {}; the crop must end on the \
         page's last column:\n{}",
        PAGE_LAST + 1,
        wrong.join("\n")
    );
}

/// AC-4's control for `f20`'s cause: the same strip, still flat by
/// `min_line_spread` but not page background by MC-049's predicate, is kept
/// today. Passes on arrival, and must keep passing: with the cause off the
/// crop is already right, so the cause is the strip being page background.
#[test]
fn control_the_same_strip_when_not_page_background_is_kept_at_both_margins() {
    let wrong = wrong_columns(&white_page(Strip::NotBackground));
    assert!(
        wrong.is_empty(),
        "MC-065 AC-4's control (f20's shape): a strip that is not page background must \
         be kept:\n{}",
        wrong.join("\n")
    );
}
