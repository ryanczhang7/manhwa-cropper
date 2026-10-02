//! MC-070 AC-3, cause A: a close call on the **page's own bottom rows** - a
//! Top or Bottom strip that is dark page margin on both sides and a distinct
//! flat tone over the page column - does not flag the page. Seen through
//! [`decide`], on generated images. The corpus half is `Screenshot (2705).png`
//! (AC-1, `crates/engine/tests/corpus_reported_ambiguous.rs`); this file is
//! what the required `unit` gate sees of cause A.
//!
//! # The cause, measured on the screenshot
//!
//! `Screenshot (2705).png` (`toongod`, dark, 2560x1440) answers
//! `Flag(Ambiguous)` on `main`. `content_box` peels Top, Bottom, Right,
//! Bottom, and then judges the Bottom strip `0,1366 2545x26` - between the
//! art's bottom edge (row 1365) and the taskbar - nearly chrome: flat fraction
//! 0.848708, inside `[0.8475, 0.85)`. The strip lies inside the crop (the
//! viewport runs to row 1392), so MC-069's rule ("a close call flags only if
//! the crop includes part of the strip") still flags it. Its parts (MC-070
//! `## Notes`, re-measured in RED with a scratch probe outside the repo):
//!
//! ```text
//! part                     columns       median  flat fraction
//! left margin              0..1073       11      1.000000
//! over the page column     1073..1472    255     0.926933  (share within 10 of 11: 0.039)
//! right margin             1472..2545    11      0.998351
//! whole strip              0..2545       11      0.848708
//! ```
//!
//! It is not chrome painted edge to edge: it is the page's own white gap,
//! between two dark margins. Its median is the margins' (11), and its tone
//! over the page column (255) differs from that by far more than
//! `uniform_tolerance`. MC-070's candidate rule (`## Notes`) reads exactly
//! that difference; this file pins the outcome, not the rule.
//!
//! # The scenes: invented here, one geometry, three bottom strips
//!
//! [`W`] x [`H`]: textured browser chrome across the top [`CHROME_H`] rows and
//! the whole width; below it a **site** of one exact value, [`SITE`], on both
//! sides of the page column [`PAGE_FIRST`] .. [`PAGE_END`] (each site 210 of
//! 500 columns, 0.42 of the width, past `chrome_max_extent`, so neither is
//! ever a side strip); textured art on the page column down to row
//! [`STRIP_TOP`]; and the bottom [`STRIP_H`] rows, the **strip**, bounded by a
//! strong row line above it and by the image's edge below.
//!
//! Every strip has exactly [`NON_FLAT`] of [`W`] pixels per row off its
//! median, so all three measure the same flat fraction, `424/500 = 0.848`,
//! inside `[0.8475, 0.85)`, and are equally near chrome by flatness alone:
//!
//! | scene | sides | over the page column | strip median | page-column median | expected |
//! |---|---|---|---|---|---|
//! | [`Strip::PageGap`] (cause) | [`SITE`] | [`GAP`] white, 76 of 80 columns | 11 | 255 | `Crop` |
//! | [`Strip::ChromeOtherTone`] (control) | [`CHROME_TONE`] + text, edge to edge | the same | 200 | 200 | `Flag(Ambiguous)` |
//! | [`Strip::ChromeMarginTone`] (control) | [`SITE`] + text, edge to edge | the same | 11 | 11 | `Flag(Ambiguous)` |
//!
//! - The **cause** is `(2705)`'s strip: the margins' tone on both sides, the
//!   page's white gap over the page column (95 % of it, the rest the margins'
//!   tone, as `(2705)`'s 0.927 flat over 255). The art's last row
//!   ([`EDGE_ROW`]) is dark, as `(2705)`'s row 1365 is (33 over the page),
//!   which is what makes the strip's top a strong line.
//! - **Control 1** is true chrome: one tone painted edge to edge, its text
//!   spread over the whole width, so its tone over the page column is its own
//!   median. It also tells the rule's reference apart: compared against the
//!   page background tone (11) rather than the strip's own median (200), its
//!   page-column tone would read as "the page's".
//! - **Control 2** is true chrome **at the margins' tone**: the same strip
//!   median as the cause, so what separates the two is the strip's tone over
//!   the page column and nothing else about the strip. Its top is a strong
//!   line only against bright art, so its art's last row is white, not dark -
//!   the one difference above the strip, and the premise test measures the
//!   line in every scene.
//!
//! Neither control is cut off by the viewport stage: a strip of [`STRIP_H`]
//! rows is shorter than `viewport::MIN_RUN` (16), so MC-054's evidence rule
//! leaves the bottom rows in, and the crop includes the strip in all three
//! scenes - the premise test checks it, at both margins. On `main` all three
//! are `Flag(Ambiguous)`.
//!
//! # Export shape
//!
//! Only API that exists on `main`: `cropper_core::{decide, detect,
//! CropDecision, FlagReason, Luma, Rect, Tuning}`, `content::content_box`,
//! `trim::trim_uniform`, `edges::{row_profile, strong_lines}`. Not pinned:
//! where the rule lives, and whether `ContentBox::ambiguous` or
//! `Detection::ambiguous` is false on the cause scene. Both controls must
//! stay strip-level close calls (`ContentBox::ambiguous`), because the rule
//! says they are nearly chrome.

use cropper_core::content::content_box;
use cropper_core::edges::{row_profile, strong_lines};
use cropper_core::trim::trim_uniform;
use cropper_core::{CropDecision, FlagReason, Luma, Rect, Tuning, decide, detect};

// --- The geometry, invented here --------------------------------------------

/// The scene's width.
const W: u32 = 500;
/// The scene's height.
const H: u32 = 240;
/// Rows of textured browser chrome across the top: more than
/// `viewport::MIN_RUN` (16), so the viewport stage cuts them.
const CHROME_H: u32 = 24;
/// The page column's first column. The left site is `0 .. PAGE_FIRST`, 210
/// columns, 0.42 of the width.
const PAGE_FIRST: u32 = 210;
/// The page column's end, exclusive: 80 columns, at least
/// `Tuning::min_content_side` (64). The right site is `PAGE_END .. W`, 210
/// columns.
const PAGE_END: u32 = 290;
/// Rows in the bottom strip: fewer than `viewport::MIN_RUN` (16), so no strip
/// here is evidence of chrome for the viewport stage.
///
/// And at most 10, so that a control's strip, painted edge to edge, leaves
/// every site column page background over the viewport's rows (206 of 216
/// rows at the site's value, 0.954 against MC-049's 0.95). That keeps any
/// page-column rule that reads the viewport out of this file: at 12 rows
/// (0.944) such a rule would see the site columns as "not page background
/// over the viewport", and MC-070's RED measured a candidate of that kind
/// (cause B in `## Notes`, since ruled out of scope) walking across the whole
/// site - which says something about that rule, not about cause A.
const STRIP_H: u32 = 10;
/// The strip's first row.
const STRIP_TOP: u32 = H - STRIP_H;
/// The art's last row, just above the strip.
const EDGE_ROW: u32 = STRIP_TOP - 1;

/// The site's value, `(2705)`'s margin tone (MC-070 `## Notes`).
const SITE: u8 = 11;
/// The page's white gap in the cause's strip, `(2705)`'s tone over the page.
const GAP: u8 = 255;
/// Control 1's chrome tone: far from the site and from the art's tones.
const CHROME_TONE: u8 = 200;
/// How far a chrome "text" pixel sits from its strip's background: twice
/// `uniform_tolerance`, as `common::CHROME_DEVIATION`. Upward only, so a strip
/// at [`SITE`] stays in range.
const TEXT_DEVIATION: u8 = 20;

/// Pixels per strip row off the strip's median: 76 of 500, flat fraction
/// `424/500 = 0.848`, inside `[0.8475, 0.85)`. 75 would be 0.85, chrome; 77
/// would be 0.846, below the band.
const NON_FLAT: u32 = 76;
/// The page-column columns of the cause's strip that are the site's tone, not
/// the gap's: 2 at each end, so the gap is exactly [`NON_FLAT`] columns.
const GAP_INSET: u32 = 2;

/// The art's two tones on the page, on a hash: band medians 40 or 200, both
/// more than `uniform_tolerance` from [`SITE`], so the page column ends at the
/// site on both sides (MC-053's branch 1).
const ART_LO: u8 = 40;
const ART_HI: u8 = 200;
/// The cause's and control 1's art edge row: dark, as `(2705)`'s.
const EDGE_DARK: [u8; 2] = [0, 40];
/// Control 2's art edge row: white, so a strip at the site's tone has a
/// strong line above it.
const EDGE_WHITE: u8 = 255;

/// The page column's rect over every row.
const PAGE: Rect = Rect {
    x: PAGE_FIRST,
    y: 0,
    w: PAGE_END - PAGE_FIRST,
    h: H,
};

/// The bottom strip's rect.
const STRIP: Rect = Rect {
    x: 0,
    y: STRIP_TOP,
    w: W,
    h: STRIP_H,
};

/// The crop every scene's geometry gives at `margin_px`: the page column,
/// from the first row below the chrome to the image's bottom edge, strip
/// included. At margin 3 the columns grow by 3 a side and the rows are
/// clamped to the viewport `CHROME_H .. H`.
fn expected_crop(margin_px: u32) -> Rect {
    Rect {
        x: PAGE_FIRST - margin_px,
        y: CHROME_H,
        w: PAGE_END - PAGE_FIRST + 2 * margin_px,
        h: H - CHROME_H,
    }
}

// --- The fixtures -------------------------------------------------------------

/// Which bottom strip a scene has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strip {
    /// `(2705)`'s: the site's tone on both sides, the page's white gap over
    /// the page column. The cause.
    PageGap,
    /// True chrome of another tone, painted edge to edge. Control 1.
    ChromeOtherTone,
    /// True chrome at the site's tone, painted edge to edge. Control 2.
    ChromeMarginTone,
}

const ALL: [Strip; 3] = [
    Strip::PageGap,
    Strip::ChromeOtherTone,
    Strip::ChromeMarginTone,
];

/// A well-mixed hash of a pixel position, so texture has no row or column
/// structure of its own.
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

/// Whether strip pixel `(x, y)` is a chrome "text" pixel: exactly
/// [`NON_FLAT`] per row, spread evenly over the whole width by a Bresenham
/// walk, rotated 37 columns a row so rows do not line up into a column.
fn is_text(x: u32, y: u32) -> bool {
    let r = (x + 37 * y) % W;
    (r + 1) * NON_FLAT / W != r * NON_FLAT / W
}

/// One pixel of the bottom strip.
fn strip_pixel(strip: Strip, x: u32, y: u32) -> u8 {
    match strip {
        Strip::PageGap => {
            let gap = PAGE_FIRST + GAP_INSET..PAGE_END - GAP_INSET;
            if gap.contains(&x) { GAP } else { SITE }
        }
        Strip::ChromeOtherTone if is_text(x, y) => CHROME_TONE + TEXT_DEVIATION,
        Strip::ChromeOtherTone => CHROME_TONE,
        Strip::ChromeMarginTone if is_text(x, y) => SITE + TEXT_DEVIATION,
        Strip::ChromeMarginTone => SITE,
    }
}

/// The art's last row, just above the strip.
fn edge_pixel(strip: Strip, x: u32, y: u32) -> u8 {
    match strip {
        Strip::ChromeMarginTone => EDGE_WHITE,
        _ => EDGE_DARK[(hash(x, y) % 2) as usize],
    }
}

/// The scene with bottom strip `strip`.
fn scene(strip: Strip) -> Luma {
    let page = PAGE_FIRST..PAGE_END;
    let mut data = Vec::with_capacity((W * H) as usize);
    for y in 0..H {
        for x in 0..W {
            data.push(if y < CHROME_H {
                chrome(x, y)
            } else if y >= STRIP_TOP {
                strip_pixel(strip, x, y)
            } else if !page.contains(&x) {
                SITE
            } else if y == EDGE_ROW {
                edge_pixel(strip, x, y)
            } else if hash(x, y).is_multiple_of(2) {
                ART_LO
            } else {
                ART_HI
            });
        }
    }
    Luma {
        width: W,
        height: H,
        data,
    }
}

// --- Measuring, test-side -------------------------------------------------------

/// `rect`'s pixels.
fn pixels(img: &Luma, rect: Rect) -> Vec<u8> {
    (rect.y..rect.y + rect.h)
        .flat_map(|y| (rect.x..rect.x + rect.w).map(move |x| (x, y)))
        .map(|(x, y)| img.data[(y * img.width + x) as usize])
        .collect()
}

/// The upper median, `sorted[n / 2]` - `content_box`'s convention.
fn median(values: &[u8]) -> u8 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

/// `rect`'s flat fraction as `content_box` defines it: the share of its
/// pixels within `tolerance` of its upper median, inclusive, in `f32`.
fn flat_fraction(img: &Luma, rect: Rect, tolerance: u8) -> f32 {
    let values = pixels(img, rect);
    let m = median(&values);
    let flat = values
        .iter()
        .filter(|&&v| v.abs_diff(m) <= tolerance)
        .count();
    flat as f32 / values.len() as f32
}

/// The strip's part over the page column.
fn over_the_page(strip: Rect) -> Rect {
    Rect {
        x: PAGE.x,
        w: PAGE.w,
        ..strip
    }
}

/// Whether `outer` contains `inner` entirely.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// Whether `a` and `b` share at least one pixel.
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// Both margins the corpus suites test: 0 (the default) and 3. The 0 is
/// written out so the pair cannot collapse if the default moves.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning {
            margin_px: 0,
            ..Tuning::default()
        },
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
    ]
}

/// `decide`'s answer for `strip` at both margins, as `(margin_px, answer)`.
fn decided(strip: Strip) -> Vec<(u32, CropDecision)> {
    let img = scene(strip);
    both_margins()
        .iter()
        .map(|t| (t.margin_px, decide(&img, t)))
        .collect()
}

// --- The scenes are what they claim to be -----------------------------------

/// Every premise the three tests below stand on, measured from the pixels:
/// each strip's flat fraction is exactly `424/500` and inside the band; its
/// median and its tone over the page column are the table's; its top is a
/// strong row line and it holds none of its own, so it is exactly
/// `content_box`'s Bottom strip; and `detect`'s rect is the expected crop at
/// both margins and includes the strip, so the close call is inside the crop
/// in every scene. Both controls are strip-level close calls.
///
/// Green on `main` and after: none of it is MC-070's behaviour. `detect`'s
/// rect is not MC-070's to move in these scenes - only whether `decide`
/// crops to it.
#[test]
fn scene_premises_hold() {
    let t = Tuning::default();
    let tol = t.uniform_tolerance;
    let foot = t.chrome_flat_fraction - t.ambiguity_band;
    let threshold = f32::from(t.edge_threshold);
    let mut wrong = Vec::new();
    let mut check = |ok: bool, what: String| {
        if !ok {
            wrong.push(what);
        }
    };

    for strip in ALL {
        let img = scene(strip);
        let flat = flat_fraction(&img, STRIP, tol);
        check(
            flat == 424.0 / 500.0 && flat >= foot && flat < t.chrome_flat_fraction,
            format!(
                "{strip:?}: the strip's flat fraction is {flat}, not 424/500 inside \
                 [{foot}, {})",
                t.chrome_flat_fraction
            ),
        );
        let own = median(&pixels(&img, STRIP));
        let page = median(&pixels(&img, over_the_page(STRIP)));
        let (want_own, want_page) = match strip {
            Strip::PageGap => (SITE, GAP),
            Strip::ChromeOtherTone => (CHROME_TONE, CHROME_TONE),
            Strip::ChromeMarginTone => (SITE, SITE),
        };
        check(
            (own, page) == (want_own, want_page),
            format!(
                "{strip:?}: strip median {own} and page-column median {page}, want \
                 {want_own} and {want_page}"
            ),
        );
        // Every row of the strip carries the same flat fraction, so no part
        // of it is chrome by itself.
        for y in STRIP_TOP..H {
            let row = Rect { y, h: 1, ..STRIP };
            let f = flat_fraction(&img, row, tol);
            check(
                f == 424.0 / 500.0,
                format!("{strip:?}: strip row {y} has flat fraction {f}, not 424/500"),
            );
        }

        // The strip is exactly content_box's Bottom strip: the last strong
        // row line of the whole image is the one above it.
        let whole = Rect {
            x: 0,
            y: 0,
            w: W,
            h: H,
        };
        let profile = row_profile(&img, whole);
        let lines = strong_lines(&profile, &t);
        let last = lines.last().map(|l| l.end);
        // The measured values, for the story's controls table (`--nocapture`).
        println!(
            "{strip:?}: strip flat fraction {flat} (424/500 = {}), strip median {own}, \
             page-column median {page}, boundary line {:.2} (edge_threshold {threshold})",
            424.0f32 / 500.0,
            profile[EDGE_ROW as usize]
        );
        check(
            last == Some(EDGE_ROW as usize),
            format!(
                "{strip:?}: the last strong row line must sit between rows {EDGE_ROW} and \
                 {STRIP_TOP} (profile index {EDGE_ROW}), found {last:?}; the boundary \
                 measures {:.2} against edge_threshold {threshold}",
                profile[EDGE_ROW as usize]
            ),
        );
        check(
            trim_uniform(&img, &t) == Some(whole),
            format!("{strip:?}: the uniform trim must leave the whole image"),
        );

        for m in both_margins() {
            let rect = detect(&img, &m).expect("the scene is textured").rect;
            check(
                rect == expected_crop(m.margin_px) && contains(rect, over_the_page(STRIP)),
                format!(
                    "{strip:?}: at margin_px {} detect's rect is {rect:?}, want {:?}, \
                     including the strip's rows {STRIP_TOP}..{H}",
                    m.margin_px,
                    expected_crop(m.margin_px)
                ),
            );
            check(
                overlaps(rect, STRIP),
                format!("{strip:?}: the crop {rect:?} must include part of the strip"),
            );
        }
    }

    for strip in [Strip::ChromeOtherTone, Strip::ChromeMarginTone] {
        let img = scene(strip);
        let first = trim_uniform(&img, &t).expect("the scene is textured");
        check(
            content_box(&img, first, &t).ambiguous,
            format!("{strip:?}: content_box must call the edge-to-edge strip a close call"),
        );
    }

    assert!(
        wrong.is_empty(),
        "MC-070 AC-3 (cause A): the scenes are not what their tests describe:\n{}",
        wrong.join("\n")
    );
}

// --- AC-3, cause A -------------------------------------------------------------

/// AC-3, cause A: `(2705)`'s defect, generated. The bottom strip is nearly
/// chrome by flatness (424/500 = 0.848), but it is the page's own white gap
/// between two dark margins - its tone over the page column (255) is not its
/// median (11) - so it is the page's rows, not a close call, and the page is
/// cropped, strip rows included, at both margins.
///
/// **Red on `main`**, which answers `Flag(Ambiguous)` at both.
#[test]
fn the_pages_own_white_bottom_rows_between_dark_margins_are_cropped_not_flagged() {
    let want: Vec<(u32, CropDecision)> = both_margins()
        .iter()
        .map(|t| (t.margin_px, CropDecision::Crop(expected_crop(t.margin_px))))
        .collect();
    assert_eq!(
        decided(Strip::PageGap),
        want,
        "MC-070 AC-3 (cause A, Screenshot (2705)'s shape): the bottom strip rows \
         {STRIP_TOP}..{H} are the margins' tone ({SITE}) on both sides and the page's \
         white gap ({GAP}) over the page column {PAGE_FIRST}..{PAGE_END}, flat fraction \
         424/500 = 0.848. Its tone over the page column differs from its own median, so \
         it is the page's bottom rows and not nearly chrome: the page must be cropped, \
         those rows included. `left` is measured, as (margin_px, decision)"
    );
}

/// AC-3, cause A, at the strip judgement. MC-070's rule is a statement about
/// the strip - one whose tone over the page column is not its own median is
/// the page's, "not chrome, and not nearly chrome" (MC-070 `## Notes`) - so
/// `content_box` no longer calls it a close call. The corpus pin that matches
/// it is `corpus.rs`'s `STRIP_AMBIGUOUS_AT_THE_BAND`, which `(2705)` leaves.
///
/// **Red on `main`**, where `ContentBox::ambiguous` is true. The controls'
/// strips stay strip-level close calls (`scene_premises_hold`).
#[test]
fn content_box_does_not_call_the_pages_own_bottom_rows_a_close_call() {
    let t = Tuning::default();
    let img = scene(Strip::PageGap);
    let first = trim_uniform(&img, &t).expect("the scene is textured");
    let found = content_box(&img, first, &t);
    assert!(
        !found.ambiguous,
        "MC-070 AC-3 (cause A): the bottom strip rows {STRIP_TOP}..{H} have median {SITE} \
         and tone {GAP} over the page column, so they are the page's rows and not nearly \
         chrome; content_box must not call them a close call. Got {found:?}"
    );
}

/// AC-3's first control: the same geometry, the strip true chrome - one tone
/// ([`CHROME_TONE`]) painted edge to edge, text spread over the whole width -
/// at the same flat fraction, inside the crop. Its tone over the page column
/// is its own median, so it is nearly chrome and the page is still for
/// review. Green on `main`; must stay green.
///
/// It fails a fix that stops flagging every Top or Bottom close call, and one
/// that compares the page-column tone with the page background tone
/// ([`SITE`]) instead of the strip's own median.
#[test]
fn control_true_chrome_of_another_tone_painted_edge_to_edge_still_flags() {
    let want: Vec<(u32, CropDecision)> = both_margins()
        .iter()
        .map(|t| (t.margin_px, CropDecision::Flag(FlagReason::Ambiguous)))
        .collect();
    assert_eq!(
        decided(Strip::ChromeOtherTone),
        want,
        "MC-070 AC-3's control: the bottom strip is one tone ({CHROME_TONE}) painted edge \
         to edge with text pixels, flat fraction 424/500 = 0.848, inside the crop - its \
         tone over the page column is its own median, so it is nearly chrome and the \
         page stays Flag(Ambiguous). `left` is measured, as (margin_px, decision)"
    );
}

/// AC-3's second control: true chrome **at the margins' tone** ([`SITE`]),
/// painted edge to edge, the same flat fraction and the same strip median as
/// the cause. What separates it from the cause is the strip's tone over the
/// page column, which here is its own median: nearly chrome, still flagged.
/// Green on `main`; must stay green.
///
/// It fails a fix that reads "a strip at the margin's tone is margin" instead
/// of comparing the strip's tone over the page column with its own median.
#[test]
fn control_true_chrome_at_the_margins_tone_painted_edge_to_edge_still_flags() {
    let want: Vec<(u32, CropDecision)> = both_margins()
        .iter()
        .map(|t| (t.margin_px, CropDecision::Flag(FlagReason::Ambiguous)))
        .collect();
    assert_eq!(
        decided(Strip::ChromeMarginTone),
        want,
        "MC-070 AC-3's control: the bottom strip is the margins' tone ({SITE}) painted \
         edge to edge with text pixels, flat fraction 424/500 = 0.848, inside the crop - \
         the same median as the page's white gap, but its tone over the page column is \
         that median too, so it is nearly chrome and the page stays Flag(Ambiguous). \
         `left` is measured, as (margin_px, decision)"
    );
}

// --- The same, upside down: a Top strip --------------------------------------

/// `img` turned upside down: row `y` becomes row `H - 1 - y`. The chrome is
/// then a band along the bottom edge (a taskbar, cut by the viewport stage
/// as the chrome was) and the strip is the page's own **top** rows, which
/// `content_box` judges as its Top strip.
fn upside_down(img: &Luma) -> Luma {
    let width = img.width as usize;
    let data = img
        .data
        .chunks_exact(width)
        .rev()
        .flatten()
        .copied()
        .collect();
    Luma {
        width: img.width,
        height: img.height,
        data,
    }
}

/// The upside-down scene's crop at `margin_px`: the page column from the
/// image's top edge, strip included, to the first chrome row.
fn expected_crop_upside_down(margin_px: u32) -> Rect {
    Rect {
        y: 0,
        ..expected_crop(margin_px)
    }
}

/// `decide` on `strip`'s scene turned upside down, at both margins.
fn decided_upside_down(strip: Strip) -> Vec<(u32, CropDecision)> {
    let img = upside_down(&scene(strip));
    both_margins()
        .iter()
        .map(|t| (t.margin_px, decide(&img, t)))
        .collect()
}

/// AC-3, cause A on a **Top** strip: the page's own white top rows between
/// two dark margins are the page's, not a close call, exactly as at the
/// bottom. MC-070's rule names "a Top or Bottom strip" (`## Notes`), and
/// `(2705)` only shows the Bottom one. **Red on `main`**.
///
/// The geometry is the cause scene's, mirrored, so its premises are the
/// bottom scene's; `detect`'s rect is checked here rather than assumed.
#[test]
fn the_pages_own_white_top_rows_between_dark_margins_are_cropped_not_flagged() {
    let img = upside_down(&scene(Strip::PageGap));
    let rects: Vec<Rect> = both_margins()
        .iter()
        .map(|t| detect(&img, t).expect("the scene is textured").rect)
        .collect();
    let want_rects: Vec<Rect> = both_margins()
        .iter()
        .map(|t| expected_crop_upside_down(t.margin_px))
        .collect();
    assert_eq!(
        rects, want_rects,
        "premise: upside down, detect's rect must be the page column from row 0, \
         the strip's rows 0..{STRIP_H} included"
    );
    let want: Vec<(u32, CropDecision)> = both_margins()
        .iter()
        .map(|t| {
            (
                t.margin_px,
                CropDecision::Crop(expected_crop_upside_down(t.margin_px)),
            )
        })
        .collect();
    assert_eq!(
        decided_upside_down(Strip::PageGap),
        want,
        "MC-070 AC-3 (cause A, Top): the top strip rows 0..{STRIP_H} are the margins' \
         tone ({SITE}) on both sides and the page's white gap ({GAP}) over the page \
         column, flat fraction 424/500; they are the page's top rows, not nearly \
         chrome, and the page must be cropped. `left` is measured, as (margin_px, decision)"
    );
}

/// AC-3's control on a Top strip: true chrome at the margins' tone, painted
/// edge to edge, along the page's top rows - still nearly chrome, still
/// flagged. Green on `main`; must stay green.
#[test]
fn control_true_chrome_at_the_margins_tone_along_the_top_still_flags() {
    let want: Vec<(u32, CropDecision)> = both_margins()
        .iter()
        .map(|t| (t.margin_px, CropDecision::Flag(FlagReason::Ambiguous)))
        .collect();
    assert_eq!(
        decided_upside_down(Strip::ChromeMarginTone),
        want,
        "MC-070 AC-3's control (Top): the top strip is the margins' tone ({SITE}) \
         painted edge to edge with text pixels, flat fraction 424/500, inside the crop - \
         nearly chrome, so the page stays Flag(Ambiguous). `left` is measured"
    );
}
