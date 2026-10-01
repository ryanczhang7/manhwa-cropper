//! MC-067 AC-4: a page whose art ends in a dark, row-varying **fringe** - one
//! column near the site's value on about half its rows, then one on about
//! 90% - beside a **single-value site background of similar brightness**,
//! keeps the fringe. Seen through [`detect`], on generated images. The corpus
//! half is `crates/engine/tests/corpus_dark_fringe.rs` (AC-1, AC-2); this file
//! is what the required `unit` gate sees of the story.
//!
//! # The bug, and where it lives
//!
//! `f09` (`2025-12-08 17_22_50.png`) loses its right page edge by 2 columns:
//! the app crops `1006..1537` and the mark, which the user ruled stands, is
//! `1006..1539` (end exclusive). Per column, luma BT.601 over the band
//! (MC-065 `## Handoff`, re-read in MC-067's RED):
//!
//! ```text
//! f09  1536 median 60 spread 16.3 | 1537 median 15 spread 5.4 share 0.882
//!      1538 median 9 spread 0.8 share 1.000 (0.862 over the viewport rows)
//!      1539 one value, 11
//! ```
//!
//! Stage 3c, `flat::page_column`, cuts the edge. Its widest textured run ends
//! on 1536, the last column whose band spread reaches `min_line_spread`, and
//! the widening outward (`extend_to_the_margin`, `Margin::belongs`) stops on
//! the very next column in **branch 1**: 1537's band median (15) is within
//! `uniform_tolerance` of the page background tone (the site's 11) and not of
//! the page edge's (60), so it is called margin. It is **not** page
//! background over the band (share 0.882), and branch 1 never asks.
//!
//! `Screenshot (3538).png` has the same two-column fringe on both sides, by
//! luma and by colour (MC-067 `## Notes`), and the user ruled on 2026-10-01
//! that it may grow by it ("Let 3538 grow"). So the fringe is page on both.
//! Luma is the only signal here and in the fix: colour is out of scope
//! (MC-067 `## Out of scope`).
//!
//! # The fixtures: invented here, the cause turned on and off
//!
//! One geometry, [`W`] x [`H`], MC-065's: a band of textured **browser
//! chrome** across the top [`CHROME_H`] rows and the whole width, then the
//! **site** - one exact value on every row below the chrome - on both sides
//! of a **page** column [`PAGE_FIRST`] ..= [`PAGE_LAST`]. Each site is wider
//! than `chrome_max_extent` (0.30) of the width, so `content_box` cannot peel
//! it as a strip. A white **panel** crosses the page on [`PANEL_ROWS`], below
//! the central band, as `f09`'s bottom panel does from about row 1290: it is
//! what makes the outer fringe column flat over the band and not over the
//! viewport, as `f09`'s 1538 is.
//!
//! The page is dark textured art ([`CORE_LO`] / [`CORE_HI`]); its left edge
//! is plain textured art against the site, the case that works today. Only
//! the **right** edge carries the cause, as on `f09`.
//!
//! - **cause on** ([`Fringe::Dark`], site [`SITE`]): the art's last two
//!   columns are the fringe ([`FRINGE_FIRST`] ..= [`PAGE_LAST`]):
//!   - the inner column is near the site on about half its rows
//!     ([`INNER_NEAR`]) and darker art ([`INNER_ART`]) or an off value
//!     ([`INNER_OFF`]) on the rest: band median [`INNER_ART`], within
//!     `uniform_tolerance` of the site, **not** page background over the band,
//!     below `min_line_spread`;
//!   - the outer column is near the site on every band row ([`OUTER_LO`] ..
//!     [`OUTER_LO`] + [`OUTER_SPAN`]) and white on the panel's rows: page
//!     background over the band, not over the viewport.
//!
//!   Today the crop ends 2 columns short.
//! - **control, no fringe** ([`Fringe::Site`]): the same geometry with the
//!   two fringe columns replaced by the site value, so the art ends at
//!   [`FRINGE_FIRST`] - 1. Today the crop is exactly the art, and a fix must
//!   keep it so: what it keeps is the fringe, never a site column.
//! - **control, a site of different brightness** ([`Fringe::Dark`], site
//!   [`FAR_SITE`]): the same fringe beside a site far from its tone. Today the
//!   crop already keeps the fringe, so what cuts it is the fringe's tone being
//!   the site's - branch 1's comparison - and nothing else about it.
//!
//! The premise test measures every statistic above on the fixtures, from the
//! definitions, so a fixture that drifts goes red rather than making a case
//! vacuous.

use cropper_core::flat::central_band;
use cropper_core::viewport::locate;
use cropper_core::{Luma, Rect, Tuning, detect};

// --- Settled constants, read out --------------------------------------------

/// MC-049's page-background share: a column is page background when at least
/// this share of its pixels lie within `uniform_tolerance` of its upper
/// median. `flat.rs`'s `PAGE_BACKGROUND_PERCENT` is crate-private, so it is
/// copied from its documentation, as `page_column_site_edge.rs` does.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

/// `f09`'s site value (`2025-12-08 17_22_50.png` column 1539, one value), and
/// `Screenshot (3538).png`'s (columns 972 and 1573).
const SITE: u8 = 11;

/// "Near the site", as the Lead PO measured the fringes (MC-065 `## Notes`,
/// 2026-10-01): within this many levels of the site value. A description of
/// the fixture, not anything the crate reads.
const NEAR_SITE: u8 = 6;

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
/// The page's last column, inclusive, with the fringe; the right site is
/// `PAGE_LAST + 1 .. W`, also 130 columns.
const PAGE_LAST: u32 = 269;
/// The fringe's width: the 2 columns `f09` loses.
const FRINGE_W: u32 = 2;
/// The fringe's first column (the inner one); the outer one is [`PAGE_LAST`].
const FRINGE_FIRST: u32 = PAGE_LAST + 1 - FRINGE_W;
/// The white panel crossing the page below the central band.
const PANEL_ROWS: std::ops::Range<u32> = 200..230;
/// The panel's value.
const PANEL: u8 = 255;

/// The dark art's two values, on a hash: band median [`CORE_HI`], spread
/// about 20, far above `min_line_spread`.
const CORE_LO: u8 = 40;
const CORE_HI: u8 = 80;

/// The inner fringe column's near-site value, on [`INNER_NEAR_PERCENT`] of
/// its rows.
const INNER_NEAR: u8 = 9;
/// The inner fringe column's darker art, on the next
/// [`INNER_ART_PERCENT`] of its rows: its band median. 7 above the site, so
/// within `uniform_tolerance` of it and not within [`NEAR_SITE`].
const INNER_ART: u8 = 18;
/// The inner fringe column's off value, on the rest of its rows: outside
/// `uniform_tolerance` of its median, so the column is not page background.
const INNER_OFF: u8 = 34;
/// Percentages of the inner column's rows, by a hash of the pixel.
const INNER_NEAR_PERCENT: u32 = 45;
const INNER_ART_PERCENT: u32 = 43;

/// The outer fringe column's values on the band: `OUTER_LO ..= OUTER_LO +
/// OUTER_SPAN - 1`, by a hash, every one near the site.
const OUTER_LO: u8 = 7;
const OUTER_SPAN: u32 = 5;

/// The control site: far above the fringe's tone, and from the art's.
const FAR_SITE: u8 = 160;

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

/// What the art's last two columns are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fringe {
    /// `f09`'s dark, row-varying fringe: the cause.
    Dark,
    /// No fringe: the two columns are the site's value, like the site.
    Site,
}

/// The art's last column, inclusive, for `fringe`.
fn art_last(fringe: Fringe) -> u32 {
    match fringe {
        Fringe::Dark => PAGE_LAST,
        Fringe::Site => FRINGE_FIRST - 1,
    }
}

/// `f09`'s shape: dark textured art ending in `fringe`, a white panel across
/// the page below the band, and a site of one value, `site`.
fn dark_page(fringe: Fringe, site: u8) -> Luma {
    let last = art_last(fringe);
    render(|x, y| {
        if !(PAGE_FIRST..=last).contains(&x) {
            site
        } else if PANEL_ROWS.contains(&y) {
            PANEL
        } else if x == FRINGE_FIRST {
            match hash(x, y) % 100 {
                r if r < INNER_NEAR_PERCENT => INNER_NEAR,
                r if r < INNER_NEAR_PERCENT + INNER_ART_PERCENT => INNER_ART,
                _ => INNER_OFF,
            }
        } else if x == PAGE_LAST {
            OUTER_LO + (hash(x, y) % OUTER_SPAN) as u8
        } else if hash(x, y).is_multiple_of(2) {
            CORE_LO
        } else {
            CORE_HI
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
/// the band holds no chrome and no panel row).
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

/// Column `x`'s pixels over `rows`.
fn column_over(img: &Luma, x: u32, rows: std::ops::Range<u32>) -> Vec<u8> {
    rows.map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

/// Column `x`'s pixels over the band.
fn column(img: &Luma, x: u32) -> Vec<u8> {
    let band = band();
    column_over(img, x, band.y..band.y + band.h)
}

/// Column `x`'s pixels over the page's rows: everything below the chrome.
fn page_rows(img: &Luma, x: u32) -> Vec<u8> {
    column_over(img, x, CHROME_H..H)
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

/// The share of `values` within [`NEAR_SITE`] of [`SITE`].
fn near_site(values: &[u8]) -> f64 {
    values
        .iter()
        .filter(|&&v| v.abs_diff(SITE) <= NEAR_SITE)
        .count() as f64
        / values.len() as f64
}

/// The mean.
fn mean(values: &[u8]) -> f64 {
    values.iter().map(|&v| f64::from(v)).sum::<f64>() / values.len() as f64
}

/// Mean absolute deviation about the mean: the spread `min_line_spread` is
/// compared with.
fn spread(values: &[u8]) -> f64 {
    let m = mean(values);
    values
        .iter()
        .map(|&v| (f64::from(v) - m).abs())
        .sum::<f64>()
        / values.len() as f64
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

/// Every margin at which `img`'s crop is not exactly the art `PAGE_FIRST ..=
/// last` plus the margin, as a line naming what it got.
fn wrong_columns(img: &Luma, last: u32) -> Vec<String> {
    both_margins()
        .iter()
        .filter_map(|t| {
            let got = crop_columns(img, t);
            let want = (PAGE_FIRST - t.margin_px, last + t.margin_px);
            (got != want).then(|| {
                let off = i64::from(want.1) - i64::from(got.1);
                let right = if off >= 0 {
                    format!("right edge short by {off}")
                } else {
                    format!("right edge {} past it, into the site", -off)
                };
                format!(
                    "margin_px {}: crop columns {}..={}, the art is {}..={} ({right})",
                    t.margin_px, got.0, got.1, want.0, want.1,
                )
            })
        })
        .collect()
}

// --- The fixtures are what they claim to be ---------------------------------

/// The premises every case below stands on, measured on the fixtures from the
/// definitions. Only `central_band` and `viewport::locate` are read from the
/// crate: the first to say which rows are the band, the second to show the
/// viewport the widening reads is the rows below the chrome.
#[test]
fn the_fixtures_fringe_and_sites_measure_as_the_cause_claims() {
    let t = Tuning::default();
    let tol = t.uniform_tolerance;
    let rule = f64::from(t.min_line_spread);
    let band = band();
    let mut wrong = Vec::new();
    let mut check = |ok: bool, what: String| {
        if !ok {
            wrong.push(what);
        }
    };

    check(
        band.y >= CHROME_H && band.y + band.h <= PANEL_ROWS.start,
        format!(
            "the central band {band:?} must hold no chrome row (< {CHROME_H}) and no panel \
             row ({PANEL_ROWS:?})"
        ),
    );

    // Every site column is one exact value below the chrome, in every case.
    for (img, fringe, site) in [
        (dark_page(Fringe::Dark, SITE), Fringe::Dark, SITE),
        (dark_page(Fringe::Site, SITE), Fringe::Site, SITE),
        (dark_page(Fringe::Dark, FAR_SITE), Fringe::Dark, FAR_SITE),
    ] {
        for x in [0, PAGE_FIRST - 1, art_last(fringe) + 1, W - 1] {
            let values = page_rows(&img, x);
            check(
                values.iter().all(|&v| v == site),
                format!("{fringe:?}, site {site}: column {x} must be the single value {site}"),
            );
        }
    }

    let dark = dark_page(Fringe::Dark, SITE);
    let (inner, outer) = (FRINGE_FIRST, PAGE_LAST);
    let core_edge = column(&dark, FRINGE_FIRST - 1);
    let (inner_band, outer_band) = (column(&dark, inner), column(&dark, outer));

    // The art's last textured column: the widest run ends on it, and its tone
    // is far from the fringe's and from both sites'.
    check(
        spread(&core_edge) >= rule,
        format!(
            "the art's last column {} must be textured: spread {:.2} >= {rule}",
            FRINGE_FIRST - 1,
            spread(&core_edge)
        ),
    );
    let core_tone = median(&core_edge);
    for (what, tone) in [
        ("the inner fringe", median(&inner_band)),
        ("the site", SITE),
        ("the far site", FAR_SITE),
    ] {
        check(
            core_tone.abs_diff(tone) > tol,
            format!("the art's edge tone {core_tone} must be more than {tol} from {what}'s {tone}"),
        );
    }

    // The inner fringe: near the site on about half its rows; band median
    // within tolerance of the site and not page background over the band;
    // below min_line_spread, so outside the widest run.
    check(
        median(&inner_band) == INNER_ART && INNER_ART.abs_diff(SITE) <= tol,
        format!(
            "inner fringe {inner}: band median {} must be {INNER_ART}, within {tol} of the \
             site {SITE}",
            median(&inner_band)
        ),
    );
    check(
        share(&inner_band, tol) < PAGE_BACKGROUND_SHARE,
        format!(
            "inner fringe {inner}: must not be page background over the band, share {:.3}",
            share(&inner_band, tol)
        ),
    );
    check(
        spread(&inner_band) < rule,
        format!(
            "inner fringe {inner}: spread {:.2} must be below {rule}",
            spread(&inner_band)
        ),
    );
    let inner_near = near_site(&page_rows(&dark, inner));
    check(
        (0.30..=0.60).contains(&inner_near),
        format!(
            "inner fringe {inner}: near the site on about half its rows (0.30..=0.60), \
             measured {inner_near:.3}"
        ),
    );

    // The outer fringe: near the site on about 90% of its rows; page
    // background over the band, not over the viewport's rows; its tone
    // carries the inner fringe's on.
    check(
        share(&outer_band, tol) >= PAGE_BACKGROUND_SHARE,
        format!(
            "outer fringe {outer}: must be page background over the band, share {:.3}",
            share(&outer_band, tol)
        ),
    );
    let outer_view = share(&page_rows(&dark, outer), tol);
    check(
        outer_view < PAGE_BACKGROUND_SHARE,
        format!(
            "outer fringe {outer}: must not be page background below the chrome, share {outer_view:.3}"
        ),
    );
    check(
        median(&outer_band).abs_diff(median(&inner_band)) <= tol
            && median(&outer_band).abs_diff(SITE) <= tol,
        format!(
            "outer fringe {outer}: band median {} must be within {tol} of the inner fringe's \
             {} and of the site {SITE}",
            median(&outer_band),
            median(&inner_band)
        ),
    );
    check(
        spread(&outer_band) < rule,
        format!(
            "outer fringe {outer}: spread {:.2} must be below {rule}",
            spread(&outer_band)
        ),
    );
    let outer_near = near_site(&page_rows(&dark, outer));
    check(
        (0.80..0.95).contains(&outer_near),
        format!(
            "outer fringe {outer}: near the site on about 90% of its rows (0.80..0.95), \
             measured {outer_near:.3}"
        ),
    );

    // Similar mean brightness: over the band, each fringe column's mean is
    // within tolerance of the site's value.
    for x in [inner, outer] {
        let m = mean(&column(&dark, x));
        check(
            (m - f64::from(SITE)).abs() <= f64::from(tol),
            format!("fringe {x}: band mean {m:.2} must be within {tol} of the site {SITE}"),
        );
    }

    // The viewport the widening reads beside the art is every row below the
    // chrome, panel included: the rows over which the outer fringe is not
    // page background.
    let art = Rect {
        x: PAGE_FIRST,
        y: 0,
        w: FRINGE_FIRST - PAGE_FIRST,
        h: H,
    };
    let view = locate(&dark, art, &t);
    // The measured values, for the story's controls table (`--nocapture`).
    println!(
        "art edge {}: median {core_tone} spread {:.2} | inner {inner}: band median {} share \
         {:.3} spread {:.2} mean {:.2} near-site {inner_near:.3} | outer {outer}: band median \
         {} share {:.3} spread {:.2} mean {:.2}, share below the chrome {outer_view:.3}, \
         near-site {outer_near:.3} | viewport {view:?}",
        FRINGE_FIRST - 1,
        spread(&core_edge),
        median(&inner_band),
        share(&inner_band, tol),
        spread(&inner_band),
        mean(&inner_band),
        median(&outer_band),
        share(&outer_band, tol),
        spread(&outer_band),
        mean(&outer_band),
    );
    check(
        view.is_some_and(|v| v.top == CHROME_H && v.bottom == H),
        format!("the viewport beside the art must be {CHROME_H}..{H}; located {view:?}"),
    );

    assert!(
        wrong.is_empty(),
        "MC-067 AC-4's fixture premises do not hold:\n{}",
        wrong.join("\n")
    );
}

// --- The cause --------------------------------------------------------------

/// AC-4, `f09`'s cause: a dark, row-varying fringe at the art's edge - near the
/// site on about half its rows, then on about 90% - beside a single-value site
/// of similar brightness is page, not margin. Fails today: the crop ends
/// [`FRINGE_W`] columns short at both margins.
#[test]
fn a_dark_row_varying_fringe_beside_a_site_of_similar_brightness_is_kept_at_both_margins() {
    let wrong = wrong_columns(&dark_page(Fringe::Dark, SITE), PAGE_LAST);
    assert!(
        wrong.is_empty(),
        "MC-067 AC-4 (f09's shape): the art's dark fringe ({FRINGE_W} columns, \
         {FRINGE_FIRST}..={PAGE_LAST}; the inner one not page background, band median \
         {INNER_ART}) is page, and the site's background ({SITE}, one value) starts at {}; \
         the crop must end on the art's last column:\n{}",
        PAGE_LAST + 1,
        wrong.join("\n")
    );
}

/// AC-4's control: the same geometry with the fringe columns replaced by the
/// site value crops exactly to the art. Passes today, and must keep passing:
/// a fix keeps the fringe, never a column of the site.
#[test]
fn control_the_same_page_without_its_fringe_is_cropped_exactly_to_the_art() {
    let last = art_last(Fringe::Site);
    let wrong = wrong_columns(&dark_page(Fringe::Site, SITE), last);
    assert!(
        wrong.is_empty(),
        "MC-067 AC-4's control: with the fringe replaced by the site value ({SITE}), the \
         art ends at {last} and the crop must end exactly there - no site column kept:\n{}",
        wrong.join("\n")
    );
}

/// A second control, on the cause itself: the same fringe beside a site far
/// from its tone is kept today. Passes today, and must keep passing. With it,
/// the cause test going red says what cuts the fringe: its tone being the
/// site's, which is branch 1's comparison, and nothing else about the fringe.
#[test]
fn control_the_same_fringe_beside_a_site_of_different_brightness_is_kept_at_both_margins() {
    let wrong = wrong_columns(&dark_page(Fringe::Dark, FAR_SITE), PAGE_LAST);
    assert!(
        wrong.is_empty(),
        "MC-067 AC-4's cause control: the same fringe beside a site of {FAR_SITE} must be \
         kept:\n{}",
        wrong.join("\n")
    );
}
