//! `cropper-engine`: everything that reads or writes a file, and nothing that
//! draws (`docs/wiki/architecture.md`, "cropper-engine").
//!
//! MC-001 walking skeleton: one function that reaches into `cropper-core`,
//! which proves the crate graph links and gives the runner and the coverage
//! gate a line to measure. MC-002 adds [`args`] (the command line) and
//! [`copy`] (the headless walking-skeleton path that copies bytes unchanged).
//! MC-008 adds the first real file path through the detector: [`codec`], which
//! turns decoded pixels into the luma plane `cropper-core` works on, and
//! [`process`], which is one file in and one file out. MC-010 adds
//! [`naming`], which chooses every output name before anything is written,
//! and MC-011 [`batch`], which is a whole run: many files in, one
//! [`RunSummary`](batch::RunSummary) out. MC-013 adds [`settings`], the one
//! piece of state that outlives a run: the chosen output folder, and MC-012
//! [`outdir`], the rule that ranks a `--out` flag, that remembered folder and
//! the `cropped` fallback against each other.
//!
//! [`process_file`] and the types it answers with are re-exported at the root,
//! because they are what a caller outside this crate wants - the GUI and the
//! batch runner ask the engine to process a file, not to reach into a module -
//! while the modules stay public for the tests that pin each of them
//! individually. `cropper-core` does the same with `pub use decide::{..}`.

#![forbid(unsafe_code)]

pub mod args;
pub mod batch;
pub mod codec;
pub mod copy;
pub mod naming;
pub mod outdir;
pub mod process;
pub mod settings;

pub use codec::SourceFormat;
pub use outdir::resolve_out_dir;
pub use process::{FileResult, Flag, Outcome, process_file};

use cropper_core::Dimensions;

/// The detector's knobs, re-exported so that a caller outside this crate can
/// name the argument [`batch::run`] and [`process_file`] take without
/// depending on `cropper-core` itself: the exe asks the engine to run a
/// batch, and everything that run needs is the engine's to hand it.
pub use cropper_core::Tuning;

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
