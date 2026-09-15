//! The two pipeline stages that locate a boundary by **flatness** rather than
//! by a step: stage 3b, which pulls the rect in to the outermost textured line
//! on whichever axis the gradient locator is blind (MC-025), and stage 3c,
//! which narrows it to the page column between two flat page margins (MC-027).
//!
//! Everything below is stage 3b's. Stage 3c has a different rule, a different
//! selector and a different guard, and it is argued in its own section further
//! down, beside [`page_column`] - the two are deliberately not merged, because
//! the conditions under which each may speak are what the two stories are
//! about and they are not the same conditions.
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

use crate::edges::{
    Axis, col_profile, row_profile, spread_within, strong_lines, textured_span, widest_textured_run,
};
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

// --- Stage 3c: the page column, located by its flat page margins (MC-027) ---
//
// A reader screenshot has two different kinds of blank space in it
// (`docs/wiki/architecture.md`, "Two kinds of blank space"), and this stage is
// the first of them: the **page margin**, the flat browser background to the
// left of the page column and to the right of it. The **panel gutter** between
// panels is the other, it is a different thing with a different detector, and
// it is not this stage's - MC-028 owns it.
//
// Why `textured_box` above does not already find it, on a real screenshot:
//
// 1. it stands down on any axis where `strong_lines` found anything, and a
//    reader page has panel borders, so on the column axis it always has;
// 2. it takes the **outermost** textured pair, which annexes any second
//    textured region beside the page - a sidebar, a scrollbar, the other half
//    of a two-page spread in the browser's furniture;
// 3. it measures a column over **all** of the rect's rows, and the rect still
//    holds the browser chrome and the taskbar, which are textured out to its
//    own edges. That is the decisive one: every column reads textured, so
//    there is no page margin anywhere to find.
//
// So this stage is a stage of its own with a rule of its own, and the three
// differences above are that rule: no `strong_lines` guard, the **widest** run
// rather than the outermost pair, measured over the **central band** of the
// rect's rows rather than over all of them.
//
// # The guard it has instead
//
// **The rect is narrowed only where the widest textured run has a flat column
// on both sides of it.** Where the run reaches column 0 or the rect's last
// column, that side has no page *margin*, and a page-margin locator that cuts
// there is cutting into something else - a bare artwork file whose left edge
// happens to be blank through the middle of the image. MC-027's `## Handoff`
// measures that rule over all 28 corpus entries: "the widest run over the band
// touches neither end" agrees with "this entry carries a marked page" on 27 of
// them, and the single disagreement is an entry MC-026's corpus criteria
// deliberately assert nothing about. Without it, one of the six files frozen
// as `Flagged(NoBorderFound)` starts being cropped.
//
// That is not a tuning knob and there is no constant in it. It is the
// definition of a page margin, restated as code.

/// `rect`, restricted to the middle [`Tuning::central_band_fraction`] of its
/// rows: the rows [`page_column`] measures a column's spread over.
///
/// `floor(rect.h * fraction)` rows, centred - `rect.y + (rect.h - that) / 2` -
/// with **the columns untouched**, since the band restricts the sample and not
/// the answer. A rect with rows keeps at least one of them: a band of no rows
/// would give every column a spread of zero and make the locator silently
/// inert rather than loudly wrong.
///
/// # Arithmetic
///
/// The product is taken in `f64` and truncated. `0.6f32` is
/// `0.60000002384185791015625`, so the two plausible spellings disagree on
/// real heights: at a fraction of 0.7 a rect 240 rows tall gives 167 truncated
/// and 168 rounded, and a caller pinning one would be wrong against the other.
/// `f64` is what makes the truncation the mathematical floor of the `f32`
/// fraction's exact value rather than a rounding artefact of the multiply -
/// `240f32 * 0.6f32` is exactly 144 by luck, `100f32 * 0.6f32` is 60.000004,
/// and both are floored correctly only if the product carries its own error.
#[must_use]
pub fn central_band(rect: Rect, t: &Tuning) -> Rect {
    // `u32` to `f64` is exact, and the product of an exact `u32` with a
    // fraction in `[0, 1]` cannot leave `u32`'s range, so the cast back is
    // lossless for every rect an image can produce.
    let rows = (f64::from(rect.h) * f64::from(t.central_band_fraction)).floor() as u32;
    let rows = rows.clamp(u32::from(rect.h > 0), rect.h);
    Rect {
        y: rect.y + (rect.h - rows) / 2,
        h: rows,
        ..rect
    }
}

/// `within`, narrowed on the **column axis** to the page column inside it:
/// the widest run of columns that are textured over
/// [`central_band`] of the rect's rows.
///
/// Returns `within` unchanged where there is no page column to find - no
/// textured column at all, or a widest run that reaches column 0 or the rect's
/// last column, which is a rect with no page margin on that side. The section
/// comment above is where that rule is argued.
///
/// The rows are **never** moved: the returned rect's `y` and `h` are
/// `within`'s own. Locating the top and bottom of the page is a different
/// problem with a different detector (the panel gutter, MC-028), and this
/// stage does not guess at it.
#[must_use]
pub fn page_column(img: &Luma, within: Rect, t: &Tuning) -> Rect {
    let spread = spread_within(img, central_band(within, t), Axis::Columns);
    let Some((first, last)) = widest_textured_run(&spread, t) else {
        return within;
    };
    // The interior rule. `spread` has one entry per column of `within` - the
    // band restricts rows only - so an index reaching either end of it is an
    // edge of the rect itself.
    if first == 0 || last + 1 == spread.len() {
        return within;
    }
    // A spread profile index *is* a line index, so the arithmetic is an offset
    // from the rect's own origin; both bounds are inclusive, hence the `+ 1`.
    Rect {
        x: within.x + first as u32,
        w: (last - first + 1) as u32,
        ..within
    }
}
