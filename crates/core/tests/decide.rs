//! MC-007, AC-1 to AC-6: uncertain images are flagged with a reason.
//!
//! `decide` is the layer above `detect` (MC-006): it takes the same pixels and
//! the same tuning, asks `detect` what it found, and turns that into the one
//! answer the engine acts on - crop to this rect, or flag this file with one
//! of the four discrete reasons in `docs/wiki/architecture.md` ("Data model",
//! and decision 3). It adds no heuristic of its own; every input to the
//! decision is a field `Detection` already carries.
//!
//! # The order is the contract
//!
//! AC-6 fixes the order the reasons are checked in, and it is **AC-1, AC-2,
//! AC-4, AC-3** - `Uniform`, then `NoBorderFound`, then `Ambiguous`, then
//! `LowContent`. Three of the four adjacent pairs are constructible and each
//! has a test below:
//!
//! * **1 before 4/3** - a uniform image is `None` from `detect`, so there is
//!   no rect for any later check to look at. What makes it a real ordering
//!   test rather than a vacuous one is that a uniform image can also be
//!   smaller than `min_content_side`, and it must still come back `Uniform`;
//! * **2 before 4** - a screenshot with no border, nothing peeled and a top
//!   strip that is *nearly* chrome has `!trimmed && removed.is_empty()` and
//!   `ambiguous` both true. The order says `NoBorderFound`. This is the pair
//!   that discriminates position 2 from position 3 and it is pinned below;
//! * **4 before 3** - the pair AC-6 names, an image that is both ambiguous and
//!   low-content. `Ambiguous` wins.
//!
//! The fourth pair, **2 against 3**, is unconstructible: if nothing was
//! trimmed and nothing was peeled then the rect is the whole image, so its
//! area is 100% of the image's and no side can be short of the whole image's.
//! Nothing below pretends to test it.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: `min_content_fraction = 0.20` and `min_content_side = 64`.
//!   Each is pinned by its own test, and neither is ever written as a literal
//!   at a comparison site - the tests read them out of the `Tuning` argument,
//!   and one test below drives `decide` at *different* limits to pin that they
//!   come from the argument rather than from the source.
//! * **Mechanical**: `decide(&Luma, &Tuning) -> CropDecision`,
//!   `CropDecision { Crop(Rect), Flag(FlagReason) }`,
//!   `FlagReason { Uniform, NoBorderFound, LowContent, Ambiguous }`, their
//!   derives, and the order above. Pinned exactly.
//! * **Measured**: only the fixtures. The generator in `common/` knows the
//!   true art rect, so every expectation here is exact. AC-3's four controls
//!   are the one place a fixture has to land on a *number* rather than a rect,
//!   and `every_low_content_fixture_has_the_geometry_its_control_claims`
//!   measures all of them - so a fixture that drifts fails there, loudly,
//!   instead of quietly turning a control into a tautology. Every measured
//!   value is in the story's Handoff.
//!
//! # The one arithmetic note for whoever implements this
//!
//! `min_content_fraction` is an `f32` field, and
//! `a_rect_at_exactly_min_content_fraction_is_not_below_it_and_is_cropped`
//! sits **exactly** on the boundary: its rect is 32,000 px of a 160,000 px
//! image, and AC-3 flags a rect whose area is *below* the fraction, so 20% of
//! 20% must be cropped. Do the arithmetic in `f32`, as `tests/content.rs`
//! already does for `chrome_flat_fraction`: `32000f32 / 160000f32` is bit for
//! bit `0.20f32`, so `<` is false and the rect is kept, whereas the same
//! quotient taken in `f64` - or `f64::from(0.20f32) * 160000.0` - lands a few
//! ulps the other side of the constant and the boundary moves without anyone
//! touching a number. Both quantities are pinned here as whole pixel counts
//! rather than as a fraction, so the test never has to make that choice
//! itself.

mod common;

use common::{Border, Chrome, Layout, Recipe, flat_band};
// MC-025 AC-3. The rest of this file is MC-007's.
use common::{FADE_GUTTER, FADE_H, fade_core_rect, fade_to_gutter};
use cropper_core::content::Side;
use cropper_core::{CropDecision, FlagReason, Luma, Rect, Tuning, decide, detect};

// --- Fixture constants ------------------------------------------------------
//
// None of these is a threshold. Thresholds are read from `Tuning::default()`,
// or from the `Tuning` under test, at every use; these are the sizes, tones
// and flat fractions of the synthetic screenshots.

/// The tone of the uniform images AC-1 is about. Any value with headroom for
/// a spread on top of it would do.
const UNIFORM_TONE: u8 = 120;

/// The chrome background of every band below: a light toolbar.
const BAND_BG: u8 = 200;

/// The four border colours, one per side, all distinct from each other and
/// from the art's two tones (40..=60 and 200..=220).
const BORDER_TOP: u8 = 255;
const BORDER_RIGHT: u8 = 0;
const BORDER_BOTTOM: u8 = 128;
const BORDER_LEFT: u8 = 64;

/// A band flat enough to be chrome: comfortably above `chrome_flat_fraction`,
/// so the scenes that use it test `decide` rather than MC-005's threshold.
const CHROME_FLAT: f64 = 0.90;

/// A band inside `[chrome_flat_fraction - ambiguity_band,
/// chrome_flat_fraction)`, which is the shape `tests/content.rs` already uses
/// for an ambiguous strip. Kept, and marks the detection ambiguous.
const AMBIGUOUS_FLAT: f64 = 0.84;

/// The same band one step *below* the foot of the ambiguity band: plain
/// content, kept without any close call. Every ambiguity test below has a
/// twin at this flat fraction, and the twin is what makes the ambiguity test
/// a test of ambiguity rather than of the geometry it shares.
const PLAIN_CONTENT_FLAT: f64 = 0.79;

// --- Fixture builders -------------------------------------------------------

/// A screenshot: textured art, optionally under a chrome band, inside four
/// solid borders `side_x` px wide and `side_y` px tall.
///
/// Every border is at least `margin_px` thick in the scenes that use it, so
/// `detect`'s outward margin never clamps and the rect it returns is exactly
/// the art (or the art plus a kept band) grown by the margin on all four
/// sides. That is what makes AC-3's pixel counts predictable.
fn bordered(art_w: u32, art_h: u32, side_x: u32, side_y: u32, chrome: Vec<Chrome>) -> Recipe {
    Recipe {
        top: Border::solid(side_y, BORDER_TOP),
        right: Border::solid(side_x, BORDER_RIGHT),
        bottom: Border::solid(side_y, BORDER_BOTTOM),
        left: Border::solid(side_x, BORDER_LEFT),
        top_chrome: chrome,
        ..Recipe::new(art_w, art_h)
    }
}

/// AC-3's area controls. All three are 400x400 images whose detected rect is
/// the art grown by the margin, so the area lands on a whole percentage of the
/// image: 19% (flagged), exactly 20% (the boundary, cropped) and 21%
/// (cropped). Both sides of every one of them are far above
/// `min_content_side`, so the area is the only thing that can decide them.
fn area_at_nineteen_percent() -> Luma {
    bordered(154, 184, 123, 108, vec![]).render()
}
fn area_at_exactly_twenty_percent() -> Luma {
    bordered(154, 194, 123, 103, vec![]).render()
}
fn area_at_twenty_one_percent() -> Luma {
    bordered(162, 194, 119, 103, vec![]).render()
}

/// AC-3's side controls: a 300 px wide rect 63 px tall (flagged) and the same
/// one 64 px tall (cropped). Both fill over 70% of their image, so the area is
/// nowhere near deciding them and the short side is the only thing that can.
fn side_at_sixty_three_px() -> Luma {
    bordered(294, 57, 13, 13, vec![]).render()
}
fn side_at_sixty_four_px() -> Luma {
    bordered(294, 58, 13, 13, vec![]).render()
}

/// AC-4's scene: a 20 px band at `flat` above textured art, inside four
/// borders. At [`AMBIGUOUS_FLAT`] the band is kept and the detection is
/// ambiguous; at [`PLAIN_CONTENT_FLAT`] the band is kept and it is not; at
/// [`CHROME_FLAT`] the band is peeled. The rect and every other report field
/// are identical in the first two cases, which is what makes them a pair.
fn band_over_art(flat: f64) -> Luma {
    bordered(200, 150, 20, 20, vec![Chrome::new(20, BAND_BG, flat)]).render()
}

/// AC-6's scene: [`band_over_art`] with the art cut down to 20 px tall and the
/// band to 6, so the detected rect is 32 px tall - below `min_content_side` -
/// while still filling 41% of the image. Low-content on the side alone, and
/// ambiguous or not according to `flat`.
fn short_band_over_short_art(flat: f64) -> Luma {
    bordered(200, 20, 20, 20, vec![Chrome::new(6, BAND_BG, flat)]).render()
}

/// The 2-before-4 scene: no borders at all, and a band at `flat` over the art.
/// Nothing is trimmed (no edge row or column of this image is uniform) and
/// nothing is peeled (the band misses `chrome_flat_fraction`), so the rect is
/// the whole image and `!trimmed && removed.is_empty()` holds - while
/// `ambiguous` is true.
fn unbordered_band_over_art(flat: f64) -> Luma {
    Recipe {
        top_chrome: vec![Chrome::new(20, BAND_BG, flat)],
        ..Recipe::new(200, 150)
    }
    .render()
}

/// The rect covering the whole of `img`.
fn whole(img: &Luma) -> Rect {
    Rect {
        x: 0,
        y: 0,
        w: img.width,
        h: img.height,
    }
}

/// The detected rect's area and the image's, as exact pixel counts.
///
/// Never a ratio. AC-3's boundary fixture sits exactly on
/// `min_content_fraction`, and a test that computed the fraction would have to
/// pick `f32` or `f64` - which is the very choice the implementation has to
/// make and the test has no business pre-empting. Two whole numbers say the
/// same thing without taking a side.
fn areas(img: &Luma, rect: Rect) -> (u64, u64) {
    (
        u64::from(rect.w) * u64::from(rect.h),
        u64::from(img.width) * u64::from(img.height),
    )
}

/// What `detect` returned for `img` at the default tuning, unwrapped.
fn detected(img: &Luma) -> cropper_core::Detection {
    detect(img, &Tuning::default()).expect("this fixture is textured, so it is never uniform")
}

// --- The settled constants --------------------------------------------------

/// `min_content_fraction` is settled at 0.20 (`architecture.md`, the `Tuning`
/// table; the corpus story MC-019 may change it under `## Amendments`). Every
/// area fixture in this file is sized around this number *because of this
/// test*, not because 0.20 was assumed.
#[test]
fn the_default_min_content_fraction_is_zero_point_two() {
    assert_eq!(
        Tuning::default().min_content_fraction,
        0.20,
        "the settled smallest content box, as a share of the image area"
    );
}

/// `min_content_side` is settled at 64 px, and `architecture.md` records it as
/// fixed rather than tuned by the corpus.
#[test]
fn the_default_min_content_side_is_sixty_four() {
    assert_eq!(
        Tuning::default().min_content_side,
        64,
        "the settled smallest content box side, in pixels"
    );
}

// --- The exported shapes ----------------------------------------------------

#[test]
fn a_crop_decision_is_either_a_crop_carrying_a_rect_or_a_flag_carrying_a_reason() {
    let rect = Rect {
        x: 17,
        y: 37,
        w: 206,
        h: 156,
    };
    let crop = CropDecision::Crop(rect);
    let flag = CropDecision::Flag(FlagReason::LowContent);

    // Two variants, each carrying exactly one payload, reachable by `match`.
    match &crop {
        CropDecision::Crop(got) => assert_eq!(*got, rect),
        CropDecision::Flag(reason) => panic!("a Crop matched as Flag({reason:?})"),
    }
    match &flag {
        CropDecision::Crop(got) => panic!("a Flag matched as Crop({got:?})"),
        CropDecision::Flag(reason) => assert_eq!(*reason, FlagReason::LowContent),
    }

    // Debug, so a failing assertion anywhere below prints something readable.
    assert!(format!("{crop:?}").contains("Crop"), "{crop:?}");
    assert!(format!("{flag:?}").contains("LowContent"), "{flag:?}");

    // Clone, PartialEq and Eq.
    #[allow(clippy::clone_on_copy, clippy::redundant_clone)]
    let cloned = crop.clone();
    assert_eq!(cloned, crop);
    assert_ne!(crop, flag);
    fn requires_eq<T: Eq>(_: &T) {}
    requires_eq(&crop);
    requires_eq(&FlagReason::Uniform);
}

#[test]
fn the_four_flag_reasons_are_four_distinct_values() {
    let all = [
        FlagReason::Uniform,
        FlagReason::NoBorderFound,
        FlagReason::LowContent,
        FlagReason::Ambiguous,
    ];
    let mut wrong = Vec::new();
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if (a == b) != (i == j) {
                wrong.push(format!("{a:?} and {b:?} compare {}", a == b));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "each reason must compare equal only to itself: {wrong:?}"
    );
}

/// MC-011 serialises the run summary, and both of these types go into it. The
/// plain derive is what the story asks for and the plain derive is what this
/// pins: unit variants as their own names, an externally tagged wrapper for
/// the payload. No `#[serde(...)]` attribute anywhere, on any of the three
/// types involved.
#[test]
fn a_crop_decision_and_a_flag_reason_serialise_by_the_plain_serde_derive() {
    fn requires_serialize<T: serde::Serialize>(_: &T) {}
    requires_serialize(&FlagReason::Uniform);
    requires_serialize(&CropDecision::Flag(FlagReason::Uniform));

    let mut wrong = Vec::new();
    for (reason, want) in [
        (FlagReason::Uniform, "\"Uniform\""),
        (FlagReason::NoBorderFound, "\"NoBorderFound\""),
        (FlagReason::LowContent, "\"LowContent\""),
        (FlagReason::Ambiguous, "\"Ambiguous\""),
    ] {
        // Taken before the reason is moved into the wrapper below: `Copy` is
        // deliberately not among the derives this story asks for, so nothing
        // here may assume it.
        let shown = format!("{reason:?}");
        let got = serde_json::to_string(&reason).expect("a flag reason must serialise");
        if got != want {
            wrong.push(format!("{shown}: wanted {want}, got {got}"));
        }
        let wrapped = serde_json::to_string(&CropDecision::Flag(reason))
            .expect("a flagged decision must serialise");
        let want_wrapped = format!("{{\"Flag\":{want}}}");
        if wrapped != want_wrapped {
            wrong.push(format!(
                "Flag({shown}): wanted {want_wrapped}, got {wrapped}"
            ));
        }
    }
    let crop = CropDecision::Crop(Rect {
        x: 17,
        y: 37,
        w: 206,
        h: 156,
    });
    let got = serde_json::to_string(&crop).expect("a crop decision must serialise");
    let want = "{\"Crop\":{\"x\":17,\"y\":37,\"w\":206,\"h\":156}}";
    if got != want {
        wrong.push(format!("Crop: wanted {want}, got {got}"));
    }
    assert!(
        wrong.is_empty(),
        "the plain serde derive is what MC-011's summary will carry: {wrong:?}"
    );
}

// --- The fixtures are what the controls claim they are ----------------------

/// AC-3's four controls are the only place in this file where a fixture has to
/// hit a *number*, and `detect` grows every rect by `margin_px` on its way
/// out - so "a rect at 19% of the area" is a claim about the rect after the
/// margin, not about the art. This measures all five area/side fixtures: the
/// rect `detect` actually returned, its area against the image's, and its
/// shorter side. A fixture that drifts fails here, by name, instead of turning
/// one of the controls below into a tautology.
#[test]
fn every_low_content_fixture_has_the_geometry_its_control_claims() {
    let t = Tuning::default();
    // (name, image, rect, rect area, image area, short side, long side)
    let cases: [(&str, Luma, Rect, u64, u64, u32, u32); 5] = [
        (
            "19% of the area",
            area_at_nineteen_percent(),
            Rect {
                x: 120,
                y: 105,
                w: 160,
                h: 190,
            },
            30_400,
            160_000,
            160,
            190,
        ),
        (
            "exactly 20% of the area",
            area_at_exactly_twenty_percent(),
            Rect {
                x: 120,
                y: 100,
                w: 160,
                h: 200,
            },
            32_000,
            160_000,
            160,
            200,
        ),
        (
            "21% of the area",
            area_at_twenty_one_percent(),
            Rect {
                x: 116,
                y: 100,
                w: 168,
                h: 200,
            },
            33_600,
            160_000,
            168,
            200,
        ),
        (
            "a 63 px side",
            side_at_sixty_three_px(),
            Rect {
                x: 10,
                y: 10,
                w: 300,
                h: 63,
            },
            18_900,
            26_560,
            63,
            300,
        ),
        (
            "a 64 px side",
            side_at_sixty_four_px(),
            Rect {
                x: 10,
                y: 10,
                w: 300,
                h: 64,
            },
            19_200,
            26_880,
            64,
            300,
        ),
    ];

    let mut drifted = Vec::new();
    for (name, img, rect, area, total, short, long) in cases {
        let found = detected(&img);
        if found.rect != rect {
            drifted.push(format!(
                "{name}: wanted rect {rect:?}, got {:?}",
                found.rect
            ));
            continue;
        }
        if areas(&img, found.rect) != (area, total) {
            drifted.push(format!(
                "{name}: wanted {area} of {total} px, got {:?}",
                areas(&img, found.rect)
            ));
        }
        if (
            found.rect.w.min(found.rect.h),
            found.rect.w.max(found.rect.h),
        ) != (short, long)
        {
            drifted.push(format!(
                "{name}: wanted sides {short}x{long}, got {}x{}",
                found.rect.w, found.rect.h
            ));
        }
        // The other three report fields must all be quiet, or one of these
        // fixtures would be decided by a criterion ahead of AC-3 in the order
        // and would stop being an AC-3 control at all.
        if !found.trimmed || !found.removed.is_empty() || found.ambiguous {
            drifted.push(format!(
                "{name}: must be decided by AC-3 alone, but trimmed {} removed {:?} \
                 ambiguous {}",
                found.trimmed, found.removed, found.ambiguous
            ));
        }
    }
    assert!(
        drifted.is_empty(),
        "an AC-3 control fixture is no longer the geometry it claims, so the \
         {} / {} px limits it brackets are not being tested: {drifted:?}",
        t.min_content_fraction,
        t.min_content_side
    );
}

/// The ambiguity pairs differ in **one** thing: the band's flat fraction. Same
/// rect, same trim, same removals - so a test that reads `Ambiguous` off one
/// of them and `Crop` or `LowContent` off its twin is reading the ambiguity
/// and nothing else. Without this the pairs could be agreeing by accident.
#[test]
fn each_ambiguity_pair_differs_only_in_whether_the_detection_is_ambiguous() {
    let mut wrong = Vec::new();
    for (name, ambiguous_img, plain_img) in [
        (
            "a band over art",
            band_over_art(AMBIGUOUS_FLAT),
            band_over_art(PLAIN_CONTENT_FLAT),
        ),
        (
            "a short band over short art",
            short_band_over_short_art(AMBIGUOUS_FLAT),
            short_band_over_short_art(PLAIN_CONTENT_FLAT),
        ),
    ] {
        let close = detected(&ambiguous_img);
        let plain = detected(&plain_img);
        if !close.ambiguous {
            wrong.push(format!(
                "{name} at {AMBIGUOUS_FLAT} must be ambiguous: it is inside \
                 [chrome_flat_fraction - ambiguity_band, chrome_flat_fraction)"
            ));
        }
        if plain.ambiguous {
            wrong.push(format!(
                "{name} at {PLAIN_CONTENT_FLAT} must not be ambiguous: it is below \
                 chrome_flat_fraction - ambiguity_band"
            ));
        }
        if close.rect != plain.rect
            || close.trimmed != plain.trimmed
            || close.removed != plain.removed
        {
            wrong.push(format!(
                "{name}: the twins must differ only in `ambiguous`; got {:?} and {:?}",
                close, plain
            ));
        }
        if !close.removed.is_empty() {
            wrong.push(format!(
                "{name}: a kept band is never a removal; got {:?}",
                close.removed
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:?}");
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn a_uniform_image_is_flagged_uniform() {
    let t = Tuning::default();
    let mut wrong = Vec::new();
    // Zero, one and many: a single pixel, a single row, a single column, two
    // realistic planes; and the whole span of "uniform", from one flat colour
    // up to the tolerance itself, which is inclusive.
    for &(w, h) in &[(1u32, 1u32), (40, 1), (1, 40), (60, 40), (200, 160)] {
        for spread in [0u8, 5, t.uniform_tolerance] {
            let img = flat_band(w, h, UNIFORM_TONE, spread);
            let got = decide(&img, &t);
            if got != CropDecision::Flag(FlagReason::Uniform) {
                wrong.push(format!("{w}x{h} at spread {spread}: {got:?}"));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-1: an image of one flat colour has nothing to crop and must be flagged \
         Uniform: {wrong:?}"
    );
}

/// Position 1 of AC-6's order, and the only form it can take: a uniform image
/// three of whose five sizes above are *also* below `min_content_side`, so an
/// implementation that asked about the size first would answer `LowContent`.
/// It cannot: `detect` returns `None` for these, there is no rect to measure,
/// and `Uniform` is the only answer available.
#[test]
fn a_uniform_image_smaller_than_min_content_side_is_flagged_uniform_not_low_content() {
    let t = Tuning::default();
    let mut wrong = Vec::new();
    for &(w, h) in &[(1u32, 1u32), (40, 1), (60, 40)] {
        assert!(
            w < t.min_content_side || h < t.min_content_side,
            "this scene only discriminates if a side is short of {}",
            t.min_content_side
        );
        let img = flat_band(w, h, UNIFORM_TONE, 0);
        assert!(
            detect(&img, &t).is_none(),
            "AC-1's precondition: a uniform image is `None` from detect"
        );
        let got = decide(&img, &t);
        if got != CropDecision::Flag(FlagReason::Uniform) {
            wrong.push(format!("{w}x{h}: {got:?}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-6: Uniform is checked first, so a uniform image is never flagged for its \
         size: {wrong:?}"
    );
}

/// The control that makes AC-1 mean something: one level past the tolerance
/// the same image is no longer uniform, and `decide` must stop saying so.
/// Without this, a `decide` that returned `Flag(Uniform)` for everything would
/// satisfy AC-1 completely.
#[test]
fn an_image_one_level_past_the_uniform_tolerance_is_not_flagged_uniform() {
    let t = Tuning::default();
    let img = flat_band(200, 160, UNIFORM_TONE, t.uniform_tolerance + 1);
    assert!(
        detect(&img, &t).is_some(),
        "max - min of {} is past a tolerance of {}, so this image is not uniform",
        t.uniform_tolerance + 1,
        t.uniform_tolerance
    );
    assert_ne!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::Uniform),
        "AC-1: only an image detect gives up on is Uniform"
    );
}

// --- AC-2 -------------------------------------------------------------------

#[test]
fn an_all_art_screenshot_is_flagged_no_border_found_even_though_its_rect_is_the_whole_image() {
    let t = Tuning::default();
    let mut wrong = Vec::new();
    for img in [
        Recipe::new(120, 80).render(),
        Recipe::new(200, 160).render(),
        flat_band(200, 160, UNIFORM_TONE, t.uniform_tolerance + 1),
    ] {
        let found = detected(&img);
        // AC-2's precondition, stated as the predicate it is: the trim removed
        // nothing and no strip was peeled - and `detect` still returned a rect,
        // the whole image.
        assert!(
            !found.trimmed && found.removed.is_empty(),
            "AC-2 needs a scene nothing was trimmed from and nothing was peeled off; \
             got trimmed {} removed {:?}",
            found.trimmed,
            found.removed
        );
        assert_eq!(
            found.rect,
            whole(&img),
            "AC-2: detect returns a rect equal to the full image here"
        );
        let got = decide(&img, &t);
        if got != CropDecision::Flag(FlagReason::NoBorderFound) {
            wrong.push(format!("{}x{}: {got:?}", img.width, img.height));
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-2: a rect equal to the whole image is not a crop, it is the detector \
         finding no border at all: {wrong:?}"
    );
}

/// The first half of AC-2's predicate, on its own. Borders were trimmed off
/// this screenshot, so `trimmed` is true while `removed` is still empty - and
/// that alone must take `NoBorderFound` off the table.
#[test]
fn a_screenshot_whose_borders_were_trimmed_is_not_flagged_no_border_found() {
    let t = Tuning::default();
    let img = band_over_art(PLAIN_CONTENT_FLAT);
    let found = detected(&img);
    assert!(
        found.trimmed && found.removed.is_empty(),
        "this control needs trimmed true and removed empty; got {} and {:?}",
        found.trimmed,
        found.removed
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(found.rect),
        "AC-2: something was trimmed, so this screenshot is not all art"
    );
}

/// The second half, on its own. Nothing was trimmed off this screenshot - no
/// edge row or column of it is uniform - but a chrome band was peeled, so
/// `removed` is not empty and `NoBorderFound` is again off the table. Between
/// this test and the one above, neither half of `!trimmed &&
/// removed.is_empty()` can be dropped without something going red.
#[test]
fn a_screenshot_whose_chrome_was_peeled_is_not_flagged_no_border_found() {
    let t = Tuning::default();
    let img = unbordered_band_over_art(CHROME_FLAT);
    let found = detected(&img);
    assert!(
        !found.trimmed && found.removed == vec![Side::Top],
        "this control needs trimmed false and the top band peeled; got {} and {:?}",
        found.trimmed,
        found.removed
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(found.rect),
        "AC-2: a strip was peeled, so this screenshot is not all art"
    );
}

// --- AC-3 -------------------------------------------------------------------

#[test]
fn a_rect_below_min_content_fraction_of_the_image_is_flagged_low_content() {
    let t = Tuning::default();
    let img = area_at_nineteen_percent();
    let found = detected(&img);
    assert_eq!(
        areas(&img, found.rect),
        (30_400, 160_000),
        "AC-3's control: 19% of the image area"
    );
    assert!(
        found.rect.w.min(found.rect.h) > t.min_content_side,
        "both sides must be well inside the {} px limit, so the area is what decides \
         this fixture",
        t.min_content_side
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::LowContent),
        "AC-3: 30400 of 160000 px is 19%, below min_content_fraction"
    );
}

#[test]
fn a_rect_above_min_content_fraction_of_the_image_is_cropped() {
    let t = Tuning::default();
    let img = area_at_twenty_one_percent();
    let found = detected(&img);
    assert_eq!(
        areas(&img, found.rect),
        (33_600, 160_000),
        "AC-3's control: 21% of the image area"
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(found.rect),
        "AC-3: 33600 of 160000 px is 21%, above min_content_fraction"
    );
}

/// The boundary between the two controls above. AC-3 flags a rect whose area
/// is **below** `min_content_fraction`, so a rect at exactly the fraction is
/// not below it and is cropped. See the arithmetic note in this file's header:
/// `f32` keeps this green, `f64` does not, and that is a decision about the
/// implementation rather than about this fixture.
#[test]
fn a_rect_at_exactly_min_content_fraction_is_not_below_it_and_is_cropped() {
    let t = Tuning::default();
    let img = area_at_exactly_twenty_percent();
    let found = detected(&img);
    let (area, total) = areas(&img, found.rect);
    assert_eq!(
        (area, total),
        (32_000, 160_000),
        "AC-3's boundary: exactly min_content_fraction of the image area"
    );
    // The fixture is tied to the settled constant rather than to the number
    // 0.20: its area is exactly `min_content_fraction` of the image's, read
    // out of the tuning. Both counts are powers-of-ten multiples that `f32`
    // represents exactly, so this equality is not a tolerance in disguise.
    assert_eq!(
        area as f32 / total as f32,
        t.min_content_fraction,
        "sanity: {area} of {total} px must be exactly min_content_fraction"
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(found.rect),
        "AC-3: 32000 of 160000 px is exactly min_content_fraction, which is not below it"
    );
}

#[test]
fn a_rect_with_a_side_below_min_content_side_is_flagged_low_content() {
    let t = Tuning::default();
    let img = side_at_sixty_three_px();
    let found = detected(&img);
    let (area, total) = areas(&img, found.rect);
    assert_eq!(
        found.rect.h,
        t.min_content_side - 1,
        "AC-3's control: one pixel short of the side limit"
    );
    assert!(
        found.rect.w >= t.min_content_side,
        "the other dimension must be well inside the limit"
    );
    assert!(
        area * 100 > total * 20,
        "the area must be well above min_content_fraction, so the short side is what \
         decides this fixture: {area} of {total}"
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::LowContent),
        "AC-3: a 63 px side is below min_content_side"
    );
}

#[test]
fn a_rect_with_a_side_at_min_content_side_is_cropped() {
    let t = Tuning::default();
    let img = side_at_sixty_four_px();
    let found = detected(&img);
    assert_eq!(
        found.rect.h, t.min_content_side,
        "AC-3's control: exactly on the side limit"
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(found.rect),
        "AC-3: a 64 px side is not below min_content_side, so it is not low content"
    );
}

/// Both limits come from the `Tuning` argument, not from the source. Each
/// fixture is driven twice, once at the default and once at a limit that
/// reverses its answer - so a `decide` with 0.20 or 64 compiled in fails four
/// of these eight and a `decide` that ignores the limits entirely fails all
/// eight.
#[test]
fn the_low_content_limits_are_read_from_the_tuning_they_were_given() {
    let default = Tuning::default();
    let mut wrong = Vec::new();

    // 21% of the area is cropped at a 0.20 limit and flagged at a 0.25 one.
    let img = area_at_twenty_one_percent();
    let rect = detected(&img).rect;
    for (fraction, want) in [
        (default.min_content_fraction, CropDecision::Crop(rect)),
        (0.25, CropDecision::Flag(FlagReason::LowContent)),
    ] {
        let t = Tuning {
            min_content_fraction: fraction,
            ..Tuning::default()
        };
        let got = decide(&img, &t);
        if got != want {
            wrong.push(format!(
                "21% at min_content_fraction {fraction}: wanted {want:?}, got {got:?}"
            ));
        }
    }

    // A 63 px side is flagged at a 64 px limit and cropped at a 63 px one.
    let img = side_at_sixty_three_px();
    let rect = detected(&img).rect;
    for (side, want) in [
        (
            default.min_content_side,
            CropDecision::Flag(FlagReason::LowContent),
        ),
        (default.min_content_side - 1, CropDecision::Crop(rect)),
    ] {
        let t = Tuning {
            min_content_side: side,
            ..Tuning::default()
        };
        let got = decide(&img, &t);
        if got != want {
            wrong.push(format!(
                "a 63 px side at min_content_side {side}: wanted {want:?}, got {got:?}"
            ));
        }
    }

    // And the same two fixtures the other way round, so neither limit can be
    // satisfied by an implementation that only ever reads one of them.
    let img = area_at_nineteen_percent();
    let rect = detected(&img).rect;
    let t = Tuning {
        min_content_fraction: 0.10,
        ..Tuning::default()
    };
    let got = decide(&img, &t);
    if got != CropDecision::Crop(rect) {
        wrong.push(format!(
            "19% at min_content_fraction 0.10: wanted Crop({rect:?}), got {got:?}"
        ));
    }
    let img = side_at_sixty_four_px();
    let t = Tuning {
        min_content_side: default.min_content_side + 1,
        ..Tuning::default()
    };
    let got = decide(&img, &t);
    if got != CropDecision::Flag(FlagReason::LowContent) {
        wrong.push(format!(
            "a 64 px side at min_content_side {}: wanted Flag(LowContent), got {got:?}",
            default.min_content_side + 1
        ));
    }

    assert!(
        wrong.is_empty(),
        "AC-3's limits must come from the Tuning argument: {wrong:?}"
    );
}

// --- AC-4 -------------------------------------------------------------------

#[test]
fn a_detection_with_a_nearly_chrome_edge_strip_is_flagged_ambiguous() {
    let t = Tuning::default();
    let img = band_over_art(AMBIGUOUS_FLAT);
    let found = detected(&img);
    assert!(
        found.ambiguous,
        "AC-4's precondition: a strip at {AMBIGUOUS_FLAT} is inside \
         [chrome_flat_fraction - ambiguity_band, chrome_flat_fraction)"
    );
    // "Regardless of the rect": this rect is 72% of the image and both its
    // sides are far above the limits, so nothing else in the order fires and
    // the only thing that can produce a flag here is the ambiguity.
    assert_eq!(areas(&img, found.rect), (36_256, 50_400));
    assert!(found.rect.w.min(found.rect.h) > t.min_content_side);
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::Ambiguous),
        "AC-4: detect made a close call on an edge strip, so this file is for review"
    );
}

/// AC-4's twin: the identical scene with the band one step below the foot of
/// the ambiguity band. Same rect, same trim, same (empty) removals, and the
/// only difference in the whole detection is `ambiguous` - so this pair pins
/// that `Flag(Ambiguous)` is read off that field and off nothing else.
#[test]
fn the_same_scene_with_a_strip_below_the_ambiguity_band_is_cropped() {
    let t = Tuning::default();
    let img = band_over_art(PLAIN_CONTENT_FLAT);
    let found = detected(&img);
    assert!(
        !found.ambiguous,
        "the control's precondition: {PLAIN_CONTENT_FLAT} is below \
         chrome_flat_fraction - ambiguity_band"
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(found.rect),
        "AC-4: nothing was a close call here, so there is nothing to flag"
    );
}

// --- AC-5 -------------------------------------------------------------------

#[test]
fn an_ordinary_screenshot_is_cropped_to_the_rect_detect_produced() {
    let t = Tuning::default();
    let mut wrong = Vec::new();
    for (name, img) in [
        ("chrome peeled off four borders", band_over_art(CHROME_FLAT)),
        ("a kept band over art", band_over_art(PLAIN_CONTENT_FLAT)),
        (
            "chrome peeled with no borders",
            unbordered_band_over_art(CHROME_FLAT),
        ),
        ("21% of the area", area_at_twenty_one_percent()),
        ("a 64 px side", side_at_sixty_four_px()),
    ] {
        // The oracle is `detect`'s own return value on the same image and the
        // same tuning, never a rect recomputed here.
        let found = detect(&img, &t).expect("these fixtures are textured");
        let got = decide(&img, &t);
        if got != CropDecision::Crop(found.rect) {
            wrong.push(format!(
                "{name}: wanted Crop({:?}), got {got:?}",
                found.rect
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-5: with nothing to flag, the decision carries exactly the rect detect \
         produced: {wrong:?}"
    );
}

/// `decide` passes the tuning it was given through to `detect` rather than
/// calling it at the defaults. Three margins on one scene, sized so none of
/// them clamps, so the three rects are three different answers - and the
/// oracle at each margin is `detect` at that same margin.
#[test]
fn the_cropped_rect_follows_the_margin_in_the_tuning_it_was_given() {
    let img = band_over_art(CHROME_FLAT);
    let mut seen = Vec::new();
    let mut wrong = Vec::new();
    for margin in [0u32, 3, 7] {
        let t = Tuning {
            margin_px: margin,
            ..Tuning::default()
        };
        let found = detect(&img, &t).expect("this fixture is textured");
        seen.push(found.rect);
        let got = decide(&img, &t);
        if got != CropDecision::Crop(found.rect) {
            wrong.push(format!(
                "margin {margin}: wanted Crop({:?}), got {got:?}",
                found.rect
            ));
        }
    }
    assert!(
        seen[0] != seen[1] && seen[1] != seen[2],
        "the fixture must give a different rect at each margin, or this test cannot \
         tell the tuning was passed through: {seen:?}"
    );
    assert!(
        wrong.is_empty(),
        "AC-5: decide must ask detect the question it was given: {wrong:?}"
    );
}

/// `Out of scope`: which pair of sides spans the image is MC-003's business
/// and `detect` has no opinion about it (MC-006 pinned that). `decide` must
/// not have grown one either.
#[test]
fn the_decision_does_not_depend_on_which_pair_of_borders_spans_the_image() {
    let t = Tuning::default();
    let mut differed = Vec::new();
    // `Copy` is deliberately not among the derives this story asks for, so the
    // expected reason is rebuilt per case rather than carried in the table.
    for (name, chrome, flagged) in [
        ("chrome", CHROME_FLAT, false),
        ("an ambiguous band", AMBIGUOUS_FLAT, true),
    ] {
        let mut answers = Vec::new();
        for layout in [Layout::Bands, Layout::Gutters] {
            let img = Recipe {
                layout,
                ..bordered(200, 150, 20, 20, vec![Chrome::new(20, BAND_BG, chrome)])
            }
            .render();
            let found = detect(&img, &t).expect("this fixture is textured");
            let expected = if flagged {
                CropDecision::Flag(FlagReason::Ambiguous)
            } else {
                CropDecision::Crop(found.rect)
            };
            let got = decide(&img, &t);
            if got != expected {
                differed.push(format!(
                    "{name} in {layout:?}: wanted {expected:?}, got {got:?}"
                ));
            }
            answers.push(got);
        }
        if answers[0] != answers[1] {
            differed.push(format!("{name}: {:?} then {:?}", answers[0], answers[1]));
        }
    }
    assert!(
        differed.is_empty(),
        "the layout must not change the decision: {differed:?}"
    );
}

// --- AC-6: the order ---------------------------------------------------------

/// The pair AC-6 names, position 3 against position 4. The band is ambiguous
/// and the rect it leaves is 32 px tall, half of `min_content_side` - so both
/// criteria fire and only the order can choose between them.
#[test]
fn ambiguous_beats_low_content_on_an_image_that_is_both() {
    let t = Tuning::default();
    let img = short_band_over_short_art(AMBIGUOUS_FLAT);
    let found = detected(&img);
    assert!(found.ambiguous, "AC-6's scene must be ambiguous");
    assert!(
        found.rect.h < t.min_content_side,
        "AC-6's scene must also be low content: a {} px side against a {} px limit",
        found.rect.h,
        t.min_content_side
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::Ambiguous),
        "AC-6: Ambiguous is checked before LowContent, so it wins where both hold"
    );
}

/// The discriminator for the test above. Identical geometry, identical short
/// side, and the band one step below the ambiguity band - so the only reason
/// left is `LowContent`. Without this, `ambiguous_beats_low_content_..` would
/// also pass against a `decide` that never looked at the side at all.
#[test]
fn the_same_short_scene_without_the_ambiguity_is_flagged_low_content() {
    let t = Tuning::default();
    let img = short_band_over_short_art(PLAIN_CONTENT_FLAT);
    let found = detected(&img);
    assert!(!found.ambiguous, "the control must not be ambiguous");
    assert!(found.rect.h < t.min_content_side);
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::LowContent),
        "AC-6: with the ambiguity gone the short side is the reason"
    );
}

/// Position 2 against position 3, which is the pair the order's middle
/// actually rests on. This screenshot has no border, nothing was peeled off it
/// and nothing was trimmed from it - `!trimmed && removed.is_empty()` - while
/// its top strip was *nearly* chrome, so `ambiguous` is true as well. AC-6
/// puts `NoBorderFound` ahead of `Ambiguous`, so `NoBorderFound` is the
/// answer; an implementation that checked ambiguity first would say
/// `Ambiguous` and fail only here.
#[test]
fn no_border_found_beats_ambiguous_on_an_image_that_is_both() {
    let t = Tuning::default();
    let img = unbordered_band_over_art(AMBIGUOUS_FLAT);
    let found = detected(&img);
    assert!(
        !found.trimmed && found.removed.is_empty(),
        "this scene must satisfy AC-2's predicate; got trimmed {} removed {:?}",
        found.trimmed,
        found.removed
    );
    assert!(found.ambiguous, "this scene must also be ambiguous");
    assert_eq!(
        found.rect,
        whole(&img),
        "nothing was trimmed or peeled, so the rect is the whole image"
    );
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::NoBorderFound),
        "AC-6: NoBorderFound is checked before Ambiguous, so it wins where both hold"
    );
}

// --- MC-025 AC-3 ------------------------------------------------------------
//
// The one criterion of MC-025 that goes through `decide`, and it lives here
// rather than in `tests/flatness.rs` on purpose. `flatness.rs` names
// `edges::spread_profile`, which does not exist yet, so that whole target
// fails to compile and **not one assertion in it runs** - which is no
// observation of a failure at all. Everything this test needs (`decide`,
// `Tuning`, `CropDecision`, and the fixture, which is a plain `Luma` built by
// the generator) exists today, so this target still compiles and this
// assertion genuinely goes red. The story's `## Handoff` carries its output.
//
// It asserts an **outcome**, never a route. Where in the pipeline the flatness
// locator is called is GREEN's engineering problem, and a hard one: wiring it
// naively into `content::strip_depth` would locate the 90%-flat band in
// `tests/content.rs::the_edge_threshold_is_read_from_the_tuning` and peel it,
// breaking a frozen assertion and with it MC-025 AC-4. Nothing here constrains
// that choice.

/// AC-3. Art that fades into the gutter over 50 px is cropped to the art.
///
/// Before this story the same fixture comes back `Flag`: the gradient locator
/// finds no step anywhere in the fade (the fixture's peak adjacent-row
/// difference is 1.8 against an `edge_threshold` of 24), the speckled gutter
/// is not uniform so `trim_uniform` will not take it either, and nothing is
/// peeled or trimmed at all.
///
/// The two halves of the criterion are checked as bounds rather than as an
/// exact rect, because **where the art starts inside a fade is the question
/// the story is about** and a test must not presume an answer to it:
///
/// * *contains the art the generator drew* - the full-amplitude core, full
///   width. No reading of the fixture calls any of that gutter;
/// * *excludes the flat gutter beyond it* - the crop reaches no further into
///   either gutter than `margin_px`, which `detect` adds back on every side
///   after the fact. The whole image misses this by 37 px at each end.
#[test]
fn art_that_fades_into_the_gutter_is_cropped_to_the_art_not_flagged() {
    let t = Tuning::default();
    let img = fade_to_gutter();
    let core = fade_core_rect();

    let CropDecision::Crop(rect) = decide(&img, &t) else {
        panic!(
            "AC-3: a page whose art fades into the gutter must be cropped, not flagged. \
             decide returned {:?} and detect returned {:?}",
            decide(&img, &t),
            detect(&img, &t)
        );
    };

    assert!(
        rect.y <= core.y && rect.y + rect.h >= core.y + core.h,
        "AC-3: the crop must contain the art the generator drew. The full-amplitude \
         core is rows {}..{} and the crop covers rows {}..{}",
        core.y,
        core.y + core.h,
        rect.y,
        rect.y + rect.h
    );
    assert!(
        rect.x == 0 && rect.w == img.width,
        "AC-3: the fixture has no left or right gutter, so the crop spans the full \
         width. Expected x 0 w {}, got x {} w {}",
        img.width,
        rect.x,
        rect.w
    );
    assert!(
        rect.y + t.margin_px >= FADE_GUTTER,
        "AC-3: the crop must exclude the flat top gutter, which is rows 0..{FADE_GUTTER}. \
         It starts at row {} and margin_px is {}",
        rect.y,
        t.margin_px
    );
    assert!(
        rect.y + rect.h <= FADE_H - FADE_GUTTER + t.margin_px,
        "AC-3: the crop must exclude the flat bottom gutter, which is rows {}..{FADE_H}. \
         It ends at row {} and margin_px is {}",
        FADE_H - FADE_GUTTER,
        rect.y + rect.h,
        t.margin_px
    );
}
