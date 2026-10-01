//! MC-065 AC-1 and AC-2: the two fresh screenshots whose right page edge was
//! cut short keep it, at both margins, and stay inside the browser viewport.
//!
//! - `f09`, `2025-12-08 17_22_50.png` (`toongod`): MC-063 cropped columns
//!   1006..1537 against a mark of 1006..1539 (end exclusive): 2 columns of the
//!   art's near-black outer edge cut.
//! - `f20`, `2025-08-07 01_13_55.png` (`rolia-scans`): cropped 1022..1516
//!   against 1022..1522: 6 columns of the page's own white paper cut.
//!
//! Both marks stand by the user's rulings of 2026-09-30 (MC-065 `## Notes`).
//! The generated reproductions of both causes, and the claim about where in
//! the pipeline they live, are `crates/core/tests/page_column_site_edge.rs`
//! (AC-4).
//!
//! # What is settled, read out and never re-derived
//!
//! - The marks, MC-062's frozen table ([`F09`], [`F20`]); the premise test
//!   checks the manifest still carries them, and that both are `tuning`.
//! - The furniture rows, MC-063's frozen oracle: `f09` 167 / 1400, `f20`
//!   115 / 1400 (end exclusive). A crop row outside them is a browser or
//!   taskbar row.
//! - Zero clips is absolute.
//!
//! # What is measured, and pinned here as the story's premise
//!
//! The story's `## Context` table, re-measured in RED from the pixels (luma
//! BT.601 over the mark's rows): the column just past each mark is the site's
//! background, one exact value (`f09` 1539 = 11, `f20` 1522 = 25), and the
//! mark's last column is not (`f09` 1538 mean 42.8 / spread 58.1, `f20` 1521
//! 253.7 / 2.4). [`the_column_past_each_mark_is_one_site_value_and_the_marks_last_column_is_not`]
//! checks both halves, so the premise is a test and not a sentence.
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_right_edge -- --ignored
//! ```
//!
//! **Held-out discipline.** These two entries only. They are `tuning` since
//! MC-064; no other fresh screenshot is decoded here.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Luma, Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// One of the story's two entries, as the story settles it.
struct Settled {
    /// MC-063's id for it.
    id: &'static str,
    /// The corpus file.
    file: &'static str,
    /// The mark, MC-062's frozen table: `[x, y, w, h]`.
    mark: [u32; 4],
    /// The furniture rows, MC-063's frozen oracle: the crop's rows must lie
    /// within `top .. bottom` (end exclusive).
    rows: (u32, u32),
    /// The site's single background value in the column just past the mark,
    /// measured in RED.
    site: u8,
    /// The mark's last column over the mark's rows, measured in RED and in
    /// the story's table: (mean, mean absolute deviation about the mean).
    art_edge: (f64, f64),
}

/// `f09`: AC-1.
const F09: Settled = Settled {
    id: "f09",
    file: "2025-12-08 17_22_50.png",
    mark: [1006, 167, 533, 1233],
    rows: (167, 1400),
    site: 11,
    art_edge: (42.8, 58.1),
};

/// `f20`: AC-2.
const F20: Settled = Settled {
    id: "f20",
    file: "2025-08-07 01_13_55.png",
    mark: [1022, 115, 500, 1285],
    rows: (115, 1400),
    site: 25,
    art_edge: (253.7, 2.4),
};

impl Settled {
    fn mark(&self) -> Rect {
        let [x, y, w, h] = self.mark;
        Rect { x, y, w, h }
    }

    /// The mark's right edge, end exclusive: the column the crop must reach.
    fn right_end(&self) -> u32 {
        self.mark[0] + self.mark[2]
    }
}

/// The corpus entry for `s`, which must be `tuning` and carry `s`'s mark.
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

/// Every way `s`'s crop breaks its criterion at either margin, one line each:
/// not cropped, a side of the mark cut, the right edge short of the mark's,
/// or a row outside the furniture rows.
fn violations(s: &Settled) -> Vec<String> {
    let entry = entry(s);
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mark = s.mark();
    let mut out = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let crop = match crop_at(&entry, &tmp, &t) {
            Ok(rect) => rect,
            Err(what) => {
                out.push(format!("margin_px {m}: not cropped ({what})"));
                continue;
            }
        };
        let (right, bottom) = (crop.x + crop.w, crop.y + crop.h);
        let shown = format!(
            "crop {},{} {}x{} (columns {}..{}, rows {}..{})",
            crop.x, crop.y, crop.w, crop.h, crop.x, right, crop.y, bottom
        );
        if right < s.right_end() {
            out.push(format!(
                "margin_px {m}: {shown} - right edge {right} cuts {} column(s) of art; it must \
                 be at {} or beyond",
                s.right_end() - right,
                s.right_end()
            ));
        }
        if crop.x > mark.x {
            out.push(format!(
                "margin_px {m}: {shown} - left edge cuts the mark's first column {}",
                mark.x
            ));
        }
        if crop.y > mark.y || bottom < mark.y + mark.h {
            out.push(format!(
                "margin_px {m}: {shown} - rows cut the mark's {}..{}",
                mark.y,
                mark.y + mark.h
            ));
        }
        if crop.y < s.rows.0 || bottom > s.rows.1 {
            out.push(format!(
                "margin_px {m}: {shown} - rows leave {}..{}, keeping a browser or taskbar row",
                s.rows.0, s.rows.1
            ));
        }
    }
    out
}

/// The luma plane `process_file` sees.
fn luma(entry: &CorpusEntry) -> Luma {
    let bytes = std::fs::read(&entry.path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", entry.path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", entry.path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// Column `x` over `rows`: (mean, mean absolute deviation about the mean,
/// distinct values).
fn column_stats(img: &Luma, x: u32, rows: std::ops::Range<u32>) -> (f64, f64, usize) {
    let values: Vec<u8> = rows
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect();
    let n = values.len() as f64;
    let mean = values.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
    let spread = values
        .iter()
        .map(|&v| (f64::from(v) - mean).abs())
        .sum::<f64>()
        / n;
    let mut distinct = values.clone();
    distinct.sort_unstable();
    distinct.dedup();
    (mean, spread, distinct.len())
}

// --- The premises -----------------------------------------------------------

/// Both entries are `tuning` (MC-064) and carry the settled marks (MC-062's
/// frozen table). Green on arrival: a guard on the oracle, so that AC-1 and
/// AC-2 below cannot be passed by a manifest edit.
#[test]
#[ignore = "integration: reads the corpus manifest"]
fn both_entries_are_tuning_and_carry_their_settled_marks() {
    for s in [&F09, &F20] {
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

/// The story's measured table, as a premise with its own control: over the
/// mark's rows, the column just past each mark is the site's background, **one
/// exact value**; the mark's last column is not, and its mean and spread are
/// the table's to 0.1.
#[test]
#[ignore = "integration: decodes two corpus entries"]
fn the_column_past_each_mark_is_one_site_value_and_the_marks_last_column_is_not() {
    let mut wrong = Vec::new();
    for s in [&F09, &F20] {
        let e = entry(s);
        let img = luma(&e);
        let mark = s.mark();
        let rows = mark.y..mark.y + mark.h;
        let site = column_stats(&img, s.right_end(), rows.clone());
        let art = column_stats(&img, s.right_end() - 1, rows);
        println!(
            "{} {}: column {} mean {:.2} spread {:.2} distinct {} | column {} mean {:.2} \
             spread {:.2} distinct {}",
            s.id,
            s.file,
            s.right_end() - 1,
            art.0,
            art.1,
            art.2,
            s.right_end(),
            site.0,
            site.1,
            site.2
        );
        if site.2 != 1 || site.0 != f64::from(s.site) {
            wrong.push(format!(
                "{}: column {} should be the single value {}; mean {:.2}, {} distinct values",
                s.id,
                s.right_end(),
                s.site,
                site.0,
                site.2
            ));
        }
        if art.2 == 1 || (art.0 - s.art_edge.0).abs() > 0.1 || (art.1 - s.art_edge.1).abs() > 0.1 {
            wrong.push(format!(
                "{}: the mark's last column {} should be the table's mean {} / spread {} and \
                 not one value; measured {:.2} / {:.2}, {} distinct",
                s.id,
                s.right_end() - 1,
                s.art_edge.0,
                s.art_edge.1,
                art.0,
                art.1,
                art.2
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-065's measured premise (## Context table) no longer holds:\n{}",
        wrong.join("\n")
    );
}

// --- AC-1 and AC-2 ----------------------------------------------------------

/// AC-1: `f09` keeps its right edge. At both margins the crop contains the
/// mark on all four sides - its right edge at column 1539 or beyond, end
/// exclusive - and its rows lie within 167..1400.
#[test]
#[ignore = "integration: decodes a corpus entry"]
fn f09_keeps_the_arts_near_black_right_edge_at_both_margins() {
    let wrong = violations(&F09);
    assert!(
        wrong.is_empty(),
        "MC-065 AC-1: {} ({}) must keep its mark {:?} whole, right edge at {} or beyond, \
         rows within {}..{}, at both margins:\n{}",
        F09.file,
        F09.id,
        F09.mark,
        F09.right_end(),
        F09.rows.0,
        F09.rows.1,
        wrong.join("\n")
    );
}

/// AC-2: `f20` keeps its right edge. At both margins the crop contains the
/// mark - its right edge at column 1522 or beyond, end exclusive - and its
/// rows lie within 115..1400.
#[test]
#[ignore = "integration: decodes a corpus entry"]
fn f20_keeps_the_pages_flat_white_right_edge_at_both_margins() {
    let wrong = violations(&F20);
    assert!(
        wrong.is_empty(),
        "MC-065 AC-2: {} ({}) must keep its mark {:?} whole, right edge at {} or beyond, \
         rows within {}..{}, at both margins:\n{}",
        F20.file,
        F20.id,
        F20.mark,
        F20.right_end(),
        F20.rows.0,
        F20.rows.1,
        wrong.join("\n")
    );
}
