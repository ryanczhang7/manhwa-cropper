//! MC-075 AC-1 and AC-3, and MC-076 AC-1: on the corpus, the crop keeps no
//! row of browser or OS furniture on the entries MC-071's held-out run moved to `tuning`, judged
//! against MC-071's **frozen** furniture oracle rather than against the rows
//! the viewport stage locates.
//!
//! # Why a check of its own
//!
//! `corpus_viewport.rs`'s furniture predicate reads the rows the viewport
//! stage located. Where the stage **declines** there are none, so the entry
//! sits on a `STAGE_DECLINED` list and no suite judges its rows at all - which
//! is how `n06`'s crop kept rows 0..1440, the browser bar, the scrollbar and
//! the taskbar, through every suite (MC-075 `## Context`). The check here reads
//! `T` and `B` out of MC-071's `## Notes`, settled oracle data, so a declined
//! stage is seen.
//!
//! # What is settled, what is mechanical
//!
//! * **Settled, read out and never derived**: [`MC071_FURNITURE`] (`T`, `B`
//!   per entry, from MC-071 `## Notes`, "The furniture oracle, FROZEN
//!   2026-10-02"); the marks (MC-068, in the manifest).
//! * **Mechanical**: AC-1's columns for `n06` ([`N06_COLUMNS`], MC-075 AC-1)
//!   and for `n13` ([`N13_COLUMNS`], MC-076 AC-1);
//!   the known exception list ([`KNOWN_FURNITURE_KEPT`]), exact in both
//!   directions.
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it.
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_furniture_rows -- --ignored --nocapture
//! ```
//!
//! **Held-out discipline.** `tuning` entries only: the four are `tuning` since
//! MC-072.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};

// --- Settled: read out of MC-071 `## Notes`, never derived ---------------------

/// MC-071's frozen furniture oracle (`docs/backlog/stories/MC-071.md`,
/// `## Notes`, "The furniture oracle, FROZEN 2026-10-02 before any crop of
/// the 15"), for the four entries its held-out run failed and MC-072 moved to
/// `tuning`: `(MC-071 id, file, T, B)`, in manifest order. `T` is the first
/// row below the browser bar; `B`, end-exclusive, is the first row of the
/// scrollbar, site footer or taskbar, whichever starts first. Copied, never
/// derived; a change here is a change to settled oracle data.
const MC071_FURNITURE: [(&str, &str, u32, u32); 4] = [
    ("n02", "2025-03-07 00_05_58.png", 115, 1399),
    ("n06", "2025-03-13 12_01_01.png", 115, 1392),
    ("n05", "2025-11-01 12_34_31.png", 167, 1400),
    ("n13", "Screenshot (2507).png", 133, 1392),
];

// --- Mechanical: the story's pins ------------------------------------------------

/// MC-075 AC-1: `n06`.
const N06: &str = "2025-03-13 12_01_01.png";

/// MC-075 AC-1: `n06`'s crop columns, `(x, w)` at `margin_px` 0 and at 3 -
/// its mark's columns, 698..1098, and those widened by the margin, 695..1101.
/// The columns are kept: the story moves rows.
const N06_COLUMNS: [(u32, (u32, u32)); 2] = [(0, (698, 400)), (3, (695, 406))];

/// MC-076 AC-1: `n13`.
const N13: &str = "Screenshot (2507).png";

/// MC-076 AC-1: `n13`'s crop columns, `(x, w)` at `margin_px` 0 and at 3 -
/// its mark's columns, 977..1577, and those widened by the margin, 974..1580.
/// The columns are kept: the story moves rows.
const N13_COLUMNS: [(u32, (u32, u32)); 2] = [(0, (977, 600)), (3, (974, 606))];

/// The moved entries whose crop still keeps a furniture row, each owned by an
/// open story that removes it here. **Exact in both directions**:
/// [`the_rows_of_every_moved_mc071_entrys_crop_lie_between_its_frozen_t_and_b`]
/// fails if any other entry keeps furniture, and fails if a listed entry stops
/// keeping it at either margin.
///
/// **MC-076 empties it** (its AC-1): `Screenshot (2507).png` (`n13`), whose
/// crop started at row 40 (37 at margin 3) against `T` 133 - the viewport
/// stage declined beside two flat reader panels of another tone hugging the
/// page (MC-076 `## Context`, "Measured") - is judged like the other three.
/// The check still runs on all four entries at both margins.
const KNOWN_FURNITURE_KEPT: [&str; 0] = [];

// --- Harness ----------------------------------------------------------------------

/// The four, with their marks and frozen rows, in [`MC071_FURNITURE`]'s order.
/// Panics unless each is a marked `tuning` entry: a name that drifted would
/// leave every test below checking nothing.
fn moved_entries() -> Vec<(&'static str, CorpusEntry, Rect, u32, u32)> {
    let marked: Vec<(CorpusEntry, Rect)> = corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .filter_map(|entry| match entry.expect {
            Expect::Rect(rect) => Some((entry, rect)),
            Expect::Flag => None,
        })
        .collect();
    MC071_FURNITURE
        .iter()
        .map(|&(id, name, top, bottom)| {
            let (entry, mark) = marked
                .iter()
                .find(|(entry, _)| entry.name() == name)
                .unwrap_or_else(|| panic!("{id} {name} must be a marked `tuning` entry"));
            (id, entry.clone(), *mark, top, bottom)
        })
        .collect()
}

/// `Tuning::default()` at `margin_px`.
fn at_margin(margin_px: u32) -> Tuning {
    Tuning {
        margin_px,
        ..Tuning::default()
    }
}

/// The rect `process_file` crops `entry` to at `t`, or the outcome as text.
fn crop_at(entry: &CorpusEntry, tmp: &tempfile::TempDir, t: &Tuning) -> Result<Rect, String> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Ok(rect),
        Outcome::Flagged { reason, .. } => Err(format!("Flagged {reason:?}")),
        Outcome::Failed { error } => Err(format!("Failed {error}")),
    }
}

/// Whether `outer` contains `inner` entirely.
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// The check: whether every row of `rect` lies in `t_row .. b_row`. `None`
/// when it does, or which furniture it keeps.
fn furniture_kept(rect: Rect, t_row: u32, b_row: u32) -> Option<String> {
    let end = rect.y + rect.h;
    match (rect.y < t_row, end > b_row) {
        (false, false) => None,
        (above, below) => Some(format!(
            "rows {}..{end} against T {t_row}, B {b_row}:{}{}",
            rect.y,
            if above {
                format!(" {} rows above T (browser bar)", t_row - rect.y)
            } else {
                String::new()
            },
            if below {
                format!(
                    " {} rows from B down (scrollbar, footer or taskbar)",
                    end - b_row
                )
            } else {
                String::new()
            }
        )),
    }
}

/// `rect` widened by one row at the top: the control's input. A crop already
/// at row 0 cannot widen up; it keeps its rows, and its check is whatever it
/// was.
fn widened_at_the_top(rect: Rect) -> Rect {
    if rect.y == 0 {
        return rect;
    }
    Rect {
        y: rect.y - 1,
        h: rect.h + 1,
        ..rect
    }
}

/// One moved entry's crop at one margin, with its frozen rows.
struct MovedCrop {
    /// MC-071's id.
    id: &'static str,
    /// The file name.
    name: String,
    /// `margin_px`.
    margin: u32,
    /// MC-071's frozen `T`.
    top: u32,
    /// MC-071's frozen `B`, end-exclusive.
    bottom: u32,
    /// What `process_file` did.
    got: Result<Rect, String>,
}

/// Each moved entry's crop at margins 0 and 3, in [`MC071_FURNITURE`]'s
/// order.
fn moved_crops() -> Vec<MovedCrop> {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut out = Vec::new();
    for (id, entry, _, top, bottom) in moved_entries() {
        for margin in [0, 3] {
            out.push(MovedCrop {
                id,
                name: entry.name(),
                margin,
                top,
                bottom,
                got: crop_at(&entry, &tmp, &at_margin(margin)),
            });
        }
    }
    out
}

// --- AC-1: n06 drops the bars ------------------------------------------------------

/// MC-075 AC-1. `n06` (`2025-03-13 12_01_01.png`), at margins 0 and 3, is
/// `Cropped`; the crop contains the mark `698,115 400x1277`, lies within rows
/// 115..1392 (MC-071's frozen `T` and `B`: no tab-strip, toolbar, scrollbar or
/// taskbar row), and keeps its columns, 698..1098 at margin 0 and 695..1101 at
/// margin 3.
///
/// On `main` (`6af092f`) the crop is rows 0..1440 at both margins: the
/// viewport stage declines beside a second window textured on every row.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn n06s_crop_drops_the_browser_bar_scrollbar_and_taskbar_and_keeps_its_page_at_both_margins() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let (_, entry, mark, top, bottom) = moved_entries()
        .into_iter()
        .find(|(_, entry, _, _, _)| entry.name() == N06)
        .expect("n06 is one of MC-071's four");
    let mut bad = Vec::new();
    let mut rows = Vec::new();
    for (margin, (x, w)) in N06_COLUMNS {
        match crop_at(&entry, &tmp, &at_margin(margin)) {
            Ok(rect) => {
                rows.push(format!(
                    "m{margin}: crop {},{} {}x{} (rows {}..{})",
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    rect.y,
                    rect.y + rect.h
                ));
                if !contains(rect, mark) {
                    bad.push(format!(
                        "m{margin}: the crop {rect:?} clips the mark {mark:?}"
                    ));
                }
                if let Some(what) = furniture_kept(rect, top, bottom) {
                    bad.push(format!("m{margin}: the crop keeps furniture, {what}"));
                }
                if (rect.x, rect.w) != (x, w) {
                    bad.push(format!(
                        "m{margin}: the crop's columns (x, w) are ({}, {}), must be ({x}, {w})",
                        rect.x, rect.w
                    ));
                }
            }
            Err(what) => bad.push(format!("m{margin}: not cropped - {what}")),
        }
    }
    assert!(
        bad.is_empty(),
        "MC-075 AC-1: {N06} (n06) must be cropped to its page, within rows {top}..{bottom}, \
         columns kept, at both margins.\n{}\n\n{}",
        bad.join("\n"),
        rows.join("\n")
    );
}

// --- MC-076 AC-1: n13 starts below the browser bar ------------------------------

/// MC-076 AC-1. `n13` (`Screenshot (2507).png`), at margins 0 and 3, is
/// `Cropped`; the crop contains the mark `977,133 600x1259` (MC-068), lies
/// within rows 133..1392 (MC-071's frozen `T` and `B`: no tab-strip or
/// toolbar row, no taskbar row), and keeps its columns, 977..1577 at margin 0
/// and 974..1580 at margin 3.
///
/// On `main` (`1d7a921`) the crop is rows 40..1392 at margin 0 and 37..1395 at
/// margin 3: the viewport stage declines beside two flat reader panels of
/// another tone hugging the page (whole-margin share 0.889), and only the
/// chrome peel's tab strip and taskbar come off.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn n13s_crop_starts_below_the_browser_bar_and_keeps_its_page_at_both_margins() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let (_, entry, mark, top, bottom) = moved_entries()
        .into_iter()
        .find(|(_, entry, _, _, _)| entry.name() == N13)
        .expect("n13 is one of MC-071's four");
    let mut bad = Vec::new();
    let mut rows = Vec::new();
    for (margin, (x, w)) in N13_COLUMNS {
        match crop_at(&entry, &tmp, &at_margin(margin)) {
            Ok(rect) => {
                rows.push(format!(
                    "m{margin}: crop {},{} {}x{} (rows {}..{})",
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    rect.y,
                    rect.y + rect.h
                ));
                if !contains(rect, mark) {
                    bad.push(format!(
                        "m{margin}: the crop {rect:?} clips the mark {mark:?}"
                    ));
                }
                if let Some(what) = furniture_kept(rect, top, bottom) {
                    bad.push(format!("m{margin}: the crop keeps furniture, {what}"));
                }
                if (rect.x, rect.w) != (x, w) {
                    bad.push(format!(
                        "m{margin}: the crop's columns (x, w) are ({}, {}), must be ({x}, {w})",
                        rect.x, rect.w
                    ));
                }
            }
            Err(what) => bad.push(format!("m{margin}: not cropped - {what}")),
        }
    }
    assert!(
        bad.is_empty(),
        "MC-076 AC-1: {N13} (n13) must be cropped to its page, within rows {top}..{bottom}, \
         columns kept, at both margins.\n{}\n\n{}",
        bad.join("\n"),
        rows.join("\n")
    );
}

// --- AC-3: a corpus check that sees a declined stage ---------------------------

/// MC-075 AC-3. On each of MC-071's four moved entries, at margins 0 and 3,
/// every crop row lies in `T .. B` from MC-071's frozen oracle
/// ([`MC071_FURNITURE`]) - whether or not the viewport stage located a
/// viewport. The entries that keep furniture are exactly
/// [`KNOWN_FURNITURE_KEPT`], at both margins.
///
/// On `main` (`6af092f`) it fails on `n06`, rows 0..1440 at both margins.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_rows_of_every_moved_mc071_entrys_crop_lie_between_its_frozen_t_and_b() {
    let mut kept: Vec<(String, u32)> = Vec::new();
    let mut rows = Vec::new();
    for MovedCrop {
        id,
        name,
        margin,
        top,
        bottom,
        got,
    } in moved_crops()
    {
        let verdict = match got {
            Ok(rect) => furniture_kept(rect, top, bottom),
            Err(what) => Some(format!("not cropped - {what}")),
        };
        rows.push(format!(
            "{id} {name:<26} m{margin}  {}",
            verdict.as_deref().unwrap_or("ok")
        ));
        if verdict.is_some() {
            kept.push((name, margin));
        }
    }
    let printed = rows.join("\n");
    println!("{printed}");
    let known: Vec<(String, u32)> = MC071_FURNITURE
        .iter()
        .filter(|(_, name, _, _)| KNOWN_FURNITURE_KEPT.contains(name))
        .flat_map(|(_, name, _, _)| [0, 3].map(|m| (name.to_string(), m)))
        .collect();
    assert_eq!(
        kept, known,
        "MC-075 AC-3: on MC-071's moved entries every crop row must lie between the frozen \
         T and B at margins 0 and 3; the only entries that may keep a furniture row are \
         KNOWN_FURNITURE_KEPT (empty since MC-076 took n13 off), and those must still keep it. \
         `(file, margin_px)`, `left` measured.\n\n{printed}"
    );
}

/// MC-075 AC-3's control: the same check, given each moved entry's crop
/// widened by one row at the top, fails on every entry at both margins - so
/// the check reads `T` to the row and is not passing for want of looking.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn widened_by_one_row_at_the_top_every_moved_entrys_crop_fails_the_check() {
    let mut passed = Vec::new();
    let mut rows = Vec::new();
    let mut seen = 0usize;
    for MovedCrop {
        id,
        name,
        margin,
        top,
        bottom,
        got,
    } in moved_crops()
    {
        seen += 1;
        let verdict = match got {
            Ok(rect) => furniture_kept(widened_at_the_top(rect), top, bottom),
            Err(what) => Some(format!("not cropped - {what}")),
        };
        rows.push(format!(
            "{id} {name:<26} m{margin} widened  {}",
            verdict.as_deref().unwrap_or("PASSES")
        ));
        if verdict.is_none() {
            passed.push(format!("{id} {name} m{margin}"));
        }
    }
    let printed = rows.join("\n");
    println!("{printed}");
    assert_eq!(seen, MC071_FURNITURE.len() * 2, "four entries, two margins");
    assert!(
        passed.is_empty(),
        "MC-075 AC-3's control: widened by one row at the top, every moved entry's crop \
         must fail the check; it passes on {passed:?}.\n\n{printed}"
    );
}
