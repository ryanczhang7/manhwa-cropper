//! MC-052, AC-6: the viewport stage on a **split-screen** screenshot - the
//! reader's browser window beside a second browser window - seen through
//! [`detect`]. This is what the required `unit` gate sees of the story; the
//! corpus half is `crates/engine/tests/corpus*.rs` (AC-1 to AC-5).
//!
//! # The bug these fixtures reproduce
//!
//! MC-048's stage scores each row by the share of **every** pixel outside the
//! page column, across the full image width, that is page background. On a
//! split-screen screenshot a second window beside the reader has its own
//! chrome and its own bottom edge, so on the rows only the reader's viewport
//! covers, the second window's chrome drags the full-width share under
//! `PAGE_LIKE` and the stage returns the **second** window's viewport. The
//! crop is clamped to it and cuts the reader's art (`2025-03-06 01_22_45.png`,
//! `2025-03-07 00_58_06.png`).
//!
//! # The fixture: what "a second window" is on a generated image
//!
//! Invented here; the story says no oracle exists. [`TwoWindows`] is:
//!
//! - **the reader window**, the [`PAGE_W`] columns next to one image edge:
//!   MC-027's page in its flat, speckled page margins over the reader's
//!   viewport rows, and seeded noise (its tab strip, address bar and
//!   bookmarks) above them and below them. Then its **scrollbar**,
//!   [`SCROLLBAR_W`] columns of one flat value that is not the page tone, over
//!   the viewport rows - the edge the corpus shows between the two windows
//!   (x 1811..1827 on `2025-03-06 01_22_45.png`, the second window from 1828).
//!   It is there so that the fixture does not forbid a fix that reads that
//!   edge; nothing here requires one to;
//! - **the second window**, [`SECOND_W`] columns beside it: a flat page
//!   background of the same tone over its own viewport rows, and noise (its
//!   chrome, its bottom panel) outside them;
//! - **the taskbar**, full-width noise across the bottom.
//!
//! The two viewports differ only in their rows. Case A gives the second
//! window a taller chrome strip and a shorter viewport than the reader's
//! ([`CASE_A`]); case B gives it a viewport reaching **past** the reader's,
//! above and below ([`CASE_B`]). Every mismatch strip is at least 20 rows -
//! over `MIN_RUN` - and lies outside `central_band` of the image's rows, so
//! the earlier stages locate the page column exactly as on a single-window
//! screenshot ([`the_page_column_reaches_the_viewport_stage_as_on_one_window`]).
//!
//! # The controls
//!
//! - MC-048's stage fails case A and passes case B, **by construction**: the
//!   full-width shares are computed and asserted below
//!   ([`on_rows_only_one_window_covers_the_full_width_share_is_not_page_like`]).
//! - Case B exists to rule out the cheap wrong fix, "a row is page-like when
//!   either side of the column is flat". [`either_side_flat`] is that stage,
//!   test-local and ported from MC-048's `locate` with only the per-row rule
//!   changed; [`an_either_side_flat_stage_passes_case_a_and_fails_case_b`]
//!   shows it would pass case A and fail case B. Without it, case B would be
//!   a test nothing plausible fails.
//!
//! Every number in the handoff table (`## Handoff: RED -> GREEN`) was measured
//! on this tree in RED.

mod common;

use common::{FADE_TONE, PAGE_W, Rng, VIEW_NOISE_HI, VIEW_NOISE_LO, page_core_rect, page_pixel};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::viewport::{MIN_RUN, PAGE_LIKE, Viewport};
use cropper_core::{Luma, Rect, Tuning, detect};

/// The second window's width. Chosen so that both controls fire; the shares
/// are asserted, not assumed
/// ([`on_rows_only_one_window_covers_the_full_width_share_is_not_page_like`]),
/// so a change to the fixture that moves one across `PAGE_LIKE` goes red there
/// rather than silently weakening a case.
///
/// The page column the earlier stages locate is 206 wide (columns 47..253 of
/// the page's 300), so 94 page-margin columns sit beside it, 47 on each side,
/// and the scrollbar's 17 beyond them. Measured in RED, both sides: see
/// `## Handoff: RED -> GREEN` for the ranges. In short, on the rows only the
/// reader's viewport covers the full-width share is far under `PAGE_LIKE`
/// (case A fails on MC-048's stage); on the rows only the second window's
/// covers it is under `PAGE_LIKE` (case B passes on MC-048's stage), while
/// the share on the second window's side of the column alone is over it (case
/// B fails on the either-side stand-in).
const SECOND_W: u32 = 680;

/// The reader window's scrollbar: its width, the corpus's 17 px, and its one
/// flat value, well outside `uniform_tolerance` of [`FADE_TONE`].
const SCROLLBAR_W: u32 = 17;
/// See [`SCROLLBAR_W`].
const SCROLLBAR_TONE: u8 = 180;

/// The reader window's width: the page in its margins, then the scrollbar.
const READER_W: u32 = PAGE_W + SCROLLBAR_W;

/// The seed the fixture's noise is drawn from. Not MC-048's, so nothing here
/// can pass by sharing its bytes.
const SEED: u32 = 52;

/// The rows of one two-window screenshot. Every range is `start .. end`.
#[derive(Debug, Clone, Copy)]
struct TwoWindows {
    /// The image's height.
    height: u32,
    /// The reader window's viewport: MC-027's page in its margins.
    reader: (u32, u32),
    /// The second window's viewport: flat page background.
    second: (u32, u32),
    /// The first row of the full-width taskbar.
    taskbar: u32,
}

/// Case A: the second window's chrome ends 20 rows below the reader's and its
/// viewport ends 20 rows above the reader's. The reader-only strips are rows
/// 40..60 and 340..360; the taskbar is 360..400.
const CASE_A: TwoWindows = TwoWindows {
    height: 400,
    reader: (40, 360),
    second: (60, 340),
    taskbar: 360,
};

/// Case B: the second window's viewport starts 30 rows above the reader's and
/// ends 20 rows below it. The second-window-only strips are rows 30..60 (the
/// reader's own bookmark bar, beside it) and 380..400 (the strip below the
/// reader window); the taskbar is 400..440.
const CASE_B: TwoWindows = TwoWindows {
    height: 440,
    reader: (60, 380),
    second: (30, 400),
    taskbar: 400,
};

/// Which side of the image the reader window sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reader {
    Left,
    Right,
}

impl TwoWindows {
    fn width() -> u32 {
        READER_W + SECOND_W
    }

    /// The screenshot, reader on the left; `Reader::Right` is the same image
    /// mirrored left to right.
    fn render(self, side: Reader) -> Luma {
        let width = Self::width();
        let mut rng = Rng::new(SEED);
        let mut data = vec![0u8; (width * self.height) as usize];
        let inside = |(a, b): (u32, u32), y: u32| (a..b).contains(&y);
        for y in 0..self.height {
            for x in 0..width {
                // Drawn for every pixel, so the noise is independent of the
                // layout and the same arguments give the same bytes.
                let noise = rng.between(VIEW_NOISE_LO, VIEW_NOISE_HI);
                let v = if y >= self.taskbar {
                    noise
                } else if x < READER_W {
                    match (inside(self.reader, y), x < PAGE_W) {
                        (true, true) => page_pixel(x, y),
                        (true, false) => SCROLLBAR_TONE,
                        (false, _) => noise,
                    }
                } else if inside(self.second, y) {
                    FADE_TONE
                } else {
                    noise
                };
                let col = match side {
                    Reader::Left => x,
                    Reader::Right => width - 1 - x,
                };
                data[(y * width + col) as usize] = v;
            }
        }
        Luma {
            width,
            height: self.height,
            data,
        }
    }

    /// The page art: MC-027's full-amplitude core columns, over every row of
    /// the reader's viewport.
    fn art(self, side: Reader) -> Rect {
        let core = page_core_rect();
        let x = match side {
            Reader::Left => core.x,
            Reader::Right => Self::width() - core.x - core.w,
        };
        Rect {
            x,
            y: self.reader.0,
            w: core.w,
            h: self.reader.1 - self.reader.0,
        }
    }
}

/// Both margins, as the story defines them.
fn margins() -> [Tuning; 2] {
    [
        Tuning::default(),
        Tuning {
            margin_px: 0,
            ..Tuning::default()
        },
    ]
}

/// The rect `detect` returns for `img` at `t`.
fn crop(img: &Luma, t: &Tuning) -> Rect {
    detect(img, t)
        .expect("a two-window scene is not one flat colour")
        .rect
}

/// Whether `outer` contains `inner` entirely.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// The page column `detect` hands the stage: its first five stages, composed
/// from their public functions in `decide.rs`'s order, before the margin.
fn page_column_of(img: &Luma, t: &Tuning) -> Rect {
    let first = trim_uniform(img, t).expect("not one flat colour");
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t).expect("not one flat colour");
    page_column(img, textured_box(img, second, t), t)
}

/// The share of row `y`'s pixels in columns `cols` that lie within
/// `uniform_tolerance` of the fixture's page tone, [`FADE_TONE`].
fn share(img: &Luma, y: u32, cols: impl Iterator<Item = u32> + Clone, t: &Tuning) -> f32 {
    let n = cols.clone().count();
    let near = cols
        .filter(|&x| {
            img.data[(y * img.width + x) as usize].abs_diff(FADE_TONE) <= t.uniform_tolerance
        })
        .count();
    near as f32 / n as f32
}

/// Every violation of case A's claim, for one side and one margin: the crop
/// contains the page art and no row of the reader's chrome (rows above its
/// viewport) or of the taskbar (rows from the end of its viewport down).
fn case_a_violations(side: Reader, t: &Tuning) -> Vec<String> {
    let scene = CASE_A;
    let img = scene.render(side);
    let got = crop(&img, t);
    let art = scene.art(side);
    let (top, bottom) = scene.reader;
    let mut bad = Vec::new();
    let tag = format!("reader on the {side:?}, margin_px {}", t.margin_px);
    if !contains(got, art) {
        bad.push(format!(
            "{tag}: the crop {got:?} cuts the page art {art:?} - rows {}..={} kept, art is rows {top}..={}",
            got.y,
            got.y + got.h - 1,
            bottom - 1
        ));
    }
    if got.y < top {
        bad.push(format!(
            "{tag}: the crop {got:?} keeps {} rows of the reader's chrome (rows 0..{top})",
            top - got.y
        ));
    }
    if got.y + got.h > bottom {
        bad.push(format!(
            "{tag}: the crop {got:?} keeps {} rows of the taskbar (rows {bottom}..{})",
            got.y + got.h - bottom,
            scene.height
        ));
    }
    bad
}

/// Every violation of case B's claim, for one side and one margin: no crop
/// row lies outside the reader window's viewport, and - the product's
/// standing rule, which no fix may trade away here - the crop contains the
/// page art.
fn case_b_violations(side: Reader, t: &Tuning) -> Vec<String> {
    let scene = CASE_B;
    let img = scene.render(side);
    let got = crop(&img, t);
    let art = scene.art(side);
    let tag = format!("reader on the {side:?}, margin_px {}", t.margin_px);
    let mut bad = rows_outside(&tag, got.y, got.y + got.h, scene.reader);
    if !contains(got, art) {
        bad.push(format!("{tag}: the crop {got:?} cuts the page art {art:?}"));
    }
    bad
}

/// The rows `start .. end` has outside the reader's viewport `view`.
fn rows_outside(tag: &str, start: u32, end: u32, (top, bottom): (u32, u32)) -> Vec<String> {
    let mut bad = Vec::new();
    if start < top {
        bad.push(format!(
            "{tag}: rows {start}..{top} are kept, above the reader window's viewport \
             ({top}..{bottom}) - its own chrome, beside the second window's viewport"
        ));
    }
    if end > bottom {
        bad.push(format!(
            "{tag}: rows {bottom}..{end} are kept, below the reader window's viewport \
             ({top}..{bottom}) - the strip under it, beside the second window's viewport"
        ));
    }
    bad
}

// --- The premise --------------------------------------------------------------

/// The fixture's premise: on every scene and both sides, the earlier stages
/// leave every row in the rect, peel nothing, and locate the page column
/// inside the reader window - the same column the reader's page gives on its
/// own. Without it, a crop outside the reader's viewport could be an earlier
/// stage's doing and would say nothing about the viewport stage.
#[test]
fn the_page_column_reaches_the_viewport_stage_as_on_one_window() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for (name, scene) in [("A", CASE_A), ("B", CASE_B)] {
        for side in [Reader::Left, Reader::Right] {
            let img = scene.render(side);
            let first = trim_uniform(&img, &t).expect("not uniform");
            let found = content_box(&img, first, &t);
            let column = page_column_of(&img, &t);
            let art = scene.art(side);
            let (lo, hi) = match side {
                Reader::Left => (0, PAGE_W),
                Reader::Right => (TwoWindows::width() - PAGE_W, TwoWindows::width()),
            };
            if (column.y, column.h, found.removed.len()) != (0, img.height, 0) {
                bad.push(format!(
                    "case {name}, reader on the {side:?}: the rows reaching the viewport \
                     stage must be the whole image with nothing peeled; got {column:?}, \
                     peeled {:?}",
                    found.removed
                ));
            }
            if !(column.x > lo
                && column.x + column.w < hi
                && column.x <= art.x
                && column.x + column.w >= art.x + art.w)
            {
                bad.push(format!(
                    "case {name}, reader on the {side:?}: the page column {column:?} must \
                     hold the art {art:?} and lie inside the reader window, columns {lo}..{hi}"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// The controls, by construction. On the rows only one window's viewport
/// covers, the share of the full width outside the page column that is page
/// background is under `PAGE_LIKE` - so MC-048's full-width stage cannot read
/// case A's reader-only rows as page, and cannot read case B's
/// second-window-only rows as page either. And on case B's
/// second-window-only rows, the side of the column the second window is on is
/// over `PAGE_LIKE` on its own, which is what makes the either-side stand-in
/// take them.
#[test]
fn on_rows_only_one_window_covers_the_full_width_share_is_not_page_like() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    let check = |bad: &mut Vec<String>, what: &str, got: f32, want_page_like: bool| {
        if (got >= PAGE_LIKE) != want_page_like {
            bad.push(format!(
                "{what}: share {got:.3} must be {} PAGE_LIKE {PAGE_LIKE}",
                if want_page_like {
                    "at or over"
                } else {
                    "under"
                }
            ));
        }
    };
    for side in [Reader::Left, Reader::Right] {
        for (name, scene, strips) in [
            ("A", CASE_A, [(40, 60), (340, 360)]),
            ("B", CASE_B, [(30, 60), (380, 400)]),
        ] {
            let img = scene.render(side);
            let column = page_column_of(&img, &t);
            let (cx, cend) = (column.x, column.x + column.w);
            let outside = (0..cx).chain(cend..img.width);
            let second_side: Vec<u32> = match side {
                Reader::Left => (cend..img.width).collect(),
                Reader::Right => (0..cx).collect(),
            };
            for (a, b) in strips {
                for y in a..b {
                    let full = share(&img, y, outside.clone(), &t);
                    check(
                        &mut bad,
                        &format!("case {name} {side:?} row {y}, full width"),
                        full,
                        false,
                    );
                    if name == "B" {
                        let one = share(&img, y, second_side.iter().copied(), &t);
                        check(
                            &mut bad,
                            &format!("case B {side:?} row {y}, second window's side"),
                            one,
                            true,
                        );
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- AC-6, case A ---------------------------------------------------------------

/// AC-6 case A, the reader on the left: beside a second window whose chrome
/// is taller and whose viewport is shorter, the crop keeps the whole page and
/// no row of the reader's chrome or the taskbar, at both margins.
///
/// On MC-048 the stage returns the second window's viewport, rows 60..340,
/// and the crop cuts 20 rows of art at the top and 20 at the bottom.
#[test]
fn beside_a_window_with_taller_chrome_the_crop_keeps_the_readers_whole_page() {
    let bad: Vec<String> = margins()
        .iter()
        .flat_map(|t| case_a_violations(Reader::Left, t))
        .collect();
    assert!(bad.is_empty(), "AC-6 case A:\n{}", bad.join("\n"));
}

/// AC-6 case A mirrored: the reader window on the right of the screenshot.
#[test]
fn with_the_reader_on_the_right_the_crop_still_keeps_its_whole_page() {
    let bad: Vec<String> = margins()
        .iter()
        .flat_map(|t| case_a_violations(Reader::Right, t))
        .collect();
    assert!(bad.is_empty(), "AC-6 case A, mirrored:\n{}", bad.join("\n"));
}

// --- AC-6, case B ---------------------------------------------------------------

/// AC-6 case B, the reader on the left: beside a second window whose viewport
/// reaches past the reader's above and below, no crop row lies outside the
/// reader window's own viewport, at both margins. This is what rules out a
/// stage that takes a row as page when either side of the column is flat.
#[test]
fn beside_a_window_with_a_taller_viewport_no_row_outside_the_readers_is_kept() {
    let bad: Vec<String> = margins()
        .iter()
        .flat_map(|t| case_b_violations(Reader::Left, t))
        .collect();
    assert!(bad.is_empty(), "AC-6 case B:\n{}", bad.join("\n"));
}

/// AC-6 case B mirrored: the reader window on the right of the screenshot.
#[test]
fn with_the_reader_on_the_right_no_row_outside_its_viewport_is_kept() {
    let bad: Vec<String> = margins()
        .iter()
        .flat_map(|t| case_b_violations(Reader::Right, t))
        .collect();
    assert!(bad.is_empty(), "AC-6 case B, mirrored:\n{}", bad.join("\n"));
}

// --- The control on case B: the fix it rules out --------------------------------

/// A stand-in viewport stage, **not** the product's: MC-048's `locate` - the
/// same margin, the same tone estimate, `PAGE_LIKE`, `MIN_RUN` - with the one
/// change case B exists to rule out. A row is page-like when the pixels on
/// **either** side of the column, taken alone, are page-like.
fn either_side_flat(img: &Luma, column: Rect, t: &Tuning) -> Option<Viewport> {
    let width = img.width as usize;
    let left_end = (column.x as usize).min(width);
    let right_start = (column.x as usize + column.w as usize).min(width);
    let rows = img.height as usize;
    let row = |y: usize| &img.data[y * width..(y + 1) * width];

    // The tone: MC-048's, over the full margin.
    let medians: Vec<u8> = (0..rows)
        .map(|y| {
            let mut m: Vec<u8> = row(y)[..left_end]
                .iter()
                .chain(&row(y)[right_start..])
                .copied()
                .collect();
            m.sort_unstable();
            m[m.len() / 2]
        })
        .collect();
    let mut bins = [0usize; 32];
    let mut exact = [0usize; 256];
    for &m in &medians {
        bins[usize::from(m / 8)] += 1;
        exact[usize::from(m)] += 1;
    }
    let last_max = |c: &[usize]| (0..c.len()).fold(0, |b, i| if c[i] >= c[b] { i } else { b });
    let best = last_max(&bins);
    let tone = (best * 8 + last_max(&exact[best * 8..best * 8 + 8])) as u8;

    let flat = |px: &[u8]| {
        !px.is_empty()
            && px
                .iter()
                .filter(|v| v.abs_diff(tone) <= t.uniform_tolerance)
                .count() as f32
                / px.len() as f32
                >= PAGE_LIKE
    };
    let page_like: Vec<bool> = (0..rows)
        .map(|y| flat(&row(y)[..left_end]) || flat(&row(y)[right_start..]))
        .collect();

    // MC-048's selector: the first run of MIN_RUN to the end of the last one.
    let mut first = None;
    let mut last = None;
    let mut y = 0;
    while y < rows {
        let start = y;
        while y < rows && page_like[y] {
            y += 1;
        }
        if y - start >= MIN_RUN {
            first.get_or_insert(start);
            last = Some(y);
        }
        y += 1;
    }
    Some(Viewport {
        top: first? as u32,
        bottom: last? as u32,
    })
}

/// Case B's control. The either-side stand-in reads case A correctly - the
/// reader's own flat side carries the reader-only rows - and case B wrongly:
/// the second window's flat side carries the rows of the reader's own chrome
/// beside it, and the strip below the reader window. So case B is the only
/// thing in this file that fails that fix, and this shows it does.
///
/// The stand-in's viewport is checked directly; `detect` clamps the crop's
/// rows to the viewport, so a viewport reaching outside the reader's is the
/// crop reaching outside it.
#[test]
fn an_either_side_flat_stage_passes_case_a_and_fails_case_b() {
    let t = Tuning::default();
    let mut bad = Vec::new();
    for side in [Reader::Left, Reader::Right] {
        let a = CASE_A.render(side);
        let view = either_side_flat(&a, page_column_of(&a, &t), &t);
        if view
            != Some(Viewport {
                top: CASE_A.reader.0,
                bottom: CASE_A.reader.1,
            })
        {
            bad.push(format!(
                "case A {side:?}: the stand-in must find the reader's viewport {:?}; got {view:?}",
                CASE_A.reader
            ));
        }

        let b = CASE_B.render(side);
        let view = either_side_flat(&b, page_column_of(&b, &t), &t)
            .expect("the stand-in locates a viewport on case B");
        let outside = rows_outside("stand-in", view.top, view.bottom, CASE_B.reader);
        if outside.len() != 2 {
            bad.push(format!(
                "case B {side:?}: the stand-in must keep rows outside the reader's viewport \
                 at the top and the bottom, so that case B fails it; got {view:?}"
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
