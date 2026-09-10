//! MC-003, AC-1 to AC-5: uniform solid borders are trimmed on all four sides.
//!
//! Every expected value here is exact. The fixture generator in `common/`
//! knows the true art rect, so nothing in this file is a calibrated number:
//! `uniform_tolerance` is read out of `Tuning::default()`, and the uniformity
//! rule is the one settled with the user on 2026-09-10 and recorded in the
//! story's Context and in `docs/wiki/architecture.md` - a row or column
//! segment, measured *within the current rect*, is uniform iff
//! `max - min <= uniform_tolerance`. No reference colour, no median, no mean.

mod common;

use common::{Border, Layout, Recipe, flat_band};
use cropper_core::trim::trim_uniform;
use cropper_core::{Rect, Tuning};
use proptest::prelude::*;

/// Render a recipe and trim it with the settled tuning. Every AC-1, AC-2,
/// AC-4 and AC-5 test goes through here, so no test can quietly use a
/// different tolerance.
fn trimmed(recipe: &Recipe) -> Option<Rect> {
    trim_uniform(&recipe.render(), &Tuning::default())
}

// --- The settled constant and the pinned shapes -----------------------------

/// `uniform_tolerance` is settled at 10 (`architecture.md`, the `Tuning`
/// table; the corpus story MC-019 may change it under `## Amendments`). Every
/// threshold in this file is 10 *because of this test*, not because 10 was
/// assumed.
#[test]
fn the_default_uniform_tolerance_is_ten() {
    assert_eq!(
        Tuning::default().uniform_tolerance,
        10,
        "the settled default tolerance"
    );
}

#[test]
fn rect_is_a_plain_comparable_copyable_value_type() {
    fn assert_value_type<T: std::fmt::Debug + Clone + Copy + PartialEq + Eq>(_: &T) {}

    let a = Rect {
        x: 1,
        y: 2,
        w: 3,
        h: 4,
    };
    let same = a;
    assert_value_type(&a);
    assert_eq!(a, same, "Rect is Copy, so `same` is a copy and still equal");
    assert_ne!(
        a,
        Rect {
            x: 1,
            y: 2,
            w: 3,
            h: 5
        },
        "a differing height must compare unequal"
    );
    assert!(
        format!("{a:?}").contains('3'),
        "Rect must be Debug, and the Debug form must show its fields"
    );
}

// --- AC-1: any textured art inside four solid borders -----------------------

proptest! {
    // 128 cases, not proptest's default 256: borders up to 200 px put the
    // largest fixture at 424x424, and 128 cases cost 0.20 s plain against
    // 0.40 s for 256. Measured, not guessed; see the story's Handoff.
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    /// AC-1 in full: art 2x2 to 24x24, four independently sized (0..=200) and
    /// independently coloured solid borders, either layout. The generator's
    /// art is textured by construction, so the art rect can never be trimmed
    /// and the borders always can.
    #[test]
    fn any_textured_art_inside_four_solid_borders_trims_to_the_art_rect(
        recipe in common::any_recipe(200),
    ) {
        let img = recipe.render();
        prop_assert_eq!(
            trim_uniform(&img, &Tuning::default()),
            Some(recipe.art_rect()),
            "AC-1: {}x{} image from {:?}",
            img.width,
            img.height,
            recipe
        );
    }
}

/// The commonest real case: a white gutter all the way round.
#[test]
fn a_white_border_on_all_four_sides_trims_to_the_art_rect() {
    for layout in [Layout::Bands, Layout::Gutters] {
        let white = Border::solid(12, 255);
        let recipe = Recipe {
            top: white,
            right: white,
            bottom: white,
            left: white,
            layout,
            ..Recipe::new(40, 30)
        };
        assert_eq!(
            trimmed(&recipe),
            Some(Rect {
                x: 12,
                y: 12,
                w: 40,
                h: 30
            }),
            "AC-1 white border, {layout:?}"
        );
    }
}

/// AC-1's stated upper bound on border width, at both layouts: a 416x416
/// image that is almost all border.
#[test]
fn borders_two_hundred_pixels_wide_are_trimmed_on_every_side() {
    for layout in [Layout::Bands, Layout::Gutters] {
        let recipe = Recipe {
            top: Border::solid(200, 0),
            right: Border::solid(200, 128),
            bottom: Border::solid(200, 255),
            left: Border::solid(200, 64),
            layout,
            ..Recipe::new(16, 16)
        };
        assert_eq!(recipe.width(), 416);
        assert_eq!(recipe.height(), 416);
        assert_eq!(
            trimmed(&recipe),
            Some(Rect {
                x: 200,
                y: 200,
                w: 16,
                h: 16
            }),
            "AC-1 200 px borders, {layout:?}"
        );
    }
}

// --- AC-1: one side at a time, the other three at width 0 -------------------
//
// Four tests rather than one, so that a trim which handles rows but not
// columns (or top but not bottom) names the side it dropped.

#[test]
fn a_top_band_alone_is_trimmed_and_no_other_side_moves() {
    let recipe = Recipe {
        top: Border::solid(7, 255),
        ..Recipe::new(30, 20)
    };
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 0,
            y: 7,
            w: 30,
            h: 20
        }),
        "AC-1 top only: 30x27 image, 7 white rows at the top"
    );
}

#[test]
fn a_bottom_band_alone_is_trimmed_and_no_other_side_moves() {
    let recipe = Recipe {
        bottom: Border::solid(7, 255),
        ..Recipe::new(30, 20)
    };
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 0,
            y: 0,
            w: 30,
            h: 20
        }),
        "AC-1 bottom only: 30x27 image, 7 white rows at the bottom"
    );
}

#[test]
fn a_left_gutter_alone_is_trimmed_and_no_other_side_moves() {
    let recipe = Recipe {
        left: Border::solid(9, 255),
        ..Recipe::new(30, 20)
    };
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 9,
            y: 0,
            w: 30,
            h: 20
        }),
        "AC-1 left only: 39x20 image, 9 white columns on the left"
    );
}

#[test]
fn a_right_gutter_alone_is_trimmed_and_no_other_side_moves() {
    let recipe = Recipe {
        right: Border::solid(9, 255),
        ..Recipe::new(30, 20)
    };
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 0,
            y: 0,
            w: 30,
            h: 20
        }),
        "AC-1 right only: 39x20 image, 9 white columns on the right"
    );
}

// --- AC-2: the tolerance boundary -------------------------------------------

/// Border pixels that deviate from the side's colour by exactly
/// `uniform_tolerance` are still removed. The deviation runs in one direction
/// only (`colour ..= colour + 10`), so each border's `max - min` is exactly
/// 10 and the fixture satisfies AC-2's wording literally.
#[test]
fn border_pixels_deviating_by_ten_from_the_side_colour_are_still_trimmed() {
    for layout in [Layout::Bands, Layout::Gutters] {
        let recipe = Recipe {
            top: Border {
                thickness: 6,
                colour: 200,
                spread: 10,
            },
            right: Border {
                thickness: 5,
                colour: 100,
                spread: 10,
            },
            bottom: Border {
                thickness: 7,
                colour: 30,
                spread: 10,
            },
            left: Border {
                thickness: 4,
                colour: 240,
                spread: 10,
            },
            layout,
            ..Recipe::new(30, 20)
        };
        assert_eq!(
            trimmed(&recipe),
            Some(Rect {
                x: 4,
                y: 6,
                w: 30,
                h: 20
            }),
            "AC-2 deviation 10, {layout:?}: max - min == 10 is still uniform"
        );
    }
}

/// The control on the number. The same fixture, with the top band alone
/// deviating by 11 instead of 10, must **not** be trimmed at the top: the
/// returned rect starts at that row (`y == 0`). The other three sides still
/// trim, which is what makes this a control on the top band and not on the
/// trim as a whole.
///
/// The gutters layout is used so the left and right gutters span the full
/// height and can be trimmed before the top band is even looked at inside the
/// art columns; the bottom band then trims to the art. Only the 11-level top
/// band survives.
#[test]
fn a_top_band_deviating_by_eleven_is_kept_and_the_rect_starts_at_that_row() {
    let recipe = Recipe {
        top: Border {
            thickness: 6,
            colour: 90,
            spread: 11,
        },
        right: Border::solid(5, 100),
        bottom: Border::solid(7, 30),
        left: Border::solid(4, 240),
        layout: Layout::Gutters,
        ..Recipe::new(30, 20)
    };
    assert_eq!(recipe.width(), 39);
    assert_eq!(recipe.height(), 33);
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 4,
            y: 0,
            w: 30,
            h: 26
        }),
        "AC-2 control: a top band spanning 11 levels is kept, so the rect \
         starts at row 0 and is 6 rows taller than the art rect {{4, 6, 30, 20}}"
    );
}

/// The same control on all four sides at once: nothing is uniform anywhere, so
/// nothing is trimmed and the whole image comes back.
#[test]
fn borders_deviating_by_eleven_on_every_side_are_all_kept() {
    let recipe = Recipe {
        top: Border {
            thickness: 6,
            colour: 90,
            spread: 11,
        },
        right: Border {
            thickness: 5,
            colour: 100,
            spread: 11,
        },
        bottom: Border {
            thickness: 7,
            colour: 30,
            spread: 11,
        },
        left: Border {
            thickness: 4,
            colour: 240,
            spread: 11,
        },
        layout: Layout::Bands,
        ..Recipe::new(30, 20)
    };
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 0,
            y: 0,
            w: 39,
            h: 33
        }),
        "AC-2 control: every side spans 11 levels, so no side may be trimmed"
    );
}

// --- AC-3: nothing to crop ---------------------------------------------------

#[test]
fn an_image_of_one_colour_trims_to_none() {
    let img = flat_band(24, 18, 200, 0);
    assert_eq!(
        trim_uniform(&img, &Tuning::default()),
        None,
        "AC-3: a single flat colour has no art to find"
    );
}

#[test]
fn an_image_whose_whole_range_is_exactly_the_tolerance_trims_to_none() {
    let img = flat_band(24, 18, 120, 10);
    assert_eq!(
        trim_uniform(&img, &Tuning::default()),
        None,
        "AC-3: max - min == 10 over the whole image is uniform everywhere"
    );
}

/// The control on AC-3's number, from the other side: one level wider and
/// **no** row or column of the image is uniform, so every pixel is kept.
#[test]
fn an_image_whose_whole_range_is_one_over_the_tolerance_keeps_every_pixel() {
    let img = flat_band(24, 18, 120, 11);
    assert_eq!(
        trim_uniform(&img, &Tuning::default()),
        Some(Rect {
            x: 0,
            y: 0,
            w: 24,
            h: 18
        }),
        "AC-3 control: max - min == 11 is not uniform, so nothing may be trimmed"
    );
}

/// The tolerance must be read from the `Tuning` that was passed in, not from a
/// constant in the trim. One image, two tunings, opposite answers.
#[test]
fn the_tolerance_is_read_from_the_tuning_and_not_hard_coded() {
    let img = flat_band(24, 18, 100, 20);
    assert_eq!(
        trim_uniform(&img, &Tuning::default()),
        Some(Rect {
            x: 0,
            y: 0,
            w: 24,
            h: 18
        }),
        "a spread of 20 is not uniform at the default tolerance of 10"
    );

    let loose = Tuning {
        uniform_tolerance: 20,
        ..Default::default()
    };
    assert_eq!(
        trim_uniform(&img, &loose),
        None,
        "the same image is uniform everywhere at a tolerance of 20"
    );
}

// --- AC-4: art fills the image ----------------------------------------------

#[test]
fn art_that_fills_the_whole_image_is_returned_whole() {
    let recipe = Recipe::new(37, 23);
    assert_eq!(recipe.width(), 37);
    assert_eq!(recipe.height(), 23);
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 0,
            y: 0,
            w: 37,
            h: 23
        }),
        "AC-4: all four borders are 0 px wide, so the art rect is the image"
    );
}

// --- AC-5: the iterative mechanism ------------------------------------------
//
// Both tests use the same recipe: art 30x20, borders top 6 (colour 10),
// right 5 (250), bottom 7 (128), left 4 (64); image 39x33; art rect
// {4, 6, 30, 20}. The four colours are deliberately far apart.
//
// This pair is the control on iteration. A trim that makes a *single* pass
// over the four sides gets exactly one of the two layouts wrong, whichever
// order it visits them in:
//
//   * top-then-sides, `Gutters` layout: row 0 spans the left gutter colour
//     (64), the top band colour (10) and the right gutter colour (250), so it
//     is not uniform and the top is refused. The gutters then trim, which
//     leaves row 0 uniform inside the new rect - but a single pass never comes
//     back to it. Wrong answer: `Some(Rect { x: 4, y: 0, w: 30, h: 33 })`.
//   * sides-then-top, `Bands` layout: column 0 spans the top band colour (10),
//     the left gutter colour (64) and the bottom band colour (128), so the
//     left is refused; the bands then trim and are never revisited. Wrong
//     answer: `Some(Rect { x: 0, y: 6, w: 39, h: 20 })`.

fn four_colour_recipe(layout: Layout) -> Recipe {
    Recipe {
        top: Border::solid(6, 10),
        right: Border::solid(5, 250),
        bottom: Border::solid(7, 128),
        left: Border::solid(4, 64),
        layout,
        ..Recipe::new(30, 20)
    }
}

#[test]
fn four_different_border_colours_trim_to_the_art_rect_with_full_width_bands() {
    let recipe = four_colour_recipe(Layout::Bands);
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 4,
            y: 6,
            w: 30,
            h: 20
        }),
        "AC-5 Bands: a sides-first single pass would return {{0, 6, 39, 20}}"
    );
}

#[test]
fn four_different_border_colours_trim_to_the_art_rect_with_full_height_gutters() {
    let recipe = four_colour_recipe(Layout::Gutters);
    assert_eq!(
        trimmed(&recipe),
        Some(Rect {
            x: 4,
            y: 6,
            w: 30,
            h: 20
        }),
        "AC-5 Gutters: a top-first single pass would return {{4, 0, 30, 33}}"
    );
}
