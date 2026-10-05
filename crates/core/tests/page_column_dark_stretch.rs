//! MC-083 AC-3: a generated reproduction of the measured cause. A reader page
//! whose art holds a vertical stretch that is **flat over the central band at
//! a value more than `uniform_tolerance` from the site's tone** is cropped
//! whole, seen through [`detect`]. The corpus half is `e03`
//! (`2025-03-07 01_02_31.png`) in `crates/engine/tests/corpus_reader_window.rs`
//! (AC-1); this file is what the required `unit` gate sees of the story.
//!
//! # The cause, as the Lead PO measured it (MC-083 `## Notes`)
//!
//! On `e03` columns 710..729 of the page's own art are near-black: band
//! median 0, flat over the central band, and 11 levels from the site's tone
//! (11) - one more than `uniform_tolerance`. So stage 3c,
//! `flat::locate_column`, sees the page as **two** textured runs, 610..709
//! and 730..1210. Neither has page margin on both sides, so MC-066's rule
//! takes the wider, 730..1210, and widens it leftward only to 727: the
//! columns there are flat over the band and carry the page's (dark) tone on,
//! so MC-055's branch 3 reads the viewport beside the run - and the viewport
//! stage declines there, because the art 610..726 sits in the margin it
//! reads. The widening stops, the crop cuts 117 columns of art, and since
//! `viewport::locate` declines beside the pipeline's column too, the rows
//! stay 0..1440: the bookmarks bar and the taskbar are kept. The kept
//! furniture is a consequence of the cut column.
//!
//! # The fixtures: invented here, the cause turned on and off
//!
//! [`W`] x [`H`], about half of a 2560x1440 screen: textured chrome over the
//! top [`CHROME_END`] rows and a textured taskbar from [`TASKBAR`], across the
//! whole width. Between them, a flat site of [`SITE_TONE`] (`e03`'s 11) and
//! one page, columns [`PAGE_FIRST`] `..` [`PAGE_END`], over every viewport
//! row. Inside the page, [`STRETCH_W`] columns of **stretch**, which split it
//! into a narrow side ([`NARROW_W`]) and a wide side ([`WIDE_W`]); [`Narrow`]
//! says which side is the narrow one (`e03`'s is the left). The art is
//! textured; the [`FRINGE_W`] columns either side of the stretch are **dark**
//! textured art, median 0 and spread well over `min_line_spread` (`e03`'s art
//! "falling to 0 by column 704"), so that the stretch carries the page's tone
//! on and branch 3 is what reads it. The stretch's content is the scene's
//! variable, [`Stretch`]:
//!
//! | scene | stretch | runs over the band | today | the claim |
//! |---|---|---|---|---|
//! | on | flat 0: 11 from the site's tone | two | the wide side only, rows 0..H | **cause on**: the page whole |
//! | control | the fringe's dark texture | one | the page whole | cause off |
//! | guard, site | flat 11, the site's tone | two | the wide side only | not joined: it is page margin |
//! | guard, edge | flat 21, `uniform_tolerance` from it | two | the wide side only | not joined: it is page margin |
//!
//! Nothing else differs between them: the stretch's columns are the only
//! pixels that change (the premise test checks it). So the stretch's tone,
//! relative to the site's, is the variable that moves the crop - and the
//! guard's two scenes pin the other side of the same boundary, a stretch that
//! **is** page margin.
//!
//! **The guard, `f18`'s shape** ([`f18_scene`]): a page in the site's margin
//! beside a second browser window whose video is wider than the page and has
//! page margin on neither side (MC-066's `2025-03-06 12_48_06.png`), with a
//! textured sliver of [`F18_SLIVER_W`] columns at the image's left edge
//! (`f18`'s, and MC-066's and MC-082's `LeftEdge::Sliver`). Textured runs
//! separated by columns that are page margin - the sliver from the page, the
//! page from the video; the crop is the page alone, today and after the fix.
//!
//! The sliver is not decoration. A join rule that reads the page background
//! tone beside the **joined** span rather than beside a run - the Lead PO's
//! trial in MC-083's `## Notes` - reads it there off the video's texture,
//! no longer finds the site's columns between sliver and page to be page
//! margin, and joins everything from column 0 to the second window: the crop
//! becomes `0,58 1278x642`. MC-083's RED measured exactly that on this scene
//! and on MC-066's and MC-082's sliver scenes (`## Handoff`).
//!
//! "Today" is measured on this tree in MC-083's RED (`## Handoff`). The
//! premise test measures every geometric claim above on the fixtures, from
//! the definitions and without the pipeline, so a fixture that drifts goes
//! red rather than making a case vacuous.
//!
//! Luma is the only signal, here and in the fix: colour and chroma are out of
//! scope (MC-083 `## Out of scope`).

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

/// MC-049's page-background share: a column is page background over some
/// rows when at least this share of its pixels lie within
/// `uniform_tolerance` of its own median.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

// --- The geometry, invented here --------------------------------------------

/// The fixture's width: half of 2560.
const W: u32 = 1280;
/// The fixture's height: half of 1440.
const H: u32 = 720;
/// The first row below the browser chrome (`e03`: 115).
const CHROME_END: u32 = 58;
/// The taskbar's first row (`e03`: 1399).
const TASKBAR: u32 = 700;

/// The page's first column.
const PAGE_FIRST: u32 = 400;
/// The narrow side of the page: `e03`'s 610..709 is 100 of a 600-column page.
const NARROW_W: u32 = 120;
/// The stretch: `e03`'s 710..729 is 20 columns.
const STRETCH_W: u32 = 20;
/// The wide side of the page: `e03`'s 730..1209 is 480.
const WIDE_W: u32 = 300;
/// One past the page's last column.
const PAGE_END: u32 = PAGE_FIRST + NARROW_W + STRETCH_W + WIDE_W;
/// Dark textured art on each side of the stretch, inside the page's sides.
const FRINGE_W: u32 = 8;

/// The site's one value (`e03`: 11).
const SITE_TONE: u8 = 11;
/// The stretch with the cause on (`e03`'s band median, 0): more than
/// `uniform_tolerance` from [`SITE_TONE`].
const STRETCH_ON: u8 = 0;
/// The guard's stretch at the edge of the site's tone: [`SITE_TONE`] +
/// [`UNIFORM_TOLERANCE`], the last value still at it.
const STRETCH_EDGE: u8 = SITE_TONE + UNIFORM_TOLERANCE;

// --- `f18`'s shape (MC-066; MC-082's geometry) -------------------------------

/// Textured columns at the image's left edge in [`f18_scene`].
const F18_SLIVER_W: u32 = 4;
/// The page's first column in [`f18_scene`].
const F18_PAGE_FIRST: u32 = 320;
/// The page's width in [`f18_scene`].
const F18_PAGE_W: u32 = 264;
/// The reader's scrollbar: its first column.
const SCROLLBAR_FIRST: u32 = 906;
/// The seam: the second window's first column.
const SEAM: u32 = 914;
/// The second window's scrollbar, to the image's right edge.
const SECOND_SCROLLBAR: u32 = 1272;
/// The video's rows: they cover the central band.
const BLOCK_ROWS: (u32, u32) = (100, 630);
/// A line of textured text across the second window below the video.
const TEXT_ROWS: (u32, u32) = (650, 690);
/// The reader's scrollbar (`e03`: 66).
const SCROLLBAR_TONE: u8 = 66;
/// The seam column.
const SEAM_TONE: u8 = 32;
/// The second window's background.
const SECOND_TONE: u8 = 16;

// --- The fixtures -----------------------------------------------------------

/// Which side of the stretch the page's narrow side is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Narrow {
    /// `e03`'s: the narrow side left of the stretch.
    Left,
    /// The mirror image.
    Right,
}

/// What the stretch's columns hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stretch {
    /// One value on every viewport row.
    Flat(u8),
    /// The fringe's dark texture: the stretch is art like its neighbours.
    Textured,
}

/// One split-page scene.
#[derive(Debug, Clone, Copy)]
struct Scene {
    narrow: Narrow,
    stretch: Stretch,
}

/// The cause on: the stretch flat at [`STRETCH_ON`].
fn on_scene(narrow: Narrow) -> Scene {
    Scene {
        narrow,
        stretch: Stretch::Flat(STRETCH_ON),
    }
}

/// The control: the same scene with the stretch carrying the art's texture.
fn control_scene(narrow: Narrow) -> Scene {
    Scene {
        narrow,
        stretch: Stretch::Textured,
    }
}

/// The guard: the same scene with the stretch flat at `tone`, at the site's
/// tone - page margin.
fn guard_scene(narrow: Narrow, tone: u8) -> Scene {
    Scene {
        narrow,
        stretch: Stretch::Flat(tone),
    }
}

impl Scene {
    /// The stretch's columns, `start .. end`.
    fn stretch_cols(self) -> (u32, u32) {
        let start = match self.narrow {
            Narrow::Left => PAGE_FIRST + NARROW_W,
            Narrow::Right => PAGE_FIRST + WIDE_W,
        };
        (start, start + STRETCH_W)
    }

    /// The wide side's columns, `start .. end`: what today's crop keeps.
    fn wide_cols(self) -> (u32, u32) {
        let (s0, s1) = self.stretch_cols();
        match self.narrow {
            Narrow::Left => (s1, PAGE_END),
            Narrow::Right => (PAGE_FIRST, s0),
        }
    }

    /// The narrow side's columns, `start .. end`.
    fn narrow_cols(self) -> (u32, u32) {
        let (s0, s1) = self.stretch_cols();
        match self.narrow {
            Narrow::Left => (PAGE_FIRST, s0),
            Narrow::Right => (s1, PAGE_END),
        }
    }

    fn render(self) -> Luma {
        let (s0, s1) = self.stretch_cols();
        render(|x, y| {
            if (s0..s1).contains(&x) {
                match self.stretch {
                    Stretch::Flat(v) => v,
                    Stretch::Textured => dark(x, y),
                }
            } else if (s0 - FRINGE_W..s0).contains(&x) || (s1..s1 + FRINGE_W).contains(&x) {
                dark(x, y)
            } else if (PAGE_FIRST..PAGE_END).contains(&x) {
                texture(x, y)
            } else {
                SITE_TONE
            }
        })
    }
}

/// `f18`'s shape: a page in the site's margin, the reader's scrollbar, the
/// seam, and a second window whose video runs from the seam to its
/// scrollbar, wider than the page, with page margin on neither side.
fn f18_scene() -> Luma {
    let page = F18_PAGE_FIRST..F18_PAGE_FIRST + F18_PAGE_W;
    let inside = |(a, b): (u32, u32), v: u32| (a..b).contains(&v);
    render(|x, y| {
        if x < SEAM {
            if x < F18_SLIVER_W || page.contains(&x) {
                texture(x, y)
            } else if x >= SCROLLBAR_FIRST {
                SCROLLBAR_TONE
            } else {
                SITE_TONE
            }
        } else if x == SEAM {
            SEAM_TONE
        } else if x >= SECOND_SCROLLBAR {
            if (SECOND_SCROLLBAR + 2..SECOND_SCROLLBAR + 6).contains(&x) {
                texture(x, y)
            } else {
                255
            }
        } else if inside(TEXT_ROWS, y) || inside(BLOCK_ROWS, y) {
            texture(x, y)
        } else {
            SECOND_TONE
        }
    })
}

/// The whole fixture: textured chrome above [`CHROME_END`] and a textured
/// taskbar from [`TASKBAR`], `body` between them.
fn render(body: impl Fn(u32, u32) -> u8) -> Luma {
    let mut data = Vec::with_capacity((W * H) as usize);
    for y in 0..H {
        for x in 0..W {
            data.push(if (CHROME_END..TASKBAR).contains(&y) {
                body(x, y)
            } else {
                texture(x, y)
            });
        }
    }
    Luma {
        width: W,
        height: H,
        data,
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

/// One pixel of dark textured art: 0 on about 60 % of pixels, 10 ..= 60 on
/// the rest. Median 0, spread far over `min_line_spread`, and nowhere near
/// page background (about 61 % of pixels within `uniform_tolerance` of the
/// median).
fn dark(x: u32, y: u32) -> u8 {
    let h = hash(x, y);
    if h % 10 < 6 {
        0
    } else {
        10 + ((h / 10) % 51) as u8
    }
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

/// Column `x` of `img` over the band.
fn band_column(img: &Luma, x: u32) -> Vec<u8> {
    let b = band();
    (b.y..b.y + b.h)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

/// Mean absolute deviation about the mean.
fn spread(values: &[u8]) -> f64 {
    let n = values.len() as f64;
    let mean = values.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
    values
        .iter()
        .map(|&v| (f64::from(v) - mean).abs())
        .sum::<f64>()
        / n
}

/// The upper median, `sorted[n / 2]`.
fn median(values: &[u8]) -> u8 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

/// MC-049's predicate: at least [`PAGE_BACKGROUND_SHARE`] of `values` within
/// `uniform_tolerance` of their median.
fn page_background(values: &[u8]) -> bool {
    let m = median(values);
    let near = values
        .iter()
        .filter(|&&v| v.abs_diff(m) <= UNIFORM_TOLERANCE)
        .count();
    near as f64 >= PAGE_BACKGROUND_SHARE * values.len() as f64
}

/// The runs of consecutive columns textured over the band, `(first, end)`
/// with `end` exclusive, in order.
fn textured_runs(img: &Luma) -> Vec<(u32, u32)> {
    let mut runs = Vec::new();
    let mut open = None;
    for x in 0..W {
        match (spread(&band_column(img, x)) >= MIN_LINE_SPREAD, open) {
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

/// Whether column `x` of `img` is page margin by the story's definition,
/// read test-side: page background over the band, its median within
/// `uniform_tolerance` of the site's tone.
fn is_page_margin(img: &Luma, x: u32) -> bool {
    let column = band_column(img, x);
    page_background(&column) && median(&column).abs_diff(SITE_TONE) <= UNIFORM_TOLERANCE
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

/// `img`'s crop at `t`, `[x, y, w, h]`.
fn crop(img: &Luma, t: &Tuning) -> [u32; 4] {
    let r = detect(img, t)
        .expect("a reader scene is not one flat colour")
        .rect;
    [r.x, r.y, r.w, r.h]
}

/// Every way `img`'s crop is not the page `first .. end`, at both margins:
/// its columns are not the page's widened by the margin, a row of the
/// browser chrome or the taskbar is kept, or a viewport row is cut.
fn not_the_page(what: &str, img: &Luma, (first, end): (u32, u32)) -> Vec<String> {
    let mut bad = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let [x, y, w, h] = crop(img, &t);
        let (right, bottom) = (x + w, y + h);
        let tag = format!(
            "{what}, margin_px {m}: crop {x},{y} {w}x{h} (columns {x}..{right}, rows {y}..{bottom})"
        );
        let want = (first - m, end + m);
        if (x, right) != want {
            bad.push(format!(
                "{tag} is not the whole page: its columns must be {}..{} (the page {first}..{end} \
                 widened by the margin){}",
                want.0,
                want.1,
                if x > want.0 {
                    format!(", and it cuts {} column(s) of art on the left", x - want.0)
                } else if right < want.1 {
                    format!(
                        ", and it cuts {} column(s) of art on the right",
                        want.1 - right
                    )
                } else {
                    String::new()
                }
            ));
        }
        if y < CHROME_END || bottom > TASKBAR {
            bad.push(format!(
                "{tag} keeps a row of browser chrome or taskbar; the viewport is rows \
                 {CHROME_END}..{TASKBAR}"
            ));
        }
        if y > CHROME_END || bottom < TASKBAR {
            bad.push(format!(
                "{tag} cuts the page's rows {CHROME_END}..{TASKBAR}"
            ));
        }
    }
    bad
}

/// Every way `scene`'s crop is not exactly its wide side's columns, widened
/// by the margin, at both margins: the two runs were joined, or the wide
/// side was cut.
fn not_the_wide_side_alone(scene: Scene) -> Vec<String> {
    let (first, end) = scene.wide_cols();
    let mut bad = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let [x, y, w, h] = crop(&scene.render(), &t);
        let want = (first - m, end + m);
        if (x, x + w) != want {
            bad.push(format!(
                "{scene:?}, margin_px {m}: crop {x},{y} {w}x{h} (columns {x}..{}) is not the wide \
                 side alone, columns {}..{} (the side {first}..{end} widened by the margin): a \
                 stretch at the site's tone is page margin, and two runs either side of it are \
                 two runs",
                x + w,
                want.0,
                want.1
            ));
        }
    }
    bad
}

// --- The premise --------------------------------------------------------------

/// The fixtures are what the module documentation says, measured from the
/// definitions without the pipeline: over the band the page is two textured
/// runs exactly - the narrow side and the wide side - with the stretch flat
/// and one run with it textured; the stretch is one value off the site's tone
/// by more than `uniform_tolerance` with the cause on, and at the tone (page
/// margin) on the guard; the dark fringe beside it is art (not page
/// background) whose median is the on-scene stretch's, so the stretch carries
/// the page's tone on; the columns just outside the page are page margin on
/// both sides; the scenes differ in the stretch's pixels and nowhere else;
/// and in `f18`'s shape the runs are the page, the video and the second
/// scrollbar's texture, the page framed by page margin and the video by none.
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
            && b.y + b.h <= BLOCK_ROWS.1,
        "the band {b:?} must hold no chrome or taskbar row and lie inside the video's rows \
         {BLOCK_ROWS:?}"
    );

    let mut bad = Vec::new();
    if NARROW_W >= WIDE_W || FRINGE_W >= NARROW_W {
        bad.push(format!(
            "the narrow side ({NARROW_W}) must be narrower than the wide side ({WIDE_W}) and \
             wider than the fringe ({FRINGE_W})"
        ));
    }
    if STRETCH_ON.abs_diff(SITE_TONE) <= UNIFORM_TOLERANCE
        || STRETCH_EDGE.abs_diff(SITE_TONE) > UNIFORM_TOLERANCE
    {
        bad.push(format!(
            "the on-scene stretch ({STRETCH_ON}) must be more than {UNIFORM_TOLERANCE} off the \
             site's tone {SITE_TONE}, and the guard's edge stretch ({STRETCH_EDGE}) within it"
        ));
    }

    for narrow in [Narrow::Left, Narrow::Right] {
        let scenes = [
            on_scene(narrow),
            control_scene(narrow),
            guard_scene(narrow, SITE_TONE),
            guard_scene(narrow, STRETCH_EDGE),
        ];
        for scene in scenes {
            let img = scene.render();
            let (s0, s1) = scene.stretch_cols();
            let want = match scene.stretch {
                Stretch::Textured => vec![(PAGE_FIRST, PAGE_END)],
                Stretch::Flat(_) => {
                    let mut sides = vec![scene.narrow_cols(), scene.wide_cols()];
                    sides.sort_unstable();
                    sides
                }
            };
            let got = textured_runs(&img);
            if got != want {
                bad.push(format!(
                    "{scene:?}: textured runs over the band {got:?}, want {want:?}"
                ));
            }
            let outside = (
                is_page_margin(&img, PAGE_FIRST - 1),
                is_page_margin(&img, PAGE_END),
            );
            if outside != (true, true) {
                bad.push(format!(
                    "{scene:?}: the columns just outside the page must be page margin on both \
                     sides; (left, right) {outside:?}"
                ));
            }
            for x in (s0 - FRINGE_W..s0).chain(s1..s1 + FRINGE_W) {
                let column = band_column(&img, x);
                if page_background(&column) || median(&column) != STRETCH_ON {
                    bad.push(format!(
                        "{scene:?}: fringe column {x} must be art (not page background) with \
                         median {STRETCH_ON}; background {}, median {}",
                        page_background(&column),
                        median(&column)
                    ));
                }
            }
            let stretch_margin: Vec<bool> = (s0..s1).map(|x| is_page_margin(&img, x)).collect();
            let want_margin = matches!(scene.stretch, Stretch::Flat(v)
                if v.abs_diff(SITE_TONE) <= UNIFORM_TOLERANCE);
            if stretch_margin.iter().any(|&m| m != want_margin) {
                bad.push(format!(
                    "{scene:?}: every stretch column must be page margin: {want_margin}; got \
                     {stretch_margin:?}"
                ));
            }
            if let Stretch::Flat(v) = scene.stretch {
                let flat = (s0..s1).all(|x| band_column(&img, x).iter().all(|&p| p == v));
                if !flat {
                    bad.push(format!(
                        "{scene:?}: the stretch must be {v} on every band row"
                    ));
                }
            }
        }
        let on = on_scene(narrow);
        let (s0, s1) = on.stretch_cols();
        let a = on.render();
        for other in [
            control_scene(narrow),
            guard_scene(narrow, SITE_TONE),
            guard_scene(narrow, STRETCH_EDGE),
        ] {
            let c = other.render();
            let differ: Vec<u32> = (0..H)
                .flat_map(|y| (0..W).map(move |x| (x, y)))
                .filter(|&(x, y)| a.data[(y * W + x) as usize] != c.data[(y * W + x) as usize])
                .map(|(x, _)| x)
                .collect();
            let outside = differ.iter().filter(|&&x| !(s0..s1).contains(&x)).count();
            if differ.is_empty() || outside > 0 {
                bad.push(format!(
                    "{other:?}: must differ from the on-scene in the stretch's columns {s0}..{s1} \
                     and nowhere else; {} pixels differ, {outside} outside it",
                    differ.len()
                ));
            }
        }
    }

    let f18 = f18_scene();
    let page_end = F18_PAGE_FIRST + F18_PAGE_W;
    let want = vec![
        (0, F18_SLIVER_W),
        (F18_PAGE_FIRST, page_end),
        (SEAM + 1, SECOND_SCROLLBAR),
        (SECOND_SCROLLBAR + 2, SECOND_SCROLLBAR + 6),
    ];
    let got = textured_runs(&f18);
    if got != want {
        bad.push(format!(
            "f18's shape: textured runs over the band {got:?}, want {want:?}"
        ));
    }
    if SECOND_SCROLLBAR - (SEAM + 1) <= F18_PAGE_W {
        bad.push("f18's shape: the video must be wider than the page".to_string());
    }
    let page_sides = (
        is_page_margin(&f18, F18_PAGE_FIRST - 1),
        is_page_margin(&f18, page_end),
    );
    let video_sides = (
        is_page_margin(&f18, SEAM),
        is_page_margin(&f18, SECOND_SCROLLBAR),
    );
    let between = (page_end..SEAM + 1).any(|x| is_page_margin(&f18, x))
        && (F18_SLIVER_W..F18_PAGE_FIRST).all(|x| is_page_margin(&f18, x));
    if page_sides != (true, true) || video_sides != (false, false) || !between {
        bad.push(format!(
            "f18's shape: the page must be framed by page margin (left, right) {page_sides:?}, \
             the video by none {video_sides:?}, and page margin must lie between them \
             ({between})"
        ));
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- The cause on ---------------------------------------------------------------

/// MC-083 AC-3, the measured cause switched on: a stretch inside the page's
/// art, flat over the band at a value 11 from the site's tone, splits the page
/// into two textured runs. The crop must be the whole page - both sides and
/// the stretch, within the viewport's rows - with the narrow side on either
/// side of the stretch. Today it is the wide side alone, at full height
/// (`e03`'s `727,0 483x1440`).
#[test]
fn a_flat_stretch_of_dark_art_off_the_sites_tone_does_not_cut_the_page_in_two() {
    let bad: Vec<String> = [Narrow::Left, Narrow::Right]
        .into_iter()
        .map(on_scene)
        .flat_map(|s| not_the_page(&format!("{s:?}"), &s.render(), (PAGE_FIRST, PAGE_END)))
        .collect();
    assert!(
        bad.is_empty(),
        "MC-083 AC-3 (the measured cause on): a stretch of the page's own art, flat over the \
         band at {STRETCH_ON} - more than uniform_tolerance ({UNIFORM_TOLERANCE}) from the site's \
         tone {SITE_TONE} - with textured art on both sides, is not page margin; the crop must be \
         the whole page, columns {PAGE_FIRST}..{PAGE_END}, rows {CHROME_END}..{TASKBAR}:\n{}",
        bad.join("\n")
    );
}

// --- The control ----------------------------------------------------------------

/// MC-083 AC-3 control, the cause off: the same scenes with the stretch
/// carrying the art's dark texture, nothing else differing (the premise).
/// The page is one run and is cropped whole, today and after the fix. With
/// the on-scene red today and this green, the flat stretch is the variable
/// that cuts the page.
#[test]
fn control_the_same_stretch_carrying_the_arts_texture_is_cropped_whole() {
    let bad: Vec<String> = [Narrow::Left, Narrow::Right]
        .into_iter()
        .map(control_scene)
        .flat_map(|s| not_the_page(&format!("{s:?}"), &s.render(), (PAGE_FIRST, PAGE_END)))
        .collect();
    assert!(
        bad.is_empty(),
        "MC-083 AC-3 control (the cause off): with the stretch textured like the art beside \
         it, the page is one run and the crop must be the whole page, columns \
         {PAGE_FIRST}..{PAGE_END}, rows {CHROME_END}..{TASKBAR}:\n{}",
        bad.join("\n")
    );
}

// --- The guard ------------------------------------------------------------------

/// MC-083 AC-3 guard, `f18`'s shape: a page in the site's margin beside a
/// second window whose video is wider than the page, the two textured runs
/// separated by columns that are page margin (and the reader's scrollbar and
/// the seam). They are not joined: the crop is the page alone, within the
/// viewport's rows, today and after the fix.
#[test]
fn guard_a_page_beside_a_second_window_across_page_margin_stays_the_page_alone() {
    let bad = not_the_page(
        "f18's shape",
        &f18_scene(),
        (F18_PAGE_FIRST, F18_PAGE_FIRST + F18_PAGE_W),
    );
    assert!(
        bad.is_empty(),
        "MC-083 AC-3 guard (no over-joining): a page and a second window separated by page \
         margin are two runs; the crop must be the page alone, columns \
         {F18_PAGE_FIRST}..{}:\n{}",
        F18_PAGE_FIRST + F18_PAGE_W,
        bad.join("\n")
    );
}

/// MC-083 AC-3 guard, the boundary: the on-scene with the stretch at the
/// site's tone - exactly ([`SITE_TONE`]) and at the edge of
/// `uniform_tolerance` ([`STRETCH_EDGE`]) - is page margin by definition, so
/// the two runs either side of it are not joined. The crop is the wide side
/// alone, exactly, today and after the fix. With the on-scene at
/// [`STRETCH_ON`], one level past the other edge (11 from the tone), these
/// pin both sides of the boundary the fix draws.
#[test]
fn guard_a_stretch_at_the_sites_tone_is_page_margin_and_the_runs_beside_it_are_not_joined() {
    let bad: Vec<String> = [Narrow::Left, Narrow::Right]
        .into_iter()
        .flat_map(|n| [guard_scene(n, SITE_TONE), guard_scene(n, STRETCH_EDGE)])
        .flat_map(not_the_wide_side_alone)
        .collect();
    assert!(
        bad.is_empty(),
        "MC-083 AC-3 guard (no over-joining): two textured runs either side of a stretch at the \
         site's tone ({SITE_TONE} or {STRETCH_EDGE}, within uniform_tolerance) are separated by \
         page margin and must not be joined:\n{}",
        bad.join("\n")
    );
}
