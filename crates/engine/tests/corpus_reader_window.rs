//! MC-082 AC-1: each of six Eleceed screenshots is cropped to the reader's
//! window, not to the second browser window on its right. In each, the
//! `toongod` reader sits on the left and a YouTube window on the right; today
//! the app crops the YouTube window (MC-081 pins those crops as known
//! exceptions; MC-082 removes them). The generated reproduction of the
//! measured cause is `crates/core/tests/page_column_one_sided_margin.rs`
//! (AC-3); MC-066's corpus half of the same shape is
//! `tests/corpus_second_window.rs`.
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_reader_window -- --ignored
//! ```
//!
//! # Settled, measured, mechanical
//!
//! * **Settled, read out**: the marks (MC-081's frozen table, in the
//!   manifest); the seam and the furniture rows `T..B` per file (MC-082's
//!   AC-1 table, the user's, frozen 2026-10-04: *"Yes, all correct"*); zero
//!   clips; and that a `Flagged` outcome does not satisfy AC-1 (*"Must crop
//!   right"*).
//! * **Measured, re-measured in MC-082's RED and pinned here**: the seam's
//!   reading, by the Lead PO's definition (MC-082 `## Notes`, item 1): a
//!   column whose luma differs from its left neighbour's on more than
//!   [`SEAM_ROWS`] of the image's 1,440 rows, looked for in columns
//!   [`SEAM_SEARCH`]. The column that changes on the most rows there is the
//!   frozen seam.
//! * **Mechanical**: AC-1's containment and its column and row bounds.
//!
//! **Held-out discipline.** All six are `tuning` (MC-081); the premise test
//! checks it.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Luma, Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// The seam definition's row count: a column is a seam candidate when its
/// luma differs from its left neighbour's on more than this many of the
/// image's rows (MC-082 `## Context`, "more than 1,300 of 1,440 rows", as
/// MC-066 measured).
const SEAM_ROWS: usize = 1300;

/// Where the Lead PO looked for the seam: columns `1500 .. 2000` (MC-082
/// `## Notes`, item 1).
const SEAM_SEARCH: std::ops::Range<u32> = 1500..2000;

/// One of the six, as MC-082's AC-1 table settles it.
struct Settled {
    /// MC-081's id for it.
    id: &'static str,
    /// The corpus file.
    file: &'static str,
    /// The user's mark, MC-081's frozen table: `[x, y, w, h]`.
    mark: [u32; 4],
    /// The seam: the second window's first column. The crop must end at or
    /// left of it (end exclusive).
    seam: u32,
    /// The furniture rows `T .. B` (end exclusive): the browser bar ends at
    /// `T`, the scrollbar or taskbar starts at `B`.
    rows: (u32, u32),
    /// Every column of [`SEAM_SEARCH`] whose luma differs from its left
    /// neighbour's on more than [`SEAM_ROWS`] rows, with that row count, in
    /// column order: the reader's vertical scrollbar, the seam, and the
    /// column after it. Measured in MC-082's RED; equal, column for column
    /// and row for row, to the Lead PO's table (`## Notes` item 1).
    seam_reading: [(u32, usize); 3],
}

/// The six, in MC-081's order. Marks from MC-081's frozen table; seams and
/// rows from MC-082's AC-1 table.
const SIX: [Settled; 6] = [
    Settled {
        id: "e01",
        file: "2025-03-16 22_56_00.png",
        mark: [768, 115, 267, 1277],
        seam: 1820,
        rows: (115, 1392),
        seam_reading: [(1803, 1307), (1820, 1427), (1821, 1387)],
    },
    Settled {
        id: "e02",
        file: "2025-03-07 00_20_37.png",
        mark: [737, 115, 345, 1284],
        seam: 1828,
        rows: (115, 1399),
        seam_reading: [(1811, 1314), (1828, 1430), (1829, 1430)],
    },
    Settled {
        id: "e04",
        file: "2025-03-16 22_48_01.png",
        mark: [768, 115, 267, 1277],
        seam: 1820,
        rows: (115, 1392),
        seam_reading: [(1803, 1307), (1820, 1417), (1821, 1413)],
    },
    Settled {
        id: "e05",
        file: "2025-03-16 22_51_37.png",
        mark: [702, 115, 400, 1277],
        seam: 1820,
        rows: (115, 1392),
        seam_reading: [(1803, 1307), (1820, 1427), (1821, 1387)],
    },
    Settled {
        id: "e06",
        file: "2025-03-16 22_51_49.png",
        mark: [702, 115, 400, 1277],
        seam: 1820,
        rows: (115, 1392),
        seam_reading: [(1803, 1307), (1820, 1427), (1821, 1387)],
    },
    Settled {
        id: "e07",
        file: "2025-03-16 22_54_27.png",
        mark: [802, 115, 200, 1277],
        seam: 1820,
        rows: (115, 1392),
        seam_reading: [(1803, 1307), (1820, 1427), (1821, 1387)],
    },
];

impl Settled {
    fn mark(&self) -> Rect {
        let [x, y, w, h] = self.mark;
        Rect { x, y, w, h }
    }
}

/// Every corpus entry, loaded once per test.
fn entry<'a>(all: &'a [CorpusEntry], s: &Settled) -> &'a CorpusEntry {
    all.iter()
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

/// The luma plane `process_file` sees.
fn luma(entry: &CorpusEntry) -> Luma {
    let bytes = std::fs::read(&entry.path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", entry.path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", entry.path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// Every way `s`'s crop breaks AC-1 at either margin, one line each: not
/// cropped (a `Flagged` outcome included), the mark not contained, a column
/// of the second window kept, or a row outside `T .. B` kept.
fn violations(s: &Settled, e: &CorpusEntry) -> Vec<String> {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mark = s.mark();
    let (top, bottom_row) = s.rows;
    let mut out = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let tag = format!("{} ({}) margin_px {m}", s.id, s.file);
        let crop = match crop_at(e, &tmp, &t) {
            Ok(rect) => rect,
            Err(what) => {
                out.push(format!(
                    "{tag}: not cropped ({what}) - the outcome must be Cropped; leaving the \
                     screenshot alone does not satisfy AC-1 (\"Must crop right\")"
                ));
                continue;
            }
        };
        let (right, bottom) = (crop.x + crop.w, crop.y + crop.h);
        let shown = format!(
            "{tag}: crop {},{} {}x{} (columns {}..{right}, rows {}..{bottom})",
            crop.x, crop.y, crop.w, crop.h, crop.x, crop.y
        );
        if crop.x > mark.x || right < mark.x + mark.w {
            out.push(format!(
                "{shown} - does not contain the mark's columns {}..{}",
                mark.x,
                mark.x + mark.w
            ));
        }
        if crop.y > mark.y || bottom < mark.y + mark.h {
            out.push(format!(
                "{shown} - does not contain the mark's rows {}..{}",
                mark.y,
                mark.y + mark.h
            ));
        }
        if right > s.seam {
            out.push(format!(
                "{shown} - keeps {} column(s) of the second window, which starts at the seam, \
                 column {}; the crop must end at {} or left of it",
                right - s.seam.max(crop.x),
                s.seam,
                s.seam
            ));
        }
        if crop.y < top {
            out.push(format!(
                "{shown} - starts above row {top}, keeping {} row(s) of the browser bar",
                top - crop.y
            ));
        }
        if bottom > bottom_row {
            out.push(format!(
                "{shown} - ends below row {bottom_row}, keeping {} row(s) of furniture \
                 (scrollbar or taskbar)",
                bottom - bottom_row
            ));
        }
    }
    out
}

// --- The premises -----------------------------------------------------------

/// All six are `tuning` (MC-081) and carry the user's marks (MC-081's frozen
/// table), and each mark is inside its settled bounds: rows exactly `T .. B`
/// and columns left of the seam. Green on arrival: a guard on the oracle, so
/// that AC-1 cannot be passed by a manifest edit or by a table that
/// contradicts the marks.
#[test]
#[ignore = "integration: reads the corpus manifest"]
fn the_six_are_tuning_and_carry_the_users_marks_inside_their_settled_bounds() {
    let all = corpus::load();
    let mut wrong = Vec::new();
    for s in &SIX {
        let e = entry(&all, s);
        if e.split != Split::Tuning {
            wrong.push(format!("{} ({}): must be tuning", s.file, s.id));
        }
        if e.expect != Expect::Rect(s.mark()) {
            wrong.push(format!(
                "{} ({}): the manifest says {:?}, MC-081's frozen mark is {:?}; never edit the \
                 manifest to pass AC-1",
                s.file, s.id, e.expect, s.mark
            ));
        }
        let [x, y, w, h] = s.mark;
        if (y, y + h) != s.rows || x + w > s.seam {
            wrong.push(format!(
                "{} ({}): the mark {:?} must span exactly the rows T..B {:?} and end at the \
                 seam {} or left of it",
                s.file, s.id, s.mark, s.rows, s.seam
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The seam, re-measured: on each screenshot, among columns
/// [`SEAM_SEARCH`], the columns whose luma differs from the left
/// neighbour's on more than [`SEAM_ROWS`] rows are exactly the three of the
/// Lead PO's table, on exactly its row counts, and the leftmost column
/// changing on the most rows is the frozen seam (on `e02` the seam and the
/// column after it tie at 1,430; the seam is the second window's *first*
/// column). The reader's vertical scrollbar, 17 columns left of the seam,
/// is over the threshold too and is not the seam: the row count, not the
/// threshold alone, is what singles the seam out.
///
/// The search range is the definition's, not decoration: under this luma
/// reading the marks' own edges change on 1,262 to 1,335 rows (MC-082's RED),
/// some over the threshold, because the art and the margin differ on every
/// viewport row. MC-066's edge control (`tests/corpus_second_window.rs`)
/// reads RGB with a per-row threshold and does not transfer.
#[test]
#[ignore = "integration: decodes six corpus entries"]
fn the_seam_between_the_windows_is_the_column_the_story_froze() {
    let all = corpus::load();
    let mut wrong = Vec::new();
    for s in &SIX {
        let img = luma(entry(&all, s));
        let changes = |x: u32| {
            (0..img.height)
                .filter(|&y| {
                    let at = |c: u32| img.data[(y * img.width + c) as usize];
                    at(x) != at(x - 1)
                })
                .count()
        };
        let counts: Vec<(u32, usize)> = SEAM_SEARCH.map(|x| (x, changes(x))).collect();
        let most = counts
            .iter()
            .copied()
            .max_by_key(|&(x, n)| (n, std::cmp::Reverse(x)))
            .expect("a non-empty search range");
        let over: Vec<(u32, usize)> = counts
            .iter()
            .copied()
            .filter(|&(_, n)| n > SEAM_ROWS)
            .collect();
        println!(
            "{} {}: most-changing column in {SEAM_SEARCH:?} {most:?}; over {SEAM_ROWS}: \
             {over:?}",
            s.id, s.file
        );
        if most.0 != s.seam {
            wrong.push(format!(
                "{} ({}): the seam should be column {}, the leftmost column changing on the \
                 most rows; measured {most:?}",
                s.id, s.file, s.seam
            ));
        }
        if over != s.seam_reading {
            wrong.push(format!(
                "{} ({}): the columns over {SEAM_ROWS} changing rows should be {:?}; measured \
                 {over:?}",
                s.id, s.file, s.seam_reading
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-082's seams (AC-1 table, the Lead PO's measurement) no longer hold:\n{}",
        wrong.join("\n")
    );
}

// --- AC-1 -------------------------------------------------------------------

/// AC-1: each of the six is cropped to the reader's window. At both margins
/// the outcome is `Cropped` (a `Flagged` outcome does not satisfy it), and
/// the crop contains the user's mark, ends at the seam or left of it
/// (`x + w <= seam`) and lies within rows `T .. B` (`y >= T`,
/// `y + h <= B`).
#[test]
#[ignore = "integration: decodes six corpus entries"]
fn each_of_the_six_is_cropped_to_the_readers_window_at_both_margins() {
    let all = corpus::load();
    let wrong: Vec<String> = SIX
        .iter()
        .flat_map(|s| violations(s, entry(&all, s)))
        .collect();
    assert!(
        wrong.is_empty(),
        "MC-082 AC-1: each of the six must be Cropped, its crop containing the user's mark, \
         ending at the seam or left of it and lying within the furniture rows T..B, at both \
         margins. {} violation(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
