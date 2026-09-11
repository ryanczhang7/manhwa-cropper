//! Decoded pixels in, the plane `cropper-core` detects on out (MC-008).
//!
//! The detector never sees a colour image: every stage in `cropper-core` works
//! on an 8-bit [`Luma`] plane (`docs/wiki/architecture.md`, "cropper-core"), so
//! exactly one function stands between a decoded file and the detector, and it
//! is [`to_luma`].
//!
//! # Why the weights are written out here
//!
//! `image` has its own conversion - [`DynamicImage::to_luma8`] - and it is the
//! wrong one for this product. It uses the Rec. 709 coefficients
//! (`image-0.25.10/src/color.rs`: `SRGB_LUMA = [2126, 7152, 722] / 10000`),
//! which turn pure green into 182. This product specified BT.601
//! (`0.299 R + 0.587 G + 0.114 B`, MC-008 AC-3), which makes it 150. The two
//! agree on every grey and disagree on everything else, so delegating would
//! look right on any greyscale fixture and be wrong on every screenshot.
//!
//! # Why the arithmetic is integer, and rounded
//!
//! `(2990 R + 5870 G + 1140 B + 5000) / 10000` is the same expression as the
//! float one with the `+ 5000` doing the rounding, and it has no rounding mode
//! of its own to get wrong. Rounding is not cosmetic: pure green is exactly
//! 149.685, so truncating gives 149 where AC-3 requires 150. The weights sum
//! to exactly 10000, which is what makes a grey pixel its own luma - and the
//! whole of MC-008's fixture work rests on that, since every colour rendering
//! of the test scene is grey and must reach the detector as the same plane.
//!
//! # Why it goes through `to_rgb8` first
//!
//! One conversion, not nine. `image` decodes into any of Luma, LumaA, Rgb or
//! Rgba at 8 or 16 bits (and 32-bit float), and matching on each variant would
//! be eight arms of the same arithmetic plus a ninth this crate would have to
//! guess at. `to_rgb8` is the crate's own narrowing - alpha dropped rather
//! than blended, and 16-bit samples scaled by `round(c * 255 / 65535)`, which
//! is exact for the `v * 257` widening AC-2's fixture uses - after which there
//! is one shape left to weight. It costs a temporary three bytes per pixel;
//! the detector is about to walk the plane several times over, so the copy is
//! not where a screenshot's time goes.
//!
//! # Alpha is ignored, not blended
//!
//! AC-3: white at zero alpha is 255. A transparent pixel is still the colour
//! it is, and blending it against an assumed background would turn a
//! screenshot's flat borders into noise the trim cannot see.

use cropper_core::Luma;
use image::DynamicImage;

/// The image formats the engine reads.
///
/// MC-008 only decodes and encodes PNG; the type is here because the outcome
/// of a run names the format it came from, and MC-009 - which adds JPEG and
/// WebP, and the detection of a format from the file rather than its
/// extension - needs the vocabulary to already exist. Note for MC-009:
/// `image` 0.25 *decodes* WebP but does not encode it, so this variant is not
/// a promise that a WebP can be written back out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    /// Portable Network Graphics.
    Png,
    /// JPEG.
    Jpeg,
    /// WebP.
    WebP,
}

/// The BT.601 luma plane of `img`: `0.299 R + 0.587 G + 0.114 B`, rounded,
/// with any alpha channel ignored.
///
/// Every colour type `image` can decode is accepted. Channels wider than eight
/// bits are narrowed first, by `image`'s own exact scaling of `0 ..= 65535`
/// onto `0 ..= 255`, so a 16-bit sample of `v * 257` comes back as `v`
/// (MC-008 AC-2's 16-bit fixture is written that way).
#[must_use]
pub fn to_luma(img: &DynamicImage) -> Luma {
    let rgb = img.to_rgb8();
    let data = rgb
        .pixels()
        .map(|p| {
            let [r, g, b] = p.0.map(u32::from);
            ((2990 * r + 5870 * g + 1140 * b + 5000) / 10_000) as u8
        })
        .collect();
    Luma {
        width: rgb.width(),
        height: rgb.height(),
        data,
    }
}
