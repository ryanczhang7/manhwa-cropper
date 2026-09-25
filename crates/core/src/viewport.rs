//! Stage 6 of the pipeline: find the browser viewport and drop what is outside
//! it (MC-048; `docs/wiki/architecture.md`, "cropper-core").
//!
//! By the time [`page_column`](crate::flat::page_column) has spoken the rect
//! is the page column on the column axis, but on the row axis it still runs
//! from the browser's tab strip to the Windows taskbar: both are textured out
//! to the image's edges, so no earlier stage can tell them from art. What
//! tells them apart is **beside** the page column. A reader page sits on a
//! flat page background, so a row inside the viewport has flat pixels to the
//! left and right of the column; a row of browser chrome or taskbar is painted
//! edge to edge and has none. [`locate`] reads that difference and returns the
//! viewport's rows.
//!
//! # The oracle, settled
//!
//! This is MC-031's chrome oracle, ported rather than tuned. Every constant
//! here is the parameterisation `docs/wiki/chrome-row-search.md` scored in
//! section 3b and read out per file in section 4, and MC-048 re-measures the
//! port against that table (`crates/engine/tests/corpus_viewport_stage.rs`).
//! None of it is calibrated in this crate; a change to any of it is a change
//! to a measured instrument, and belongs to a story that re-measures it.
//!
//! 1. **The margin**: for every row, the pixels outside the column's
//!    `x .. x + w`, over the **full** image width, with no gap next to the
//!    column.
//! 2. **The page background tone**, once per image: each row's margin median,
//!    binned into 32 bins of 8 grey levels; the modal bin wins, and inside it
//!    the modal exact median is the tone. Ties go to the higher bin and the
//!    higher value.
//! 3. **Per-row flatness**: the share of that row's margin pixels within
//!    [`Tuning::uniform_tolerance`] of the tone. A row is **page-like** at
//!    [`PAGE_LIKE`] or above.
//! 4. **The chrome strips**: the viewport runs from the start of the first
//!    run of at least [`MIN_RUN`] page-like rows to the end of the last one.
//!    Anything above is the browser chrome strip and anything below is the
//!    taskbar strip, short page-like runs inside either included - a flat
//!    band in a tab strip does not end it. This is the *ChromeStrip* selector
//!    of MC-031's harness: scanning in from each edge for the first page-like
//!    run that is long enough, which is the same pair of rows.
//! 5. **The reader's own window** (MC-052, below): when step 4's viewport
//!    overlaps the column's rows, steps 3 and 4 run a second time over only
//!    the columns of the reader's window, found from step 4's viewport, and
//!    where that second viewport reaches further up or down than step 4's, the
//!    viewport is widened to it. It is never narrowed.
//!
//! # The second reading: the reader's own window (MC-052)
//!
//! MC-048 scored step 3 over the whole margin of step 1. On a split-screen
//! screenshot that margin includes a **second** browser window beside the
//! reader's, with its own chrome and its own bottom edge, and on the rows only
//! the reader's viewport covers, that window's chrome pulled the share under
//! [`PAGE_LIKE`]: the stage returned the second window's viewport and the crop
//! cut the reader's art (`2025-03-06 01_22_45.png`,
//! `2025-03-07 00_58_06.png`). So step 5 reads step 3 again over only the
//! columns of the reader's window, found once per image:
//!
//! - A column is **page background** when, over the rows of step 4's
//!   viewport, a majority - the column's median there, in effect - lie within
//!   `uniform_tolerance` of the tone from step 2. Inside a reader window its
//!   margin columns are flat on all of those rows; the reader's scrollbar and
//!   frame are flat on none of them.
//! - From each side of the page column the reader's window runs outward over
//!   page-background columns and stops at the first column that is not: the
//!   window's edge, which on the corpus is the reader's scrollbar
//!   (x 1811..1827 on both files above, the second window from 1828). Pixels
//!   past it belong to whatever sits beside the window and are not read.
//!   A side with no edge at all is read to the image's edge, which on a
//!   single-window screenshot differs from MC-048's margin only by the
//!   reader's own scrollbar at the image edge.
//!
//! **Why it only widens, and only a viewport over the page.** The window
//! reading at first *replaced* MC-048's. `tests/detect.rs`'s generator
//! property found what that costs. Where the columns beside the page are
//! textured chrome at a flat fraction near [`PAGE_LIKE`] - the generator draws
//! 0.86 to 0.98; 0.89 and 0.93 in the case it found - the window can be those
//! columns and nothing else, and their per-row share wobbles either side of
//! 0.90: 0.887 on the last art row there, so the replaced reading ended the
//! viewport one row into the art. The whole margin, which there also holds a
//! solid border and a chrome band of another tone, is 0.787 on every row, and
//! MC-048 declines. The same wobble carried a viewport that MC-048 found in a
//! solid border *above* the art - which the pipeline treats as a decline,
//! because it misses the column's rows - some rows down into the art, and
//! cropped there. A split-screen shot is neither: the whole margin finds the
//! rows both windows cover, over the page, and the window finds the reader's
//! rows, a superset. So MC-048's reading is kept exactly and decides alone
//! whether the stage speaks at all; only a viewport that overlaps the
//! column's rows is widened; and the window reading can only add rows. Every
//! row MC-048's stage kept, this one keeps, and wherever MC-048's stage
//! declined, this one declines.
//!
//! There is no minimum distance between the page column and a window edge.
//! GREEN had one (`MIN_WINDOW_MARGIN`, 16 columns) so that on the two
//! `2025-08-05` WebPs, where a reader panel of another tone hugs the page, the
//! side was read over its full width and the stage still declined. Under the
//! rule above the whole margin already declines there, and the guard did no
//! work any test can see: every test passes without it, and over 75,000
//! generated cases (`tests/detect.rs`'s generator, 15 fixed seeds) the rule
//! without it clipped none, against 2 with it (both also clipped by MC-048's
//! stage). MC-052 `## Notes`, "GATES: the proptest clip", has the counts.
//!
//! The tone (step 2), [`PAGE_LIKE`], [`MIN_RUN`] and `uniform_tolerance` are
//! unchanged: this changes which pixels the settled instrument reads, not the
//! instrument. It is deliberately **not** "a row is page-like when either side
//! of the column is flat": beside a second window whose viewport reaches past
//! the reader's, that rule reads the reader's own chrome as page (MC-052
//! AC-6, case B). Widening reads the reader's window, not either side, so the
//! second window's taller viewport is never read.
//!
//! # When it declines
//!
//! `None` when the whole-margin reading finds no run of [`MIN_RUN`] page-like
//! rows - including when there is no pixel beside the column at all - however
//! the window reading would come out. A whole-margin viewport clear of the
//! column's rows is returned as it is, unwidened, and the pipeline declines on
//! it as MC-048's did. The pipeline then leaves the rows exactly as
//! the earlier stages did. MC-048 AC-3 relies on this: on the two `2025-08-05`
//! WebPs the full-margin reading finds no such run, and their crops are
//! unchanged. There is deliberately **no** narrower fallback reading tried
//! after a decline (MC-048 `## Amendments`): a decline of the whole margin is
//! final, and the window reading can only widen a viewport the whole margin
//! found.

use crate::{Luma, Rect, Tuning};

/// The share of a row's margin pixels that must lie within
/// [`Tuning::uniform_tolerance`] of the page background tone for the row to
/// be page-like: `chrome-row-search.md` section 3b and section 4's threshold.
///
/// A module constant rather than a [`Tuning`] field because it is part of a
/// settled instrument, not a knob: section 3c measures browser chrome rows at
/// a median of 0.000 and page rows at 0.993, so the threshold sits far from
/// both populations.
pub const PAGE_LIKE: f32 = 0.90;

/// The fewest consecutive page-like rows that end a chrome strip:
/// `chrome-row-search.md` section 4's minimum run. Shorter page-like runs are
/// read as part of the strip they sit in.
pub const MIN_RUN: usize = 16;

/// Number of 8-level bins the per-row margin medians are histogrammed into to
/// find the page background tone.
const TONE_BINS: usize = 32;

/// The browser viewport's rows: everything between the browser chrome and the
/// taskbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    /// The first row of the viewport: the first row below the browser chrome.
    pub top: u32,
    /// The first row of the taskbar, exclusive: the viewport is
    /// `top .. bottom`.
    pub bottom: u32,
}

/// The browser viewport's rows in `img`, read from the pixels beside
/// `column`, or `None` when no viewport can be located.
///
/// `column` is the page column [`page_column`](crate::flat::page_column)
/// returned, before the margin. Its `x` and `w` say which pixels are beside
/// it, over the whole image's rows; its `y` and `h` are read only to tell
/// whether the whole-margin viewport lies over the page at all, which is the
/// one condition under which the reader-window reading may widen it (MC-052).
/// See the module documentation for the method and for when it declines.
#[must_use]
pub fn locate(img: &Luma, column: Rect, t: &Tuning) -> Option<Viewport> {
    let width = img.width as usize;
    let left_end = (column.x as usize).min(width);
    let right_start = (column.x as usize + column.w as usize).min(width);
    if left_end == 0 && right_start == width {
        return None;
    }
    let margin = |y: usize| {
        let row = &img.data[y * width..(y + 1) * width];
        row[..left_end].iter().chain(&row[right_start..])
    };
    let rows = img.height as usize;

    let medians: Vec<u8> = (0..rows).map(|y| median(margin(y))).collect();
    let tone = background_tone(&medians);
    let tol = t.uniform_tolerance;

    // MC-048's reading: the whole margin. It alone decides whether there is a
    // viewport at all.
    let whole = (left_end + width - right_start) as f32;
    let page_like: Vec<bool> = (0..rows)
        .map(|y| {
            let near = margin(y).filter(|&&v| v.abs_diff(tone) <= tol).count();
            near as f32 / whole >= PAGE_LIKE
        })
        .collect();
    let (mut top, mut bottom) = outer_runs(&page_like)?;
    // A viewport clear of the column's rows is one the pipeline treats as a
    // decline; the window reading must not turn it into a crop.
    let (first, end) = (column.y as usize, column.y as usize + column.h as usize);
    if top >= end || bottom <= first {
        return viewport(top, bottom);
    }

    // MC-052's reading: the reader's own window. It may only widen MC-048's
    // rows, never narrow them.
    let (lo, hi) = reader_window(img, left_end, right_start, top..bottom, tone, tol);
    let beside = |y: usize| {
        let row = &img.data[y * width..(y + 1) * width];
        row[lo..left_end].iter().chain(&row[right_start..hi])
    };
    let total = (left_end - lo + hi - right_start) as f32;
    let in_window: Vec<bool> = (0..rows)
        .map(|y| {
            let near = beside(y).filter(|&&v| v.abs_diff(tone) <= tol).count();
            near as f32 / total >= PAGE_LIKE
        })
        .collect();
    if let Some((window_top, window_bottom)) = outer_runs(&in_window) {
        top = top.min(window_top);
        bottom = bottom.max(window_bottom);
    }
    viewport(top, bottom)
}

/// Rows `top .. bottom` as a [`Viewport`], or `None` past `u32`.
fn viewport(top: usize, bottom: usize) -> Option<Viewport> {
    Some(Viewport {
        top: u32::try_from(top).ok()?,
        bottom: u32::try_from(bottom).ok()?,
    })
}

/// The reader window's columns beside the page column, `lo .. hi`: the page
/// column's `left_end` and `right_start` widened outward, one column at a
/// time, for as long as each column is page background at its median over
/// `viewport` - that is, for as long as a majority of the rows the whole
/// margin reads as page lie within `tol` of `tone` in that column.
///
/// The first column that is not stops the widening on that side: the edge of
/// the reader's window (its scrollbar, its frame), past which the pixels
/// belong to whatever sits beside the window (MC-052).
///
/// `viewport` is the whole-margin reading's viewport, not every row, because
/// those are the rows on which a window's own margin is known to be showing
/// page: a column that is page there and not elsewhere is still the window's.
/// Read over every row instead, a column of textured chrome at a flat
/// fraction over one half counts as page background whatever else is going
/// on, and where the whole-margin viewport is a solid border above the art,
/// that let the window reading carry it a few rows into the art (crops of 5
/// to 23 rows, found by stressing `tests/detect.rs`'s generator; `locate`'s
/// overlap check stops most of them too, and the two together stop all it
/// found). On those border rows every column is flat, so the window is the
/// whole margin and the two readings agree. If both sides stop at once the
/// window is empty, every share is NaN and not page-like, and nothing widens.
fn reader_window(
    img: &Luma,
    left_end: usize,
    right_start: usize,
    viewport: std::ops::Range<usize>,
    tone: u8,
    tol: u8,
) -> (usize, usize) {
    let width = img.width as usize;
    let rows = viewport.len();
    let mut near = vec![0usize; width];
    for row in img.data.chunks_exact(width).skip(viewport.start).take(rows) {
        for (count, &v) in near.iter_mut().zip(row) {
            *count += usize::from(v.abs_diff(tone) <= tol);
        }
    }
    let background = |x: usize| near[x] * 2 > rows;
    let lo = (0..left_end)
        .rev()
        .find(|&x| !background(x))
        .map_or(0, |edge| edge + 1);
    let hi = (right_start..width)
        .find(|&x| !background(x))
        .unwrap_or(width);
    (lo, hi)
}

/// The upper median of `values`, which must be non-empty: the element at
/// index `len / 2` once sorted. Counted in a 256-bin histogram rather than
/// sorted, because a screenshot's margin is thousands of pixels a row.
fn median<'a>(values: impl Iterator<Item = &'a u8>) -> u8 {
    let mut counts = [0usize; 256];
    let mut len = 0usize;
    for &v in values {
        counts[usize::from(v)] += 1;
        len += 1;
    }
    let mut seen = 0usize;
    for (value, &count) in (0u8..=255).zip(&counts) {
        seen += count;
        if seen > len / 2 {
            return value;
        }
    }
    unreachable!("median of an empty row")
}

/// The page background tone: the modal per-row median in 8-level bins,
/// refined to the modal exact median inside the winning bin. Ties go to the
/// higher bin and the higher value.
fn background_tone(medians: &[u8]) -> u8 {
    let mut bins = [0usize; TONE_BINS];
    let mut exact = [0usize; 256];
    for &m in medians {
        bins[usize::from(m / 8)] += 1;
        exact[usize::from(m)] += 1;
    }
    let best = last_max(&bins);
    let winning = &exact[best * 8..best * 8 + 8];
    u8::try_from(best * 8 + last_max(winning)).unwrap_or(u8::MAX)
}

/// The index of the largest count, the last one on a tie.
fn last_max(counts: &[usize]) -> usize {
    let mut best = 0;
    for (i, &c) in counts.iter().enumerate() {
        if c >= counts[best] {
            best = i;
        }
    }
    best
}

/// `[start, end)` from the start of the first run of at least [`MIN_RUN`]
/// `true`s to the end of the last one, or `None` when there is no such run.
fn outer_runs(page_like: &[bool]) -> Option<(usize, usize)> {
    let mut first = None;
    let mut last = None;
    let mut y = 0;
    while y < page_like.len() {
        if !page_like[y] {
            y += 1;
            continue;
        }
        let start = y;
        while y < page_like.len() && page_like[y] {
            y += 1;
        }
        if y - start >= MIN_RUN {
            first.get_or_insert(start);
            last = Some(y);
        }
    }
    Some((first?, last?))
}
