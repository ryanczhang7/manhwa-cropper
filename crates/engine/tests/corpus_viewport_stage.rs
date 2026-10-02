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

/// MC-052's two split-screen screenshots, moved from `held-out` to `tuning`
/// by the user on 2026-09-24. Section 4 never saw them, so they are in
/// neither table above and are **never** merged into [`SECTION_4`], which
/// claims to be section 4: this readout skips them, and
/// `tests/corpus_viewport.rs` (`READER_WINDOW`) is where MC-052 judges
/// them, against rows it measured itself.
const NOT_IN_SECTION_4: [&str; 2] = ["2025-03-06 01_22_45.png", "2025-03-07 00_58_06.png"];

/// MC-053's three dark-art entries, moved from `held-out` to `tuning` by the
/// user on 2026-09-24, `(file, top, bottom)`. Section 4 never saw them either,
/// so they too are **never** merged into [`SECTION_4`]. Unlike MC-052's two
/// they are single-window screenshots, and their rows are the ones this stage
/// **measured itself** on `c004d96` (post-MC-052 `main`) in MC-053's RED - the
/// same rows `tests/corpus_viewport.rs` (`STAGE_MEASURED`) holds. This readout
/// checks them exactly but apart from section 4: they are a regression guard
/// on the stage's own answer, not a reproduction of MC-031's, and they do not
/// count toward [`REPRODUCED_REQUIRED`].
///
/// **MC-056 adds its three**, moved from `held-out` by the user on 2026-09-29,
/// interleaved in manifest order: `2025-07-17 14_20_23.png`, `2025-08-03
/// 11_27_49.png` and `Screenshot (68).png`. Section 4 never saw them either.
/// Their rows are what `locate` returns beside the page column on `c3fee28`
/// (crates unchanged since `d2876f5`), read out of one run in MC-056's RED on
/// a scratch copy with the move applied - not chosen. `14_20_23`, which the
/// detector flags `Ambiguous`, still has a viewport: the stage locates it.
///
/// **MC-062 adds the 16 marked entries among the 23 spent held-out entries**,
/// moved to `tuning` by the user on 2026-09-30, interleaved in manifest order.
/// Section 4 never saw them either. Same provenance: what `locate` returns
/// beside the page column on `7c36b5d` (crates unchanged since `d2876f5`),
/// read out of one run in MC-062's RED on a scratch copy with the move
/// applied - not chosen. The stage locates a viewport on all 16.
///
/// **MC-064 adds three of its four**, the fresh entries MC-063 read per file,
/// moved to `tuning` by the user's ruling of 2026-09-30, last in manifest
/// order. Section 4 never saw them either. Same provenance: what `locate`
/// returns beside the page column on `43e8e61` (crates unchanged since
/// `d2876f5`), read out of one run in MC-064's RED on a scratch copy with only
/// the four `split` values changed - not chosen. On the fourth the stage
/// declines: [`STAGE_DECLINED`].
///
/// **MC-066 re-measures `2025-03-16 22_47_44.png` (`f13`) and adds `2025-03-06
/// 12_48_06.png` (`f18`)**, off [`STAGE_DECLINED`]. Their rows are **forced by
/// MC-066's criteria, not measured or chosen**: each crop's rows must be exactly
/// its furniture rows (MC-063's frozen oracle: `f18` 115..1399, `f13`
/// 115..1392) at margins 0 and 3 (MC-066 AC-1, AC-2), and
/// `corpus_sides.rs::the_top_and_bottom_edges_move_by_exactly_the_margin_change`
/// (unloosened, MC-066 AC-3) allows that only where `locate`, beside the page
/// column composed as here, returns exactly those rows - an uncut side moves
/// by the margin.
///
/// **MC-068 adds 21**, the rest of MC-062's fresh draw, spent by MC-063's run
/// and moved to `tuning` (MC-068 AC-4), interleaved with MC-064's four in
/// manifest order. Section 4 never saw them either. Same provenance as
/// MC-062's: what `locate` returns beside the page column on `345a9eb`
/// (release), read out of one run in MC-068's RED on a scratch copy with the
/// move applied - not chosen. The stage locates a viewport on all 21.
///
/// **MC-069 adds its 14**, the screenshots the user's run of the app
/// answered `Ambiguous` on, last in manifest order. Same provenance: what
/// `locate` returns beside the pipeline's page column on `afeaf3b`
/// (release), read out of one run in MC-069's RED with the 14 added - not
/// chosen. The stage locates a viewport on all 14; MC-069 does not touch it.
///
/// **MC-072 adds one of its four**, the fresh entries MC-071 read per file,
/// moved to `tuning` by the user's answer of 2026-10-02 ("all recommended"),
/// last in manifest order: `2025-11-01 12_34_31.png` (`n05`), 167..1400,
/// beside the page column `1007,40 531x1400`. Section 4 never saw it. Same
/// provenance: what `locate` returns beside the page column on `0f9c579`
/// (release), read out of one run in MC-072's RED on a scratch copy with only
/// the four `split` values changed - not chosen. On the other three the stage
/// declines: [`STAGE_DECLINED`].
///
/// **MC-074 adds `2025-03-07 00_05_58.png` (`n02`)**, off [`STAGE_DECLINED`],
/// in manifest order before `n05`: 115..1399. **Settled, not measured or
/// chosen**: MC-071's frozen oracle for `n02` (`T` 115, `B` 1399), which is
/// what the Lead PO measured `locate` to return beside the right page column
/// `643,0 533x1440` on `d2ad8ad` (MC-074 `## Context`). Its drawn panels sit
/// on flat white page paper the user ruled is page (*"Box stands"*,
/// 2026-10-02); once the page column spans the paper, the stage locates the
/// viewport beside it (MC-074 AC-1).
///
/// **MC-075 adds `2025-03-13 12_01_01.png` (`n06`)**, off [`STAGE_DECLINED`],
/// in manifest order between `n02` and `n05`: 115..1392. **Settled, not
/// measured or chosen**: MC-071's frozen oracle for `n06` (`T` 115, `B` 1392,
/// the scrollbar start the user ruled), which MC-075 AC-1 requires the crop's
/// rows to lie within, and which the Lead PO's scratch trial measured `locate`
/// returning beside the page column `698,0 400x1440` (MC-075 `## Context`,
/// trial rule 3). Beside a second window textured on every row the
/// whole-margin share is 0.648 and the stage declined; after MC-075 it reads
/// the reader's window.
const STAGE_MEASURED: [(&str, u32, u32); 64] = [
    ("2025-03-04 11_09_29.png", 115, 1400),
    ("2025-03-07 00_41_10.png", 115, 1399),
    ("2025-03-07 01_10_37.png", 115, 1399),
    ("2025-07-17 14_20_23.png", 167, 1400),
    ("2025-07-17 14_41_58.png", 167, 1400),
    ("2025-07-17 14_55_10.png", 167, 1400),
    ("2025-08-03 11_13_19.png", 115, 1400),
    ("2025-08-03 11_27_49.png", 115, 1400),
    ("2025-08-03 20_54_19.png", 115, 1400),
    ("2025-08-04 23_24_37.png", 115, 1400),
    ("2025-08-07 15_07_56.png", 115, 1400),
    ("2025-08-07 15_47_10.png", 124, 1400),
    ("2025-08-07 15_57_50.png", 115, 1400),
    ("2025-09-29 14_33_15.png", 167, 1400),
    ("Screenshot (56).png", 167, 1392),
    ("Screenshot (59).png", 167, 1392),
    ("Screenshot (68).png", 167, 1392),
    ("Screenshot (73).png", 167, 1392),
    ("Screenshot (1720).png", 133, 1392),
    ("Screenshot (3605).png", 137, 1392),
    ("Screenshot (3606).png", 137, 1392),
    ("Screenshot (3625).png", 137, 1392),
    // MC-064's four and MC-068's 21, interleaved in manifest order (see the doc comment).
    ("2025-03-04 14_28_54.png", 115, 1400),
    ("2025-03-06 02_01_06.png", 115, 1374),
    ("2025-03-06 12_48_06.png", 115, 1399),
    ("2025-03-07 16_07_21.png", 115, 1392),
    ("2025-03-16 22_47_44.png", 115, 1392),
    ("2025-03-18 12_37_27.png", 115, 1392),
    ("2025-03-23 23_56_16.png", 115, 1400),
    ("2025-03-24 22_44_31.png", 115, 1400),
    ("2025-03-25 22_06_29.png", 115, 1400),
    ("2025-07-17 23_45_48.png", 167, 1400),
    ("2025-07-21 08_26_37.png", 115, 1400),
    ("2025-07-21 17_47_22.png", 115, 1400),
    ("2025-08-04 17_10_16.png", 115, 1400),
    ("2025-08-07 01_13_55.png", 115, 1400),
    ("2025-08-07 11_20_12.png", 115, 1400),
    ("2025-08-07 14_33_43.png", 115, 1400),
    ("2025-10-23 11_31_40.png", 167, 1400),
    ("2025-11-12 17_43_44.png", 167, 1400),
    ("2025-12-08 17_22_50.png", 167, 1400),
    ("2025-12-09 00_00_17.png", 167, 1400),
    ("Screenshot (9).png", 167, 1392),
    ("Screenshot (2368).png", 133, 1392),
    ("Screenshot (2461).png", 133, 1392),
    ("Screenshot (2486).png", 133, 1392),
    ("Screenshot (2669).png", 133, 1392),
    // MC-069's 14, last in manifest order: what `locate` returns beside the
    // pipeline's page column on `afeaf3b` (release), read out of one run in
    // MC-069's RED with its entries added - not chosen. `(2705)`, which
    // `decide` flags `Ambiguous`, still has a viewport.
    ("Screenshot (14).png", 167, 1392),
    ("Screenshot (19).png", 167, 1392),
    ("Screenshot (20).png", 167, 1392),
    ("Screenshot (23).png", 167, 1392),
    ("Screenshot (42).png", 167, 1392),
    ("Screenshot (48).png", 167, 1392),
    ("Screenshot (49).png", 167, 1392),
    ("Screenshot (50).png", 167, 1392),
    ("Screenshot (51).png", 167, 1392),
    ("Screenshot (52).png", 167, 1392),
    ("Screenshot (53).png", 167, 1392),
    ("Screenshot (57).png", 167, 1392),
    ("Screenshot (58).png", 167, 1392),
    ("Screenshot (2705).png", 133, 1392),
    // MC-074: `n02`, before `n05` in manifest order (see the doc comment).
    ("2025-03-07 00_05_58.png", 115, 1399),
    // MC-075: `n06`, between `n02` and `n05` in manifest order (see the doc comment).
    ("2025-03-13 12_01_01.png", 115, 1392),
    // MC-072: `n05`, last in manifest order (see the doc comment).
    ("2025-11-01 12_34_31.png", 167, 1400),
];

/// MC-064: the marked `tuning` entry on which the stage, handed the
/// pipeline's page column, **declines** (`locate` returns `None`), measured
/// the same way as [`STAGE_MEASURED`]'s MC-064 rows: `2025-03-06
/// 12_48_06.png`, one of the four fresh entries MC-063 read per file, whose
/// page column is `1828,0 717x1440`, beside the art and the whole image
/// height. Section 4 never saw it, so it is not one of [`DECLINES`], which
/// claims to be section 5e's. **Exact**: this readout fails if the stage
/// locates a viewport on it, and fails if it is not a marked `tuning` entry.
///
/// MC-066 took it off (its AC-1, AC-3): [`STAGE_MEASURED`] holds the rows the
/// stage must locate beside its page column now. The list was empty, and
/// stayed exact.
///
/// **MC-072 adds three of its four**, in manifest order: `2025-03-07
/// 00_05_58.png` (`n02`, page column `703,0 398x1440`), `2025-03-13
/// 12_01_01.png` (`n06`, `698,0 400x1440`) and `Screenshot (2507).png`
/// (`n13`, `977,40 600x1352`). Beside each, `locate` returns `None`, measured
/// on `0f9c579` (release) in MC-072's RED on a scratch copy with only the four
/// `split` values changed; they are MC-071's three declines. Section 4 never
/// saw them, so they are not [`DECLINES`] either.
///
/// MC-074 took `n02` off (its AC-1): with its page column spanning the white
/// page paper the user ruled is page (*"Box stands"*, 2026-10-02), the stage
/// locates 115..1399 beside it, and [`STAGE_MEASURED`] holds those rows.
///
/// MC-075 took `n06` off (its AC-1): beside a second window textured on every
/// row the stage reads the reader's window, and [`STAGE_MEASURED`] holds
/// 115..1392, MC-071's frozen `T` and `B`. `n13` stays until MC-076, which
/// owns its different cause (reader panels of another tone, share 0.889).
const STAGE_DECLINED: [&str; 1] = ["Screenshot (2507).png"];

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
    let mut skipped = Vec::new();
    let mut stage_seen = Vec::new();
    let mut stage_moved = Vec::new();
    let mut declined_seen = Vec::new();

    for entry in marked() {
        let name = entry.name();
        let img = luma(&entry.path);
        let column = page_column_of(&img, &t);
        let got = locate(&img, column, &t);

        if NOT_IN_SECTION_4.contains(&name.as_str()) {
            skipped.push(name.clone());
            rows.push(format!(
                "{name:<26} not in section 4 (MC-052's; see corpus_viewport.rs)"
            ));
            continue;
        }
        if let Some(&(_, top, bottom)) = STAGE_MEASURED.iter().find(|(file, _, _)| *file == name) {
            let expected = Viewport { top, bottom };
            if got.as_ref() != Some(&expected) {
                stage_moved.push(format!("{name}: expected {expected:?}, got {got:?}"));
            }
            stage_seen.push(name.clone());
            rows.push(format!(
                "{name:<26} measured by the stage {top}..{bottom}  got {got:?} (MC-053's)"
            ));
            continue;
        }
        if STAGE_DECLINED.contains(&name.as_str()) {
            if got.is_some() {
                stage_moved.push(format!("{name}: expected None (declines), got {got:?}"));
            }
            declined_seen.push(name.clone());
            rows.push(format!(
                "{name:<26} measured by the stage: declines  got {got:?} (MC-064's)"
            ));
            continue;
        }
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
    assert_eq!(
        skipped,
        NOT_IN_SECTION_4.map(String::from).to_vec(),
        "MC-052's two must be marked tuning entries, skipped here and nowhere else"
    );
    assert_eq!(
        stage_seen,
        STAGE_MEASURED.map(|(name, _, _)| name.to_string()).to_vec(),
        "MC-053's three, MC-056's three, MC-062's sixteen, MC-064's four, MC-068's \
         21 and MC-069's 14 must be marked tuning entries, reached here in manifest order"
    );
    assert_eq!(
        declined_seen,
        STAGE_DECLINED.map(String::from).to_vec(),
        "MC-064's STAGE_DECLINED must be marked tuning entries, reached here in \
         manifest order"
    );
    assert!(
        stage_moved.is_empty(),
        "MC-053 / MC-056 / MC-062 / MC-064 / MC-066 / MC-068: on its forty-seven the stage \
         must keep the rows it located on c004d96 (MC-053's), c3fee28 (MC-056's), 7c36b5d \
         (MC-062's), 43e8e61 (MC-064's) and 345a9eb (MC-068's), and locate the rows MC-066's criteria force \
         on f18 and f13 (STAGE_MEASURED), and must still decline where it declined \
         (STAGE_DECLINED).\n{}\n\n{printed}",
        stage_moved.join("\n")
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
