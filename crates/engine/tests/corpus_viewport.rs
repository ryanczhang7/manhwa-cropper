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
    let output = tmp.path().join(entry.name());
    match process_file(&entry.path, &output, &Tuning::default()).outcome {
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
/// marked `tuning` entries other than [`WEBPS`], in manifest order. A name
/// that drifted would turn a 19-of-19 claim into a claim about fewer files
/// without anything going red.
fn viewport_entries() -> Vec<(CorpusEntry, Rect, u32, u32)> {
    let marked = marked();
    let names: Vec<String> = marked
        .iter()
        .map(|(entry, _)| entry.name())
        .filter(|name| !WEBPS.contains(&name.as_str()))
        .collect();
    assert_eq!(
        names,
        VIEWPORT.map(|(name, _, _)| name.to_string()).to_vec(),
        "VIEWPORT must list exactly the marked tuning entries other than the two \
         WebPs, in manifest order"
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
