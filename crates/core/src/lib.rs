//! `cropper-core`: pixels in, crop decision out.
//!
//! This crate never touches a file or a widget. Everything here is a pure
//! function over pixel data so the crop algorithm is testable without a
//! window (`docs/wiki/architecture.md`, "cropper-core").
//!
//! MC-001 leaves it as a walking skeleton: one type with one method, so the
//! test runner and the coverage gate have something real to see. MC-003 adds
//! the data model ([`Luma`], [`Rect`], [`Tuning`]) and the first pipeline
//! stage, [`trim`]; MC-004 adds the second, [`edges`]; MC-005 the third,
//! [`content`]; MC-006 the fourth, [`margin`], and [`decide`], which is where
//! they are finally composed into [`detect`]; MC-007 puts [`decide()`] on top
//! of that, which turns a [`Detection`] into the one answer the engine acts
//! on; the rest arrives story by story.
//!
//! [`decide()`], [`CropDecision`], [`FlagReason`], [`detect`] and
//! [`Detection`] are re-exported at the root because they are what a caller
//! outside the crate wants - the engine asks this crate one question - while
//! the stages stay behind their module names, where the tests that pin each of
//! them individually reach for them. The module [`decide`] and the function
//! [`decide()`] share a name without clashing: one lives in the type namespace
//! and the other in the value namespace.

#![forbid(unsafe_code)]

pub mod content;
pub mod decide;
pub mod edges;
pub mod margin;
pub mod trim;

pub use decide::{CropDecision, Detection, FlagReason, decide, detect};

/// Width and height of a pixel plane, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimensions {
    /// Number of columns.
    pub width: u32,
    /// Number of rows.
    pub height: u32,
}

impl Dimensions {
    /// Number of pixels in the plane. Widened to `u64` so a 4K screenshot
    /// (or anything larger) cannot overflow.
    #[must_use]
    pub fn area(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

/// An 8-bit greyscale plane: what every detector in this crate works on.
///
/// `data` is row-major and holds exactly `width * height` bytes. Colour
/// images are converted to luma before they get here (MC-008), so nothing
/// downstream of this type knows about channels.
pub struct Luma {
    /// Number of columns.
    pub width: u32,
    /// Number of rows.
    pub height: u32,
    /// `width * height` luma samples, row by row from the top left.
    pub data: Vec<u8>,
}

/// A rectangle in pixel coordinates, measured from the top left.
///
/// `Serialize` because [`CropDecision::Crop`] carries one and MC-011's run
/// summary serialises that (MC-007). The plain derive, with no `#[serde(..)]`
/// attribute: the field names are the JSON keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Rect {
    /// Left edge, in pixels from the left of the image.
    pub x: u32,
    /// Top edge, in pixels from the top of the image.
    pub y: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

/// The constants the detector is tuned by: `docs/wiki/architecture.md`'s
/// `Tuning` table, whole. Compiled in - v1 has no settings pane - and settled
/// for every story except the corpus story MC-019, which may change a default
/// with the corpus as its evidence.
///
/// Only [`uniform_tolerance`](Tuning::uniform_tolerance) is read yet; the
/// stages that read the rest land in MC-004..MC-007. The table is taken whole
/// rather than field by field because a `Tuning` with one field makes
/// `Tuning { uniform_tolerance: 20, ..Default::default() }` - the way a
/// caller overrides a single constant - a
/// `clippy::needless_update` error under the `lint` gate.
///
/// Deliberately not `#[non_exhaustive]`: that would forbid the struct literal
/// above outside this crate entirely.
pub struct Tuning {
    /// How far a row or column segment may span, darkest to lightest, and
    /// still count as uniform: it is uniform iff
    /// `max - min <= uniform_tolerance`. Read by [`trim::trim_uniform`].
    pub uniform_tolerance: u8,
    /// Mean absolute luma difference at or above which a row or column is a
    /// **strong line** (MC-004).
    pub edge_threshold: u8,
    /// Luma standard deviation the region left after removing a chrome strip
    /// must still have (MC-005).
    pub min_content_stddev: f32,
    /// Share of a strip's pixels that must lie within `uniform_tolerance` of
    /// the strip's median luma for it to be chrome-like (MC-005).
    pub chrome_flat_fraction: f32,
    /// Largest a chrome strip may be, as a share of the image's height
    /// (horizontal strips) or width (vertical strips) (MC-005).
    pub chrome_max_extent: f32,
    /// How far below `chrome_flat_fraction` a strip's flat fraction may fall
    /// and still be called ambiguous rather than content (MC-005).
    pub ambiguity_band: f32,
    /// Smallest content box, as a share of the image area, that is not
    /// flagged `LowContent` (MC-007).
    pub min_content_fraction: f32,
    /// Smallest content box side, in pixels, that is not flagged
    /// `LowContent` (MC-007). Fixed, not tuned by the corpus.
    pub min_content_side: u32,
    /// How far the final rect is expanded on every side, in pixels, before it
    /// is clamped to the image (MC-006).
    pub margin_px: u32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            uniform_tolerance: 10,
            edge_threshold: 24,
            min_content_stddev: 12.0,
            chrome_flat_fraction: 0.85,
            chrome_max_extent: 0.30,
            ambiguity_band: 0.05,
            min_content_fraction: 0.20,
            min_content_side: 64,
            margin_px: 3,
        }
    }
}
