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
//! 4. [`textured_box`] over what is left, which pulls the rect in to the
//!    outermost textured line on whichever axis the gradient locator found no
//!    strong line at all (MC-025). Everything about *when* that happens, and
//!    why it stands outside [`content_box`] rather than inside it, is in
//!    [`flat`](crate::flat)'s module documentation;
//! 5. [`page_column`] over what is left, on the **column axis only**, which
//!    narrows the rect to the page column between its two flat page margins
//!    (MC-027). It has no `strong_lines` guard - a reader page has panel
//!    borders, so on the column axis that guard is unconditional and stage 4
//!    never speaks there - and a rule of its own instead: it narrows only
//!    where the widest textured run has a flat column on both sides of it.
//!    [`flat`](crate::flat)'s "Stage 3c" section is where that is argued. The
//!    row axis stays exactly where stage 4 left it;
//! 6. [`margin::expand`] by [`Tuning::margin_px`], clamped to the image.
//!
//! [`decide`] is the other half of this module (MC-007): the same pixels and
//! the same tuning in, and the one answer the engine acts on out - crop to
//! this rect, or copy the file unchanged and flag it with one of four discrete
//! reasons ([`FlagReason`]). It adds no heuristic of its own. Every input to
//! the decision is a field [`Detection`] already carries, so the whole of it
//! is four questions asked in a fixed order.
//!
//! # The order is the contract
//!
//! MC-007 AC-6 fixes it: [`Uniform`](FlagReason::Uniform),
//! [`NoBorderFound`](FlagReason::NoBorderFound),
//! [`Ambiguous`](FlagReason::Ambiguous), then
//! [`LowContent`](FlagReason::LowContent). It matters wherever two of them
//! hold at once, and two such scenes are real:
//!
//! * a screenshot with no border, nothing peeled and a top strip that is
//!   *nearly* chrome satisfies both `NoBorderFound` and `Ambiguous`. The order
//!   answers `NoBorderFound`, so the near miss on the strip is not reported.
//!   That is AC-6 as written and it is deliberate rather than incidental; if
//!   the product owner ever wants the close call to win there, it is the two
//!   `if`s below, swapped, and MC-007's story records the question against
//!   MC-015, where the words for each reason are chosen;
//! * a rect that is both ambiguous and too small: `Ambiguous` wins, which is
//!   the pair AC-6 itself names.
//!
//! The remaining pair, `NoBorderFound` against `LowContent`, cannot arise: if
//! nothing was trimmed and nothing was peeled then the rect is the whole
//! image, whose area is all of the image's and whose sides are the image's
//! own.
//!
//! # Arithmetic
//!
//! The area share is computed in `f32`, for the same reason
//! [`content`](crate::content)'s two shares are and with the same consequence:
//! [`Tuning::min_content_fraction`] is an `f32`, and AC-3 flags a rect whose
//! area is *below* it, so a rect sitting exactly on the limit must be cropped.
//! `8000f32 / 160000f32` is bit for bit `0.05f32` and `<` is false;
//! `f64::from(8000) / f64::from(160000)` is 0.05000000000000000278, which
//! sits below `f64::from(0.05f32)` = 0.05000000074505805969, and the boundary
//! moves without anyone touching a constant. MC-007 pins that boundary with a
//! fixture measured in whole pixels; MC-026 rebuilt that fixture at 8000 px
//! of 160000 when it re-settled
//! [`min_content_fraction`](Tuning::min_content_fraction) from 0.20 to 0.05,
//! and 0.05 was chosen partly because it keeps the fixture exact.
//!
//! # What `trimmed` means
//!
//! **A blank border was removed from some edge**: either uniform trim moved
//! an edge, or - since MC-025 - the flat-border stage did. The story names
//! the field and not its reading, and MC-006's RED pinned this one, with
//! `trimmed_is_true_when_only_the_second_trim_moved_an_edge` as the
//! discriminator: on AC-2's scene the first trim does nothing and `trimmed`
//! must still be true, because a gutter trimmed inside the content box is
//! still a uniform border that was trimmed. The alternative reading - the
//! first trim only - fails exactly that test and nothing else.
//!
//! MC-025 extends the reading rather than changing it. A gutter the flat
//! stage pulls the rect in past is a blank border that was removed, told apart
//! from the art by a different statistic; reading it as anything else would
//! make [`decide`] answer [`NoBorderFound`](FlagReason::NoBorderFound) - "the
//! screenshot is all art and the rect is the whole image" - for a page where
//! neither half of that sentence is true.
//!
//! MC-027 extends it once more, and for the identical reason: a page margin
//! the column locator pulls the rect in past is a blank border that was
//! removed. Without that clause the fixture in
//! `tests/decide.rs::a_page_column_between_flat_page_margins_is_cropped_to_the_page`
//! reaches [`decide`] with `!trimmed && removed.is_empty()` and is flagged
//! `NoBorderFound` however well the page was located.
//!
//! The field is mechanical rather than statistical, so it is compared rather
//! than tracked: each stage moved something iff it returned a rect other than
//! the one it was given.

use crate::content::{Side, content_box};
use crate::flat::{page_column, textured_box};
use crate::margin;
use crate::trim::{trim_uniform, trim_within};
use crate::{Dimensions, Luma, Rect, Tuning};

/// What [`detect`] found: the rect to crop to, and what the stages did on the
/// way there.
///
/// The three report fields are what [`decide`] turns into the reasons a crop
/// is flagged for review; nothing else in this crate reads them.
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
/// The six steps are in the module documentation above.
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
    let textured = textured_box(img, second, t);
    let column = page_column(img, textured, t);

    Some(Detection {
        rect: margin::expand(
            column,
            t.margin_px,
            Dimensions {
                width: img.width,
                height: img.height,
            },
        ),
        trimmed: first != whole || second != found.rect || textured != second || column != textured,
        removed: found.removed,
        ambiguous: found.ambiguous,
    })
}

/// What to do with one image: crop it, or leave it alone and say why
/// (`docs/wiki/architecture.md`, "Data model", and decision 3).
///
/// Deliberately two variants and no third: there is no numeric confidence
/// score anywhere in v1 (decision 3), so a caller never has to decide what a
/// number means. `Serialize` because MC-011's run summary carries this,
/// externally tagged by the plain derive.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum CropDecision {
    /// Crop to this rect: exactly the rect [`detect`] returned, margin and
    /// all, never a rect recomputed here.
    Crop(Rect),
    /// Do not crop. The engine copies the file unchanged and reports this
    /// reason (MC-009/MC-011).
    Flag(FlagReason),
}

/// Why no confident crop was found.
///
/// Four discrete reasons and no free text: the words a person reads for each
/// of them are a design decision (`docs/wiki/design/voice.md`, MC-015) and
/// they do not belong in the value the engine serialises.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum FlagReason {
    /// The whole image is one flat colour, within
    /// [`Tuning::uniform_tolerance`]: [`detect`] gave up and there is nothing
    /// to crop at any margin.
    Uniform,
    /// Nothing was trimmed and no strip was peeled, so the screenshot is all
    /// art and the rect [`detect`] returned is the whole image. Cropping to it
    /// would be a no-op dressed up as a decision.
    NoBorderFound,
    /// The rect is smaller than [`Tuning::min_content_fraction`] of the image,
    /// or one of its sides is shorter than [`Tuning::min_content_side`]. Too
    /// little survived to be a page.
    LowContent,
    /// Some edge strip was *nearly* chrome-like, so the rect turns on a close
    /// call the detector is not confident about.
    Ambiguous,
}

/// The decision for `img`: the rect to crop to, or the reason not to.
///
/// A chain of four questions over one [`Detection`], asked in MC-007 AC-6's
/// order - see the module documentation, which is where that order and its
/// consequences are argued. Nothing here looks at a pixel: [`detect`] has
/// already answered everything this needs.
#[must_use]
pub fn decide(img: &Luma, t: &Tuning) -> CropDecision {
    let Some(found) = detect(img, t) else {
        return CropDecision::Flag(FlagReason::Uniform);
    };
    if !found.trimmed && found.removed.is_empty() {
        return CropDecision::Flag(FlagReason::NoBorderFound);
    }
    if found.ambiguous {
        return CropDecision::Flag(FlagReason::Ambiguous);
    }
    // `f32` throughout, and both counts in `u64` first so a large screenshot
    // cannot overflow the multiplication: see the module note on arithmetic
    // for why the width of the division is load-bearing at the boundary.
    let area = u64::from(found.rect.w) * u64::from(found.rect.h);
    let total = u64::from(img.width) * u64::from(img.height);
    if area as f32 / (total as f32) < t.min_content_fraction
        || found.rect.w < t.min_content_side
        || found.rect.h < t.min_content_side
    {
        return CropDecision::Flag(FlagReason::LowContent);
    }
    CropDecision::Crop(found.rect)
}
