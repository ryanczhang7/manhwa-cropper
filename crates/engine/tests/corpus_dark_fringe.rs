//! MC-067 AC-1 and AC-2: the two corpus screenshots whose page edge ends in a
//! dark, two-column **fringe** keep it, at both margins.
//!
//! - `f09`, `2025-12-08 17_22_50.png` (`toongod`): MC-063 cropped columns
//!   1006..1537 against a mark of 1006..1539 (end exclusive), cutting the
//!   art's near-black outer edge, 1537..1538. The user ruled the mark stands
//!   (2026-09-30, MC-065 `## Notes`).
//! - `Screenshot (3538).png` (`toongod`): the same fringe on both sides,
//!   973..974 and 1571..1572, which the app trims today (`975,137 596x1255`
//!   at margin 0). By luma and by colour it is the same thing as `f09`'s
//!   (MC-067 `## Notes`), and the user ruled on 2026-10-01 that 3538's crop
//!   grows by it: **"Let 3538 grow"**, to `973,137 600x1255` at margin 0 and
//!   `970,137 606x1255` at margin 3. That reverses MC-053's preference for
//!   3538 (its Open question 5) by the user's own ruling.
//!
//! The generated reproduction of the cause is
//! `crates/core/tests/page_column_dark_fringe.rs` (AC-4).
//!
//! # What is settled, read out and never re-derived
//!
//! - `f09`'s mark, MC-062's frozen table, `1006,167 533x1233`; the guard test
//!   checks the manifest still carries it, and that the entry is `tuning`.
//! - `f09`'s furniture rows, MC-063's frozen oracle: 167 / 1400 (end
//!   exclusive).
//! - 3538's crop at both margins, the user's ruling of 2026-10-01.
//! - Zero clips is absolute.
//!
//! # What is measured, and pinned here as the story's premise
//!
//! The Lead PO's luma table (MC-065 `## Notes`, 2026-10-01, BT.601): on both
//! screenshots the site is one exact value, 11, from the column just past the
//! fringe; and each fringe is one column near the site (within 6 of 11) on
//! about half its rows, then one on about 90%.
//! [`the_columns_beside_both_fringes_are_one_site_value_and_the_fringes_measure_as_the_story_says`]
//! checks both halves, so the premise is a test and not a sentence. It is
//! also what forces `f09`'s margin-0 crop to be exactly the mark's columns:
//! the columns either side, 1005 and 1539, are the site, one value.
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_dark_fringe -- --ignored
//! ```
//!
//! **Held-out discipline.** These two `tuning` entries only.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Luma, Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// `f09`'s file.
const F09: &str = "2025-12-08 17_22_50.png";
/// `f09`'s mark, MC-062's frozen table and the user's ruling of 2026-09-30.
const F09_MARK: [u32; 4] = [1006, 167, 533, 1233];
/// `f09`'s furniture rows, MC-063's frozen oracle, end exclusive.
const F09_ROWS: (u32, u32) = (167, 1400);

/// 3538's file.
const S3538: &str = "Screenshot (3538).png";
/// 3538's crop, `[x, y, w, h]`, at margin_px 0 and 3: the user's ruling of
/// 2026-10-01 ("Let 3538 grow"), MC-067 AC-2.
const S3538_AT_0: [u32; 4] = [973, 137, 600, 1255];
const S3538_AT_3: [u32; 4] = [970, 137, 606, 1255];

/// The site's value on both screenshots.
const SITE: u8 = 11;
/// "Near the site", as the Lead PO measured the fringes: within this many
/// levels of [`SITE`].
const NEAR_SITE: u8 = 6;

/// A range of image rows, `(first, one past the last)`.
type Rows = (u32, u32);

/// "About half its rows": the near-site share an **inner** fringe column must
/// fall in. Invented here, a description of the story's words; the measured
/// values sit near its middle (0.500 to 0.510).
const ABOUT_HALF: std::ops::RangeInclusive<f64> = 0.40..=0.60;
/// "About 90%": the near-site share an **outer** fringe column must fall in.
/// Invented here; measured 0.845 to 0.861.
const ABOUT_NINETY: std::ops::RangeInclusive<f64> = 0.80..=0.95;

/// Which fringe column: the inner one, next to the art, or the outer one,
/// next to the site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ring {
    Inner,
    Outer,
}

/// The story's fringe table: `(file, column, rows, which, the Lead PO's
/// near-site share)`. The share is of the column's pixels over `rows` within
/// [`NEAR_SITE`] of [`SITE`]. `f09` over its mark's rows; 3538 over its
/// viewport rows, 137..1392, as the Lead PO's RGB table reads it.
///
/// **Re-measured in MC-067's RED, and the Lead PO's numbers do not reproduce
/// to 0.01** on this crate's luma (`codec::to_luma`, BT.601 rounded to an
/// integer): 0.507 / 0.861 for `f09`, 0.848 / 0.510 / 0.500 / 0.845 for 3538.
/// The Lead PO's program read fractional luma (its medians are 15.2, 9.2).
/// The claim the story makes - one column near the site on about half its
/// rows, then one on about 90% - holds on both instruments, so that is what
/// is asserted ([`ABOUT_HALF`], [`ABOUT_NINETY`]); the Lead PO's value is
/// kept for the record and printed beside the measured one.
const FRINGES: [(&str, u32, Rows, Ring, f64); 6] = [
    (F09, 1537, (167, 1400), Ring::Inner, 0.472),
    (F09, 1538, (167, 1400), Ring::Outer, 0.861),
    (S3538, 973, (137, 1392), Ring::Outer, 0.915),
    (S3538, 974, (137, 1392), Ring::Inner, 0.539),
    (S3538, 1571, (137, 1392), Ring::Inner, 0.520),
    (S3538, 1572, (137, 1392), Ring::Outer, 0.912),
];

/// The columns just outside each fringe, which must be the site, one value:
/// `(file, column, rows)`.
const SITE_COLUMNS: [(&str, u32, Rows); 4] = [
    (F09, 1005, (167, 1400)),
    (F09, 1539, (167, 1400)),
    (S3538, 972, (137, 1392)),
    (S3538, 1573, (137, 1392)),
];

fn rect([x, y, w, h]: [u32; 4]) -> Rect {
    Rect { x, y, w, h }
}

/// The corpus entry named `file`.
fn entry(file: &str) -> CorpusEntry {
    corpus::load()
        .into_iter()
        .find(|e| e.name() == file)
        .unwrap_or_else(|| panic!("{file} is not in the corpus manifest"))
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

/// A crop as text, with its end-exclusive bounds.
fn shown(crop: Rect) -> String {
    format!(
        "crop {},{} {}x{} (columns {}..{}, rows {}..{})",
        crop.x,
        crop.y,
        crop.w,
        crop.h,
        crop.x,
        crop.x + crop.w,
        crop.y,
        crop.y + crop.h
    )
}

/// The luma plane `process_file` sees.
fn luma(entry: &CorpusEntry) -> Luma {
    let bytes = std::fs::read(&entry.path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", entry.path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", entry.path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// Column `x`'s pixels over `rows` (end exclusive).
fn column(img: &Luma, x: u32, (top, bottom): Rows) -> Vec<u8> {
    (top..bottom)
        .map(|y| img.data[(y * img.width + x) as usize])
        .collect()
}

// --- The premises -----------------------------------------------------------

/// `f09` is `tuning` (MC-064) and carries its settled mark (MC-062's frozen
/// table). Green on arrival: a guard on the oracle, so that AC-1 below cannot
/// be passed by a manifest edit. 3538 must be `tuning` too; its mark is not
/// what AC-2 pins, so it is not guarded here.
#[test]
#[ignore = "integration: reads the corpus manifest"]
fn f09_and_3538_are_tuning_and_f09_carries_its_settled_mark() {
    let f09 = entry(F09);
    assert_eq!(f09.split, Split::Tuning, "{F09} (f09) must be tuning");
    assert_eq!(
        f09.expect,
        Expect::Rect(rect(F09_MARK)),
        "{F09} (f09): the mark is MC-062's frozen table and the user's ruling of \
         2026-09-30 that it stands; never edit the manifest to pass AC-1"
    );
    assert_eq!(entry(S3538).split, Split::Tuning, "{S3538} must be tuning");
}

/// The story's measured premise, with its own control: the column just
/// outside each fringe is the site, **one exact value** (11), and each fringe
/// column is not one value: the inner one near the site on about half its
/// rows ([`ABOUT_HALF`]), the outer one on about 90% ([`ABOUT_NINETY`]).
#[test]
#[ignore = "integration: decodes two corpus entries"]
fn the_columns_beside_both_fringes_are_one_site_value_and_the_fringes_measure_as_the_story_says() {
    let mut wrong = Vec::new();
    for file in [F09, S3538] {
        let img = luma(&entry(file));
        for &(_, x, rows) in SITE_COLUMNS.iter().filter(|(f, _, _)| *f == file) {
            let values = column(&img, x, rows);
            if !values.iter().all(|&v| v == SITE) {
                let mut distinct = values.clone();
                distinct.sort_unstable();
                distinct.dedup();
                wrong.push(format!(
                    "{file}: column {x} over rows {}..{} should be the single site value \
                     {SITE}; it holds {distinct:?}",
                    rows.0, rows.1
                ));
            }
        }
        for &(_, x, rows, ring, lead_po) in FRINGES.iter().filter(|(f, ..)| *f == file) {
            let values = column(&img, x, rows);
            let near = values
                .iter()
                .filter(|&&v| v.abs_diff(SITE) <= NEAR_SITE)
                .count() as f64
                / values.len() as f64;
            let one_value = values.iter().all(|&v| v == values[0]);
            let range = match ring {
                Ring::Inner => ABOUT_HALF,
                Ring::Outer => ABOUT_NINETY,
            };
            println!(
                "{file}: {ring:?} fringe column {x} near-site {near:.3} (Lead PO {lead_po:.3})"
            );
            if one_value || !range.contains(&near) {
                wrong.push(format!(
                    "{file}: {ring:?} fringe column {x} over rows {}..{} should be near the \
                     site on {range:?} of its rows, and not one value; measured {near:.3}, one \
                     value {one_value}",
                    rows.0, rows.1
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-067's measured premise (the Lead PO's fringe table) no longer holds:\n{}",
        wrong.join("\n")
    );
}

// --- AC-1 -------------------------------------------------------------------

/// AC-1: `f09` keeps its right edge. At both margins the outcome is
/// `Cropped`, the crop contains the mark on all four sides - its right edge at
/// column 1539 or beyond, end exclusive - and its rows lie within 167..1400.
/// At margin 0 it keeps no column of the site beyond the mark: its columns are
/// exactly the mark's, 1006..1539.
#[test]
#[ignore = "integration: decodes a corpus entry"]
fn f09_keeps_the_arts_near_black_right_edge_at_both_margins() {
    let e = entry(F09);
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mark = rect(F09_MARK);
    let (mark_right, mark_bottom) = (mark.x + mark.w, mark.y + mark.h);
    let mut wrong = Vec::new();
    for t in both_margins() {
        let m = t.margin_px;
        let crop = match crop_at(&e, &tmp, &t) {
            Ok(rect) => rect,
            Err(what) => {
                wrong.push(format!("margin_px {m}: not cropped ({what})"));
                continue;
            }
        };
        let (right, bottom) = (crop.x + crop.w, crop.y + crop.h);
        let s = shown(crop);
        if right < mark_right {
            wrong.push(format!(
                "margin_px {m}: {s} - right edge {right} cuts {} column(s) of art; it must be \
                 at {mark_right} or beyond",
                mark_right - right
            ));
        }
        if crop.x > mark.x {
            wrong.push(format!(
                "margin_px {m}: {s} - left edge cuts the mark's first column {}",
                mark.x
            ));
        }
        if crop.y > mark.y || bottom < mark_bottom {
            wrong.push(format!(
                "margin_px {m}: {s} - rows cut the mark's {}..{mark_bottom}",
                mark.y
            ));
        }
        if crop.y < F09_ROWS.0 || bottom > F09_ROWS.1 {
            wrong.push(format!(
                "margin_px {m}: {s} - rows leave {}..{}, keeping a browser or taskbar row",
                F09_ROWS.0, F09_ROWS.1
            ));
        }
        if m == 0 && (crop.x, right) != (mark.x, mark_right) {
            wrong.push(format!(
                "margin_px 0: {s} - columns must be exactly the mark's, {}..{mark_right}: no \
                 column of the site (one value, {SITE}) beyond it, and none of the art cut",
                mark.x
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-067 AC-1: {F09} (f09) must be Cropped, keep its mark {F09_MARK:?} whole (right \
         edge at {mark_right} or beyond), rows within {}..{}, at both margins, and at \
         margin 0 be exactly the mark's columns:\n{}",
        F09_ROWS.0,
        F09_ROWS.1,
        wrong.join("\n")
    );
}

// --- AC-2 -------------------------------------------------------------------

/// AC-2: 3538 grows by its fringe, as the user ruled on 2026-10-01, and by
/// nothing more: exactly `973,137 600x1255` at margin 0 and `970,137
/// 606x1255` at margin 3.
#[test]
#[ignore = "integration: decodes a corpus entry"]
fn screenshot_3538_grows_by_its_fringe_on_each_side_and_by_nothing_more() {
    let e = entry(S3538);
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut wrong = Vec::new();
    for (t, want) in both_margins().iter().zip([S3538_AT_0, S3538_AT_3]) {
        match crop_at(&e, &tmp, t) {
            Ok(crop) if crop == rect(want) => {}
            Ok(crop) => wrong.push(format!(
                "margin_px {}: {}, ruled {want:?} (left edge off by {:+}, right edge off by {:+})",
                t.margin_px,
                shown(crop),
                i64::from(crop.x) - i64::from(want[0]),
                i64::from(crop.x + crop.w) - i64::from(want[0] + want[2])
            )),
            Err(what) => wrong.push(format!("margin_px {}: not cropped ({what})", t.margin_px)),
        }
    }
    assert!(
        wrong.is_empty(),
        "MC-067 AC-2: {S3538}'s crop must grow by its 2-column fringe on each side, the \
         user's ruling of 2026-10-01 (\"Let 3538 grow\"), and by nothing more:\n{}",
        wrong.join("\n")
    );
}
