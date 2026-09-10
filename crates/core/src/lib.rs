//! `cropper-core`: pixels in, crop decision out.
//!
//! This crate never touches a file or a widget. Everything here is a pure
//! function over pixel data so the crop algorithm is testable without a
//! window (`docs/wiki/architecture.md`, "cropper-core").
//!
//! MC-001 leaves it as a walking skeleton: one type with one method, so the
//! test runner and the coverage gate have something real to see. The
//! algorithm arrives story by story from MC-003.

#![forbid(unsafe_code)]

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
