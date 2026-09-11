//! Synthetic image files for the engine's tests (MC-008): one screenshot
//! scene, rendered to a real PNG in four colour types, and one uniform image.
//!
//! `crates/core/tests/common/` already has a far richer generator, and this is
//! deliberately **not** it. That module lives in another crate's test tree and
//! pulls in `proptest`; reaching it with `#[path = ...]` would drag a
//! `proptest` dev-dependency into `cropper-engine` for a generator whose
//! recipes, layouts and strategies this story has no use for. MC-008 needs
//! exactly two scenes, so it builds exactly two.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: nothing here. Every threshold is read from
//!   `Tuning::default()` at the point of use; the constants below are the
//!   sizes and colours of a synthetic screenshot and none of them is a
//!   threshold.
//! * **Mechanical**: the PNG container facts - the eight-byte signature, IHDR
//!   at offset 8, bit depth at byte 24 and colour type at byte 25 - which
//!   [`ihdr`] reads straight out of the file rather than asking a decoder what
//!   it made of them.
//! * **Measured**: the rect [`crop_rect`] returns, the rendered flat fraction
//!   of the chrome band, and the file lengths. Every one of them is recorded
//!   in the story's `## Handoff: RED -> GREEN`, and the tests in
//!   `tests/process_file.rs` measure them again so a drifting fixture fails
//!   loudly instead of quietly moving an expectation.
//!
//! # Why the scene is built this way
//!
//! [`screenshot`] is a luma **plane**, and every colour rendering of it is
//! grey: `r = g = b = v`. The BT.601 weights sum to exactly 1.0, so a correct
//! `to_luma` maps a grey pixel back to `v` whatever its container - which is
//! what lets one expected rect serve all four colour types in AC-1 and AC-2.
//! The alpha channel of the RGBA8 rendering is the opposite: a coarse
//! non-uniform pattern ([`alpha_at`]) that a correct `to_luma` ignores
//! entirely. An implementation that blended it into the luma would turn the
//! flat borders into noise, no border would be trimmed and AC-1's rect would
//! be wrong - so "alpha ignored" is pinned at the whole-file level as well as
//! by AC-3's pixels.
//!
//! Cargo compiles this module separately into every test binary in the
//! directory, so a helper used by one target is dead code in the other. That
//! is why `dead_code` is allowed here, exactly as in `crates/core/tests/`.
#![allow(dead_code)]

use std::fs;
use std::path::Path;

use cropper_core::{Luma, Rect, Tuning};
use image::{DynamicImage, ImageBuffer, ImageFormat};

// --- The scene's geometry ---------------------------------------------------
//
// None of these is a threshold. They are the widths, heights and colours of a
// synthetic screenshot: four uniform borders, a chrome band below the top
// border, and textured art inside.

/// Top border depth, in pixels.
pub const TOP: u32 = 10;
/// Right border depth, in pixels.
pub const RIGHT: u32 = 12;
/// Bottom border depth, in pixels.
pub const BOTTOM: u32 = 8;
/// Left border depth, in pixels.
pub const LEFT: u32 = 15;
/// Chrome band depth, in pixels, immediately below the top border.
pub const BAND: u32 = 12;
/// Art width, in pixels. A multiple of [`TEXT_PERIOD`] so the chrome band's
/// flat fraction is exact rather than quantised.
pub const ART_W: u32 = 120;
/// Art height, in pixels.
pub const ART_H: u32 = 90;

/// The four border colours, each different, so no two borders can be confused
/// for one another and a trim that stops early leaves a visible tell.
pub const TOP_COLOUR: u8 = 255;
/// See [`TOP_COLOUR`].
pub const BOTTOM_COLOUR: u8 = 128;
/// See [`TOP_COLOUR`].
pub const LEFT_COLOUR: u8 = 64;
/// See [`TOP_COLOUR`].
pub const RIGHT_COLOUR: u8 = 0;

/// The chrome band's flat background: a light toolbar.
pub const BAND_BG: u8 = 200;
/// How far a chrome band "text" pixel sits from the background, either way.
/// Twice `Tuning::uniform_tolerance`, so a moved pixel is unambiguously not
/// flat, and small enough that a run of them never reaches `edge_threshold` -
/// the same choice `crates/core/tests/common/` makes, for the same reasons.
pub const BAND_DEVIATION: u8 = 20;
/// One pixel in every [`TEXT_PERIOD`] along a band row is a "text" pixel, so
/// the band's flat fraction is exactly `1 - 1 / TEXT_PERIOD` = 0.9.
pub const TEXT_PERIOD: u32 = 10;

/// The art is a checkerboard of a dark and a light tone, each jittered, so
/// every art row and column spans at least `ART_HI - (ART_LO + ART_JITTER)`
/// = 140 - far past any tolerance in `Tuning` - and no art line can ever be
/// trimmed or mistaken for chrome.
pub const ART_LO: u8 = 40;
/// See [`ART_LO`].
pub const ART_HI: u8 = 200;
/// See [`ART_LO`].
pub const ART_JITTER: u8 = 20;

/// Seed for the art texture. Fixed: every expectation in these tests is a
/// rect or a byte comparison, never a pixel value.
pub const SEED: u32 = 7;

/// The uniform image's size and single luma value.
pub const UNIFORM_W: u32 = 60;
/// See [`UNIFORM_W`].
pub const UNIFORM_H: u32 = 40;
/// See [`UNIFORM_W`].
pub const UNIFORM_VALUE: u8 = 120;

/// PNG colour type 0: greyscale.
pub const PNG_GREY: u8 = 0;
/// PNG colour type 2: truecolour.
pub const PNG_RGB: u8 = 2;
/// PNG colour type 6: truecolour with alpha.
pub const PNG_RGBA: u8 = 6;

/// Rendered image width.
#[must_use]
pub const fn width() -> u32 {
    LEFT + ART_W + RIGHT
}

/// Rendered image height.
#[must_use]
pub const fn height() -> u32 {
    TOP + BAND + ART_H + BOTTOM
}

/// The art rect in image coordinates: inside the borders, below the band.
#[must_use]
pub const fn art_rect() -> Rect {
    Rect {
        x: LEFT,
        y: TOP + BAND,
        w: ART_W,
        h: ART_H,
    }
}

/// The rect a correct `process_file` must crop [`screenshot`] to: the art,
/// expanded by `Tuning::margin_px` on every side and clamped to the image.
///
/// Derived from [`art_rect`] and the tuning rather than written as literals,
/// so a change to `margin_px` moves the expectation instead of breaking it.
/// The value it takes at `Tuning::default()` is measured in
/// `tests/process_file.rs` and recorded in the story's handoff.
#[must_use]
pub fn crop_rect() -> Rect {
    let margin = Tuning::default().margin_px;
    let art = art_rect();
    let x = art.x.saturating_sub(margin);
    let y = art.y.saturating_sub(margin);
    let right = (art.x + art.w + margin).min(width());
    let bottom = (art.y + art.h + margin).min(height());
    Rect {
        x,
        y,
        w: right - x,
        h: bottom - y,
    }
}

// --- Seeded randomness ------------------------------------------------------

/// The 32-bit xorshift `crates/core/tests/common/` uses, so the art texture is
/// deterministic on every machine without a dependency.
pub struct Rng(u32);

impl Rng {
    /// Seed 0 is a fixed point of xorshift, so it is replaced.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self(if seed == 0 { 0x2545_F491 } else { seed })
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    /// A value in `0 ..= hi`.
    pub fn upto(&mut self, hi: u8) -> u8 {
        (self.next_u32() % (u32::from(hi) + 1)) as u8
    }
}

// --- The scenes -------------------------------------------------------------

/// The screenshot scene as a luma plane: four uniform borders, a chrome band
/// below the top border spanning the width between the side borders, and
/// checkerboard art inside.
///
/// The top and bottom borders span the full width and the side borders only
/// the rows between them, so the *first* pass of the uniform trim can only
/// reach the top and bottom; the sides become uniform only once those are
/// gone. That is the iteration MC-003 AC-5 pins, and it is what makes this a
/// screenshot rather than a frame.
#[must_use]
pub fn screenshot() -> Luma {
    let w = width();
    let h = height();
    let mut data = vec![0u8; w as usize * h as usize];
    let put = |data: &mut Vec<u8>, x: u32, y: u32, v: u8| {
        data[y as usize * w as usize + x as usize] = v;
    };

    // Art: a jittered checkerboard.
    let mut rng = Rng::new(SEED);
    let art = art_rect();
    for y in art.y..art.y + art.h {
        for x in art.x..art.x + art.w {
            let base = if (x + y).is_multiple_of(2) {
                ART_LO
            } else {
                ART_HI
            };
            put(&mut data, x, y, base + rng.upto(ART_JITTER));
        }
    }

    // Chrome band: a flat background with `1 / TEXT_PERIOD` of each row moved
    // off it, alternately up and down so the band's median stays exactly on
    // the background. The pattern shifts by three columns per row, so no two
    // adjacent rows deviate in the same places and the band carries no strong
    // line of its own.
    for row in 0..BAND {
        let y = TOP + row;
        let mut moved = 0u32;
        for i in 0..ART_W {
            let x = LEFT + i;
            let v = if (i + 3 * row).is_multiple_of(TEXT_PERIOD) {
                let up = moved.is_multiple_of(2);
                moved += 1;
                if up {
                    BAND_BG + BAND_DEVIATION
                } else {
                    BAND_BG - BAND_DEVIATION
                }
            } else {
                BAND_BG
            };
            put(&mut data, x, y, v);
        }
    }

    // Borders. Top and bottom span the full width; the sides fill only the
    // rows between them.
    for y in 0..TOP {
        for x in 0..w {
            put(&mut data, x, y, TOP_COLOUR);
        }
    }
    for y in h - BOTTOM..h {
        for x in 0..w {
            put(&mut data, x, y, BOTTOM_COLOUR);
        }
    }
    for y in TOP..h - BOTTOM {
        for x in 0..LEFT {
            put(&mut data, x, y, LEFT_COLOUR);
        }
        for x in w - RIGHT..w {
            put(&mut data, x, y, RIGHT_COLOUR);
        }
    }

    Luma {
        width: w,
        height: h,
        data,
    }
}

/// The uniform scene: one flat colour, which is what the detector flags
/// `Uniform`.
#[must_use]
pub fn uniform() -> Luma {
    Luma {
        width: UNIFORM_W,
        height: UNIFORM_H,
        data: vec![UNIFORM_VALUE; (UNIFORM_W * UNIFORM_H) as usize],
    }
}

/// The alpha channel of the RGBA8 rendering: coarse, non-uniform, and nowhere
/// near opaque. A correct `to_luma` ignores it; anything that blends it in
/// turns the flat borders into noise and gets a different rect.
#[must_use]
pub const fn alpha_at(x: u32, y: u32) -> u8 {
    (255 - (x * 13 + y * 7) % 200) as u8
}

// --- Writing the scenes out as real files -----------------------------------

/// One of the four fixture writers below: a luma plane and a path in, a PNG
/// file of one colour type out. Named because a table of `(name, writer,
/// depth, colour type)` rows is `clippy::type_complexity` without it.
pub type Writer = fn(&Luma, &Path);

/// `plane` as an 8-bit greyscale PNG at `path`. PNG colour type 0, bit depth 8.
pub fn write_grey8(plane: &Luma, path: &Path) {
    let buf =
        ImageBuffer::<image::Luma<u8>, _>::from_raw(plane.width, plane.height, plane.data.clone())
            .expect("the plane holds width * height samples");
    save(&DynamicImage::ImageLuma8(buf), path);
}

/// `plane` as an 8-bit RGB PNG at `path`, grey: `r = g = b = v`. PNG colour
/// type 2, bit depth 8.
pub fn write_rgb8(plane: &Luma, path: &Path) {
    let mut data = Vec::with_capacity(plane.data.len() * 3);
    for &v in &plane.data {
        data.extend_from_slice(&[v, v, v]);
    }
    let buf = ImageBuffer::<image::Rgb<u8>, _>::from_raw(plane.width, plane.height, data)
        .expect("three samples per pixel");
    save(&DynamicImage::ImageRgb8(buf), path);
}

/// `plane` as a 16-bit RGB PNG at `path`, grey, each 8-bit sample `v` widened
/// to `v * 257` - the exact scaling of 0..=255 onto 0..=65535, so the source
/// luma survives the widening whichever way an implementation narrows it back.
/// PNG colour type 2, bit depth 16.
pub fn write_rgb16(plane: &Luma, path: &Path) {
    let mut data = Vec::with_capacity(plane.data.len() * 3);
    for &v in &plane.data {
        let wide = u16::from(v) * 257;
        data.extend_from_slice(&[wide, wide, wide]);
    }
    let buf = ImageBuffer::<image::Rgb<u16>, _>::from_raw(plane.width, plane.height, data)
        .expect("three samples per pixel");
    save(&DynamicImage::ImageRgb16(buf), path);
}

/// `plane` as an 8-bit RGBA PNG at `path`, grey, with [`alpha_at`] in the
/// alpha channel. PNG colour type 6, bit depth 8.
pub fn write_rgba8(plane: &Luma, path: &Path) {
    let mut data = Vec::with_capacity(plane.data.len() * 4);
    for (i, &v) in plane.data.iter().enumerate() {
        let x = i as u32 % plane.width;
        let y = i as u32 / plane.width;
        data.extend_from_slice(&[v, v, v, alpha_at(x, y)]);
    }
    let buf = ImageBuffer::<image::Rgba<u8>, _>::from_raw(plane.width, plane.height, data)
        .expect("four samples per pixel");
    save(&DynamicImage::ImageRgba8(buf), path);
}

fn save(img: &DynamicImage, path: &Path) {
    img.save_with_format(path, ImageFormat::Png)
        .unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
}

// --- The marked uniform PNG -------------------------------------------------

/// The text a [`write_marked_uniform_png`] file carries in a `tEXt` chunk.
pub const MARKER: &[u8] = b"MC-008 fixture: a copy keeps this chunk; a re-encode drops it";

/// The uniform scene as an 8-bit greyscale PNG carrying a `tEXt` chunk between
/// IHDR and the image data.
///
/// The chunk is what gives AC-5's "the output bytes equal the input's" any
/// force. Without it the input is a PNG this crate's own encoder produced from
/// a flat image, and an implementation that decoded it and encoded it again
/// could plausibly reproduce it byte for byte - the assertion would pass while
/// testing nothing. Metadata chunks are explicitly not preserved by the
/// encoder (the story's `## Out of scope`), so a re-encode cannot carry this
/// one and only a real copy can.
///
/// `tests/process_file.rs` measures both halves: that the marker is in the
/// file, and that decoding and re-encoding the file does *not* reproduce it.
pub fn write_marked_uniform_png(plane: &Luma, path: &Path) {
    let plain = path.with_extension("plain.png");
    write_grey8(plane, &plain);
    let bytes = fs::read(&plain).expect("the plain PNG was just written");
    fs::remove_file(&plain).expect("the plain PNG is ours to remove");

    // IHDR is always the first chunk: 8 bytes of signature, then 4 length + 4
    // type + 13 data + 4 CRC. The new chunk goes straight after it.
    const IHDR_END: usize = 8 + 4 + 4 + 13 + 4;
    let mut out = Vec::with_capacity(bytes.len() + MARKER.len() + 24);
    out.extend_from_slice(&bytes[..IHDR_END]);
    out.extend_from_slice(&text_chunk(b"Comment", MARKER));
    out.extend_from_slice(&bytes[IHDR_END..]);
    fs::write(path, out).unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
}

/// A PNG `tEXt` chunk: length, type, `keyword\0text`, CRC-32 of type and data.
fn text_chunk(keyword: &[u8], text: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(keyword.len() + 1 + text.len());
    payload.extend_from_slice(keyword);
    payload.push(0);
    payload.extend_from_slice(text);

    let mut typed = b"tEXt".to_vec();
    typed.extend_from_slice(&payload);

    let mut chunk = (payload.len() as u32).to_be_bytes().to_vec();
    chunk.extend_from_slice(&typed);
    chunk.extend_from_slice(&crc32(&typed).to_be_bytes());
    chunk
}

/// The CRC-32 PNG specifies, computed bitwise: a table would be faster and
/// this runs over a few dozen bytes, twice.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

// --- Reading files back -----------------------------------------------------

/// A PNG's IHDR header, read out of the file's own bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ihdr {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bits per sample: 8 or 16 here.
    pub bit_depth: u8,
    /// PNG colour type: [`PNG_GREY`], [`PNG_RGB`] or [`PNG_RGBA`].
    pub colour_type: u8,
}

/// The IHDR of the PNG at `path`, read from the container rather than from a
/// decoder's interpretation of it.
///
/// AC-2 is about what was *written*: "the same colour type and bit depth as
/// its input". Decoding and matching on a `DynamicImage` variant asks the
/// decoder what it made of the file; these two bytes are the file's own answer,
/// and they are at fixed offsets in every PNG ever written.
///
/// # Panics
///
/// If `path` cannot be read, is not a PNG, or does not open with IHDR.
#[must_use]
pub fn ihdr(path: &Path) -> Ihdr {
    let bytes = fs::read(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    assert!(
        bytes.len() > 26,
        "{} is too short to be a PNG ({} bytes)",
        path.display(),
        bytes.len()
    );
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        "{} does not start with the PNG signature",
        path.display()
    );
    assert_eq!(
        &bytes[12..16],
        b"IHDR",
        "{}'s first chunk is not IHDR",
        path.display()
    );
    Ihdr {
        width: u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        height: u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
        bit_depth: bytes[24],
        colour_type: bytes[25],
    }
}

/// The raw samples of `rect`, row by row, in the image's own colour type and
/// bit depth - no conversion, no normalisation.
///
/// This is what "pixel-identical, channel for channel" means for AC-1 and
/// AC-2: the bytes the decoder produced for the source rect against the bytes
/// it produced for the whole output.
///
/// # Panics
///
/// If `rect` reaches outside `img`.
#[must_use]
pub fn crop_bytes(img: &DynamicImage, rect: Rect) -> Vec<u8> {
    assert!(
        rect.x + rect.w <= img.width() && rect.y + rect.h <= img.height(),
        "{rect:?} reaches outside a {}x{} image",
        img.width(),
        img.height()
    );
    let bpp = usize::from(img.color().bytes_per_pixel());
    let stride = img.width() as usize * bpp;
    let bytes = img.as_bytes();
    let mut out = Vec::with_capacity(rect.w as usize * rect.h as usize * bpp);
    for dy in 0..rect.h as usize {
        let start = (rect.y as usize + dy) * stride + rect.x as usize * bpp;
        out.extend_from_slice(&bytes[start..start + rect.w as usize * bpp]);
    }
    out
}

/// Decode the PNG at `path`, or panic naming it.
///
/// # Panics
///
/// If the file cannot be read or decoded.
#[must_use]
pub fn decode(path: &Path) -> DynamicImage {
    image::open(path).unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()))
}
