//! MC-054 AC-3: nothing on the corpus moves. Every marked `tuning` crop is
//! exactly what `main` produces, `x`, `y`, `w` and `h`, at both margins.
//!
//! MC-054 changes the viewport stage (stage 6, the row axis) so that no
//! generated page loses rows to it. The corpus is where that stage was
//! measured (MC-031 section 4, MC-048, MC-052), so a fix that touches it must
//! leave every real crop where it is. The oracle is `main` itself: the table
//! below is `process_file`'s crop of each marked `tuning` entry on `3449baa`
//! (post-MC-055 `main`, release), measured in MC-054's RED. It is a
//! regression pin, green on arrival by construction; MC-054's `## Test plan`
//! says what earns it.
//!
//! Why a new table and not `corpus_page_column.rs`'s: that one pins the 23
//! entries MC-055 had to leave alone and skips the three it moved on purpose
//! (`THE_THREE`), whose crops are held there only by "keeps the art" checks.
//! AC-3 here is all 26. The 23 rows below are that table's, re-confirmed on
//! `3449baa`; the three are new.
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it.
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_tuning_crops_unmoved -- --ignored
//! ```
//!
//! **Held-out discipline.** `tuning` entries only (MC-054 `## Out of scope`).

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// `(file, [x, y, w, h] at margin_px 3, [x, y, w, h] at margin_px 0)` for
/// every marked `tuning` entry, in manifest order, as `process_file` crops it
/// on `3449baa` (release). Measured, never calibrated.
const MAIN_CROPS: [(&str, [u32; 4], [u32; 4]); 48] = [
    (
        "2025-08-05 00_11_13.webp",
        [950, 15, 646, 1425],
        [953, 18, 640, 1422],
    ),
    (
        "2025-08-05 00_11_27.webp",
        [1003, 15, 539, 1425],
        [1006, 18, 533, 1422],
    ),
    (
        "2025-10-14 23_29_06.png",
        [1000, 167, 546, 1233],
        [1003, 167, 540, 1233],
    ),
    (
        "2025-10-14 23_30_20.png",
        [1030, 167, 486, 1233],
        [1033, 167, 480, 1233],
    ),
    (
        "2025-10-20 15_37_25.png",
        [1004, 167, 538, 1233],
        [1007, 167, 532, 1233],
    ),
    (
        "2026-01-05 13_33_41.png",
        [1071, 167, 404, 1233],
        [1074, 167, 398, 1233],
    ),
    (
        "2026-01-05 13_45_59.png",
        [1036, 167, 473, 1233],
        [1039, 167, 467, 1233],
    ),
    (
        "2026-01-05 13_49_39.png",
        [1036, 167, 473, 1233],
        [1039, 167, 467, 1233],
    ),
    (
        "Screenshot (67).png",
        [1007, 167, 531, 1225],
        [1010, 167, 525, 1225],
    ),
    (
        "Screenshot (70).jpg",
        [1007, 167, 531, 1225],
        [1010, 167, 525, 1225],
    ),
    (
        "Screenshot (75).png",
        [1070, 167, 406, 1225],
        [1073, 167, 400, 1225],
    ),
    (
        "Screenshot (93).jpg",
        [1136, 167, 273, 1225],
        [1139, 167, 267, 1225],
    ),
    (
        "Screenshot (103).jpg",
        [1070, 167, 406, 1225],
        [1073, 167, 400, 1225],
    ),
    (
        "Screenshot (1661).png",
        [1070, 133, 406, 1259],
        [1073, 133, 400, 1259],
    ),
    (
        "Screenshot (2582).jpg",
        [1003, 133, 539, 1259],
        [1006, 133, 533, 1259],
    ),
    (
        "Screenshot (2630).jpg",
        [952, 133, 642, 1259],
        [955, 133, 636, 1259],
    ),
    (
        "Screenshot (2698).jpg",
        [950, 133, 645, 1259],
        [953, 133, 639, 1259],
    ),
    (
        "Screenshot (2708).jpg",
        [1071, 133, 405, 1259],
        [1074, 133, 399, 1259],
    ),
    (
        "Screenshot (2744).jpg",
        [945, 133, 654, 1259],
        [948, 133, 648, 1259],
    ),
    (
        "Screenshot (3187).png",
        [981, 137, 582, 1255],
        [984, 137, 576, 1255],
    ),
    (
        "Screenshot (3538).png",
        [972, 137, 602, 1255],
        [975, 137, 596, 1255],
    ),
    (
        "2025-03-06 01_22_45.png",
        [640, 115, 539, 1259],
        [643, 115, 533, 1259],
    ),
    (
        "2025-03-07 00_58_06.png",
        [667, 115, 486, 1284],
        [670, 115, 480, 1284],
    ),
    (
        "2025-07-17 14_41_58.png",
        [925, 167, 696, 1233],
        [928, 167, 690, 1233],
    ),
    (
        "2025-07-17 14_55_10.png",
        [925, 167, 696, 1233],
        [928, 167, 690, 1233],
    ),
    (
        "Screenshot (73).png",
        [1000, 167, 546, 1225],
        [1003, 167, 540, 1225],
    ),
    // MC-056: two of the three the user moved from `held-out` on 2026-09-29,
    // read out of one run of `process_file` on `c3fee28` (release; crates
    // unchanged since `d2876f5`) in MC-056's RED, on a scratch copy with the
    // move applied. Measured, never chosen. The third, `2025-07-17
    // 14_20_23.png`, is not cropped at either margin (`Flagged
    // Detector(Ambiguous)`), which a row of this table cannot say; it is named
    // in KNOWN_NOT_CROPPED below (MC-056's AC-4, as amended).
    (
        "2025-08-03 11_27_49.png",
        [1003, 115, 539, 1285],
        [1006, 115, 533, 1285],
    ),
    (
        "Screenshot (68).png",
        [955, 167, 635, 1225],
        [958, 167, 629, 1225],
    ),
    // MC-062: the 16 marked entries among the 23 spent held-out entries the
    // user moved to `tuning` on 2026-09-30, in manifest order, read out of one
    // run of `process_file` on `7c36b5d` (release; crates unchanged since
    // `d2876f5`) in MC-062's RED, on a scratch copy with the move applied.
    // Measured, never chosen. All 16 are cropped at both margins.
    (
        "2025-03-04 11_09_29.png",
        [1010, 115, 523, 1285],
        [1013, 115, 517, 1285],
    ),
    (
        "2025-03-07 00_41_10.png",
        [677, 115, 466, 1284],
        [680, 115, 460, 1284],
    ),
    (
        "2025-03-07 01_10_37.png",
        [607, 115, 606, 1284],
        [610, 115, 600, 1284],
    ),
    (
        "2025-08-03 11_13_19.png",
        [950, 115, 646, 1285],
        [953, 115, 640, 1285],
    ),
    (
        "2025-08-03 20_54_19.png",
        [1003, 115, 539, 1285],
        [1006, 115, 533, 1285],
    ),
    (
        "2025-08-04 23_24_37.png",
        [1003, 115, 539, 1285],
        [1006, 115, 533, 1285],
    ),
    (
        "2025-08-07 15_07_56.png",
        [1070, 115, 406, 1285],
        [1073, 115, 400, 1285],
    ),
    (
        "2025-08-07 15_47_10.png",
        [970, 124, 606, 1276],
        [973, 124, 600, 1276],
    ),
    (
        "2025-08-07 15_57_50.png",
        [988, 115, 569, 1285],
        [991, 115, 563, 1285],
    ),
    (
        "2025-09-29 14_33_15.png",
        [981, 167, 582, 1233],
        [984, 167, 576, 1233],
    ),
    (
        "Screenshot (56).png",
        [1030, 167, 485, 1225],
        [1033, 167, 479, 1225],
    ),
    (
        "Screenshot (59).png",
        [1036, 167, 473, 1225],
        [1039, 167, 467, 1225],
    ),
    (
        "Screenshot (1720).png",
        [1007, 133, 540, 1259],
        [1010, 133, 534, 1259],
    ),
    (
        "Screenshot (3605).png",
        [1004, 137, 538, 1245],
        [1007, 137, 532, 1242],
    ),
    (
        "Screenshot (3606).png",
        [1003, 137, 539, 1251],
        [1006, 137, 533, 1248],
    ),
    (
        "Screenshot (3625).png",
        [945, 137, 654, 1255],
        [948, 137, 648, 1255],
    ),
    // MC-064: the four fresh entries MC-063 read per file, moved to `tuning` by
    // the user's ruling of 2026-09-30, in manifest order, read out of one run
    // of `process_file` on `43e8e61` (release; crates unchanged since
    // `d2876f5`) in MC-064's RED, on a scratch copy with only the four `split`
    // values changed. Measured, never chosen. All four are cropped at both
    // margins. Three clip their mark (`2025-03-06 12_48_06.png`, `2025-08-07
    // 01_13_55.png`, `2025-12-08 17_22_50.png`) and `2025-03-16 22_47_44.png`
    // keeps the browser scrollbar: these rows pin that, as `main` produces it,
    // and the zero-clip suites name them as known exceptions. MC-065 moves them.
    (
        "2025-03-06 12_48_06.png",
        [1825, 0, 723, 1440],
        [1828, 0, 717, 1440],
    ),
    (
        "2025-03-16 22_47_44.png",
        [632, 115, 1928, 1288],
        [635, 115, 1922, 1285],
    ),
    //
    // MC-065 re-pins `f20` (`2025-08-07 01_13_55.png`), whose right edge it
    // fixes. Its rows were MC-064's measured clip, `[1019, 115, 500, 1285]` /
    // `[1022, 115, 494, 1285]`. The rows below are **forced by the criteria,
    // not chosen or guessed**. At margin 0 the crop must contain the mark
    // (MC-065 AC-2), its rows must lie within the furniture rows (115..1400,
    // MC-063's frozen oracle), and no column outside the mark may be page
    // background (`corpus_sides.rs` AC-1, unloosened by MC-065 AC-3). The
    // column just past the mark (1522) is the site's single-valued background
    // (spread 0.0), and so is the column just before it. That leaves exactly
    // the mark, `1022,115 500x1285`. At margin 3 `margin::expand` widens the
    // columns by 3 on each side, and the rows stay clamped to the viewport,
    // which the same row bound requires.
    (
        "2025-08-07 01_13_55.png",
        [1019, 115, 506, 1285],
        [1022, 115, 500, 1285],
    ),
    // `f09` (`2025-12-08 17_22_50.png`) keeps MC-064's rows, its measured
    // clip: MC-065 moved it to MC-067 (MC-065 `## Amendments`, 2026-10-01).
    (
        "2025-12-08 17_22_50.png",
        [1003, 167, 537, 1233],
        [1006, 167, 531, 1233],
    ),
];

/// MC-056, AC-4 as amended on 2026-09-29 (the user's ruling on Open question
/// 2): the one marked `tuning` entry `main` does not crop - the detector flags
/// it `Ambiguous` at both margins - so it has no row in [`MAIN_CROPS`], and is
/// named here instead. **Exact in both directions**: the test fails if any
/// other marked `tuning` entry is not cropped at either margin, and fails if
/// this one is cropped at either, so the story that fixes it has to move it
/// from this list into [`MAIN_CROPS`] with the crop it then measures.
const KNOWN_NOT_CROPPED: [&str; 1] = ["2025-07-17 14_20_23.png"];

/// The marked `tuning` entries, in manifest order. Never `held-out`.
fn marked() -> Vec<CorpusEntry> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .filter(|entry| matches!(entry.expect, Expect::Rect(_)))
        .collect()
}

/// Both margins, as the story defines them: 3 and 0. The 3 is written out
/// because MC-049 moved `Tuning::default().margin_px` to 0, and the pins
/// above were measured at 3.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
        Tuning {
            margin_px: 0,
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

#[test]
#[ignore = "integration: decodes the whole corpus"]
fn every_marked_tuning_crop_is_exactly_what_main_produces_at_both_margins() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut moved = Vec::new();
    let mut seen = Vec::new();
    // MC-056: every entry not cropped, `(file, margin_px)`, compared exactly
    // with KNOWN_NOT_CROPPED at both margins below. Before MC-056 a
    // non-cropped entry went into `moved`; any entry other than the known
    // exception still fails, now on that comparison.
    let mut not_cropped: Vec<(String, u32)> = Vec::new();
    for entry in marked() {
        let name = entry.name();
        seen.push(name.clone());
        let pinned = MAIN_CROPS
            .iter()
            .find(|(file, _, _)| *file == name)
            .map(|&(_, m3, m0)| [m3, m0]);
        for (i, t) in both_margins().iter().enumerate() {
            let want = pinned.map(|p| {
                let [x, y, w, h] = p[i];
                Rect { x, y, w, h }
            });
            match crop_at(&entry, &tmp, t) {
                Ok(rect) if Some(rect) == want => {}
                Ok(rect) => moved.push(format!(
                    "{name} at margin_px {}: {rect:?}, main {want:?}",
                    t.margin_px
                )),
                Err(what) => {
                    moved.extend(
                        pinned.map(|_| format!("{name} at margin_px {}: {what}", t.margin_px)),
                    );
                    not_cropped.push((name.clone(), t.margin_px));
                }
            }
        }
    }
    let known: Vec<(String, u32)> = KNOWN_NOT_CROPPED
        .iter()
        .flat_map(|name| both_margins().map(|t| (name.to_string(), t.margin_px)))
        .collect();
    assert_eq!(
        not_cropped, known,
        "MC-056: at both margins, the only marked tuning entry main does not crop is the \
         known exception (KNOWN_NOT_CROPPED), and it must still not be cropped - if a fix \
         makes it crop, move it into MAIN_CROPS. `(file, margin_px)`, `left` measured"
    );
    let mut pinned: Vec<String> = MAIN_CROPS
        .map(|(name, _, _)| name.to_string())
        .into_iter()
        .chain(KNOWN_NOT_CROPPED.map(String::from))
        .collect();
    pinned.sort();
    seen.sort();
    assert_eq!(
        seen, pinned,
        "MC-054 AC-3 must reach every marked tuning entry, and pin no other: MAIN_CROPS \
         and MC-056's KNOWN_NOT_CROPPED together"
    );
    assert!(
        moved.is_empty(),
        "MC-054 AC-3: a fix for the viewport stage must leave every marked tuning crop \
         exactly as main (3449baa) produces it, x, y, w and h, at both margins. Moved:\n{}",
        moved.join("\n")
    );
}
