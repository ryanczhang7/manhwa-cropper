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
