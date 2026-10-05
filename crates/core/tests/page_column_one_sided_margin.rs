//! MC-082 AC-3: a generated reproduction of the measured cause. A reader page
//! beside a **second browser window** whose textured run is wider than the
//! page and has a column at the site's tone on **one side only** is cropped
//! to the reader's page, seen through [`detect`]. The corpus half is
//! `crates/engine/tests/corpus_reader_window.rs` (AC-1); this file is what
//! the required `unit` gate sees of the story.
//!
//! # The cause, as the Lead PO measured it (MC-082 `## Notes`)
//!
//! Stage 3c, `flat::page_column`, takes the **widest** textured run whose
//! widened column has a page-margin column - page background at the page
//! background tone - just past it on **either** side (MC-066). On six of
//! MC-081's Eleceed screenshots the second window's (YouTube's) textured run
//! is wider than the reader's page (402 or 595 columns against 200 to 400)
//! and has a column at the site's tone, 11, on one side: right of the video
//! on five, left of it on `e02`. The page has page margin on both sides. So
//! the rule takes the video. On `e01`, `e04`, `e05` and `e06` stage 2 then
//! also peels the page's left margin, because MC-066's guard
//! (`content::page_margin`) asks the same locator where the page is and is
//! told "the video": the same cause, downstream.
//!
//! MC-066's own scenes (`tests/page_column_second_window.rs`) never switch
//! this on: their wide video runs from the seam to the second window's
//! scrollbar, so it has page margin on neither side (`f18`'s shape) and the
//! rule skips it.
//!
//! # The fixtures: invented here, the cause turned on and off
//!
//! MC-066's geometry, [`W`] x [`H`], about half of a 2560x1440 screen:
//! textured chrome over the top [`CHROME_END`] rows and a textured taskbar
//! from [`TASKBAR`], across the whole width. Between them:
//!
//! - **the reader window**, columns `0 .. SEAM`: the page (textured art,
//!   [`PAGE_W`] columns) in a flat page margin of [`MARGIN_TONE`] - the
//!   site's tone - on **both** sides, then the reader's scrollbar
//!   ([`SCROLLBAR_TONE`]). Its left edge is one of [`LeftEdge`];
//! - **the seam**, column [`SEAM`], the second window's first column;
//! - **the second window**: a background of [`SECOND_TONE`], a textured block
//!   (the video, [`VIDEO_W`] columns, **wider than the page**) and a textured
//!   line of text below it, and its own scrollbar at the right edge. Between
//!   the video and either the seam or the scrollbar is a strip of
//!   [`STRIP_W`] columns: the video's one-side margin. [`Strip`] says which
//!   side, and the strip's value is the scene's variable.
//!
//! | scene | strip value | page margin beside the video | today | the claim |
//! |---|---|---|---|---|
//! | on | [`SECOND_TONE`], 16: within `uniform_tolerance` of 11 | one side | the video | **cause on** |
//! | control | [`OFF_TONE`], 22: 11 more than 11 | neither side | the page | cause off (`f18`'s shape) |
//!
//! Both scenes are built for each [`Strip`] side and each [`LeftEdge`]: the
//! [`LeftEdge::Flat`] edge is `e01`'s, where stage 2 joins in. Nothing else
//! differs between on and control: the strip's columns are the only pixels
//! that change, so the one-sided margin is the variable that moves the crop.
//!
//! **The fallback** ([`fallback_scene`]): a page whose only page margin is on
//! one side - a flat panel off the site's tone hugs its right edge - beside a
//! video wider than it with margin on neither side, so **no** run has page
//! margin on both sides (`2025-08-05 00_11_13.webp`'s shape: four `tuning`
//! entries have no run framed on both sides, and MC-066's rule is what keeps
//! them where they are). It is cropped as it is today.
//!
//! "Today" is measured on this tree in MC-082's RED (`## Handoff`). The
//! premise test measures every geometric claim above on the fixtures, from
//! the definitions and without the pipeline, so a fixture that drifts goes
//! red rather than making a case vacuous.
//!
//! Luma is the only signal, here and in the fix: colour and chroma are out of
//! scope (MC-082 `## Out of scope`).

use cropper_core::flat::central_band;
use cropper_core::{Luma, Rect, Tuning, detect};

// --- Settled constants, read out --------------------------------------------

/// `Tuning::default().min_line_spread`: a column is textured over the band
/// at or above this mean absolute deviation. Asserted equal to the shipped
/// value in the premise test.
const MIN_LINE_SPREAD: f64 = 8.0;

/// `Tuning::default().uniform_tolerance`: how far a column's value may lie
/// from the page background tone and still be at it. Asserted equal to the
/// shipped value in the premise test.
const UNIFORM_TOLERANCE: u8 = 10;

// --- The geometry, invented here (MC-066's, `page_column_second_window.rs`) ---

/// The fixture's width: half of 2560.
const W: u32 = 1280;
/// The fixture's height: half of 1440.
const H: u32 = 720;
/// The first row below the browser chrome (the six: 115).
const CHROME_END: u32 = 58;
/// The taskbar's first row (the six: 1392 or 1399 to the scrollbar, 1400 to
/// the taskbar).
const TASKBAR: u32 = 700;

/// The page's first column: 320 of 1280.
const PAGE_FIRST: u32 = 320;
/// The page's width: 264, narrower than the video (`e01`'s page 267 and
/// video 402; `e02`'s 345 and 595).
const PAGE_W: u32 = 264;
/// Textured columns at the image's left edge on [`LeftEdge::Sliver`].
const SLIVER_W: u32 = 4;
/// The reader's scrollbar: its first column.
const SCROLLBAR_FIRST: u32 = 906;
/// The seam: the second window's first column, one flat column of
/// [`SEAM_TONE`]. The reader's window is `0 .. SEAM`.
const SEAM: u32 = 914;
/// The second window's scrollbar, to the image's right edge: two columns of
/// 255, four textured, two of 255.
const SECOND_SCROLLBAR: u32 = 1272;
/// The video's one-side margin: this many columns of the strip's value
/// between the video and the seam ([`Strip::Left`]) or the scrollbar
/// ([`Strip::Right`]).
const STRIP_W: u32 = 10;
/// The video's width: the second window less the seam and the strip, 347
/// columns, wider than the page's [`PAGE_W`].
const VIDEO_W: u32 = SECOND_SCROLLBAR - (SEAM + 1) - STRIP_W;
/// The video's rows: they cover the central band.
const BLOCK_ROWS: (u32, u32) = (100, 630);
/// A line of textured text across the second window below the video.
/// Outside the central band.
const TEXT_ROWS: (u32, u32) = (650, 690);

/// The flat panel hugging the page's right edge in [`fallback_scene`]: its
/// columns `PAGE_FIRST + PAGE_W .. PANEL_END`.
const PANEL_END: u32 = PAGE_FIRST + PAGE_W + 56;

/// The page margin's one value: the site's tone (the six: 11).
const MARGIN_TONE: u8 = 11;
/// The reader's scrollbar (the six: 66).
const SCROLLBAR_TONE: u8 = 66;
/// The seam column.
const SEAM_TONE: u8 = 32;
/// The second window's background (MC-066's `f13`: 16 to 18). Within
/// `uniform_tolerance` of [`MARGIN_TONE`]: a column of it beside the video is
/// a page-margin column. That is the cause.
const SECOND_TONE: u8 = 16;
/// The strip's value with the cause off: [`MARGIN_TONE`] +
/// [`UNIFORM_TOLERANCE`] + 1, the first value more than `uniform_tolerance`
/// off the site's tone.
const OFF_TONE: u8 = MARGIN_TONE + UNIFORM_TOLERANCE + 1;
/// The fallback's panel: flat, and off the site's tone by far more than
/// `uniform_tolerance`.
const PANEL_TONE: u8 = 40;

// --- The fixtures -----------------------------------------------------------

/// What the reader window has at the image's left edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftEdge {
    /// [`SLIVER_W`] textured columns, then the flat margin: stage 2 peels
    /// nothing (`f18`, MC-052's split screens).
    Sliver,
    /// The flat margin from column 0, under `chrome_max_extent` of the width:
    /// stage 2's chrome strip, which MC-066's `page_margin` guard keeps only
    /// if the locator finds the page (`e01`, `e04`, `e05`, `e06`).
    Flat,
}

/// Which side of the video the strip lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strip {
    /// Between the video and the second window's scrollbar (`e01`, `e04` to
    /// `e07`: right of the video).
    Right,
    /// Between the seam and the video (`e02`: left of the video).
    Left,
}

/// One scene.
#[derive(Debug, Clone, Copy)]
struct Scene {
    left: LeftEdge,
    strip: Strip,
    /// The strip's value: [`SECOND_TONE`] with the cause on, [`OFF_TONE`]
    /// with it off.
    strip_tone: u8,
    /// [`fallback_scene`]'s shape instead: no strip (the video runs from the
    /// seam to the scrollbar) and a flat panel of [`PANEL_TONE`] hugging the
    /// page's right edge.
    fallback: bool,
}

/// The scene with the cause on.
fn on_scene(left: LeftEdge, strip: Strip) -> Scene {
    Scene {
        left,
        strip,
        strip_tone: SECOND_TONE,
        fallback: false,
    }
}

/// The same scene with the strip moved off the site's tone.
fn control_scene(left: LeftEdge, strip: Strip) -> Scene {
    Scene {
        strip_tone: OFF_TONE,
        ..on_scene(left, strip)
    }
}

/// The fallback: page margin on the page's left only, and a video with page
/// margin on neither side.
fn fallback_scene() -> Scene {
    Scene {
        left: LeftEdge::Sliver,
        strip: Strip::Right,
        strip_tone: SECOND_TONE,
        fallback: true,
    }
}

impl Scene {
    /// The page art: its columns, over every row of the reader's viewport.
    fn art() -> Rect {
        Rect {
            x: PAGE_FIRST,
            y: CHROME_END,
            w: PAGE_W,
            h: TASKBAR - CHROME_END,
        }
    }

    /// The strip's columns, `start .. end`; empty on the fallback.
    fn strip_cols(self) -> (u32, u32) {
        match (self.fallback, self.strip) {
            (true, _) => (SEAM + 1, SEAM + 1),
            (false, Strip::Left) => (SEAM + 1, SEAM + 1 + STRIP_W),
            (false, Strip::Right) => (SECOND_SCROLLBAR - STRIP_W, SECOND_SCROLLBAR),
        }
    }

    /// The video's columns, `start .. end`.
    fn video_cols(self) -> (u32, u32) {
        match (self.fallback, self.strip) {
            (true, _) => (SEAM + 1, SECOND_SCROLLBAR),
            (false, Strip::Left) => (SEAM + 1 + STRIP_W, SECOND_SCROLLBAR),
            (false, Strip::Right) => (SEAM + 1, SECOND_SCROLLBAR - STRIP_W),
        }
    }

    fn render(self) -> Luma {
        let page = PAGE_FIRST..PAGE_FIRST + PAGE_W;
        let panel = PAGE_FIRST + PAGE_W..PANEL_END;
        let (s0, s1) = self.strip_cols();
        let (v0, v1) = self.video_cols();
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
                    } else if self.fallback && panel.contains(&x) {
                        PANEL_TONE
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
                } else if inside(TEXT_ROWS, y) || (inside((v0, v1), x) && inside(BLOCK_ROWS, y)) {
                    texture(x, y)
                } else if inside((s0, s1), x) {
                    self.strip_tone
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

/// Whether column `x` of `img` is one value over the band, within
/// `uniform_tolerance` of the site's tone: a page-margin column, by the
/// story's definition, read test-side.
fn at_site_tone(img: &Luma, x: u32) -> bool {
    let b = band();
    (b.y..b.y + b.h)
        .all(|y| img.data[(y * img.width + x) as usize].abs_diff(MARGIN_TONE) <= UNIFORM_TOLERANCE)
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

/// `scene`'s crop at `t`, `[x, y, w, h]`.
fn crop(scene: Scene, t: &Tuning) -> [u32; 4] {
    let r = detect(&scene.render(), t)
        .expect("a two-window scene is not one flat colour")
        .rect;
    [r.x, r.y, r.w, r.h]
}

/// Every way `scene`'s crop is not the reader's page column, at both margins:
/// its columns are not the page's ([`PAGE_FIRST`] `..` `+ PAGE_W`, widened
/// by the margin), a column of the seam or the second window is kept, or a
/// row is outside the reader's viewport (`CHROME_END .. TASKBAR`) or the
/// page art's rows are cut.
fn not_the_page_column(scene: Scene) -> Vec<String> {
    let art = Scene::art();
    let mut bad = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let [x, y, w, h] = crop(scene, &t);
        let (end, bottom) = (x + w, y + h);
        let tag = format!(
            "{scene:?}, margin_px {m}: crop {x},{y} {w}x{h} (columns {x}..{end}, rows {y}..{bottom})"
        );
        let want = (art.x - m, art.x + art.w + m);
        if (x, end) != want {
            bad.push(format!(
                "{tag} is not the page column: its columns must be {}..{} (the page {}..{} \
                 widened by the margin)",
                want.0,
                want.1,
                art.x,
                art.x + art.w
            ));
        }
        if end > SEAM {
            bad.push(format!(
                "{tag} keeps {} column(s) of the second window, which starts at the seam, \
                 column {SEAM}; the crop must end at {SEAM} or left of it",
                end - SEAM.max(x)
            ));
        }
        if y > art.y || bottom < art.y + art.h {
            bad.push(format!(
                "{tag} cuts the page art's rows {}..{}",
                art.y,
                art.y + art.h
            ));
        }
        if y < CHROME_END || bottom > TASKBAR {
            bad.push(format!(
                "{tag} keeps a row of browser chrome or taskbar; the reader's viewport is \
                 rows {CHROME_END}..{TASKBAR}"
            ));
        }
    }
    bad
}

/// Every scene the on/control pair is built over.
fn variants() -> [(LeftEdge, Strip); 4] {
    [
        (LeftEdge::Sliver, Strip::Right),
        (LeftEdge::Sliver, Strip::Left),
        (LeftEdge::Flat, Strip::Right),
        (LeftEdge::Flat, Strip::Left),
    ]
}

// --- The premise --------------------------------------------------------------

/// The fixtures are what the module documentation says, measured from the
/// definitions without the pipeline: over the band the textured runs are
/// exactly the page, the video, the sliver (where there is one) and the
/// second scrollbar's texture; the video is wider than the page; the columns
/// just outside the page are at the site's tone on both sides (on the
/// fallback, on the left only); the column just outside the video on the
/// strip's side is at the site's tone with the cause on and is not with it
/// off, and the column on its other side never is; and the on and control
/// scenes differ in the strip's pixels and nowhere else.
#[test]
fn the_scenes_measure_as_the_cause_claims() {
    let shipped = Tuning::default();
    assert!(
        (f64::from(shipped.min_line_spread) - MIN_LINE_SPREAD).abs() < 1e-9
            && shipped.uniform_tolerance == UNIFORM_TOLERANCE,
        "the copied constants must be the shipped ones: min_line_spread {} (copied \
         {MIN_LINE_SPREAD}), uniform_tolerance {} (copied {UNIFORM_TOLERANCE})",
        shipped.min_line_spread,
        shipped.uniform_tolerance
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
    if VIDEO_W <= PAGE_W {
        bad.push(format!(
            "the video ({VIDEO_W}) must be wider than the page ({PAGE_W})"
        ));
    }
    if SECOND_TONE.abs_diff(MARGIN_TONE) > UNIFORM_TOLERANCE
        || OFF_TONE.abs_diff(MARGIN_TONE) <= UNIFORM_TOLERANCE
        || PANEL_TONE.abs_diff(MARGIN_TONE) <= UNIFORM_TOLERANCE
    {
        bad.push(format!(
            "the strip must be at the site's tone {MARGIN_TONE} with the cause on \
             ({SECOND_TONE}) and more than {UNIFORM_TOLERANCE} off it with the cause off \
             ({OFF_TONE}); the panel ({PANEL_TONE}) off it"
        ));
    }
    let page_end = PAGE_FIRST + PAGE_W;
    let mut scenes: Vec<Scene> = variants()
        .into_iter()
        .flat_map(|(l, s)| [on_scene(l, s), control_scene(l, s)])
        .collect();
    scenes.push(fallback_scene());
    for scene in scenes {
        let img = scene.render();
        let (v0, v1) = scene.video_cols();
        let mut want = Vec::new();
        if scene.left == LeftEdge::Sliver {
            want.push((0, SLIVER_W));
        }
        want.push((PAGE_FIRST, page_end));
        want.push((v0, v1));
        want.push((SECOND_SCROLLBAR + 2, SECOND_SCROLLBAR + 6));
        let got = textured_runs(&img);
        if got != want {
            bad.push(format!(
                "{scene:?}: textured runs over the band {got:?}, want {want:?}"
            ));
        }
        let page_sides = (
            at_site_tone(&img, PAGE_FIRST - 1),
            at_site_tone(&img, page_end),
        );
        let want_page = (true, !scene.fallback);
        if page_sides != want_page {
            bad.push(format!(
                "{scene:?}: the columns just outside the page at the site's tone (left, \
                 right) {page_sides:?}, want {want_page:?}"
            ));
        }
        let video_sides = (at_site_tone(&img, v0 - 1), at_site_tone(&img, v1));
        let cause = !scene.fallback && scene.strip_tone == SECOND_TONE;
        let want_video = match scene.strip {
            Strip::Left => (cause, false),
            Strip::Right => (false, cause),
        };
        if video_sides != want_video {
            bad.push(format!(
                "{scene:?}: the columns just outside the video at the site's tone (left, \
                 right) {video_sides:?}, want {want_video:?}"
            ));
        }
    }
    for (l, s) in variants() {
        let (on, off) = (on_scene(l, s), control_scene(l, s));
        let (a, c) = (on.render(), off.render());
        let (s0, s1) = on.strip_cols();
        let differ: Vec<(u32, u32)> = (0..H)
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .filter(|&(x, y)| a.data[(y * W + x) as usize] != c.data[(y * W + x) as usize])
            .collect();
        let outside = differ
            .iter()
            .filter(|&&(x, _)| !(s0..s1).contains(&x))
            .count();
        if differ.is_empty() || outside > 0 {
            bad.push(format!(
                "{l:?}/{s:?}: the on and control scenes must differ in the strip's columns \
                 {s0}..{s1} and nowhere else; {} pixels differ, {outside} outside it",
                differ.len()
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- The cause on ---------------------------------------------------------------

/// MC-082 AC-3, the measured cause switched on: the second window's video is
/// wider than the reader's page and has a column at the site's tone on one
/// side only, the seam or its scrollbar on the other; the page has page
/// margin on both sides. The crop must be the page column and end left of
/// the seam, on either side of the video and with or without stage 2's flat
/// left margin. Today it is the video.
#[test]
fn a_wider_second_window_run_with_page_margin_on_one_side_loses_to_a_page_framed_on_both() {
    let bad: Vec<String> = variants()
        .into_iter()
        .map(|(l, s)| on_scene(l, s))
        .flat_map(not_the_page_column)
        .collect();
    assert!(
        bad.is_empty(),
        "MC-082 AC-3 (the measured cause on): a second window's textured run wider than the \
         page, with a column at the site's tone on one side only, must not be taken over a \
         page with page margin on both sides; the crop must be the page column, inside the \
         reader's window (columns 0..{SEAM}):\n{}",
        bad.join("\n")
    );
}

// --- The control ----------------------------------------------------------------

/// MC-082 AC-3 control, the cause off: the same scenes with the strip moved
/// off the site's tone by more than `uniform_tolerance`, so the video has
/// page margin on neither side (`f18`'s shape). The crop is the page column,
/// today and after the fix. With the on-scene red today and this green, and
/// the two differing in the strip's pixels alone (the premise), the
/// one-sided margin is the variable that moves the crop.
#[test]
fn control_the_same_video_with_its_margin_column_off_the_sites_tone_is_not_the_crop() {
    let bad: Vec<String> = variants()
        .into_iter()
        .map(|(l, s)| control_scene(l, s))
        .flat_map(not_the_page_column)
        .collect();
    assert!(
        bad.is_empty(),
        "MC-082 AC-3 control (the cause off): with the video's margin column more than \
         uniform_tolerance off the site's tone the crop must be the page column, inside the \
         reader's window (columns 0..{SEAM}):\n{}",
        bad.join("\n")
    );
}

// --- The fallback ---------------------------------------------------------------

/// MC-082 AC-3 fallback: a page with page margin on one side only (a flat
/// panel off the site's tone hugs its right edge), beside a video wider than
/// it with page margin on neither side, so no run is framed on both sides.
/// It is cropped as it is today: the page column, exactly, at both margins.
/// Pinned to the rects measured on this tree in MC-082's RED.
#[test]
fn fallback_a_page_with_margin_on_one_side_and_no_run_framed_on_both_is_cropped_as_today() {
    let scene = fallback_scene();
    let pinned: [(u32, [u32; 4]); 2] = [
        (0, [PAGE_FIRST, CHROME_END, PAGE_W, TASKBAR - CHROME_END]),
        (
            3,
            [PAGE_FIRST - 3, CHROME_END, PAGE_W + 6, TASKBAR - CHROME_END],
        ),
    ];
    let bad: Vec<String> = both_margins()
        .iter()
        .zip(pinned)
        .filter_map(|(t, (m, want))| {
            let got = crop(scene, t);
            (got != want).then(|| {
                format!("margin_px {m}: crop {got:?}, as today {want:?} (the page column)")
            })
        })
        .collect();
    assert!(
        bad.is_empty(),
        "MC-082 AC-3 fallback: where no textured run has page margin on both sides, the page \
         with page margin on one side is cropped as it is today (MC-066's rule), not the \
         wider video with none and not the whole rect:\n{}",
        bad.join("\n")
    );
}
