//! MC-008, AC-3: `codec::to_luma` converts decoded pixels to the 8-bit luma
//! plane `cropper-core` detects on, with the BT.601 weights and alpha ignored.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: the weights and the rounding. `0.299 R + 0.587 G + 0.114 B`
//!   (`docs/wiki/architecture.md`, "cropper-core"), so `(255, 0, 0)` is `76`,
//!   `(0, 255, 0)` is `150`, `(0, 0, 255)` is `29` and white is `255`. These
//!   numbers are read out of AC-3, not derived here and not tuned here.
//! * **Mechanical**: `to_luma(&DynamicImage) -> Luma` and
//!   `SourceFormat { Png, Jpeg, WebP }`. Pinned exactly.
//! * **Measured**: one thing, and it is why AC-3 is worth a test at all. The
//!   `image` crate's own `to_luma8` uses the **BT.709** weights, not BT.601:
//!   measured in RED, the same four pixels come back `[54, 182, 18, 255]`
//!   through it. An implementation that delegates to the decoder therefore
//!   fails this file, and only this file - see the story's handoff, where the
//!   grey fixtures the rest of the suite uses are shown to give the identical
//!   plane under either set of weights.
//!
//! # What is deliberately not constrained
//!
//! Nothing here says how `to_luma` narrows a 16-bit sample, only that a grey
//! `v * 257` comes back as `v` - which is true of `>> 8`, of `/ 257` and of a
//! normalised float alike. Nor does anything here pin a `SourceFormat`
//! constructor: MC-008 only needs the type to exist with these three
//! variants, and detecting a format from a decoded container is MC-009's.

mod common;

use cropper_engine::SourceFormat;
use cropper_engine::codec::to_luma;
use image::{DynamicImage, ImageBuffer};

/// An `n x 1` RGB8 image from `n` triples.
fn rgb8_strip(pixels: &[[u8; 3]]) -> DynamicImage {
    let data: Vec<u8> = pixels.iter().flatten().copied().collect();
    DynamicImage::ImageRgb8(
        ImageBuffer::from_raw(pixels.len() as u32, 1, data).expect("three samples per pixel"),
    )
}

// --- AC-3 as written --------------------------------------------------------

#[test]
fn to_luma_weights_the_channels_with_the_bt601_coefficients() {
    let strip = rgb8_strip(&[[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]]);
    let luma = to_luma(&strip);

    assert_eq!(
        (luma.width, luma.height),
        (4, 1),
        "AC-3: the plane keeps the image's dimensions"
    );
    assert_eq!(
        luma.data,
        vec![76u8, 150, 29, 255],
        "AC-3: pure red, green, blue and white are 76, 150, 29 and 255 under \
         0.299 R + 0.587 G + 0.114 B, rounded"
    );
    // The control that makes those four numbers mean something: they are not
    // what the decoder hands out for free. `image` 0.25's own conversion is
    // BT.709 and gives [54, 182, 18, 255] for the same pixels.
    assert_ne!(
        luma.data,
        strip.to_luma8().into_raw(),
        "AC-3's weights are BT.601; the image crate's to_luma8 is BT.709, and \
         delegating to it is not the conversion this product specified"
    );
}

#[test]
fn to_luma_ignores_the_alpha_channel() {
    let pixel = DynamicImage::ImageRgba8(
        ImageBuffer::from_raw(1, 1, vec![255u8, 255, 255, 0]).expect("one RGBA pixel"),
    );
    assert_eq!(
        to_luma(&pixel).data,
        vec![255u8],
        "AC-3: white at zero alpha is still 255 - alpha is ignored, not blended"
    );
}

// --- The preconditions AC-1 and AC-2 rest on --------------------------------

/// Every fixture in `tests/process_file.rs` is grey - `r = g = b = v` - and
/// every expectation there assumes the luma plane is the grey plane it was
/// rendered from. That holds because the BT.601 weights sum to exactly 1.0,
/// and this is where it is pinned rather than assumed.
#[test]
fn a_grey_pixel_converts_to_its_own_value() {
    let greys = [0u8, 1, 40, 64, 120, 128, 200, 254, 255];
    let strip = rgb8_strip(&greys.map(|v| [v, v, v]));
    assert_eq!(
        to_luma(&strip).data,
        greys.to_vec(),
        "the weights sum to 1.0, so a grey pixel is its own luma"
    );
}

/// AC-2's 16-bit case reaches the detector through the same plane as the 8-bit
/// ones, so a 16-bit grey sample has to narrow back to the 8-bit value it was
/// widened from. `v * 257` is the exact scaling of `0 ..= 255` onto
/// `0 ..= 65535`, which is what `common::write_rgb16` writes.
#[test]
fn a_sixteen_bit_grey_pixel_narrows_to_its_eight_bit_value() {
    let greys = [0u8, 40, 76, 128, 200, 255];
    let data: Vec<u16> = greys
        .iter()
        .flat_map(|&v| [u16::from(v) * 257; 3])
        .collect();
    let strip = DynamicImage::ImageRgb16(
        ImageBuffer::from_raw(greys.len() as u32, 1, data).expect("three samples per pixel"),
    );
    assert_eq!(
        to_luma(&strip).data,
        greys.to_vec(),
        "a 16-bit grey sample of v * 257 is the 8-bit luma v"
    );
}

/// `to_luma` sees whole planes, not single pixels: the fixture's own
/// rendering, converted, must be the plane it was rendered from. This is the
/// assumption `tests/process_file.rs` makes about all four colour types, made
/// once, over every one of the fixture's pixels rather than a chosen few.
#[test]
fn converting_the_fixtures_own_renderings_gives_the_plane_they_were_rendered_from() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let plane = common::screenshot();
    let writers: [(&str, common::Writer); 4] = [
        ("grey.png", common::write_grey8),
        ("rgb8.png", common::write_rgb8),
        ("rgb16.png", common::write_rgb16),
        ("rgba8.png", common::write_rgba8),
    ];
    let mut wrong = Vec::new();
    for (name, writer) in writers {
        let path = tmp.path().join(name);
        writer(&plane, &path);
        let got = to_luma(&common::decode(&path));
        if (got.width, got.height) != (plane.width, plane.height) {
            wrong.push(format!(
                "{name}: {}x{} instead of {}x{}",
                got.width, got.height, plane.width, plane.height
            ));
        } else if got.data != plane.data {
            let first: Vec<String> = got
                .data
                .iter()
                .zip(&plane.data)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .take(3)
                .map(|(i, (a, b))| {
                    format!(
                        "({}, {}): {a} instead of {b}",
                        i as u32 % plane.width,
                        i as u32 / plane.width
                    )
                })
                .collect();
            wrong.push(format!("{name}: {first:?}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "every colour rendering of the fixture is grey, so every one of them must \
         convert back to the same plane: {wrong:?}"
    );
}

// --- The pinned shape -------------------------------------------------------

#[test]
fn a_source_format_is_png_jpeg_or_webp() {
    let all = [SourceFormat::Png, SourceFormat::Jpeg, SourceFormat::WebP];
    assert_eq!(
        format!("{all:?}"),
        "[Png, Jpeg, WebP]",
        "the mechanical partition: these three variants, these names"
    );
    assert_ne!(all[0], all[1], "the variants are distinguishable");
    assert_eq!(all[2], all[2].clone(), "a SourceFormat is Copy or Clone");
}
