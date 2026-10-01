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

use std::cell::OnceCell;

use crate::edges::{
    Axis, col_profile, row_profile, spread_within, strong_lines, textured_span, widest_textured_run,
};
use crate::viewport;
use crate::{Luma, Rect, Tuning};

/// MC-049's page-background share, as a whole percentage: a column is page
/// background over some rows when at least this share of its pixels there lie
/// within `uniform_tolerance` of its own upper median.
const PAGE_BACKGROUND_PERCENT: u64 = 95;

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
//
// # Where the page column ends: at the margin, not at the threshold (MC-053)
//
// Up to MC-053 the widest run's own ends were the answer, which made
// `min_line_spread` the page column's side boundary. That cut into dark,
// low-texture art: the outer columns of a dark page fade below 8.0 well
// before the page ends - 1.8 to 7.8 on the three corpus entries MC-053 moved
// to `tuning`, `2025-07-17 14_41_58.png`, `2025-07-17 14_55_10.png` and
// `Screenshot (73).png` - and the crop lost 2 to 24 columns of art on a side.
// Lowering the threshold is no fix: a JPEG margin column can have more
// spread (5.27 on `Screenshot (2630).jpg`) than dark art has (1.9), so no
// single value keeps one and drops the other (MC-053 AC-2's and AC-7's
// controls).
//
// So the widest run now only *finds* the page, and the page column is that
// run widened outward, one column at a time on each side, until the next
// column is page margin. A column is **margin**, and the widening stops, on
// the first of these that holds:
//
// 1. **Its tone is the margin's and not the page's.** Its median over the
//    band lies within `uniform_tolerance` of the page background tone, as
//    the viewport stage reads that tone (`viewport::page_background_tone`,
//    MC-048's instrument), and *not* within `uniform_tolerance` of the column
//    just inside it. This is the column where the page's own tone has already
//    handed over to the margin's: the resampled edge of a bright page on a
//    dark margin, the near-flat grey columns MC-049 names on
//    `Screenshot (3538).png`, which are not page background by MC-049's
//    predicate and must still not be taken.
// 2. Otherwise, a column that is **not page background over the band** -
//    MC-049's predicate: under 95 % of its pixels within `uniform_tolerance`
//    of its own median - is page, whatever its spread. That is the story's
//    definition of dark, low-texture art.
// 3. A column that **is** page background over the band is margin, unless
//    both of these hold (MC-055):
//    - **it carries the page's tone on**: its band median is within
//      `uniform_tolerance` of the page column's edge column, the same
//      comparison branch 1 makes, read the other way; and
//    - **it is not page background over the viewport's rows** (MC-048's and
//      MC-052's instrument, read beside the run).
//
//    Why those rows. MC-049's predicate is defined over the *page's* rows,
//    and the band is only a stand-in for them: the middle of the rect,
//    chosen so that the browser chrome and the taskbar cannot reach it.
//    That makes it a sample, and a dark page can be flat all the way through
//    it. The outer columns of `2025-07-17 14_41_58.png` are 0.95 to 0.99
//    within `uniform_tolerance` of their median over the band, and 0.73 to
//    0.76 over the viewport, where the page's white panel gutters cross them.
//    The viewport is the measured set of rows between the chrome and the
//    taskbar, so it holds every row the page can be on. By its definition,
//    the margin beside the page is flat on those rows; that margin is what
//    the viewport stage is read from.
//
//    Why continuity. A column that is flat over the band is either margin or
//    art that looks like margin in the middle of the frame. If it is art, it
//    is the page carried on, and a person reads that edge the same way: the
//    column has the tone of the page column just inside it. Reading the
//    viewport only for such a column keeps the second reading from speaking
//    for a column whose tone has already handed over. Two columns show why
//    this matters. `Screenshot (2630).jpg`'s 1591 has band median 1 beside a
//    page edge of 72, and `Screenshot (2708).jpg`'s 1073 has 12 beside 28.
//    Each is flat over the band and not over the viewport. Branch 1 stops
//    both on its own, but with no slack in one case and little in the other:
//    1591 is exactly `uniform_tolerance` from the tone, and 1073 is 16 levels
//    from its edge. Continuity stops them with differences of 71 and 16, so
//    no single comparison decides either one alone.
//
//    Neither half compares against a value finer than `uniform_tolerance`.
//    The tone, which MC-053's version of this branch compared *exactly*
//    against the band median, is not read here at all. A column flat at the
//    page's tone that continues a page edge of the same tone is re-read,
//    and the margin is flat over the viewport by construction. So where page
//    background and art are the same grey over the band -
//    `tests/flatness.rs`' waist fixture at a band of 0.5, where branch 1 has
//    already stopped the widening - the band still decides, as MC-027
//    settled.
//
//    One more column is page although it is flat over the viewport too
//    (MC-065): one whose band median is neither the page background tone
//    nor the tone of the margin beyond it, where that margin is a site of
//    one exact value - the first column past it that holds a single value
//    on every row of the band. A margin of one exact value has that value in
//    every one of its columns, so a flat column of another tone, carrying
//    the page's tone on, is the page's own paper. `2025-08-07 01_13_55.png`'s
//    white strip (255) beside a site of 25 is the case.
//
// A widening that reaches either end of the rect falls under the interior
// rule above exactly as the run itself does.
//
// None of this adds a constant: 0.95 is MC-049's predicate, the tolerance is
// `uniform_tolerance`, the tone and the viewport are MC-048's and MC-052's
// settled instrument, and `min_line_spread` keeps its value and its meaning
// everywhere else - `textured_box` above does not read any of this. MC-055's
// `## Notes` has each branch's tolerance probe: every comparison above moved
// by up to `uniform_tolerance`, over the tuning corpus.

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
/// the widest run of columns that are textured over [`central_band`] of the
/// rect's rows, widened outward on each side up to the page margin (MC-053,
/// MC-055).
///
/// Returns `within` unchanged where there is no page column to find - no
/// textured column at all, or a page column that reaches column 0 or the
/// rect's last column, which is a rect with no page margin on that side. The
/// section comments above are where both rules are argued: the run and the
/// interior rule are MC-027's, and where the page column ends is MC-053's
/// and MC-055's.
///
/// The rows are **never** moved: the returned rect's `y` and `h` are
/// `within`'s own. Locating the top and bottom of the page is a different
/// problem with a different detector (the panel gutter, MC-028), and this
/// stage does not guess at it.
#[must_use]
pub fn page_column(img: &Luma, within: Rect, t: &Tuning) -> Rect {
    let band = central_band(within, t);
    let spread = spread_within(img, band, Axis::Columns);
    let Some((first, last)) = widest_textured_run(&spread, t) else {
        return within;
    };
    // A spread profile index *is* a line index, so the arithmetic is an offset
    // from the rect's own origin; both bounds are inclusive, hence the `+ 1`.
    let run = Rect {
        x: within.x + first as u32,
        w: (last - first + 1) as u32,
        ..within
    };
    let (first, last) = extend_to_the_margin(img, within, band, run, t);
    // The interior rule, read on the column the extension arrived at: a page
    // column that reaches either end of `within` has no page margin on that
    // side.
    if first == within.x || last + 1 == within.x + within.w {
        return within;
    }
    Rect {
        x: first,
        w: last - first + 1,
        ..within
    }
}

/// The first and last image column, both inclusive, of `run` widened outward
/// on each side for as long as the next column [`Margin::belongs`] to the page.
fn extend_to_the_margin(img: &Luma, within: Rect, band: Rect, run: Rect, t: &Tuning) -> (u32, u32) {
    let margin = Margin {
        img,
        within,
        run,
        t,
        band: band.y..band.y + band.h,
        view: OnceCell::new(),
        tone: viewport::page_background_tone(img, run),
        tol: t.uniform_tolerance,
    };

    let mut first = run.x;
    let mut edge = margin.over_band(first).0;
    while first > within.x {
        match margin.belongs(first - 1, edge, false) {
            Some(median) => {
                first -= 1;
                edge = median;
            }
            None => break,
        }
    }
    let mut last = run.x + run.w - 1;
    let mut edge = margin.over_band(last).0;
    while last + 1 < within.x + within.w {
        match margin.belongs(last + 1, edge, true) {
            Some(median) => {
                last += 1;
                edge = median;
            }
            None => break,
        }
    }
    (first, last)
}

/// What [`extend_to_the_margin`] reads a column against.
struct Margin<'a> {
    img: &'a Luma,
    /// The rect the page column is being located in.
    within: Rect,
    /// The widest textured run: what the viewport is read beside.
    run: Rect,
    t: &'a Tuning,
    /// The central band's rows.
    band: std::ops::Range<u32>,
    /// The viewport's rows inside the rect, where one was located. Read only
    /// when a column flat over the band continues the page's tone - branch 3
    /// of the section comment - which on most pages never happens, so the
    /// viewport stage's full-image passes are not paid for here unless they
    /// are needed.
    view: OnceCell<Option<std::ops::Range<u32>>>,
    /// The page background tone, where there is a margin to read it from.
    tone: Option<u8>,
    tol: u8,
}

impl Margin<'_> {
    /// Column `x`'s median over the band, and whether it is page background
    /// there.
    fn over_band(&self, x: u32) -> (u8, bool) {
        column_background(self.img, x, self.band.clone(), self.tol)
    }

    /// The viewport's rows inside the rect, located once, on first use.
    fn view(&self) -> Option<std::ops::Range<u32>> {
        self.view
            .get_or_init(|| {
                viewport::locate(self.img, self.run, self.t).and_then(|v| {
                    let top = v.top.max(self.within.y);
                    let bottom = v.bottom.min(self.within.y + self.within.h);
                    (top < bottom).then_some(top..bottom)
                })
            })
            .clone()
    }

    /// `Some(median over the band)` when column `x`, just outside a page
    /// column whose edge column has band median `edge`, is part of the page;
    /// `None` when it is the margin. The three branches are the section
    /// comment's, in its order.
    ///
    /// `outward` is the direction of the widening: `true` to the right.
    fn belongs(&self, x: u32, edge: u8, outward: bool) -> Option<u8> {
        let (median, background) = self.over_band(x);
        let continues = median.abs_diff(edge) <= self.tol;
        let at_tone = self
            .tone
            .is_some_and(|tone| median.abs_diff(tone) <= self.tol);
        // 1. The margin's tone and not the page's.
        if at_tone && !continues {
            return None;
        }
        // 2. Not page background over the band.
        if !background {
            return Some(median);
        }
        // 3. Flat over the band, but the page's tone carried on, and either
        //    not page background over the viewport's rows, or of neither the
        //    margin's tone nor the tone of the single-value site beyond it
        //    (MC-065).
        if !continues {
            return None;
        }
        let (_, flat) = column_background(self.img, x, self.view()?, self.tol);
        let not_the_site = || !at_tone && self.site_differs(x, median, outward);
        (!flat || not_the_site()).then_some(median)
    }

    /// Whether the margin beyond column `x`, read outward, is a site of one
    /// exact value whose tone is not `median`'s (MC-065): the first column
    /// past `x` that holds one value on every row of the band exists, and
    /// that value lies more than `uniform_tolerance` from `median`.
    ///
    /// A flat column at the page's tone cannot be such a site's margin: a
    /// margin of one exact value has that value in every one of its columns,
    /// and this column is neither that value nor within tolerance of it.
    fn site_differs(&self, x: u32, median: u8, outward: bool) -> bool {
        let width = self.img.width;
        let single = |c: u32| {
            let at = |y: u32| self.img.data[(y * width + c) as usize];
            let first = at(self.band.start);
            self.band.clone().all(|y| at(y) == first).then_some(first)
        };
        let site = if outward {
            (x + 1..self.within.x + self.within.w).find_map(single)
        } else {
            (self.within.x..x).rev().find_map(single)
        };
        site.is_some_and(|value| value.abs_diff(median) > self.tol)
    }
}

/// Column `x`'s upper median over `rows`, and whether it is page background
/// there by MC-049's predicate: at least [`PAGE_BACKGROUND_PERCENT`] percent of its
/// pixels within `tol` of that median.
fn column_background(img: &Luma, x: u32, rows: std::ops::Range<u32>, tol: u8) -> (u8, bool) {
    let mut counts = [0u32; 256];
    let len = rows.len() as u32;
    for y in rows {
        counts[usize::from(img.data[(y * img.width + x) as usize])] += 1;
    }
    let mut seen = 0;
    let mut median = 0u8;
    for (value, &count) in (0u8..=255).zip(&counts) {
        seen += count;
        if seen > len / 2 {
            median = value;
            break;
        }
    }
    let lo = usize::from(median.saturating_sub(tol));
    let hi = usize::from(median.saturating_add(tol));
    let near: u32 = counts[lo..=hi].iter().sum();
    (
        median,
        u64::from(near) * 100 >= u64::from(len) * PAGE_BACKGROUND_PERCENT,
    )
}
