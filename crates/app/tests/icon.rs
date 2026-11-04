//! MC-086: the icon `gui::viewport` hands Windows for the title bar, the
//! taskbar button and alt-tab.
//!
//! # Where every number comes from
//!
//! Every coordinate and colour below is read out of
//! `docs/wiki/design/icon.md`, "Sample points". They are settled: they were
//! derived from the SVG's geometry (one unit is 4 px at 256), not measured off
//! a renderer, and they are never moved to suit one. A sample that lands on an
//! anti-aliased edge in the real raster is a finding to report, not a
//! coordinate to change.
//!
//! # How the checks prove they can tell shapes apart
//!
//! The checks are functions over any `egui::IconData`, returning the checks
//! that failed. The real icon must fail none of them. Each negative control
//! named in the story's acceptance criteria is an image built pixel by pixel
//! in this file from plain rectangles and run through the same functions, and
//! its test asserts exactly which checks it fails. A positive control - the
//! same rectangles assembled into the intended icon - must fail none, so a
//! checker that rejects everything cannot pass the controls either.

use std::collections::BTreeSet;

use eframe::egui;
use egui::IconData;
use manhwa_cropper::gui;

// --- Settled values, `icon.md` -----------------------------------------------

/// The runtime icon is the 256 raster (`icon.md`, "Rasters").
const SIZE: u32 = 256;

/// `primary` light, `#0067C0` (`tokens.md`), the tile.
const TILE: [u8; 3] = [0x00, 0x67, 0xC0];
/// The crop corners, opaque white.
const WHITE: [u8; 3] = [0xFF, 0xFF, 0xFF];
/// White at 35 % over `#0067C0`: 0.35 * 255 = 89.25, 103 + 0.35 * 152 =
/// 156.2, 192 + 0.35 * 63 = 214.05.
const PANEL: [u8; 3] = [89, 156, 214];
/// Each panel channel may sit within this much of `PANEL`.
const PANEL_TOLERANCE: u8 = 2;

/// Outside the tile's rounded corners: alpha 0.
const CORNERS: [(u32, u32); 4] = [(0, 0), (255, 0), (0, 255), (255, 255)];
/// The tile, clear of panel and brackets: `#0067C0`, alpha 255.
const TILE_SAMPLES: [(u32, u32); 4] = [(32, 128), (128, 32), (224, 128), (128, 224)];
/// The vertical arm of each bracket: `#FFFFFF`, alpha 255.
const ARMS: [(u32, u32); 4] = [(56, 72), (200, 72), (56, 184), (200, 184)];
/// Between brackets on the left edge (unit 14,32) and the top edge (unit
/// 32,13): `#0067C0`, alpha 255.
const GAPS: [(u32, u32); 2] = [(56, 128), (128, 52)];
/// The panel: about (89,156,214), alpha 255.
const PANEL_SAMPLE: (u32, u32) = (128, 128);

// --- The checks --------------------------------------------------------------

/// One failed check: which sample, and what it read.
#[derive(Debug)]
struct Failure {
    check: String,
    detail: String,
}

fn ids(failures: &[Failure]) -> BTreeSet<String> {
    failures.iter().map(|f| f.check.clone()).collect()
}

fn names(kind: &str, points: &[(u32, u32)]) -> BTreeSet<String> {
    points
        .iter()
        .map(|&(x, y)| format!("{kind} ({x},{y})"))
        .collect()
}

/// The RGBA at (x, y), or `None` if the buffer does not hold that pixel.
fn pixel(icon: &IconData, x: u32, y: u32) -> Option<[u8; 4]> {
    if x >= icon.width || y >= icon.height {
        return None;
    }
    let at = ((y * icon.width + x) * 4) as usize;
    let px = icon.rgba.get(at..at + 4)?;
    Some([px[0], px[1], px[2], px[3]])
}

/// Every sample in `points` must be exactly `rgb` at alpha 255.
fn exact_opaque(icon: &IconData, kind: &str, points: &[(u32, u32)], rgb: [u8; 3]) -> Vec<Failure> {
    let want = [rgb[0], rgb[1], rgb[2], 255];
    points
        .iter()
        .filter_map(|&(x, y)| {
            let got = pixel(icon, x, y);
            (got != Some(want)).then(|| Failure {
                check: format!("{kind} ({x},{y})"),
                detail: format!("expected RGBA {want:?}, read {got:?}"),
            })
        })
        .collect()
}

/// AC-1: the window settings carry a 256 x 256 RGBA icon.
fn size_failures(viewport: &egui::ViewportBuilder) -> Vec<Failure> {
    let Some(icon) = viewport.icon.as_deref() else {
        return vec![Failure {
            check: "icon present".into(),
            detail: "viewport().icon is None: the window hands Windows no icon".into(),
        }];
    };
    let mut failures = Vec::new();
    if (icon.width, icon.height) != (SIZE, SIZE) {
        failures.push(Failure {
            check: "256 x 256".into(),
            detail: format!("icon is {} x {}", icon.width, icon.height),
        });
    }
    let want_len = (SIZE * SIZE * 4) as usize;
    if icon.rgba.len() != want_len {
        failures.push(Failure {
            check: "RGBA buffer length".into(),
            detail: format!("buffer is {} bytes, expected {want_len}", icon.rgba.len()),
        });
    }
    failures
}

/// AC-2, corners: alpha 0 outside the tile's rounded corners.
fn corner_failures(icon: &IconData) -> Vec<Failure> {
    CORNERS
        .iter()
        .filter_map(|&(x, y)| {
            let got = pixel(icon, x, y);
            (got.map(|p| p[3]) != Some(0)).then(|| Failure {
                check: format!("corner ({x},{y})"),
                detail: format!("expected alpha 0, read {got:?}"),
            })
        })
        .collect()
}

/// AC-2, tile: exactly `#0067C0`, opaque.
fn tile_failures(icon: &IconData) -> Vec<Failure> {
    exact_opaque(icon, "tile", &TILE_SAMPLES, TILE)
}

/// AC-3, arms: exactly `#FFFFFF`, opaque.
fn arm_failures(icon: &IconData) -> Vec<Failure> {
    exact_opaque(icon, "arm", &ARMS, WHITE)
}

/// AC-3, gaps: exactly `#0067C0`, opaque - corners, not a frame.
fn gap_failures(icon: &IconData) -> Vec<Failure> {
    exact_opaque(icon, "gap", &GAPS, TILE)
}

/// AC-4: the panel is opaque and each channel within 2 of (89,156,214).
fn panel_failures(icon: &IconData) -> Vec<Failure> {
    let (x, y) = PANEL_SAMPLE;
    let got = pixel(icon, x, y);
    let ok = got.is_some_and(|p| {
        p[3] == 255
            && p[..3]
                .iter()
                .zip(PANEL)
                .all(|(&have, want)| have.abs_diff(want) <= PANEL_TOLERANCE)
    });
    if ok {
        Vec::new()
    } else {
        vec![Failure {
            check: format!("panel ({x},{y})"),
            detail: format!(
                "expected alpha 255 and RGB within {PANEL_TOLERANCE} of {PANEL:?}, read {got:?}"
            ),
        }]
    }
}

/// Every pixel check of AC-2 to AC-4.
fn pixel_failures(icon: &IconData) -> Vec<Failure> {
    let mut all = corner_failures(icon);
    all.extend(tile_failures(icon));
    all.extend(arm_failures(icon));
    all.extend(gap_failures(icon));
    all.extend(panel_failures(icon));
    all
}

fn report(failures: &[Failure]) -> String {
    failures
        .iter()
        .map(|f| format!("\n  {}: {}", f.check, f.detail))
        .collect()
}

// --- The real icon -----------------------------------------------------------

/// The icon `viewport()` carries; panics with a bug-report message if none.
fn real_icon() -> IconData {
    let viewport = gui::viewport();
    let icon = viewport.icon.as_deref().expect(
        "gui::viewport() carries no icon: the title bar, taskbar and alt-tab show the default",
    );
    icon.clone()
}

#[test]
fn the_window_hands_windows_a_256_by_256_rgba_icon() {
    let failures = size_failures(&gui::viewport());
    assert!(
        failures.is_empty(),
        "AC-1: the window icon is not a 256 x 256 RGBA raster:{}",
        report(&failures)
    );
}

#[test]
fn the_icon_is_a_blue_tile_with_transparent_rounded_corners() {
    let icon = real_icon();
    let mut failures = corner_failures(&icon);
    failures.extend(tile_failures(&icon));
    assert!(
        failures.is_empty(),
        "AC-2: the icon is not a #0067C0 tile with transparent corners:{}",
        report(&failures)
    );
}

#[test]
fn the_icon_has_four_white_crop_brackets_at_the_corners() {
    let icon = real_icon();
    let failures = arm_failures(&icon);
    assert!(
        failures.is_empty(),
        "AC-3: a crop bracket's vertical arm is not opaque white:{}",
        report(&failures)
    );
}

#[test]
fn the_brackets_are_four_corners_not_a_closed_frame() {
    let icon = real_icon();
    let failures = gap_failures(&icon);
    assert!(
        failures.is_empty(),
        "AC-3: the tile shows through between the brackets, or it is a frame:{}",
        report(&failures)
    );
}

#[test]
fn the_icon_has_the_faint_panel_in_the_middle() {
    let icon = real_icon();
    let failures = panel_failures(&icon);
    assert!(
        failures.is_empty(),
        "AC-4: the 35 % white panel is missing or the wrong shade:{}",
        report(&failures)
    );
}

// --- Control images, built from rectangles -----------------------------------

/// A 256 x 256 canvas, transparent black.
struct Canvas(Vec<u8>);

impl Canvas {
    fn new() -> Self {
        Self(vec![0; (SIZE * SIZE * 4) as usize])
    }

    /// Paint `rgb` opaque over every pixel for which `inside` holds.
    fn paint(mut self, rgb: [u8; 3], inside: impl Fn(u32, u32) -> bool) -> Self {
        for y in 0..SIZE {
            for x in 0..SIZE {
                if inside(x, y) {
                    let at = ((y * SIZE + x) * 4) as usize;
                    self.0[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
                }
            }
        }
        self
    }

    fn icon(self) -> IconData {
        IconData {
            rgba: self.0,
            width: SIZE,
            height: SIZE,
        }
    }
}

/// Inclusive pixel ranges.
fn within(v: u32, lo: u32, hi: u32) -> bool {
    (lo..=hi).contains(&v)
}

/// The tile: units 2..62 (pixels 8..=247), corner radius 14 units (56 px).
fn in_tile(x: u32, y: u32) -> bool {
    if !within(x, 8, 247) || !within(y, 8, 247) {
        return false;
    }
    let (cx, cy) = (f64::from(x) + 0.5, f64::from(y) + 0.5);
    let nearest_x = cx.clamp(64.0, 192.0);
    let nearest_y = cy.clamp(64.0, 192.0);
    (cx - nearest_x).hypot(cy - nearest_y) <= 56.0
}

/// The panel: units 22..42 across, 14..50 down (pixels 88..=167, 56..=199).
fn in_panel(x: u32, y: u32) -> bool {
    within(x, 88, 167) && within(y, 56, 199)
}

/// The bracket line's bands, stroke 4 units centred on unit 14 and unit 50:
/// pixels 48..=63 and 192..=207.
fn on_band(v: u32) -> bool {
    within(v, 48, 63) || within(v, 192, 207)
}

/// Four L brackets: each arm 8 units from its corner plus the stroke's
/// half-width, so pixels 48..=95 (or 160..=207) along the arm.
fn in_brackets(x: u32, y: u32) -> bool {
    let near = |v: u32| within(v, 48, 95) || within(v, 160, 207);
    (on_band(x) && near(y)) || (on_band(y) && near(x))
}

/// The whole perimeter of the bracket square, units 14..50, stroke 4.
fn in_frame(x: u32, y: u32) -> bool {
    let span = |v: u32| within(v, 48, 207);
    (on_band(x) && span(y)) || (on_band(y) && span(x))
}

fn tile() -> Canvas {
    Canvas::new().paint(TILE, in_tile)
}

fn the_intended_icon() -> IconData {
    tile()
        .paint(PANEL, in_panel)
        .paint(WHITE, in_brackets)
        .icon()
}

// --- The controls ------------------------------------------------------------

#[test]
fn a_viewport_with_no_icon_fails_the_size_check() {
    // Today's viewport: `ViewportBuilder::default()` with nothing set.
    let failures = size_failures(&egui::ViewportBuilder::default());
    assert_eq!(
        ids(&failures),
        BTreeSet::from(["icon present".to_owned()]),
        "AC-1 control: a builder with no icon must fail the presence check, and only that"
    );
}

#[test]
fn a_wrong_size_icon_fails_the_size_check() {
    let icon = IconData {
        rgba: vec![0; 128 * 128 * 4],
        width: 128,
        height: 128,
    };
    let failures = size_failures(&egui::ViewportBuilder::default().with_icon(icon));
    assert_eq!(
        ids(&failures),
        BTreeSet::from(["256 x 256".to_owned(), "RGBA buffer length".to_owned()]),
        "AC-1 control: a 128 x 128 icon must fail both the size and the buffer length"
    );
}

#[test]
fn the_checks_pass_an_icon_drawn_to_the_design() {
    // Positive control: without it, a checker that rejected everything would
    // satisfy every negative control below.
    let icon = the_intended_icon();
    let failures = pixel_failures(&icon);
    assert!(
        failures.is_empty(),
        "positive control: the rectangles drawn to icon.md fail a check:{}",
        report(&failures)
    );
    let viewport = egui::ViewportBuilder::default().with_icon(icon);
    assert!(
        size_failures(&viewport).is_empty(),
        "positive control: a 256 x 256 RGBA icon fails AC-1"
    );
}

#[test]
fn a_solid_square_fails_on_all_four_corners() {
    let icon = Canvas::new().paint(TILE, |_, _| true).icon();
    assert_eq!(
        ids(&corner_failures(&icon)),
        names("corner", &CORNERS),
        "AC-2 control: a solid opaque #0067C0 square must fail every corner"
    );
    assert!(
        tile_failures(&icon).is_empty(),
        "AC-2 control: the solid square is the right blue, so only corners may fail"
    );
}

#[test]
fn a_tile_without_brackets_fails_all_four_arm_samples() {
    let icon = tile().icon();
    assert_eq!(
        ids(&arm_failures(&icon)),
        names("arm", &ARMS),
        "AC-3 control (a): the bare tile must fail every arm sample"
    );
    assert!(
        corner_failures(&icon).is_empty() && tile_failures(&icon).is_empty(),
        "AC-3 control (a): the bare tile must still pass AC-2"
    );
}

#[test]
fn a_closed_white_frame_passes_the_arms_and_fails_both_gaps() {
    let icon = tile().paint(PANEL, in_panel).paint(WHITE, in_frame).icon();
    assert!(
        arm_failures(&icon).is_empty(),
        "AC-3 control (b): a frame covers every arm sample:{}",
        report(&arm_failures(&icon))
    );
    assert_eq!(
        ids(&pixel_failures(&icon)),
        names("gap", &GAPS),
        "AC-3 control (b): a frame must fail exactly the two gap samples, nothing else"
    );
}

#[test]
fn an_icon_without_the_panel_fails_only_the_panel_sample() {
    let icon = tile().paint(WHITE, in_brackets).icon();
    assert_eq!(
        pixel(&icon, 128, 128),
        Some([0, 103, 192, 255]),
        "AC-4 control: with no panel the centre reads the bare tile"
    );
    assert_eq!(
        ids(&pixel_failures(&icon)),
        names("panel", &[PANEL_SAMPLE]),
        "AC-4 control: an icon with the panel left out must fail the panel sample, and only it"
    );
}

#[test]
fn the_panel_tolerance_is_two_per_channel_and_no_more() {
    let at = |rgb: [u8; 3], alpha: u8| {
        let mut canvas = tile().paint(rgb, in_panel);
        let i = ((128 * SIZE + 128) * 4) as usize;
        canvas.0[i + 3] = alpha;
        panel_failures(&canvas.icon()).len()
    };
    assert_eq!(
        at([91, 154, 216], 255),
        0,
        "2 off on every channel is inside"
    );
    assert_eq!(at([92, 156, 214], 255), 1, "3 off on red is outside");
    assert_eq!(at([89, 153, 214], 255), 1, "3 off on green is outside");
    assert_eq!(at([89, 156, 217], 255), 1, "3 off on blue is outside");
    assert_eq!(at(PANEL, 254), 1, "a panel that is not opaque is outside");
}
