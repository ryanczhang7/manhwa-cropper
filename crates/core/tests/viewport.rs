//! MC-048: the viewport stage, asked directly, on the generated screenshots
//! `tests/viewport_detect.rs` runs through `detect`.
//!
//! This file pins the stage's public surface - [`cropper_core::viewport`],
//! [`locate`] and [`Viewport`] - and until MC-048 adds that module it does not
//! compile. It is a separate target from `tests/viewport_detect.rs` so that
//! the AC-6 and AC-7 assertions there can run, and fail on their own terms,
//! before it does.
//!
//! # The stage's contract, as pinned here
//!
//! `locate(img, column, t)` reads the pixels of `img` **outside** `column`'s
//! columns, over the full image width, and returns the browser viewport's
//! rows - `top`, its first page row, and `bottom`, the first row of the
//! taskbar, exclusive - or `None` when it cannot locate one. The statistic and
//! selector are MC-031's chrome oracle as `docs/wiki/chrome-row-search.md`
//! section 3b and section 4 scored it: *Bg* at threshold 0.90, *ChromeStrip*,
//! minimum run 16, gap 0, the full margin width, `uniform_tolerance` read from
//! `t`. The rows expected below are that oracle's own answers on these
//! fixtures, measured in RED by MC-031's harness; see the story's `## Handoff`.

mod common;

use common::{Bands, PAGE_W, viewport_scene};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::viewport::{Viewport, locate};
use cropper_core::{Luma, Rect, Tuning};

/// The page column `detect` hands the stage: its first five stages, composed
/// from their public functions in `decide.rs`'s order, before the margin.
fn page_column_of(img: &Luma, t: &Tuning) -> Rect {
    let first = trim_uniform(img, t).expect("not one flat colour");
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t).expect("not one flat colour");
    page_column(img, textured_box(img, second, t), t)
}

/// `locate` over the column the pipeline would hand it, at `t`.
fn located(img: &Luma, t: &Tuning) -> Option<Viewport> {
    locate(img, page_column_of(img, t), t)
}

/// AC-6's scene: 40-row bands above and below 160 rows of page. The viewport
/// is the page rows exactly.
#[test]
fn the_viewport_is_the_rows_between_the_tab_strip_and_the_taskbar() {
    let img = viewport_scene(40, 40, Bands::Textured, None);
    assert_eq!(
        located(&img, &Tuning::default()),
        Some(Viewport {
            top: 40,
            bottom: 200
        })
    );
}

/// AC-7's scene, with unequal bands of 24 and 32 rows.
#[test]
fn unequal_bands_give_their_own_viewport_rows() {
    let img = viewport_scene(24, 32, Bands::Textured, None);
    assert_eq!(
        located(&img, &Tuning::default()),
        Some(Viewport {
            top: 24,
            bottom: 184
        })
    );
}

/// AC-6's decline: the page margin broken every 16 page rows leaves no run of
/// 16 page-like rows, and the stage says so rather than guessing.
#[test]
fn the_stage_declines_when_no_run_of_sixteen_page_rows_exists() {
    let img = viewport_scene(40, 40, Bands::Textured, Some(16));
    assert_eq!(located(&img, &Tuning::default()), None);
}

/// The control on the decline: broken every 17 rows, runs of 16 exist and the
/// stage speaks. The last page run, 7 rows below the final break, is shorter
/// than 16 and is read as part of the taskbar strip - which is what
/// *ChromeStrip* does with a short page-like run next to the edge band.
#[test]
fn runs_of_sixteen_page_rows_are_enough_to_locate_the_viewport() {
    let img = viewport_scene(40, 40, Bands::Textured, Some(17));
    assert_eq!(
        located(&img, &Tuning::default()),
        Some(Viewport {
            top: 40,
            bottom: 192
        })
    );
}

/// The empty case: a column spanning the whole width leaves no pixel beside it
/// to read, and the stage declines. This is the case every older fixture in
/// this crate whose art runs edge to edge presents.
#[test]
fn a_column_with_no_margin_beside_it_locates_nothing() {
    let img = viewport_scene(40, 40, Bands::Textured, None);
    let whole = Rect {
        x: 0,
        y: 0,
        w: PAGE_W,
        h: img.height,
    };
    assert_eq!(locate(&img, whole, &Tuning::default()), None);
}

/// `uniform_tolerance` is read from the `Tuning` the stage is given. At 0 the
/// page margin's speckle and the fades' faint art put every page row near 0.81
/// flat, under the 0.90 threshold, and the stage declines; at the default 10
/// the same rows are 0.96 and it locates. The column is the one located at the
/// default, so only the stage sees the change.
#[test]
fn the_stage_reads_uniform_tolerance_from_the_tuning() {
    let img = viewport_scene(40, 40, Bands::Textured, None);
    let column = page_column_of(&img, &Tuning::default());
    let strict = Tuning {
        uniform_tolerance: 0,
        ..Tuning::default()
    };
    assert_eq!(locate(&img, column, &strict), None);
    assert!(locate(&img, column, &Tuning::default()).is_some());
}
