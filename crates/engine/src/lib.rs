//! `cropper-engine`: everything that reads or writes a file, and nothing that
//! draws (`docs/wiki/architecture.md`, "cropper-engine").
//!
//! MC-001 walking skeleton: one function that reaches into `cropper-core`,
//! which proves the crate graph links and gives the runner and the coverage
//! gate a line to measure. MC-002 adds [`args`] (the command line) and
//! [`copy`] (the headless walking-skeleton path that copies bytes unchanged).

#![forbid(unsafe_code)]

pub mod args;
pub mod copy;

use cropper_core::Dimensions;

/// Dimensions of the largest image the engine will accept, 8K in each
/// direction. A screenshot larger than this is not a screenshot.
pub const MAX_DIMENSIONS: Dimensions = Dimensions {
    width: 7680,
    height: 4320,
};

/// Whether an image of these dimensions is within what the engine handles.
#[must_use]
pub fn within_limits(dims: Dimensions) -> bool {
    dims.width <= MAX_DIMENSIONS.width && dims.height <= MAX_DIMENSIONS.height
}
