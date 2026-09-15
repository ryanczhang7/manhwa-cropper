//! Stage 2 of the pipeline: row and column edge profiles, and the strong
//! lines in them (`docs/wiki/architecture.md`, "cropper-core", step 2).
//!
//! A **profile** has one entry per adjacent pair of lines inside a rect. The
//! row profile's entry `i` is the mean absolute luma difference between the
//! rect's row `i` and its row `i + 1`, taken over the rect's columns; the
//! column profile is the same statement with the axes swapped. So a rect of
//! `h` rows has `h - 1` row entries and `w - 1` column entries, and a rect
//! with fewer than two lines on an axis has none on that axis.
//!
//! A **strong line** is a run of consecutive profile indices whose values are
//! at or above [`Tuning::edge_threshold`], merged into one [`Line`] carrying
//! the run's first and last index, both inclusive. The threshold is read from
//! the [`Tuning`] argument at the one comparison site, never written as a
//! literal: the constant is settled at 24 today and the corpus story MC-019
//! may move it.
//!
//! The mean is a mean, not a sum: a profile entry has to be comparable against
//! a single threshold whatever the rect's width, which is the whole reason
//! `architecture.md` specifies the mean.
//!
//! What lives here stops at *locating* lines. Deciding which side of a line is
//! chrome is MC-005's job, and nothing in this module knows about it.
//!
//! # The other question: is this line flat? (MC-025)
//!
//! Everything above asks *how much did this line change from the last one*,
//! and on a gradual fade - a dark panel bleeding into a dark gutter over tens
//! of pixels - there is no answer to it. MC-019 measured seven such edges in
//! the corpus: peak adjacent-line step 0.59 to 7.2 against an
//! [`edge_threshold`](Tuning::edge_threshold) of 24, while the control taken
//! at a title bar in the same file measured 27 to 29. The transition is real
//! and it is simply spread too thin for any step-detector to see. Colour was
//! measured there too and ruled out (peak chroma step 0.07 to 4.18, weaker
//! still), so the signal has to be something other than a step.
//!
//! [`spread_profile`] asks the other question. Its entry for a line is that
//! line's **mean absolute deviation about its own mean**, so it is a property
//! of one line rather than of a pair, and a plane of `h` rows has `h` row
//! entries and `w` column entries - not `h - 1` and `w - 1`. Flat gutter reads
//! near zero however gradually the art faded into it; art reads well above
//! zero however little it changes from row to row.
//!
//! [`textured_span`] is the decision over that profile, and it mirrors
//! [`strong_lines`] deliberately: a function of a profile and a [`Tuning`],
//! knowing nothing about images or rects, with its threshold
//! ([`Tuning::min_line_spread`]) read from the argument at the one comparison
//! site. It returns the **outermost** textured indices and does not stop at a
//! flat run between them, because a flat run between two textured ones is a
//! panel gutter and MC-005's decision 13 keeps the panels rather than cutting
//! there.
//!
//! [`widest_textured_run`] (MC-027) is the *other* decision over the same
//! profile: the widest single stretch of texture, which does stop at a flat
//! run. It does not replace [`textured_span`] and neither is a better version
//! of the other - they answer different questions, and the doc comment on each
//! says which. The page-margin locator in [`crate::flat`] is the one caller of
//! the new one.
//!
//! Where in the pipeline these are called is [`crate::flat`]'s business, not
//! this module's - the same separation [`strong_lines`] and
//! [`crate::content`] already have.

use crate::{Luma, Rect, Tuning};

/// A run of consecutive strong profile indices, merged into one line.
///
/// Both bounds are **inclusive** indices into the profile the line was found
/// in, so `Line { start: 40, end: 40 }` is one strong index rather than an
/// empty range. A profile index `i` sits between lines `i` and `i + 1` of the
/// rect it was measured over; turning that back into a pixel coordinate is the
/// caller's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    /// First strong index of the run.
    pub start: usize,
    /// Last strong index of the run, inclusive.
    pub end: usize,
}

/// The mean absolute luma difference between each adjacent pair of **rows**
/// inside `rect`, top to bottom.
///
/// `rect.h.saturating_sub(1)` entries: a rect one row tall, or none at all,
/// has no adjacent pair and yields an empty profile. Only pixels inside the
/// rect are read, so a bright band outside it changes nothing.
#[must_use]
pub fn row_profile(img: &Luma, rect: Rect) -> Vec<f32> {
    let stride = img.width as usize;
    let (x, y) = (rect.x as usize, rect.y as usize);
    let (w, h) = (rect.w as usize, rect.h as usize);

    (0..h.saturating_sub(1))
        .map(|i| {
            let upper = (y + i) * stride + x;
            let lower = upper + stride;
            let total: u64 = (0..w)
                .map(|dx| difference(img.data[upper + dx], img.data[lower + dx]))
                .sum();
            mean(total, w)
        })
        .collect()
}

/// The mean absolute luma difference between each adjacent pair of **columns**
/// inside `rect`, left to right.
///
/// `rect.w.saturating_sub(1)` entries, by the mirror of [`row_profile`]'s
/// rule.
#[must_use]
pub fn col_profile(img: &Luma, rect: Rect) -> Vec<f32> {
    let stride = img.width as usize;
    let (x, y) = (rect.x as usize, rect.y as usize);
    let (w, h) = (rect.w as usize, rect.h as usize);

    (0..w.saturating_sub(1))
        .map(|j| {
            let left = x + j;
            let total: u64 = (0..h)
                .map(|dy| {
                    let row = (y + dy) * stride;
                    difference(img.data[row + left], img.data[row + left + 1])
                })
                .sum();
            mean(total, h)
        })
        .collect()
}

/// Every run of consecutive indices of `profile` at or above
/// `t.edge_threshold`, in ascending index order.
///
/// "At or above" is `>=`: a value of exactly the threshold is strong. Runs are
/// merged by adjacency alone, so a single weak index splits one run into two
/// lines, and a run still open when the profile ends is closed at its last
/// index.
#[must_use]
pub fn strong_lines(profile: &[f32], t: &Tuning) -> Vec<Line> {
    // The only place the threshold is read, and it is read from the argument.
    // `u8` to `f32` is exact, and the profile is deliberately not rounded to
    // an integer to meet it.
    let threshold = f32::from(t.edge_threshold);
    let mut lines = Vec::new();
    let mut open: Option<usize> = None;

    for (i, &value) in profile.iter().enumerate() {
        match (value >= threshold, open) {
            (true, None) => open = Some(i),
            (false, Some(start)) => {
                lines.push(Line { start, end: i - 1 });
                open = None;
            }
            // Continuing a run, or continuing to be below the threshold.
            (true, Some(_)) | (false, None) => {}
        }
    }
    if let Some(start) = open {
        // The profile ended mid-run: close it at the last index rather than
        // dropping it.
        lines.push(Line {
            start,
            end: profile.len() - 1,
        });
    }

    lines
}

/// Which way a profile runs.
///
/// A plain value type with no data: [`spread_profile`] needs to be told which
/// lines to measure, and `Rows` / `Columns` is the whole of that argument.
/// [`row_profile`] and [`col_profile`] predate it and keep their own names,
/// which MC-004's tests pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Lines running left to right, indexed top to bottom.
    Rows,
    /// Lines running top to bottom, indexed left to right.
    Columns,
}

/// The mean absolute deviation of each line of `img` about **its own mean**,
/// one entry per line, in the axis's natural order.
///
/// `img.height` entries on [`Axis::Rows`] and `img.width` on
/// [`Axis::Columns`]; an empty plane yields an empty profile on both. See the
/// module documentation for why this is the statistic and why the count is one
/// per line rather than one per adjacent pair.
///
/// It measures the whole plane and takes no rect, which is MC-025 AC-1
/// verbatim. Callers inside the crate that need the spread of a line clipped
/// to a rect use [`spread_within`], which is what this is a thin wrapper over.
#[must_use]
pub fn spread_profile(img: &Luma, axis: Axis) -> Vec<f32> {
    spread_within(
        img,
        Rect {
            x: 0,
            y: 0,
            w: img.width,
            h: img.height,
        },
        axis,
    )
}

/// [`spread_profile`] over `rect` instead of over the whole plane: one entry
/// per line of the rect, measured only over the pixels inside it.
///
/// Crate-private on purpose. AC-1 fixes the public signature at two arguments,
/// and the pipeline needs the same statistic over the rect the earlier stages
/// left, so the rect-taking form is the one with the narrower audience.
pub(crate) fn spread_within(img: &Luma, rect: Rect, axis: Axis) -> Vec<f32> {
    let stride = img.width as usize;
    let (x, y) = (rect.x as usize, rect.y as usize);
    let (w, h) = (rect.w as usize, rect.h as usize);

    match axis {
        Axis::Rows => (0..h)
            .map(|dy| {
                let start = (y + dy) * stride + x;
                let line = &img.data[start..start + w];
                deviation(w, |i| line[i])
            })
            .collect(),
        Axis::Columns => (0..w)
            .map(|dx| {
                let column = x + dx;
                deviation(h, |i| img.data[(y + i) * stride + column])
            })
            .collect(),
    }
}

/// The first and last index of `spread` at or above `t.min_line_spread`, both
/// **inclusive**, or `None` when no index reaches it.
///
/// "At or above" is `>=`, exactly as it is for
/// [`edge_threshold`](Tuning::edge_threshold) in [`strong_lines`]. The span
/// reaches the *outermost* textured indices and is not ended by a flat run
/// between them: see the module documentation, and MC-005's decision 13.
///
/// For a spread profile an index **is** a line index, which is the other half
/// of what makes this locator comparable with the gradient one - a
/// [`Line`]'s index `i` sits *between* lines `i` and `i + 1` and has to be
/// converted, and this one does not.
#[must_use]
pub fn textured_span(spread: &[f32], t: &Tuning) -> Option<(usize, usize)> {
    // The only place the threshold is read, and it is read from the argument.
    let threshold = t.min_line_spread;
    let textured = |&value: &f32| value >= threshold;
    // Both ends found independently, from their own end. A scan that stopped
    // at the first flat index after the first textured one would cut at a
    // panel gutter; searching backwards for the last one cannot.
    let first = spread.iter().position(textured)?;
    let last = spread.iter().rposition(textured)?;
    Some((first, last))
}

/// The **widest** run of consecutive indices of `spread` at or above
/// `t.min_line_spread`, both bounds **inclusive**, or `None` when no index
/// reaches it.
///
/// The other decision over the same profile, and the contrast with
/// [`textured_span`] is the whole of it: where the span reaches the outermost
/// textured indices *across* any flat run between them, this stops at the flat
/// run and returns the widest single stretch of texture. On the profile
/// `[0, 20, 0, 0, 30, 30, 30, 0]` the span is `(1, 6)` and this is `(4, 6)`.
///
/// Both are right, for different questions. `textured_span` answers "where
/// does the art in this rect begin and end", where a flat run between two
/// textured ones is a panel gutter and MC-005's decision 13 keeps the panels.
/// This answers "which stretch of texture is the page", where a second
/// textured region beside the page - a sidebar, a second column of browser
/// furniture - is not the page and must not be annexed. MC-026 measured the
/// two on the corpus's column axis at 19 of 21 for this rule against 15 for
/// the span, and identically on the row axis.
///
/// "At or above" is `>=`, read from the argument at the one comparison site,
/// exactly as in [`strong_lines`] and [`textured_span`]. A **tie is broken by
/// taking the first run**: nothing on the corpus turns on it, and it is fixed
/// so the answer is the same on every machine.
#[must_use]
pub fn widest_textured_run(spread: &[f32], t: &Tuning) -> Option<(usize, usize)> {
    // The only place the threshold is read, and it is read from the argument.
    let threshold = t.min_line_spread;
    let mut widest: Option<(usize, usize)> = None;
    let mut open: Option<usize> = None;

    for (i, &value) in spread.iter().enumerate() {
        match (value >= threshold, open) {
            (true, None) => open = Some(i),
            (false, Some(start)) => {
                widest = wider(widest, (start, i - 1));
                open = None;
            }
            // Continuing a run, or continuing to be below the threshold.
            (true, Some(_)) | (false, None) => {}
        }
    }
    if let Some(start) = open {
        // The profile ended mid-run: close it at the last index, as
        // `strong_lines` does, rather than dropping it.
        widest = wider(widest, (start, spread.len() - 1));
    }

    widest
}

/// `run` if it is **strictly** wider than `widest`, otherwise `widest`.
///
/// The strictness is the tie-break: runs are offered in ascending index order,
/// so a later run of equal width leaves the earlier one in place.
fn wider(widest: Option<(usize, usize)>, run: (usize, usize)) -> Option<(usize, usize)> {
    match widest {
        Some(best) if best.1 - best.0 >= run.1 - run.0 => Some(best),
        _ => Some(run),
    }
}

/// The mean absolute deviation of `count` samples about their own mean.
///
/// # Arithmetic
///
/// Taken as one exact rational rather than as two floating passes. With
/// `total` the integer sum of the line, `n * x_i - total` is an integer, and
/// `MAD = sum(|n * x_i - total|) / n^2` is that sum of integers over an
/// integer count - so the only rounding in the whole statistic is the final
/// division. That is what makes the exact cases exact under any width: an art
/// row of the fade fixture measures its amplitude to the bit, a chrome band's
/// column measures 0.0, and a caller can compare with `==` instead of
/// inventing a tolerance.
///
/// The widths are not thrift either. `n * x_i` reaches `65535 * 255`, the sum
/// of `n` of them `n^2 * 255`, which is 1.1e12 for a 65535-line image: past
/// `u32` and comfortable in `u64`. The division is done in `f64` because
/// `n^2` itself stops being exactly representable in `f32` at 4096 lines,
/// which a tall webtoon page passes, and the quotient is narrowed once at the
/// end - [`Tuning::min_line_spread`] is an `f32` and the profile is compared
/// against it.
fn deviation(count: usize, sample: impl Fn(usize) -> u8) -> f32 {
    if count == 0 {
        // A line with no pixels deviates from nothing. Only reachable for a
        // rect that is zero-wide or zero-tall on the other axis, which no
        // pipeline stage produces; the guard is here so the division below
        // never sees a zero.
        return 0.0;
    }
    let n = count as u64;
    let total: u64 = (0..count).map(|i| u64::from(sample(i))).sum();
    let deviations: u64 = (0..count)
        .map(|i| (n * u64::from(sample(i))).abs_diff(total))
        .sum();
    (deviations as f64 / (n * n) as f64) as f32
}

/// Absolute difference of two luma samples, widened so the sum of a whole
/// line's worth cannot overflow.
fn difference(a: u8, b: u8) -> u64 {
    u64::from(a.abs_diff(b))
}

/// `total / count` as an `f32`. Both are exact for every image that fits in
/// memory, so the quotient is as close to the true mean as `f32` allows - and
/// is the exactly representable integer when the true mean is one.
fn mean(total: u64, count: usize) -> f32 {
    total as f32 / count as f32
}
