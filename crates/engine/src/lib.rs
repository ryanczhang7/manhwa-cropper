//! `cropper-engine`: everything that reads or writes a file, and nothing that
//! draws (`docs/wiki/architecture.md`, "cropper-engine").
//!
//! MC-001 walking skeleton: one function that reaches into `cropper-core`,
//! which proves the crate graph links and gives the runner and the coverage
//! gate a line to measure. MC-002 adds [`args`] (the command line) and
//! [`copy`] (the headless walking-skeleton path that copies bytes unchanged).
//! MC-008 adds the first real file path through the detector: [`codec`], which
//! turns decoded pixels into the luma plane `cropper-core` works on, and
//! [`process`], which is one file in and one file out.
//!
//! [`process_file`] and the types it answers with are re-exported at the root,
//! because they are what a caller outside this crate wants - the GUI and the
//! batch runner ask the engine to process a file, not to reach into a module -
//! while the modules stay public for the tests that pin each of them
//! individually. `cropper-core` does the same with `pub use decide::{..}`.

#![forbid(unsafe_code)]

pub mod args;
pub mod codec;
pub mod copy;
pub mod process;

pub use codec::SourceFormat;
pub use process::{FileResult, Flag, Outcome, process_file};

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
