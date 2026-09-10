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
//! Note for whoever extends it: this module is compiled into a test target
//! that is linted at `-D warnings`, so an unused helper is a build failure.
//! Add a field or a function in the story that first uses it.

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
