//! Stage 3b of the pipeline: pull the rect in to the outermost **textured**
//! line, on whichever axis the gradient locator is blind (MC-025).
//!
//! [`trim`](crate::trim) drops borders that are one flat colour.
//! [`content`](crate::content) peels edge strips that a strong line marks the
//! inner end of. Between them they cover every boundary that is either exactly
//! uniform or announced by a step - and MC-019 measured seven real corpus
//! edges that are neither. A dark panel bleeding into a dark gutter over tens
//! of pixels has no step anywhere in the bleed (peak adjacent-line difference
//! 0.59 to 7.2 against an [`edge_threshold`](Tuning::edge_threshold) of 24)
//! and no uniform gutter either (a real gutter carries compression noise).
//! Both earlier stages walk past it, and the detector returns the whole page.
//!
//! [`edges::textured_span`](crate::edges::textured_span) sees it, because it
//! asks whether a line is flat rather than whether it changed. This module is
//! the one place that locator is wired into the pipeline, and the whole of its
//! content is *when* to listen to it.
//!
//! # Why it stands down where the gradient can see
//!
//! The rule is one line of code and it is the story's central engineering
//! decision: **on each axis, the flatness locator speaks only when
//! [`strong_lines`] found nothing on that axis.** Where the gradient found a
//! line, stages 2 and 3 have already judged what is on each side of it -
//! against the extent limit, the flat fraction and the content left behind -
//! and this stage has nothing to add and every opportunity to disagree. Where
//! the gradient found nothing, there is no judgement to contradict: either the
//! page is all art, in which case every line is textured and the span is the
//! whole rect, or there is a boundary no step-detector can see, which is the
//! case this module exists for.
//!
//! The alternative - wiring flatness into
//! [`content::strip_depth`](crate::content) as a fallback candidate - is what
//! MC-025's story rejects, and the reason is a frozen assertion rather than a
//! preference. `tests/content.rs::the_edge_threshold_is_read_from_the_tuning`
//! stacks a 6-row chrome band at a 0.90 flat fraction on 94 rows of art,
//! raises `edge_threshold` to 200 so that no strong line exists anywhere, and
//! requires `content_box` to return the whole image untouched. That band's row
//! spread is exactly 2.0, a quarter of [`Tuning::min_line_spread`], so a
//! flatness fallback inside `content_box` locates it and peels it - and the
//! band is a chrome candidate on all three of `judge`'s conditions, so making
//! the fallback pass them too changes nothing. The assertion is what says
//! `edge_threshold` is the constant that finds a strip, and it has to keep
//! saying it.
//!
//! Standing outside `content_box` is what keeps that promise: this stage runs
//! in [`detect`](crate::detect) and nothing that calls `content_box` directly
//! can see it.
//!
//! # Why it runs after the second uniform trim, and why nothing follows it
//!
//! The order matters in one direction only, and it is settled by an
//! invariant. A line this stage keeps has spread at or above
//! [`min_line_spread`](Tuning::min_line_spread), and
//! [`min_line_spread`](Tuning::min_line_spread) sits strictly above
//! `uniform_tolerance / 2`, which is the largest mean absolute deviation a
//! line [`trim_uniform`](crate::trim::trim_uniform) calls uniform can have.
//! So **the edge this stage leaves behind is never uniform** and it can never
//! expose trimming work for a later pass - whereas running it last means it
//! sees a rect whose uniform borders are already gone, which is the cleaner
//! signal. That is also why the floor under `min_line_spread` is a floor and
//! not a preference: at or below `uniform_tolerance / 2` the two stages would
//! contradict each other and the order would start to matter.
//!
//! # What it reports
//!
//! Nothing of its own. Moving an edge here sets
//! [`Detection::trimmed`](crate::Detection::trimmed), for the same reason the
//! second uniform trim does: a flat border removed is a border removed, and
//! [`decide`](crate::decide()) must not then call the page "all art, nothing
//! found". The field's definition is in [`decide`](crate::decide)'s module
//! documentation.

use crate::edges::{Axis, col_profile, row_profile, spread_within, strong_lines, textured_span};
use crate::{Luma, Rect, Tuning};

/// `within`, pulled in on each axis to the outermost line of it whose spread
/// reaches [`Tuning::min_line_spread`].
///
/// Returns `within` unchanged on any axis where [`strong_lines`] found a
/// strong line - see the module documentation, which is where that rule is
/// argued - and on any axis whose lines are *all* flat, since a rect with no
/// textured line at all has no boundary to place and belongs to
/// [`decide`](crate::decide()), which flags it.
///
/// The rows are narrowed first and the columns then measured over what is
/// left, rather than both being measured over `within` and applied together:
/// the axes are not independent, and a column's spread over a rect that still
/// holds 40 rows of gutter is not the spread of that column of the art.
#[must_use]
pub fn textured_box(img: &Luma, within: Rect, t: &Tuning) -> Rect {
    let rows = narrow(img, within, Axis::Rows, t);
    narrow(img, rows, Axis::Columns, t)
}

/// [`textured_box`] on one axis.
fn narrow(img: &Luma, rect: Rect, axis: Axis, t: &Tuning) -> Rect {
    let gradient = match axis {
        Axis::Rows => row_profile(img, rect),
        Axis::Columns => col_profile(img, rect),
    };
    if !strong_lines(&gradient, t).is_empty() {
        return rect;
    }

    let Some((first, last)) = textured_span(&spread_within(img, rect, axis), t) else {
        return rect;
    };
    // A spread profile index *is* a line index - unlike a gradient profile
    // index, which sits between two lines - so the arithmetic is an offset
    // from the rect's own origin and nothing else. Both bounds are inclusive,
    // hence the `+ 1`. `usize` to `u32` cannot lose anything: the index came
    // from a profile with one entry per line of a rect whose extent is a
    // `u32`.
    let (first, last) = (first as u32, last as u32);
    match axis {
        Axis::Rows => Rect {
            y: rect.y + first,
            h: last - first + 1,
            ..rect
        },
        Axis::Columns => Rect {
            x: rect.x + first,
            w: last - first + 1,
            ..rect
        },
    }
}
