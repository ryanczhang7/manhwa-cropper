//! MC-082 AC-1: each of six Eleceed screenshots is cropped to the reader's
//! window, not to the second browser window on its right. In each, the
//! `toongod` reader sits on the left and a YouTube window on the right; today
//! the app crops the YouTube window (MC-081 pins those crops as known
//! exceptions; MC-082 removes them). The generated reproduction of the
//! measured cause is `crates/core/tests/page_column_one_sided_margin.rs`
//! (AC-3); MC-066's corpus half of the same shape is
//! `tests/corpus_second_window.rs`.
//!
//! **MC-083 AC-1 joins it**: `e03` (`2025-03-07 01_02_31.png`), the seventh
//! of MC-081's Eleceed screenshots, judged on the same three things - its
//! mark contained, rows within its frozen `T..B`, columns left of its frozen
//! seam - at both margins ([`E03`]). Its cause is not the six's: the page is
//! split in two at a near-black stretch of its own art and the crop keeps
//! the wider half at full height (`727,0 483x1440`). The generated
//! reproduction is `crates/core/tests/page_column_dark_stretch.rs` (MC-083
//! AC-3). Its seam is read by MC-083's definition, which differs from
//! MC-082's on `e03` alone: see [`E03`] and
//! [`e03s_seam_is_the_first_column_over_the_threshold_past_the_readers_scrollbar`].
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

/// MC-083's `e03`, as its AC-1 settles it: the mark is MC-081's frozen one,
/// and `T..B` = 115..1399 and the seam 1828 are the user's, frozen
/// 2026-10-05 (*"Yes, all correct"*). The seam reading is the Lead PO's
/// (MC-083 `## Notes`, "Seam and furniture rows"): the reader's scrollbar
/// 1811 on 1,314 rows, the seam 1828 on 1,412 and the column after it, 1829,
/// on 1,424.
///
/// **The seam is the first column over [`SEAM_ROWS`] past the reader's
/// scrollbar**, MC-083 AC-1's definition. On the six that is also the
/// leftmost of the most-changing columns, MC-082's reading, and the two
/// agree (the premise below checks it from the six's pinned readings). On
/// `e03` they do not: 1829 changes on more rows than 1828, so MC-082's
/// reading would give 1829. MC-082's seam test is therefore left over the
/// six exactly as it was, and `e03`'s seam has a test of its own.
const E03: Settled = Settled {
    id: "e03",
    file: "2025-03-07 01_02_31.png",
    mark: [610, 115, 600, 1284],
    seam: 1828,
    rows: (115, 1399),
    seam_reading: [(1811, 1314), (1828, 1412), (1829, 1424)],
};

/// How far left of the seam the reader's vertical scrollbar starts, in
/// columns, on all seven: 1820 - 1803 on `e01`, `e04`..`e07`, 1828 - 1811 on
/// `e02` and `e03` (MC-082's and MC-083's readings).
const SCROLLBAR_TO_SEAM: u32 = 17;

/// The seam by MC-083 AC-1's definition, read off a seam reading (the columns
/// over [`SEAM_ROWS`] in [`SEAM_SEARCH`], in column order): the first column
/// past the reader's scrollbar, which is the first column of the reading.
fn first_past_the_scrollbar(reading: &[(u32, usize)]) -> Option<u32> {
    let (scrollbar, _) = *reading.first()?;
    reading.iter().map(|&(x, _)| x).find(|&x| x > scrollbar)
}

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
    let wrong: Vec<String> = SIX.iter().flat_map(|s| oracle_wrong(s, &all)).collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Every way `s`'s corpus entry or settled bounds contradict the oracle: not
/// `tuning`, a manifest mark that is not MC-081's frozen one, or a mark that
/// does not span exactly `T .. B` or does not end at the seam or left of it.
fn oracle_wrong(s: &Settled, all: &[CorpusEntry]) -> Vec<String> {
    let mut wrong = Vec::new();
    let e = entry(all, s);
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
    wrong
}

/// `img`'s seam reading over [`SEAM_SEARCH`]: the column that changes against
/// its left neighbour on the most rows (the leftmost, on a tie), with its
/// count, and every column changing on more than [`SEAM_ROWS`] rows, with its
/// count, in column order.
fn seam_reading(img: &Luma) -> ((u32, usize), Vec<(u32, usize)>) {
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
    let over = counts
        .iter()
        .copied()
        .filter(|&(_, n)| n > SEAM_ROWS)
        .collect();
    (most, over)
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
        let (most, over) = seam_reading(&img);
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

// --- MC-083: e03 ------------------------------------------------------------

/// MC-083's premise: `e03` is `tuning` (MC-081) and carries the user's mark
/// (MC-081's frozen table), and the mark spans exactly its frozen `T..B`,
/// 115..1399, and ends left of its frozen seam, 1828. Green on arrival: a
/// guard on the oracle, so that AC-1 cannot be passed by a manifest edit.
#[test]
#[ignore = "integration: reads the corpus manifest"]
fn e03_is_tuning_and_carries_the_users_mark_inside_its_settled_bounds() {
    let wrong = oracle_wrong(&E03, &corpus::load());
    assert!(wrong.is_empty(), "MC-083: {}", wrong.join("\n"));
}

/// MC-083's seam, re-measured by its AC-1 definition: on `e03`, among
/// columns [`SEAM_SEARCH`], the columns whose luma differs from the left
/// neighbour's on more than [`SEAM_ROWS`] rows are exactly the Lead PO's
/// three, on exactly his row counts; the first is the reader's scrollbar,
/// [`SCROLLBAR_TO_SEAM`] columns left of the seam; and the first column over
/// the threshold past it is the frozen seam, 1828.
///
/// It also pins why `e03` is not in MC-082's seam test: there the
/// most-changing column is 1829 (1,424 rows against the seam's 1,412), so
/// MC-082's "leftmost most-changing" reading would move the seam one column
/// right. And it checks that the two readings agree on the six, from their
/// pinned readings (which MC-082's seam test measures), so one definition
/// holds for all seven.
#[test]
#[ignore = "integration: decodes one corpus entry"]
fn e03s_seam_is_the_first_column_over_the_threshold_past_the_readers_scrollbar() {
    let all = corpus::load();
    let mut wrong = Vec::new();
    let (most, over) = seam_reading(&luma(entry(&all, &E03)));
    println!(
        "{} {}: most-changing column in {SEAM_SEARCH:?} {most:?}; over {SEAM_ROWS}: {over:?}",
        E03.id, E03.file
    );
    if over != E03.seam_reading {
        wrong.push(format!(
            "the columns over {SEAM_ROWS} changing rows should be {:?}; measured {over:?}",
            E03.seam_reading
        ));
    }
    if over.first().map(|&(x, _)| x + SCROLLBAR_TO_SEAM) != Some(E03.seam) {
        wrong.push(format!(
            "the first column over the threshold should be the reader's scrollbar, \
             {SCROLLBAR_TO_SEAM} columns left of the seam {}; measured {over:?}",
            E03.seam
        ));
    }
    if first_past_the_scrollbar(&over) != Some(E03.seam) {
        wrong.push(format!(
            "the seam should be column {}, the first column over the threshold past the \
             reader's scrollbar; measured {over:?}",
            E03.seam
        ));
    }
    if most != (1829, 1424) {
        wrong.push(format!(
            "MC-082's reading (the leftmost most-changing column) should give (1829, 1424) on \
             e03, one column right of the seam - the reason e03 has a seam test of its own; \
             measured {most:?}"
        ));
    }
    for s in &SIX {
        let first_past = first_past_the_scrollbar(&s.seam_reading);
        let scrollbar = s.seam_reading[0].0 + SCROLLBAR_TO_SEAM;
        if first_past != Some(s.seam) || scrollbar != s.seam {
            wrong.push(format!(
                "{} ({}): MC-083's definition must give MC-082's frozen seam {} from its pinned \
                 reading {:?}; gives {first_past:?}, scrollbar + {SCROLLBAR_TO_SEAM} = {scrollbar}",
                s.id, s.file, s.seam, s.seam_reading
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-083's seam for e03 ({}; AC-1, the Lead PO's measurement) no longer holds:\n{}",
        E03.file,
        wrong.join("\n")
    );
}

/// MC-083 AC-1: `e03` keeps its whole page and drops the furniture. At both
/// margins the outcome is `Cropped` (a `Flagged` outcome does not satisfy
/// it), and the crop contains the user's mark `610,115 600x1284` - so no
/// column of the art left of the near-black stretch is cut - lies within rows
/// 115..1399 (`y >= 115`, `y + h <= 1399`) and ends at the seam or left of it
/// (`x + w <= 1828`). Today the crop is `727,0 483x1440` (`724,0 489x1440`
/// at margin 3): 117 columns of art cut, the bookmarks bar and the taskbar
/// kept.
#[test]
#[ignore = "integration: decodes one corpus entry"]
fn e03_keeps_its_whole_page_and_drops_the_furniture_at_both_margins() {
    let wrong = violations(&E03, entry(&corpus::load(), &E03));
    assert!(
        wrong.is_empty(),
        "MC-083 AC-1: e03 must be Cropped, its crop containing the user's mark, lying within \
         the furniture rows T..B 115..1399 and ending at the seam 1828 or left of it, at both \
         margins. {} violation(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
