//! MC-069 AC-5: a close call on a strip the crop leaves out does not flag.
//!
//! The rule the story ships (the user's answer of 2026-10-01, "Ignore if crop
//! excludes it"): an edge strip that is *nearly* chrome - a flat fraction in
//! `[chrome_flat_fraction - ambiguity_band, chrome_flat_fraction)` - makes the
//! **decision** `Flag(Ambiguous)` only if the final crop rect, margin and all,
//! includes at least one of that strip's pixels. The strip-level judgement
//! (`content_box` and `ContentBox::ambiguous`) does not change; that is pinned
//! here too, so a fix that reached green by narrowing the strip judgement
//! instead of the decision fails.
//!
//! # The scene, and why it is this one
//!
//! The 13 real screenshots in MC-069 have one shape: a reader page between
//! flat page margins, and at the image's right edge the browser's scrollbar -
//! a 15 px strip, full height, flat fraction 0.8478 to 0.8499. `content_box`
//! keeps the scrollbar (it is not chrome) and calls it a close call; the page
//! column stage (MC-027) then narrows the rect to the page, so the crop ends
//! some 1,000 px left of the scrollbar, and `decide` flags it anyway.
//!
//! The generated twin is MC-027's own page fixture (`common::page_in_margins`,
//! 300x240: 40 px of flat speckled margin, the page, 40 px of margin) with a
//! [`SCROLLBAR_W`]-px scrollbar appended on the right: a flat
//! [`SCROLLBAR_BG`] with exactly `text` pixels moved [`SCROLLBAR_DEVIATION`]
//! off it, spread so that every one of its columns is within one pixel of
//! every other. Its flat fraction is therefore `1 - text / 3600` exactly, and
//! chosen, not measured:
//!
//! | `text` | flat fraction | where |
//! |---|---|---|
//! | 545 | 3055/3600 = 0.848611 | inside `[0.8475, 0.85)`: nearly chrome |
//! | 550 | 3050/3600 = 0.847222 | below the band's foot: content, no close call |
//!
//! The margin column next to the scrollbar is `FADE_TONE` (128) and the
//! scrollbar is 200, so their boundary is the rect's last strong column line
//! and `content_box`'s Right strip is exactly the scrollbar, columns
//! `300..315`. Its Left strip runs to the panel seam at column 149, half the
//! image, past `chrome_max_extent`, so it is content and no close call. The
//! rows are the page's, every one a strong line, so the Top and Bottom strips
//! are one row of art each. The scrollbar is the only close call in the scene,
//! and `scene_preconditions_hold` checks each of these rather than assuming it.
//!
//! # The crop, and the control
//!
//! At the default margin the crop is the page column, `45..255`, so the
//! scrollbar lies wholly outside it and the answer must be `Crop` - **red on
//! `main`**, which answers `Flag(Ambiguous)`.
//!
//! The control is the same image, the same strip, with the crop made to reach
//! it. Only the margin moves (`Tuning::margin_px`), because the margin is the
//! last thing `detect` does to the rect and the columns take all of it: at
//! [`MARGIN_TO_THE_STRIP`] the crop ends exactly where the scrollbar starts
//! (column 300, exclusive) and includes none of it - `Crop`; at one more it
//! includes column 300 and nothing else of the strip - `Flag(Ambiguous)`. The
//! pair is the rule's boundary, "at least one pixel", and it is also what
//! tells the final crop apart from the rect before the margin: tested against
//! the pre-margin rect, the second of the pair would answer `Crop`.
//!
//! A second control holds the crop where it is and moves the strip below the
//! band: no close call anywhere, `Crop` on `main` and after. With the first
//! scene it is the twin `decide.rs` uses for every ambiguity test - same
//! geometry, one flat fraction apart.
//!
//! # Export shape
//!
//! Only API that exists on `main`: `cropper_core::{decide, detect, Tuning,
//! Rect, Luma, CropDecision, FlagReason}`, `content::content_box`,
//! `trim::trim_uniform`. Nothing here reads `Detection::ambiguous`: whether
//! that field narrows with the decision or keeps the strip-level reading is
//! the implementer's choice, and only `decide` and `ContentBox::ambiguous` are
//! pinned.

mod common;

use common::{PAGE_H, PAGE_W, page_pixel};
use cropper_core::content::content_box;
use cropper_core::trim::trim_uniform;
use cropper_core::{CropDecision, FlagReason, Luma, Rect, Tuning, decide, detect};

/// The scrollbar's width: the real one's, 15 px (MC-069 `## Amendments`,
/// `x 2545..2560`).
const SCROLLBAR_W: u32 = 15;

/// The scrollbar's background: far enough from the page margin's 128 that the
/// boundary is a strong column line (a step of 72 against `edge_threshold` 24).
const SCROLLBAR_BG: u8 = 200;

/// How far a "text" pixel sits from [`SCROLLBAR_BG`]: twice
/// `uniform_tolerance`, so it is never flat, and small enough that no column
/// of the scrollbar is a strong line against its neighbour - the same choice
/// and the same number as `common::CHROME_DEVIATION`.
const SCROLLBAR_DEVIATION: u8 = 20;

/// Text pixels in the nearly-chrome scrollbar: 545 of 3600, flat fraction
/// 0.848611, inside `[0.8475, 0.85)`.
const NEARLY_CHROME_TEXT: u32 = 545;

/// Text pixels in the control scrollbar: 550 of 3600, flat fraction 0.847222,
/// below the band's foot of 0.8475.
const BELOW_THE_BAND_TEXT: u32 = 550;

/// The first column of the scrollbar, which is also the image width of the
/// page fixture it is appended to.
const SCROLLBAR_X: u32 = PAGE_W;

/// The scene's width.
const SCENE_W: u32 = PAGE_W + SCROLLBAR_W;

/// The margin at which the crop's right edge lands exactly on
/// [`SCROLLBAR_X`] (exclusive), so the crop touches the scrollbar and includes
/// none of it. Measured on `main` from `detect`'s rect at margin 0, whose
/// right edge is column 255 exclusive (`common::page_last_art_column` is 254):
/// `300 - 255 = 45`. `the_crop_reaches_exactly_to_the_strip_at_the_boundary_margin`
/// checks the arithmetic rather than trusting it.
const MARGIN_TO_THE_STRIP: u32 = 45;

/// Whether scrollbar pixel `(dx, y)` is a "text" pixel, for `text` text pixels
/// over the whole [`SCROLLBAR_W`] x [`PAGE_H`] strip.
///
/// Column `dx` holds `text / SCROLLBAR_W` text pixels, plus one for the first
/// `text % SCROLLBAR_W` columns, placed by a Bresenham walk down the column
/// and rotated by 16 rows per column so neighbouring columns do not line up.
/// The count is exact, so the strip's flat fraction is exactly
/// `1 - text / (SCROLLBAR_W * PAGE_H)`.
fn is_text(dx: u32, y: u32, text: u32) -> bool {
    let k = text / SCROLLBAR_W + u32::from(dx < text % SCROLLBAR_W);
    let r = (y + 16 * dx) % PAGE_H;
    (r + 1) * k / PAGE_H != r * k / PAGE_H
}

/// One scrollbar pixel: the background, or a text pixel alternating above and
/// below it down the column, so the strip's median stays on the background.
fn scrollbar_pixel(dx: u32, y: u32, text: u32) -> u8 {
    if !is_text(dx, y, text) {
        return SCROLLBAR_BG;
    }
    let r = (y + 16 * dx) % PAGE_H;
    let k = text / SCROLLBAR_W + u32::from(dx < text % SCROLLBAR_W);
    let index = r * k / PAGE_H;
    if index.is_multiple_of(2) {
        SCROLLBAR_BG + SCROLLBAR_DEVIATION
    } else {
        SCROLLBAR_BG - SCROLLBAR_DEVIATION
    }
}

/// MC-027's page between flat margins, with a scrollbar of `text` text pixels
/// appended at the right edge.
fn page_with_scrollbar(text: u32) -> Luma {
    let mut data = Vec::with_capacity((SCENE_W * PAGE_H) as usize);
    for y in 0..PAGE_H {
        for x in 0..SCENE_W {
            data.push(if x < SCROLLBAR_X {
                page_pixel(x, y)
            } else {
                scrollbar_pixel(x - SCROLLBAR_X, y, text)
            });
        }
    }
    Luma {
        width: SCENE_W,
        height: PAGE_H,
        data,
    }
}

/// The scrollbar's rect in image coordinates.
fn scrollbar() -> Rect {
    Rect {
        x: SCROLLBAR_X,
        y: 0,
        w: SCROLLBAR_W,
        h: PAGE_H,
    }
}

/// The share of `rect`'s pixels within `tolerance` of its upper median,
/// inclusive - `content_box`'s flat fraction, recomputed here from the pixels
/// so the fixture's number is a measurement and not the generator's say-so.
/// In `f32`, as `content_box` takes it, so the band comparison below is the
/// same comparison.
fn flat_fraction(img: &Luma, rect: Rect, tolerance: u8) -> f32 {
    let mut histogram = [0u64; 256];
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            histogram[img.data[(y * img.width + x) as usize] as usize] += 1;
        }
    }
    let total = u64::from(rect.w) * u64::from(rect.h);
    let mut seen = 0;
    let centre = histogram
        .iter()
        .position(|&count| {
            seen += count;
            seen > total / 2
        })
        .expect("a non-empty strip has a median") as u8;
    let lo = usize::from(centre.saturating_sub(tolerance));
    let hi = usize::from(centre.saturating_add(tolerance));
    let flat: u64 = histogram[lo..=hi].iter().sum();
    flat as f32 / total as f32
}

/// Whether `a` and `b` share at least one pixel.
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// `content_box`'s strip-level judgement over the rect `detect` hands it -
/// `trim_uniform` first, exactly as `detect` composes them. MC-069 does not
/// change this.
fn strip_level_ambiguous(img: &Luma, t: &Tuning) -> bool {
    let first = trim_uniform(img, t).expect("the scene is textured");
    content_box(img, first, t).ambiguous
}

/// `detect`'s rect, which is what `decide` crops to when it crops.
fn crop_rect(img: &Luma, t: &Tuning) -> Rect {
    detect(img, t).expect("the scene is textured").rect
}

fn at_margin(margin_px: u32) -> Tuning {
    Tuning {
        margin_px,
        ..Tuning::default()
    }
}

// --- The fixture is what it claims -------------------------------------------

/// Every premise the four tests below lean on, measured: the two strips' flat
/// fractions against the band, the strip-level judgement on each, and where
/// the crop lies relative to the scrollbar at the three margins used.
///
/// Green on `main` and after: nothing here is MC-069's behaviour. It is what
/// keeps the scene from quietly becoming a different scene - a scrollbar that
/// drifted into the chrome range, or a crop that grew into it, would make the
/// AC-5 tests below pass or fail for a reason that is not the rule.
#[test]
fn scene_preconditions_hold() {
    let t = Tuning::default();
    let foot = t.chrome_flat_fraction - t.ambiguity_band;
    let near = page_with_scrollbar(NEARLY_CHROME_TEXT);
    let below = page_with_scrollbar(BELOW_THE_BAND_TEXT);
    let mut wrong = Vec::new();

    let near_flat = flat_fraction(&near, scrollbar(), t.uniform_tolerance);
    let below_flat = flat_fraction(&below, scrollbar(), t.uniform_tolerance);
    if !(near_flat >= foot && near_flat < t.chrome_flat_fraction) {
        wrong.push(format!(
            "the nearly-chrome scrollbar measures {near_flat}, outside [{foot}, {})",
            t.chrome_flat_fraction
        ));
    }
    if near_flat != 3055.0 / 3600.0 {
        wrong.push(format!(
            "the nearly-chrome scrollbar measures {near_flat}, not 3055/3600"
        ));
    }
    if below_flat >= foot {
        wrong.push(format!(
            "the control scrollbar measures {below_flat}, not below the foot {foot}"
        ));
    }
    if below_flat != 3050.0 / 3600.0 {
        wrong.push(format!(
            "the control scrollbar measures {below_flat}, not 3050/3600"
        ));
    }
    if !strip_level_ambiguous(&near, &t) {
        wrong.push("content_box does not call the nearly-chrome scrollbar a close call".into());
    }
    if strip_level_ambiguous(&below, &t) {
        wrong.push("content_box calls something in the control scene a close call".into());
    }
    if strip_level_ambiguous(&page_with_scrollbar(0), &t) {
        // Flat fraction 1.0: chrome, if anything; never a close call. Keeps
        // the close call attributable to the scrollbar's flat fraction alone.
        wrong.push("a perfectly flat scrollbar is called a close call".into());
    }
    let crop = crop_rect(&near, &t);
    if overlaps(crop, scrollbar()) {
        wrong.push(format!(
            "at the default margin the crop {crop:?} includes part of the scrollbar"
        ));
    }
    if crop_rect(&below, &t) != crop {
        wrong.push(format!(
            "the two scenes crop differently: {crop:?} and {:?}",
            crop_rect(&below, &t)
        ));
    }

    assert!(
        wrong.is_empty(),
        "MC-069 AC-5's scene is not the scene its tests describe:\n{}",
        wrong.join("\n")
    );
}

/// The arithmetic behind [`MARGIN_TO_THE_STRIP`], checked: at it, the crop's
/// right edge is the scrollbar's first column, exclusive; at one more, the
/// crop includes exactly one column of the scrollbar. Green on `main` and
/// after - `detect`'s rect is not MC-069's to move.
#[test]
fn the_crop_reaches_exactly_to_the_strip_at_the_boundary_margin() {
    let img = page_with_scrollbar(NEARLY_CHROME_TEXT);
    let touching = crop_rect(&img, &at_margin(MARGIN_TO_THE_STRIP));
    let into = crop_rect(&img, &at_margin(MARGIN_TO_THE_STRIP + 1));
    assert_eq!(
        (touching.x + touching.w, into.x + into.w),
        (SCROLLBAR_X, SCROLLBAR_X + 1),
        "at margin_px {MARGIN_TO_THE_STRIP} the crop must end exactly where the \
         scrollbar starts, and at one more it must take exactly its first column. \
         Crops: {touching:?} and {into:?}"
    );
}

// --- AC-5 ---------------------------------------------------------------------

/// AC-5, the reproduction. The scrollbar is nearly chrome and the crop lies
/// wholly left of it, so the close call changes nothing in the answer: `Crop`,
/// to exactly the rect `detect` produced.
///
/// **Red on `main`**, which answers `Flag(Ambiguous)` - the 13 screenshots'
/// defect, generated.
#[test]
fn a_nearly_chrome_strip_the_crop_leaves_out_does_not_flag_the_page() {
    let t = Tuning::default();
    let img = page_with_scrollbar(NEARLY_CHROME_TEXT);
    let rect = crop_rect(&img, &t);
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(rect),
        "MC-069 AC-5: the scrollbar at columns {SCROLLBAR_X}..{SCENE_W} is nearly \
         chrome (flat fraction 3055/3600 = 0.848611) and the crop {rect:?} \
         includes none of it, so the close call must not flag the page"
    );
}

/// AC-5 at the boundary from the outside: the crop's right edge sits on the
/// scrollbar's first column (exclusive) and includes none of its pixels, so
/// it is still `Crop`. **Red on `main`.** An implementation that tested
/// "touches" rather than "includes", or used `<=` where `<` belongs, flags
/// this one.
#[test]
fn a_crop_ending_exactly_where_the_nearly_chrome_strip_starts_is_still_cropped() {
    let t = at_margin(MARGIN_TO_THE_STRIP);
    let img = page_with_scrollbar(NEARLY_CHROME_TEXT);
    let rect = crop_rect(&img, &t);
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(rect),
        "MC-069 AC-5: at margin_px {MARGIN_TO_THE_STRIP} the crop {rect:?} ends at \
         column {SCROLLBAR_X}, exclusive, where the nearly-chrome scrollbar starts; \
         it includes none of the strip's pixels, so it must be cropped"
    );
}

/// AC-5's control: the same image and the same nearly-chrome scrollbar, with
/// the margin one pixel wider, so the final crop includes the scrollbar's
/// first column. The close call is inside the crop now, and the answer is
/// `Flag(Ambiguous)` - on `main` and after.
///
/// It is also the test that tells the **final** crop from the rect before the
/// margin: before the margin the rect is the page column, `45..255`, nowhere
/// near the scrollbar, so an implementation that intersected that rect would
/// crop here.
#[test]
fn the_same_nearly_chrome_strip_one_pixel_inside_the_crop_still_flags_the_page() {
    let t = at_margin(MARGIN_TO_THE_STRIP + 1);
    let img = page_with_scrollbar(NEARLY_CHROME_TEXT);
    assert_eq!(
        decide(&img, &t),
        CropDecision::Flag(FlagReason::Ambiguous),
        "MC-069 AC-5's control: at margin_px {} the crop {:?} includes column \
         {SCROLLBAR_X} of the nearly-chrome scrollbar, so the close call is inside \
         what would be kept and the page is for review",
        MARGIN_TO_THE_STRIP + 1,
        crop_rect(&img, &t)
    );
}

/// AC-5's second control: the crop where the reproduction has it, the strip
/// one rung below the band. No close call anywhere, so `Crop` - on `main` and
/// after. With the reproduction it is a twin pair one flat fraction apart:
/// the answer is the same on both after MC-069, and differs on `main` only
/// because `main` flags a close call the crop leaves out.
#[test]
fn the_same_scene_with_the_strip_below_the_band_is_cropped() {
    let t = Tuning::default();
    let img = page_with_scrollbar(BELOW_THE_BAND_TEXT);
    let rect = crop_rect(&img, &t);
    assert_eq!(
        decide(&img, &t),
        CropDecision::Crop(rect),
        "MC-069 AC-5's control: the scrollbar's flat fraction is 3050/3600 = \
         0.847222, below the band, so nothing is a close call and the page is cropped"
    );
}

/// The strip-level judgement does not change (MC-069's rule, "the strip-level
/// judgement itself is unchanged"): `content_box` still calls the scrollbar a
/// close call at every margin - it never sees the margin - including the one
/// where `decide` now crops. A fix that reached AC-5 by narrowing
/// `ContentBox::ambiguous` instead of the decision fails here.
#[test]
fn content_box_still_calls_the_strip_the_crop_leaves_out_a_close_call() {
    let img = page_with_scrollbar(NEARLY_CHROME_TEXT);
    let judged: Vec<(u32, bool)> = [0, 3, MARGIN_TO_THE_STRIP, MARGIN_TO_THE_STRIP + 1]
        .into_iter()
        .map(|m| (m, strip_level_ambiguous(&img, &at_margin(m))))
        .collect();
    assert!(
        judged.iter().all(|&(_, ambiguous)| ambiguous),
        "MC-069: ContentBox::ambiguous is the strip-level judgement and does not \
         change; on the nearly-chrome scrollbar it must be true at every margin. \
         (margin_px, ambiguous): {judged:?}"
    );
}
