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

use cropper_core::content::Side;
use cropper_core::{Luma, Rect, Tuning};
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

// --- Chrome bands inside the borders (MC-006) -------------------------------

/// How far a chrome band's "text" pixels sit from its background: twice
/// `Tuning::uniform_tolerance` (10), so a deviated pixel is unambiguously not
/// flat, and small enough that no run of them ever reaches `edge_threshold`
/// (24) - the same choice, and the same number, `tests/content.rs` made for
/// [`chrome_band`].
pub const CHROME_DEVIATION: u8 = 20;

/// How far a chrome band's rendered flat fraction may sit from
/// [`Chrome::flat_fraction`] before [`Recipe::render`] refuses the fixture.
/// Matches [`CHROME_BAND_FLAT_TOLERANCE`], for the same reason.
pub const CHROME_FLAT_TOLERANCE: f64 = 0.01;

/// The largest share of the image dimension one band may span, from MC-006
/// AC-4 ("heights up to 25% of the dimension"). Asserted on every render, so
/// a recipe that would quietly hand `content_box` a strip past
/// `chrome_max_extent` (0.30) fails in the generator instead.
pub const CHROME_MAX_SHARE: f64 = 0.25;

/// One band of browser or reader chrome, sitting inside a [`Border`] and
/// outside the art.
///
/// The band's **long axis** is the one parallel to the edge it sits on - rows
/// for a top or bottom band, columns for a left or right one - and that axis
/// is `art_w` or `art_h` at least, so at least 100 px under MC-006 AC-4. Along
/// it, each line carries exactly `k = round((1 - flat_fraction) * long)`
/// "text" pixels, chosen by a Bresenham walk over a coordinate rotated by one
/// step per line; their value is `background + deviation` or
/// `background - deviation`, the sign alternating along the line so the band's
/// median stays exactly on its background. Three properties every MC-006 test
/// leans on, and all three are why the pattern is this and not a scatter:
///
/// * the **flat fraction** is exactly `1 - k / long` on every line and so on
///   the band as a whole, whatever the band's other dimension, and sits in
///   `[asked, asked + 1 / long]` - at most 0.01 above, never below. A band's
///   chrome-likeness is chosen rather than measured. (The first attempt here
///   deviated pixels where `(x + y) % period < run`, which is exact only when
///   the band's extent is a multiple of the period: a 126x20 band at a
///   requested 0.865 rendered 0.875, and the generator's own assertion caught
///   it. The numbers are in the story's Handoff.)
/// * **every line along the long axis holds `k >= 2` "text" pixels**, and that
///   axis is the one facing the trim, so a band's outermost row or column is
///   never uniform and `trim_uniform` cannot nibble into it. A row-major
///   scatter does not have this property: at a flat fraction of 0.98 in a band
///   4 px wide, only every other column is ever touched;
/// * the band holds **no strong line of its own**. Two adjacent lines differ
///   in about `2k` places out of `long`, by `deviation` or `2 * deviation`, so
///   their mean absolute difference is at most
///   `2 * 2 * deviation * (1 - flat_fraction)` - 11.2 at AC-4's flattest -
///   against an `edge_threshold` of 24.
#[derive(Debug, Clone, Copy)]
pub struct Chrome {
    /// Depth in pixels, measured inward from the border that encloses it.
    pub thickness: u32,
    /// The flat background the band's median sits on.
    pub background: u8,
    /// How far the "text" pixels sit from the background, either way.
    pub deviation: u8,
    /// The share of the band's pixels left on the background, which is its
    /// flat fraction whenever `deviation > uniform_tolerance`.
    pub flat_fraction: f64,
}

impl Chrome {
    /// A band at [`CHROME_DEVIATION`] with the given flat fraction.
    ///
    /// # Panics
    ///
    /// If the band is degenerate (`thickness == 0`), if `flat_fraction` is not
    /// a share, or if `background +/- CHROME_DEVIATION` leaves `u8`.
    pub fn new(thickness: u32, background: u8, flat_fraction: f64) -> Self {
        assert!(thickness > 0, "a chrome band needs at least one line");
        assert!(
            (0.0..=1.0).contains(&flat_fraction),
            "flat fraction {flat_fraction} is not a share"
        );
        assert!(
            background.checked_add(CHROME_DEVIATION).is_some()
                && background.checked_sub(CHROME_DEVIATION).is_some(),
            "background {background} +/- {CHROME_DEVIATION} leaves u8; pick a background \
             with headroom"
        );
        Self {
            thickness,
            background,
            deviation: CHROME_DEVIATION,
            flat_fraction,
        }
    }

    /// How many "text" pixels each line along the long axis carries.
    ///
    /// Rounded **down**, so the band always renders at least as flat as it was
    /// asked for and the quantisation error can only ever push it away from
    /// `chrome_flat_fraction`, never towards it. Rounding to nearest instead
    /// put a band asked for 0.86 on a 140 px axis at 0.857, which is still
    /// chrome but is 0.007 from the threshold rather than 0.01 - a margin no
    /// fixture should be relying on. The cost is the other end: a band asked
    /// for 0.98 renders at up to `0.98 + 1 / long`.
    fn text_per_line(&self, long: u32) -> u32 {
        // The nudge is not slop: `1.0 - 0.9` is 0.09999999999999998, so the
        // floor of `(1.0 - 0.9) * 100` is 9 and a band asked for 0.9 renders
        // at 0.91. One part in a billion is far below a whole pixel at any
        // size this generator draws, and far above the representation error.
        ((1.0 - self.flat_fraction) * f64::from(long) + 1e-9).floor() as u32
    }

    /// The largest flat fraction this band can render at on an axis of `long`
    /// pixels: `text_per_line` rounds down, so the band is at most one "text"
    /// pixel per line flatter than it was asked for.
    fn flat_ceiling(&self, long: u32) -> f64 {
        self.flat_fraction + 1.0 / f64::from(long)
    }

    /// The band's pixel at band-local `(x, y)`, given the band's own size and
    /// which edge it sits on.
    fn tone(&self, side: Side, w: u32, h: u32, x: u32, y: u32) -> u8 {
        let horizontal = matches!(side, Side::Top | Side::Bottom);
        let (long, along, across) = if horizontal { (w, x, y) } else { (h, y, x) };
        let k = self.text_per_line(long);
        if k == 0 {
            return self.background;
        }
        // One step of rotation per line, so the "text" walks diagonally and
        // every line across the short axis is reached in turn.
        let t = (along + across) % long;
        let before = (t * k) / long;
        if before == ((t + 1) * k) / long {
            self.background
        } else if before.is_multiple_of(2) {
            self.background + self.deviation
        } else {
            self.background - self.deviation
        }
    }
}

/// The total depth of a stack of bands.
fn stack_depth(bands: &[Chrome]) -> u32 {
    bands.iter().map(|band| band.thickness).sum()
}

/// A whole fixture image: a textured art rect, optionally wrapped in chrome
/// bands, wrapped in four uniform borders.
///
/// The image is `left + left_chrome + art_w + right_chrome + right` by
/// `top + top_chrome + art_h + bottom_chrome + bottom`, and
/// [`Recipe::art_rect`] is the answer every trim and detect test compares
/// against.
///
/// The nesting - borders **outside**, chrome **inside**, art innermost - is
/// MC-006's choice and is the one the pipeline reads cleanly: the first trim
/// takes the whole border frame off, `content_box` then works on a rect that
/// is exactly chrome plus art, and every chrome/art boundary is a strong line
/// because the art is a checkerboard. The reverse nesting (chrome outside a
/// uniform page gutter) is MC-006 AC-2's scene and is built by hand there;
/// see the story's Handoff for why it is not in this generator.
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
    /// Chrome bands below the top border, outermost first.
    pub top_chrome: Vec<Chrome>,
    /// Chrome bands above the bottom border, outermost first.
    pub bottom_chrome: Vec<Chrome>,
    /// Chrome bands right of the left border, outermost first.
    pub left_chrome: Vec<Chrome>,
    /// Chrome bands left of the right border, outermost first.
    pub right_chrome: Vec<Chrome>,
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
            top_chrome: Vec::new(),
            bottom_chrome: Vec::new(),
            left_chrome: Vec::new(),
            right_chrome: Vec::new(),
        }
    }

    /// Rendered image width.
    pub fn width(&self) -> u32 {
        self.left.thickness + self.inner_w() + self.right.thickness
    }

    /// Rendered image height.
    pub fn height(&self) -> u32 {
        self.top.thickness + self.inner_h() + self.bottom.thickness
    }

    /// Width of the region inside the borders: chrome plus art.
    fn inner_w(&self) -> u32 {
        stack_depth(&self.left_chrome) + self.art_w + stack_depth(&self.right_chrome)
    }

    /// Height of the region inside the borders: chrome plus art.
    fn inner_h(&self) -> u32 {
        stack_depth(&self.top_chrome) + self.art_h + stack_depth(&self.bottom_chrome)
    }

    /// The rect a correct detect must return, less the outward margin, and the
    /// rect a correct trim returns for a recipe with no chrome.
    pub fn art_rect(&self) -> Rect {
        Rect {
            x: self.left.thickness + stack_depth(&self.left_chrome),
            y: self.top.thickness + stack_depth(&self.top_chrome),
            w: self.art_w,
            h: self.art_h,
        }
    }

    /// Every chrome band's rect in image coordinates, outermost first, side by
    /// side: top, bottom, left, right. Used by the tests that measure what the
    /// generator rendered.
    pub fn chrome_rects(&self) -> Vec<(Side, Chrome, Rect)> {
        let x0 = self.left.thickness;
        let y0 = self.top.thickness;
        let (iw, ih) = (self.inner_w(), self.inner_h());
        let mut out = Vec::new();

        let mut offset = 0;
        for &band in &self.top_chrome {
            out.push((
                Side::Top,
                band,
                Rect {
                    x: x0,
                    y: y0 + offset,
                    w: iw,
                    h: band.thickness,
                },
            ));
            offset += band.thickness;
        }
        let mut offset = 0;
        for &band in &self.bottom_chrome {
            offset += band.thickness;
            out.push((
                Side::Bottom,
                band,
                Rect {
                    x: x0,
                    y: y0 + ih - offset,
                    w: iw,
                    h: band.thickness,
                },
            ));
        }
        // The vertical bands fill only the rows between the horizontal ones,
        // so a chrome frame's corners belong to the top and bottom bands - the
        // shape a browser actually has, and the shape that keeps the image's
        // outermost column spanning more than one background.
        let art_y = y0 + stack_depth(&self.top_chrome);
        let mut offset = 0;
        for &band in &self.left_chrome {
            out.push((
                Side::Left,
                band,
                Rect {
                    x: x0 + offset,
                    y: art_y,
                    w: band.thickness,
                    h: self.art_h,
                },
            ));
            offset += band.thickness;
        }
        let mut offset = 0;
        for &band in &self.right_chrome {
            offset += band.thickness;
            out.push((
                Side::Right,
                band,
                Rect {
                    x: x0 + iw - offset,
                    y: art_y,
                    w: band.thickness,
                    h: self.art_h,
                },
            ));
        }
        out
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
        self.paint_chrome(&mut data, w);
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

        let img = Luma {
            width: w,
            height: h,
            data,
        };
        self.assert_chrome_rendered(&img);
        img
    }

    /// Every band is what it was asked to be: a long axis of at least
    /// `1 / CHROME_FLAT_TOLERANCE` pixels, its median on its background, its
    /// flat fraction in `[asked, asked + 1 / long]`, and its extent within
    /// [`CHROME_MAX_SHARE`] of the image dimension on its axis.
    ///
    /// Checked on every render, in the house style of [`chrome_band`]: a
    /// fixture that drifts has quietly moved a threshold, and this is what a
    /// self-asserting generator is for.
    ///
    /// # Panics
    ///
    /// If any of the four misses.
    fn assert_chrome_rendered(&self, img: &Luma) {
        let tolerance = Tuning::default().uniform_tolerance;
        for (side, band, rect) in self.chrome_rects() {
            let (dimension, long) = match side {
                Side::Top | Side::Bottom => (img.height, rect.w),
                Side::Left | Side::Right => (img.width, rect.h),
            };
            let share = f64::from(band.thickness) / f64::from(dimension);
            assert!(
                share <= CHROME_MAX_SHARE,
                "chrome band {band:?} spans {share} of the image's {dimension}, past \
                 the {CHROME_MAX_SHARE} MC-006 AC-4 allows"
            );
            assert!(
                f64::from(long) >= 1.0 / CHROME_FLAT_TOLERANCE,
                "a chrome band's long axis must be at least {} px for its flat fraction \
                 to land within {CHROME_FLAT_TOLERANCE} of the one asked for; {side:?} \
                 has {long}",
                1.0 / CHROME_FLAT_TOLERANCE
            );

            let mut values = Vec::with_capacity(rect.w as usize * rect.h as usize);
            for dy in 0..rect.h as usize {
                let start = (rect.y as usize + dy) * img.width as usize + rect.x as usize;
                values.extend_from_slice(&img.data[start..start + rect.w as usize]);
            }
            values.sort_unstable();
            let median = values[values.len() / 2];
            assert_eq!(
                median, band.background,
                "the band's median moved off its background colour: {band:?}"
            );
            // At a deviation inside the tolerance every pixel counts as flat
            // however many were moved, which is the case `chrome_band` uses to
            // pin that "within `uniform_tolerance`" is inclusive.
            let (floor, ceiling) = if band.deviation > tolerance {
                (band.flat_fraction, band.flat_ceiling(long))
            } else {
                (1.0, 1.0)
            };
            let flat = values
                .iter()
                .filter(|&&px| px.abs_diff(median) <= tolerance)
                .count();
            let rendered = flat as f64 / values.len() as f64;
            assert!(
                (floor..=ceiling).contains(&rendered),
                "chrome band {band:?} at {}x{} should have flat fraction in \
                 {floor}..={ceiling} at tolerance {tolerance} and rendered {rendered}",
                rect.w,
                rect.h
            );
        }
    }

    fn paint_chrome(&self, data: &mut [u8], w: u32) {
        for (side, band, rect) in self.chrome_rects() {
            for dy in 0..rect.h {
                let start = (rect.y + dy) as usize * w as usize + rect.x as usize;
                for (dx, px) in data[start..start + rect.w as usize].iter_mut().enumerate() {
                    *px = band.tone(side, rect.w, rect.h, dx as u32, dy);
                }
            }
        }
    }

    fn paint_art(&self, data: &mut [u8], w: u32) {
        let mut rng = Rng::new(self.seed);
        let art = self.art_rect();
        let ax = art.x as usize;
        let ay = art.y as usize;
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
            ..Recipe::new(art_w, art_h)
        })
}

// --- proptest strategy: MC-006 AC-4's recipes -------------------------------

/// One chrome band as the strategy draws it, before the art size is known:
/// thickness as a share of the art extent in permille, and the flat fraction
/// the band is to render at.
type ChromeSpec = (u32, f64);

/// Thickness of a chrome band, in permille of the art extent on its axis.
///
/// Expressed against the art rather than the image because the image size
/// depends on it, and AC-4's "heights up to 25% of the dimension" has to hold
/// by construction. It does: a single band of 330 permille on an otherwise
/// bare edge is `0.33 * art / 1.33 * art` = 0.248 of the dimension, and every
/// other arrangement - a second band, a border, chrome on the opposite
/// edge - makes the image larger and the share smaller.
const CHROME_PERMILLE: std::ops::RangeInclusive<u32> = 50..=330;

/// AC-4's flat fractions, whole. Every one of them is above
/// `chrome_flat_fraction` (0.85), which is what makes every band in a
/// generated recipe chrome rather than a coin toss on the threshold - MC-005
/// owns the threshold itself, and pinning it again here would be testing
/// `content_box` through `detect`.
const CHROME_FLAT: std::ops::RangeInclusive<f64> = 0.86..=0.98;

/// Smallest chrome band the generator will draw, so a 50-permille band on a
/// 100 px art is a band rather than a line.
const CHROME_MIN_THICKNESS: u32 = 8;

/// How far apart two stacked bands' backgrounds are put. Comfortably past
/// `edge_threshold` (24) even after the dilution of a pass that has not yet
/// peeled the horizontal chrome, so the boundary between two stacked bands is
/// always a strong line and the outer one is always the outermost strip.
const CHROME_BACKGROUND_STEP: u8 = 90;

/// Materialise one edge's chrome stack once the art extent is known.
///
/// Backgrounds alternate `base`, `base + 90`, so *adjacent* bands are always
/// far apart; `base` is drawn from the seed rather than from the strategy so
/// that the tuple the strategy builds stays inside proptest's arity.
fn chrome_stack(specs: &[ChromeSpec], art: u32, seed: u32, salt: u32) -> Vec<Chrome> {
    // `base + step` must still leave `CHROME_DEVIATION` of headroom under 255.
    let base = Rng::new(seed ^ salt).between(
        CHROME_DEVIATION,
        255 - CHROME_DEVIATION - CHROME_BACKGROUND_STEP,
    );
    specs
        .iter()
        .enumerate()
        .map(|(i, &(permille, flat))| {
            let thickness = (art * permille / 1000).max(CHROME_MIN_THICKNESS);
            let background = if i.is_multiple_of(2) {
                base
            } else {
                base + CHROME_BACKGROUND_STEP
            };
            Chrome::new(thickness, background, flat)
        })
        .collect()
}

/// MC-006 AC-4's generator, whole: per-side border widths `0..=200` and
/// colours `0..=255`, either layout, zero to two chrome bands per edge with
/// flat fraction in `0.86..=0.98` and thickness up to a quarter of the
/// dimension, art `100..=1200` per side, seeded texture.
///
/// The art is the checkerboard of [`Recipe`], not [`soft_art`], and that is
/// the load-bearing choice. The checkerboard is a strong line at *every* row
/// and column (165.7 mean absolute difference, measured in MC-004), so on
/// every pass of the chrome peel there is a candidate strip one line deep on
/// each of the four sides, and the only thing keeping the art whole is that
/// the strip fails the flatness test. Under [`soft_art`] the art carries no
/// strong line at all, so the peel stops for want of a candidate and the
/// property would be testing the fixture rather than `detect`.
pub fn any_chrome_recipe() -> impl Strategy<Value = Recipe> {
    (
        (
            100u32..=1200u32,
            100u32..=1200u32,
            proptest::array::uniform4(0u32..=200u32),
            proptest::array::uniform4(0u8..=255u8),
            proptest::bool::ANY,
            proptest::num::u32::ANY,
        ),
        (
            proptest::collection::vec((CHROME_PERMILLE, CHROME_FLAT), 0..=2),
            proptest::collection::vec((CHROME_PERMILLE, CHROME_FLAT), 0..=2),
            proptest::collection::vec((CHROME_PERMILLE, CHROME_FLAT), 0..=2),
            proptest::collection::vec((CHROME_PERMILLE, CHROME_FLAT), 0..=2),
        ),
    )
        .prop_map(
            |(
                (art_w, art_h, thickness, colour, gutters, seed),
                (top_spec, bottom_spec, left_spec, right_spec),
            )| Recipe {
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
                top_chrome: chrome_stack(&top_spec, art_h, seed, 0x9E37_79B9),
                bottom_chrome: chrome_stack(&bottom_spec, art_h, seed, 0x85EB_CA6B),
                left_chrome: chrome_stack(&left_spec, art_w, seed, 0xC2B2_AE35),
                right_chrome: chrome_stack(&right_spec, art_w, seed, 0x27D4_EB2F),
            },
        )
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
