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
//!
//! # When it declines
//!
//! `None` when no run of [`MIN_RUN`] page-like rows exists - including when
//! there is no pixel beside the column at all. The pipeline then leaves the
//! rows exactly as the earlier stages did. MC-048 AC-3 relies on this: on the
//! two `2025-08-05` WebPs the full-margin reading finds no such run, and their
//! crops are unchanged. There is deliberately **no** narrower fallback reading
//! (MC-048 `## Amendments`).

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
/// returned, before the margin. Only its `x` and `w` are read; the rows are
/// the whole image's. See the module documentation for the method and for
/// when it declines.
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
    let total = (left_end + width - right_start) as f32;
    let page_like: Vec<bool> = (0..rows)
        .map(|y| {
            let near = margin(y).filter(|&&v| v.abs_diff(tone) <= tol).count();
            near as f32 / total >= PAGE_LIKE
        })
        .collect();

    let (top, bottom) = outer_runs(&page_like)?;
    Some(Viewport {
        top: u32::try_from(top).ok()?,
        bottom: u32::try_from(bottom).ok()?,
    })
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
