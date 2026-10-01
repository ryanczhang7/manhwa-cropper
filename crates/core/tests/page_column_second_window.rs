//! MC-066 AC-4: a reader window beside a **second browser window** whose
//! content is a large textured block - a video frame - is cropped inside the
//! reader's window, seen through [`detect`], on generated images. The corpus
//! half is `crates/engine/tests/corpus_second_window.rs` (AC-1, AC-2); this
//! file is what the required `unit` gate sees of the story.
//!
//! # The bug: two wrong shapes, and two causes
//!
//! Two fresh corpus screenshots are split screens, the `toongod` reader on
//! the left and a YouTube window on the right (MC-066 `## Context`):
//!
//! - `f18` (`2025-03-06 12_48_06.png`) is cropped to the **second window
//!   only**, `1828,0 717x1440`;
//! - `f13` (`2025-03-16 22_47_44.png`) is cropped **across both windows**,
//!   `635,115 1922x1285`.
//!
//! The story names the textured video frame as a *candidate*. MC-066's RED
//! read the stages on both screenshots (`## Handoff: RED -> GREEN`) and found
//! that it is the cause of `f18`'s shape and **not** of `f13`'s:
//!
//! - **`f18`: the widest textured run is the video.** Stage 3c,
//!   `flat::page_column`, takes the *widest* run of columns textured over the
//!   central band (`edges::widest_textured_run`). Over that band the page is
//!   517 columns (651..1167) and the video 716 (1829..2544), so the run is the
//!   video, widened to `1828..2544`. The viewport stage then declines beside
//!   it and the rows stay the whole image.
//! - **`f13`: the page's left margin is peeled as chrome, and the interior
//!   rule then refuses.** `f13`'s left page margin is one flat value (11)
//!   from image column 0 to the page (635 columns, 0.248 of the width, under
//!   `chrome_max_extent`, 0.30). Its first strong column line is the page's
//!   left edge, so stage 2, `content_box`, peels columns 0..634 as a chrome
//!   strip (`removed` holds `Left`). The widest run is the page (533 columns;
//!   the video is 402), but it now starts on the rect's own first column, and
//!   stage 3c's interior rule - "a page column that reaches either end of the
//!   rect has no page margin on that side" - hands back the whole rect,
//!   `635..2556`: the page, the empty margin, the scrollbars, the seam and
//!   the second window. `f18` and MC-052's two split screens are not peeled,
//!   because a few textured columns sit at their image's left edge (`f18`
//!   0..6), so their first strong line is not the page's edge.
//!
//! Luma is the only signal, here and in the fix: colour and chroma are out of
//! scope (MC-066 `## Out of scope`).
//!
//! # The fixtures: invented here, each cause turned on and off
//!
//! One geometry, [`W`] x [`H`], about half of a 2560x1440 screen: textured
//! chrome over the top [`CHROME_END`] rows and a textured taskbar from
//! [`TASKBAR`], across the whole width. Between them:
//!
//! - **the reader window**, columns `0 .. SEAM`: the page (textured art) in a
//!   flat page margin of [`MARGIN_TONE`], then the reader's scrollbar
//!   ([`SCROLLBAR_TONE`]). Its left edge is one of [`LeftEdge`];
//! - **the seam**, column [`SEAM`], the second window's first column;
//! - **the second window**: a flat dark background ([`SECOND_TONE`]) holding
//!   a textured block (the video) at its left, a textured line of text below
//!   it, and its own scrollbar at the right edge. The block is one of
//!   [`Block`].
//!
//! The scenes:
//!
//! | left edge | block | today | the claim |
//! |---|---|---|---|
//! | sliver | wide | second window | `f18`'s shape: **cause on** |
//! | sliver | narrow | the page | `f18`'s cause off: the block narrower than the page |
//! | sliver | flat | the page | `f18`'s cause off: no block at all |
//! | flat | narrow | both windows | `f13`'s shape: **cause on** |
//! | flat | flat | both windows | `f13`'s shape without any block: the block is not its cause |
//! | flat | wide | second window | both causes at once |
//! | flat, wide margin | narrow | the page | `f13`'s cause off: the margin wider than `chrome_max_extent` |
//!
//! "Today" is measured on this tree in MC-066's RED. The premise test
//! measures every geometric claim above on the fixtures, from the
//! definitions, without the pipeline, so a fixture that drifts goes red
//! rather than making a case vacuous.

use cropper_core::flat::central_band;
use cropper_core::{Luma, Rect, Tuning, detect};

// --- Settled constants, read out --------------------------------------------

/// `Tuning::default().chrome_max_extent`, copied from `lib.rs` so that the
/// premise can say which side of it each left margin falls. Asserted equal to
/// the shipped value in the premise test.
const CHROME_MAX_EXTENT: f32 = 0.30;

/// `Tuning::default().min_line_spread`: a column is textured over the band
/// at or above this mean absolute deviation. Asserted equal to the shipped
/// value in the premise test.
const MIN_LINE_SPREAD: f64 = 8.0;

// --- The geometry, invented here --------------------------------------------

/// The fixture's width: half of 2560.
const W: u32 = 1280;
/// The fixture's height: half of 1440.
const H: u32 = 720;
/// The first row below the browser chrome (`f13`/`f18`: 115).
const CHROME_END: u32 = 58;
/// The taskbar's first row (`f13`/`f18`: 1400).
const TASKBAR: u32 = 700;

/// The page's first column where the left margin is under
/// `chrome_max_extent`: 320 of 1280, 0.25 of the width (`f13`: 635 of 2560,
/// 0.248).
const PAGE_FIRST: u32 = 320;
/// The page's first column where the left margin is past
/// `chrome_max_extent`: 400 of 1280, 0.3125 of the width.
const PAGE_FIRST_WIDE: u32 = 400;
/// The page's width (`f13`'s 533 and `f18`'s 517, halved).
const PAGE_W: u32 = 264;
/// Textured columns at the image's left edge on [`LeftEdge::Sliver`]
/// (`f18` 0..6).
const SLIVER_W: u32 = 4;
/// The reader's scrollbar: its first column (`f13`/`f18` about 1803..1827).
const SCROLLBAR_FIRST: u32 = 906;
/// The seam: the second window's first column, one flat column of
/// [`SEAM_TONE`] (`f18` 1828). The reader's window is `0 .. SEAM`.
const SEAM: u32 = 914;
/// The video's first column, just past the seam.
const BLOCK_FIRST: u32 = SEAM + 1;
/// The wide video's end, exclusive: 357 columns, wider than the page
/// (`f18`'s 716 against 517).
const WIDE_END: u32 = 1272;
/// The narrow video's end, exclusive: 200 columns, narrower than the page
/// (`f13`'s 402 against 533).
const NARROW_END: u32 = 1115;
/// The video's rows (`f18` 194..1255, `f13` 505..1258, halved and rounded
/// out): they cover the central band.
const BLOCK_ROWS: (u32, u32) = (100, 630);
/// A line of textured text across the second window below the video (the
/// title under a YouTube video). Outside the central band.
const TEXT_ROWS: (u32, u32) = (650, 690);
/// The second window's scrollbar, to the image's right edge: two columns of
/// 255, four textured, two of 255 (`f18`/`f13` 2545..2559).
const SECOND_SCROLLBAR: u32 = 1272;

/// The page margin's one value (`f13`/`f18`: 11).
const MARGIN_TONE: u8 = 11;
/// The reader's scrollbar (`f13`/`f18`: 66).
const SCROLLBAR_TONE: u8 = 66;
/// The seam column (`f18`: 32).
const SEAM_TONE: u8 = 32;
/// The second window's background (`f13`: 16 to 18).
const SECOND_TONE: u8 = 16;

// --- The fixtures -----------------------------------------------------------

/// What the reader window has at the image's left edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftEdge {
    /// [`SLIVER_W`] textured columns, then the flat margin, page at
    /// [`PAGE_FIRST`] (`f18`, MC-052's split screens).
    Sliver,
    /// The flat margin from column 0, page at [`PAGE_FIRST`], under
    /// `chrome_max_extent` (`f13`).
    Flat,
    /// The flat margin from column 0, page at [`PAGE_FIRST_WIDE`], past
    /// `chrome_max_extent`.
    FlatWide,
}

/// What the second window shows beside its background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Block {
    /// A textured block wider than the page (`f18`).
    Wide,
    /// A textured block narrower than the page (`f13`).
    Narrow,
    /// No block: the background throughout.
    Flat,
}

/// One scene.
#[derive(Debug, Clone, Copy)]
struct Scene {
    left: LeftEdge,
    block: Block,
}

impl Scene {
    /// The page's first column.
    fn page_first(self) -> u32 {
        match self.left {
            LeftEdge::Sliver | LeftEdge::Flat => PAGE_FIRST,
            LeftEdge::FlatWide => PAGE_FIRST_WIDE,
        }
    }

    /// The page art: its columns, over every row of the reader's viewport.
    fn art(self) -> Rect {
        Rect {
            x: self.page_first(),
            y: CHROME_END,
            w: PAGE_W,
            h: TASKBAR - CHROME_END,
        }
    }

    /// The video's columns, `start .. end`; empty for [`Block::Flat`].
    fn block_cols(self) -> (u32, u32) {
        match self.block {
            Block::Wide => (BLOCK_FIRST, WIDE_END),
            Block::Narrow => (BLOCK_FIRST, NARROW_END),
            Block::Flat => (BLOCK_FIRST, BLOCK_FIRST),
        }
    }

    fn render(self) -> Luma {
        let page = self.page_first()..self.page_first() + PAGE_W;
        let (b0, b1) = self.block_cols();
        let inside = |(a, b): (u32, u32), v: u32| (a..b).contains(&v);
        let mut data = Vec::with_capacity((W * H) as usize);
        for y in 0..H {
            for x in 0..W {
                let sliver = self.left == LeftEdge::Sliver && x < SLIVER_W;
                let v = if !(CHROME_END..TASKBAR).contains(&y) {
                    texture(x, y)
                } else if x < SEAM {
                    if sliver || page.contains(&x) {
                        texture(x, y)
                    } else if x >= SCROLLBAR_FIRST {
                        SCROLLBAR_TONE
                    } else {
                        MARGIN_TONE
                    }
                } else if x == SEAM {
                    SEAM_TONE
                } else if x >= SECOND_SCROLLBAR {
                    if (SECOND_SCROLLBAR + 2..SECOND_SCROLLBAR + 6).contains(&x) {
                        texture(x, y)
                    } else {
                        255
                    }
                } else if inside(TEXT_ROWS, y) || (inside((b0, b1), x) && inside(BLOCK_ROWS, y)) {
                    texture(x, y)
                } else {
                    SECOND_TONE
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

/// One textured pixel: 30 ..= 220.
fn texture(x: u32, y: u32) -> u8 {
    30 + (hash(x, y) % 191) as u8
}

// --- Measuring, test-side ---------------------------------------------------

/// The central band of the whole fixture: the rows stage 3c measures a
/// column over, when nothing earlier moves the rect's rows.
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

/// Column `x`'s mean absolute deviation about its mean, over the band.
fn spread(img: &Luma, x: u32) -> f64 {
    let b = band();
    let values: Vec<f64> = (b.y..b.y + b.h)
        .map(|y| f64::from(img.data[(y * img.width + x) as usize]))
        .collect();
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|v| (v - mean).abs()).sum::<f64>() / values.len() as f64
}

/// The runs of consecutive columns textured over the band, `(first, end)`
/// with `end` exclusive, in order.
fn textured_runs(img: &Luma) -> Vec<(u32, u32)> {
    let mut runs = Vec::new();
    let mut open = None;
    for x in 0..W {
        match (spread(img, x) >= MIN_LINE_SPREAD, open) {
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

/// Every way `scene`'s crop leaves the reader's window, at both margins: a
/// column of the seam or the second window kept, a row of chrome or taskbar
/// kept, or the page art cut.
fn outside_the_reader(scene: Scene) -> Vec<String> {
    let img = scene.render();
    let art = scene.art();
    let mut bad = Vec::new();
    for t in both_margins() {
        let got = detect(&img, &t)
            .expect("a two-window scene is not one flat colour")
            .rect;
        let (end, bottom) = (got.x + got.w, got.y + got.h);
        let tag = format!(
            "{scene:?}, margin_px {}: crop {},{} {}x{} (columns {}..{end}, rows {}..{bottom})",
            t.margin_px, got.x, got.y, got.w, got.h, got.x, got.y
        );
        if end > SEAM {
            bad.push(format!(
                "{tag} keeps {} column(s) of the second window, which starts at the seam, \
                 column {SEAM}; the crop must end at {SEAM} or left of it",
                end - SEAM.max(got.x)
            ));
        }
        if got.x > art.x || end < art.x + art.w || got.y > art.y || bottom < art.y + art.h {
            bad.push(format!(
                "{tag} cuts the page art, columns {}..{} rows {}..{}",
                art.x,
                art.x + art.w,
                art.y,
                art.y + art.h
            ));
        }
        if got.y < CHROME_END || bottom > TASKBAR {
            bad.push(format!(
                "{tag} keeps a row of browser chrome or taskbar; the reader's viewport is \
                 rows {CHROME_END}..{TASKBAR}"
            ));
        }
    }
    bad
}

// --- The premise --------------------------------------------------------------

/// The fixture is what the table in the module documentation says, measured
/// from the definitions without the pipeline: the band holds no chrome or
/// taskbar row and crosses the video; over it the page, the video and the
/// sliver are textured and nothing else is, so the textured runs are exactly
/// those; the wide video is wider than the page and the narrow one narrower;
/// and the flat left margin is under `chrome_max_extent` of the width on
/// [`LeftEdge::Flat`] and past it on [`LeftEdge::FlatWide`].
#[test]
fn the_scenes_measure_as_the_causes_claim() {
    let shipped = Tuning::default();
    assert!(
        (shipped.chrome_max_extent - CHROME_MAX_EXTENT).abs() < f32::EPSILON
            && (f64::from(shipped.min_line_spread) - MIN_LINE_SPREAD).abs() < 1e-9,
        "the copied constants must be the shipped ones: chrome_max_extent {} (copied \
         {CHROME_MAX_EXTENT}), min_line_spread {} (copied {MIN_LINE_SPREAD})",
        shipped.chrome_max_extent,
        shipped.min_line_spread
    );
    let b = band();
    assert!(
        b.y >= CHROME_END
            && b.y + b.h <= TASKBAR
            && b.y >= BLOCK_ROWS.0
            && b.y + b.h <= BLOCK_ROWS.1
            && b.y + b.h <= TEXT_ROWS.0,
        "the band {b:?} must hold no chrome, taskbar or text row and lie inside the \
         video's rows {BLOCK_ROWS:?}"
    );

    let mut bad = Vec::new();
    for left in [LeftEdge::Sliver, LeftEdge::Flat, LeftEdge::FlatWide] {
        for block in [Block::Wide, Block::Narrow, Block::Flat] {
            let scene = Scene { left, block };
            let img = scene.render();
            let page = (scene.page_first(), scene.page_first() + PAGE_W);
            let mut want = Vec::new();
            if left == LeftEdge::Sliver {
                want.push((0, SLIVER_W));
            }
            want.push(page);
            let (b0, b1) = scene.block_cols();
            if b1 > b0 {
                want.push((b0, b1));
            }
            want.push((SECOND_SCROLLBAR + 2, SECOND_SCROLLBAR + 6));
            let got = textured_runs(&img);
            if got != want {
                bad.push(format!(
                    "{scene:?}: textured runs over the band {got:?}, want {want:?}"
                ));
            }
        }
    }
    let wide = WIDE_END - BLOCK_FIRST;
    let narrow = NARROW_END - BLOCK_FIRST;
    if !(wide > PAGE_W && narrow < PAGE_W) {
        bad.push(format!(
            "the wide video ({wide}) must be wider than the page ({PAGE_W}) and the narrow \
             one ({narrow}) narrower"
        ));
    }
    let share = |first: u32| first as f32 / W as f32;
    if !(share(PAGE_FIRST) <= CHROME_MAX_EXTENT && share(PAGE_FIRST_WIDE) > CHROME_MAX_EXTENT) {
        bad.push(format!(
            "the flat left margin must be under chrome_max_extent ({CHROME_MAX_EXTENT}) of \
             the width on LeftEdge::Flat ({:.4}) and past it on LeftEdge::FlatWide ({:.4})",
            share(PAGE_FIRST),
            share(PAGE_FIRST_WIDE)
        ));
    }
    if !(PAGE_FIRST_WIDE + PAGE_W < SCROLLBAR_FIRST && SLIVER_W < PAGE_FIRST) {
        bad.push("the page must sit in the flat margin on every left edge".into());
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- f18's shape ----------------------------------------------------------------

/// `f18`'s shape, cause on: the second window's video is wider than the page,
/// so the widest textured run is the video and the crop is the second window.
/// The crop must stay inside the reader's window and keep the page. The
/// second scene adds `f13`'s cause (a flat left margin under
/// `chrome_max_extent`): both causes at once.
#[test]
fn a_video_wider_than_the_page_in_a_second_window_is_never_the_crop() {
    let bad: Vec<String> = [
        Scene {
            left: LeftEdge::Sliver,
            block: Block::Wide,
        },
        Scene {
            left: LeftEdge::Flat,
            block: Block::Wide,
        },
    ]
    .into_iter()
    .flat_map(outside_the_reader)
    .collect();
    assert!(
        bad.is_empty(),
        "MC-066 AC-4 (f18's shape): a second browser window beside the reader holds a \
         textured block wider than the page; the crop must be the reader's page, inside \
         the reader's window (columns 0..{SEAM}):\n{}",
        bad.join("\n")
    );
}

/// `f18`'s cause off: the same scene with the video narrower than the page,
/// or with no video at all. Green today, and must stay green: what moved the
/// crop to the second window was the video's width, and nothing else about
/// the scene.
#[test]
fn control_the_same_second_window_with_a_narrower_video_or_none_is_not_the_crop() {
    let bad: Vec<String> = [
        Scene {
            left: LeftEdge::Sliver,
            block: Block::Narrow,
        },
        Scene {
            left: LeftEdge::Sliver,
            block: Block::Flat,
        },
    ]
    .into_iter()
    .flat_map(outside_the_reader)
    .collect();
    assert!(
        bad.is_empty(),
        "MC-066 AC-4 control (f18's cause off): the crop must be the reader's page, inside \
         the reader's window (columns 0..{SEAM}):\n{}",
        bad.join("\n")
    );
}

// --- f13's shape ----------------------------------------------------------------

/// `f13`'s shape, cause on: the reader's left page margin is one flat value
/// from the image's left edge, under `chrome_max_extent` of the width. The
/// crop spans both windows today, with the video narrower than the page and
/// with no video at all - so the video is not this shape's cause. The crop
/// must stay inside the reader's window and keep the page.
#[test]
fn a_page_whose_flat_left_margin_reaches_the_image_edge_is_not_joined_to_the_second_window() {
    let bad: Vec<String> = [
        Scene {
            left: LeftEdge::Flat,
            block: Block::Narrow,
        },
        Scene {
            left: LeftEdge::Flat,
            block: Block::Flat,
        },
    ]
    .into_iter()
    .flat_map(outside_the_reader)
    .collect();
    assert!(
        bad.is_empty(),
        "MC-066 AC-4 (f13's shape): the reader's flat page margin runs to the image's left \
         edge; the crop must be the reader's page, inside the reader's window (columns \
         0..{SEAM}), and never join it to the second window:\n{}",
        bad.join("\n")
    );
}

/// `f13`'s cause off, two ways: the same scene with a few textured columns at
/// the image's left edge (`f18`'s and MC-052's left edge), or with the flat
/// left margin wider than `chrome_max_extent` of the width. Green today, and
/// must stay green: what joined the windows was a flat left margin narrow
/// enough to be peeled as a chrome strip.
#[test]
fn control_the_same_page_with_a_textured_left_edge_or_a_wider_margin_is_not_joined() {
    let bad: Vec<String> = [
        Scene {
            left: LeftEdge::Sliver,
            block: Block::Narrow,
        },
        Scene {
            left: LeftEdge::FlatWide,
            block: Block::Narrow,
        },
    ]
    .into_iter()
    .flat_map(outside_the_reader)
    .collect();
    assert!(
        bad.is_empty(),
        "MC-066 AC-4 control (f13's cause off): the crop must be the reader's page, inside \
         the reader's window (columns 0..{SEAM}):\n{}",
        bad.join("\n")
    );
}
