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
const MAIN_CROPS: [(&str, [u32; 4], [u32; 4]); 26] = [
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
];

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
                Err(what) => moved.push(format!("{name} at margin_px {}: {what}", t.margin_px)),
            }
        }
    }
    let mut pinned: Vec<String> = MAIN_CROPS.map(|(name, _, _)| name.to_string()).to_vec();
    pinned.sort();
    seen.sort();
    assert_eq!(
        seen, pinned,
        "MC-054 AC-3 must reach every marked tuning entry, and pin no other"
    );
    assert!(
        moved.is_empty(),
        "MC-054 AC-3: a fix for the viewport stage must leave every marked tuning crop \
         exactly as main (3449baa) produces it, x, y, w and h, at both margins. Moved:\n{}",
        moved.join("\n")
    );
}
