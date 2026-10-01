//! MC-066 AC-1 and AC-2: a second browser window beside the reader is never
//! cropped. Two fresh corpus screenshots are split screens - the `toongod`
//! reader on the left, a second browser window playing a video on the right -
//! and today the app crops `f18` to the second window only and `f13` across
//! both windows (MC-064 pinned both as known exceptions; MC-066 removes them).
//! The generated reproduction, and the two causes RED found, are
//! `crates/core/tests/page_column_second_window.rs` (AC-4).
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_second_window -- --ignored
//! ```
//!
//! # Settled, measured, mechanical
//!
//! * **Settled, read out**: the marks (MC-062's frozen table, which the user
//!   ruled stand on 2026-09-30); the furniture rows (MC-063's frozen oracle,
//!   `f18` 115 / 1399, `f13` 115 / 1392, the last being `f13`'s horizontal
//!   scrollbar); zero clips.
//! * **Measured, re-measured in MC-066's RED and pinned here**: the seam
//!   between the windows, `f18` column 1828 and `f13` column 1820. The
//!   definition is the Lead PO's (MC-066 `## Notes`): a column whose colour
//!   changes against the column to its left - the sum of the three channels'
//!   absolute differences above [`SEAM_CHANGE`] - on more than
//!   [`SEAM_ROWS`] of the image's 1,440 rows. It reads **RGB**, and only to
//!   locate the measurement, as the Lead PO's did; the crop it bounds is
//!   judged on the crop alone, and the fix stays luma-only.
//! * **Mechanical**: each criterion's containment, row and column bounds.
//!
//! **Held-out discipline.** Both entries are `tuning` (MC-064); the premise
//! test checks it.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Luma, Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// The seam definition's per-row threshold: the sum over R, G and B of the
/// absolute difference against the column to the left must exceed this.
/// The Lead PO's number (MC-066 `## Notes`).
const SEAM_CHANGE: i32 = 30;

/// The seam definition's row count: a column is a seam when it changes on
/// more than this many of the image's rows. The story's number (MC-066
/// `## Model guidance`, "more than 1,300 of 1,440 rows").
const SEAM_ROWS: usize = 1300;

/// One of the story's two entries, as the story settles it.
struct Settled {
    /// MC-063's id for it.
    id: &'static str,
    /// The corpus file.
    file: &'static str,
    /// The mark, MC-062's frozen table: `[x, y, w, h]`.
    mark: [u32; 4],
    /// The furniture rows, MC-063's frozen oracle, `top .. bottom` (end
    /// exclusive).
    rows: (u32, u32),
    /// The seam: the second window's first column, measured. The crop must
    /// end at or left of it (end exclusive).
    seam: u32,
    /// The rows the seam changes on, measured in MC-066's RED (the story's
    /// table: 1,399 and 1,390).
    seam_rows: usize,
}

/// `f18`: AC-1.
const F18: Settled = Settled {
    id: "f18",
    file: "2025-03-06 12_48_06.png",
    mark: [651, 115, 517, 1284],
    rows: (115, 1399),
    seam: 1828,
    seam_rows: 1399,
};

/// `f13`: AC-2.
const F13: Settled = Settled {
    id: "f13",
    file: "2025-03-16 22_47_44.png",
    mark: [635, 115, 533, 1277],
    rows: (115, 1392),
    seam: 1820,
    seam_rows: 1390,
};

impl Settled {
    fn mark(&self) -> Rect {
        let [x, y, w, h] = self.mark;
        Rect { x, y, w, h }
    }
}

/// The corpus entry for `s`.
fn entry(s: &Settled) -> CorpusEntry {
    corpus::load()
        .into_iter()
        .find(|e| e.name() == s.file)
        .unwrap_or_else(|| panic!("{} ({}) is not in the corpus manifest", s.file, s.id))
}

/// Both margins the corpus suites test: 0 (the default) and 3, written out
/// because MC-049 moved the default to 0.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning::default(),
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
    ]
}

/// The rect `process_file` cropped `entry` to at `t`, or the outcome as text.
fn crop_at(entry: &CorpusEntry, tmp: &tempfile::TempDir, t: &Tuning) -> Result<Rect, String> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Ok(rect),
        Outcome::Flagged { reason, .. } => Err(format!("Flagged {reason:?}")),
        Outcome::Failed { error } => Err(format!("Failed {error}")),
    }
}

/// What a criterion bounds the crop's top by: AC-1 bounds `f18`'s rows on
/// both sides; AC-2 bounds `f13`'s bottom only.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Top {
    Bounded,
    Free,
}

/// Every way `s`'s crop breaks its criterion at either margin, one line each:
/// not cropped (a `Flagged` outcome included), the mark not contained, a row
/// outside the furniture rows, or a column of the second window kept.
fn violations(s: &Settled, top: Top) -> Vec<String> {
    let entry = entry(s);
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mark = s.mark();
    let mut out = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let crop = match crop_at(&entry, &tmp, &t) {
            Ok(rect) => rect,
            Err(what) => {
                out.push(format!(
                    "margin_px {m}: not cropped ({what}) - the outcome must be Cropped"
                ));
                continue;
            }
        };
        let (right, bottom) = (crop.x + crop.w, crop.y + crop.h);
        let shown = format!(
            "crop {},{} {}x{} (columns {}..{right}, rows {}..{bottom})",
            crop.x, crop.y, crop.w, crop.h, crop.x, crop.y
        );
        if crop.x > mark.x || right < mark.x + mark.w {
            out.push(format!(
                "margin_px {m}: {shown} - does not cover the mark's columns {}..{}",
                mark.x,
                mark.x + mark.w
            ));
        }
        if crop.y > mark.y || bottom < mark.y + mark.h {
            out.push(format!(
                "margin_px {m}: {shown} - does not cover the mark's rows {}..{}",
                mark.y,
                mark.y + mark.h
            ));
        }
        if top == Top::Bounded && crop.y < s.rows.0 {
            out.push(format!(
                "margin_px {m}: {shown} - starts above row {}, keeping a tab strip or \
                 bookmarks bar row",
                s.rows.0
            ));
        }
        if bottom > s.rows.1 {
            out.push(format!(
                "margin_px {m}: {shown} - ends below row {}, keeping {} row(s) of furniture \
                 (scrollbar or taskbar)",
                s.rows.1,
                bottom - s.rows.1
            ));
        }
        if right > s.seam {
            out.push(format!(
                "margin_px {m}: {shown} - keeps {} column(s) of the second window, which \
                 starts at the seam, column {}; the crop must end at {} or left of it",
                right - s.seam.max(crop.x),
                s.seam,
                s.seam
            ));
        }
    }
    out
}

/// For every column `x` of `rgb`, the number of rows on which it changes
/// against column `x - 1` by more than [`SEAM_CHANGE`] (sum of the channels'
/// absolute differences). Column 0 has nothing to its left and counts 0.
fn change_counts(rgb: &image::RgbImage) -> Vec<usize> {
    let (w, h) = rgb.dimensions();
    let mut counts = vec![0usize; w as usize];
    for y in 0..h {
        for x in 1..w {
            let (a, b) = (rgb.get_pixel(x - 1, y), rgb.get_pixel(x, y));
            let change: i32 = (0..3)
                .map(|c| (i32::from(a[c]) - i32::from(b[c])).abs())
                .sum();
            if change > SEAM_CHANGE {
                counts[x as usize] += 1;
            }
        }
    }
    counts
}

/// The luma plane `process_file` sees.
fn luma(entry: &CorpusEntry) -> Luma {
    let bytes = std::fs::read(&entry.path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", entry.path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", entry.path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

// --- The premises -----------------------------------------------------------

/// Both entries are `tuning` (MC-064) and carry their settled marks (MC-062's
/// frozen table). Green on arrival: a guard on the oracle, so that AC-1 and
/// AC-2 cannot be passed by a manifest edit.
#[test]
#[ignore = "integration: reads the corpus manifest"]
fn both_entries_are_tuning_and_carry_their_settled_marks() {
    for s in [&F18, &F13] {
        let e = entry(s);
        assert_eq!(
            e.split,
            Split::Tuning,
            "{} ({}) must be tuning",
            s.file,
            s.id
        );
        assert_eq!(
            e.expect,
            Expect::Rect(s.mark()),
            "{} ({}): the mark is MC-062's frozen table and the user's ruling of \
             2026-09-30 that it stands; never edit the manifest to pass AC-1 or AC-2",
            s.file,
            s.id
        );
    }
}

/// The measured seam, re-measured: on each screenshot, right of the mark, the
/// **only** column whose colour changes against its left neighbour on more
/// than [`SEAM_ROWS`] rows is the seam the story's table names, on the number
/// of rows it names. The control is the mark's own edges, which change on
/// fewer rows than the threshold (they span the viewport, not the screen) and
/// must not be read as a seam.
#[test]
#[ignore = "integration: decodes two corpus entries"]
fn the_seam_between_the_windows_is_the_column_the_story_measured() {
    let mut wrong = Vec::new();
    for s in [&F18, &F13] {
        let e = entry(s);
        let decoded = image::open(&e.path)
            .unwrap_or_else(|err| panic!("decoding {}: {err}", e.path.display()))
            .to_rgb8();
        let counts = change_counts(&decoded);
        let mark_end = s.mark[0] + s.mark[2];
        let seams: Vec<(u32, usize)> = (mark_end..decoded.width())
            .map(|x| (x, counts[x as usize]))
            .filter(|&(_, n)| n > SEAM_ROWS)
            .collect();
        println!(
            "{} {}: columns right of the mark changing on more than {SEAM_ROWS} rows: \
             {seams:?}; the mark's edges {} and {} change on {} and {}",
            s.id,
            s.file,
            s.mark[0],
            mark_end,
            counts[s.mark[0] as usize],
            counts[mark_end as usize]
        );
        if seams != [(s.seam, s.seam_rows)] {
            wrong.push(format!(
                "{} ({}): the seam right of the mark should be exactly column {} changing \
                 on {} rows; measured {seams:?}",
                s.id, s.file, s.seam, s.seam_rows
            ));
        }
        for x in [s.mark[0], mark_end] {
            if counts[x as usize] > SEAM_ROWS {
                wrong.push(format!(
                    "{} ({}): the mark's edge column {x} changes on {} rows, over the seam \
                     threshold - the definition would not single out the seam",
                    s.id, s.file, counts[x as usize]
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-066's measured seams (## Context table) no longer hold:\n{}",
        wrong.join("\n")
    );
}

/// The premise under the `MAIN_CROPS` rows MC-066 re-pins in
/// `corpus_tuning_crops_unmoved.rs` (MC-066 `## Test plan`): on each
/// screenshot the column just left of the mark and the column just right of
/// it hold **one luma value**, the page margin's 11, on every row of the mark.
/// So `corpus_sides.rs` AC-1 (no page-background column outside the mark)
/// leaves the crop exactly the mark's columns at margin 0. The control is the
/// mark's own first and last columns, which are not one value.
#[test]
#[ignore = "integration: decodes two corpus entries"]
fn the_columns_beside_each_mark_are_the_page_margin_and_its_edges_are_not() {
    let mut wrong = Vec::new();
    for s in [&F18, &F13] {
        let e = entry(s);
        let img = luma(&e);
        let mark = s.mark();
        let distinct = |x: u32| {
            let mut v: Vec<u8> = (mark.y..mark.y + mark.h)
                .map(|y| img.data[(y * img.width + x) as usize])
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        let (left, right) = (mark.x - 1, mark.x + mark.w);
        for x in [left, right] {
            let v = distinct(x);
            if v != [11] {
                wrong.push(format!(
                    "{} ({}): column {x}, beside the mark, should be the page margin's one \
                     value 11 on every row of the mark; it holds {} value(s), first {:?}",
                    s.id,
                    s.file,
                    v.len(),
                    v.first()
                ));
            }
        }
        for x in [mark.x, right - 1] {
            if distinct(x).len() < 2 {
                wrong.push(format!(
                    "{} ({}): the mark's own edge column {x} is one value - the control \
                     on the premise fails",
                    s.id, s.file
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-066's premise for the re-pinned MAIN_CROPS rows no longer holds:\n{}",
        wrong.join("\n")
    );
}

// --- AC-1 -------------------------------------------------------------------

/// AC-1: `f18` is cropped to the reader's window. At both margins the outcome
/// is `Cropped` (a `Flagged` outcome does not satisfy it), and the crop covers
/// the mark's columns 651..1168 and rows 115..1399, lies within rows
/// 115..1399, and ends at column 1828 or left of it.
#[test]
#[ignore = "integration: decodes a corpus entry"]
fn f18_is_cropped_to_the_readers_window_at_both_margins() {
    let wrong = violations(&F18, Top::Bounded);
    assert!(
        wrong.is_empty(),
        "MC-066 AC-1: {} ({}) must be Cropped, its crop covering the mark {:?}, within rows \
         {}..{}, ending at column {} or left of it, at both margins:\n{}",
        F18.file,
        F18.id,
        F18.mark,
        F18.rows.0,
        F18.rows.1,
        F18.seam,
        wrong.join("\n")
    );
}

// --- AC-2 -------------------------------------------------------------------

/// AC-2: `f13` stays in the reader's window and drops the scrollbar. At both
/// margins the crop covers the mark's columns 635..1168 and rows 115..1392,
/// its bottom edge is at row 1392 or above, and it ends at column 1820 or
/// left of it.
#[test]
#[ignore = "integration: decodes a corpus entry"]
fn f13_stays_in_the_readers_window_and_drops_the_scrollbar_at_both_margins() {
    let wrong = violations(&F13, Top::Free);
    assert!(
        wrong.is_empty(),
        "MC-066 AC-2: {} ({}) must be cropped covering the mark {:?}, its bottom at row {} \
         or above, ending at column {} or left of it, at both margins:\n{}",
        F13.file,
        F13.id,
        F13.mark,
        F13.rows.1,
        F13.seam,
        wrong.join("\n")
    );
}
