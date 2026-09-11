//! Stage 4 of the pipeline: give the crop a little air
//! (`docs/wiki/architecture.md`, "cropper-core", step 4, and decision 5).
//!
//! The earlier stages converge on the tightest rect that still holds art. That
//! is the right thing to *find* and the wrong thing to *cut*: a detector one
//! pixel optimistic on any side shaves the outline off a panel, and the brief's
//! feature 2 is "never clip artwork". So the answer is expanded outward by
//! [`Tuning::margin_px`] on all four sides before it leaves the crate, and the
//! cost of being wrong becomes a thin band of page colour instead of a missing
//! line of art.
//!
//! Two rules, and both are ordinary arithmetic that is only interesting at the
//! boundary:
//!
//! * the margin is **clamped, not refused**. Art flush with an edge - MC-006
//!   AC-3's scene, where the right border is 0 px wide - gets whatever margin
//!   fits on that side, which is none, and the full margin on the other three.
//!   An expansion that overflowed the image would describe a crop the engine
//!   cannot take;
//! * the margin is a **fixed pixel count**, not a share of the image (decision
//!   5). A webtoon strip is tall and narrow, so a proportional margin would be
//!   invisible across and enormous down.
//!
//! `margin_px` is 3 by default and is read from the [`Tuning`] argument at
//! every call, never compiled in: MC-006 drives `detect` at 0, 3 and 7 on one
//! fixture to pin that.

use crate::{Dimensions, Rect};

/// `rect` grown by `by` pixels on every side, clipped to `bounds`.
///
/// Saturating on the near edges and `min` on the far ones, so a rect already
/// touching an edge simply keeps it, and a rect at the origin with a margin
/// larger than the image comes back as the whole image rather than wrapping.
/// `rect` must lie inside `bounds` - it does for everything the pipeline hands
/// this function, since every stage only ever shrinks the image rect - and the
/// precondition is not checked.
///
/// # Panics
///
/// In debug builds, if `rect` starts beyond the far edge of `bounds`: the
/// clamped far edge would then sit left of the clamped near one and the width
/// underflows.
#[must_use]
pub fn expand(rect: Rect, by: u32, bounds: Dimensions) -> Rect {
    let x = rect.x.saturating_sub(by);
    let y = rect.y.saturating_sub(by);
    // Saturating on the way out too: `rect.x + rect.w` is inside `bounds` for
    // every caller, but adding the margin to it need not fit in `u32` if some
    // future caller passes a large one, and the `min` below would then be
    // taken against a wrapped number.
    let right = rect
        .x
        .saturating_add(rect.w)
        .saturating_add(by)
        .min(bounds.width);
    let bottom = rect
        .y
        .saturating_add(rect.h)
        .saturating_add(by)
        .min(bounds.height);
    Rect {
        x,
        y,
        w: right - x,
        h: bottom - y,
    }
}
