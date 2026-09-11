//! MC-009, AC-1, AC-2, AC-4, AC-5 and AC-6: JPEG and WebP are cropped and
//! re-saved at maximum quality, the format comes from the file's content, and
//! anything the engine cannot crop still reaches the output folder unchanged.
//!
//! MC-008 put one PNG through `process_file`. This file puts the same scene
//! through the other two containers the engine handles, and puts three files
//! through it that it does not handle at all.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**, and read out rather than re-derived: JPEG quality **100**,
//!   WebP **lossless** (`docs/wiki/architecture.md` decision 4, and the story's
//!   `## Model guidance`), and AC-1's bound of **2.0** mean absolute error.
//!   Two of those three are pinned mechanically as well as behaviourally -
//!   quality 100 is visible in a JPEG's own quantisation tables and lossless is
//!   visible in a WebP's chunk name - because a bound alone cannot tell a
//!   deliberate quality from a default one.
//! * **Mechanical**: the outcomes, the file names, and the containers. A PNG
//!   starts with an eight-byte signature, a JPEG with `FF D8`, a WebP is a RIFF
//!   file whose first chunk is `VP8L` when it is lossless, and a JPEG's frame
//!   header carries its sampling factors. All of that is read out of the bytes
//!   by `common`, never from a decoder's opinion of them.
//! * **Oracle-free**: exactly one thing, AC-1's control. The quality-30
//!   re-encode of the same crop was measured in RED at **15.0638** against a
//!   floor of 4.0 - the fixture's period-2 jittered checkerboard is about the
//!   roughest texture a DCT can be asked to carry, and it shows. Every number
//!   in this file is in the story's `## Handoff: RED -> GREEN`.
//!
//! # The rect is the same for every container, and that is measured
//!
//! `decide` answers `Crop(Rect { x: 12, y: 19, w: 126, h: 96 })` for the PNG
//! rendering (MC-008), and - measured in RED - for the quality-95 JPEG, for the
//! lossless WebP and for the PNG named `x.jpg` as well. Quality 95 is gentle
//! enough that no border row leaves `uniform_tolerance`, so one oracle serves
//! the whole file and [`every_container_of_the_fixture_decides_to_crop_the_same_rect`]
//! is where it is checked. If a future codec change moves any of them, that
//! test says so before any criterion below fails for a reason that has nothing
//! to do with `process_file`.
//!
//! # AC-3 is not here
//!
//! AC-3 asks for "a recipe saved as lossy WebP". Nothing in this workspace can
//! write one: `image` 0.25's `WebPEncoder` is documented lossless-only and
//! `image-webp` 0.2.4 writes a `VP8L` chunk and no other. It is escalated in
//! the story's `## Notes`, not quietly reinterpreted here.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use cropper_core::{CropDecision, Luma, Rect, Tuning, decide};
use cropper_engine::codec::{detect_format, to_luma};
use cropper_engine::{Flag, Outcome, SourceFormat, process_file};
use image::ImageFormat;

// --- Harness ----------------------------------------------------------------

/// A temp directory and an existing output directory inside it.
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

/// Write a fixture with `write`, run `process_file` on it, and insist the
/// outcome is a crop.
fn crop_file(write: impl FnOnce(&Path), file_name: &str, tmp: &Path, out: &Path) -> Cropped {
    let input = tmp.join(file_name);
    write(&input);
    let result = process_file(&input, out, &Tuning::default());
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

/// Write a fixture with `write`, run `process_file` on it, and insist it was
/// flagged. Returns the input path, the reason and the output path, for the
/// caller to judge - AC-4 and AC-5 want different reasons out of the same
/// shape.
fn flag_file(
    write: impl FnOnce(&Path),
    file_name: &str,
    tmp: &Path,
    out: &Path,
) -> (PathBuf, Flag, PathBuf) {
    let input = tmp.join(file_name);
    write(&input);
    let result = process_file(&input, out, &Tuning::default());
    assert_eq!(
        result.input, input,
        "{file_name}: the result must name the file it was given"
    );
    match result.outcome {
        Outcome::Flagged { reason, output } => (input, reason, output),
        other => panic!(
            "{file_name}: a file the engine cannot crop must be flagged and copied, \
             not {other:?} - every input reaches the output folder"
        ),
    }
}

/// The output is the input, byte for byte. The length is asserted first so a
/// failure carries two numbers instead of a screenful of hex.
fn assert_copied(input: &Path, output: &Path, ac: &str) {
    let before = fs::read(input).expect("the input is still readable");
    let after = fs::read(output).expect("the output exists");
    assert_eq!(
        after.len(),
        before.len(),
        "{ac}: the output must be the input's bytes - {} bytes in, {} out",
        before.len(),
        after.len()
    );
    assert_eq!(
        after, before,
        "{ac}: the output's bytes must equal the input's - copied, not re-encoded"
    );
}

/// The rect every fixture in this file is cropped to, measured in RED and
/// re-measured by [`every_container_of_the_fixture_decides_to_crop_the_same_rect`].
fn rect() -> Rect {
    common::crop_rect()
}

/// The screenshot scene's decoded source pixels, windowed to [`rect`].
fn source_crop(path: &Path, format: ImageFormat) -> image::DynamicImage {
    let src = common::decode_as(path, format);
    let r = rect();
    src.crop_imm(r.x, r.y, r.w, r.h)
}

// --- The fixtures are what they claim to be ---------------------------------

/// Every assertion below about "the output is a JPEG" or "the bytes were
/// copied" leans on the fixtures really being the containers they are named
/// for. Read from the files' own bytes, never from a decoder.
#[test]
fn each_fixture_is_the_container_its_name_claims() {
    let (tmp, _out) = workspace();
    let dir = tmp.path();
    let plane = common::screenshot();

    let png = dir.join("shot.png");
    common::write_rgb8(&plane, &png);
    let jpg = dir.join("shot.jpg");
    common::write_jpeg_q95(&plane, &jpg);
    let webp = dir.join("shot.webp");
    common::write_webp(&plane, &webp);
    let bmp = dir.join("x.bmp");
    common::write_bmp(&common::small_gradient(), &bmp);
    let txt = dir.join("x.txt");
    common::write_text(&txt);
    let junk = dir.join("x.png");
    common::write_random_bytes(&junk);

    assert_eq!(
        common::head(&png, 8),
        common::PNG_SIGNATURE,
        "the PNG fixture must carry the PNG signature"
    );

    assert_eq!(
        common::head(&jpg, 2),
        common::JPEG_SIGNATURE,
        "the JPEG fixture must start with SOI"
    );
    let head = common::jpeg_info(&jpg);
    assert_eq!(
        (head.width, head.height),
        (common::width() as u16, common::height() as u16),
        "the JPEG fixture is the whole scene"
    );
    assert_eq!(
        head.quant_tables.len(),
        2,
        "a quality-{} JPEG declares a luma and a chroma quantisation table",
        common::JPEG_SOURCE_QUALITY
    );
    assert!(
        head.quant_tables.iter().flatten().any(|&entry| entry > 1),
        "the source JPEG is quality {} and must NOT carry the all-ones tables \
         quality 100 writes, or AC-1 would be comparing a file with itself: {:?}",
        common::JPEG_SOURCE_QUALITY,
        head.quant_tables
    );

    assert_eq!(
        common::webp_fourcc(&webp),
        "VP8L",
        "the WebP fixture must be lossless - the only mode this crate writes"
    );

    assert_eq!(
        common::head(&bmp, 2),
        b"BM",
        "the BMP fixture must carry the BMP signature"
    );
    assert_eq!(
        image::guess_format(&fs::read(&bmp).expect("the BMP fixture")).ok(),
        Some(ImageFormat::Bmp),
        "AC-4's BMP must be a real BMP by content: the criterion is about a valid \
         image in a format the engine does not handle, not about a corrupt file"
    );

    assert_eq!(
        fs::read(&txt).expect("the text fixture"),
        common::TEXT_FIXTURE,
        "the text fixture is plain text"
    );
    assert_eq!(
        fs::metadata(&junk).expect("the junk fixture").len() as usize,
        common::RANDOM_LEN,
        "AC-5's file is {} bytes",
        common::RANDOM_LEN
    );
}

/// Every container this engine can write, in a form the control below can walk.
///
/// The list itself is [`SourceFormat`]: `process.rs`'s `write_crop` matches on
/// it with one arm per variant, so what the engine can encode and what this
/// enum names are the same three things by construction.
const ENGINE_CONTAINERS: [SourceFormat; 3] =
    [SourceFormat::Png, SourceFormat::Jpeg, SourceFormat::WebP];

/// One container the engine writes, the name a fixture of the scene takes in
/// it, and the `common` writer that produces that fixture.
type ContainerFixture = (SourceFormat, &'static str, fn(&Luma, &Path));

/// The bytes every file this engine writes in `format` must begin with.
///
/// The `match` is exhaustive on purpose, and that is load-bearing rather than
/// idiomatic: a control which claims "the engine can write nothing else" is
/// only as good as its enumeration, so a fourth [`SourceFormat`] variant has to
/// stop this file compiling rather than quietly shrink the claim.
fn container_signature(format: SourceFormat) -> &'static [u8] {
    match format {
        SourceFormat::Png => common::PNG_SIGNATURE,
        SourceFormat::Jpeg => common::JPEG_SIGNATURE,
        // A WebP is a RIFF container; which codec chunk sits inside it is
        // AC-2's business, not this test's.
        SourceFormat::WebP => b"RIFF",
    }
}

/// The control that gives AC-4's and AC-5's byte comparisons their force.
///
/// MC-008 needed a `tEXt` chunk to make "copied, not re-encoded" mean
/// something, because a flat PNG re-encoded by this crate came back
/// byte-identical. Here the argument is different and stronger, and this test
/// is where it is made executable, in three linked steps:
///
/// 1. **The engine writes three containers and no others.** That is not
///    asserted from the dependency's feature list: each of the three is put
///    through `process_file` and the container it produced is read out of the
///    output's first bytes. The enumeration is closed by
///    [`container_signature`]'s exhaustive `match`.
/// 2. **The engine calls none of these three fixtures one of those formats.**
///    `detect_format` answers `None` for the BMP, the text file and the junk,
///    from the bytes alone.
/// 3. **None of the three begins with any of the signatures from step 1.** The
///    BMP begins `42 4D`, which is not the PNG signature, not `FF D8` and not
///    `RIFF`; the other two are not images at all.
///
/// So whichever of its three encoders had run, the file it produced would have
/// begun with a signature none of these three carries. Byte equality therefore
/// means a copy, and AC-4 and AC-5 are testing what they claim to test.
///
/// # Why this is no longer the argument RED first made
///
/// The test originally asserted that `ImageFormat::Bmp` was neither readable
/// nor writable here - that the `bmp` codec was compiled out of this build.
/// That premise is false under `cargo test --workspace`, which is the `unit`
/// gate's own command, and it is false *only on some platforms*:
/// `crates/app` -> `eframe` -> `egui-winit/clipboard` -> `arboard/image-data`
/// takes `image` with `features = ["png", "bmp"]` on Windows
/// (`arboard-3.6.1/Cargo.toml:156`), `["png"]` on Linux and `["tiff"]` on
/// macOS, and cargo unifies features across a workspace build. A control that
/// passes or fails according to the operating system is not a control.
///
/// Nothing about the engine's behaviour changed, and nothing here is weaker
/// for it: `codec::source_format` maps `ImageFormat::Bmp` to `None` because
/// MC-009 AC-4 says a BMP is `Unsupported`, and it never asks which decoders
/// happen to be compiled in - so neither do the assertions below.
#[test]
fn no_copy_of_an_unsupported_file_could_be_a_re_encode() {
    let (tmp, out) = workspace();
    let dir = tmp.path();

    // 1. What the engine can write, measured through the engine itself.
    let plane = common::screenshot();
    let sources: [ContainerFixture; 3] = [
        (SourceFormat::Png, "shot.png", common::write_rgb8),
        (SourceFormat::Jpeg, "shot.jpg", common::write_jpeg_q95),
        (SourceFormat::WebP, "shot.webp", common::write_webp),
    ];
    for (format, name, write) in sources {
        let done = crop_file(|path| write(&plane, path), name, dir, &out);
        let signature = container_signature(format);
        assert_eq!(
            common::head(&done.output, signature.len()),
            signature,
            "the crop the engine wrote from a {format:?} source must begin with \
             that container's signature: this control's whole argument is that \
             the engine writes these three containers and nothing else"
        );
    }

    // 2 and 3. The three files AC-4 and AC-5 are about.
    let bmp = dir.join("x.bmp");
    common::write_bmp(&common::small_gradient(), &bmp);
    let txt = dir.join("x.txt");
    common::write_text(&txt);
    let junk = dir.join("x.png");
    common::write_random_bytes(&junk);

    for (path, what) in [
        (&bmp, "a BMP"),
        (&txt, "a text file"),
        (&junk, "200 random bytes"),
    ] {
        let bytes = fs::read(path).expect("the fixture");
        assert_eq!(
            detect_format(&bytes),
            None,
            "{what} must be none of the formats the engine encodes, so that it is \
             copied rather than decoded and written back out - and the engine \
             decides that from these bytes as a matter of policy, never from \
             which decoders this build happens to have compiled in"
        );
        for format in ENGINE_CONTAINERS {
            let signature = container_signature(format);
            assert!(
                !bytes.starts_with(signature),
                "{what} begins {:02X?}, which must not be the {format:?} \
                 signature {signature:02X?}: if it were, a re-encode could \
                 coincide with a copy and AC-4's byte comparison would prove \
                 nothing",
                &bytes[..signature.len().min(bytes.len())]
            );
        }
    }

    // The text file and the 200 junk bytes carry the argument one step
    // further, and that step is platform-independent for them in a way it
    // could never be for the BMP: they are not images in any format, so no
    // decoder could get far enough to re-encode them whatever is compiled in.
    for (path, what) in [(&txt, "the text fixture"), (&junk, "AC-5's 200 bytes")] {
        assert!(
            image::guess_format(&fs::read(path).expect("the fixture")).is_err(),
            "{what} must not carry any image format's signature - the junk is \
             drawn from a fixed seed, so this is an assertion and not a hope"
        );
        assert!(
            image::open(path).is_err(),
            "{what} must not decode, or its output bytes could be a re-encode \
             rather than a copy"
        );
    }
}

// --- The format comes from the content --------------------------------------

/// AC-6's mechanism, and the vocabulary AC-4 and AC-5 rest on: what the engine
/// thinks a file is, decided by its bytes alone.
///
/// The `x.jpg` row is AC-6 itself - a PNG under a JPEG's name - and the last
/// three rows are what "not PNG, JPEG or WebP" means for AC-4 and AC-5.
#[test]
fn the_format_of_every_fixture_is_read_from_its_content() {
    let (tmp, _out) = workspace();
    let dir = tmp.path();
    let plane = common::screenshot();

    let png = dir.join("shot.png");
    common::write_rgb8(&plane, &png);
    let jpg = dir.join("shot.jpg");
    common::write_jpeg_q95(&plane, &jpg);
    let webp = dir.join("shot.webp");
    common::write_webp(&plane, &webp);
    let lying = dir.join("x.jpg");
    common::write_rgb8(&plane, &lying);
    let bmp = dir.join("x.bmp");
    common::write_bmp(&common::small_gradient(), &bmp);
    let txt = dir.join("x.txt");
    common::write_text(&txt);
    let junk = dir.join("x.png");
    common::write_random_bytes(&junk);

    let cases: [(&Path, Option<SourceFormat>); 7] = [
        (&png, Some(SourceFormat::Png)),
        (&jpg, Some(SourceFormat::Jpeg)),
        (&webp, Some(SourceFormat::WebP)),
        (&lying, Some(SourceFormat::Png)),
        (&bmp, None),
        (&txt, None),
        (&junk, None),
    ];
    let mut wrong = Vec::new();
    for (path, want) in cases {
        let bytes = fs::read(path).expect("the fixture");
        let got = detect_format(&bytes);
        if got != want {
            wrong.push(format!(
                "{}: wanted {want:?}, got {got:?}",
                path.file_name().expect("a file name").to_string_lossy()
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the format is the container's, never the name's: {wrong:?}"
    );
}

/// The oracle every rect in this file rests on, measured rather than assumed.
///
/// The four containers hold the same grey scene and the two lossy ones are
/// gentle enough not to move a border out of `uniform_tolerance`, so all four
/// decide on the identical rect - the one MC-008 settled. If this moves, the
/// criteria below are failing for a reason that is not `process_file`'s.
#[test]
fn every_container_of_the_fixture_decides_to_crop_the_same_rect() {
    let (tmp, _out) = workspace();
    let dir = tmp.path();
    let plane = common::screenshot();

    assert_eq!(
        rect(),
        Rect {
            x: 12,
            y: 19,
            w: 126,
            h: 96
        },
        "MC-008's rect: the art plus a margin of {}",
        Tuning::default().margin_px
    );

    let png = dir.join("shot.png");
    common::write_rgb8(&plane, &png);
    let jpg = dir.join("shot.jpg");
    common::write_jpeg_q95(&plane, &jpg);
    let webp = dir.join("shot.webp");
    common::write_webp(&plane, &webp);
    let lying = dir.join("x.jpg");
    common::write_rgb8(&plane, &lying);

    let cases: [(&Path, ImageFormat); 4] = [
        (&png, ImageFormat::Png),
        (&jpg, ImageFormat::Jpeg),
        (&webp, ImageFormat::WebP),
        (&lying, ImageFormat::Png),
    ];
    let mut wrong = Vec::new();
    for (path, format) in cases {
        let decoded = common::decode_as(path, format);
        let answer = decide(&to_luma(&decoded), &Tuning::default());
        if answer != CropDecision::Crop(rect()) {
            wrong.push(format!(
                "{}: {answer:?}",
                path.file_name().expect("a file name").to_string_lossy()
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "every container of the scene must decide on {:?}: {wrong:?}",
        rect()
    );
}

// --- AC-1: JPEG -------------------------------------------------------------

#[test]
fn a_jpeg_is_cropped_and_re_encoded_as_a_jpeg() {
    let (tmp, out) = workspace();
    let plane = common::screenshot();
    let done = crop_file(
        |path| common::write_jpeg_q95(&plane, path),
        "shot.jpg",
        tmp.path(),
        &out,
    );

    assert_eq!(
        done.rect,
        rect(),
        "AC-1: the reported rect is the detector's"
    );
    assert_eq!(
        done.output,
        out.join("shot.jpg"),
        "AC-1: the output is named like the input"
    );
    assert!(
        done.output.is_file(),
        "AC-1: {} must exist",
        done.output.display()
    );
    assert_eq!(
        common::head(&done.output, 2),
        common::JPEG_SIGNATURE,
        "AC-1: a JPEG in means a JPEG out - the output must start with SOI"
    );
    let decoded = common::decode_as(&done.output, ImageFormat::Jpeg);
    assert_eq!(
        (decoded.width(), decoded.height()),
        (done.rect.w, done.rect.h),
        "AC-1: the output decodes to rect.w x rect.h - the original resolution, not resampled"
    );
}

#[test]
fn a_re_encoded_jpeg_stays_within_two_levels_of_the_decoded_source() {
    let (tmp, out) = workspace();
    let plane = common::screenshot();
    let done = crop_file(
        |path| common::write_jpeg_q95(&plane, path),
        "shot.jpg",
        tmp.path(),
        &out,
    );

    let src = common::decode_as(&done.input, ImageFormat::Jpeg);
    let got = common::decode_as(&done.output, ImageFormat::Jpeg);
    let error = common::mean_abs_error(&src, done.rect, &got);
    assert!(
        error <= 2.0,
        "AC-1: the output must be within a mean absolute per-channel error of 2.0 of the \
         DECODED source inside {:?}; measured {error:.4}. Measured in RED on this fixture: \
         quality 100 scores 0.3467 and quality 75 - what save_with_format writes by \
         default - scores 5.7838",
        done.rect
    );
}

/// AC-1's control, and the only oracle-free number in the story: the bound of
/// 2.0 has to be one a wrong implementation misses.
///
/// The same crop, from the same decoded source, re-encoded at quality 30.
/// Measured in RED: **15.0638**, against a floor of 4.0. The fixture's art is a
/// period-2 jittered checkerboard, which is the highest spatial frequency an
/// 8x8 DCT can carry, so the gap is not marginal.
#[test]
fn a_quality_thirty_re_encode_of_the_same_crop_is_far_outside_the_bound() {
    let (tmp, _out) = workspace();
    let dir = tmp.path();
    let plane = common::screenshot();
    let input = dir.join("shot.jpg");
    common::write_jpeg_q95(&plane, &input);

    let crop = source_crop(&input, ImageFormat::Jpeg);
    let low = dir.join("q30.jpg");
    common::encode_jpeg(&crop, &low, common::JPEG_CONTROL_QUALITY);
    let error = common::mean_abs_error(
        &crop,
        Rect {
            x: 0,
            y: 0,
            w: crop.width(),
            h: crop.height(),
        },
        &common::decode_as(&low, ImageFormat::Jpeg),
    );
    assert!(
        error > 4.0,
        "AC-1's control: a quality-{} re-encode of the same crop must score well above \
         the 2.0 bound, or the bound cannot tell a correct implementation from a lossy \
         one; measured {error:.4}",
        common::JPEG_CONTROL_QUALITY
    );
}

/// The settled number, read out of `## Model guidance` and pinned where it is
/// written down rather than inferred: **quality 100**, and 4:4:4.
///
/// This encoder derives its quantisation tables with the libjpeg scaling
/// (`image-0.25.10/src/codecs/jpeg/encoder.rs:430`), and at quality 100 the
/// scale is zero, so every entry of every table clamps to 1. At quality 99 some
/// entries are 2; at the 75 `save_with_format` writes, most are far larger.
/// `JpegEncoder` always declares `h = v = 1` for all three components, so the
/// sampling assertion pins `architecture.md` decision 4's "4:4:4" against a
/// future encoder change rather than against this one.
#[test]
fn a_re_encoded_jpeg_carries_the_quality_100_tables_and_no_chroma_subsampling() {
    let (tmp, out) = workspace();
    let plane = common::screenshot();
    let done = crop_file(
        |path| common::write_jpeg_q95(&plane, path),
        "shot.jpg",
        tmp.path(),
        &out,
    );

    let head = common::jpeg_info(&done.output);
    assert_eq!(
        (head.width, head.height),
        (done.rect.w as u16, done.rect.h as u16),
        "the frame header is the size of the rect"
    );
    assert!(
        !head.quant_tables.is_empty(),
        "the output declares no quantisation table at all: {head:?}"
    );
    let worst = head
        .quant_tables
        .iter()
        .flatten()
        .copied()
        .max()
        .unwrap_or(0);
    assert_eq!(
        worst, 1,
        "AC-1 / decision 4: JPEG is re-encoded at quality 100, whose quantisation \
         tables are all ones; the largest entry here is {worst}, which is a lower \
         quality. Measured on this crop in RED: quality 99 gives 2, quality 95 gives \
         12, and the quality 75 that save_with_format writes by default gives 61"
    );
    assert_eq!(
        head.components
            .iter()
            .map(|&(_, h, v)| (h, v))
            .collect::<Vec<_>>(),
        vec![(1, 1), (1, 1), (1, 1)],
        "decision 4: 4:4:4, no chroma subsampling"
    );
}

// --- AC-2: lossless WebP ----------------------------------------------------

#[test]
fn a_lossless_webp_is_cropped_and_re_encoded_as_a_lossless_webp() {
    let (tmp, out) = workspace();
    let plane = common::screenshot();
    let done = crop_file(
        |path| common::write_webp(&plane, path),
        "shot.webp",
        tmp.path(),
        &out,
    );

    assert_eq!(
        done.rect,
        rect(),
        "AC-2: the reported rect is the detector's"
    );
    assert_eq!(
        done.output,
        out.join("shot.webp"),
        "AC-2: the output is named like the input"
    );
    assert!(
        done.output.is_file(),
        "AC-2: {} must exist",
        done.output.display()
    );
    assert_eq!(
        common::webp_fourcc(&done.output),
        "VP8L",
        "AC-2 / decision 4: WebP is encoded lossless, which the container says in \
         its own first chunk name"
    );
    let decoded = common::decode_as(&done.output, ImageFormat::WebP);
    assert_eq!(
        (decoded.width(), decoded.height()),
        (done.rect.w, done.rect.h),
        "AC-2: the output decodes to rect.w x rect.h"
    );
}

#[test]
fn a_lossless_webp_crop_is_pixel_identical_to_its_source_inside_the_rect() {
    let (tmp, out) = workspace();
    let plane = common::screenshot();
    let done = crop_file(
        |path| common::write_webp(&plane, path),
        "shot.webp",
        tmp.path(),
        &out,
    );

    let src = common::decode_as(&done.input, ImageFormat::WebP);
    let got = common::decode_as(&done.output, ImageFormat::WebP);
    let differing = common::sample_differences(&src, done.rect, &got);
    assert!(
        differing.is_empty(),
        "AC-2: every output pixel must equal the decoded source pixel at \
         (rect.x + i, rect.y + j); first differences inside {:?}: {differing:?}",
        done.rect
    );
}

// --- AC-4: a format the engine does not handle ------------------------------

#[test]
fn a_bmp_is_flagged_unsupported_and_copied_byte_for_byte() {
    let (tmp, out) = workspace();
    let (input, reason, output) = flag_file(
        |path| common::write_bmp(&common::small_gradient(), path),
        "x.bmp",
        tmp.path(),
        &out,
    );

    assert_eq!(
        reason,
        Flag::Unsupported,
        "AC-4: a valid image in a format the engine does not handle is Unsupported - \
         not DecodeFailed, which is for a file that claims a format it cannot honour"
    );
    assert_eq!(
        output,
        out.join("x.bmp"),
        "AC-4: a flagged file is still written under its own name"
    );
    assert_copied(&input, &output, "AC-4");
}

#[test]
fn a_text_file_is_flagged_unsupported_and_copied_byte_for_byte() {
    let (tmp, out) = workspace();
    let (input, reason, output) = flag_file(common::write_text, "x.txt", tmp.path(), &out);

    assert_eq!(
        reason,
        Flag::Unsupported,
        "AC-4: a file that is not an image at all is Unsupported"
    );
    assert_eq!(
        output,
        out.join("x.txt"),
        "AC-4: a flagged file is still written under its own name"
    );
    assert_copied(&input, &output, "AC-4");
}

// --- AC-5: a file that claims a format it cannot honour ---------------------

#[test]
fn two_hundred_random_bytes_named_as_a_png_are_flagged_decode_failed_and_copied() {
    let (tmp, out) = workspace();
    let (input, reason, output) = flag_file(common::write_random_bytes, "x.png", tmp.path(), &out);

    match &reason {
        Flag::DecodeFailed(message) => assert!(
            !message.trim().is_empty(),
            "AC-5: DecodeFailed carries what went wrong, for the run summary; got an \
             empty string"
        ),
        other => panic!(
            "AC-5: {} bytes of junk under a .png name is a DecodeFailed, not {other:?}. \
             Unsupported is for content in a format the engine does not handle; this \
             file claims one it does",
            common::RANDOM_LEN
        ),
    }
    assert_eq!(
        output,
        out.join("x.png"),
        "AC-5: a flagged file is still written under its own name"
    );
    assert_copied(&input, &output, "AC-5");
}

// --- AC-6: the extension lies -----------------------------------------------

#[test]
fn a_png_named_jpg_is_cropped_as_a_png_under_its_own_name() {
    let (tmp, out) = workspace();
    let plane = common::screenshot();
    let done = crop_file(
        |path| common::write_rgb8(&plane, path),
        "x.jpg",
        tmp.path(),
        &out,
    );

    assert_eq!(
        detect_format(&fs::read(&done.input).expect("the fixture")),
        Some(SourceFormat::Png),
        "AC-6: the file is a PNG, whatever its name says"
    );
    assert_eq!(
        done.output,
        out.join("x.jpg"),
        "AC-6: the output is out_dir/x.jpg - the name is not corrected to match the format"
    );
    assert_eq!(done.rect, rect(), "AC-6: the rect is the PNG fixture's");
    // The container, first: an implementation that believed the extension would
    // write a JPEG here, and `FF D8` is not `89 P N G`.
    assert_eq!(
        common::head(&done.output, 8),
        common::PNG_SIGNATURE,
        "AC-6: treated as PNG means encoded as PNG - a JPEG would start FF D8"
    );
    let head = common::ihdr(&done.output);
    assert_eq!(
        (head.width, head.height, head.bit_depth, head.colour_type),
        (done.rect.w, done.rect.h, 8, common::PNG_RGB),
        "AC-6: cropped losslessly at the source's own colour type, like any other PNG"
    );
    // And the pixels, which catch a lossy re-encode even if it somehow kept the
    // PNG container: measured in RED, a quality-100 JPEG of this very crop
    // differs from it by up to 3 levels.
    let src = common::decode_as(&done.input, ImageFormat::Png);
    let got = common::decode_as(&done.output, ImageFormat::Png);
    let differing = common::sample_differences(&src, done.rect, &got);
    assert!(
        differing.is_empty(),
        "AC-6: cropped losslessly - every output pixel equals the source pixel inside \
         {:?}; first differences: {differing:?}",
        done.rect
    );
}

/// The control for AC-6's pixel comparison: "lossless" has to be a claim a
/// lossy implementation would fail.
///
/// Measured in RED: the same crop encoded as a quality-100 JPEG and decoded
/// back differs from the source by up to 3 levels per channel, mean 0.3389. So
/// the assertion above discriminates even against the *best* JPEG this encoder
/// can write, not merely against a careless one.
#[test]
fn a_quality_100_jpeg_of_the_same_crop_is_not_pixel_identical() {
    let (tmp, _out) = workspace();
    let dir = tmp.path();
    let plane = common::screenshot();
    let input = dir.join("x.jpg");
    common::write_rgb8(&plane, &input); // a PNG, under a JPEG's name

    let crop = source_crop(&input, ImageFormat::Png);
    let whole = Rect {
        x: 0,
        y: 0,
        w: crop.width(),
        h: crop.height(),
    };
    let best = dir.join("q100.jpg");
    common::encode_jpeg(&crop, &best, 100);
    let back = common::decode_as(&best, ImageFormat::Jpeg);

    assert!(
        !common::sample_differences(&crop, whole, &back).is_empty(),
        "AC-6's control: a quality-100 JPEG of this crop must NOT be pixel-identical \
         to it, or the lossless assertion in \
         a_png_named_jpg_is_cropped_as_a_png_under_its_own_name would pass for an \
         implementation that believed the .jpg extension"
    );
    let error = common::mean_abs_error(&crop, whole, &back);
    assert!(
        error > 0.0,
        "AC-6's control: measured 0.3389 in RED, got {error:.4}"
    );
}
