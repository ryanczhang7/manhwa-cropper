//! MC-048, the settled readout: the viewport stage, asked directly, reproduces
//! `docs/wiki/chrome-row-search.md` section 4 on the corpus.
//!
//! The story's success condition, which can come out either way: the stage
//! committed in `crates/core` re-derives MC-031's chrome oracle - statistic
//! *Bg*, selector *ChromeStrip*, threshold 0.90, minimum run 16, gap 0, the
//! full margin width - and on the nineteen marked `tuning` entries where
//! section 4 lists a `chromeEnd` and a `taskbar` it must locate both **to the
//! row** on at least [`REPRODUCED_REQUIRED`] of them. On the two `2025-08-05`
//! WebPs, where section 5e records the full-margin oracle declining, it must
//! decline. If it does not reproduce section 4, the port is wrong or section 4
//! is, and the story stops and says which - the threshold is never calibrated
//! toward the table.
//!
//! # Why this is its own target
//!
//! It imports [`cropper_core::viewport`], which does not exist before MC-048.
//! Cargo compiles each file under `tests/` into its own binary, so the import
//! failure is confined here and `tests/corpus_viewport.rs` - which asks the same
//! corpus through `process_file` only - compiles and runs against either tree.
//!
//! # Which column the stage is handed
//!
//! The page column the pipeline itself locates: `detect`'s first five stages,
//! composed here from their public functions in the order
//! `crates/core/src/decide.rs` composes them - `trim_uniform`, `content_box`,
//! `trim_within`, `textured_box`, `page_column` - with no margin. That is the
//! rect MC-031's harness handed the oracle (`stages().column`), so the rows are
//! comparable with section 4's.
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_viewport_stage -- --ignored --nocapture
//! ```

#[path = "common/corpus.rs"]
mod corpus;

use std::path::Path;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::content::content_box;
use cropper_core::flat::{page_column, textured_box};
use cropper_core::trim::{trim_uniform, trim_within};
use cropper_core::viewport::{Viewport, locate};
use cropper_core::{Luma, Rect, Tuning};

/// `chrome-row-search.md` section 4, read out: `(file, chromeEnd, taskbar)`.
/// The same nineteen rows `tests/corpus_viewport.rs` reads; repeated rather
/// than shared because the two targets compile separately and one of them must
/// compile before the stage exists.
const SECTION_4: [(&str, u32, u32); 19] = [
    ("2025-10-14 23_29_06.png", 167, 1400),
    ("2025-10-14 23_30_20.png", 167, 1400),
    ("2025-10-20 15_37_25.png", 167, 1400),
    ("2026-01-05 13_33_41.png", 167, 1400),
    ("2026-01-05 13_45_59.png", 167, 1400),
    ("2026-01-05 13_49_39.png", 167, 1400),
    ("Screenshot (67).png", 167, 1392),
    ("Screenshot (70).jpg", 167, 1392),
    ("Screenshot (75).png", 167, 1392),
    ("Screenshot (93).jpg", 167, 1392),
    ("Screenshot (103).jpg", 167, 1392),
    ("Screenshot (1661).png", 133, 1392),
    ("Screenshot (2582).jpg", 133, 1392),
    ("Screenshot (2630).jpg", 133, 1392),
    ("Screenshot (2698).jpg", 133, 1392),
    ("Screenshot (2708).jpg", 133, 1392),
    ("Screenshot (2744).jpg", 133, 1392),
    ("Screenshot (3187).png", 137, 1392),
    ("Screenshot (3538).png", 137, 1392),
];

/// The two entries section 4 and section 5e record the full-margin oracle
/// declining on.
const DECLINES: [&str; 2] = ["2025-08-05 00_11_13.webp", "2025-08-05 00_11_27.webp"];

/// The story's success condition: section 4 reproduced to the row on at least
/// this many of the nineteen.
const REPRODUCED_REQUIRED: usize = 18;

/// The marked `tuning` entries, in manifest order. Never `held-out`.
fn marked() -> Vec<CorpusEntry> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .filter(|entry| matches!(entry.expect, Expect::Rect(_)))
        .collect()
}

/// The luma plane of `path`, through the engine's own decoder and luma
/// conversion, so the stage sees what `process_file` hands `detect`.
fn luma(path: &Path) -> Luma {
    let bytes =
        std::fs::read(path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let decoded = image::load_from_memory(&bytes)
        .unwrap_or_else(|err| panic!("decoding {}: {err}", path.display()));
    cropper_engine::codec::to_luma(&decoded)
}

/// The page column `detect` locates, before the viewport stage and before the
/// margin: the first five stages, composed as `decide.rs` composes them.
fn page_column_of(img: &Luma, t: &Tuning) -> Rect {
    let first = trim_uniform(img, t).expect("a corpus page is not one flat colour");
    let found = content_box(img, first, t);
    let second = trim_within(img, found.rect, t).expect("the content box is not one flat colour");
    let textured = textured_box(img, second, t);
    page_column(img, textured, t)
}

/// The settled readout. `locate` reproduces section 4's `chromeEnd` and
/// `taskbar` to the row on at least [`REPRODUCED_REQUIRED`] of nineteen, and
/// declines on both WebPs.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_viewport_stage_reproduces_the_rows_mc031_located_and_declines_where_it_declined() {
    let t = Tuning::default();
    let mut rows = Vec::new();
    let mut reproduced = 0usize;
    let mut differs = Vec::new();
    let mut spoke_on_a_webp = Vec::new();
    let mut seen = Vec::new();

    for entry in marked() {
        let name = entry.name();
        let img = luma(&entry.path);
        let column = page_column_of(&img, &t);
        let got = locate(&img, column, &t);

        if DECLINES.contains(&name.as_str()) {
            seen.push(name.clone());
            if got.is_some() {
                spoke_on_a_webp.push(format!("{name}: {got:?}"));
            }
            rows.push(format!("{name:<26} expected None       got {got:?}"));
            continue;
        }
        let Some(&(_, top, bottom)) = SECTION_4.iter().find(|(file, _, _)| *file == name) else {
            panic!("{name} is a marked tuning entry in neither SECTION_4 nor DECLINES");
        };
        seen.push(name.clone());
        let expected = Viewport { top, bottom };
        if got.as_ref() == Some(&expected) {
            reproduced += 1;
        } else {
            differs.push(format!("{name}: expected {expected:?}, got {got:?}"));
        }
        rows.push(format!("{name:<26} expected {top}..{bottom}  got {got:?}"));
    }

    let mut printed = String::from(
        "the viewport stage on the marked tuning entries, against chrome-row-search.md section 4\n",
    );
    for row in &rows {
        printed.push_str(row);
        printed.push('\n');
    }
    println!("{printed}");

    assert_eq!(
        seen.len(),
        SECTION_4.len() + DECLINES.len(),
        "every one of the twenty-one marked tuning entries must be reached"
    );
    assert!(
        spoke_on_a_webp.is_empty(),
        "section 5e: at the full margin width the oracle declines on both WebPs, \
         and the stage must too. It spoke on {spoke_on_a_webp:?}.\n\n{printed}"
    );
    assert!(
        reproduced >= REPRODUCED_REQUIRED,
        "the stage must reproduce section 4's chromeEnd and taskbar to the row on at \
         least {REPRODUCED_REQUIRED} of {}; it does on {reproduced}. The port is \
         wrong or section 4 is - say which; never calibrate toward the table.\n{}\n\n{printed}",
        SECTION_4.len(),
        differs.join("\n")
    );
}
