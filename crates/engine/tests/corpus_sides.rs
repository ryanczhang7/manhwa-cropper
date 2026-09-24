//! MC-049, AC-1, AC-2 and AC-4: the crop's side edges carry no page background,
//! still never clip, and the rows move only by the margin.
//!
//! The user, 2026-09-23: *"The cropping from the side isnt tight enough and I
//! still see the page on the right and left side."* MC-027's column locator
//! already stops on the boundary between flat page background and textured
//! art; `margin::expand` then adds `Tuning::margin_px` columns of that page
//! back on every side. MC-049 rules the margin to 0 on all four sides and
//! corrects the seven marks that contained flat page columns (`corpus.md`,
//! "Corrected marks").
//!
//! # Why this is a new file and not more of `tests/corpus.rs`
//!
//! `tests/corpus.rs` (MC-026, MC-027) and `tests/corpus_accuracy.rs` (MC-019)
//! are other stories' criteria, and MC-048 is editing `corpus.rs` in parallel.
//! This target shares their loader (`common/corpus.rs`) by the same `#[path]`
//! include and nothing else. It sorts after `corpus.rs`, so the `integration`
//! gate's floor - read off `corpus.rs`, the first engine target with ignored
//! tests - is not moved by it.
//!
//! The criteria this file does **not** restate, because an existing test
//! already says them and must keep saying them: AC-3 is
//! `corpus.rs::the_six_pages_flagged_today_are_still_flagged_for_no_border_found`;
//! AC-5 is `corpus.rs::the_left_and_right_edges_of_every_marked_page_land_inside_the_window`
//! and `corpus_accuracy.rs::at_least_nine_corpus_screenshots_in_ten_are_right_on_the_column_axis`;
//! AC-2's "the existing inset-10 controls stay" is `corpus.rs`'s two
//! `no_crop_clips_*` tests and `corpus_accuracy.rs`'s zero-clip test, all of
//! which now read the corrected marks.
//!
//! # Running it
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_sides -- --ignored --nocapture
//! ```
//!
//! # What is settled, what is invented, what is mechanical
//!
//! * **Settled, read out**: `uniform_tolerance` (10), from the `Tuning` under
//!   test; the containment predicate (`corpus.rs`'s); the marking rule and the
//!   user's rulings on the corrected marks, which live in the manifest.
//! * **Invented here, with two controls - the page-background predicate.** A
//!   column is page background when at least [`PAGE_BACKGROUND_SHARE`] of its
//!   pixels over the mark's rows lie within `uniform_tolerance` of the
//!   column's median luma (the upper median, `sorted[n / 2]`). The story's
//!   definition verbatim. The control on the metric - the mark's own first and
//!   last columns are **not** background - and the control on the number - a
//!   crop one column wider **is** caught - are both in AC-1's test.
//! * **Mechanical**: the margin before this story ([`MARGIN_BEFORE_MC049`]),
//!   which AC-4 measures the row move against.
//!
//! # Measured in RED (story `## Handoff`)
//!
//! At `cb2deef` + the corrected marks, margin 3: AC-1 fails on 42 of 42 sides;
//! AC-2's narrowed-crop control clips 0 entries. Against a candidate at
//! margin 0 (a scratch copy outside the repository): AC-1 0 of 42; the
//! widened control fires on 19 of 21 left sides and 20 of 21 right; the metric
//! control holds on 21 of 21; AC-2 0 clips and the narrowed control clips 9.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use std::path::Path;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Luma, Rect, Tuning, detect};
use cropper_engine::{Outcome, process_file};

// --- Constants ----------------------------------------------------------------

/// The page-background predicate's share: the story's definition.
const PAGE_BACKGROUND_SHARE: f64 = 0.95;

/// AC-1's control on the metric: on at least this many of the 21 entries the
/// mark's own first **and** last columns must read as *not* page background.
/// The story's number. Measured 21 of 21 on the corrected marks, and 14 of 21
/// on the uncorrected ones - the seven corrected entries each had a flat
/// outermost column, which is exactly what this control exists to notice.
const METRIC_CONTROL_REQUIRED: usize = 19;

/// AC-1's control on the number: widening the crop by one column on one side
/// must make that side fail, on at least this many of the 21 entries **per
/// side**.
///
/// The story predicted 21 of 21, "because the next column out is page
/// background on every one". RED measured that premise false on three sides
/// under the story's own 0.95 predicate: the column just outside the locator
/// is a near-flat transition column on `2025-10-20 15_37_25.png` left (share
/// 0.94) and on `Screenshot (3538).png` both sides (0.86 left, 0.89 right).
/// 19 is the measured left count and one below the measured right count
/// (20). **Escalated in the story's `## Handoff`**: this number, or the
/// predicate's share, needs the product owner's ruling before GREEN.
const WIDENED_CONTROL_REQUIRED_PER_SIDE: usize = 19;

/// AC-2's control: the crop narrowed by one column on each side must clip on
/// at least this many entries. The story's number; RED measured 9 against the
/// margin-0 candidate (the seven corrected marks, `Screenshot (3538).png`'s
/// left and `2026-01-05 13_45_59.png`'s right, whose MC-027 correction already
/// put its mark on the locator's edge).
const NARROWED_CLIPS_REQUIRED: usize = 8;

/// `Tuning::margin_px` before MC-049, which AC-4 measures the row move from.
/// Mechanical: the value `architecture.md` decision 5 carried until this story.
const MARGIN_BEFORE_MC049: u32 = 3;

// --- Harness ------------------------------------------------------------------

/// The marked `tuning` entries, in manifest order. Tuning only - see
/// `corpus.rs::tuning_only` for why no accuracy suite reads held-out entries.
fn marked() -> Vec<(CorpusEntry, Rect)> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .filter_map(|entry| match entry.expect {
            Expect::Rect(rect) => Some((entry, rect)),
            Expect::Flag => None,
        })
        .collect()
}

/// The luma plane `process_file` sees: the engine's decoder and conversion.
fn luma(path: &Path) -> Luma {
    let bytes =
        std::fs::read(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// The rect `process_file` cropped `entry` to at `t`, or the outcome as text.
fn cropped(entry: &CorpusEntry, t: &Tuning, tmp: &tempfile::TempDir) -> Result<Rect, String> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Ok(rect),
        Outcome::Flagged { reason, .. } => Err(format!("Flagged {reason:?}")),
        Outcome::Failed { error } => Err(format!("Failed {error}")),
    }
}

/// The share of column `x`'s pixels over `mark`'s rows lying within
/// `tolerance` of the column's median luma (upper median).
fn background_share(img: &Luma, x: u32, mark: Rect, tolerance: u8) -> f64 {
    let mut values: Vec<u8> = (mark.y..mark.y + mark.h)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect();
    values.sort_unstable();
    let median = values[values.len() / 2];
    let near = values
        .iter()
        .filter(|&&v| v.abs_diff(median) <= tolerance)
        .count();
    near as f64 / values.len() as f64
}

/// The story's page-background predicate.
fn is_page_background(img: &Luma, x: u32, mark: Rect, t: &Tuning) -> bool {
    background_share(img, x, mark, t.uniform_tolerance) >= PAGE_BACKGROUND_SHARE
}

/// The last column of `r`, inclusive.
fn right(r: Rect) -> u32 {
    r.x + r.w - 1
}

/// The crop's columns that lie outside the mark on the left and on the right.
fn outside_columns(crop: Rect, mark: Rect) -> (Vec<u32>, Vec<u32>) {
    let left = (crop.x..mark.x.min(right(crop) + 1)).collect();
    let right_side = ((right(mark) + 1).max(crop.x)..=right(crop)).collect();
    (left, right_side)
}

/// Whether `outer` contains `inner` on all four sides - `corpus.rs`'s AC-5
/// predicate.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// A table printed under `--nocapture` and returned for the assertion message.
fn table(title: &str, rows: &[String]) -> String {
    let out = format!("{title}\n{}\n", rows.join("\n"));
    println!("{out}");
    out
}

// --- AC-1 ---------------------------------------------------------------------

/// AC-1, with both of its controls.
///
/// For each of the 21 marked `tuning` entries, `process_file` at
/// `Tuning::default()`: no column of the crop lying outside the mark may be
/// page background, on either side. Before MC-049 every side carries
/// `margin_px` columns of flat page - 42 of 42 sides fail.
///
/// * **Control on the metric**: the mark's own first and last columns are
///   art, and the predicate must say so on at least
///   [`METRIC_CONTROL_REQUIRED`] entries - a predicate that calls everything
///   background fails here.
/// * **Control on the number**: the crop widened by one column on one side
///   must fail that side on at least [`WIDENED_CONTROL_REQUIRED_PER_SIDE`]
///   entries per side - a predicate that calls nothing background, or an
///   assertion blind to a single column, fails here.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_column_the_crop_keeps_outside_the_mark_is_page_background() {
    let t = Tuning::default();
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut failing_sides = Vec::new();
    let mut metric_ok = 0usize;
    let mut metric_missed = Vec::new();
    let (mut widened_left, mut widened_right) = (0usize, 0usize);
    let mut widened_missed = Vec::new();
    let mut entries = 0usize;

    for (entry, mark) in marked() {
        entries += 1;
        let img = luma(&entry.path);
        let bg = |x: u32| is_page_background(&img, x, mark, &t);

        // The control on the metric does not need a crop.
        let (first_bg, last_bg) = (bg(mark.x), bg(right(mark)));
        if !first_bg && !last_bg {
            metric_ok += 1;
        } else {
            metric_missed.push(format!(
                "{}: mark column {} background {first_bg}, column {} background {last_bg}",
                entry.name(),
                mark.x,
                right(mark)
            ));
        }

        let crop = match cropped(&entry, &t, &tmp) {
            Ok(rect) => rect,
            Err(outcome) => {
                rows.push(format!("{:<30} {outcome}", entry.name()));
                failing_sides.push(format!("{}: not cropped ({outcome})", entry.name()));
                failing_sides.push(format!("{}: not cropped ({outcome})", entry.name()));
                continue;
            }
        };

        let (left_cols, right_cols) = outside_columns(crop, mark);
        let left_bg: Vec<u32> = left_cols.iter().copied().filter(|&x| bg(x)).collect();
        let right_bg: Vec<u32> = right_cols.iter().copied().filter(|&x| bg(x)).collect();
        if !left_bg.is_empty() {
            failing_sides.push(format!(
                "{} left: page-background columns {left_bg:?} outside the mark's {}",
                entry.name(),
                mark.x
            ));
        }
        if !right_bg.is_empty() {
            failing_sides.push(format!(
                "{} right: page-background columns {right_bg:?} outside the mark's {}",
                entry.name(),
                right(mark)
            ));
        }

        // The control on the number: one more column on each side, in turn.
        let next_left = crop.x.checked_sub(1).filter(|&x| x < mark.x);
        let next_right = Some(right(crop) + 1).filter(|&x| x < img.width && x > right(mark));
        let fired_left = next_left.is_some_and(bg);
        let fired_right = next_right.is_some_and(bg);
        widened_left += usize::from(fired_left);
        widened_right += usize::from(fired_right);
        for (side, col, fired) in [
            ("left", next_left, fired_left),
            ("right", next_right, fired_right),
        ] {
            if !fired {
                let share = col.map(|x| background_share(&img, x, mark, t.uniform_tolerance));
                widened_missed.push(format!(
                    "{} {side}: next column {col:?} share {share:.2?}",
                    entry.name()
                ));
            }
        }

        rows.push(format!(
            "{:<30} crop {:>4}..{:<4} mark {:>4}..{:<4} outside L {:>2} (bg {:>2})  R {:>2} (bg {:>2})  widened L {} R {}",
            entry.name(),
            crop.x,
            right(crop),
            mark.x,
            right(mark),
            left_cols.len(),
            left_bg.len(),
            right_cols.len(),
            right_bg.len(),
            if fired_left { "fires" } else { "-" },
            if fired_right { "fires" } else { "-" },
        ));
    }

    let printed = table(
        &format!(
            "AC-1: page-background columns outside the mark, margin_px {}, share >= {PAGE_BACKGROUND_SHARE} within {} of the column median",
            t.margin_px, t.uniform_tolerance
        ),
        &rows,
    );

    assert!(
        entries >= METRIC_CONTROL_REQUIRED,
        "AC-1 read only {entries} marked tuning entries"
    );
    assert!(
        metric_ok >= METRIC_CONTROL_REQUIRED,
        "AC-1's control on the metric: the mark's own first and last columns are \
         art, so the predicate must call them not page background on at least \
         {METRIC_CONTROL_REQUIRED} of {entries} entries; it did on {metric_ok}. \
         Either the predicate calls art background, or a mark still contains \
         flat page columns:\n{}\n\n{printed}",
        metric_missed.join("\n")
    );
    assert!(
        widened_left >= WIDENED_CONTROL_REQUIRED_PER_SIDE
            && widened_right >= WIDENED_CONTROL_REQUIRED_PER_SIDE,
        "AC-1's control on the number: a crop widened by one column must be caught \
         carrying page background on that side, on at least \
         {WIDENED_CONTROL_REQUIRED_PER_SIDE} of {entries} entries per side; it was \
         on {widened_left} left and {widened_right} right. Misses:\n{}\n\n{printed}",
        widened_missed.join("\n")
    );
    assert!(
        failing_sides.is_empty(),
        "AC-1: the crop must carry no page background outside the mark, on either \
         side. {} of {} sides do (margin_px is {}); the first is {}.\n\nAll:\n{}\n\n{printed}",
        failing_sides.len(),
        2 * entries,
        t.margin_px,
        failing_sides[0],
        failing_sides.join("\n")
    );
}

// --- AC-2 ---------------------------------------------------------------------

/// AC-2, with its control.
///
/// Against the corrected marks, every `Cropped` rect contains its mark on all
/// four sides - 0 clips. With the margin at 0 the crop's side edges rest on
/// the locator alone, so the control is what shows the crop is *tight*: the
/// same crop narrowed by one column on each side must clip on at least
/// [`NARROWED_CLIPS_REQUIRED`] entries. Before MC-049 it clips on none - three
/// columns of margin hide every one.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn every_crop_contains_its_corrected_mark_and_one_column_narrower_clips() {
    let t = Tuning::default();
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut clips = Vec::new();
    let mut narrowed_clips = Vec::new();
    let mut cropped_count = 0usize;

    for (entry, mark) in marked() {
        let crop = match cropped(&entry, &t, &tmp) {
            Ok(rect) => rect,
            Err(outcome) => {
                rows.push(format!("{:<30} {outcome}", entry.name()));
                continue;
            }
        };
        cropped_count += 1;
        let holds = contains(crop, mark);
        if !holds {
            clips.push(format!(
                "{}: crop {},{} {}x{} does not contain the mark {},{} {}x{}",
                entry.name(),
                crop.x,
                crop.y,
                crop.w,
                crop.h,
                mark.x,
                mark.y,
                mark.w,
                mark.h
            ));
        }
        let narrowed = Rect {
            x: crop.x + 1,
            w: crop.w - 2,
            ..crop
        };
        let narrowed_holds = contains(narrowed, mark);
        if !narrowed_holds {
            narrowed_clips.push(entry.name());
        }
        rows.push(format!(
            "{:<30} crop {:>4}..{:<4} mark {:>4}..{:<4} slack L {:>+3} R {:>+3}  {}  narrowed {}",
            entry.name(),
            crop.x,
            right(crop),
            mark.x,
            right(mark),
            i64::from(mark.x) - i64::from(crop.x),
            i64::from(right(crop)) - i64::from(right(mark)),
            if holds { "holds" } else { "CLIPS" },
            if narrowed_holds { "holds" } else { "clips" },
        ));
    }

    let printed = table(
        &format!(
            "AC-2: containment of the corrected marks, margin_px {} ({cropped_count} cropped)",
            t.margin_px
        ),
        &rows,
    );

    assert!(
        cropped_count > 0,
        "AC-2 compared nothing: no marked entry was cropped\n\n{printed}"
    );
    assert!(
        clips.is_empty(),
        "AC-2: a crop that cuts into the page a person marked is the worst defect \
         this product has, and with margin_px {} nothing but the locator stands \
         between the art and the cut. {} of {cropped_count} clip; the first is {}.\n\n\
         All:\n{}\n\n{printed}",
        t.margin_px,
        clips.len(),
        clips[0],
        clips.join("\n")
    );
    assert!(
        narrowed_clips.len() >= NARROWED_CLIPS_REQUIRED,
        "AC-2's control: the crop narrowed by one column on each side must clip on \
         at least {NARROWED_CLIPS_REQUIRED} entries, or the crop still carries \
         slack beside the art and 'zero clips' is measured through a margin. It \
         clipped on {} ({narrowed_clips:?}); margin_px is {}.\n\n{printed}",
        narrowed_clips.len(),
        t.margin_px
    );
}

// --- AC-4 ---------------------------------------------------------------------

/// `rect`'s rows grown by `by` on each side and clamped to `height`, as
/// `(first row, one past the last)`. Test-side arithmetic, deliberately not
/// `margin::expand`, so a change to that function cannot move both sides of
/// the comparison at once.
fn grown_rows(rect: Rect, by: u32, height: u32) -> (u32, u32) {
    (
        rect.y.saturating_sub(by),
        (rect.y + rect.h + by).min(height),
    )
}

/// AC-4: each crop's top and bottom edge moves by exactly the change in
/// `margin_px` and no more, clamped at the image edge, and still contains the
/// mark.
///
/// Three detections per entry: the pre-margin rect (`margin_px` 0), the crop
/// as it was before MC-049 ([`MARGIN_BEFORE_MC049`]), and the crop at
/// `Tuning::default()`. Both crops' rows must be the pre-margin rows grown by
/// their own margin, which is the statement that the margin is the *only*
/// thing on the row axis that differs between them. A change that zeroed the
/// columns' margin and left the rows' at 3 fails here, as does one that moved
/// the row locator.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_top_and_bottom_edges_move_by_exactly_the_margin_change() {
    let t = Tuning::default();
    let bare = Tuning {
        margin_px: 0,
        ..Tuning::default()
    };
    let before = Tuning {
        margin_px: MARGIN_BEFORE_MC049,
        ..Tuning::default()
    };
    let mut rows = Vec::new();
    let mut wrong = Vec::new();
    let mut clamped = 0usize;

    for (entry, mark) in marked() {
        let img = luma(&entry.path);
        let at = |tuning: &Tuning| {
            detect(&img, tuning)
                .unwrap_or_else(|| panic!("{} is not a uniform image", entry.name()))
                .rect
        };
        let (located, old, new) = (at(&bare), at(&before), at(&t));
        let rows_of = |r: Rect| (r.y, r.y + r.h);
        let want_old = grown_rows(located, MARGIN_BEFORE_MC049, img.height);
        let want_new = grown_rows(located, t.margin_px, img.height);
        if want_old.1 - want_old.0 < located.h + 2 * MARGIN_BEFORE_MC049 {
            clamped += 1;
        }
        let contains_rows = new.y <= mark.y && new.y + new.h >= mark.y + mark.h;
        let ok = rows_of(old) == want_old && rows_of(new) == want_new && contains_rows;
        if !ok {
            wrong.push(format!(
                "{}: located rows {:?}; at margin {MARGIN_BEFORE_MC049} {:?} (want {want_old:?}); \
                 at margin {} {:?} (want {want_new:?}); mark rows {}..{} contained {contains_rows}",
                entry.name(),
                rows_of(located),
                rows_of(old),
                t.margin_px,
                rows_of(new),
                mark.y,
                mark.y + mark.h
            ));
        }
        rows.push(format!(
            "{:<30} located {:>4}..{:<4} before {:>4}..{:<4} now {:>4}..{:<4} {}",
            entry.name(),
            located.y,
            located.y + located.h,
            old.y,
            old.y + old.h,
            new.y,
            new.y + new.h,
            if ok { "ok" } else { "MOVED" }
        ));
    }

    let printed = table(
        &format!(
            "AC-4: rows before ({MARGIN_BEFORE_MC049}) and now ({}), {clamped} entries clamped at an image edge",
            t.margin_px
        ),
        &rows,
    );
    assert!(
        wrong.is_empty(),
        "AC-4: the rows may move by the change in margin_px and nothing else. {} \
         entries moved otherwise:\n{}\n\n{printed}",
        wrong.len(),
        wrong.join("\n")
    );
}
