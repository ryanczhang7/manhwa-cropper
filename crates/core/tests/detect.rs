//! MC-006, AC-1 to AC-5: the crop rectangle expands outward and never clips
//! art.
//!
//! `detect` composes the pipeline of `docs/wiki/architecture.md`,
//! "cropper-core": the uniform trim (MC-003), the content box (MC-005), a
//! **second** uniform trim inside that box - because peeling chrome often
//! exposes a gutter - and then the outward margin, expanded by
//! `Tuning::margin_px` on every side and clamped to the image. It reports what
//! the stages did, so MC-007 can turn it into a decision.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: `margin_px` (3 when this file was written; **0 since
//!   MC-049**, the user's ruling of 2026-09-23), and the `+ 2` slack AC-4
//!   allows on top of it. Both are read out of `Tuning::default()` or out of the criterion;
//!   neither was calibrated here, and `margin_px` is never written as a
//!   literal - a test below drives `detect` at three different margins to pin
//!   that it is read from the `Tuning` argument rather than compiled in.
//! * **Mechanical**: `detect(&Luma, &Tuning) -> Option<Detection>` and
//!   `Detection { rect, trimmed, removed, ambiguous }`. Pinned exactly.
//! * **Measured**: only the fixtures. The generator in `common/` knows the
//!   true art rect, so every expectation here is exact rather than observed,
//!   and the measurements below exist so a fixture that drifts fails loudly
//!   instead of quietly moving a threshold. Every measured value is in the
//!   story's Handoff.
//!
//! # Why the art is the checkerboard and not `soft_art`
//!
//! The one thing AC-4 is really asking is whether the chrome peel can be made
//! to eat the art. Under [`common::soft_art`] it cannot even try: that texture
//! carries no strong line, so once the chrome is gone there is no candidate
//! strip on any side and the peel stops for want of something to judge. The
//! checkerboard is the opposite - a strong line at *every* row and column
//! (160.1 mean absolute difference, measured below) - so on every pass there
//! is a one-line candidate strip on all four sides and the only thing keeping
//! the art whole is that a line of it is nowhere near flat enough to be
//! chrome (0.30 at worst, against a threshold of 0.85). Two tests below
//! measure exactly those two numbers, because without them the property is a
//! test of the fixture rather than of `detect`.

mod common;

use common::{
    Border, CHROME_MAX_SHARE, Chrome, Layout, Recipe, any_chrome_recipe, chrome_band, flat_band,
    soft_art,
};
use cropper_core::content::Side;
use cropper_core::edges::{col_profile, row_profile, strong_lines};
use cropper_core::trim::trim_uniform;
use cropper_core::{Detection, Luma, Rect, Tuning, detect};
use proptest::prelude::*;

// --- Fixture constants ------------------------------------------------------
//
// None of these is a threshold. Thresholds are read from `Tuning::default()`
// at every use; these are the sizes and colours of the synthetic screenshots.

/// AC-2's scene is 200x100, so a percentage is a pixel count: the chrome band
/// is 10 rows (10% of H, inside `chrome_max_extent`) and each gutter is 70
/// columns (35% of W, past it - which is what sends the gutters to the second
/// trim instead of the chrome peel).
const AC2_W: u32 = 200;
const AC2_H: u32 = 100;
const AC2_BAND_H: u32 = 10;
const AC2_GUTTER_W: u32 = 70;

/// The chrome background of the hand-built scenes: a light toolbar.
const BAND_BG: u8 = 200;
/// The page behind the art in AC-2: white.
const PAGE_BG: u8 = 255;

/// Seed for the art textures. Fixed, because every expectation in this file is
/// a rect, not a pixel value.
const SEED: u32 = 7;

/// The flat fraction the hand-built chrome bands render at: comfortably inside
/// AC-4's `0.86..=0.98` and above `chrome_flat_fraction`, so these scenes test
/// `detect`'s composition rather than MC-005's threshold, which MC-005 already
/// pins on both sides.
const BAND_FLAT: f64 = 0.90;

// --- Fixture builders -------------------------------------------------------

/// The rect covering the whole of `img`.
fn whole(img: &Luma) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    }
}

/// `rect` grown by `margin` on every side and clamped to `img`: the answer
/// AC-1 and AC-3 demand, written once so no test can quietly use a different
/// margin.
fn expanded(rect: Rect, img: &Luma, margin: u32) -> Rect {
    let x = rect.x.saturating_sub(margin);
    let y = rect.y.saturating_sub(margin);
    let right = (rect.x + rect.w + margin).min(img.width);
    let bottom = (rect.y + rect.h + margin).min(img.height);
    Rect {
        x,
        y,
        w: right - x,
        h: bottom - y,
    }
}

/// Whether `outer` holds all of `inner`.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// AC-1's scene: textured art inside a chrome band inside four uniform
/// borders, each border a different width and colour.
fn ac1_recipe() -> Recipe {
    Recipe {
        top: Border::solid(10, PAGE_BG),
        right: Border::solid(12, 0),
        bottom: Border::solid(8, 128),
        left: Border::solid(15, 64),
        top_chrome: vec![Chrome::new(12, BAND_BG, BAND_FLAT)],
        ..Recipe::new(120, 80)
    }
}

/// AC-3's scene: AC-1's with the right border taken away, so the art runs to
/// the image's right edge and the margin has nowhere to go on that side.
fn ac3_recipe() -> Recipe {
    Recipe {
        top: Border::solid(10, PAGE_BG),
        right: Border::solid(0, 0),
        bottom: Border::solid(10, 128),
        left: Border::solid(20, 64),
        top_chrome: vec![Chrome::new(12, BAND_BG, BAND_FLAT)],
        ..Recipe::new(120, 80)
    }
}

/// AC-2's scene: full-width chrome above a page whose art has uniform gutters
/// left and right of it and nothing above or below it.
///
/// Every edge line of this image is non-uniform - the top row is chrome, the
/// side columns span chrome and page, the bottom row holds art - so the
/// *first* trim removes nothing and the gutters can only be reached by the
/// second one, after the chrome is peeled. That is the whole point of the
/// scene, and the test below measures it rather than asserting it in a
/// comment.
fn ac2_scene() -> Luma {
    stack(&[
        chrome_band(
            AC2_W,
            AC2_BAND_H,
            BAND_BG,
            common::CHROME_DEVIATION,
            Tuning::default().uniform_tolerance,
            BAND_FLAT,
        ),
        beside(&[
            gutter(AC2_GUTTER_W, AC2_H - AC2_BAND_H),
            soft_art(ac2_art().w, ac2_art().h, SEED),
            gutter(AC2_GUTTER_W, AC2_H - AC2_BAND_H),
        ]),
    ])
}

/// AC-2's scene with the chrome band replaced by a band of one flat colour:
/// the negative control for "the first trim removes nothing". Same geometry,
/// same gutters, and the only difference is that the top rows are now uniform
/// rather than chrome-like - so the first trim must reach them.
fn ac2_scene_with_a_uniform_top_band() -> Luma {
    stack(&[
        flat_band(AC2_W, AC2_BAND_H, BAND_BG, 0),
        beside(&[
            gutter(AC2_GUTTER_W, AC2_H - AC2_BAND_H),
            soft_art(ac2_art().w, ac2_art().h, SEED),
            gutter(AC2_GUTTER_W, AC2_H - AC2_BAND_H),
        ]),
    ])
}

/// The art rect of [`ac2_scene`]: below the band, between the gutters, running
/// to the bottom edge.
fn ac2_art() -> Rect {
    Rect {
        x: AC2_GUTTER_W,
        y: AC2_BAND_H,
        w: AC2_W - 2 * AC2_GUTTER_W,
        h: AC2_H - AC2_BAND_H,
    }
}

/// A uniform page gutter.
fn gutter(width: u32, height: u32) -> Luma {
    flat_band(width, height, PAGE_BG, 0)
}

/// Blocks of equal width, stacked top to bottom.
fn stack(blocks: &[Luma]) -> Luma {
    let width = blocks[0].width;
    let mut data = Vec::new();
    let mut height = 0;
    for block in blocks {
        assert_eq!(block.width, width, "stacked blocks must share a width");
        data.extend_from_slice(&block.data);
        height += block.height;
    }
    Luma {
        width,
        height,
        data,
    }
}

/// Blocks of equal height, placed left to right.
fn beside(blocks: &[Luma]) -> Luma {
    let height = blocks[0].height;
    let width: u32 = blocks.iter().map(|block| block.width).sum();
    let mut data = Vec::with_capacity(width as usize * height as usize);
    for y in 0..height as usize {
        for block in blocks {
            assert_eq!(block.height, height, "adjacent blocks must share a height");
            let w = block.width as usize;
            data.extend_from_slice(&block.data[y * w..(y + 1) * w]);
        }
    }
    Luma {
        width,
        height,
        data,
    }
}

// --- Measurement, so a drifting fixture fails loudly ------------------------

/// The pixels of `rect`, row by row.
fn pixels(img: &Luma, rect: Rect) -> Vec<u8> {
    let stride = img.width as usize;
    let mut out = Vec::with_capacity(rect.w as usize * rect.h as usize);
    for dy in 0..rect.h as usize {
        let start = (rect.y as usize + dy) * stride + rect.x as usize;
        out.extend_from_slice(&img.data[start..start + rect.w as usize]);
    }
    out
}

/// The share of `rect`'s pixels within `tolerance` of its median luma - the
/// definition `chrome_flat_fraction` is measured against.
fn flat_fraction(img: &Luma, rect: Rect, tolerance: u8) -> f64 {
    let mut values = pixels(img, rect);
    values.sort_unstable();
    let median = values[values.len() / 2];
    let flat = values
        .iter()
        .filter(|&&px| px.abs_diff(median) <= tolerance)
        .count();
    flat as f64 / values.len() as f64
}

// --- The settled constant and the pinned shape ------------------------------

/// `margin_px` is settled at **0** (`architecture.md`, decision 5 and the
/// `Tuning` table). It was 3 from MC-006 until MC-049, when the user saw the
/// three flat columns it adds on each side as "the page on the right and left
/// side" (2026-09-23) and ruled it to 0 on all four sides. Every default
/// margin in this file is what it is *because of this test*, not because a
/// value was assumed; MC-049 AC-6's synthetic pins live in
/// `tests/page_edges.rs`.
#[test]
fn the_default_margin_px_is_zero() {
    assert_eq!(
        Tuning::default().margin_px,
        0,
        "MC-049: the default margin is 0 on all four sides - the user's ruling of \
         2026-09-23 that the crop's side edges carry no page background"
    );
}

#[test]
fn a_detection_carries_a_rect_a_trim_flag_a_removal_order_and_an_ambiguity_flag() {
    let recipe = ac1_recipe();
    let img = recipe.render();
    let Detection {
        rect,
        trimmed,
        removed,
        ambiguous,
    } = detect(&img, &Tuning::default()).expect("AC-1's scene is not a uniform image");
    let _: Rect = rect;
    let _: bool = trimmed;
    let _: Vec<Side> = removed;
    let _: bool = ambiguous;
    // Debug, so a failing assertion elsewhere prints something readable.
    let printed = format!("{:?}", detect(&img, &Tuning::default()));
    assert!(printed.contains("Detection"), "got {printed}");
}

// --- The fixtures are what they claim to be ---------------------------------

#[test]
fn every_generated_chrome_band_renders_at_the_flat_fraction_it_was_asked_for() {
    let tolerance = Tuning::default().uniform_tolerance;
    let mut drifted = Vec::new();
    for &requested in &[0.86f64, 0.90, 0.94, 0.98] {
        for &(thickness, art_w, art_h) in &[(8u32, 100u32, 100u32), (33, 120, 120), (25, 300, 140)]
        {
            let recipe = Recipe {
                top_chrome: vec![Chrome::new(thickness, BAND_BG, requested)],
                left_chrome: vec![Chrome::new(thickness, 60, requested)],
                ..Recipe::new(art_w, art_h)
            };
            let img = recipe.render();
            for (side, _, rect) in recipe.chrome_rects() {
                let rendered = flat_fraction(&img, rect, tolerance);
                // A band carries a whole number of "text" pixels per line, so
                // the rendered fraction lands within `1 / long` of the one
                // asked for - and, because that count is rounded down, always
                // on the flatter side. Both halves matter: the first says the
                // fixture is the one AC-4 describes, the second that
                // quantisation can never drift a band down onto
                // `chrome_flat_fraction` and turn a chrome band into an
                // ambiguous one behind the test's back.
                if !(requested..=requested + common::CHROME_FLAT_TOLERANCE).contains(&rendered) {
                    drifted.push(format!(
                        "{side:?} {thickness}px on {art_w}x{art_h}: asked {requested}, \
                         rendered {rendered}"
                    ));
                }
                if rendered < f64::from(Tuning::default().chrome_flat_fraction) {
                    drifted.push(format!(
                        "{side:?} {thickness}px on {art_w}x{art_h}: rendered {rendered}, \
                         which is not chrome at all"
                    ));
                }
            }
        }
    }
    assert!(
        drifted.is_empty(),
        "the chrome band generator missed its requested flat fraction: {drifted:?}"
    );
}

#[test]
fn a_generated_chrome_band_carries_no_strong_line_of_its_own() {
    let t = Tuning::default();
    let mut split = Vec::new();
    for &requested in &[0.86f64, 0.90, 0.98] {
        let recipe = Recipe {
            top_chrome: vec![Chrome::new(30, BAND_BG, requested)],
            left_chrome: vec![Chrome::new(30, 60, requested)],
            ..Recipe::new(200, 160)
        };
        let img = recipe.render();
        for (side, _, rect) in recipe.chrome_rects() {
            let rows = strong_lines(&row_profile(&img, rect), &t);
            let cols = strong_lines(&col_profile(&img, rect), &t);
            if !rows.is_empty() || !cols.is_empty() {
                split.push(format!("{side:?} at {requested}: {rows:?} {cols:?}"));
            }
        }
    }
    assert!(
        split.is_empty(),
        "a band holding a strong line is no longer the strip these tests build: {split:?}"
    );
}

/// The generator never hands `content_box` a strip bigger than
/// `chrome_max_extent`, so no band in this file is left in place for being too
/// big - the case MC-005 owns, and not the one MC-006 is about.
#[test]
fn no_generated_chrome_band_reaches_the_chrome_max_extent() {
    let t = Tuning::default();
    assert!(
        CHROME_MAX_SHARE < f64::from(t.chrome_max_extent),
        "AC-4's 25% of the dimension must stay inside chrome_max_extent"
    );
    let mut oversize = Vec::new();
    for recipe in [ac1_recipe(), ac3_recipe()] {
        let img = recipe.render();
        for (side, band, _) in recipe.chrome_rects() {
            let dimension = match side {
                Side::Top | Side::Bottom => img.height,
                Side::Left | Side::Right => img.width,
            };
            let share = f64::from(band.thickness) / f64::from(dimension);
            if share > CHROME_MAX_SHARE {
                oversize.push(format!("{side:?}: {share}"));
            }
        }
    }
    assert!(
        oversize.is_empty(),
        "bands past the stated share: {oversize:?}"
    );
}

/// The negative control the whole "never clip" property rests on: a single
/// line of the checkerboard art is a candidate strip on every pass, and the
/// *only* thing that stops it being peeled is that it is nowhere near flat
/// enough. If this ever drifted up to `chrome_flat_fraction` the art would be
/// eaten one line per pass and AC-4 would be measuring nothing.
#[test]
fn one_line_of_the_checkerboard_art_is_far_from_flat_enough_to_be_chrome() {
    let t = Tuning::default();
    let ceiling = f64::from(t.chrome_flat_fraction - t.ambiguity_band);
    let mut too_flat = Vec::new();
    for seed in [1u32, 7, 99, 12_345, 0xDEAD_BEEF] {
        for &(w, h) in &[(100u32, 100u32), (100, 1200), (1200, 100), (640, 480)] {
            let img = Recipe {
                seed,
                ..Recipe::new(w, h)
            }
            .render();
            let row = flat_fraction(
                &img,
                Rect {
                    x: 0,
                    y: 0,
                    w,
                    h: 1,
                },
                t.uniform_tolerance,
            );
            let col = flat_fraction(
                &img,
                Rect {
                    x: 0,
                    y: 0,
                    w: 1,
                    h,
                },
                t.uniform_tolerance,
            );
            if row >= ceiling || col >= ceiling {
                too_flat.push(format!("seed {seed} at {w}x{h}: row {row}, column {col}"));
            }
        }
    }
    assert!(
        too_flat.is_empty(),
        "an art line this flat would be peeled as chrome, or flagged ambiguous: {too_flat:?}"
    );
}

/// The other half of that control: there *is* a candidate strip to reject.
/// The checkerboard is one merged strong line spanning every profile index in
/// both directions, so `content_box` finds a one-line strip on each of the
/// four sides on every pass.
#[test]
fn the_checkerboard_art_is_one_strong_line_spanning_every_row_and_column() {
    let t = Tuning::default();
    let img = Recipe::new(300, 240).render();
    let rows = strong_lines(&row_profile(&img, whole(&img)), &t);
    let cols = strong_lines(&col_profile(&img, whole(&img)), &t);
    assert_eq!(
        rows,
        vec![cropper_core::edges::Line { start: 0, end: 238 }],
        "every adjacent row pair of the art must be strong"
    );
    assert_eq!(
        cols,
        vec![cropper_core::edges::Line { start: 0, end: 298 }],
        "every adjacent column pair of the art must be strong"
    );
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn a_top_chrome_band_inside_four_borders_leaves_the_art_rect_plus_the_margin() {
    let t = Tuning::default();
    let recipe = ac1_recipe();
    let img = recipe.render();
    let found = detect(&img, &t).expect("AC-1's scene is not a uniform image");
    assert_eq!(
        found.rect,
        expanded(recipe.art_rect(), &img, t.margin_px),
        "AC-1: art {:?} in a {}x{} image must come back expanded by {} on every side",
        recipe.art_rect(),
        img.width,
        img.height,
        t.margin_px
    );
}

#[test]
fn a_top_chrome_band_inside_four_borders_records_a_trim_and_one_peeled_side() {
    let recipe = ac1_recipe();
    let img = recipe.render();
    let found = detect(&img, &Tuning::default()).expect("AC-1's scene is not a uniform image");
    assert!(
        found.trimmed,
        "AC-1: four uniform borders were trimmed, so `trimmed` must say so"
    );
    assert_eq!(
        found.removed,
        vec![Side::Top],
        "AC-1: the top chrome band is the only strip peeled"
    );
    assert!(
        !found.ambiguous,
        "AC-1: a band at {BAND_FLAT} is clear of the ambiguity band"
    );
}

/// `margin_px` is read from the `Tuning` argument, not compiled in. Three
/// margins, one scene, and the fixture is sized so none of them clamps - so a
/// detector that expands by a literal 3 fails two of the three.
#[test]
fn the_margin_is_read_from_the_tuning_it_was_given() {
    let recipe = ac1_recipe();
    let img = recipe.render();
    let art = recipe.art_rect();
    let mut wrong = Vec::new();
    for margin in [0u32, 3, 7] {
        let t = Tuning {
            margin_px: margin,
            ..Tuning::default()
        };
        let found = detect(&img, &t).expect("AC-1's scene is not a uniform image");
        let want = expanded(art, &img, margin);
        assert!(
            want.x > 0 && want.y > 0 && want.x + want.w < img.width,
            "the fixture must have room for a margin of {margin} without clamping"
        );
        if found.rect != want {
            wrong.push(format!(
                "margin {margin}: wanted {want:?}, got {:?}",
                found.rect
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the margin must come from the Tuning argument: {wrong:?}"
    );
}

// --- AC-2 -------------------------------------------------------------------

/// The precondition AC-2 names, measured both ways. The first trim removes
/// nothing from the real scene because every edge line of it is non-uniform;
/// swap the chrome band for a band of one flat colour and the first trim
/// reaches it immediately. Without the second half, "removes nothing" would
/// also be satisfied by a trim that never removes anything.
#[test]
fn the_first_trim_removes_nothing_while_the_top_rows_are_chrome() {
    let t = Tuning::default();
    let scene = ac2_scene();
    assert_eq!(
        trim_uniform(&scene, &t),
        Some(whole(&scene)),
        "AC-2 needs a scene the first trim cannot touch"
    );

    let control = ac2_scene_with_a_uniform_top_band();
    let trimmed = trim_uniform(&control, &t).expect("the control still has art in it");
    assert!(
        trimmed.y >= AC2_BAND_H,
        "the control must lose its flat top band to the first trim; it kept {trimmed:?}"
    );
}

#[test]
fn chrome_above_a_gutter_page_is_peeled_and_the_gutters_go_to_the_second_trim() {
    let t = Tuning::default();
    let scene = ac2_scene();
    let found = detect(&scene, &t).expect("AC-2's scene is not a uniform image");
    assert_eq!(
        found.rect,
        expanded(ac2_art(), &scene, t.margin_px),
        "AC-2: the band is peeled, the gutters are trimmed, and what is left is the \
         art rect {:?} plus the margin, clamped at the bottom edge it touches",
        ac2_art()
    );
    assert_eq!(
        found.removed,
        vec![Side::Top],
        "AC-2: only the chrome band is peeled - the gutters are too wide to be chrome \
         and are the second trim's work"
    );
}

/// The discriminator for the second trim: on this scene the *first* trim moved
/// nothing at all, so a `trimmed` that is true can only have come from the
/// second one.
#[test]
fn trimmed_is_true_when_only_the_second_trim_moved_an_edge() {
    let scene = ac2_scene();
    let found = detect(&scene, &Tuning::default()).expect("AC-2's scene is not a uniform image");
    assert!(
        found.trimmed,
        "AC-2: the gutters were trimmed inside the content box, so `trimmed` must say so"
    );
}

// --- AC-3 -------------------------------------------------------------------

#[test]
fn art_flush_with_the_right_edge_clamps_the_margin_instead_of_overflowing() {
    let t = Tuning::default();
    let recipe = ac3_recipe();
    let img = recipe.render();
    let art = recipe.art_rect();
    assert_eq!(
        art.x + art.w,
        img.width,
        "AC-3 needs a fixture whose art touches the right edge"
    );
    let found = detect(&img, &t).expect("AC-3's scene is not a uniform image");
    assert_eq!(
        found.rect.x + found.rect.w,
        img.width,
        "AC-3: the margin clamps at the right edge rather than overflowing it"
    );
    assert_eq!(
        found.rect.x,
        art.x - t.margin_px,
        "AC-3: the left side still carries the full margin"
    );
    assert_eq!(
        found.rect.y,
        art.y - t.margin_px,
        "AC-3: the top side still carries the full margin"
    );
    assert_eq!(
        found.rect.y + found.rect.h,
        art.y + art.h + t.margin_px,
        "AC-3: the bottom side still carries the full margin"
    );
}

#[test]
fn art_filling_the_whole_image_clamps_the_margin_on_every_side() {
    let t = Tuning::default();
    let recipe = Recipe::new(120, 80);
    let img = recipe.render();
    let found = detect(&img, &t).expect("textured art is not a uniform image");
    assert_eq!(
        found.rect,
        whole(&img),
        "AC-3: with nothing to trim and nothing to peel, the margin clamps to the image"
    );
    assert!(
        !found.trimmed,
        "no border was trimmed here, so `trimmed` must be false"
    );
    assert!(
        found.removed.is_empty(),
        "no strip was peeled here, so `removed` must be empty; got {:?}",
        found.removed
    );
    assert!(!found.ambiguous, "no strip was close to chrome-like here");
}

// --- AC-4 -------------------------------------------------------------------

proptest! {
    // AC-4's stated minimum, at AC-4's stated ranges. The images reach
    // 3184x3184 and the suite is slow in a debug build in proportion: 99 s for
    // `cargo test --workspace` and 120 s for the `coverage` gate, against 6 s
    // and 22 s with `[profile.test] opt-level = 2`. Measured, not guessed, and
    // the numbers and the recommendation are in the story's Handoff.
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    /// AC-4 in full: any recipe the generator can produce, and the rect
    /// `detect` returns holds all of the art and little more.
    ///
    /// Both halves matter, and they fail for opposite reasons. A detector that
    /// crops too tight clips art and fails the first; one that gives up and
    /// returns the whole image keeps the chrome and fails the second.
    #[test]
    fn a_detected_rect_holds_all_the_art_and_no_more_than_two_pixels_of_junk(
        recipe in any_chrome_recipe(),
    ) {
        let t = Tuning::default();
        let img = recipe.render();
        let art = recipe.art_rect();
        let found = detect(&img, &t);
        prop_assert!(
            found.is_some(),
            "the generator's art is textured, so it is never a uniform image: {:?}",
            recipe
        );
        let found = found.unwrap();
        prop_assert!(
            contains(found.rect, art),
            "clipped: art {:?} is not inside {:?}; {}x{} image from {:?}",
            art, found.rect, img.width, img.height, recipe
        );
        prop_assert!(
            contains(expanded(art, &img, t.margin_px + 2), found.rect),
            "junk: {:?} reaches outside the art plus {} px; {}x{} image from {:?}",
            found.rect, t.margin_px + 2, img.width, img.height, recipe
        );
    }
}

// --- AC-5 -------------------------------------------------------------------

#[test]
fn a_uniform_image_has_nothing_to_crop() {
    let t = Tuning::default();
    let mut cropped = Vec::new();
    // Zero, one and many: a single pixel, a single row, a realistic plane; and
    // the whole span of "uniform", from one flat colour up to the tolerance
    // itself, which is inclusive.
    for &(w, h) in &[(1u32, 1u32), (40, 1), (1, 40), (60, 40)] {
        for spread in [0u8, 5, t.uniform_tolerance] {
            let img = flat_band(w, h, 120, spread);
            if let Some(found) = detect(&img, &t) {
                cropped.push(format!("{w}x{h} at spread {spread}: {:?}", found.rect));
            }
        }
    }
    assert!(
        cropped.is_empty(),
        "AC-5: a uniform image must yield None, not a rect: {cropped:?}"
    );
}

/// The control that makes AC-5 mean something: one level past the tolerance
/// the same image is no longer uniform, and `detect` returns a rect. Without
/// this, a `detect` that returned `None` for everything would satisfy AC-5.
#[test]
fn an_image_one_level_past_the_tolerance_is_not_uniform_and_is_detected() {
    let t = Tuning::default();
    let img = flat_band(60, 40, 120, t.uniform_tolerance + 1);
    assert!(
        detect(&img, &t).is_some(),
        "AC-5's boundary: max - min of {} is past a tolerance of {}, so this image is \
         not uniform",
        t.uniform_tolerance + 1,
        t.uniform_tolerance
    );
}

// --- The non-goals, where they are cheap to pin -----------------------------

/// `Out of scope`: the layout flag is MC-003's, and `detect` must not have
/// grown an opinion about which pair of sides spans the image. Same art, same
/// borders, same chrome, both layouts, same answer.
#[test]
fn the_answer_does_not_depend_on_which_pair_of_borders_spans_the_image() {
    let t = Tuning::default();
    let mut differed = Vec::new();
    for layout in [Layout::Bands, Layout::Gutters] {
        let recipe = Recipe {
            layout,
            ..ac1_recipe()
        };
        let img = recipe.render();
        let found = detect(&img, &t).expect("AC-1's scene is not a uniform image");
        let want = expanded(recipe.art_rect(), &img, t.margin_px);
        if found.rect != want {
            differed.push(format!("{layout:?}: wanted {want:?}, got {:?}", found.rect));
        }
    }
    assert!(
        differed.is_empty(),
        "the layout must not change the answer: {differed:?}"
    );
}
