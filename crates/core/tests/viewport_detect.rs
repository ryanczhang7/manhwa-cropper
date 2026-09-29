//! MC-048, AC-6 and AC-7: the viewport stage, seen through `detect` on
//! generated screenshots. This is what the required `unit` gate sees of the
//! story; the corpus half is `crates/engine/tests/corpus_viewport*.rs`.
//!
//! Everything here goes through [`detect`] and the stages that already exist,
//! so the file compiles against the tree before MC-048 and its assertions fail
//! there for the reason the story names. The stage's own answer, asked
//! directly, is `tests/viewport.rs`.
//!
//! # The fixtures
//!
//! [`viewport_scene`] (in `common/mod.rs`): MC-027's page between two flat page
//! margins, 300 px wide, with full-width **noise** bands above and below it -
//! the browser chrome and the taskbar. Why noise rather than the stripes
//! `page_in_margins_with_chrome_bands` uses: a stripe row is one value edge to
//! edge, which is a uniform line, and stage 1 trims a band of them away before
//! the viewport stage runs, so a test on it would pass on the old tree.
//! [`the_bands_reach_the_viewport_stage_untouched`] pins that premise.
//!
//! # What is invented, and the control on each side of it
//!
//! AC-6's "cannot locate a viewport" has no oracle; the story says to invent
//! it. It is invented on the settled minimum run: the page margin is broken by
//! a row of noise every 16th page row, so the longest run of flat-margin page
//! rows is 15 and no viewport can be located. The near twin breaks it every
//! 17th row instead - runs of 16 - and there the viewport *is* located and the
//! tab strip removed. The two fixtures differ in that one number.
//!
//! Every number below was measured on this tree in RED, and the stage's own
//! values by MC-031's harness driving the same fixtures; the story's
//! `## Handoff` has the table.

mod common;

use common::{Bands, VIEW_BODY, viewport_scene, viewport_scene_art};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::{Luma, Rect, Tuning, detect};

/// AC-6's bands: 40 rows each, the outer sixths of a 240-row scene.
const BAND: u32 = 40;

/// AC-7's bands: deliberately unequal, and not the outer sixths, so the rows it
/// pins cannot be a coincidence of AC-6's geometry.
const AC7_TOP: u32 = 24;
const AC7_BOTTOM: u32 = 32;

/// The decline fixture's break period: the longest run of page-like rows is
/// one short of the settled minimum run of 16.
const DECLINES_EVERY: u32 = 16;

/// Its near twin's: runs of exactly 16.
const LOCATES_EVERY: u32 = 17;

/// The crop `detect` made of each fixture on `cb2deef`, before this story, at
/// `Tuning::default()`. Measured in RED.
///
/// **Pinned at `margin_px` 3, explicitly (MC-049, 2026-09-28).** These rects
/// were measured when the default margin was 3, and the tests below run at
/// [`at_margin_3`], not at `Tuning::default()`: MC-049 moved the default to 0,
/// and what these identities pin is that the viewport stage moves nothing,
/// which is a claim about the stage and not about the margin.
///
/// **Columns re-pointed by MC-053, approved by the Lead PO on 2026-09-25**;
/// the rows are untouched. On `cb2deef` both were `x 44, w 212`: the page
/// column 47..=252 grown by `margin_px` 3. MC-053 AC-7 case A requires the
/// page column to keep art below `min_line_spread` that is not page
/// background, and in every `viewport_scene` the fade's columns of amplitude
/// 6 and 7 (45, 46, 253, 254) are exactly that - share 0.50 over the body
/// rows - while the fade's columns of amplitude 1..=5 (40..=44, 255..=259)
/// are page background, share 1.00 (`common::page_column_is_background`).
/// So the page column is 45..=254 on both scenes, and grown by 3 it is
/// 42..=257: `x 42, w 216`. On the decline scene the outer margin (0..=39)
/// reads a share below 0.95 over the body rows, because every 16th row of it is
/// noise; the page background that bounds the page column there is the
/// fade's 40..=44 and 255..=259.
const BLANK_BEFORE: Rect = Rect {
    x: 42,
    y: 37,
    w: 216,
    h: 166,
};
const DECLINE_BEFORE: Rect = Rect {
    x: 42,
    y: 0,
    w: 216,
    h: 240,
};

/// AC-7's columns, before and after alike: the page column grown by
/// `margin_px` at 3 and not at all at 0. The viewport stage moves rows only.
///
/// **Re-pointed by MC-053, approved by the Lead PO on 2026-09-25.** On
/// `cb2deef` the page column here was 48..=251, one column narrower each side
/// than on the 240-row scenes, because this scene's central band is 129 rows,
/// an odd count, and an alternating column of amplitude 8 then reads a spread
/// just under 8.0. By MC-053's definition the page column is every column that is
/// not page background over the body rows, 45..=254, as in
/// [`BLANK_BEFORE`]'s derivation: `(45, 210)` at margin 0 and `(42, 216)` at
/// margin 3. Were `(48, 204)` and `(45, 210)`.
const AC7_COLUMNS_AT_3: (u32, u32) = (42, 216);
const AC7_COLUMNS_AT_0: (u32, u32) = (45, 210);

/// The default tuning at the margin [`BLANK_BEFORE`] and [`DECLINE_BEFORE`]
/// were measured at.
fn at_margin_3() -> Tuning {
    Tuning {
        margin_px: 3,
        ..Tuning::default()
    }
}

/// The rect `detect` returns for `img` at `t`.
fn crop(img: &Luma, t: &Tuning) -> Rect {
    detect(img, t)
        .expect("a scene with a page in it is not one flat colour")
        .rect
}

/// The crop's first and last row, both inclusive.
fn rows(rect: Rect) -> (u32, u32) {
    (rect.y, rect.y + rect.h - 1)
}

/// Whether `outer` contains `inner` entirely.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

// --- The premise ------------------------------------------------------------

/// The fixture's premise, which holds before this story and after it: the
/// stages that already exist leave both bands in the rect. Without it the
/// AC-6 assertion below could pass because stage 1 or the chrome peel removed
/// the bands, and would prove nothing about the viewport stage.
#[test]
fn the_bands_reach_the_viewport_stage_untouched() {
    let t = Tuning::default();
    let img = viewport_scene(BAND, BAND, Bands::Textured, None);
    let first = trim_uniform(&img, &t).expect("not uniform");
    let found = content_box(&img, first, &t);
    let second = trim_within(&img, found.rect, &t).expect("not uniform");
    let column = page_column(&img, textured_box(&img, second, &t), &t);

    assert_eq!(
        (column.y, column.h, found.removed.len()),
        (0, img.height, 0),
        "no earlier stage may touch the bands: the rows reaching the viewport stage \
         must be the whole image, with nothing peeled"
    );
    assert!(
        column.x > 0 && column.x + column.w < img.width,
        "and the page column must be located, so there is a margin beside it to \
         read: got {column:?}"
    );
}

// --- AC-6 -------------------------------------------------------------------

/// AC-6. A textured page column on a flat page background, with a full-width
/// textured band across the top (the tab strip) and another across the
/// bottom (the taskbar): the crop keeps the page art and **no row of either
/// band**.
///
/// On `cb2deef` the crop is `44,0 212x240` - both bands whole.
#[test]
fn the_crop_keeps_the_page_and_no_row_of_the_browser_chrome_or_the_taskbar() {
    let img = viewport_scene(BAND, BAND, Bands::Textured, None);
    let got = crop(&img, &Tuning::default());
    let art = viewport_scene_art(BAND);

    assert!(
        contains(got, art),
        "AC-6: the crop {got:?} must contain the page art {art:?}"
    );
    assert!(
        got.y >= BAND,
        "AC-6: the crop {got:?} keeps {} rows of the browser chrome (rows 0..{BAND})",
        BAND - got.y.min(BAND)
    );
    assert!(
        got.y + got.h <= BAND + VIEW_BODY,
        "AC-6: the crop {got:?} keeps {} rows of the taskbar (rows {}..{})",
        (got.y + got.h).saturating_sub(BAND + VIEW_BODY),
        BAND + VIEW_BODY,
        img.height
    );
}

/// AC-6, the first identity. The same scene with both bands painted as flat
/// page background: there is nothing to remove, and the crop is exactly the
/// one the pipeline made before this story.
///
/// Green before the stage exists, by construction; what earns it is that it
/// runs the same code path as the test above on a near twin of its fixture,
/// and goes red if the stage cuts into a page whose every row is page-like.
#[test]
fn a_page_with_page_background_above_and_below_is_cropped_exactly_as_before() {
    let img = viewport_scene(BAND, BAND, Bands::Blank, None);
    assert_eq!(
        crop(&img, &at_margin_3()),
        BLANK_BEFORE,
        "AC-6: with nothing to remove the viewport stage must move nothing"
    );
}

/// AC-6, the second identity. The page margin is broken by a row of noise
/// every [`DECLINES_EVERY`] page rows, so no run of page-like rows reaches the
/// minimum of 16 and no viewport can be located. The stage declines and the
/// crop - bands and all - is exactly the one made before this story.
///
/// Green before the stage exists, by construction. Its control is the next
/// test, one row of period away, which does locate.
#[test]
fn where_no_viewport_can_be_located_the_crop_is_exactly_as_before() {
    let img = viewport_scene(BAND, BAND, Bands::Textured, Some(DECLINES_EVERY));
    assert_eq!(
        crop(&img, &at_margin_3()),
        DECLINE_BEFORE,
        "AC-6: with the page margin broken every {DECLINES_EVERY} rows no viewport \
         can be located, and the stage must decline rather than guess"
    );
}

/// AC-6's control on the decline condition. The decline fixture's near twin -
/// the margin broken every [`LOCATES_EVERY`] rows instead, leaving runs of 16 -
/// is on the other side of the minimum run: the viewport is located and the
/// tab strip removed. Without this, the identity above would pass for a stage
/// that never did anything.
#[test]
fn one_row_further_apart_the_breaks_leave_a_viewport_and_the_tab_strip_goes() {
    let img = viewport_scene(BAND, BAND, Bands::Textured, Some(LOCATES_EVERY));
    let got = crop(&img, &Tuning::default());
    assert!(
        got.y >= BAND,
        "the control: with the page margin broken every {LOCATES_EVERY} rows a run \
         of 16 page rows exists, the viewport is located, and the crop must drop \
         the tab strip (rows 0..{BAND}). Got {got:?}"
    );
}

// --- AC-7 -------------------------------------------------------------------

/// AC-7. The page art starts on the viewport's first page row and ends on the
/// row above the taskbar. At `margin_px` 3 the crop's first row is that
/// viewport row - not three rows above it, inside the chrome - and its last
/// row is the one above the taskbar. The columns still get their margin.
///
/// On `cb2deef` the crop is `45,0 210x216`.
#[test]
fn the_margin_does_not_put_the_browser_chrome_or_the_taskbar_back() {
    let t = Tuning {
        margin_px: 3,
        ..Tuning::default()
    };
    let img = viewport_scene(AC7_TOP, AC7_BOTTOM, Bands::Textured, None);
    let got = crop(&img, &t);

    assert_eq!(
        rows(got),
        (AC7_TOP, AC7_TOP + VIEW_BODY - 1),
        "AC-7: at margin_px 3 the crop must run from the first viewport row to the \
         row above the taskbar; got {got:?}"
    );
    assert_eq!(
        (got.x, got.w),
        AC7_COLUMNS_AT_3,
        "AC-7: the columns keep their margin - the stage moves rows only"
    );
}

/// AC-7's control: at `margin_px` 0 the rows are the same.
#[test]
fn with_no_margin_the_crop_runs_over_the_same_viewport_rows() {
    let t = Tuning {
        margin_px: 0,
        ..Tuning::default()
    };
    let img = viewport_scene(AC7_TOP, AC7_BOTTOM, Bands::Textured, None);
    let got = crop(&img, &t);

    assert_eq!(
        rows(got),
        (AC7_TOP, AC7_TOP + VIEW_BODY - 1),
        "AC-7's control: at margin_px 0 the crop must cover the same viewport rows; \
         got {got:?}"
    );
    assert_eq!(
        (got.x, got.w),
        AC7_COLUMNS_AT_0,
        "and the page column exactly"
    );
}
