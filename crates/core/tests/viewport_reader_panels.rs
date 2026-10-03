//! MC-076 AC-4: the viewport stage beside **two flat reader panels of another
//! tone hugging the page** - `n13`'s cause, `Screenshot (2507).png`, and the
//! two `2025-08-05` WebPs' - reproduced on a generated screen and seen through
//! [`locate`] and [`detect`]. This is what the required `unit` gate sees of
//! the story; the corpus half is `crates/engine/tests/corpus_furniture_rows.rs`
//! (AC-1), `corpus_viewport.rs` (AC-2) and the lists in
//! `corpus_viewport_stage.rs`, `corpus_tuning_crops_unmoved.rs` and
//! `corpus_sides.rs` (AC-1 to AC-3).
//!
//! # The cause, as the Lead PO measured it (MC-076 `## Context`, "Measured")
//!
//! On `n13` the page column is exact, `977..1577`, and on the viewport rows
//! the pixels beside it are, outward from the page, a flat **reader panel**
//! of another tone (medians 34 and 42) 105 columns wide on each side, then the
//! site's flat background (tone 22) to the image's edges. A panel pixel is not
//! within `uniform_tolerance` of the tone, so the whole-margin share is the
//! site background's width over the margin's: 0.889 on **every** viewport
//! row, under `PAGE_LIKE`. MC-048's reading finds no run; MC-075's reading
//! after a decline needs a reader window of `MIN_WINDOW_MARGIN` columns on each
//! side, and the window here stops at the panel, 0 columns from the page. So
//! `locate` returns `None` and the crop keeps the browser toolbar. The WebPs
//! are the same shape with a sliver of 0 to 4 background-tone columns between
//! page and panel (`00_11_13.webp`: 1 on the left, 4 on the right; share
//! 0.862), and the user ruled they move too (MC-076 `## Notes`).
//!
//! # The fixture: invented here
//!
//! No oracle exists (the story partitions AC-4 as oracle-free). [`Scene`]
//! draws, top to bottom, at [`HEIGHT`] rows:
//!
//! - **the browser bar**, rows `0 ..` [`BAR_END`], seeded noise across the
//!   whole width;
//! - **the viewport**, [`BAR_END`] `..` [`TASKBAR`]: left to right, the
//!   scene's columns ([`Fill`]): flat site background at [`SITE_TONE`], flat
//!   reader panels at [`LEFT_PANEL_TONE`] and [`RIGHT_PANEL_TONE`], and the
//!   **page column itself** - MC-027's page fixture's columns 45..=254, the
//!   ones MC-053 makes the page column, with none of its page-tone margin, so
//!   a panel hugs the page with no sliver between;
//! - **the taskbar**, [`TASKBAR`] `..` [`HEIGHT`], noise across the whole
//!   width.
//!
//! The bars lie outside `central_band` of the rows (48..192 of 240), so the
//! earlier stages see only the viewport's columns, and the premise test checks
//! that they hand the stage exactly the page's columns over every row.
//!
//! # The scenes
//!
//! - [`Scene::Panels`], **the reproduction**: 160 site columns, a 20-column
//!   panel, the page, a 20-column panel, 160 site columns. Share 320 / 360 =
//!   0.889 on every viewport row: `n13`'s, to the third place.
//! - [`Scene::Sliver`], **the reproduction with a sliver**: as `Panels` with
//!   24-column panels, and 1 site-tone column between the page and the left
//!   panel and 4 between the page and the right one - `00_11_13.webp`'s 1
//!   and 4. Share 325 / 373 = 0.871 (`00_11_13.webp`: 0.862). The user ruled
//!   the WebPs move, so this must be located too.
//! - [`Scene::Narrow`], **control (a)**: 12-column panels, share 320 / 344 =
//!   0.930, over `PAGE_LIKE` - `(2486)`'s kind (0.933). The stage locates the
//!   page's rows today, and must after.
//! - [`Scene::OneSided`], **control (b)**: a 40-column panel on the left
//!   only. On the right, 60 site columns and then 100 columns of **site
//!   content** - flat site background beside the upper half of the page,
//!   textured beside the lower half ([`CONTENT_FROM`] down). The whole margin
//!   is 0.875 on the upper rows and 0.563 on the lower: the stage declines
//!   today and the crop keeps the whole page. A stage that, after a decline,
//!   leaves out a band hugging the page on **either** side - the trial rule 1
//!   that broke `tests/detect.rs`'s one-sided case - leaves out the panel,
//!   reads the rest, finds the upper rows page-like and the lower ones not,
//!   and cuts the page's lower art rows off as if they were taskbar.
//!   [`a_stage_that_drops_a_band_hugging_either_side_fixes_the_panels_and_cuts_the_one_sided_page`]
//!   shows that with [`drop_any_hugging_band`], a test-local stand-in written
//!   from that description (not the Lead PO's trial code, which this file has
//!   never seen). Without it, (b) would be a control nothing plausible fails.
//!
//! The reproductions' panels are 20 and 24 columns, over `MIN_RUN` (16), so
//! a fix that asks a panel to be as wide as `MIN_WINDOW_MARGIN` before it is
//! left out is not ruled out by the fixture (`n13`'s are 105 columns, the
//! WebPs' 107 to 133); the fade columns `tests/viewport.rs`'s
//! `the_stage_reads_uniform_tolerance_from_the_tuning` turns non-background at
//! `uniform_tolerance` 0 are 5.
//!
//! MC-075's controls (`tests/viewport_textured_second_window.rs`) are the
//! third control the story names, and stay in their own file, unchanged.
//!
//! Every scene is rendered as drawn and mirrored left to right ([`Side`]),
//! so a fix that reads one side differently from the other is seen.
//!
//! Every share and tone this file relies on is asserted, not assumed, in
//! [`the_whole_margin_shares_are_the_ones_the_lead_po_measured_on_n13_and_2486`]
//! and the premise test; the measured values are in MC-076's handoff.

mod common;

use common::{Rng, VIEW_NOISE_HI, VIEW_NOISE_LO, page_core_rect, page_pixel};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::viewport::{MIN_RUN, PAGE_LIKE, Viewport, locate};
use cropper_core::{Luma, Rect, Tuning, detect};

// --- The geometry, invented here --------------------------------------------

/// The first row below the browser bar (`n13`: 133).
const BAR_END: u32 = 40;
/// The first row of the taskbar (`n13`: 1392).
const TASKBAR: u32 = 200;
/// The image's height (`n13`: 1440).
const HEIGHT: u32 = 240;

/// The page column's columns in MC-027's page fixture: `45 ..= 254`, where
/// MC-053 puts its edges at `uniform_tolerance` 10 (`common::page_first_art_column`).
const PAGE_FIRST: u32 = 45;
/// One past the page column's last fixture column.
const PAGE_END: u32 = 255;
/// The page column's width.
const PAGE_COLUMN_W: u32 = PAGE_END - PAGE_FIRST;

/// The site's flat background beside the panels (`n13`: 22).
const SITE_TONE: u8 = 22;
/// The left reader panel's flat tone, 12 off the site's (`n13`: 34).
const LEFT_PANEL_TONE: u8 = 34;
/// The right reader panel's flat tone, 20 off the site's (`n13`: 42).
const RIGHT_PANEL_TONE: u8 = 42;

/// Control (b): the first row of the viewport on which the right side's site
/// content is textured; above it, it is flat site background.
const CONTENT_FROM: u32 = 120;

/// The seed the fixtures' noise is drawn from. Not MC-048's, MC-052's or
/// MC-075's, so nothing here can pass by sharing their bytes.
const SEED: u32 = 76;

/// What fills a run of columns on the viewport's rows. On the bar and taskbar
/// rows every column is noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fill {
    /// Flat site background at [`SITE_TONE`].
    Site,
    /// A flat reader panel at this tone.
    Panel(u8),
    /// The page column: the page fixture's columns [`PAGE_FIRST`]`..`[`PAGE_END`].
    Page,
    /// Control (b)'s site content: flat site background above
    /// [`CONTENT_FROM`], noise from it down.
    Content,
}

/// Which way round a scene is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// As [`Scene::columns`] lists it.
    AsDrawn,
    /// Mirrored left to right.
    Mirrored,
}

const SIDES: [Side; 2] = [Side::AsDrawn, Side::Mirrored];

/// The four screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    /// The reproduction: 20-column panels hugging the page, share 0.889.
    Panels,
    /// The reproduction with `00_11_13.webp`'s slivers: 24-column panels, 1
    /// and 4 site-tone columns between them and the page, share 0.871.
    Sliver,
    /// Control (a): 12-column panels, share 0.930.
    Narrow,
    /// Control (b): one 40-column panel, on the left only.
    OneSided,
}

const REPRODUCTIONS: [Scene; 2] = [Scene::Panels, Scene::Sliver];

impl Scene {
    /// The viewport rows' columns, left to right, as drawn.
    fn columns(self) -> Vec<(u32, Fill)> {
        let left = Fill::Panel(LEFT_PANEL_TONE);
        let right = Fill::Panel(RIGHT_PANEL_TONE);
        match self {
            Scene::Panels => vec![
                (160, Fill::Site),
                (20, left),
                (PAGE_COLUMN_W, Fill::Page),
                (20, right),
                (160, Fill::Site),
            ],
            Scene::Sliver => vec![
                (160, Fill::Site),
                (24, left),
                (1, Fill::Site),
                (PAGE_COLUMN_W, Fill::Page),
                (4, Fill::Site),
                (24, right),
                (160, Fill::Site),
            ],
            Scene::Narrow => vec![
                (160, Fill::Site),
                (12, left),
                (PAGE_COLUMN_W, Fill::Page),
                (12, right),
                (160, Fill::Site),
            ],
            Scene::OneSided => vec![
                (120, Fill::Site),
                (40, left),
                (PAGE_COLUMN_W, Fill::Page),
                (60, Fill::Site),
                (100, Fill::Content),
            ],
        }
    }

    fn width(self) -> u32 {
        self.columns().iter().map(|&(w, _)| w).sum()
    }

    /// What fills image column `x` (as drawn), and for the page its fixture
    /// column.
    fn fill_at(self, x: u32) -> (Fill, u32) {
        let mut start = 0;
        for (w, fill) in self.columns() {
            if x < start + w {
                return (fill, PAGE_FIRST + (x - start));
            }
            start += w;
        }
        unreachable!("column {x} is outside {self:?}")
    }

    /// The first image column of the page, as drawn.
    fn page_start(self) -> u32 {
        let mut start = 0;
        for (w, fill) in self.columns() {
            if fill == Fill::Page {
                return start;
            }
            start += w;
        }
        unreachable!("{self:?} has a page")
    }

    /// The screenshot.
    fn render(self, side: Side) -> Luma {
        let width = self.width();
        let mut rng = Rng::new(SEED);
        let mut data = vec![0u8; (width * HEIGHT) as usize];
        for y in 0..HEIGHT {
            for x in 0..width {
                // Drawn for every pixel, so the noise is independent of the
                // layout and the same arguments give the same bytes.
                let noise = rng.between(VIEW_NOISE_LO, VIEW_NOISE_HI);
                let v = if (BAR_END..TASKBAR).contains(&y) {
                    match self.fill_at(x) {
                        (Fill::Site, _) => SITE_TONE,
                        (Fill::Panel(tone), _) => tone,
                        (Fill::Page, local) => page_pixel(local, y),
                        (Fill::Content, _) if y >= CONTENT_FROM => noise,
                        (Fill::Content, _) => SITE_TONE,
                    }
                } else {
                    noise
                };
                data[(y * width + mirror(side, width, x)) as usize] = v;
            }
        }
        Luma {
            width,
            height: HEIGHT,
            data,
        }
    }

    /// The page column `detect` must hand the viewport stage: the page's
    /// columns, every row.
    fn page_column(self, side: Side) -> Rect {
        let x = match side {
            Side::AsDrawn => self.page_start(),
            Side::Mirrored => self.width() - self.page_start() - PAGE_COLUMN_W,
        };
        Rect {
            x,
            y: 0,
            w: PAGE_COLUMN_W,
            h: HEIGHT,
        }
    }

    /// The page art: MC-027's full-amplitude core columns, over every
    /// viewport row.
    fn art(self, side: Side) -> Rect {
        let core = page_core_rect();
        let drawn = self.page_start() + core.x - PAGE_FIRST;
        let x = match side {
            Side::AsDrawn => drawn,
            Side::Mirrored => self.width() - drawn - core.w,
        };
        Rect {
            x,
            y: BAR_END,
            w: core.w,
            h: TASKBAR - BAR_END,
        }
    }
}

/// Image column of drawn column `x`.
fn mirror(side: Side, width: u32, x: u32) -> u32 {
    match side {
        Side::AsDrawn => x,
        Side::Mirrored => width - 1 - x,
    }
}

/// The page's rows, as the stage must return them.
const PAGE_ROWS: Viewport = Viewport {
    top: BAR_END,
    bottom: TASKBAR,
};

/// Both margins, as the story defines them: 0 (the default) and 3.
fn margins() -> [Tuning; 2] {
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

/// The page column `detect` hands the viewport stage: its first five stages,
/// composed from their public functions in `decide.rs`'s order, before the
/// margin.
fn page_column_of(img: &Luma, t: &Tuning) -> Rect {
    let first = trim_uniform(img, t).expect("not one flat colour");
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t).expect("not one flat colour");
    page_column(img, textured_box(img, second, t), t)
}

/// The rect `detect` returns for `img` at `t`.
fn crop(img: &Luma, t: &Tuning) -> Rect {
    detect(img, t)
        .expect("a screen is not one flat colour")
        .rect
}

/// Whether `outer` contains `inner` entirely.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

// --- MC-048's instrument, ported for measurement ------------------------------

/// The page background tone beside `column`, as `viewport.rs` step 2 reads
/// it: each row's upper-median margin value, binned into 32 bins of 8; the
/// modal bin, then the modal exact value in it, ties to the higher. Ported
/// because the crate's own is private; the premise test checks the port gives
/// [`SITE_TONE`].
fn tone(img: &Luma, column: Rect) -> u8 {
    let width = img.width as usize;
    let (left_end, right_start) = (column.x as usize, (column.x + column.w) as usize);
    let mut bins = [0usize; 32];
    let mut exact = [0usize; 256];
    for row in img.data.chunks_exact(width) {
        let mut m: Vec<u8> = row[..left_end]
            .iter()
            .chain(&row[right_start..])
            .copied()
            .collect();
        m.sort_unstable();
        let median = m[m.len() / 2];
        bins[usize::from(median / 8)] += 1;
        exact[usize::from(median)] += 1;
    }
    let last_max = |c: &[usize]| (0..c.len()).fold(0, |b, i| if c[i] >= c[b] { i } else { b });
    let best = last_max(&bins);
    u8::try_from(best * 8 + last_max(&exact[best * 8..best * 8 + 8])).expect("a u8 tone")
}

/// The share of row `y`'s pixels in `cols` within `uniform_tolerance` of
/// `tone`.
fn share(img: &Luma, y: u32, cols: &[u32], tone: u8, t: &Tuning) -> f32 {
    let near = cols
        .iter()
        .filter(|&&x| img.data[(y * img.width + x) as usize].abs_diff(tone) <= t.uniform_tolerance)
        .count();
    near as f32 / cols.len() as f32
}

/// Every image column outside `column`: MC-048's whole margin.
fn whole_margin(img: &Luma, column: Rect) -> Vec<u32> {
    (0..column.x)
        .chain(column.x + column.w..img.width)
        .collect()
}

// --- The premise ----------------------------------------------------------------

/// The fixture's premise: on every scene and both sides, the earlier stages
/// peel nothing, and hand the stage exactly the page's columns over every row,
/// so the panels hug the page column itself, as on `n13` (`977..1577`, the
/// panels from 872 and from 1577). And the ported tone is the site's. Without
/// it, a crop outside the page's rows could be an earlier stage's doing and
/// say nothing about the viewport stage.
#[test]
fn the_page_column_reaching_the_viewport_stage_is_the_page_with_the_panels_beside_it() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for scene in [Scene::Panels, Scene::Sliver, Scene::Narrow, Scene::OneSided] {
        for side in SIDES {
            let img = scene.render(side);
            let first = trim_uniform(&img, &t).expect("not uniform");
            let found = content_box(&img, first, &t);
            let column = page_column_of(&img, &t);
            let want = scene.page_column(side);
            if !found.removed.is_empty() {
                bad.push(format!(
                    "{scene:?} {side:?}: content_box must peel nothing; peeled {:?}",
                    found.removed
                ));
            }
            if column != want {
                bad.push(format!(
                    "{scene:?} {side:?}: the page column must be the page's columns over \
                     every row, {want:?}; got {column:?}"
                ));
            }
            let tone = tone(&img, want);
            if tone != SITE_TONE {
                bad.push(format!(
                    "{scene:?} {side:?}: the margin's tone must be the site's, {SITE_TONE}; \
                     got {tone}"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The cause, measured on the fixture as the Lead PO measured it on `n13`:
/// on every viewport row the whole-margin share is under `PAGE_LIKE` on the
/// reproductions (`n13`: 0.889; `00_11_13.webp`: 0.862) and at or over it on
/// control (a) (`(2486)`: 0.933); on the bar and taskbar rows it is far
/// under on every scene (`n13`: 0.000 to 0.010 on rows 0..130). Control (b)
/// is under on every viewport row too, so the stage declines there today.
///
/// Exact, not a range: the panels and site are flat, so each scene's share is
/// one value on every viewport row (two on control (b), above and below
/// [`CONTENT_FROM`]), asserted here to the share the geometry gives. This
/// holds today and after: it is the fixture, not the fix. Printed under
/// `--nocapture`; the values are in MC-076's handoff.
#[test]
fn the_whole_margin_shares_are_the_ones_the_lead_po_measured_on_n13_and_2486() {
    let t = Tuning::default();
    // (scene, share on the viewport rows above CONTENT_FROM, below it)
    let want: [(Scene, f32, f32); 4] = [
        (Scene::Panels, 320.0 / 360.0, 320.0 / 360.0),
        (Scene::Sliver, 325.0 / 373.0, 325.0 / 373.0),
        (Scene::Narrow, 320.0 / 344.0, 320.0 / 344.0),
        (Scene::OneSided, 280.0 / 320.0, 180.0 / 320.0),
    ];
    let mut bad = Vec::new();
    for (scene, upper, lower) in want {
        for side in SIDES {
            let img = scene.render(side);
            let column = scene.page_column(side);
            let whole = whole_margin(&img, column);
            let mut furniture_hi = f32::MIN;
            for y in 0..HEIGHT {
                let s = share(&img, y, &whole, SITE_TONE, &t);
                if !(BAR_END..TASKBAR).contains(&y) {
                    furniture_hi = furniture_hi.max(s);
                    continue;
                }
                let expected = if y < CONTENT_FROM { upper } else { lower };
                if (s - expected).abs() > 1e-6 {
                    bad.push(format!(
                        "{scene:?} {side:?} row {y}: whole-margin share {s:.4}, the geometry \
                         gives {expected:.4}"
                    ));
                }
            }
            if furniture_hi >= 0.1 {
                bad.push(format!(
                    "{scene:?} {side:?}: a bar or taskbar row reads {furniture_hi:.3} over the \
                     whole margin; it must be far under PAGE_LIKE"
                ));
            }
            println!(
                "{scene:?} {side:?}: {} margin columns; viewport rows {upper:.3} (above row \
                 {CONTENT_FROM}) and {lower:.3} (from it); bar and taskbar rows at most \
                 {furniture_hi:.3}",
                whole.len()
            );
        }
        let located_today = upper >= PAGE_LIKE;
        if located_today != (scene == Scene::Narrow) {
            bad.push(format!(
                "{scene:?}: share {upper:.3} must be {} PAGE_LIKE {PAGE_LIKE}",
                if scene == Scene::Narrow {
                    "at or over"
                } else {
                    "under"
                }
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- AC-4: the reproduction -------------------------------------------------------

/// AC-4, the stage: beside two flat reader panels of another tone hugging the
/// page - with no sliver between (`n13`) or with 1 and 4 site-tone columns
/// (`00_11_13.webp`) - the viewport is exactly the page's rows, the first row
/// below the browser bar to the first row of the taskbar.
///
/// On `main` (`1d7a921`) `locate` returns `None`: the whole-margin share is
/// under `PAGE_LIKE` on every row, and the reader's window is 0 to 4 columns
/// a side, under `MIN_WINDOW_MARGIN`, so MC-075's reading does not run.
#[test]
fn beside_reader_panels_hugging_the_page_the_viewport_is_the_pages_rows() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for scene in REPRODUCTIONS {
        for side in SIDES {
            let img = scene.render(side);
            let got = locate(&img, page_column_of(&img, &t), &t);
            if got != Some(PAGE_ROWS) {
                bad.push(format!(
                    "{scene:?} {side:?}: the viewport must be the page's rows {PAGE_ROWS:?}; \
                     got {got:?}"
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "MC-076 AC-4: beside flat reader panels of another tone hugging the page, the \
         stage must find the viewport.\n{}",
        bad.join("\n")
    );
}

/// AC-4, the crop: at both margins `detect` keeps no row of the browser bar or
/// the taskbar - every crop row lies in the page's rows - and keeps the whole
/// page art.
///
/// On `main` the crop keeps the full height, rows 0..240.
#[test]
fn beside_reader_panels_hugging_the_page_the_crop_drops_the_browser_bar_and_taskbar() {
    let mut bad = Vec::new();
    for scene in REPRODUCTIONS {
        for side in SIDES {
            let img = scene.render(side);
            let art = scene.art(side);
            for t in margins() {
                let got = crop(&img, &t);
                let tag = format!("{scene:?} {side:?}, margin_px {}", t.margin_px);
                if got.y < BAR_END {
                    bad.push(format!(
                        "{tag}: the crop {got:?} keeps {} rows of the browser bar (rows \
                         0..{BAR_END})",
                        BAR_END - got.y
                    ));
                }
                if got.y + got.h > TASKBAR {
                    bad.push(format!(
                        "{tag}: the crop {got:?} keeps {} rows of the taskbar (rows \
                         {TASKBAR}..{HEIGHT})",
                        got.y + got.h - TASKBAR
                    ));
                }
                if !contains(got, art) {
                    bad.push(format!("{tag}: the crop {got:?} cuts the page art {art:?}"));
                }
            }
        }
    }
    assert!(bad.is_empty(), "MC-076 AC-4:\n{}", bad.join("\n"));
}

// --- AC-4, control (a): narrower panels -------------------------------------------

/// Control (a): the same screen with 12-column panels, whose share, 0.930, is
/// over `PAGE_LIKE` - `(2486)`'s kind. The stage locates the page's rows and
/// the crop is exactly those rows at both margins, holding the art. Holds on
/// `main` and must hold after.
#[test]
fn beside_narrower_panels_the_stage_locates_the_same_rows_as_before() {
    let mut bad = Vec::new();
    for side in SIDES {
        let img = Scene::Narrow.render(side);
        let art = Scene::Narrow.art(side);
        let t = Tuning::default();
        let got = locate(&img, page_column_of(&img, &t), &t);
        if got != Some(PAGE_ROWS) {
            bad.push(format!(
                "{side:?}: the viewport must be {PAGE_ROWS:?}; got {got:?}"
            ));
        }
        for t in margins() {
            let rect = crop(&img, &t);
            if (rect.y, rect.y + rect.h) != (BAR_END, TASKBAR) || !contains(rect, art) {
                bad.push(format!(
                    "{side:?}, margin_px {}: the crop {rect:?} must be rows \
                     {BAR_END}..{TASKBAR} and hold the art {art:?}",
                    t.margin_px
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "MC-076 AC-4 control (a), narrower panels:\n{}",
        bad.join("\n")
    );
}

// --- AC-4, control (b): a panel on one side only ------------------------------------

/// Control (b): a panel on the left only, and on the right site content
/// textured beside the lower half of the page. The stage declines, or never
/// cuts a row of the page's art; at both margins the crop keeps the whole
/// page. Holds on `main` (the stage declines) and must hold after.
#[test]
fn a_panel_on_one_side_only_never_costs_the_page_a_row_of_its_art() {
    let mut bad = Vec::new();
    for side in SIDES {
        let img = Scene::OneSided.render(side);
        let art = Scene::OneSided.art(side);
        let t = Tuning::default();
        let column = page_column_of(&img, &t);
        let got = locate(&img, column, &t);
        let column_end = column.y + column.h;
        let cuts_art = got.is_some_and(|v| {
            // A viewport clear of the column's rows is a decline to `detect`.
            let overlaps = v.top < column_end && v.bottom > column.y;
            overlaps && (v.top > art.y || v.bottom < art.y + art.h)
        });
        if cuts_art {
            bad.push(format!(
                "{side:?}: the viewport {got:?} cuts the page's art rows {}..{}",
                art.y,
                art.y + art.h
            ));
        }
        for t in margins() {
            let rect = crop(&img, &t);
            if !contains(rect, art) {
                bad.push(format!(
                    "{side:?}, margin_px {}: the crop {rect:?} cuts the page art {art:?}",
                    t.margin_px
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "MC-076 AC-4 control (b), a panel on one side only:\n{}",
        bad.join("\n")
    );
}

// --- Control (b)'s control: the fix it rules out --------------------------------

/// A stand-in viewport stage, **not** the product's: wherever [`locate`] finds
/// a viewport this returns exactly that. Where it **declines**, it leaves out
/// any band of non-background columns hugging the page column on **either**
/// side - a column is background when a majority of its pixels lie within
/// `uniform_tolerance` of the tone, as MC-052's window reads it - re-reads
/// the whole margin without those bands, takes the outer runs of [`MIN_RUN`]
/// page-like rows, and keeps MC-054's evidence-of-chrome check on the
/// re-read margin. That is the rule "after a decline, drop any non-background
/// band hugging the page on either side and re-read the margin", written here
/// from that description.
fn drop_any_hugging_band(img: &Luma, column: Rect, t: &Tuning) -> Option<Viewport> {
    if let Some(view) = locate(img, column, t) {
        return Some(view);
    }
    let width = img.width as usize;
    let rows = img.height as usize;
    let left_end = column.x as usize;
    let right_start = (column.x + column.w) as usize;
    if left_end == 0 && right_start == width {
        return None;
    }
    let tone = tone(img, column);
    let near = |v: u8| v.abs_diff(tone) <= t.uniform_tolerance;
    let row = |y: usize| &img.data[y * width..(y + 1) * width];
    let background = |x: usize| (0..rows).filter(|&y| near(row(y)[x])).count() * 2 > rows;
    // The band hugging each side: from the page column outward while the
    // column is not background.
    let lo = (0..left_end)
        .rev()
        .find(|&x| background(x))
        .map_or(0, |x| x + 1);
    let hi = (right_start..width)
        .find(|&x| background(x))
        .unwrap_or(width);
    let kept: Vec<usize> = (0..lo).chain(hi..width).collect();
    if kept.is_empty() {
        return None;
    }
    let page_like: Vec<bool> = (0..rows)
        .map(|y| {
            let n = kept.iter().filter(|&&x| near(row(y)[x])).count();
            n as f32 / kept.len() as f32 >= PAGE_LIKE
        })
        .collect();
    let (mut top, mut bottom) = outer_runs(&page_like)?;
    let (first, end) = (column.y as usize, (column.y + column.h) as usize);
    if top > first && !has_run(page_like[first..top].iter().map(|&p| !p)) {
        top = 0;
    }
    if bottom < end && !has_run(page_like[bottom..end.min(rows)].iter().map(|&p| !p)) {
        bottom = rows;
    }
    Some(Viewport {
        top: u32::try_from(top).ok()?,
        bottom: u32::try_from(bottom).ok()?,
    })
}

/// From the start of the first run of at least [`MIN_RUN`] `true`s to the end
/// of the last one; `None` when there is no such run.
fn outer_runs(page_like: &[bool]) -> Option<(usize, usize)> {
    let (mut first, mut last) = (None, None);
    let mut y = 0;
    while y < page_like.len() {
        let start = y;
        while y < page_like.len() && page_like[y] {
            y += 1;
        }
        if y - start >= MIN_RUN {
            first.get_or_insert(start);
            last = Some(y);
        }
        y += 1;
    }
    Some((first?, last?))
}

/// Whether `rows` holds a run of at least [`MIN_RUN`] `true`s.
fn has_run(rows: impl Iterator<Item = bool>) -> bool {
    let mut run = 0;
    for row in rows {
        run = if row { run + 1 } else { 0 };
        if run >= MIN_RUN {
            return true;
        }
    }
    false
}

/// Control (b) is not vacuous. The stand-in [`drop_any_hugging_band`] is a
/// real candidate fix - it finds the page's rows on `n13`'s reproduction, and
/// it leaves control (a) as it is - and on control (b) it leaves out the one
/// panel, reads the upper rows as page and the lower ones (beside the site
/// content) as not, and returns rows [`BAR_END`]`..`[`CONTENT_FROM`], cutting
/// every art row below. So (b) is what fails that fix.
///
/// The stand-in's viewport is checked directly: `detect` clamps the crop's
/// rows to the viewport, so a viewport cutting the art is a crop cutting it.
#[test]
fn a_stage_that_drops_a_band_hugging_either_side_fixes_the_panels_and_cuts_the_one_sided_page() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for side in SIDES {
        for scene in [Scene::Panels, Scene::Narrow] {
            let img = scene.render(side);
            let got = drop_any_hugging_band(&img, page_column_of(&img, &t), &t);
            if got != Some(PAGE_ROWS) {
                bad.push(format!(
                    "{scene:?} {side:?}: the stand-in must find the page's rows \
                     {PAGE_ROWS:?}; got {got:?}"
                ));
            }
        }
        let img = Scene::OneSided.render(side);
        let got = drop_any_hugging_band(&img, page_column_of(&img, &t), &t);
        let want = Some(Viewport {
            top: BAR_END,
            bottom: CONTENT_FROM,
        });
        if got != want {
            bad.push(format!(
                "OneSided {side:?}: the stand-in must read only the rows beside flat \
                 background, {want:?}, and so cut the page's art rows \
                 {CONTENT_FROM}..{TASKBAR}; got {got:?}"
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
