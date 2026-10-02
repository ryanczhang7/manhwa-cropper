//! MC-075 AC-5: the viewport stage beside a **second browser window textured
//! on every row** - `n06`'s cause, `2025-03-13 12_01_01.png` - reproduced on
//! a generated split screen and seen through [`locate`] and [`detect`]. This
//! is what the required `unit` gate sees of the story; the corpus half is
//! `crates/engine/tests/corpus_furniture_rows.rs` (AC-1, AC-3) and the lists
//! in `corpus_viewport.rs`, `corpus_viewport_stage.rs` and
//! `corpus_tuning_crops_unmoved.rs` (AC-1, AC-4).
//!
//! # The cause, as the Lead PO measured it (MC-075 `## Context`, "Measured")
//!
//! On `n06` the page column is `698,0 400x1440`. Over rows 115..1392 the share
//! of a column within `uniform_tolerance` of the page tone is 1.00 on 0..697
//! and 1098..1798 - the reader's window - and 0.00 from 1799 to the image's
//! edge: a second window, textured on every row, with no flat viewport of its
//! own. So the whole-margin share is 0.648 on **every** viewport row, under
//! `PAGE_LIKE`; MC-048's reading finds no run, and because that reading alone
//! decides whether the stage speaks (MC-052), the reader-window reading never
//! runs. `locate` returns `None` and the crop keeps rows 0..1440: the browser
//! bar, the scrollbar and the taskbar.
//!
//! # The fixture: invented here, `n06`'s bars at a sixth of the scale
//!
//! No oracle exists (the story partitions AC-5 as oracle-free).
//! [`Scene::render`] draws, top to bottom, at [`HEIGHT`] rows:
//!
//! - **the browser bar**, rows `0 ..` [`BAR_END`], seeded noise across the
//!   whole width (`n06`: `T` 115);
//! - **the reader's viewport**, [`BAR_END`] `..` [`SCROLLBAR`]: MC-027's page
//!   in its flat, speckled page margins, [`PAGE_W`] columns from one image
//!   edge;
//! - **the reader's horizontal scrollbar**, [`SCROLLBAR`] `..` [`TASKBAR`]:
//!   one flat value, [`SCROLLBAR_TONE`], far from the page tone, over the
//!   reader's columns (`n06`: 1392..1400, the user's ruling);
//! - **the taskbar**, [`TASKBAR`] `..` [`HEIGHT`], noise across the whole
//!   width (`n06`: 1400..1440);
//! - **the second window**, [`SECOND_W`] columns beside the reader's,
//!   noise on **every** row - no flat row anywhere, as `n06`'s (1799..2560).
//!
//! The bars lie outside `central_band` of the image's rows (rows 48..192 of
//! 240 at 0.6), and the second window is narrower than the page's textured
//! run, so the earlier stages locate the page column exactly as on one window
//! ([`the_page_column_reaches_the_viewport_stage_as_on_one_window`]).
//!
//! # The controls
//!
//! - **(a) the same screen without the second window**
//!   ([`Scene::Alone`]): the reader's window runs to the image edge. The stage
//!   locates the reader's rows, today and after. A fix that moved them broke
//!   the one-window case it was not asked to touch.
//! - **(b) a page hugged by a reader panel** ([`Scene::Hugged`]), the
//!   `2025-08-05` WebPs' shape as MC-052 GREEN described it (`00_11_13.webp`:
//!   page column 953, reader window 952..1597, so 1 and 4 page-tone columns
//!   between the page and a panel). Its rows: noise above, then the page, then
//!   noise below. The panel, beside the page on both sides, is textured on
//!   every row, so the whole margin is page-like on none and the stage
//!   declines, as on the WebPs. In the 1 + 4 page-tone columns between, one
//!   pixel is off the tone every [`HUG_PERIOD`] rows below a short band of
//!   [`HUG_FLAT`] rows - each column is still page background, a majority of
//!   its rows at the tone, but a row is page-like there only when all five
//!   are. The stage must decline, or at least never cut a row of the page's
//!   art: the crop keeps the whole page.
//!
//!   It is the control a fix that reads the reader's window after **any**
//!   decline fails - trial rules 1 and 2 in MC-075 `## Context`, which took
//!   the WebP to rows 0..76. [`window_after_any_decline`] is that stage,
//!   test-local, written from the story's description of rule 1 (not the
//!   Lead PO's trial code, which this file has never seen), and
//!   [`a_stage_that_reads_the_window_after_any_decline_fixes_the_split_screen_and_cuts_the_hugged_page`]
//!   shows it would pass the reproduction and fail control (b). Without that,
//!   (b) would be a control nothing plausible fails.
//!
//!   The panel is noise centred on the page tone, not a flat panel of another
//!   tone: a flat panel wide enough to hold most of the margin would *be* the
//!   margin's tone (the modal row median), every row would read page-like
//!   over it and the stage would speak on the panel - not the WebPs, where
//!   the whole margin declines. What (b) needs of the panel is what the WebPs
//!   show: page-like on no row, and not page background as a column.
//!
//! Every share and tone this file relies on is asserted, not assumed, in
//! [`the_shares_on_the_reader_viewport_rows_are_as_the_lead_po_measured_them_on_n06`]
//! and the premise test; the measured values are in MC-075's handoff.

mod common;

use common::{FADE_TONE, PAGE_W, Rng, VIEW_NOISE_HI, VIEW_NOISE_LO, page_core_rect, page_pixel};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::viewport::{MIN_RUN, PAGE_LIKE, Viewport, locate};
use cropper_core::{Luma, Rect, Tuning, detect};

// --- The geometry, invented here --------------------------------------------

/// The first row below the browser bar (`n06`: 115).
const BAR_END: u32 = 40;
/// The first row of the reader's horizontal scrollbar, which is the first row
/// below its viewport (`n06`: 1392).
const SCROLLBAR: u32 = 200;
/// The first row of the taskbar (`n06`: 1400).
const TASKBAR: u32 = 208;
/// The image's height (`n06`: 1440).
const HEIGHT: u32 = 240;

/// The second window's width. Narrower than the page's textured run (206
/// columns), so the page is the widest run `page_column` sees, and wide enough
/// that the whole-margin share is far under `PAGE_LIKE` on every viewport row
/// (asserted, with the measured value, in
/// [`the_shares_on_the_reader_viewport_rows_are_as_the_lead_po_measured_them_on_n06`]).
const SECOND_W: u32 = 64;

/// The reader's horizontal scrollbar: one flat value, well outside
/// `uniform_tolerance` of [`FADE_TONE`].
const SCROLLBAR_TONE: u8 = 180;

/// The seed the fixtures' noise is drawn from. Not MC-048's or MC-052's, so
/// nothing here can pass by sharing their bytes.
const SEED: u32 = 75;

/// Control (b): the page columns, in the page fixture's own coordinates, that
/// lie between the page column and the panel - one on the left and four on
/// the right, as on `2025-08-05 00_11_13.webp` (MC-052 GREEN). They are the
/// fade's columns of amplitude 5 down to 2, within `uniform_tolerance` of the
/// tone on every row, which the page column ends just inside (MC-053: its
/// first and last art columns are 45 and 254 at `uniform_tolerance` 10).
const HUG_SLIVERS: [u32; 5] = [44, 255, 256, 257, 258];
/// Control (b): the panel covers the page fixture's columns left of this one
/// ...
const HUG_LEFT_END: u32 = 44;
/// ... and from this one to the image edge.
const HUG_RIGHT_START: u32 = 259;
/// Control (b): the first rows of the page, below the noise, on which every
/// sliver pixel is at the tone: a run of page-like rows over the slivers
/// longer than `MIN_RUN`, as the WebP's 0..76.
const HUG_FLAT: u32 = 30;
/// Control (b): below the flat band, one sliver pixel is off the tone every
/// this many rows, the five slivers in turn. Under `MIN_RUN`, so no run of
/// page-like rows over the slivers is long enough below the band.
const HUG_PERIOD: u32 = 15;
/// Control (b): how far an off sliver pixel sits above the tone.
const HUG_OFF: u8 = 30;

/// Which side of the image the reader's window sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

/// The three screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    /// The reproduction: the reader's window beside a second window textured
    /// on every row.
    Split,
    /// Control (a): the same screen without the second window.
    Alone,
    /// Control (b): a page hugged on both sides by a textured reader panel.
    Hugged,
}

impl Scene {
    fn width(self) -> u32 {
        match self {
            Scene::Split => PAGE_W + SECOND_W,
            Scene::Alone | Scene::Hugged => PAGE_W,
        }
    }

    /// The screenshot, reader on the left; `Side::Right` is the same image
    /// mirrored left to right.
    fn render(self, side: Side) -> Luma {
        let width = self.width();
        let mut rng = Rng::new(SEED);
        let mut data = vec![0u8; (width * HEIGHT) as usize];
        for y in 0..HEIGHT {
            for x in 0..width {
                // Drawn for every pixel, so the noise is independent of the
                // layout and the same arguments give the same bytes.
                let noise = rng.between(VIEW_NOISE_LO, VIEW_NOISE_HI);
                let v = match self {
                    Scene::Split | Scene::Alone => {
                        if !(BAR_END..TASKBAR).contains(&y) || x >= PAGE_W {
                            noise
                        } else if y >= SCROLLBAR {
                            SCROLLBAR_TONE
                        } else {
                            page_pixel(x, y)
                        }
                    }
                    Scene::Hugged => {
                        if !(BAR_END..SCROLLBAR).contains(&y)
                            || !(HUG_LEFT_END..HUG_RIGHT_START).contains(&x)
                        {
                            noise
                        } else if hug_off(x, y) {
                            FADE_TONE + HUG_OFF
                        } else {
                            page_pixel(x, y)
                        }
                    }
                };
                let col = match side {
                    Side::Left => x,
                    Side::Right => width - 1 - x,
                };
                data[(y * width + col) as usize] = v;
            }
        }
        Luma {
            width,
            height: HEIGHT,
            data,
        }
    }

    /// The page art: MC-027's full-amplitude core columns, over every row of
    /// the reader's viewport (for (b), every page row).
    fn art(self, side: Side) -> Rect {
        let core = page_core_rect();
        let x = match side {
            Side::Left => core.x,
            Side::Right => self.width() - core.x - core.w,
        };
        Rect {
            x,
            y: BAR_END,
            w: core.w,
            h: SCROLLBAR - BAR_END,
        }
    }

    /// The reader's window, in image columns: the page fixture's columns.
    fn reader_columns(self, side: Side) -> std::ops::Range<u32> {
        match side {
            Side::Left => 0..PAGE_W,
            Side::Right => self.width() - PAGE_W..self.width(),
        }
    }
}

/// Whether control (b)'s sliver pixel at page-fixture `(x, y)` is drawn off
/// the tone: below the flat band, on every [`HUG_PERIOD`]-th row, one sliver
/// in turn.
fn hug_off(x: u32, y: u32) -> bool {
    let start = BAR_END + HUG_FLAT;
    if y < start || !(y - start).is_multiple_of(HUG_PERIOD) {
        return false;
    }
    let turn = ((y - start) / HUG_PERIOD) as usize % HUG_SLIVERS.len();
    HUG_SLIVERS[turn] == x
}

/// The reader's viewport, as the stage must return it.
const READER_VIEW: Viewport = Viewport {
    top: BAR_END,
    bottom: SCROLLBAR,
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
/// because the crate's own is private; the premise test checks the port
/// gives the fixture's [`FADE_TONE`] give or take the tolerance.
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
/// leave every row in the rect, peel nothing, and hand the stage the page
/// column inside the reader's window, holding the art - the same column on the
/// split screen as without the second window. And the ported tone is the
/// page's. Without it, a crop outside the reader's rows could be an earlier
/// stage's doing and say nothing about the viewport stage.
#[test]
fn the_page_column_reaches_the_viewport_stage_as_on_one_window() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for scene in [Scene::Split, Scene::Alone, Scene::Hugged] {
        for side in [Side::Left, Side::Right] {
            let img = scene.render(side);
            let first = trim_uniform(&img, &t).expect("not uniform");
            let found = content_box(&img, first, &t);
            let column = page_column_of(&img, &t);
            let art = scene.art(side);
            let reader = scene.reader_columns(side);
            if (column.y, column.h, found.removed.len()) != (0, HEIGHT, 0) {
                bad.push(format!(
                    "{scene:?}, reader on the {side:?}: the rows reaching the viewport stage \
                     must be the whole image with nothing peeled; got {column:?}, peeled {:?}",
                    found.removed
                ));
            }
            if !(column.x > reader.start
                && column.x + column.w < reader.end
                && column.x <= art.x
                && column.x + column.w >= art.x + art.w)
            {
                bad.push(format!(
                    "{scene:?}, reader on the {side:?}: the page column {column:?} must hold \
                     the art {art:?} and lie inside the reader's window, columns {reader:?}"
                ));
            }
            // The page column, in the page fixture's own coordinates, is the
            // same on every scene: MC-053's 45..=254.
            let local = match side {
                Side::Left => column.x,
                Side::Right => reader.end - (column.x + column.w),
            };
            if (local, column.w) != (45, 210) {
                bad.push(format!(
                    "{scene:?}, reader on the {side:?}: the page column must be the page \
                     fixture's columns 45..=254 as on one window; got {column:?}"
                ));
            }
            let tone = tone(&img, column);
            if tone.abs_diff(FADE_TONE) > 4 {
                bad.push(format!(
                    "{scene:?}, reader on the {side:?}: the margin's tone must be the page's, \
                     {FADE_TONE} give or take 4; got {tone}"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The cause, measured on the fixture as the Lead PO measured it on `n06`:
/// on every row of the reader's viewport the whole-margin share is under
/// `PAGE_LIKE` (`n06`: 0.648 on every row), while over only the reader's
/// window it is at or over it (`n06`: 1.000 on 1255 of 1277 rows, at least
/// 0.9 on the rest). On the bar, scrollbar and taskbar rows the reader's
/// window is not page-like (`n06`: 0.000 to 0.006). And without the second
/// window (control (a)) the whole margin is page-like on every viewport row.
///
/// This holds today and after: it is the fixture, not the fix. The values
/// are printed under `--nocapture` and recorded in MC-075's handoff.
#[test]
fn the_shares_on_the_reader_viewport_rows_are_as_the_lead_po_measured_them_on_n06() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for side in [Side::Left, Side::Right] {
        for scene in [Scene::Split, Scene::Alone] {
            let img = scene.render(side);
            let column = page_column_of(&img, &t);
            let tone = tone(&img, column);
            let whole = whole_margin(&img, column);
            let window: Vec<u32> = whole
                .iter()
                .copied()
                .filter(|x| scene.reader_columns(side).contains(x))
                .collect();
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            let mut window_lo = f32::MAX;
            for y in BAR_END..SCROLLBAR {
                let s = share(&img, y, &whole, tone, &t);
                lo = lo.min(s);
                hi = hi.max(s);
                let in_window = share(&img, y, &window, tone, &t);
                window_lo = window_lo.min(in_window);
                let want_page_like = scene == Scene::Alone;
                if (s >= PAGE_LIKE) != want_page_like {
                    bad.push(format!(
                        "{scene:?} {side:?} row {y}: whole-margin share {s:.3} must be {} \
                         PAGE_LIKE {PAGE_LIKE}",
                        if want_page_like {
                            "at or over"
                        } else {
                            "under"
                        }
                    ));
                }
                if in_window < PAGE_LIKE {
                    bad.push(format!(
                        "{scene:?} {side:?} row {y}: the reader's window share {in_window:.3} \
                         must be at or over PAGE_LIKE {PAGE_LIKE}"
                    ));
                }
            }
            let mut furniture_hi = f32::MIN;
            for y in (0..BAR_END).chain(SCROLLBAR..HEIGHT) {
                let in_window = share(&img, y, &window, tone, &t);
                furniture_hi = furniture_hi.max(in_window);
                if in_window >= PAGE_LIKE {
                    bad.push(format!(
                        "{scene:?} {side:?} row {y}: a bar, scrollbar or taskbar row must not \
                         be page-like over the reader's window; share {in_window:.3}"
                    ));
                }
            }
            println!(
                "{scene:?} reader {side:?}: tone {tone}; viewport rows: whole margin \
                 {lo:.3}..={hi:.3} over {} columns, reader's window min {window_lo:.3} over {} \
                 columns; furniture rows: reader's window max {furniture_hi:.3}",
                whole.len(),
                window.len()
            );
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- AC-5: the reproduction -----------------------------------------------------

/// AC-5, the stage: beside a second window textured on every row, the
/// viewport is exactly the reader's rows, the first row below the browser bar
/// to the first row of its scrollbar.
///
/// On `main` (`6af092f`) `locate` returns `None`: the whole-margin share is
/// under `PAGE_LIKE` on every row, MC-048's reading finds no run, and the
/// reader's window is never read.
#[test]
fn beside_a_second_window_textured_on_every_row_the_viewport_is_the_readers_rows() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for side in [Side::Left, Side::Right] {
        let img = Scene::Split.render(side);
        let got = locate(&img, page_column_of(&img, &t), &t);
        if got != Some(READER_VIEW) {
            bad.push(format!(
                "reader on the {side:?}: the viewport must be the reader's rows \
                 {READER_VIEW:?}; got {got:?}"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "MC-075 AC-5: beside a second window textured on every row the stage must \
         find the reader's own viewport.\n{}",
        bad.join("\n")
    );
}

/// AC-5, the crop: at both margins `detect` keeps no row of the browser bar,
/// the scrollbar or the taskbar - every crop row lies in the reader's
/// viewport - and keeps the whole page.
///
/// On `main` the crop keeps the full height, rows 0..240.
#[test]
fn beside_a_second_window_textured_on_every_row_the_crop_drops_the_bar_scrollbar_and_taskbar() {
    let mut bad = Vec::new();
    for side in [Side::Left, Side::Right] {
        let img = Scene::Split.render(side);
        let art = Scene::Split.art(side);
        for t in margins() {
            let got = crop(&img, &t);
            let tag = format!("reader on the {side:?}, margin_px {}", t.margin_px);
            if got.y < BAR_END {
                bad.push(format!(
                    "{tag}: the crop {got:?} keeps {} rows of the browser bar (rows 0..{BAR_END})",
                    BAR_END - got.y
                ));
            }
            if got.y + got.h > SCROLLBAR {
                bad.push(format!(
                    "{tag}: the crop {got:?} keeps {} rows of the scrollbar and taskbar (rows \
                     {SCROLLBAR}..{HEIGHT})",
                    got.y + got.h - SCROLLBAR
                ));
            }
            if !contains(got, art) {
                bad.push(format!("{tag}: the crop {got:?} cuts the page art {art:?}"));
            }
        }
    }
    assert!(bad.is_empty(), "MC-075 AC-5:\n{}", bad.join("\n"));
}

// --- AC-5, control (a): one window ----------------------------------------------

/// Control (a): the same screen without the second window, the reader's
/// window running to the image edge. The stage locates the same rows, and the
/// crop is the reader's rows at both margins. Holds on `main` and must hold
/// after.
#[test]
fn without_the_second_window_the_stage_locates_the_same_rows() {
    let mut bad = Vec::new();
    for side in [Side::Left, Side::Right] {
        let img = Scene::Alone.render(side);
        let art = Scene::Alone.art(side);
        let t = Tuning::default();
        let got = locate(&img, page_column_of(&img, &t), &t);
        if got != Some(READER_VIEW) {
            bad.push(format!(
                "reader on the {side:?}: the viewport must be {READER_VIEW:?}; got {got:?}"
            ));
        }
        for t in margins() {
            let rect = crop(&img, &t);
            if (rect.y, rect.y + rect.h) != (BAR_END, SCROLLBAR) || !contains(rect, art) {
                bad.push(format!(
                    "reader on the {side:?}, margin_px {}: the crop {rect:?} must be rows \
                     {BAR_END}..{SCROLLBAR} and hold the art {art:?}",
                    t.margin_px
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "MC-075 AC-5 control (a), one window:\n{}",
        bad.join("\n")
    );
}

// --- AC-5, control (b): a page hugged by a panel ----------------------------------

/// Control (b): a page hugged on both sides by a textured reader panel, with
/// one and four page-tone columns between (the `2025-08-05` WebPs' shape).
/// The stage declines, or never cuts a row of the page's art; at both margins
/// the crop keeps the whole page. Holds on `main` (the stage declines) and
/// must hold after.
#[test]
fn a_page_hugged_by_a_reader_panel_never_loses_a_row_of_its_art() {
    let mut bad = Vec::new();
    for side in [Side::Left, Side::Right] {
        let img = Scene::Hugged.render(side);
        let art = Scene::Hugged.art(side);
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
                "reader on the {side:?}: the viewport {got:?} cuts the page's art rows {}..{}",
                art.y,
                art.y + art.h
            ));
        }
        for t in margins() {
            let rect = crop(&img, &t);
            if !contains(rect, art) {
                bad.push(format!(
                    "reader on the {side:?}, margin_px {}: the crop {rect:?} cuts the page art \
                     {art:?}",
                    t.margin_px
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "MC-075 AC-5 control (b), a page hugged by a panel:\n{}",
        bad.join("\n")
    );
}

// --- Control (b)'s control: the fix it rules out --------------------------------

/// A stand-in viewport stage, **not** the product's: MC-048's whole-margin
/// reading, and wherever it finds a viewport this returns exactly what
/// [`locate`] returns. Where it **declines**, it reads the reader's window
/// anyway - MC-052's window, found over every row since there is no viewport
/// to find it over - takes the outer runs of [`MIN_RUN`] page-like rows there,
/// and keeps MC-054's evidence-of-chrome check. That is trial rule 1 of MC-075
/// `## Context` ("on a whole-margin decline, read the reader's window over
/// every row and take its outer runs, then MC-054's chrome check"), written
/// here from that description.
fn window_after_any_decline(img: &Luma, column: Rect, t: &Tuning) -> Option<Viewport> {
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
    let page_like_over = |y: usize, cols: &mut dyn Iterator<Item = usize>| {
        let (mut n, mut total) = (0usize, 0usize);
        for x in cols {
            n += usize::from(near(row(y)[x]));
            total += 1;
        }
        total > 0 && n as f32 / total as f32 >= PAGE_LIKE
    };
    // MC-048's reading, which declined: every row's whole margin.
    let whole: Vec<bool> = (0..rows)
        .map(|y| page_like_over(y, &mut (0..left_end).chain(right_start..width)))
        .collect();
    // MC-052's window, over every row: outward from the column while a
    // majority of the column's pixels is at the tone.
    let background = |x: usize| (0..rows).filter(|&y| near(row(y)[x])).count() * 2 > rows;
    let lo = (0..left_end)
        .rev()
        .find(|&x| !background(x))
        .map_or(0, |edge| edge + 1);
    let hi = (right_start..width)
        .find(|&x| !background(x))
        .unwrap_or(width);
    let in_window: Vec<bool> = (0..rows)
        .map(|y| page_like_over(y, &mut (lo..left_end).chain(right_start..hi)))
        .collect();
    let (mut top, mut bottom) = outer_runs(&in_window)?;
    // MC-054's evidence of chrome, read on the whole margin.
    let (first, end) = (column.y as usize, (column.y + column.h) as usize);
    if top > first && !has_run(whole[first..top].iter().map(|&p| !p)) {
        top = 0;
    }
    if bottom < end && !has_run(whole[bottom..end.min(rows)].iter().map(|&p| !p)) {
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

/// Control (b) is not vacuous. The stand-in [`window_after_any_decline`] is a
/// real candidate fix - it finds the reader's rows on the reproduction, and
/// it leaves control (a) as it is - and on control (b) it reads the flat band
/// over the five page-tone columns as the viewport, rows
/// [`BAR_END`]`..`[`BAR_END`]` + `[`HUG_FLAT`], cutting every page row below
/// it, as trial rule 1 cut the WebP to 0..76. So (b) is what fails that fix.
///
/// The stand-in's viewport is checked directly: `detect` clamps the crop's
/// rows to the viewport, so a viewport cutting the art is a crop cutting it.
#[test]
fn a_stage_that_reads_the_window_after_any_decline_fixes_the_split_screen_and_cuts_the_hugged_page()
{
    let t = Tuning::default();
    let mut bad = Vec::new();
    for side in [Side::Left, Side::Right] {
        for scene in [Scene::Split, Scene::Alone] {
            let img = scene.render(side);
            let got = window_after_any_decline(&img, page_column_of(&img, &t), &t);
            if got != Some(READER_VIEW) {
                bad.push(format!(
                    "{scene:?} {side:?}: the stand-in must find the reader's rows \
                     {READER_VIEW:?}; got {got:?}"
                ));
            }
        }
        let img = Scene::Hugged.render(side);
        let got = window_after_any_decline(&img, page_column_of(&img, &t), &t);
        let want = Some(Viewport {
            top: BAR_END,
            bottom: BAR_END + HUG_FLAT,
        });
        if got != want {
            bad.push(format!(
                "Hugged {side:?}: the stand-in must read only the flat band, {want:?}, and so \
                 cut the page's art rows {BAR_END}..{SCROLLBAR}; got {got:?}"
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
