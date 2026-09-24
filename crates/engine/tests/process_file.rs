//! MC-008, AC-1, AC-2, AC-4 and AC-5: one PNG in, one PNG out, losslessly and
//! at the original resolution.
//!
//! `process_file` is the single-file entry MC-011's batch will parallelise:
//! decode the path, convert to luma with the BT.601 weights, ask
//! `cropper_core::decide`, and then either crop the decoded pixels to the rect
//! and encode PNG at the source bit depth and colour type, or - when the
//! detector flags the image - copy the input's bytes unchanged. Either way the
//! output lands at `out_dir/<the input's own file name>`.
//!
//! **MC-010 moved the choice of that name out of `process_file`**, which now
//! takes the output *path* rather than the output *directory*: the planner
//! (`naming::plan_outputs`) decides names and the codec only writes. The call
//! sites below pass `out_dir.join(<the input's own file name>)`, which is the
//! name MC-008 pinned, so every assertion in this file is the one it was.
//! `the_output_keeps_the_input_file_name_case_and_extension` still reads the
//! directory back, so a writer that mangles the name it is handed still fails
//! here.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: nothing in this file. `margin_px` and every other threshold
//!   is read from `Tuning::default()`; the one place a settled number appears
//!   is AC-3's BT.601 weights, which are pinned in `tests/codec.rs`.
//! * **Mechanical**: `process_file(&Path, &Path, &Tuning) -> FileResult` -
//!   input, output path (MC-010; it was the output directory in MC-008 and
//!   MC-009), tuning -
//!   `FileResult { input, outcome }`, `Outcome { Cropped { rect, output },
//!   Flagged { reason, output }, Failed { error } }` and
//!   `Flag { Detector(FlagReason), Unsupported, DecodeFailed(String) }`.
//!   Pinned exactly, including the names the `Debug` derive prints. The PNG
//!   container facts - bit depth at byte 24, colour type at byte 25 - are
//!   mechanical too, and `common::ihdr` reads them out of the file rather than
//!   asking a decoder what it made of them.
//! * **Measured**: the fixture. `common::screenshot` renders one luma plane;
//!   `cropper_core::decide` turns it into `Crop(Rect { x: 15, y: 22, w: 120,
//!   h: 90 })` - `{ 12, 19, 126, 96 }` until MC-049 set the margin to 0 -
//!   and that rect - measured in RED with the plane driven straight
//!   through `decide`, and re-measured by
//!   [`the_fixture_decides_to_crop_the_art_plus_the_margin`] on every run - is
//!   the oracle every expectation below rests on. The four colour renderings
//!   of that plane are grey, so a correct `to_luma` gives all four the same
//!   plane and so the same rect. Every number is in the story's
//!   `## Handoff: RED -> GREEN`.
//!
//! # Why AC-5's fixture carries a `tEXt` chunk
//!
//! "The output file's bytes equal the input's (copied, not re-encoded)" is
//! only a test if a re-encode would produce something else. It would not: a
//! flat image written by this crate's own PNG encoder, decoded and encoded
//! again, comes back **byte for byte identical** (measured in RED: 278 bytes
//! either way). So the fixture carries a metadata chunk the encoder does not
//! preserve - the story's `## Out of scope` says metadata is not kept - and
//! only a real copy can reproduce it.
//! [`re_encoding_the_uniform_fixture_does_not_reproduce_its_bytes`] is the
//! negative control that keeps that reasoning honest.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use cropper_core::{CropDecision, FlagReason, Rect, Tuning, decide};
use cropper_engine::{FileResult, Flag, Outcome, process_file};
use image::DynamicImage;

// --- Harness ----------------------------------------------------------------

/// A temp directory and an existing output directory inside it.
///
/// The output directory is created by the test, so nothing here says whether
/// `process_file` must create a missing one - `copy::copy_unchanged` already
/// does, and that is GREEN's call, not a criterion.
fn workspace() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("an output dir");
    (tmp, out)
}

/// What a successful crop produced.
struct Cropped {
    input: PathBuf,
    output: PathBuf,
    rect: Rect,
}

/// Write the screenshot fixture with `writer`, run `process_file` on it, and
/// insist the outcome is a crop.
fn crop_file(writer: common::Writer, file_name: &str, tmp: &Path, out: &Path) -> Cropped {
    let input = tmp.join(file_name);
    writer(&common::screenshot(), &input);
    let result = process_file(&input, &out.join(file_name), &Tuning::default());
    assert_eq!(
        result.input, input,
        "{file_name}: the result must name the file it was given"
    );
    match result.outcome {
        Outcome::Cropped { rect, output } => Cropped {
            input,
            output,
            rect,
        },
        other => panic!("{file_name}: expected a crop, got {other:?}"),
    }
}

/// Where the output's samples differ from the source's inside `rect`, at most
/// five of them, in the images' own colour type and bit depth.
///
/// Accumulated rather than asserted one pixel at a time: a failure that prints
/// five located samples reads like a bug report, and a failure that prints two
/// 48 KB byte vectors does not.
fn differences(src: &DynamicImage, rect: Rect, out: &DynamicImage) -> Vec<String> {
    let bpp = usize::from(src.color().bytes_per_pixel());
    let want = common::crop_bytes(src, rect);
    let got = common::crop_bytes(
        out,
        Rect {
            x: 0,
            y: 0,
            w: out.width(),
            h: out.height(),
        },
    );
    if want.len() != got.len() {
        return vec![format!(
            "{} source bytes ({:?}, {bpp} per pixel) against {} output bytes ({:?}, {} per pixel)",
            want.len(),
            src.color(),
            got.len(),
            out.color(),
            out.color().bytes_per_pixel()
        )];
    }
    want.iter()
        .zip(&got)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .take(5)
        .map(|(i, (a, b))| {
            let pixel = (i / bpp) as u32;
            format!(
                "({}, {}) channel {}: source {a}, output {b}",
                pixel % rect.w,
                pixel / rect.w,
                i % bpp
            )
        })
        .collect()
}

// --- The fixture is what it claims to be ------------------------------------

/// The oracle every expectation in this file rests on, measured rather than
/// assumed: the detector's own answer for the fixture's luma plane.
///
/// If this ever moves, every rect below is wrong for a reason that has nothing
/// to do with `process_file`, and this test says so first. The literals are
/// the values measured in RED and recorded in the story's handoff.
#[test]
fn the_fixture_decides_to_crop_the_art_plus_the_margin() {
    let t = Tuning::default();
    // MC-049 moved the settled margin from 3 to 0, and with it this rect from
    // `{ 12, 19, 126, 96 }` to the art rect itself.
    assert_eq!(t.margin_px, 0, "the settled default margin");
    assert_eq!(
        common::art_rect(),
        Rect {
            x: 15,
            y: 22,
            w: 120,
            h: 90
        },
        "the fixture's art rect"
    );
    assert_eq!(
        common::crop_rect(),
        Rect {
            x: 15,
            y: 22,
            w: 120,
            h: 90
        },
        "the art rect plus a margin of {}, clamped to a {}x{} image",
        t.margin_px,
        common::width(),
        common::height()
    );
    assert_eq!(
        decide(&common::screenshot(), &t),
        CropDecision::Crop(common::crop_rect()),
        "the detector must crop this scene to the art plus the margin"
    );
}

/// The other half of that oracle: the uniform fixture is the one the detector
/// flags, and it is flagged for the reason AC-5 names rather than some other.
#[test]
fn the_uniform_fixture_decides_to_flag_uniform() {
    assert_eq!(
        decide(&common::uniform(), &Tuning::default()),
        CropDecision::Flag(FlagReason::Uniform),
        "AC-5's fixture must be the one the detector calls uniform"
    );
}

/// Each rendering really is the colour type it is named for. Without this,
/// AC-2's "the output has the same colour type as its input" would be
/// satisfied by an implementation that wrote RGBA8 everywhere *if* the
/// fixtures were RGBA8 everywhere too.
#[test]
fn the_four_renderings_of_the_fixture_are_the_colour_types_they_claim() {
    let (tmp, _out) = workspace();
    let plane = common::screenshot();
    let cases: [(&str, common::Writer, u8, u8); 4] = [
        ("grey.png", common::write_grey8, 8, common::PNG_GREY),
        ("rgb8.png", common::write_rgb8, 8, common::PNG_RGB),
        ("rgb16.png", common::write_rgb16, 16, common::PNG_RGB),
        ("rgba8.png", common::write_rgba8, 8, common::PNG_RGBA),
    ];
    let mut wrong = Vec::new();
    for (name, writer, bit_depth, colour_type) in cases {
        let path = tmp.path().join(name);
        writer(&plane, &path);
        let head = common::ihdr(&path);
        let want = common::Ihdr {
            width: common::width(),
            height: common::height(),
            bit_depth,
            colour_type,
        };
        if head != want {
            wrong.push(format!("{name}: wanted {want:?}, got {head:?}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "the fixture writers no longer produce the colour types AC-2 is about: {wrong:?}"
    );
}

// --- The pinned shape -------------------------------------------------------

/// `FileResult` is two fields with these names and these types, destructured
/// here the way MC-011's batch will destructure it.
#[test]
fn a_file_result_carries_the_input_path_and_one_outcome() {
    let (tmp, out) = workspace();
    let input = tmp.path().join("shot.png");
    common::write_rgba8(&common::screenshot(), &input);

    let FileResult {
        input: reported,
        outcome,
    } = process_file(&input, &out.join("shot.png"), &Tuning::default());

    let _: PathBuf = reported;
    let _: Outcome = outcome;
    let printed = format!(
        "{:?}",
        process_file(&input, &out.join("shot.png"), &Tuning::default())
    );
    assert!(
        printed.contains("FileResult"),
        "a FileResult prints its own name in Debug, so a failing assertion \
         elsewhere reads: got {printed}"
    );
}

#[test]
fn an_outcome_is_a_crop_a_flag_or_a_failure() {
    let all = [
        Outcome::Cropped {
            rect: Rect {
                x: 1,
                y: 2,
                w: 3,
                h: 4,
            },
            output: PathBuf::from("out/a.png"),
        },
        Outcome::Flagged {
            reason: Flag::Detector(FlagReason::Uniform),
            output: PathBuf::from("out/b.png"),
        },
        Outcome::Failed {
            error: String::from("no such file"),
        },
    ];
    assert_ne!(all[0], all[1], "the variants are distinguishable");
    assert_eq!(all[2].clone(), all[2], "an Outcome is Clone and PartialEq");
    let printed = format!("{all:?}");
    for expected in [
        "Cropped",
        "rect: Rect { x: 1, y: 2, w: 3, h: 4 }",
        "Flagged",
        "Detector(Uniform)",
        r#"Failed { error: "no such file" }"#,
    ] {
        assert!(
            printed.contains(expected),
            "an Outcome must print {expected} in its Debug; got {printed}"
        );
    }
}

#[test]
fn a_flag_is_a_detector_reason_an_unsupported_format_or_a_decode_failure() {
    let all = [
        Flag::Detector(FlagReason::Ambiguous),
        Flag::Unsupported,
        Flag::DecodeFailed(String::from("bad header")),
    ];
    assert_eq!(
        format!("{all:?}"),
        r#"[Detector(Ambiguous), Unsupported, DecodeFailed("bad header")]"#,
        "AC's mechanical partition: these three variants, these names"
    );
    assert_ne!(all[1], all[2], "the variants are distinguishable");
    assert_eq!(
        Flag::DecodeFailed(String::from("bad header")),
        all[2].clone(),
        "a Flag is Clone and PartialEq"
    );
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn an_rgba8_screenshot_is_cropped_to_the_detected_rect() {
    let (tmp, out) = workspace();
    let done = crop_file(common::write_rgba8, "shot.png", tmp.path(), &out);

    assert_eq!(
        done.rect,
        common::crop_rect(),
        "AC-1: the reported rect is the detector's"
    );
    assert_eq!(
        done.output,
        out.join("shot.png"),
        "AC-1: the output lands in out_dir under the input's own name"
    );
    assert!(
        done.output.is_file(),
        "AC-1: {} must exist",
        done.output.display()
    );
    let head = common::ihdr(&done.output);
    assert_eq!(
        (head.width, head.height),
        (done.rect.w, done.rect.h),
        "AC-1: the output is the size of the rect - the original resolution, not resampled"
    );
    assert_eq!(
        (head.bit_depth, head.colour_type),
        (8, common::PNG_RGBA),
        "AC-1: an RGBA8 source round trips as RGBA8"
    );
}

#[test]
fn an_rgba8_crop_is_pixel_identical_to_its_source_inside_the_rect() {
    let (tmp, out) = workspace();
    let done = crop_file(common::write_rgba8, "shot.png", tmp.path(), &out);
    let src = common::decode(&done.input);
    let got = common::decode(&done.output);
    let differing = differences(&src, done.rect, &got);
    assert!(
        differing.is_empty(),
        "AC-1: every output pixel must equal the source pixel at (rect.x + i, rect.y + j), \
         channel for channel; first differences inside {:?}: {differing:?}",
        done.rect
    );
}

// --- AC-2 -------------------------------------------------------------------

/// Same scene, one colour type per test, so a GREEN that encoded everything as
/// RGBA8 fails by name rather than in a list.
fn same_colour_type_and_pixels(
    writer: common::Writer,
    file_name: &str,
    bit_depth: u8,
    colour_type: u8,
) {
    let (tmp, out) = workspace();
    let done = crop_file(writer, file_name, tmp.path(), &out);

    let source = common::ihdr(&done.input);
    assert_eq!(
        (source.bit_depth, source.colour_type),
        (bit_depth, colour_type),
        "AC-2: {file_name} must be written as the colour type this test is about"
    );
    let head = common::ihdr(&done.output);
    assert_eq!(
        (head.bit_depth, head.colour_type),
        (source.bit_depth, source.colour_type),
        "AC-2: {file_name} must keep its colour type and bit depth; \
         PNG colour types are 0 greyscale, 2 truecolour, 6 truecolour with alpha"
    );
    assert_eq!(
        (head.width, head.height),
        (done.rect.w, done.rect.h),
        "AC-2: {file_name} is cropped to the rect at the original resolution"
    );
    assert_eq!(
        done.rect,
        common::crop_rect(),
        "AC-2: {file_name} is the same scene, so it gets the same rect"
    );

    let src = common::decode(&done.input);
    let got = common::decode(&done.output);
    let differing = differences(&src, done.rect, &got);
    assert!(
        differing.is_empty(),
        "AC-2: {file_name} must be pixel-identical inside the rect; \
         first differences: {differing:?}"
    );
}

#[test]
fn an_eight_bit_greyscale_png_stays_eight_bit_greyscale() {
    same_colour_type_and_pixels(common::write_grey8, "grey.png", 8, common::PNG_GREY);
}

#[test]
fn an_eight_bit_rgb_png_stays_eight_bit_rgb() {
    same_colour_type_and_pixels(common::write_rgb8, "rgb8.png", 8, common::PNG_RGB);
}

#[test]
fn a_sixteen_bit_rgb_png_stays_sixteen_bit_rgb() {
    same_colour_type_and_pixels(common::write_rgb16, "rgb16.png", 16, common::PNG_RGB);
}

// --- AC-4 -------------------------------------------------------------------

#[test]
fn the_output_keeps_the_input_file_name_case_and_extension() {
    let (tmp, out) = workspace();
    let done = crop_file(common::write_rgba8, "Shot.PNG", tmp.path(), &out);

    assert_eq!(
        done.output,
        out.join("Shot.PNG"),
        "AC-4: the original file name, case and extension, untouched"
    );
    // The path is what `process_file` reported; this is what is on the disk.
    // On a case-insensitive filesystem the two differ whenever an
    // implementation lowercases the name it writes.
    let written: Vec<String> = fs::read_dir(&out)
        .expect("the output dir")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        written,
        vec![String::from("Shot.PNG")],
        "AC-4: the file on disk is named Shot.PNG, not shot.png or Shot.png"
    );
}

// --- AC-5 -------------------------------------------------------------------

#[test]
fn a_uniform_png_is_flagged_and_copied_byte_for_byte() {
    let (tmp, out) = workspace();
    let input = tmp.path().join("flat.png");
    common::write_marked_uniform_png(&common::uniform(), &input);

    let result = process_file(&input, &out.join("flat.png"), &Tuning::default());
    assert_eq!(
        result.input, input,
        "the result names the file it was given"
    );
    let (reason, output) = match result.outcome {
        Outcome::Flagged { reason, output } => (reason, output),
        other => panic!("AC-5: a uniform PNG must be flagged, not cropped: {other:?}"),
    };
    assert_eq!(
        reason,
        Flag::Detector(FlagReason::Uniform),
        "AC-5: the detector's own reason, wrapped"
    );
    assert_eq!(
        output,
        out.join("flat.png"),
        "AC-5: a flagged file is still written under its own name"
    );
    let before = fs::read(&input).expect("the input is still readable");
    let after = fs::read(&output).expect("the output exists");
    assert_eq!(
        after.len(),
        before.len(),
        "AC-5: the output must be the input's bytes - {} bytes in, {} out",
        before.len(),
        after.len()
    );
    assert_eq!(
        after, before,
        "AC-5: the output's bytes must equal the input's - copied, not re-encoded"
    );
}

/// The control that gives the byte comparison above its force.
///
/// Measured in RED: the marked fixture is 359 bytes, and decoding it and
/// encoding it again gives 278 bytes without the marker. Were the fixture a
/// plain PNG from this crate's encoder, the re-encode would reproduce it
/// exactly and AC-5 would pass for an implementation that decoded and
/// re-encoded every flagged file.
#[test]
fn re_encoding_the_uniform_fixture_does_not_reproduce_its_bytes() {
    let (tmp, _out) = workspace();
    let input = tmp.path().join("flat.png");
    common::write_marked_uniform_png(&common::uniform(), &input);
    let original = fs::read(&input).expect("the fixture was just written");
    assert!(
        original
            .windows(common::MARKER.len())
            .any(|w| w == common::MARKER),
        "the fixture must carry the marker chunk"
    );

    let again = tmp.path().join("re-encoded.png");
    common::decode(&input)
        .save_with_format(&again, image::ImageFormat::Png)
        .expect("re-encoding the fixture");
    let re_encoded = fs::read(&again).expect("the re-encode exists");

    assert!(
        !re_encoded
            .windows(common::MARKER.len())
            .any(|w| w == common::MARKER),
        "a re-encode must not carry the marker - metadata is not preserved"
    );
    assert_ne!(
        re_encoded, original,
        "a re-encode must not reproduce the fixture's bytes, or AC-5's byte comparison \
         would pass without anything being copied"
    );
}
