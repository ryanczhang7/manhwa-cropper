//! MC-054: generated recipes on which the viewport stage (MC-048, widened by
//! MC-052) cut, or once cut, rows of art - frozen as deterministic tests.
//!
//! `tests/detect.rs`'s property test draws 256 fresh recipes a run from
//! `common::any_chrome_recipe()`, and its failures are replayed only from a
//! gitignored regressions file, so CI never sees a recipe twice. The family
//! these belong to - textured chrome bands beside the page whose flat fraction
//! straddles `PAGE_LIKE` (the generator draws 0.86 to 0.98) - turns up about
//! once in 12,500 draws, so without these the required `unit` gate goes red on
//! `main` at random, for no commit's reason, and a fix that removes one of
//! MC-052's rules passes it most of the time.
//!
//! Each test asserts exactly what the property test asserts of one draw: the
//! crop holds all of the art, and reaches no more than `margin_px + 2` pixels
//! past it on any side. The recipes are literal values, printed by the
//! generator; where they came from is in MC-054's `## Test plan`.

mod common;

use common::{Border, Chrome, Layout, Recipe};
use cropper_core::{Luma, Rect, Tuning, detect};

/// `inner` grown by `margin` on every side, clamped to `img`: the property
/// test's `expanded`, verbatim in effect.
fn expanded(inner: Rect, img: &Luma, margin: u32) -> Rect {
    let x = inner.x.saturating_sub(margin);
    let y = inner.y.saturating_sub(margin);
    let right = (inner.x + inner.w + margin).min(img.width);
    let bottom = (inner.y + inner.h + margin).min(img.height);
    Rect {
        x,
        y,
        w: right - x,
        h: bottom - y,
    }
}

fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// The art rows `crop` leaves out, as `(above, below)`: how many art rows
/// above the crop's top and below its bottom. `(0, 0)` when none are cut.
fn rows_cut(crop: Rect, art: Rect) -> (u32, u32) {
    let above = crop.y.saturating_sub(art.y).min(art.h);
    let below = (art.y + art.h).saturating_sub(crop.y + crop.h).min(art.h);
    (above, below)
}

/// The property test's two assertions, on one recipe, at `Tuning::default()`.
/// `what` names the recipe in the failure message.
fn assert_holds_the_art_and_little_more(recipe: &Recipe, what: &str) {
    let t = Tuning::default();
    let img = recipe.render();
    let art = recipe.art_rect();
    let found = detect(&img, &t).unwrap_or_else(|| {
        panic!("{what}: the generator's art is textured, so `detect` must find a rect")
    });
    let (above, below) = rows_cut(found.rect, art);
    assert!(
        contains(found.rect, art),
        "{what}: the crop cuts the art - {above} art rows lost above and {below} below \
         (art {art:?}, crop {:?}, {}x{} image)",
        found.rect,
        img.width,
        img.height
    );
    let limit = expanded(art, &img, t.margin_px + 2);
    assert!(
        contains(limit, found.rect),
        "{what}: the crop {:?} reaches past the art {art:?} plus {} px ({limit:?}) - \
         junk kept",
        found.rect,
        t.margin_px + 2
    );
}

// --- AC-1 -------------------------------------------------------------------

/// AC-1's recipe: `detect.rs`'s property test drew it on 2026-09-25 (proptest
/// seed `cc 3deeecef...`, shrunk). The page column is a 156 px strip of art
/// between a left border (140 px on 127) and a right border (183 px on 137),
/// each with a textured chrome band against the art, and the right band's flat
/// fraction is 0.951: over `PAGE_LIKE`. On `c004d96` and on `3449baa` the crop
/// is `147,0 162x186` against art `150,0 156x206`: the viewport ends 20 rows
/// above the art's last row.
fn ac1_recipe() -> Recipe {
    Recipe {
        art_w: 156,
        art_h: 206,
        top: Border {
            thickness: 0,
            colour: 0,
            spread: 0,
        },
        right: Border {
            thickness: 183,
            colour: 137,
            spread: 0,
        },
        bottom: Border {
            thickness: 0,
            colour: 0,
            spread: 0,
        },
        left: Border {
            thickness: 140,
            colour: 127,
            spread: 0,
        },
        layout: Layout::Bands,
        seed: 2_647_894_579,
        top_chrome: vec![],
        bottom_chrome: vec![],
        left_chrome: vec![Chrome {
            thickness: 10,
            background: 68,
            deviation: 20,
            flat_fraction: 0.86,
        }],
        right_chrome: vec![Chrome {
            thickness: 26,
            background: 108,
            deviation: 20,
            flat_fraction: 0.951_421_261_619_281_9,
        }],
    }
}

#[test]
fn a_page_between_two_borders_and_near_page_like_chrome_keeps_its_last_twenty_rows() {
    let recipe = ac1_recipe();
    // The recipe is the one the story names, not a near twin: its geometry.
    let img = recipe.render();
    assert_eq!(
        (img.width, img.height, recipe.art_rect()),
        (
            515,
            206,
            Rect {
                x: 150,
                y: 0,
                w: 156,
                h: 206
            }
        ),
        "AC-1's recipe must render the story's 515x206 image with art 150,0 156x206"
    );
    assert_holds_the_art_and_little_more(
        &recipe,
        "AC-1 (the 2026-09-25 recipe: art 150,0 156x206 on 515x206)",
    );
}

// --- AC-2 -------------------------------------------------------------------

/// MC-052 `## Notes`, "GATES: the proptest clip": seed 107 case 3690 of the
/// stress harness, the Gutters layout. MC-048's whole-margin reading finds its
/// viewport in the top border, rows 0..103, clear of the page column's rows;
/// the pipeline declines on such a viewport. Read over the reader's window
/// instead (an 8 px non-flat left border dropped, 142 columns of chrome at
/// 0.886 and the flat right border kept), rows wobble around 0.904, and
/// widening that border viewport carried it to row 730 of an art ending at
/// 757. It is held by MC-052's rule "only a viewport that overlaps the
/// column's rows is widened" (`locate`'s overlap check).
fn seed_107_case_3690() -> Recipe {
    Recipe {
        art_w: 529,
        art_h: 654,
        top: Border {
            thickness: 103,
            colour: 88,
            spread: 0,
        },
        right: Border {
            thickness: 27,
            colour: 87,
            spread: 0,
        },
        bottom: Border {
            thickness: 53,
            colour: 136,
            spread: 0,
        },
        left: Border {
            thickness: 8,
            colour: 40,
            spread: 0,
        },
        layout: Layout::Gutters,
        seed: 1_812_840_014,
        top_chrome: vec![],
        bottom_chrome: vec![],
        left_chrome: vec![Chrome {
            thickness: 142,
            background: 85,
            deviation: 20,
            flat_fraction: 0.885_619_844_751_822,
        }],
        right_chrome: vec![],
    }
}

/// MC-052 `## Notes`: seed 7 case 1458 of the stress harness, "a viewport in a
/// solid border above the art". MC-048's reading finds a page-like run in the
/// top border (102 px on 32), clear of the column's rows; a window reading
/// taken over every row carried it a few rows into the art, and the crop was
/// those rows (`202,99 198x5`).
fn seed_7_case_1458() -> Recipe {
    Recipe {
        art_w: 192,
        art_h: 219,
        top: Border {
            thickness: 102,
            colour: 32,
            spread: 0,
        },
        right: Border {
            thickness: 126,
            colour: 174,
            spread: 0,
        },
        bottom: Border {
            thickness: 40,
            colour: 155,
            spread: 0,
        },
        left: Border {
            thickness: 161,
            colour: 5,
            spread: 0,
        },
        layout: Layout::Bands,
        seed: 2_934_397_223,
        top_chrome: vec![],
        bottom_chrome: vec![],
        left_chrome: vec![
            Chrome {
                thickness: 23,
                background: 53,
                deviation: 20,
                flat_fraction: 0.872_851_900_909_354,
            },
            Chrome {
                thickness: 21,
                background: 143,
                deviation: 20,
                flat_fraction: 0.955_546_595_649_206_5,
            },
        ],
        right_chrome: vec![Chrome {
            thickness: 23,
            background: 39,
            deviation: 20,
            flat_fraction: 0.875_333_395_682_150_1,
        }],
    }
}

/// MC-052 `## Notes`: seed 103 case 1539 of the stress harness, one of the two
/// clips MC-048's own stage made (23 art rows cut at the top). A regression
/// recipe with no rule claim (MC-054 AC-2, amended twice). On `3449baa` it went
/// red with MC-052's rule 3 removed (crop `14,398 486x408` against art
/// `17,375 480x428`), but MC-054's stage rescues it by its own rule as well,
/// so on the shipped stage it no longer notices rule 3 gone. Rule 3's pin is
/// [`seed_104_case_191`].
fn seed_103_case_1539() -> Recipe {
    Recipe {
        art_w: 480,
        art_h: 428,
        top: Border {
            thickness: 130,
            colour: 233,
            spread: 0,
        },
        right: Border {
            thickness: 188,
            colour: 144,
            spread: 0,
        },
        bottom: Border {
            thickness: 72,
            colour: 16,
            spread: 0,
        },
        left: Border {
            thickness: 17,
            colour: 250,
            spread: 0,
        },
        layout: Layout::Gutters,
        seed: 4_211_171_728,
        top_chrome: vec![
            Chrome {
                thickness: 121,
                background: 82,
                deviation: 20,
                flat_fraction: 0.971_153_205_600_34,
            },
            Chrome {
                thickness: 124,
                background: 172,
                deviation: 20,
                flat_fraction: 0.950_119_289_196_669_7,
            },
        ],
        bottom_chrome: vec![
            Chrome {
                thickness: 77,
                background: 139,
                deviation: 20,
                flat_fraction: 0.911_320_234_430_234_6,
            },
            Chrome {
                thickness: 105,
                background: 229,
                deviation: 20,
                flat_fraction: 0.925_528_922_931_250_3,
            },
        ],
        left_chrome: vec![],
        right_chrome: vec![Chrome {
            thickness: 89,
            background: 144,
            deviation: 20,
            flat_fraction: 0.862_932_398_328_746_5,
        }],
    }
}

/// MC-052 `## Notes`: seed 104 case 191 of the stress harness, the other of
/// MC-048's own two clips there. It pins MC-052's rule 3 on the shipped stage:
/// the reader's window is found from the columns that are page background
/// over the rows of the whole-margin viewport (`reader_window`'s majority over
/// `top..bottom`). Read over every row instead, the crop on the MC-054 stage
/// (`ee1a151`) is `83,390 1084x795` against art `86,390 1078x813`, 18 art rows
/// lost below (MC-054 `## Regressions`, Return to RED from GREEN).
fn seed_104_case_191() -> Recipe {
    Recipe {
        art_w: 1078,
        art_h: 813,
        top: Border {
            thickness: 8,
            colour: 72,
            spread: 0,
        },
        right: Border {
            thickness: 8,
            colour: 119,
            spread: 0,
        },
        bottom: Border {
            thickness: 178,
            colour: 123,
            spread: 0,
        },
        left: Border {
            thickness: 29,
            colour: 85,
            spread: 0,
        },
        layout: Layout::Bands,
        seed: 4_229_688_960,
        top_chrome: vec![
            Chrome {
                thickness: 118,
                background: 126,
                deviation: 20,
                flat_fraction: 0.883_685_788_521_287_5,
            },
            Chrome {
                thickness: 264,
                background: 216,
                deviation: 20,
                flat_fraction: 0.866_621_555_859_456_4,
            },
        ],
        bottom_chrome: vec![
            Chrome {
                thickness: 220,
                background: 127,
                deviation: 20,
                flat_fraction: 0.936_766_847_801_378_6,
            },
            Chrome {
                thickness: 115,
                background: 217,
                deviation: 20,
                flat_fraction: 0.890_608_177_087_83,
            },
        ],
        left_chrome: vec![Chrome {
            thickness: 57,
            background: 86,
            deviation: 20,
            flat_fraction: 0.973_496_365_637_986_6,
        }],
        right_chrome: vec![],
    }
}

#[test]
fn a_viewport_found_in_the_top_border_is_not_widened_into_a_gutters_page() {
    assert_holds_the_art_and_little_more(
        &seed_107_case_3690(),
        "AC-2, MC-052 seed 107 case 3690 (art 150,103 529x654 on 706x810)",
    );
}

#[test]
fn a_viewport_found_in_a_solid_border_above_the_art_does_not_become_the_crop() {
    assert_holds_the_art_and_little_more(
        &seed_7_case_1458(),
        "AC-2, MC-052 seed 7 case 1458 (art 205,102 192x219 on 546x361)",
    );
}

#[test]
fn the_reader_window_read_over_the_viewport_rows_keeps_the_top_of_a_gutters_page() {
    assert_holds_the_art_and_little_more(
        &seed_103_case_1539(),
        "AC-2, MC-052 seed 103 case 1539 (art 17,375 480x428 on 774x1057)",
    );
}

#[test]
fn the_reader_window_found_over_the_viewport_rows_keeps_the_foot_of_a_bands_page() {
    assert_holds_the_art_and_little_more(
        &seed_104_case_191(),
        "AC-2, MC-052 seed 104 case 191 (art 86,390 1078x813 on 1172x1716)",
    );
}

// --- AC-4 (amended): the recipes seeds 201-232 clip on `main` -----------------

/// Seed 207 case 464 of the stress harness (MC-054 `## Notes`, AC-4). A Bands
/// page 1118 rows tall under a 250 px top band at 0.916 and over two bottom
/// bands at 0.871 and 0.884; on `3449baa` the crop is `353,1262 826x265`
/// against art `356,406 823x1118`: 856 art rows lost at the top.
fn seed_207_case_464() -> Recipe {
    Recipe {
        art_w: 823,
        art_h: 1118,
        top: Border {
            thickness: 156,
            colour: 121,
            spread: 0,
        },
        right: Border {
            thickness: 0,
            colour: 227,
            spread: 0,
        },
        bottom: Border {
            thickness: 112,
            colour: 232,
            spread: 0,
        },
        left: Border {
            thickness: 93,
            colour: 19,
            spread: 0,
        },
        layout: Layout::Bands,
        seed: 4_262_505_740,
        top_chrome: vec![Chrome {
            thickness: 250,
            background: 71,
            deviation: 20,
            flat_fraction: 0.916_483_381_192_210_1,
        }],
        bottom_chrome: vec![
            Chrome {
                thickness: 110,
                background: 26,
                deviation: 20,
                flat_fraction: 0.870_986_669_377_179_5,
            },
            Chrome {
                thickness: 282,
                background: 116,
                deviation: 20,
                flat_fraction: 0.883_983_802_875_407_5,
            },
        ],
        left_chrome: vec![Chrome {
            thickness: 263,
            background: 21,
            deviation: 20,
            flat_fraction: 0.866_326_200_734_376_4,
        }],
        right_chrome: vec![],
    }
}

/// Seed 217 case 884 of the stress harness. On `3449baa` the crop is
/// `86,126 144x916` against art `89,126 138x925`: 9 art rows lost at the
/// bottom.
fn seed_217_case_884() -> Recipe {
    Recipe {
        art_w: 138,
        art_h: 925,
        top: Border {
            thickness: 40,
            colour: 0,
            spread: 0,
        },
        right: Border {
            thickness: 191,
            colour: 174,
            spread: 0,
        },
        bottom: Border {
            thickness: 87,
            colour: 16,
            spread: 0,
        },
        left: Border {
            thickness: 61,
            colour: 181,
            spread: 0,
        },
        layout: Layout::Bands,
        seed: 1_548_026_652,
        top_chrome: vec![Chrome {
            thickness: 86,
            background: 110,
            deviation: 20,
            flat_fraction: 0.957_422_903_331_831_6,
        }],
        bottom_chrome: vec![
            Chrome {
                thickness: 46,
                background: 45,
                deviation: 20,
                flat_fraction: 0.926_464_231_288_291_1,
            },
            Chrome {
                thickness: 214,
                background: 135,
                deviation: 20,
                flat_fraction: 0.876_876_402_837_218_8,
            },
        ],
        left_chrome: vec![
            Chrome {
                thickness: 19,
                background: 82,
                deviation: 20,
                flat_fraction: 0.919_525_585_335_332_4,
            },
            Chrome {
                thickness: 9,
                background: 172,
                deviation: 20,
                flat_fraction: 0.961_446_630_997_874_6,
            },
        ],
        right_chrome: vec![Chrome {
            thickness: 9,
            background: 120,
            deviation: 20,
            flat_fraction: 0.958_698_347_580_336_8,
        }],
    }
}

/// Seed 227 case 1996 of the stress harness: a Gutters page with only two
/// thin right chrome bands, at 0.915 and 0.912. On `3449baa` the crop is
/// `92,181 199x5` against art `95,184 193x690` - a viewport at the top border
/// that overlaps the art by two rows, so MC-052's overlap rule does not stop
/// it, and the page is cropped to its first two rows.
fn seed_227_case_1996() -> Recipe {
    Recipe {
        art_w: 193,
        art_h: 690,
        top: Border {
            thickness: 184,
            colour: 109,
            spread: 0,
        },
        right: Border {
            thickness: 48,
            colour: 121,
            spread: 0,
        },
        bottom: Border {
            thickness: 139,
            colour: 38,
            spread: 0,
        },
        left: Border {
            thickness: 95,
            colour: 116,
            spread: 0,
        },
        layout: Layout::Gutters,
        seed: 402_128_851,
        top_chrome: vec![],
        bottom_chrome: vec![],
        left_chrome: vec![],
        right_chrome: vec![
            Chrome {
                thickness: 16,
                background: 30,
                deviation: 20,
                flat_fraction: 0.914_742_317_383_591_9,
            },
            Chrome {
                thickness: 10,
                background: 120,
                deviation: 20,
                flat_fraction: 0.912_190_065_401_801_3,
            },
        ],
    }
}

#[test]
fn a_tall_bands_page_under_near_page_like_chrome_keeps_its_top_856_rows() {
    assert_holds_the_art_and_little_more(
        &seed_207_case_464(),
        "AC-4, seed 207 case 464 (art 356,406 823x1118 on 1179x2028)",
    );
}

#[test]
fn a_narrow_bands_page_between_stacked_chrome_keeps_its_last_nine_rows() {
    assert_holds_the_art_and_little_more(
        &seed_217_case_884(),
        "AC-4, seed 217 case 884 (art 89,126 138x925 on 427x1398)",
    );
}

#[test]
fn a_viewport_in_the_top_border_that_grazes_the_art_does_not_crop_a_gutters_page_to_five_rows() {
    assert_holds_the_art_and_little_more(
        &seed_227_case_1996(),
        "AC-4, seed 227 case 1996 (art 95,184 193x690 on 362x1013)",
    );
}
