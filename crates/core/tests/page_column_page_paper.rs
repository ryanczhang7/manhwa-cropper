//! MC-074 AC-3: a page whose drawn panels sit on **flat white page paper**,
//! between a dark site margin of one exact value on both sides, has a page
//! column that is the paper's full width - seen through
//! [`page_column`] and [`detect`], on generated images. The corpus half is
//! `n02` (`2025-03-07 00_05_58.png`) in `crates/engine/tests` (AC-1, AC-2);
//! this file is what the required `unit` gate sees of the story.
//!
//! # The bug, and where it lives
//!
//! Measured by the Lead PO on `n02` (MC-074 `## Context`, "Measured, and
//! ruled by the user"): site tone 11 on every margin column; columns
//! 643..700 and 1103..1175 white page paper, band median 255, spread 0;
//! the drawn panels 703..1100 the widest textured run. Stage 3c,
//! `flat::page_column`, takes the run and widens it (`Margin::belongs`).
//! The first paper column is page background over the band, is **not** at
//! the site's tone, and does **not** continue the run's edge (band median 38,
//! dark art, against 251..255), so branch 3 returns margin there: the page
//! column is the panels only, and the paper - which the user ruled is page
//! ("Box stands") - is cut. Beside that wrong column the viewport stage
//! declines, so the rows also run the full image height.
//!
//! # The fixtures: invented here, the cause turned on and off
//!
//! One geometry, [`W`] x [`H`], half of a 2560x1440 screen. Textured browser
//! chrome over the top [`CHROME_END`] rows and a textured taskbar from
//! [`TASKBAR`], across the whole width. Between them the **site**, exactly
//! [`SITE`] on every pixel, and a page whose drawn **panels** are columns
//! [`PANELS_FIRST`] .. [`PANELS_END`]: textured, mostly dark ink (median near
//! 38, as `n02`'s edge column), so their edge does not carry a white tone on.
//! The left site is wider than `chrome_max_extent` (0.30) of the width, so
//! `content_box` cannot peel it as a strip.
//!
//! - [`Scene::Paper`], **cause on** (`n02`'s shape): [`PAPER_W`] columns of
//!   paper, exactly [`PAPER`] on every pixel, on each side of the panels,
//!   then the site. The page is [`PAGE_FIRST`] .. [`PAGE_END`]. Fails on
//!   `main`: the page column is the panels only, and the viewport declines
//!   beside them (the paper is 120 of the 1080 margin columns, so a row's
//!   share at the site's tone is 0.889, under `PAGE_LIKE` 0.90), so the crop
//!   also keeps the chrome and the taskbar.
//! - [`Scene::NoPaper`], **cause off** (the control): the same geometry with
//!   the paper strips replaced by site, so the panels meet the margin
//!   directly. The page column is the panels, today and after.
//! - [`Scene::SecondWindow`], **the hazard control** for the two trial rules
//!   MC-074 `## Context` rejected (they moved `f18`, `n06` and six more
//!   corpus crops): the control's page, then site, then a second window
//!   whose background is flat [`PAPER`] - exactly the paper's value, so luma
//!   alone cannot tell it from paper - holding a textured block (a video)
//!   **wider than the panels**, the flat bright background running on to the
//!   image's right edge. Under trial rule 1 the video, the widest run, widens
//!   across its flat bright flanks to the site margin, reads as "in the page
//!   margin" (MC-066) and is taken as the page; the crop leaves the reader.
//!   Today, and after, the crop is the panels.
//!
//!   The hazard page has no paper of its own, deliberately: with paper on
//!   both sides and a block wider than the panels, `main` already picks the
//!   block (neither candidate's column is flanked by margin on `main`, so
//!   MC-066 keeps the widest), and a control that is red today guards
//!   nothing. MC-074 `## Handoff: RED -> GREEN` has the measurement.
//!
//! The premise test measures every geometric claim above on the fixtures,
//! from the definitions, without the pipeline, so a fixture that drifts goes
//! red rather than making a case vacuous.

use cropper_core::flat::{central_band, page_column};
use cropper_core::{Luma, Rect, Tuning, detect};

// --- Settled constants, read out --------------------------------------------

/// `viewport::PAGE_LIKE`: the share of a row's margin pixels within
/// `uniform_tolerance` of the tone at which the row is page-like. Asserted
/// equal to the shipped value in the premise test.
const PAGE_LIKE: f64 = 0.90;

// --- The geometry, invented here --------------------------------------------

/// The fixture's width: half of 2560.
const W: u32 = 1280;
/// The fixture's height: half of 1440.
const H: u32 = 720;
/// The first row below the browser chrome (`n02`: 115, halved and rounded).
const CHROME_END: u32 = 58;
/// The taskbar's first row (`n02`: 1400, halved).
const TASKBAR: u32 = 700;

/// The page's first column, the paper's first: 400 of 1280, 0.3125 of the
/// width, past `chrome_max_extent`.
const PAGE_FIRST: u32 = 400;
/// The paper's width on each side (`n02`: 60 and 75; tens of columns).
const PAPER_W: u32 = 60;
/// The panels' first column.
const PANELS_FIRST: u32 = PAGE_FIRST + PAPER_W;
/// The panels' end, exclusive: 200 columns (`n02`: 398, halved).
const PANELS_END: u32 = 660;
/// The page's end, exclusive: the right paper is `PANELS_END .. PAGE_END`.
const PAGE_END: u32 = PANELS_END + PAPER_W;

/// The second window's first column, on [`Scene::SecondWindow`]: 140
/// columns of site past the page's end.
const SECOND_FIRST: u32 = 860;
/// The video's first column: 40 columns of flat bright background before it.
const VIDEO_FIRST: u32 = 900;
/// The video's end, exclusive: 250 columns, wider than the 200 of the
/// panels (`f18`: 716 against 517). Flat bright background from here to the
/// image's right edge.
const VIDEO_END: u32 = 1150;

/// The site margin's one value (`n02`: 11).
const SITE: u8 = 11;
/// The page paper's one value (`n02`: 255), and the second window's
/// background.
const PAPER: u8 = 255;

// --- The fixtures -----------------------------------------------------------

/// One scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    /// `n02`'s shape: panels on flat white paper, between a single-value site.
    Paper,
    /// The control: the paper replaced by site.
    NoPaper,
    /// The hazard control: [`Scene::NoPaper`]'s page beside a second window
    /// of flat [`PAPER`] holding a textured block wider than the panels.
    SecondWindow,
}

/// A well-mixed hash of a pixel position, so texture has no row or column
/// structure, and the same pixel has the same value in every scene.
fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

/// One pixel of chrome, taskbar or video: textured grey, 60 ..= 200.
fn texture(x: u32, y: u32) -> u8 {
    60 + (hash(x, y) % 141) as u8
}

/// One pixel of the drawn panels: dark ink (30 ..= 46) on seven pixels in
/// ten, light (150 ..= 230) on the rest, so a column's median is dark and its
/// spread far above `min_line_spread`.
fn panel(x: u32, y: u32) -> u8 {
    let h = hash(x, y);
    if h % 10 < 7 {
        30 + ((h >> 8) % 17) as u8
    } else {
        150 + ((h >> 8) % 81) as u8
    }
}

impl Scene {
    fn render(self) -> Luma {
        let mut data = Vec::with_capacity((W * H) as usize);
        for y in 0..H {
            for x in 0..W {
                let v = if !(CHROME_END..TASKBAR).contains(&y) {
                    texture(x, y)
                } else if (PANELS_FIRST..PANELS_END).contains(&x) {
                    panel(x, y)
                } else if (PAGE_FIRST..PAGE_END).contains(&x) {
                    match self {
                        Scene::Paper => PAPER,
                        Scene::NoPaper | Scene::SecondWindow => SITE,
                    }
                } else if self == Scene::SecondWindow && x >= SECOND_FIRST {
                    if (VIDEO_FIRST..VIDEO_END).contains(&x) {
                        texture(x, y)
                    } else {
                        PAPER
                    }
                } else {
                    SITE
                };
                data.push(v);
            }
        }
        Luma {
            width: W,
            height: H,
            data,
        }
    }
}

// --- Measuring, test-side ---------------------------------------------------

/// The whole fixture as a rect.
fn whole() -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: W,
        h: H,
    }
}

/// The central band of the whole fixture: the rows stage 3c measures a
/// column over, when nothing earlier moves the rect's rows.
fn band() -> Rect {
    central_band(whole(), &Tuning::default())
}

/// Column `x`'s pixels over the band.
fn column(img: &Luma, x: u32) -> Vec<u8> {
    let b = band();
    (b.y..b.y + b.h)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

/// The upper median, `sorted[n / 2]`.
fn median(values: &[u8]) -> u8 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
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

/// The runs of consecutive columns textured over the band, `(first, end)`
/// with `end` exclusive, in order.
fn textured_runs(img: &Luma, rule: f64) -> Vec<(u32, u32)> {
    let mut runs = Vec::new();
    let mut open = None;
    for x in 0..W {
        match (spread(&column(img, x)) >= rule, open) {
            (true, None) => open = Some(x),
            (false, Some(start)) => {
                runs.push((start, x));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(start) = open {
        runs.push((start, W));
    }
    runs
}

/// The share of row `y`'s pixels outside columns `first .. end` that lie
/// within `tol` of [`SITE`]: the viewport stage's per-row reading, with the
/// tone the premise test shows it reads.
fn row_share_beside(img: &Luma, y: u32, (first, end): (u32, u32), tol: u8) -> f64 {
    let near = (0..W)
        .filter(|&x| !(first..end).contains(&x))
        .filter(|&x| img.data[(y * W + x) as usize].abs_diff(SITE) <= tol)
        .count();
    near as f64 / f64::from(W - (end - first))
}

/// Both margins, as the story defines them: 0 (the default) and 3.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning::default(),
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
    ]
}

/// `detect`'s rect for `img` at `t`.
fn crop(img: &Luma, t: &Tuning) -> Rect {
    detect(img, t)
        .expect("the fixture is not one flat colour")
        .rect
}

/// `r` as `x,y wxh` and its end-exclusive columns and rows, for messages.
fn show(r: Rect) -> String {
    format!(
        "{},{} {}x{} (columns {}..{}, rows {}..{})",
        r.x,
        r.y,
        r.w,
        r.h,
        r.x,
        r.x + r.w,
        r.y,
        r.y + r.h
    )
}

// --- The fixtures are what they claim to be ---------------------------------

/// The premises every case below stands on, measured on the fixtures from the
/// definitions and not through the pipeline.
#[test]
fn the_fixtures_paper_site_panels_and_second_window_measure_as_claimed() {
    let t = Tuning::default();
    let tol = t.uniform_tolerance;
    let rule = f64::from(t.min_line_spread);
    assert!(
        (f64::from(cropper_core::viewport::PAGE_LIKE) - PAGE_LIKE).abs() < 1e-6,
        "PAGE_LIKE copied here must equal the shipped value"
    );
    let b = band();
    assert!(
        b.y >= CHROME_END && b.y + b.h <= TASKBAR,
        "the central band {b:?} must hold no chrome or taskbar row"
    );
    assert!(
        f64::from(PAGE_FIRST) / f64::from(W) > f64::from(t.chrome_max_extent),
        "the left site must be wider than chrome_max_extent, so content_box cannot peel it"
    );

    let paper = Scene::Paper.render();
    let no_paper = Scene::NoPaper.render();
    let second = Scene::SecondWindow.render();
    let viewport_rows = CHROME_END..TASKBAR;

    // The site and the paper: one exact value each, on every viewport row.
    let exact = |img: &Luma, x: u32, value: u8| {
        viewport_rows
            .clone()
            .all(|y| img.data[(y * W + x) as usize] == value)
    };
    for x in (0..PAGE_FIRST).chain(PAGE_END..W) {
        assert!(
            exact(&paper, x, SITE),
            "Paper: site column {x} must be {SITE}"
        );
        assert!(
            exact(&no_paper, x, SITE),
            "NoPaper: site column {x} must be {SITE}"
        );
    }
    for x in (PAGE_FIRST..PANELS_FIRST).chain(PANELS_END..PAGE_END) {
        assert!(
            exact(&paper, x, PAPER),
            "Paper: paper column {x} must be {PAPER}"
        );
        assert!(
            exact(&no_paper, x, SITE) && exact(&second, x, SITE),
            "NoPaper/SecondWindow: column {x} (paper in Paper) must be site {SITE}"
        );
    }
    for x in (SECOND_FIRST..VIDEO_FIRST).chain(VIDEO_END..W) {
        assert!(
            exact(&second, x, PAPER),
            "SecondWindow: the second window's background column {x} must be {PAPER}, the \
             paper's own value"
        );
    }
    for x in PAGE_END..SECOND_FIRST {
        assert!(
            exact(&second, x, SITE),
            "SecondWindow: column {x} between the page and the second window must be site"
        );
    }

    // The panels' edge columns: textured, with a dark median that neither
    // carries the paper's white on nor sits at the site's tone.
    for x in [PANELS_FIRST, PANELS_END - 1] {
        let c = column(&paper, x);
        let m = median(&c);
        assert!(
            spread(&c) >= rule,
            "panel edge column {x}: spread {:.2} must reach min_line_spread {rule}",
            spread(&c)
        );
        assert!(
            m.abs_diff(PAPER) > tol && m.abs_diff(SITE) > tol,
            "panel edge column {x}: median {m} must be more than {tol} from both the paper \
             ({PAPER}) and the site ({SITE})"
        );
    }

    // The textured runs over the band: the panels alone, and on the hazard
    // the video too, wider than the panels.
    assert_eq!(
        textured_runs(&paper, rule),
        vec![(PANELS_FIRST, PANELS_END)],
        "Paper: the only textured run over the band is the panels"
    );
    assert_eq!(
        textured_runs(&no_paper, rule),
        vec![(PANELS_FIRST, PANELS_END)],
        "NoPaper: the only textured run over the band is the panels"
    );
    assert_eq!(
        textured_runs(&second, rule),
        vec![(PANELS_FIRST, PANELS_END), (VIDEO_FIRST, VIDEO_END)],
        "SecondWindow: the panels and the video are the textured runs over the band"
    );
    const {
        assert!(
            VIDEO_END - VIDEO_FIRST > PANELS_END - PANELS_FIRST,
            "the video must be wider than the panels, so it is the first candidate"
        );
    }

    // The viewport's reading on Paper: beside the panels a viewport row is
    // under PAGE_LIKE (the paper is off the site's tone), beside the whole
    // page it is wholly at the tone.
    let mid = (CHROME_END + TASKBAR) / 2;
    let beside_panels = row_share_beside(&paper, mid, (PANELS_FIRST, PANELS_END), tol);
    let beside_page = row_share_beside(&paper, mid, (PAGE_FIRST, PAGE_END), tol);
    assert!(
        beside_panels < PAGE_LIKE,
        "Paper: a viewport row beside the panels must be under PAGE_LIKE: share \
         {beside_panels:.4}"
    );
    assert!(
        (beside_page - 1.0).abs() < 1e-12,
        "Paper: a viewport row beside the whole page must be wholly at the site's tone: share \
         {beside_page:.4}"
    );
}

// --- n02's shape ------------------------------------------------------------

/// AC-3, the cause: stage 3c's page column, read over the whole image, is the
/// panels **and** the flat white paper they sit on, up to the site margin of
/// one exact value. Fails on `main`: the column is the panels only.
#[test]
fn the_page_column_of_panels_on_flat_white_paper_spans_the_paper_to_the_site_margin() {
    let img = Scene::Paper.render();
    let got = page_column(&img, whole(), &Tuning::default());
    let want = Rect {
        x: PAGE_FIRST,
        w: PAGE_END - PAGE_FIRST,
        ..whole()
    };
    assert_eq!(
        got,
        want,
        "MC-074 AC-3 (n02's shape): the page column must be the paper's full width, \
         {}, not {} - the panels are {PANELS_FIRST}..{PANELS_END} and the {PAPER_W} columns \
         of flat {PAPER} paper on each side are page (the user's ruling, \"Box stands\")",
        show(want),
        show(got)
    );
}

/// AC-3, the cause through the pipeline: at both margins the crop is the
/// paper's full width plus the margin, and the viewport's rows - no chrome,
/// no taskbar - since beside the right column the viewport is located. Fails
/// on `main`: the crop is the panels, over the whole image height.
#[test]
fn panels_on_flat_white_paper_are_cropped_to_the_whole_page_and_its_viewport_at_both_margins() {
    let img = Scene::Paper.render();
    let wrong: Vec<String> = both_margins()
        .iter()
        .filter_map(|t| {
            let m = t.margin_px;
            let want = Rect {
                x: PAGE_FIRST - m,
                y: CHROME_END,
                w: PAGE_END - PAGE_FIRST + 2 * m,
                h: TASKBAR - CHROME_END,
            };
            let got = crop(&img, t);
            (got != want).then(|| format!("margin_px {m}: crop {}, want {}", show(got), show(want)))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "MC-074 AC-3 (n02's shape): the crop must be the page, panels and paper \
         ({PAGE_FIRST}..{PAGE_END}), over the viewport's rows ({CHROME_END}..{TASKBAR}):\n{}",
        wrong.join("\n")
    );
}

// --- The control: no paper --------------------------------------------------

/// AC-3's control: the same geometry with the paper strips replaced by site,
/// so the panels meet the margin directly. The page column, and the crop at
/// both margins, are the panels and the viewport's rows. Passes on `main`
/// and must keep passing: with the paper gone the crop is already right, so
/// what cuts `n02`'s page is the paper.
#[test]
fn control_panels_meeting_the_site_margin_directly_are_cropped_to_the_panels_at_both_margins() {
    let img = Scene::NoPaper.render();
    let mut wrong = Vec::new();
    let column = page_column(&img, whole(), &Tuning::default());
    let want_column = Rect {
        x: PANELS_FIRST,
        w: PANELS_END - PANELS_FIRST,
        ..whole()
    };
    if column != want_column {
        wrong.push(format!(
            "page column {}, want {}",
            show(column),
            show(want_column)
        ));
    }
    for t in both_margins() {
        let m = t.margin_px;
        let want = Rect {
            x: PANELS_FIRST - m,
            y: CHROME_END,
            w: PANELS_END - PANELS_FIRST + 2 * m,
            h: TASKBAR - CHROME_END,
        };
        let got = crop(&img, &t);
        if got != want {
            wrong.push(format!(
                "margin_px {m}: crop {}, want {}",
                show(got),
                show(want)
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-074 AC-3's control (no paper): panels meeting the site margin are the page, \
         exactly:\n{}",
        wrong.join("\n")
    );
}

// --- The hazard control: a flat bright region that is not paper -------------

/// AC-3's hazard control: beside the page, past the site margin, a second
/// window whose background is flat and exactly the paper's value, holding a
/// textured block wider than the panels, and running to the image's right
/// edge. It is not page paper and must not be annexed: at both margins the
/// crop's columns are the panels', exactly. Passes on `main`; fails under
/// either trial rule MC-074 `## Context` rejected.
#[test]
fn hazard_a_second_windows_flat_bright_background_past_the_site_margin_is_never_annexed() {
    let img = Scene::SecondWindow.render();
    let mut wrong = Vec::new();
    let column = page_column(&img, whole(), &Tuning::default());
    if (column.x, column.w) != (PANELS_FIRST, PANELS_END - PANELS_FIRST) {
        wrong.push(format!(
            "page column {}, want columns {PANELS_FIRST}..{PANELS_END}",
            show(column)
        ));
    }
    for t in both_margins() {
        let m = t.margin_px;
        let got = crop(&img, &t);
        let want = (PANELS_FIRST - m, PANELS_END + m);
        if (got.x, got.x + got.w) != want {
            wrong.push(format!(
                "margin_px {m}: crop {}, want columns {}..{}{}",
                show(got),
                want.0,
                want.1,
                if got.x + got.w > SECOND_FIRST {
                    format!(" - it keeps the second window (from column {SECOND_FIRST})")
                } else {
                    String::new()
                }
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-074 AC-3's hazard control: a flat {PAPER} region past the site margin is a \
         second window, not page paper, and the page is the panels \
         {PANELS_FIRST}..{PANELS_END}:\n{}",
        wrong.join("\n")
    );
}
