//! MC-048, AC-1 to AC-3 and AC-8: on the corpus, the crop removes the browser
//! chrome and the taskbar, and leaves the columns where they were.
//!
//! Every test here goes through the real pipeline - [`process_file`] at
//! `Tuning::default()` - and reads nothing but the rect it returns, so this
//! file compiles against the tree before MC-048 as well as after it. The
//! stage's own located rows are asked of the stage directly in
//! `tests/corpus_viewport_stage.rs`, which is a separate target on purpose:
//! it imports the stage, and until the stage exists that target does not
//! compile. Keeping it apart is what lets these tests run - and fail on their
//! assertions - in RED.
//!
//! # What is settled, what is mechanical
//!
//! * **Settled, read out and never re-derived here**: each entry's first
//!   viewport row (`chromeEnd`) and first taskbar row, from
//!   `docs/wiki/chrome-row-search.md` section 4 ([`VIEWPORT`]); the
//!   containment predicate.
//! * **Mechanical**: AC-2's predicate, `chromeEnd <= r < taskbar` for every
//!   crop row `r = y .. y + h - 1`, on 19 of 19; AC-3's two WebP rects
//!   ([`WEBPS_BEFORE`]) and AC-8's `(x, w)` per file ([`COLUMNS_BEFORE`]),
//!   both pinned from the RED measurement on `cb2deef`.
//!
//! # MC-052: split-screen screenshots
//!
//! MC-052 added two marked `tuning` entries, both split-screen shots with a
//! second browser window beside the reader. They are **not** section 4's and
//! are kept out of every MC-048 constant above; [`READER_WINDOW`] holds their
//! reader-window rows, measured by MC-052. The tests at the end of this file
//! run every MC-052 criterion at both margins: AC-1 (the two keep their art on
//! the row axis), AC-2 and its decline control, AC-3 ([`ORIGINALS_BEFORE`],
//! the 21 unchanged), AC-4 (0 clips over all 23) and AC-5's column pins
//! ([`READER_WINDOW_COLUMNS`]).
//!
//! # Running it
//!
//! `#[ignore]`d like every corpus suite, so the `unit` gate never runs it and
//! the `integration` gate does, in release:
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_viewport -- --ignored --nocapture
//! ```
//!
//! Every assertion message carries its per-file table, because the gate does
//! not pass `--nocapture`.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};

// --- Constants read out of the story, never calibrated here -----------------

/// `chrome-row-search.md` section 4, the "perfect chrome oracle" table, read
/// out per file: `(file, chromeEnd, taskbar)`, where `chromeEnd` is the first
/// row of the browser viewport and `taskbar` the first row of the Windows
/// taskbar. The nineteen marked `tuning` entries for which the oracle speaks,
/// in manifest order. The two `2025-08-05` WebPs are absent because it
/// declines on them (section 5e); they are [`WEBPS`].
const VIEWPORT: [(&str, u32, u32); 19] = [
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

/// AC-3's two entries: the marked `tuning` entries on which the oracle
/// declines at its full-margin setting.
const WEBPS: [&str; 2] = ["2025-08-05 00_11_13.webp", "2025-08-05 00_11_27.webp"];

/// AC-3, as amended on 2026-09-23: each WebP's whole crop as `process_file`
/// produces it at `Tuning::default()` on `cb2deef`, measured in RED (release)
/// and pinned. The stage declines on both and the pipeline must fall back to
/// exactly this; chrome removal on them is a named limitation.
const WEBPS_BEFORE: [(&str, Rect); 2] = [
    (
        "2025-08-05 00_11_13.webp",
        Rect {
            x: 950,
            y: 15,
            w: 646,
            h: 1425,
        },
    ),
    (
        "2025-08-05 00_11_27.webp",
        Rect {
            x: 1003,
            y: 15,
            w: 539,
            h: 1425,
        },
    ),
];

/// AC-8: every marked `tuning` entry's crop `(file, x, w)` as `process_file`
/// produces it at `Tuning::default()` on `cb2deef`, measured in RED (release)
/// and pinned. The side edges are MC-049's; this story must not move them.
const COLUMNS_BEFORE: [(&str, u32, u32); 21] = [
    ("2025-08-05 00_11_13.webp", 950, 646),
    ("2025-08-05 00_11_27.webp", 1003, 539),
    ("2025-10-14 23_29_06.png", 1000, 546),
    ("2025-10-14 23_30_20.png", 1030, 486),
    ("2025-10-20 15_37_25.png", 1004, 538),
    ("2026-01-05 13_33_41.png", 1071, 404),
    ("2026-01-05 13_45_59.png", 1036, 473),
    ("2026-01-05 13_49_39.png", 1036, 473),
    ("Screenshot (67).png", 1007, 531),
    ("Screenshot (70).jpg", 1007, 531),
    ("Screenshot (75).png", 1070, 406),
    ("Screenshot (93).jpg", 1136, 273),
    ("Screenshot (103).jpg", 1070, 406),
    ("Screenshot (1661).png", 1070, 406),
    ("Screenshot (2582).jpg", 1003, 539),
    ("Screenshot (2630).jpg", 952, 642),
    ("Screenshot (2698).jpg", 950, 645),
    ("Screenshot (2708).jpg", 1071, 405),
    ("Screenshot (2744).jpg", 945, 654),
    ("Screenshot (3187).png", 981, 582),
    ("Screenshot (3538).png", 972, 602),
];

// --- MC-052: the two split-screen screenshots -------------------------------

/// MC-052 AC-2: the two split-screen screenshots the user moved from
/// `held-out` to `tuning` on 2026-09-24, with the **reader window's own**
/// viewport, `(file, first viewport row, first row below it)`.
///
/// **Measured by MC-052, not read out of `chrome-row-search.md` section 4**,
/// which never saw these files. That is why they are a separate constant and
/// never rows of [`VIEWPORT`], which claims to be section 4. The Lead PO's
/// probe gave 115..1374 and 115..1399 from its own tone estimate; MC-052's RED
/// re-measured both independently, on the reader's side of the window edge
/// only (x < 1811, where the reader's scrollbar starts; the second window
/// starts at x 1828):
///
/// * the per-row mean and median of the reader's margins, x 0..600 and
///   1200..1800 - page background is median 11; the reader's bookmark bar is
///   median 48 (rows 104..114 on `01_22_45`, row 114 on `00_58_06`); the row
///   under the reader window is a border at 112 / 108; the strip below is
///   195..244 on `01_22_45`; the taskbar starts at row 1400;
/// * renders of rows 95..135 and 1355..1415 at 6x, read by eye.
///
/// Both agree with the probe to the row: `01_22_45` is page from 115 through
/// 1373 (1374 is the window's bottom border, 1375..1399 the desktop below
/// it); `00_58_06` is page from 115 through 1398 (1399 is the border).
const READER_WINDOW: [(&str, u32, u32); 2] = [
    ("2025-03-06 01_22_45.png", 115, 1374),
    ("2025-03-07 00_58_06.png", 115, 1399),
];

/// A crop's columns, `(x, w)`.
type Columns = (u32, u32);

/// MC-052 AC-5: the columns of the two, `(file, x, w at margin 3, x, w at
/// margin 0)`, as `process_file` produces them on `26eddcb` - measured in RED
/// and pinned. The columns are MC-053's and MC-049's; this story moves rows.
const READER_WINDOW_COLUMNS: [(&str, Columns, Columns); 2] = [
    ("2025-03-06 01_22_45.png", (640, 539), (643, 533)),
    ("2025-03-07 00_58_06.png", (667, 486), (670, 480)),
];

/// MC-052 AC-3: the whole crop of each of the 21 marked `tuning` entries
/// MC-052 did not add, `(file, [x, y, w, h] at margin 3, [x, y, w, h] at
/// margin 0)`, as `process_file` produces it on `26eddcb` (release) - the Lead
/// PO's probe values, re-measured in MC-052's RED and identical. A fix that
/// reads different pixels on a split-screen shot must not move a single-window
/// one by a row.
const ORIGINALS_BEFORE: [(&str, [u32; 4], [u32; 4]); 21] = [
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
];

/// Whether `name` is one of MC-052's two.
fn is_reader_window_entry(name: &str) -> bool {
    READER_WINDOW.iter().any(|(file, _, _)| *file == name)
}

// --- Harness ----------------------------------------------------------------

/// The marked `tuning` entries, with their marks, in manifest order. **Never
/// `held-out`**: `docs/wiki/corpus.md` rules that a held-out entry is scored
/// once, at the end of a v2 attempt, and this is not that attempt.
fn marked() -> Vec<(CorpusEntry, Rect)> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .filter_map(|entry| match entry.expect {
            Expect::Rect(rect) => Some((entry, rect)),
            Expect::Flag => None,
        })
        .collect()
}

/// The rect `process_file` crops `entry` to, or a description of what it did
/// instead. A marked page that is not cropped is a failure of every criterion
/// here, so it is reported rather than skipped.
fn crop(entry: &CorpusEntry, tmp: &tempfile::TempDir) -> Result<Rect, String> {
    crop_at(entry, tmp, &Tuning::default())
}

/// [`crop`] at `t`: MC-052 runs every criterion at both margins.
fn crop_at(entry: &CorpusEntry, tmp: &tempfile::TempDir, t: &Tuning) -> Result<Rect, String> {
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, t).outcome {
        Outcome::Cropped { rect, .. } => Ok(rect),
        Outcome::Flagged { reason, .. } => Err(format!("Flagged {reason:?}")),
        Outcome::Failed { error } => Err(format!("Failed {error}")),
    }
}

/// The image's height, from its header alone and with the format guessed from
/// the content rather than the extension.
fn height(entry: &CorpusEntry) -> u32 {
    image::ImageReader::open(&entry.path)
        .and_then(|reader| reader.with_guessed_format())
        .ok()
        .and_then(|reader| reader.into_dimensions().ok())
        .unwrap_or_else(|| panic!("reading the dimensions of {}", entry.name()))
        .1
}

/// AC-2's predicate, split into its two halves: whether some crop row lies
/// **above** `first_page_row` (the rect's first row `y` is above it), and
/// whether some crop row lies **at or below** `first_taskbar_row` (the rect's
/// last row `y + h - 1` is at or below it). `(false, false)` is "every crop
/// row `r` satisfies `first_page_row <= r < first_taskbar_row`".
fn outside_viewport(rect: Rect, first_page_row: u32, first_taskbar_row: u32) -> (bool, bool) {
    let last = rect.y + rect.h - 1;
    (rect.y < first_page_row, last >= first_taskbar_row)
}

/// Whether `outer` contains `inner` entirely: the settled containment
/// predicate, the definition of "not a clip".
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// Print a table under `--nocapture` and return it for an assertion message.
fn table(title: &str, header: &str, rows: &[String]) -> String {
    let mut out = format!("{title}\n{header}\n");
    for row in rows {
        out.push_str(row);
        out.push('\n');
    }
    println!("{out}");
    out
}

/// The premise every test below leans on: [`VIEWPORT`] names exactly the
/// marked `tuning` entries other than [`WEBPS`] and MC-052's [`READER_WINDOW`],
/// in manifest order. A name
/// that drifted would turn a 19-of-19 claim into a claim about fewer files
/// without anything going red.
fn viewport_entries() -> Vec<(CorpusEntry, Rect, u32, u32)> {
    let marked = marked();
    let names: Vec<String> = marked
        .iter()
        .map(|(entry, _)| entry.name())
        .filter(|name| !WEBPS.contains(&name.as_str()))
        .filter(|name| !is_reader_window_entry(name))
        .collect();
    assert_eq!(
        names,
        VIEWPORT.map(|(name, _, _)| name.to_string()).to_vec(),
        "VIEWPORT must list exactly the marked tuning entries other than the two \
         WebPs and MC-052's two split-screen shots (READER_WINDOW), in manifest order"
    );
    marked
        .into_iter()
        .filter_map(|(entry, mark)| {
            VIEWPORT
                .iter()
                .find(|(name, _, _)| *name == entry.name())
                .map(|&(_, top, taskbar)| (entry, mark, top, taskbar))
        })
        .collect()
}

// --- AC-1 and AC-2: no crop row in the browser chrome or the taskbar --------

/// AC-2, and AC-1 as the bug it reproduces.
///
/// On the nineteen entries for which `chrome-row-search.md` section 4 locates
/// the viewport, every crop row must lie in `[chromeEnd, taskbar)`. On `main`
/// at `cb2deef` the crop starts at y = 15..37 - inside the tab strip - and
/// ends at row 1439, inside the taskbar, so both halves fail on every entry;
/// the message counts the two halves separately.
///
/// Its control on the metric is the next test: the same predicate, with the
/// viewport widened to the whole image, over the same crops.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_crop_row_lies_in_the_browser_chrome_or_the_taskbar_where_the_viewport_was_located() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let (mut above, mut below, mut failed) = (Vec::new(), Vec::new(), Vec::new());

    let entries = viewport_entries();
    for (entry, _, top, taskbar) in &entries {
        match crop(entry, &tmp) {
            Ok(rect) => {
                let (up, down) = outside_viewport(rect, *top, *taskbar);
                if up {
                    above.push(entry.name());
                }
                if down {
                    below.push(entry.name());
                }
                rows.push(format!(
                    "{:<26} {:>5} {:>5} {:>6} {:>6} {:>4} {:>4}",
                    entry.name(),
                    top,
                    taskbar,
                    rect.y,
                    rect.y + rect.h - 1,
                    if up { "TOP" } else { "ok" },
                    if down { "BOT" } else { "ok" },
                ));
            }
            Err(what) => {
                failed.push(format!("{}: {what}", entry.name()));
                rows.push(format!(
                    "{:<26} {:>5} {:>5} {what}",
                    entry.name(),
                    top,
                    taskbar
                ));
            }
        }
    }

    let printed = table(
        "AC-2: crop rows against the viewport located in chrome-row-search.md section 4",
        &format!(
            "{:<26} {:>5} {:>5} {:>6} {:>6} {:>4} {:>4}",
            "file", "chrEnd", "tbar", "first", "last", "top", "bot"
        ),
        &rows,
    );

    let bad: std::collections::BTreeSet<&String> = above.iter().chain(&below).collect();
    assert!(
        failed.is_empty() && bad.is_empty(),
        "AC-2: every crop row must lie inside the browser viewport, below the \
         browser chrome and above the taskbar. {} of {} entries break it: {} start \
         above chromeEnd (browser chrome kept), {} end at or below the taskbar row \
         (taskbar kept), {} were not cropped at all.\n\n{printed}\nnot cropped: {failed:?}",
        bad.len() + failed.len(),
        entries.len(),
        above.len(),
        below.len(),
        failed.len(),
    );
}

/// AC-2's control on the metric, which runs - and must pass - before and after
/// this story alike.
///
/// The same predicate over the same crops, with each entry's `chromeEnd`
/// replaced by 0 and its `taskbar` by the image height. Every crop lies inside
/// its own image, so this must hold on 19 of 19. Beside the test above, which
/// fails on today's crops, it shows the predicate is reading the viewport rows
/// and not something else about the rect.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_viewport_predicate_holds_on_every_crop_when_the_viewport_is_the_whole_image() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut broken = Vec::new();

    let entries = viewport_entries();
    for (entry, _, _, _) in &entries {
        let whole = height(entry);
        match crop(entry, &tmp) {
            Ok(rect) => {
                let verdict = outside_viewport(rect, 0, whole);
                if verdict != (false, false) {
                    broken.push(entry.name());
                }
                rows.push(format!(
                    "{:<26} 0..{:<5} {:>6} {:>6} {verdict:?}",
                    entry.name(),
                    whole,
                    rect.y,
                    rect.y + rect.h - 1
                ));
            }
            Err(what) => {
                broken.push(entry.name());
                rows.push(format!("{:<26} {what}", entry.name()));
            }
        }
    }

    let printed = table(
        "AC-2 control: the same predicate with the viewport widened to the whole image",
        &format!(
            "{:<26} {:<8} {:>6} {:>6} (above, below)",
            "file", "rows", "first", "last"
        ),
        &rows,
    );
    assert_eq!(
        entries.len(),
        VIEWPORT.len(),
        "the control must see all nineteen"
    );
    assert!(
        broken.is_empty(),
        "AC-2's control: with chromeEnd = 0 and taskbar = the image height the \
         predicate must hold on every crop, or it is not reading the viewport \
         rows at all. It fails on {broken:?}.\n\n{printed}"
    );
}

// --- AC-3: the two WebPs ----------------------------------------------------

/// AC-3, as amended on 2026-09-23. On both `2025-08-05` WebPs, where the
/// oracle declines at its full-margin setting, the crop is **identical** -
/// `x`, `y`, `w` and `h` - to the one made on `cb2deef` ([`WEBPS_BEFORE`]), and
/// neither crop clips its mark. Chrome removal on them is a named limitation.
///
/// Green on arrival by construction, since the rects were measured off this
/// tree, and earned by a probe recorded in the story's `## Handoff`: one pinned
/// value changed by one, watched to fail naming that file, reverted.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn both_webp_crops_are_exactly_as_before_and_keep_their_marks() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut wrong = Vec::new();
    let mut seen = Vec::new();

    for (entry, mark) in marked() {
        if !WEBPS.contains(&entry.name().as_str()) {
            continue;
        }
        seen.push(entry.name());
        let pinned = WEBPS_BEFORE
            .iter()
            .find(|(name, _)| *name == entry.name())
            .map(|&(_, rect)| rect);
        match crop(&entry, &tmp) {
            Ok(rect) => {
                let moved = pinned != Some(rect);
                let clips = !contains(rect, mark);
                if moved {
                    wrong.push(format!(
                        "{}: cropped to {rect:?}, pinned {pinned:?}",
                        entry.name()
                    ));
                }
                if clips {
                    wrong.push(format!("{}: the crop clips the mark", entry.name()));
                }
                rows.push(format!(
                    "{:<26} crop {},{} {}x{}  mark {},{} {}x{}  moved {moved}  clips {clips}",
                    entry.name(),
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    mark.x,
                    mark.y,
                    mark.w,
                    mark.h
                ));
            }
            Err(what) => {
                wrong.push(format!("{}: {what}", entry.name()));
                rows.push(format!("{:<26} {what}", entry.name()));
            }
        }
    }

    let printed = table("AC-3: the two WebPs", "file / crop / mark", &rows);
    assert_eq!(
        seen,
        WEBPS.map(String::from).to_vec(),
        "AC-3 must reach both WebPs, in manifest order"
    );
    assert!(
        wrong.is_empty(),
        "AC-3: on the WebPs the viewport stage declines, and the crop must be \
         exactly the one made on cb2deef and still contain the mark.\n{}\n\n{printed}",
        wrong.join("\n")
    );
}

// --- AC-8: the columns do not move ------------------------------------------

/// AC-8. Every marked `tuning` entry's crop keeps exactly the `x` and `w` it
/// had on `cb2deef` ([`COLUMNS_BEFORE`]). The side edges are MC-049's, and
/// this story is not allowed to move them as a side effect.
///
/// Green on arrival by construction, since the numbers were measured off this
/// tree, and earned by a probe recorded in the story's `## Handoff`: one pinned
/// value changed by one, watched to fail naming that file, reverted.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn every_marked_crop_keeps_the_columns_it_had_before_the_viewport_stage() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut moved = Vec::new();
    let mut seen = Vec::new();

    for (entry, _) in marked() {
        // MC-052: the two it added are pinned by READER_WINDOW_COLUMNS, not
        // here; this pin is MC-048's, over the twenty-one it measured.
        if is_reader_window_entry(&entry.name()) {
            continue;
        }
        seen.push(entry.name());
        let pinned = COLUMNS_BEFORE
            .iter()
            .find(|(name, _, _)| *name == entry.name())
            .map(|&(_, x, w)| (x, w));
        match crop(&entry, &tmp) {
            Ok(rect) => {
                if pinned != Some((rect.x, rect.w)) {
                    moved.push(format!(
                        "{}: x {} w {}, pinned {pinned:?}",
                        entry.name(),
                        rect.x,
                        rect.w
                    ));
                }
                rows.push(format!(
                    "{:<26} x {:>4} y {:>4} w {:>4} h {:>4}",
                    entry.name(),
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h
                ));
            }
            Err(what) => {
                moved.push(format!("{}: {what}", entry.name()));
                rows.push(format!("{:<26} {what}", entry.name()));
            }
        }
    }

    let printed = table("AC-8: every marked crop, whole", "file / rect", &rows);
    assert_eq!(
        seen,
        COLUMNS_BEFORE.map(|(name, _, _)| name.to_string()).to_vec(),
        "AC-8 must pin all twenty-one marked tuning entries, in manifest order"
    );
    assert!(
        moved.is_empty(),
        "AC-8: the crop's x and w must be exactly what they were on cb2deef; the \
         side edges are MC-049's. Moved:\n{}\n\n{printed}",
        moved.join("\n")
    );
}

// --- MC-052: the split-screen shots, and nothing else moves -----------------

/// Both margins, as MC-052 defines them: `Tuning::default()` and
/// `margin_px: 0`.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning::default(),
        Tuning {
            margin_px: 0,
            ..Tuning::default()
        },
    ]
}

/// The two, with their marks and their reader-window rows, in
/// [`READER_WINDOW`]'s order. Panics unless each is a marked `tuning` entry:
/// a name that drifted would leave every test below checking nothing.
fn reader_window_entries() -> Vec<(CorpusEntry, Rect, u32, u32)> {
    let marked = marked();
    READER_WINDOW
        .iter()
        .map(|&(name, top, bottom)| {
            let (entry, mark) = marked
                .iter()
                .find(|(entry, _)| entry.name() == name)
                .unwrap_or_else(|| panic!("{name} must be a marked `tuning` entry (MC-052)"));
            (entry.clone(), *mark, top, bottom)
        })
        .collect()
}

/// The rows `rect` cuts off `mark` at the top and at the bottom: how many of
/// the mark's rows lie above the crop's first row and below its last. `(0, 0)`
/// is "contains the mark on the top and on the bottom".
fn row_cuts(rect: Rect, mark: Rect) -> (u32, u32) {
    (
        rect.y.saturating_sub(mark.y),
        (mark.y + mark.h).saturating_sub(rect.y + rect.h),
    )
}

/// MC-052 AC-1, the bug reproduced. On the two split-screen screenshots, at
/// both margins, the crop contains its mark on the top and on the bottom.
///
/// On `26eddcb` the stage reads the second window's viewport, rows
/// 121..1259, and the crop cuts `01_22_45` by 3 rows at the top and 103 at
/// the bottom, and `00_58_06` by 31 at the bottom - at both margins.
/// `corpus.rs::no_crop_clips_a_marked_page` and
/// `corpus_accuracy.rs::no_corpus_crop_cuts_into_the_artwork_its_manifest_entry_marked`
/// see the same cuts at margin 3; this is the margin-0 half, and the named
/// reproduction.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn on_both_split_screen_shots_the_crop_keeps_the_top_and_bottom_of_the_art_at_both_margins() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut cut = Vec::new();
    for t in both_margins() {
        for (entry, mark, _, _) in reader_window_entries() {
            let row = match crop_at(&entry, &tmp, &t) {
                Ok(rect) => {
                    let (top, bottom) = row_cuts(rect, mark);
                    if (top, bottom) != (0, 0) {
                        cut.push(format!(
                            "{} at margin_px {}: cuts {top} rows of art at the top and \
                             {bottom} at the bottom",
                            entry.name(),
                            t.margin_px
                        ));
                    }
                    format!(
                        "{:<26} m{} crop rows {}..={}  mark rows {}..={}  cut top {top} bottom {bottom}",
                        entry.name(),
                        t.margin_px,
                        rect.y,
                        rect.y + rect.h - 1,
                        mark.y,
                        mark.y + mark.h - 1
                    )
                }
                Err(what) => {
                    cut.push(format!(
                        "{} at margin_px {}: {what}",
                        entry.name(),
                        t.margin_px
                    ));
                    format!("{:<26} m{} {what}", entry.name(), t.margin_px)
                }
            };
            rows.push(row);
        }
    }
    let printed = table(
        "MC-052 AC-1: the two split-screen shots",
        "file / rows",
        &rows,
    );
    assert!(
        cut.is_empty(),
        "MC-052 AC-1: on a split-screen screenshot the crop must keep the reader's \
         art whole on the row axis - the second window's chrome and bottom panel \
         are not the reader's.\n{}\n\n{printed}",
        cut.join("\n")
    );
}

/// MC-052 AC-4: zero clips over every marked `tuning` entry - the 21 and the
/// two - at both margins. At margin 3 all four sides count. At margin 0 only
/// the top and the bottom do: the 21's column-axis clips there are MC-049's
/// seven mark errors, which AC-3 pins rather than this.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn no_marked_tuning_crop_clips_its_mark_at_either_margin() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let entries = marked();
    let mut clips = Vec::new();
    for t in both_margins() {
        for (entry, mark) in &entries {
            match crop_at(entry, &tmp, &t) {
                Ok(rect) => {
                    let (top, bottom) = row_cuts(rect, *mark);
                    let columns_count = t.margin_px != 0;
                    let side_clip =
                        columns_count && (rect.x > mark.x || rect.x + rect.w < mark.x + mark.w);
                    if top > 0 || bottom > 0 || side_clip {
                        clips.push(format!(
                            "{:<26} m{} crop {},{} {}x{}  mark {},{} {}x{}  cut top {top} bottom {bottom}{}",
                            entry.name(),
                            t.margin_px,
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            mark.x,
                            mark.y,
                            mark.w,
                            mark.h,
                            if side_clip { "  and a side" } else { "" }
                        ));
                    }
                }
                Err(what) => clips.push(format!("{:<26} m{} {what}", entry.name(), t.margin_px)),
            }
        }
    }
    assert_eq!(
        entries.len(),
        23,
        "MC-052 AC-4 is over the 23 marked tuning entries"
    );
    assert!(
        clips.is_empty(),
        "MC-052 AC-4: 0 clips over the 23 marked tuning entries at both margins \
         (top and bottom only at margin 0). {} clip:\n{}",
        clips.len(),
        clips.join("\n")
    );
}

/// MC-052 AC-2: the reader's own chrome still goes. On the two, at both
/// margins, every crop row lies inside the **reader window's** viewport
/// ([`READER_WINDOW`]) - not the whole page column.
///
/// Holds on `26eddcb` (rows 121..1258). It is the guard against the cheapest
/// wrong fix, a stage that declines on a split-screen shot; the next test is
/// what that fix would leave.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn on_both_split_screen_shots_no_crop_row_lies_outside_the_reader_windows_own_viewport() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut outside = Vec::new();
    let mut rows = Vec::new();
    for t in both_margins() {
        for (entry, _, top, bottom) in reader_window_entries() {
            match crop_at(&entry, &tmp, &t) {
                Ok(rect) => {
                    let verdict = outside_viewport(rect, top, bottom);
                    if verdict != (false, false) {
                        outside.push(format!(
                            "{} at margin_px {}: crop rows {}..={} against the reader \
                             window's {top}..{bottom} (above, below) = {verdict:?}",
                            entry.name(),
                            t.margin_px,
                            rect.y,
                            rect.y + rect.h - 1
                        ));
                    }
                    rows.push(format!(
                        "{:<26} m{} crop rows {}..={}  reader window {top}..{bottom}  {verdict:?}",
                        entry.name(),
                        t.margin_px,
                        rect.y,
                        rect.y + rect.h - 1
                    ));
                }
                Err(what) => {
                    outside.push(format!(
                        "{} at margin_px {}: {what}",
                        entry.name(),
                        t.margin_px
                    ));
                    rows.push(format!("{:<26} m{} {what}", entry.name(), t.margin_px));
                }
            }
        }
    }
    let printed = table(
        "MC-052 AC-2: the reader window's own viewport",
        "file / rows",
        &rows,
    );
    assert!(
        outside.is_empty(),
        "MC-052 AC-2: the crop must keep none of the reader's own browser chrome, \
         nor the strip below the reader window, nor the taskbar.\n{}\n\n{printed}",
        outside.join("\n")
    );
}

/// MC-052 AC-2's control, the fix it exists to rule out. A stage that
/// **declined** on the two would leave the page column's rows, the whole image
/// height (the page column is `643,0 533x1440` and `670,0 480x1440`). Against
/// the reader window's viewport that keeps 115 rows above it on both - the
/// reader's tab strip and bookmark bar - and 66 rows and 41 rows below it -
/// the strip under the reader window and the taskbar. So AC-2's predicate
/// fires on that fix, by those depths.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn a_stage_that_declined_on_the_split_screen_shots_would_keep_the_readers_chrome() {
    let mut got = Vec::new();
    for (entry, _, top, bottom) in reader_window_entries() {
        let whole = height(&entry);
        let declined = Rect {
            x: 0,
            y: 0,
            w: 1,
            h: whole,
        };
        let verdict = outside_viewport(declined, top, bottom);
        got.push((entry.name(), verdict, top, whole - bottom));
    }
    assert_eq!(
        got,
        vec![
            ("2025-03-06 01_22_45.png".to_string(), (true, true), 115, 66),
            ("2025-03-07 00_58_06.png".to_string(), (true, true), 115, 41),
        ],
        "MC-052 AC-2's control: a decline keeps rows 0..height, which AC-2's \
         predicate must reject on both halves, by 115 rows at the top and 66 / 41 \
         at the bottom. `(file, (above, below), rows above, rows below)`"
    );
}

/// MC-052 AC-3: the 21 originals do not move. Each crop's `x`, `y`, `w` and
/// `h` at both margins are exactly `26eddcb`'s ([`ORIGINALS_BEFORE`]),
/// including the two WebPs, which still decline.
///
/// Green on arrival by construction, since the rects were measured off this
/// tree; earned by a probe recorded in MC-052's `## Handoff`.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_twenty_one_original_crops_do_not_move_by_a_pixel_at_either_margin() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut moved = Vec::new();
    let mut seen = Vec::new();
    for (entry, _) in marked() {
        let name = entry.name();
        if is_reader_window_entry(&name) {
            continue;
        }
        seen.push(name.clone());
        let pinned = ORIGINALS_BEFORE
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
                    "{name} at margin_px {}: {rect:?}, pinned {want:?}",
                    t.margin_px
                )),
                Err(what) => moved.push(format!("{name} at margin_px {}: {what}", t.margin_px)),
            }
        }
    }
    assert_eq!(
        seen,
        ORIGINALS_BEFORE
            .map(|(name, _, _)| name.to_string())
            .to_vec(),
        "MC-052 AC-3 must reach all 21 originals, in manifest order"
    );
    assert!(
        moved.is_empty(),
        "MC-052 AC-3: a fix for split-screen shots must leave every single-window \
         crop exactly as it was on 26eddcb, at both margins. Moved:\n{}",
        moved.join("\n")
    );
}

/// MC-052 AC-5: the columns of the two do not move - `x`, `w` are
/// [`READER_WINDOW_COLUMNS`] at both margins. The column axis is MC-053's and
/// MC-049's; this story moves rows.
#[test]
#[ignore = "integration: decodes the whole corpus"]
fn the_split_screen_shots_keep_their_columns_at_both_margins() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut got = Vec::new();
    for (entry, _, _, _) in reader_window_entries() {
        let mut at = Vec::new();
        for t in both_margins() {
            at.push(crop_at(&entry, &tmp, &t).map(|rect| (rect.x, rect.w)));
        }
        got.push((entry.name(), at));
    }
    let want: Vec<(String, Vec<Result<Columns, String>>)> = READER_WINDOW_COLUMNS
        .iter()
        .map(|&(name, m3, m0)| (name.to_string(), vec![Ok(m3), Ok(m0)]))
        .collect();
    assert_eq!(
        got, want,
        "MC-052 AC-5: the two's crop columns (x, w) at margin 3 and margin 0 must \
         be exactly 26eddcb's"
    );
}
