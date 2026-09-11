//! One file in, one file out (MC-008): the single-file entry the batch runner
//! parallelises (MC-011).
//!
//! [`process_file`] is the whole of the engine's per-file work, in five steps
//! and no branches of its own beyond the format's and the detector's answers:
//!
//! 1. read the format from the file's *content*
//!    ([`codec::detect_format`](crate::codec::detect_format)'s rule, applied
//!    here through `ImageReader`) - anything that is not PNG, JPEG or WebP is
//!    [`Flag::Unsupported`] and goes no further (MC-009 AC-4);
//! 2. decode the path with `image`; a file that claims one of the three and
//!    will not decode is [`Flag::DecodeFailed`] (MC-009 AC-5);
//! 3. convert to the luma plane with [`codec::to_luma`](crate::codec::to_luma)
//!    - BT.601, alpha ignored;
//! 4. ask [`cropper_core::decide`], which is the only place a crop is judged;
//! 5. act on that answer: crop the *decoded pixels* to the rect and re-encode
//!    them in the format they came in - PNG lossless, JPEG at quality 100,
//!    WebP lossless (`docs/wiki/architecture.md` decision 4) - or copy the
//!    input's bytes unchanged with
//!    [`copy::copy_unchanged`](crate::copy::copy_unchanged).
//!
//! Every one of those paths writes to `out_dir/<the input's own file name>`,
//! case and extension untouched (MC-008 AC-4), so no input is ever dropped on
//! the floor: the user's output folder has as many files in it as they
//! selected. Name collisions are MC-010's.
//!
//! # Why the content decides the format and the name does not
//!
//! A screenshot's extension is whatever the last tool to touch it called the
//! file. MC-009 AC-6 is the case that follows: a PNG named `x.jpg` is cropped
//! losslessly as the PNG it is - `image::open` would have refused it, because
//! it guesses from the path - and written back as `x.jpg`, because renaming a
//! user's file is not this program's business.
//!
//! # Unsupported and DecodeFailed are a distinction the extension makes
//!
//! A BMP and a text file are both `Unsupported`; 200 bytes of junk named
//! `x.png` is `DecodeFailed`. The first two carry no format this engine
//! handles - the BMP's own container names one it does not - while the third
//! claims PNG in its name and cannot honour the claim. Only the name can tell
//! the text file from the junk, since neither is recognisable by content, and
//! that is precisely what the distinction is *for*: "you gave me a PNG and I
//! could not read it" is a different thing to tell a user than "this is not an
//! image I work with".
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
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use cropper_core::{CropDecision, FlagReason, Rect, Tuning, decide};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageFormat, ImageReader};

use crate::codec::SourceFormat;
use crate::{codec, copy};

/// The quality every JPEG the engine writes is encoded at
/// (`docs/wiki/architecture.md` decision 4, MC-009 AC-1).
///
/// It has to be said out loud: `JpegEncoder::new` - and therefore
/// `DynamicImage::save_with_format` - is `new_with_quality(w, 75)`
/// (`image-0.25.10/src/codecs/jpeg/encoder.rs:391`), which on MC-009's fixture
/// costs a mean absolute error of 5.78 levels against the decoded source where
/// quality 100 costs 0.35. A user who re-crops the same page twice would be
/// re-compressing it each time, and the default is where that damage would
/// come from silently.
const JPEG_QUALITY: u8 = 100;

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
        // use `?` and read as the four-step pipeline it is. Since MC-009 the
        // codec's own two failures are outcomes rather than errors - a format
        // the engine does not handle is `Unsupported` and a file that claims
        // one it does and will not decode is `DecodeFailed`, and both are
        // still copied - so what reaches here is I/O: an unreadable input, an
        // output folder that cannot be created or written.
        outcome: match run(input, out_dir, tuning) {
            Ok(outcome) => outcome,
            Err(err) => Outcome::Failed {
                error: err.to_string(),
            },
        },
    }
}

/// The pipeline itself, with every I/O error left to the caller.
fn run(input: &Path, out_dir: &Path, tuning: &Tuning) -> Result<Outcome, Box<dyn Error>> {
    // Not `image::open`, which is `ImageReader::open(path)?.decode()` and
    // guesses the format from the *path* alone: on a PNG named `x.jpg` it
    // fails with "Illegal start bytes:8950" (AC-6). `with_guessed_format`
    // replaces that guess with the content's whenever the content is
    // recognisable, and keeps the extension's otherwise - which is what
    // separates the story's two flag reasons, because AC-4's text file and
    // AC-5's 200 random bytes are equally unrecognisable by content and only
    // the latter claims a format the engine handles.
    let reader = ImageReader::open(input)?.with_guessed_format()?;
    let Some(format) = reader.format().and_then(codec::source_format) else {
        return flagged(Flag::Unsupported, input, out_dir);
    };
    let img = match reader.decode() {
        Ok(img) => img,
        Err(err) => return flagged(Flag::DecodeFailed(err.to_string()), input, out_dir),
    };
    match decide(&codec::to_luma(&img), tuning) {
        CropDecision::Crop(rect) => Ok(Outcome::Cropped {
            rect,
            output: write_crop(&img, rect, format, input, out_dir)?,
        }),
        CropDecision::Flag(reason) => flagged(Flag::Detector(reason), input, out_dir),
    }
}

/// Copy `input` into `out_dir` unchanged and report it as flagged for
/// `reason`. All three reasons end the same way, by design: every input
/// reaches the output folder, and a file nobody cropped is handed back exactly
/// as it arrived.
fn flagged(reason: Flag, input: &Path, out_dir: &Path) -> Result<Outcome, Box<dyn Error>> {
    Ok(Outcome::Flagged {
        reason,
        output: copy::copy_unchanged(input, out_dir)?,
    })
}

/// Write `rect` of `img` to `out_dir/<input's file name>` in `format`, at the
/// highest quality that format has. Returns the path written.
///
/// The name is the input's, extension and case untouched, even when the
/// extension disagrees with `format` (AC-6): the container follows the file's
/// content, the name follows the user.
fn write_crop(
    img: &DynamicImage,
    rect: Rect,
    format: SourceFormat,
    input: &Path,
    out_dir: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let name = input
        .file_name()
        .ok_or_else(|| format!("{} has no file name", input.display()))?;
    fs::create_dir_all(out_dir)?;
    let output = out_dir.join(name);
    let cropped = img.crop_imm(rect.x, rect.y, rect.w, rect.h);
    // `save_with_format` rather than `save`: the format is the one detected
    // from the input's content, never the one its extension claims.
    match format {
        // Both of these are lossless and both keep the `DynamicImage`'s own
        // colour type, so the crop's pixels survive exactly. `save_with_format`
        // is the whole of the WebP case because `image`'s only WebP encoder is
        // `WebPEncoder::new_lossless` - decision 4's "lossless" comes free, and
        // the crate narrows a 16-bit source to 8 bits on the way rather than
        // refusing it.
        SourceFormat::Png => cropped.save_with_format(&output, ImageFormat::Png)?,
        SourceFormat::WebP => cropped.save_with_format(&output, ImageFormat::WebP)?,
        SourceFormat::Jpeg => write_jpeg(&cropped, &output)?,
    }
    Ok(output)
}

/// Write `img` as a baseline JPEG at [`JPEG_QUALITY`], 4:4:4.
///
/// Not `save_with_format`, which would silently be quality 75 (see
/// [`JPEG_QUALITY`]). The cost of going to the encoder directly is that it is
/// narrower than the PNG one - `JpegEncoder::encode` accepts only `L8` and
/// `Rgb8`, and hands back `UnsupportedErrorKind::Color` for anything else,
/// where `save_with_format` would have narrowed the image first. So the
/// narrowing is done here, by `to_rgb8`: JPEG has no alpha channel and no
/// 16-bit mode to preserve, and a greyscale source comes back as a grey RGB
/// JPEG rather than an error. Sampling factors are not set because
/// `JpegEncoder` writes `h = v = 1` for all three components, which is the
/// 4:4:4 decision 4 asks for.
fn write_jpeg(img: &DynamicImage, output: &Path) -> Result<(), Box<dyn Error>> {
    let rgb = img.to_rgb8();
    let mut file = BufWriter::new(File::create(output)?);
    JpegEncoder::new_with_quality(&mut file, JPEG_QUALITY).encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        ExtendedColorType::Rgb8,
    )?;
    // A `BufWriter` flushes on drop and swallows the error when it does; this
    // is where a full disk is allowed to be an `Outcome::Failed` rather than a
    // truncated JPEG nobody was told about.
    file.flush()?;
    Ok(())
}
