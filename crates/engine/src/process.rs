//! One file in, one file out (MC-008): the single-file entry the batch runner
//! parallelises (MC-011).
//!
//! [`process_file`] is the whole of the engine's per-file work, in four steps
//! and no branches of its own beyond the detector's answer:
//!
//! 1. decode the path with `image`;
//! 2. convert to the luma plane with [`codec::to_luma`](crate::codec::to_luma)
//!    - BT.601, alpha ignored;
//! 3. ask [`cropper_core::decide`], which is the only place a crop is judged;
//! 4. act on that answer: crop the *decoded pixels* to the rect and encode a
//!    PNG, or - when the detector flags the image - copy the input's bytes
//!    unchanged with [`copy::copy_unchanged`](crate::copy::copy_unchanged).
//!
//! Either way the output lands at `out_dir/<the input's own file name>`, case
//! and extension untouched (AC-4). Name collisions are MC-010's.
//!
//! # Why a flagged file is copied rather than re-encoded
//!
//! A flagged image is one the detector would not commit to, so the file the
//! user gets back has to be the file they gave us - not a re-encoding of it
//! that happens to look the same. Re-encoding drops every metadata chunk the
//! source carried and re-compresses pixels nobody asked us to touch, and for a
//! flat image it is nearly indistinguishable from a copy, which is exactly why
//! MC-008 AC-5 compares bytes and not pixels.
//!
//! # Why the crop cannot resample
//!
//! The brief's constraint is "PNG in means identical pixels out inside the
//! crop, same resolution". `DynamicImage::crop_imm` windows the decoded buffer
//! and keeps its variant, and saving a `DynamicImage` as PNG takes the colour
//! type and bit depth from that variant: `ImageLuma8` writes greyscale 8,
//! `ImageRgb16` writes truecolour 16. Nothing here calls `to_rgba8` on the way
//! out - that would silently promote every greyscale page to four 8-bit
//! channels and fail AC-2 - and nothing here scales.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cropper_core::{CropDecision, FlagReason, Rect, Tuning, decide};
use image::{DynamicImage, ImageFormat};

use crate::{codec, copy};

/// What became of one input file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    /// The file that was processed, exactly as it was given.
    pub input: PathBuf,
    /// What was done with it.
    pub outcome: Outcome,
}

/// The three things that can become of a file.
///
/// `Failed` is the one the caller cannot act on: [`process_file`] returns a
/// `FileResult` rather than a `Result` because MC-011's batch reports every
/// file it touched, including the ones it could not read, in one summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The detector found a crop; the cropped PNG was written to `output`.
    Cropped {
        /// The rect the source was cropped to, in source coordinates.
        rect: Rect,
        /// Where the cropped file was written.
        output: PathBuf,
    },
    /// The detector would not commit to a crop; the input was copied to
    /// `output` unchanged and is for a person to look at.
    Flagged {
        /// Why no crop was made.
        reason: Flag,
        /// Where the unchanged copy was written.
        output: PathBuf,
    },
    /// No output was produced: the file could not be read, decoded or
    /// written.
    Failed {
        /// What went wrong, for the run summary.
        error: String,
    },
}

/// Why a file was flagged for review rather than cropped.
///
/// [`Detector`](Flag::Detector) wraps `cropper-core`'s own reason untouched;
/// the other two are the engine's, and the behaviour that produces them
/// belongs to MC-009 (unsupported and corrupt files). They are here now
/// because [`Outcome`] is the type MC-011 serialises and the vocabulary is
/// cheaper to settle once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flag {
    /// The detector declined to crop, for one of its four reasons.
    Detector(FlagReason),
    /// The file is not an image format the engine handles.
    Unsupported,
    /// The file claims a format the engine handles but could not be decoded.
    DecodeFailed(String),
}

/// Crop `input` into `out_dir`, or copy it there unchanged when the detector
/// flags it.
///
/// The output is always `out_dir/<input's file name>`; `out_dir` is created if
/// it does not exist. Returns what happened - this function does not fail, it
/// reports (see [`Outcome::Failed`]).
#[must_use]
pub fn process_file(input: &Path, out_dir: &Path, tuning: &Tuning) -> FileResult {
    FileResult {
        input: input.to_path_buf(),
        // One place turns an error into an outcome, so every step below can
        // use `?` and read as the four-step pipeline it is. MC-009 is what
        // will distinguish "unreadable" from "unsupported" from "corrupt";
        // until then every failure is reported with the message its source
        // produced.
        outcome: match run(input, out_dir, tuning) {
            Ok(outcome) => outcome,
            Err(err) => Outcome::Failed {
                error: err.to_string(),
            },
        },
    }
}

/// The pipeline itself, with every I/O and codec error left to the caller.
fn run(input: &Path, out_dir: &Path, tuning: &Tuning) -> Result<Outcome, Box<dyn Error>> {
    let img = image::open(input)?;
    match decide(&codec::to_luma(&img), tuning) {
        CropDecision::Crop(rect) => Ok(Outcome::Cropped {
            rect,
            output: write_crop(&img, rect, input, out_dir)?,
        }),
        CropDecision::Flag(reason) => Ok(Outcome::Flagged {
            reason: Flag::Detector(reason),
            output: copy::copy_unchanged(input, out_dir)?,
        }),
    }
}

/// Write `rect` of `img` to `out_dir/<input's file name>` as a PNG, at the
/// bit depth and colour type `img` already has. Returns the path written.
fn write_crop(
    img: &DynamicImage,
    rect: Rect,
    input: &Path,
    out_dir: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let name = input
        .file_name()
        .ok_or_else(|| format!("{} has no file name", input.display()))?;
    fs::create_dir_all(out_dir)?;
    let output = out_dir.join(name);
    // `save_with_format` rather than `save`: the format is PNG because this is
    // what MC-008 encodes, not because the input's extension said so.
    img.crop_imm(rect.x, rect.y, rect.w, rect.h)
        .save_with_format(&output, ImageFormat::Png)?;
    Ok(output)
}
