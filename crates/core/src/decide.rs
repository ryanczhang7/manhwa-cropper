//! The pipeline, composed: pixels in, one rectangle out
//! (`docs/wiki/architecture.md`, "cropper-core").
//!
//! Every stage before this one answers a narrow question about an image -
//! which borders are one flat colour, where the strong lines are, which edge
//! strips look like browser chrome, how much air the answer needs. [`detect`]
//! is the only place they are put in order, and the order is the whole of the
//! algorithm:
//!
//! 1. [`trim_uniform`](crate::trim::trim_uniform) over the whole image. If it
//!    finds nothing there is no art, and `detect` returns `None` - the image
//!    was one flat colour and there is nothing to crop (MC-006 AC-5);
//! 2. [`content_box`] inside what survived, which peels chrome strips off the
//!    edges one per side per pass. Its `removed` and `ambiguous` are carried
//!    into [`Detection`] untouched: MC-007 turns them into flags, and nothing
//!    here reinterprets them;
//! 3. [`trim_within`] again, **inside the content box**. Peeling a toolbar off
//!    the top of a screenshot routinely exposes a page gutter that was not an
//!    edge of the image a moment ago, and the first trim has no way to see it.
//!    MC-006 AC-2 is that scene exactly, and on it the first trim moves
//!    nothing while the second moves both side edges by 70 px;
//! 4. [`margin::expand`] by [`Tuning::margin_px`], clamped to the image.
//!
//! MC-007 adds the other half of this module - the flag reasons and
//! `CropDecision` - on top of the same composition. `Detection` is what it
//! will be given.
//!
//! # What `trimmed` means
//!
//! **Either uniform trim moved an edge.** The story names the field and not
//! its reading, and MC-006's RED pinned this one, with
//! `trimmed_is_true_when_only_the_second_trim_moved_an_edge` as the
//! discriminator: on AC-2's scene the first trim does nothing and `trimmed`
//! must still be true, because a gutter trimmed inside the content box is
//! still a uniform border that was trimmed. The alternative reading - the
//! first trim only - fails exactly that test and nothing else.
//!
//! The field is mechanical rather than statistical, so it is compared rather
//! than tracked: the first trim moved something iff it returned a rect other
//! than the whole image, and the second iff it returned a rect other than the
//! content box it started from.

use crate::content::{Side, content_box};
use crate::margin;
use crate::trim::{trim_uniform, trim_within};
use crate::{Dimensions, Luma, Rect, Tuning};

/// What [`detect`] found: the rect to crop to, and what the stages did on the
/// way there.
///
/// The three report fields exist for MC-007, which turns them into the reasons
/// a crop is flagged for review. Nothing in this crate reads them.
#[derive(Debug, Clone)]
pub struct Detection {
    /// The crop: the art, plus [`Tuning::margin_px`] on every side that had
    /// room for it.
    pub rect: Rect,
    /// Whether either uniform trim moved an edge - see the module note above,
    /// which is the definition and not a description of one.
    pub trimmed: bool,
    /// The sides chrome strips were peeled from, in the order they were
    /// peeled, straight out of
    /// [`ContentBox::removed`](crate::content::ContentBox::removed).
    pub removed: Vec<Side>,
    /// Whether some edge strip was *nearly* chrome-like, straight out of
    /// [`ContentBox::ambiguous`](crate::content::ContentBox::ambiguous).
    pub ambiguous: bool,
}

/// The rect to crop `img` to, or `None` when there is nothing to crop.
///
/// `None` means the image is wholly uniform: both trims drain it to nothing,
/// and there is no art to find at any margin. Every other image yields a rect,
/// even one that is all art - in which case the margin clamps on all four
/// sides and the answer is the image itself.
///
/// The five steps are in the module documentation above.
#[must_use]
pub fn detect(img: &Luma, t: &Tuning) -> Option<Detection> {
    let whole = Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    };

    let first = trim_uniform(img, t)?;
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t)?;

    Some(Detection {
        rect: margin::expand(
            second,
            t.margin_px,
            Dimensions {
                width: img.width,
                height: img.height,
            },
        ),
        trimmed: first != whole || second != found.rect,
        removed: found.removed,
        ambiguous: found.ambiguous,
    })
}
