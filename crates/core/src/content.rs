//! Stage 3 of the pipeline: peel browser and reader chrome off the edges of
//! the rect the earlier stages left (`docs/wiki/architecture.md`,
//! "cropper-core", step 3, and decision 13).
//!
//! [`trim_uniform`](crate::trim::trim_uniform) drops borders that are *one
//! flat colour*. Real chrome is not: a tab strip carries tab titles, a URL bar
//! carries a URL, a sidebar carries a chapter list. So the rule here is
//! statistical where stage 1's is exact. On each side of the current rect the
//! **outermost strip** is the band between that edge and the nearest strong
//! line (MC-004) parallel to it, and it is **chrome-like** when all three
//! hold:
//!
//! 1. its extent is at most [`Tuning::chrome_max_extent`] of the **image's**
//!    dimension on that axis, not of the current rect's. A toolbar is a small
//!    share of a screenshot however much has already been peeled off around
//!    it; taking the share of the shrinking rect would let a long enough chain
//!    of removals eat into the art a sliver at a time;
//! 2. its **flat fraction** - the share of its pixels within
//!    [`Tuning::uniform_tolerance`] of the strip's *median* luma - is at least
//!    [`Tuning::chrome_flat_fraction`]. The median rather than the mean
//!    because dark text drags the mean off the background it is written on,
//!    and it is the background that decides whether the strip is chrome;
//! 3. what is left after removing it still has luma standard deviation at or
//!    above [`Tuning::min_content_stddev`]. Chrome is never peeled off an
//!    image with no content left to keep: that image is a detection failure
//!    for MC-007 to flag, not a crop.
//!
//! Chrome-like strips are removed **one per side per pass**, sides visited
//! `Top, Bottom, Left, Right`, and the passes repeat until one of them removes
//! nothing. Stacked chrome - a tab strip above a bookmarks bar - therefore
//! takes one pass per band. Draining a single side greedily before moving on
//! would reach the same rect by a different route, and [`ContentBox::removed`]
//! is where the difference shows, so the rule is pinned rather than left to
//! whichever loop shape came first.
//!
//! What is *not* peeled matters as much (decision 13). Only a strip touching
//! an edge is ever a candidate, so a flat panel border in the middle of a
//! multi-panel page stays and the page keeps all its panels instead of being
//! cut down to the largest region the brief's literal wording would pick.
//!
//! A strip that is *otherwise* a chrome candidate - inside the extent limit,
//! and leaving content behind - and misses on flatness alone by less than
//! [`Tuning::ambiguity_band`] is kept, and marks the result
//! [`ambiguous`](ContentBox::ambiguous) for MC-007 to turn into a flag. That
//! qualification is deliberate and is settled in MC-005's `## Notes`: read
//! without it, ordinary art covering half the image would flag every
//! screenshot it appeared in, while `architecture.md` describes the flag as
//! "an edge strip was **nearly** chrome-like".
//!
//! # Arithmetic
//!
//! Both shares are compared against `f32` fields of [`Tuning`], and both are
//! computed in `f32` for that reason rather than out of thrift: `510f32 /
//! 600f32` is bit for bit `0.85f32`, so a strip of exactly the threshold is
//! chrome, whereas the same quotient taken in `f64` lands one ulp below
//! `f64::from(0.85f32)` and the boundary moves without anyone touching a
//! constant. The extent test is a division for a related reason - `30f32 /
//! 100f32` is exactly `0.30f32`, while `0.30f32 * 100f32` is 30.000001 and
//! leaves `<` and `<=` indistinguishable there, which is to say untestable.
//!
//! The standard deviation is a population one, taken in `f64`. Nothing pins
//! that choice: every fixture is far from the threshold on either convention.

use crate::edges::{col_profile, row_profile, strong_lines};
use crate::{Luma, Rect, Tuning};

/// Which edge of a rect a strip was taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
}

/// The four sides, in the order a pass visits them.
///
/// Load-bearing here: [`ContentBox::removed`] records the order strips were
/// peeled in, so for an image whose four sides are all chrome this array *is*
/// the answer. [`crate::trim`] reuses it because it needs some order and this
/// one is as good as another there.
pub(crate) const SIDES: [Side; 4] = [Side::Top, Side::Bottom, Side::Left, Side::Right];

/// What [`content_box`] found: the rect, how it got there, and whether it had
/// to make a close call on the way.
#[derive(Debug)]
pub struct ContentBox {
    /// The content box: the rect `content_box` was given, less every chrome
    /// strip that was peeled off it.
    pub rect: Rect,
    /// The sides strips were taken from, in the order they were taken. One
    /// entry per strip, so stacked chrome on one side appears twice.
    pub removed: Vec<Side>,
    /// Whether some strip was *nearly* chrome-like: a candidate on extent and
    /// on the content it would leave behind, missing on flatness alone by less
    /// than [`Tuning::ambiguity_band`]. It was kept; this says the call was
    /// close. MC-007 turns it into a flag.
    pub ambiguous: bool,
}

/// The content box of `img` inside `within`: everything that is not chrome.
///
/// `within` is where the search *starts*, not a hint - a strong line outside
/// it is never looked at - so the earlier stages' output flows straight in.
/// The rule, and why it is this rule rather than "the largest region", is in
/// the module documentation above.
#[must_use]
pub fn content_box(img: &Luma, within: Rect, t: &Tuning) -> ContentBox {
    let mut rect = within;
    let mut removed = Vec::new();
    let mut ambiguous = false;

    loop {
        let mut peeled = false;
        // One strip per side per pass: each side is judged exactly once here,
        // against whatever the sides before it left. A second band on the same
        // side waits for the next pass.
        for side in SIDES {
            match judge(img, rect, side, t) {
                Verdict::Chrome(remainder) => {
                    rect = remainder;
                    removed.push(side);
                    peeled = true;
                }
                Verdict::NearlyChrome => ambiguous = true,
                Verdict::Content => {}
            }
        }
        if !peeled {
            // A pass that removes nothing would remove nothing next time
            // either, and every removal shrinks the rect, so this terminates.
            return ContentBox {
                rect,
                removed,
                ambiguous,
            };
        }
    }
}

/// What one side offered on one pass.
enum Verdict {
    /// The strip is chrome. This is what is left of the rect without it.
    Chrome(Rect),
    /// The strip is a chrome candidate that missed on flatness alone, by less
    /// than the ambiguity band. It stays, and the result is ambiguous.
    NearlyChrome,
    /// There is no strip on this side, or there is one and it is content.
    Content,
}

/// Judge the outermost strip on `side` of `rect` against the three conditions.
fn judge(img: &Luma, rect: Rect, side: Side, t: &Tuning) -> Verdict {
    let Some(depth) = strip_depth(img, rect, side, t) else {
        return Verdict::Content;
    };
    let (strip, remainder) = split(rect, side, depth);

    // Extent and remaining content are tested first and flatness last, because
    // that ordering is what qualifies the ambiguity band: a strip too big to
    // be chrome, or one whose removal would leave nothing, is not "nearly
    // chrome-like" however flat it is, and must not reach the band below.
    let dimension = match side {
        Side::Top | Side::Bottom => img.height,
        Side::Left | Side::Right => img.width,
    };
    if depth as f32 / dimension as f32 > t.chrome_max_extent {
        return Verdict::Content;
    }
    if stddev(img, remainder) < f64::from(t.min_content_stddev) {
        return Verdict::Content;
    }

    let flat = flat_fraction(img, strip, t.uniform_tolerance);
    if flat >= t.chrome_flat_fraction {
        Verdict::Chrome(remainder)
    } else if flat >= t.chrome_flat_fraction - t.ambiguity_band {
        Verdict::NearlyChrome
    } else {
        Verdict::Content
    }
}

/// How many lines deep the outermost strip on `side` runs, or `None` when no
/// strong line inside `rect` is parallel to that edge - a side with no line
/// has no strip, and nothing there can be peeled.
///
/// The profile is measured over `rect`, so a strong line outside it is
/// invisible and the search genuinely starts from the rect it was given.
/// Profile index `i` sits between lines `i` and `i + 1` of the rect, and the
/// strip stops at the run's **outer** bound - the end nearest the edge - so a
/// Top strip whose run starts at `i` is `i + 1` lines deep and a Bottom strip
/// whose run ends at `i` is `h - i - 1` deep. A run is wider than one index
/// only where a distinct seam line sits between the chrome and the art, and
/// taking the outer bound is what keeps that seam with the content.
fn strip_depth(img: &Luma, rect: Rect, side: Side, t: &Tuning) -> Option<u32> {
    let (profile, extent) = match side {
        Side::Top | Side::Bottom => (row_profile(img, rect), rect.h),
        Side::Left | Side::Right => (col_profile(img, rect), rect.w),
    };
    let lines = strong_lines(&profile, t);
    match side {
        Side::Top | Side::Left => Some(lines.first()?.start as u32 + 1),
        Side::Bottom | Side::Right => Some(extent - lines.last()?.end as u32 - 1),
    }
}

/// The strip `depth` lines deep on `side` of `rect`, and what is left of
/// `rect` without it.
///
/// `depth` never reaches the rect's extent on that axis - a profile of `n`
/// lines has `n - 1` entries, and a strip stops at one of them - so the
/// remainder always holds at least one line and the statistics below always
/// have something to measure.
fn split(rect: Rect, side: Side, depth: u32) -> (Rect, Rect) {
    match side {
        Side::Top => (
            Rect { h: depth, ..rect },
            Rect {
                y: rect.y + depth,
                h: rect.h - depth,
                ..rect
            },
        ),
        Side::Bottom => (
            Rect {
                y: rect.y + rect.h - depth,
                h: depth,
                ..rect
            },
            Rect {
                h: rect.h - depth,
                ..rect
            },
        ),
        Side::Left => (
            Rect { w: depth, ..rect },
            Rect {
                x: rect.x + depth,
                w: rect.w - depth,
                ..rect
            },
        ),
        Side::Right => (
            Rect {
                x: rect.x + rect.w - depth,
                w: depth,
                ..rect
            },
            Rect {
                w: rect.w - depth,
                ..rect
            },
        ),
    }
}

/// The share of `rect`'s pixels lying within `tolerance` of its median luma,
/// inclusive at the tolerance itself.
///
/// One pass builds a 256-bin histogram and both the median and the count come
/// out of it: sorting the strip would allocate a copy of it for nothing, and
/// there are only ever 256 distinct luma values to count.
fn flat_fraction(img: &Luma, rect: Rect, tolerance: u8) -> f32 {
    let mut histogram = [0u64; 256];
    for row in rows(img, rect) {
        for &px in row {
            histogram[px as usize] += 1;
        }
    }
    let total = u64::from(rect.w) * u64::from(rect.h);
    let centre = median(&histogram, total);
    // Saturating, so a median near either end of the range clips the window
    // instead of wrapping it round to the other end.
    let lo = usize::from(centre.saturating_sub(tolerance));
    let hi = usize::from(centre.saturating_add(tolerance));
    let flat: u64 = histogram[lo..=hi].iter().sum();
    // `f32`, deliberately: see the module note on arithmetic.
    flat as f32 / total as f32
}

/// The luma at sorted position `total / 2`, which is the upper median when the
/// pixel count is even.
///
/// Which median an even count gets is not pinned by anything - every strip the
/// detector has been shown has the same lower and upper median - but it has to
/// be *a* convention, and this is the one the fixtures are checked against.
fn median(histogram: &[u64; 256], total: u64) -> u8 {
    let half = total / 2;
    let mut seen = 0;
    // The first luma whose cumulative count passes the halfway mark. `total`
    // is non-zero for every strip that can reach here, so the search always
    // finds one.
    histogram
        .iter()
        .position(|&count| {
            seen += count;
            seen > half
        })
        .unwrap_or(0) as u8
}

/// Population standard deviation of `rect`'s luma.
///
/// Two passes rather than the sum-of-squares shortcut: the shortcut subtracts
/// two large and nearly equal numbers, and telling "nearly flat" from
/// "textured" is the entire job of this number.
fn stddev(img: &Luma, rect: Rect) -> f64 {
    let n = f64::from(rect.w) * f64::from(rect.h);
    let total: u64 = rows(img, rect).flatten().map(|&px| u64::from(px)).sum();
    let mean = total as f64 / n;
    let variance = rows(img, rect)
        .flatten()
        .map(|&px| (f64::from(px) - mean).powi(2))
        .sum::<f64>()
        / n;
    variance.sqrt()
}

/// The rows of `rect`, as slices borrowed straight out of `img`.
fn rows(img: &Luma, rect: Rect) -> impl Iterator<Item = &[u8]> {
    let stride = img.width as usize;
    let (x, y) = (rect.x as usize, rect.y as usize);
    let (w, h) = (rect.w as usize, rect.h as usize);

    (0..h).map(move |dy| {
        let start = (y + dy) * stride + x;
        &img.data[start..start + w]
    })
}
