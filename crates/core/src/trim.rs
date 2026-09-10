//! Stage 1 of the pipeline: drop uniform solid borders from all four sides
//! (`docs/wiki/architecture.md`, "cropper-core", step 1).
//!
//! The rule, settled with the user on 2026-09-10 (MC-003): a row or column
//! segment, measured *within the current rect*, is uniform iff
//! `max - min <= Tuning::uniform_tolerance`. There is no reference colour, no
//! median and no mean, so a textured art row - one holding two pixels further
//! apart than the tolerance - can never be judged uniform.
//!
//! Trimming iterates. Each side drops its outermost uniform lines, and the
//! four sides are revisited until none of them moves. That is what makes
//! full-width top/bottom bands and full-height left/right gutters both work
//! without the detector being told which it has: under `Gutters`, the top row
//! also spans both gutter colours and is not uniform until the gutters have
//! gone.

use crate::{Luma, Rect, Tuning};

/// Which edge of the current rect a scan is taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// Visiting order is not load-bearing: the loop below runs to a fixed point,
/// and every order reaches the same one.
const SIDES: [Side; 4] = [Side::Top, Side::Bottom, Side::Left, Side::Right];

/// The largest rect of `img` whose four edges are not uniform, or `None` when
/// nothing survives - a wholly uniform image has no art to find.
///
/// A zero-sized `Luma` also yields `None`; MC-003 leaves that unconstrained,
/// and it falls out of the emptiness check rather than needing a guard.
#[must_use]
pub fn trim_uniform(img: &Luma, t: &Tuning) -> Option<Rect> {
    let mut rect = Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    };
    loop {
        let mut changed = false;
        for side in SIDES {
            // Drain the side greedily, then check emptiness *before* moving
            // to the next one: a wholly uniform image loses all its rows here,
            // and a column scan over a zero-height rect has no min or max.
            while !is_empty(rect) && is_uniform(img, rect, side, t.uniform_tolerance) {
                rect = shrink(rect, side);
                changed = true;
            }
            if is_empty(rect) {
                return None;
            }
        }
        if !changed {
            return Some(rect);
        }
    }
}

fn is_empty(rect: Rect) -> bool {
    rect.w == 0 || rect.h == 0
}

/// Move `side`'s edge one pixel inward. The rect must not be empty.
fn shrink(rect: Rect, side: Side) -> Rect {
    match side {
        Side::Top => Rect {
            y: rect.y + 1,
            h: rect.h - 1,
            ..rect
        },
        Side::Bottom => Rect {
            h: rect.h - 1,
            ..rect
        },
        Side::Left => Rect {
            x: rect.x + 1,
            w: rect.w - 1,
            ..rect
        },
        Side::Right => Rect {
            w: rect.w - 1,
            ..rect
        },
    }
}

/// Whether `side`'s outermost line, clipped to `rect`, spans at most
/// `tolerance` levels.
fn is_uniform(img: &Luma, rect: Rect, side: Side, tolerance: u8) -> bool {
    let (min, max) = span(img, rect, side);
    max - min <= tolerance
}

/// The darkest and lightest sample on `side`'s outermost line, looking only at
/// the part of it inside `rect`. The rect must not be empty, so the line holds
/// at least one pixel.
fn span(img: &Luma, rect: Rect, side: Side) -> (u8, u8) {
    let stride = img.width as usize;
    let (x, y) = (rect.x as usize, rect.y as usize);
    let (w, h) = (rect.w as usize, rect.h as usize);
    let mut min = u8::MAX;
    let mut max = u8::MIN;

    match side {
        Side::Top | Side::Bottom => {
            let row = if side == Side::Top { y } else { y + h - 1 };
            let start = row * stride + x;
            for &px in &img.data[start..start + w] {
                min = min.min(px);
                max = max.max(px);
            }
        }
        Side::Left | Side::Right => {
            let column = if side == Side::Left { x } else { x + w - 1 };
            for row in img.data[y * stride..(y + h) * stride].chunks(stride) {
                min = min.min(row[column]);
                max = max.max(row[column]);
            }
        }
    }

    (min, max)
}
