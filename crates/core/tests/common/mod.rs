//! Synthetic `Luma` fixtures, built from a recipe rather than stored
//! (`docs/wiki/architecture.md`, "Test support"). The generator knows the true
//! art rect, so every detector test in `crates/core/tests/` has an exact
//! oracle and never has to guess a number.
//!
//! Introduced by MC-003 for the uniform-border trim. MC-004..MC-007 extend
//! it - chrome bands, strong interior lines, margins - so it is written as a
//! generator with a `Recipe` and a seeded RNG, not as a bag of one-off images.
//! Everything here is deterministic: the same `Recipe` always renders the same
//! bytes, on any machine.
//!
//! Note for whoever extends it: **add a field or a function in the story that
//! first uses it.** MC-003 wrote that this was enforced mechanically - an
//! unused helper being a build failure under `-D warnings` - and while there
//! was exactly one test target that was true. MC-004 added a second
//! (`tests/edges.rs`), and Cargo compiles `tests/common/mod.rs` separately
//! into *each* test binary, so from that moment on every helper used by one
//! target is dead code in the other: `flat_band`, `any_recipe`,
//! `Recipe::art_rect` and `Layout::Gutters` are all unused in the `edges`
//! target, and `soft_art` is unused in the `trim` target. The lint stopped
//! being a guard and became noise the moment the second target existed, which
//! is why the `allow` below is here and why the rule above is now a rule for
//! people rather than for the compiler. It is not an invitation to leave dead
//! fixtures behind.
#![allow(dead_code)]

use cropper_core::{Luma, Rect};
use proptest::strategy::Strategy;

// --- Seeded randomness ------------------------------------------------------

/// A 32-bit xorshift, the same shape `crates/app/tests/cli.rs` uses for its
/// sample PNG. Hand-rolled because `cropper-core` has exactly one
/// dev-dependency (`proptest`) and a fixture generator is not a reason to add
/// another.
pub struct Rng(u32);

impl Rng {
    /// Seed 0 is a fixed point of xorshift and would emit nothing but zeros,
    /// so it is replaced with the constant `cli.rs` uses.
    pub fn new(seed: u32) -> Self {
        Self(if seed == 0 { 0x2545_F491 } else { seed })
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    /// A value in `lo ..= hi`. Requires `lo <= hi`.
    pub fn between(&mut self, lo: u8, hi: u8) -> u8 {
        let span = u32::from(hi - lo) + 1;
        lo + (self.next_u32() % span) as u8
    }
}

// --- Art texture ------------------------------------------------------------

/// The art alternates between a dark tone and a light tone on a checkerboard,
/// each jittered by the seeded RNG. Two consequences the tests depend on:
///
/// * any art row or column of at least two pixels contains one pixel from
///   `ART_LO ..= ART_LO + ART_JITTER` and one from
///   `ART_HI ..= ART_HI + ART_JITTER`, so its spread is at least
///   `ART_HI - (ART_LO + ART_JITTER)` = 140 - far above any tolerance in
///   `Tuning`. That is exactly AC-1's "textured" precondition, and under the
///   settled rule (`max - min <= uniform_tolerance`) it is precisely the
///   negation of "uniform", so no art row or column can ever be trimmed;
/// * the art is never one flat colour, so a fixture cannot accidentally
///   satisfy AC-3.
const ART_LO: u8 = 40;
const ART_HI: u8 = 200;
const ART_JITTER: u8 = 20;

// --- The recipe -------------------------------------------------------------

/// Which pair of sides spans the whole image.
///
/// A real screenshot has one or the other and the detector is not told which,
/// which is why the trim has to iterate (MC-003 AC-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Top and bottom bands span the full width; the left and right gutters
    /// occupy only the rows between them.
    Bands,
    /// Left and right gutters span the full height; the top and bottom bands
    /// occupy only the columns between them.
    Gutters,
}

/// One side's border: how far it reaches inward, and what colour it is.
#[derive(Debug, Clone, Copy)]
pub struct Border {
    /// Depth in pixels, measured inward from that edge. 0 means no border.
    pub thickness: u32,
    /// The side's colour. Every border pixel is either this or
    /// `colour + spread`; there is no other value.
    pub colour: u8,
    /// Deviation from `colour`, in one direction only, as MC-003 AC-2
    /// requires. The two tones alternate on a checkerboard, so *every* row and
    /// column segment of at least two border pixels contains both endpoints
    /// and has a spread of exactly `spread`. `spread == 0` is a perfectly
    /// solid border.
    pub spread: u8,
}

impl Border {
    /// A border of one flat colour.
    pub fn solid(thickness: u32, colour: u8) -> Self {
        Self {
            thickness,
            colour,
            spread: 0,
        }
    }

    fn tone(self, x: usize, y: usize) -> u8 {
        if (x + y).is_multiple_of(2) {
            self.colour
        } else {
            self.colour.checked_add(self.spread).unwrap_or_else(|| {
                panic!(
                    "border colour {} + spread {} overflows u8; pick a colour with headroom",
                    self.colour, self.spread
                )
            })
        }
    }
}

/// A whole fixture image: a textured art rect surrounded by four borders.
///
/// The image is `left + art_w + right` by `top + art_h + bottom`, and
/// [`Recipe::art_rect`] is the answer every trim test compares against.
#[derive(Debug, Clone)]
pub struct Recipe {
    /// Art width in pixels. At least 2, so every art row holds two pixels.
    pub art_w: u32,
    /// Art height in pixels. At least 2, so every art column holds two pixels.
    pub art_h: u32,
    /// Top border.
    pub top: Border,
    /// Right border.
    pub right: Border,
    /// Bottom border.
    pub bottom: Border,
    /// Left border.
    pub left: Border,
    /// Which pair of sides spans the whole image.
    pub layout: Layout,
    /// Seed for the art texture. The borders are deterministic without it.
    pub seed: u32,
}

impl Recipe {
    /// Art of the given size with no borders at all: the AC-4 fixture, and the
    /// base every other fixture is built from with struct-update syntax.
    pub fn new(art_w: u32, art_h: u32) -> Self {
        Self {
            art_w,
            art_h,
            top: Border::solid(0, 0),
            right: Border::solid(0, 0),
            bottom: Border::solid(0, 0),
            left: Border::solid(0, 0),
            layout: Layout::Bands,
            seed: 1,
        }
    }

    /// Rendered image width.
    pub fn width(&self) -> u32 {
        self.left.thickness + self.art_w + self.right.thickness
    }

    /// Rendered image height.
    pub fn height(&self) -> u32 {
        self.top.thickness + self.art_h + self.bottom.thickness
    }

    /// The rect a correct trim must return, exactly.
    pub fn art_rect(&self) -> Rect {
        Rect {
            x: self.left.thickness,
            y: self.top.thickness,
            w: self.art_w,
            h: self.art_h,
        }
    }

    /// Render the recipe to an 8-bit luma plane.
    pub fn render(&self) -> Luma {
        assert!(
            self.art_w >= 2 && self.art_h >= 2,
            "art must be at least 2x2 for every art row and column to be textured"
        );
        let w = self.width();
        let h = self.height();
        let mut data = vec![0u8; w as usize * h as usize];

        self.paint_art(&mut data, w);
        // The pair that spans the whole image is painted last, so it overwrites
        // the other pair's ends. That one line is the whole difference between
        // the two layouts.
        match self.layout {
            Layout::Bands => {
                self.paint_gutters(&mut data, w);
                self.paint_bands(&mut data, w);
            }
            Layout::Gutters => {
                self.paint_bands(&mut data, w);
                self.paint_gutters(&mut data, w);
            }
        }

        Luma {
            width: w,
            height: h,
            data,
        }
    }

    fn paint_art(&self, data: &mut [u8], w: u32) {
        let mut rng = Rng::new(self.seed);
        let ax = self.left.thickness as usize;
        let ay = self.top.thickness as usize;
        let aw = self.art_w as usize;
        for (y, row) in data
            .chunks_mut(w as usize)
            .enumerate()
            .skip(ay)
            .take(self.art_h as usize)
        {
            for (dx, px) in row[ax..ax + aw].iter_mut().enumerate() {
                *px = if (ax + dx + y).is_multiple_of(2) {
                    rng.between(ART_LO, ART_LO + ART_JITTER)
                } else {
                    rng.between(ART_HI, ART_HI + ART_JITTER)
                };
            }
        }
    }

    fn paint_bands(&self, data: &mut [u8], w: u32) {
        let height = data.len() / w as usize;
        let top = self.top.thickness as usize;
        let bottom = self.bottom.thickness as usize;
        for (y, row) in data.chunks_mut(w as usize).enumerate() {
            let band = if y < top {
                self.top
            } else if y >= height - bottom {
                self.bottom
            } else {
                continue;
            };
            for (x, px) in row.iter_mut().enumerate() {
                *px = band.tone(x, y);
            }
        }
    }

    fn paint_gutters(&self, data: &mut [u8], w: u32) {
        let width = w as usize;
        let left = self.left.thickness as usize;
        let start = width - self.right.thickness as usize;
        for (y, row) in data.chunks_mut(width).enumerate() {
            for (x, px) in row[..left].iter_mut().enumerate() {
                *px = self.left.tone(x, y);
            }
            for (dx, px) in row[start..].iter_mut().enumerate() {
                *px = self.right.tone(start + dx, y);
            }
        }
    }
}

// --- A flat image, for the "nothing to crop" cases --------------------------

/// A `Luma` whose every pixel is `low` or `low + spread`, alternating on a
/// checkerboard, so **every** row and column segment of at least two pixels
/// has a spread of exactly `spread`, and the whole image's `max - min` is
/// exactly `spread`.
///
/// That makes it the sharpest available probe of the settled rule
/// (`max - min <= uniform_tolerance`): at `spread == 10` nothing in the image
/// is ever non-uniform, and at `spread == 11` nothing in it is ever uniform.
pub fn flat_band(width: u32, height: u32, low: u8, spread: u8) -> Luma {
    let mut data = vec![0u8; width as usize * height as usize];
    for (y, row) in data.chunks_mut(width as usize).enumerate() {
        for (x, px) in row.iter_mut().enumerate() {
            *px = if (x + y).is_multiple_of(2) {
                low
            } else {
                low + spread
            };
        }
    }
    Luma {
        width,
        height,
        data,
    }
}

// --- proptest strategy ------------------------------------------------------

/// Every combination AC-1 talks about: art of 2x2 to 24x24, four solid borders
/// each `0 ..= max_border` px with independently chosen colours over the whole
/// 0..=255 range, and either layout.
///
/// Border thicknesses are drawn independently, so 0 on one, two, three or all
/// four sides all occur; `max_border = 200` is AC-1's stated upper bound.
pub fn any_recipe(max_border: u32) -> impl Strategy<Value = Recipe> {
    (
        2u32..=24u32,
        2u32..=24u32,
        proptest::array::uniform4(0u32..=max_border),
        proptest::array::uniform4(0u8..=255u8),
        proptest::bool::ANY,
        proptest::num::u32::ANY,
    )
        .prop_map(|(art_w, art_h, thickness, colour, gutters, seed)| Recipe {
            art_w,
            art_h,
            top: Border::solid(thickness[0], colour[0]),
            right: Border::solid(thickness[1], colour[1]),
            bottom: Border::solid(thickness[2], colour[2]),
            left: Border::solid(thickness[3], colour[3]),
            layout: if gutters {
                Layout::Gutters
            } else {
                Layout::Bands
            },
            seed,
        })
}

// --- A second art texture: soft, for the edge tests (MC-004) ----------------

/// The checkerboard art above is *maximally* textured on purpose: adjacent
/// rows are opposite phases of the board, so the mean absolute difference
/// between one art row and the next measures **165.7** (200 seeds, measured in
/// MC-004's RED). Under `Tuning::edge_threshold = 24` that art is a strong
/// line at every single row and column, which is exactly what MC-003 needed
/// (nothing in it is ever uniform) and exactly the wrong fixture for MC-004
/// AC-5, which needs art whose *own* texture is not an edge.
///
/// [`soft_art`] is that second texture. It is a triangular ramp in both axes
/// plus seeded jitter: no discontinuity anywhere, so every adjacent-row and
/// adjacent-column mean absolute difference stays well under
/// `edge_threshold`, while the field as a whole still spans far more than
/// `uniform_tolerance` and so is genuinely textured rather than flat.
///
/// Measured over 200 seeds at four sizes (7x5, 24x18, 40x30, 64x48): the
/// largest adjacent-row mean absolute difference is **13.14** and the largest
/// adjacent-column one **13.20**, against a threshold of 24. Whole-field
/// spread ranges 49..96.
const SOFT_BASE: u8 = 48;
const SOFT_PERIOD_X: usize = 8;
const SOFT_STEP_X: u32 = 6;
const SOFT_PERIOD_Y: usize = 6;
const SOFT_STEP_Y: u32 = 6;
const SOFT_JITTER: u8 = 12;

/// A triangular wave: `0, 1, .. period, period - 1, .. 0, 1, ..`. Consecutive
/// values always differ by exactly 1, so a ramp driven by it has no
/// discontinuity at any width or height - which is the whole point, since a
/// wrapping sawtooth would put a strong line at every wrap.
fn triangle(t: usize, period: usize) -> u32 {
    let p = t % (2 * period);
    (if p <= period { p } else { 2 * period - p }) as u32
}

/// Art whose own texture is not an edge: see the constants above. Deterministic
/// in `seed`, and bounded well inside `u8` for the fixture sizes this suite
/// uses (the ramps contribute at most `6 * 8 + 6 * 6 = 84` over a base of 48).
///
/// Used by `tests/edges.rs` (MC-004 AC-5); unused by `tests/trim.rs`, which is
/// what the module-level `allow(dead_code)` at the top of this file is for.
pub fn soft_art(width: u32, height: u32, seed: u32) -> Luma {
    let mut rng = Rng::new(seed);
    let mut data = vec![0u8; width as usize * height as usize];
    for (y, row) in data.chunks_mut(width as usize).enumerate() {
        for (x, px) in row.iter_mut().enumerate() {
            let ramp = u32::from(SOFT_BASE)
                + triangle(x, SOFT_PERIOD_X) * SOFT_STEP_X
                + triangle(y, SOFT_PERIOD_Y) * SOFT_STEP_Y;
            *px = (ramp.min(255) as u8).saturating_add(rng.between(0, SOFT_JITTER));
        }
    }
    Luma {
        width,
        height,
        data,
    }
}

// --- A chrome band texture, at a requested flat fraction (MC-005) -----------

/// How far a rendered flat fraction may sit from the requested one before
/// [`chrome_band`] refuses to hand the fixture over. MC-005's Model guidance
/// fixes this at 0.01; a fixture that drifts past it has quietly moved a
/// threshold, which is exactly what a self-asserting generator is for.
pub const CHROME_BAND_FLAT_TOLERANCE: f64 = 0.01;

/// A browser-chrome band: a flat background with sparse "text" pixels on it,
/// rendered so that its **flat fraction** - the share of its pixels within
/// `tolerance` of the band's median luma, which is the quantity MC-005's
/// `chrome_flat_fraction` is measured against - is `flat_fraction` to within
/// [`CHROME_BAND_FLAT_TOLERANCE`].
///
/// The band is `background` everywhere except for exactly
/// `round((1 - flat_fraction) * width * height)` pixels, which alternate
/// between `background + deviation` and `background - deviation`. Two
/// properties the tests lean on, and both are why the deviations are
/// symmetric and evenly spread rather than random or clustered:
///
/// * the **median is exactly `background`** for every `flat_fraction` down to
///   0.0, because the deviated pixels are split evenly above and below it. A
///   one-sided deviation would leave the median undefined-by-convention at a
///   flat fraction of 0.5, which is one of MC-005 AC-3's controls;
/// * the band carries **no strong line of its own**. The deviated pixels are
///   spread evenly over the band in row-major order (a Bresenham selection, so
///   the count is exact), so no row, column or edge between them ever reaches
///   `edge_threshold`. A band that contained a strong line would be split by
///   `edges::strong_lines` and would no longer be the strip the test built.
///
/// `flat_fraction` is always the share of pixels left **on the background**,
/// and that is asserted directly. It is also the band's flat fraction whenever
/// `deviation > tolerance`, which is the ordinary case and the one the
/// assertion checks. At `deviation <= tolerance` every pixel is within
/// tolerance of the median, so the band's flat fraction is 1.0 however many
/// pixels were moved - a fixture MC-005 uses deliberately to pin that "within
/// `uniform_tolerance`" is inclusive - and the assertion checks that instead.
/// Either way, what was rendered is checked against what it should be.
///
/// # Panics
///
/// If `background +/- deviation` leaves `u8`, if the band is empty, if
/// `flat_fraction` is outside `0.0..=1.0`, or if the rendered band misses
/// either the requested share on the background or the flat fraction that
/// implies by more than [`CHROME_BAND_FLAT_TOLERANCE`].
pub fn chrome_band(
    width: u32,
    height: u32,
    background: u8,
    deviation: u8,
    tolerance: u8,
    flat_fraction: f64,
) -> Luma {
    let n = width as usize * height as usize;
    assert!(n > 0, "a chrome band needs at least one pixel");
    assert!(
        (0.0..=1.0).contains(&flat_fraction),
        "flat fraction {flat_fraction} is not a share"
    );
    assert!(
        background.checked_add(deviation).is_some() && background.checked_sub(deviation).is_some(),
        "background {background} +/- deviation {deviation} leaves u8; pick a background with headroom"
    );

    // Exactly `deviated` pixels are moved off the background, and the
    // Bresenham test below picks them evenly spread through the band: the
    // number of selections among the first `i` indices is `floor(i * k / n)`,
    // so the total is exactly `k` and the gaps differ by at most one.
    let deviated = ((1.0 - flat_fraction) * n as f64).round() as usize;
    let mut data = vec![background; n];
    let mut above = true;
    for (i, px) in data.iter_mut().enumerate() {
        if (i * deviated) / n != ((i + 1) * deviated) / n {
            *px = if above {
                background + deviation
            } else {
                background - deviation
            };
            above = !above;
        }
    }

    let rendered_median = median(&data);
    assert_eq!(
        rendered_median, background,
        "the band's median moved off its background colour"
    );
    let on_background = flat_share(&data, rendered_median, 0);
    assert!(
        (on_background - flat_fraction).abs() <= CHROME_BAND_FLAT_TOLERANCE,
        "chrome band {width}x{height} asked for {flat_fraction} of its pixels on the \
         background and rendered {on_background}; tolerance is {CHROME_BAND_FLAT_TOLERANCE}"
    );
    // Every deviated pixel is `deviation` away from the median, so the flat
    // fraction is the share on the background - unless the deviation is
    // inside the tolerance, in which case *every* pixel is flat and the band
    // is the fixture that pins "within `uniform_tolerance`" as inclusive.
    let expected = if deviation > tolerance {
        flat_fraction
    } else {
        1.0
    };
    let rendered = flat_share(&data, rendered_median, tolerance);
    assert!(
        (rendered - expected).abs() <= CHROME_BAND_FLAT_TOLERANCE,
        "chrome band {width}x{height} (background {background}, deviation {deviation}) \
         should have flat fraction {expected} at tolerance {tolerance} and rendered \
         {rendered}; tolerance is {CHROME_BAND_FLAT_TOLERANCE}"
    );

    Luma {
        width,
        height,
        data,
    }
}

/// The upper median of `data`. Used only to check what [`chrome_band`]
/// rendered; the fixtures are built so the two median conventions agree.
fn median(data: &[u8]) -> u8 {
    let mut sorted = data.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

/// The share of `data` within `tolerance` of `centre`, inclusive.
fn flat_share(data: &[u8], centre: u8, tolerance: u8) -> f64 {
    let flat = data
        .iter()
        .filter(|&&px| px.abs_diff(centre) <= tolerance)
        .count();
    flat as f64 / data.len() as f64
}
