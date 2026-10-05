//! MC-019, AC-1 to AC-3: the detector meets the accuracy bar on the corpus
//! **column axis**.
//!
//! The product brief's section 7, expressed as a test, and narrowed by
//! MC-019's `## Amendments` to the axis the evidence supports. Every entry of
//! the calibration corpus (MC-018) goes through the real pipeline -
//! [`process_file`], `Tuning::default()`, no stubs and no shortcuts - and the
//! outcomes are measured against the rectangles a person drew by hand:
//!
//! * **AC-1, zero clips.** No cropped rect may cut into its expected rect, on
//!   any of the four sides. Unchanged by the amendment.
//! * **AC-2, nine in ten on the column axis.** At least 90% of entries are
//!   right, where an entry with a mark is right when its crop contains the
//!   mark (AC-1) and its **left and right** edges sit inside the mark's left
//!   and right grown by `margin_px + 8`. The row deltas are printed on every
//!   run and **not asserted**: they are MC-032's, openly, rather than silently
//!   dropped.
//! * **AC-3.** Nothing fails: every corpus file is readable and writable.
//!
//! This is a **characterisation** test. The detector already meets this bar on
//! this tree - 0 clips, 20 of 21 columns, 26 of 28 = 92.9% - so every
//! assertion here was green the first time it ran. MC-019's `## Regressions`
//! carries the probe that earned each one: a non-default `Tuning` passed to
//! the same real pipeline, watched to turn that one assertion red, reverted.
//!
//! # Why this is a new file and not more of `tests/corpus.rs`
//!
//! `tests/corpus.rs` is MC-026's and `tests/corpus_manifest.rs` is MC-018's,
//! both DONE and frozen. This target shares their loader (`common/corpus.rs`)
//! by the same `#[path]` include and nothing else.
//!
//! # Running it
//!
//! Every test here is `#[ignore]`d, so `cargo test --workspace` (the `unit`
//! gate, debug) never runs it and the `integration` gate does, in release -
//! and MC-019 makes that gate *required* through the story's
//! `required_gates`:
//!
//! ```text
//! cargo test --workspace --release -- --ignored
//! ```
//!
//! To run it alone, with the per-file table on stdout:
//!
//! ```text
//! cargo test -p cropper-engine --test corpus_accuracy -- --ignored --nocapture
//! ```
//!
//! **Release is not optional**, for the same reason `tests/perf.rs` says so:
//! decoding 21 MB of screenshots in an unoptimised build measures the profile.
//!
//! # The table has to survive capture
//!
//! AC-2 requires a per-file table, and the gate does not pass `--nocapture`,
//! so a `println!` would be swallowed on exactly the run whose output somebody
//! needs. Every assertion here therefore carries [`table`]'s output *in its
//! own message*. It is printed as well, for `--nocapture` runs; the printed
//! copy is a convenience and the message is the contract.
//!
//! # What is settled, what is mechanical, what is the oracle
//!
//! * **Settled**, read out of the acceptance criteria and never re-derived:
//!   the 90% bar ([`RIGHT_FRACTION`]), the `margin_px + 8` slack
//!   ([`SLACK_OVER_MARGIN`], 11 px at today's `margin_px` of 3), the
//!   containment predicate ([`clipped_sides`]), and AC-1's control at 1 px on
//!   the columns ([`CONTROL_GROW_PX`]).
//! * **Mechanical**: AC-3, and the arithmetic of the two counts.
//! * **The oracle is the corpus.** `manifest.json` is the answer key, and
//!   nothing here re-derives it. It can still be *wrong*: a reported clip may
//!   be a mis-marked rectangle rather than a detector fault, which is why AC-1
//!   names the file and the offending side rather than only counting. That
//!   finding goes to the user, who opens the image and decides; the manifest
//!   is never edited to make this test pass.
//!
//! # Why the slack is computed from `margin_px` rather than written as 11
//!
//! AC-2's band is `margin_px + 8`, and `margin_px` is a constant MC-019 is
//! allowed to change. Reading it from the `Tuning` the run was given means a
//! change moves the expectation instead of breaking the test - the same choice
//! `tests/perf.rs` made for the output dimensions.
//!
//! It is not a loophole, which is worth showing rather than asserting. Raising
//! `margin_px` by *k* grows the produced rect by *k* on every side and grows
//! the band by *k* at the same time: the left check `rect.x + slack >= e.x`
//! has `rect.x` fall by *k* and `slack` rise by *k*, and the right check
//! `rect.x + rect.w <= e.x + e.w + slack` has both sides rise by *k*. The
//! comparison is invariant. `margin_px` cannot be turned up to buy AC-2 - and
//! the AC-2 probe in `## Regressions` is the measurement of that: at
//! `margin_px: 20` the count collapses rather than improving, because the
//! detector's rect grows past the band's own growth at the image's edges.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus.rs` and `tests/corpus_manifest.rs` both give at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use std::fmt::Write as _;
use std::path::Path;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};
use tempfile::TempDir;

// --- The settled numbers ----------------------------------------------------

/// AC-2's bar, from the brief's section 7: nine in ten need no manual fix.
/// Settled - if this is wrong it goes back to the product owner, never down.
const RIGHT_FRACTION: f64 = 0.90;

/// AC-2's slack over `Tuning::margin_px`: a crop may be loose by the margin
/// the detector deliberately adds plus eight pixels, and no more. Settled. At
/// today's `margin_px` of 3 the band is 11 px on the left and on the right.
const SLACK_OVER_MARGIN: u32 = 8;

/// How far AC-1's negative control widens every expected rect on the **left
/// and right** before re-running the containment check.
///
/// One pixel, which is the criterion's own first rung, and it is not a guess:
/// MC-019's second `## Amendments` entry moved this control from the top edge
/// to the columns precisely because the top ladder is exhausted without ever
/// firing (1 px and the 4 px fallback both clip nothing; the first rung that
/// fires on the top is 98 px, which is MC-032's deficit and not AC-1's
/// tightness). On the columns the same 1 px fires on exactly one entry,
/// `2025-10-14 23_30_20.png`, whose crop sits flush with the marked right
/// edge; 2 px fires on two and 3 px on seven.
const CONTROL_GROW_PX: u32 = 1;

/// MC-018 AC-1's floor, read out here as a vacuity guard: a percentage over a
/// handful of entries would not mean anything.
const MIN_ENTRIES: usize = 20;

/// MC-062, the user's rulings of 2026-09-30 ("Skip named misses", then "Skip
/// all four"): the entries AC-2's 90 % bar leaves out, because each is a miss
/// already named as a known exception elsewhere in the corpus suites. In
/// manifest order.
///
/// * `2025-02-27 22_46_15.png`: a flag entry the app crops, excluded by name
///   since MC-026 (`corpus.rs`, `KNOWN_CROPPED_FLAGS`).
/// * `2025-03-03 11_00_13.png`, `2025-05-12 20_48_42.png`: flag entries the
///   app crops, moved from held-out by MC-062 and ruled known misses (MC-051:
///   the "Cropped 2" of its 7 held-out flag entries).
/// * `Screenshot (2705).png`: a marked entry `decide` flags `Ambiguous`,
///   MC-069's known exception (MC-070 fixes it). It was
///   `2025-07-17 14_20_23.png`, MC-056's (MC-051: `h09`, failed its bar),
///   until MC-069 cropped it; see the MC-069 paragraph below.
///
/// **Exact in both directions**: AC-2 fails if a listed entry is no longer in
/// the `tuning` corpus or is no longer a miss, so the story that fixes one has
/// to take it off this list, and it counts again. Everything else still
/// counts, `Screenshot (93).jpg` and `Screenshot (1720).png` included. Over the
/// 59 `tuning` entries after MC-062 the bar reads 53 of 55.
///
/// **MC-064 adds four**, by the user's ruling of 2026-09-30 (*"List them as
/// known"*): the fresh entries MC-063 read per file, moved to `tuning`. Three
/// clip ([`KNOWN_CLIPS`]) and `2025-03-16 22_47_44.png` keeps the browser
/// scrollbar (right loose by 1381 px beyond the band). Each is held to its
/// measured crop by [`MC064_CROPS`]. Over the 63 `tuning` entries after MC-064
/// the bar still reads 53 of 55.
///
/// **MC-065 takes one off** (its AC-2): `2025-08-07 01_13_55.png` (`f20`),
/// whose right edge it fixes. It counts toward the bar again. `f13` and `f18`
/// stay for MC-066, and `2025-12-08 17_22_50.png` (`f09`) stays for MC-067
/// (MC-065 `## Amendments`, 2026-10-01).
///
/// **MC-066 takes two off** (its AC-1 and AC-2): `f18` (`2025-03-06
/// 12_48_06.png`) and `f13` (`2025-03-16 22_47_44.png`), whose crops it keeps
/// inside the reader's window. They count toward the bar again.
///
/// **MC-067 takes one off** (its AC-1): `2025-12-08 17_22_50.png` (`f09`),
/// which keeps its art's near-black right edge. It counts toward the bar
/// again.
///
/// **MC-069 swaps one** (its AC-3 and AC-4): `2025-07-17 14_20_23.png` comes
/// off - its close call is the browser scrollbar, outside its crop, so it
/// crops (`962,167 621x1233` at margin 0, right 7 px past its mark, inside the
/// 8 px band) and counts toward the bar again - and `Screenshot (2705).png`
/// goes on, MC-069's named known exception, flagged `Ambiguous` until MC-070.
/// MC-069's other 13 count: after MC-069 each crops to its mark (`(48)` one
/// row taller), so over the 98 `tuning` entries the bar reads 92 of 94 there,
/// and 78 of 94 on `main` at `afeaf3b`, where the 13 and `14_20_23` are
/// flagged.
///
/// **MC-070 takes one off** (its AC-1): `Screenshot (2705).png` crops to its
/// mark, `1073,133 399x1259` at margin 0 (as amended by MC-070's
/// `## Amendments`), and counts toward the bar again:
/// 93 of 95 there. On `main` at `eb54767`, where it is still flagged, it
/// counts as a miss, 92 of 95 - still over the bar, so this file stays green
/// on `main`; `(2705)`'s crop is pinned red elsewhere (MC-070 `## Test plan`).
const KNOWN_MISSES: [&str; 3] = [
    "2025-02-27 22_46_15.png",
    "2025-03-03 11_00_13.png",
    "2025-05-12 20_48_42.png",
];

/// MC-064: the crop `process_file` makes at `Tuning::default()` (margin_px 0)
/// of each of the four fresh entries MC-063 read per file, `(file, [x, y, w,
/// h])`, in manifest order. **Measured, not chosen**: read out of one run on
/// `43e8e61` (release; crates unchanged since `d2876f5`) in MC-064's RED, on a
/// scratch copy with only the four `split` values changed. Both of this file's
/// MC-064 exceptions - [`KNOWN_CLIPS`] in AC-1 and the four in
/// [`KNOWN_MISSES`] in AC-2 - hold each entry to exactly this crop, so a fix
/// that moves one has to take it off both lists.
///
/// MC-065 took `f20` (`2025-08-07 01_13_55.png`) off this list and off both
/// of those (its AC-2). `f09` (`2025-12-08 17_22_50.png`) stays at its MC-064
/// pin for MC-067 (MC-065 `## Amendments`, 2026-10-01).
///
/// MC-066 took `f18` and `f13` off this list and off both of those (its AC-1
/// and AC-2).
///
/// MC-067 took `f09` off this list and off both of those (its AC-1); its pin
/// was `[1006, 167, 531, 1233]`. The list was empty, and stayed exact.
///
/// **MC-072 adds two** (the user's answer of 2026-10-02, *"all
/// recommended"*: list them by name as known problems): the two fresh entries
/// MC-071 read per file whose crop clips the mark, `n02` and `n05`, in
/// manifest order. **Measured, not chosen**: read out of one run on `0f9c579`
/// (release) in MC-072's RED, on a scratch copy with only the four `split`
/// values changed; they equal MC-071's recorded crops. They are held here for
/// [`KNOWN_CLIPS`] only: neither is in [`KNOWN_MISSES`], because AC-2's bar
/// holds with both counted (MC-072 `## Handoff`), so no test needs them there.
///
/// MC-073 took `n05` (`2025-11-01 12_34_31.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-2): the user re-marked it, ruling on 2026-10-02
/// *"Box starts at 1007"* - column 1006 is the dark seam beside the art, as
/// on `Screenshot (3605).png` and `2025-10-20 15_37_25.png`. Its mark is
/// `1007,167 531x1233`, which its crop at margin 0 already equals, so AC-1
/// judges it like every other entry. Its pin was `[1007, 167, 531, 1233]`.
///
/// MC-074 took `n02` (`2025-03-07 00_05_58.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-1): its drawn panels sit on flat white page paper,
/// which the user ruled on 2026-10-02 is page (*"Box stands"*), so its crop
/// at margin 0 is to equal its mark `643,115 533x1284` and AC-1 judges it like
/// every other entry. Its pin was `[703, 0, 398, 1440]`. The list was empty,
/// and stayed exact.
///
/// **MC-079 adds one** (its AC-3): `n05` of MC-077's fresh draw,
/// `2025-10-26 12_13_16.png`, the one entry MC-078 read per file, whose crop
/// clips its mark `973,167 600x1233` by one column on the left. **Measured,
/// not chosen**: read out of one run on `6343029` (release) in MC-079's RED,
/// on a scratch copy with only its `split` value changed; it equals MC-078's
/// recorded crop, columns 974..1573, rows 167..1400. It is held here for
/// [`KNOWN_CLIPS`] only: it is not in [`KNOWN_MISSES`], because AC-2's bar
/// holds with it counted (MC-079 `## Handoff`), so no test needs it there.
///
/// MC-080 took `n05` (`2025-10-26 12_13_16.png`) off this list and off
/// [`KNOWN_CLIPS`] (its AC-2): the user re-marked it, ruling on 2026-10-04
/// *"Box starts at 974"* - column 973 is the dark seam beside the art, as on
/// MC-073's `2025-11-01 12_34_31.png`, `Screenshot (3605).png` and
/// `2025-10-20 15_37_25.png`. Its mark is `974,167 599x1233`, which its crop
/// at margin 0 already equals, so AC-1 judges it like every other entry. Its
/// pin was `[974, 167, 599, 1233]`. The list is empty, and stays exact.
///
/// **MC-081 adds all seven** of its Eleceed screenshots (its AC-4), in manifest
/// order, each the crop that clips its mark: six of the right-hand (YouTube)
/// window, and `e03` (`2025-03-07 01_02_31.png`) of the manhwa's window at full
/// height, starting at column 727 against a page that starts at 610.
/// **Measured, not chosen**: read out of one run on `ebdedd7` (release,
/// margin_px 0) in MC-081's RED, on a scratch copy with the seven files and
/// entries added; they equal the crops the Lead PO measured on `main`. They
/// are held here for [`KNOWN_CLIPS`] only: none is in [`KNOWN_MISSES`],
/// because AC-2's bar holds with all seven counted as misses (MC-081
/// `## Handoff`), so no test needs them there. MC-082 and MC-083 take them
/// off.
const MC064_CROPS: [(&str, [u32; 4]); 7] = [
    ("2025-03-16 22_56_00.png", [1820, 121, 402, 1138]), // e01
    ("2025-03-07 00_20_37.png", [1936, 0, 609, 1440]),   // e02
    ("2025-03-07 01_02_31.png", [727, 0, 483, 1440]),    // e03
    ("2025-03-16 22_48_01.png", [1820, 121, 402, 1138]), // e04
    ("2025-03-16 22_51_37.png", [1820, 0, 402, 1400]),   // e05
    ("2025-03-16 22_51_49.png", [1820, 179, 402, 1080]), // e06
    ("2025-03-16 22_54_27.png", [1820, 173, 402, 1086]), // e07
];

/// MC-064, the user's ruling of 2026-09-30: the two of [`MC064_CROPS`] whose
/// crop clips the mark, AC-1's only known exceptions. **Exact in both
/// directions**: AC-1 fails if any other crop clips, and fails if a listed
/// entry's crop is anything but its pin in [`MC064_CROPS`]. MC-065 took
/// `f20` off, MC-066 took `f18` off (its AC-1) and MC-067 took `f09` off (its
/// AC-1). The list was empty: no known clip was left.
///
/// **MC-072 adds two**, by the user's answer of 2026-10-02 (*"all
/// recommended"*): `n02` (`2025-03-07 00_05_58.png`) and `n05` (`2025-11-01
/// 12_34_31.png`), whose crops MC-071 recorded as clips, each held to its
/// measured pin in [`MC064_CROPS`]. The fix stories take them off.
///
/// MC-073 took `n05` off (its AC-2): the user re-marked it to
/// `1007,167 531x1233` (*"Box starts at 1007"*, 2026-10-02), which its crop
/// already equals, so it no longer clips. The list stays exact both ways.
///
/// MC-074 took `n02` off (its AC-1): the user ruled its flat white page paper
/// is page (*"Box stands"*, 2026-10-02), so its crop is to equal its mark
/// `643,115 533x1284` and it no longer clips. The list was empty, and stayed
/// exact both ways: no known clip was left.
///
/// **MC-079 adds one** (its AC-3): `n05` (`2025-10-26 12_13_16.png`), the one
/// entry of MC-077's fresh draw MC-078 read per file, whose crop clips its
/// mark by one column on the left, held to its measured pin in
/// [`MC064_CROPS`]. MC-080 takes it off.
///
/// MC-080 took `n05` off (its AC-2): the user re-marked it to
/// `974,167 599x1233` (*"Box starts at 974"*, 2026-10-04), which its crop
/// already equals, so it no longer clips. The list is empty, and stays exact
/// both ways: no known clip is left.
///
/// **MC-081 adds all seven** of its Eleceed screenshots (its AC-4), each held
/// to its measured pin in [`MC064_CROPS`]: six cropped to the right-hand
/// (YouTube) window, and `e03` cut 117 columns short on the left. MC-082 and
/// MC-083 take them off.
const KNOWN_CLIPS: [&str; 7] = [
    "2025-03-16 22_56_00.png",
    "2025-03-07 00_20_37.png",
    "2025-03-07 01_02_31.png",
    "2025-03-16 22_48_01.png",
    "2025-03-16 22_51_37.png",
    "2025-03-16 22_51_49.png",
    "2025-03-16 22_54_27.png",
];

/// MC-064: every one of `names` whose crop in `rows` is not exactly its pin in
/// [`MC064_CROPS`], as a row naming both. `Scored::got` is compared as text,
/// in the form [`score`] renders it.
fn mc064_crops_moved(rows: &[Scored], names: &[&str]) -> Vec<String> {
    MC064_CROPS
        .iter()
        .filter(|(file, _)| names.contains(file))
        .filter_map(|&(file, [x, y, w, h])| {
            let pinned = format!("crop {x},{y} {w}x{h}");
            match rows.iter().find(|r| r.name == file) {
                None => Some(format!("{file}: not a `tuning` entry here")),
                Some(r) if r.got == pinned => None,
                Some(r) => Some(format!("{file}: got {}, pinned {pinned}", r.got)),
            }
        })
        .collect()
}

// --- One pass over the corpus -----------------------------------------------

/// One corpus entry, its pixel dimensions, and what the real pipeline did with
/// it.
struct Run {
    entry: CorpusEntry,
    /// `(width, height)` from the file's own header, or `None` if it could not
    /// be read. Used only to clamp AC-1's control so that widening a mark that
    /// is already flush with the image edge counts as the no-op it is.
    dims: Option<(u32, u32)>,
    outcome: Outcome,
}

/// Every corpus entry through [`process_file`], in manifest order (MC-018
/// AC-5 guarantees that order, and a table whose rows move between runs is one
/// nobody can diff).
///
/// Each output goes to its own name inside `out`, because `process_file` takes
/// the whole output path rather than a directory (MC-010 plans the name; this
/// test is not the planner and only needs somewhere writable).
/// **Runs over the `tuning` half of the corpus only**, never `corpus::load()`.
///
/// MC-037 added thirty-one `held-out` entries, and `docs/wiki/corpus.md` rules
/// that a held-out entry is scored once, at the end of a v2 attempt, and never
/// before. This is v1's recorded accuracy, not that attempt: scoring held-out
/// entries here would spend the set every time the `integration` gate runs, and
/// would silently change the denominator of the AC-2 bar below from
/// twenty-eight entries to fifty-nine.
fn run_corpus(out: &Path, tuning: &Tuning) -> Vec<Run> {
    corpus::load()
        .into_iter()
        .filter(|entry| entry.split == Split::Tuning)
        .map(|entry| {
            let dest = out.join(entry.name());
            let dims = dimensions(&entry.path);
            let outcome = process_file(&entry.path, &dest, tuning).outcome;
            Run {
                entry,
                dims,
                outcome,
            }
        })
        .collect()
}

/// The image's pixel dimensions from its header alone, format guessed from
/// content rather than from the extension (the corpus has a PNG named `.jpg`
/// in spirit if not in fact, and MC-009 AC-6 is the reason this project never
/// trusts a suffix).
fn dimensions(path: &Path) -> Option<(u32, u32)> {
    image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
}

/// AC-2's slack, in pixels, for the tuning a run was given.
fn slack(tuning: &Tuning) -> u32 {
    tuning.margin_px + SLACK_OVER_MARGIN
}

// --- The two predicates AC-1 and AC-2 turn on -------------------------------

/// The sides on which `rect` fails to contain `expected` - AC-1's predicate,
/// read out of the criterion and not re-derived: `rect.x <= e.x`,
/// `rect.y <= e.y`, `rect.x + rect.w >= e.x + e.w`, `rect.y + rect.h >= e.y +
/// e.h`. All four sides; the amendment narrowed AC-2, not this.
///
/// Empty means no clip. Each entry names the side and both numbers, because
/// "it clipped" is not a bug report and the person reading it has to decide
/// whether the detector or the hand-marked rectangle is wrong.
fn clipped_sides(rect: &Rect, expected: &Rect) -> Vec<String> {
    let mut sides = Vec::new();
    if rect.x > expected.x {
        sides.push(format!(
            "left (crop starts at x={} but the art starts at x={}, {} px of art cut)",
            rect.x,
            expected.x,
            rect.x - expected.x
        ));
    }
    if rect.y > expected.y {
        sides.push(format!(
            "top (crop starts at y={} but the art starts at y={}, {} px of art cut)",
            rect.y,
            expected.y,
            rect.y - expected.y
        ));
    }
    if rect.x + rect.w < expected.x + expected.w {
        sides.push(format!(
            "right (crop ends at x={} but the art ends at x={}, {} px of art cut)",
            rect.x + rect.w,
            expected.x + expected.w,
            (expected.x + expected.w) - (rect.x + rect.w)
        ));
    }
    if rect.y + rect.h < expected.y + expected.h {
        sides.push(format!(
            "bottom (crop ends at y={} but the art ends at y={}, {} px of art cut)",
            rect.y + rect.h,
            expected.y + expected.h,
            (expected.y + expected.h) - (rect.y + rect.h)
        ));
    }
    sides
}

/// How far `rect` sticks out beyond `expected` grown by `slack`, **on the left
/// and right only** - AC-2's "needs no manual fix" predicate as the amendment
/// narrowed it. `None` means the columns are inside the band.
///
/// The row axis is deliberately absent. It is reported by [`overshoot`] and
/// printed by [`table`] on every run, and MC-032 owns it.
fn outside_column_band(rect: &Rect, expected: &Rect, slack: u32) -> Option<String> {
    let mut sides = Vec::new();
    if rect.x + slack < expected.x {
        sides.push(format!("left by {} px", expected.x - rect.x - slack));
    }
    if rect.x + rect.w > expected.x + expected.w + slack {
        sides.push(format!(
            "right by {} px",
            rect.x + rect.w - (expected.x + expected.w) - slack
        ));
    }
    if sides.is_empty() {
        None
    } else {
        Some(format!(
            "columns loose beyond the {slack} px band: {}",
            sides.join(", ")
        ))
    }
}

/// How far the crop sits outside the mark on each side, as
/// `(top, bottom, left, right)`, positive meaning the crop is outside the mark
/// and negative meaning it cut in.
///
/// The same sign convention as MC-019's `## Amendments` table, so the printed
/// numbers can be diffed against the orchestrator's own reproduction by eye.
fn overshoot(rect: &Rect, expected: &Rect) -> (i64, i64, i64, i64) {
    let (rx, ry, rw, rh) = (
        i64::from(rect.x),
        i64::from(rect.y),
        i64::from(rect.w),
        i64::from(rect.h),
    );
    let (ex, ey, ew, eh) = (
        i64::from(expected.x),
        i64::from(expected.y),
        i64::from(expected.w),
        i64::from(expected.h),
    );
    (
        ey - ry,
        (ry + rh) - (ey + eh),
        ex - rx,
        (rx + rw) - (ex + ew),
    )
}

// --- Scoring ----------------------------------------------------------------

/// One row of AC-2's table.
struct Scored {
    name: String,
    expected: String,
    got: String,
    /// `None` when the entry is right; otherwise why it is a miss.
    miss: Option<String>,
    /// The sides AC-1 says were cut into, if any. A clip is always a miss too.
    clip: Vec<String>,
    /// AC-2's column verdict as prose, when the columns are outside the band.
    /// Distinct from [`Scored::miss`], which also carries clips, wrong
    /// outcomes and failures.
    columns_outside: Option<String>,
    /// `(top, bottom, left, right)` overshoot, when a mark and a crop both
    /// exist. The row half is printed and never asserted on.
    delta: Option<(i64, i64, i64, i64)>,
    /// How much the control actually managed to widen this entry's mark, as
    /// `(left, right)`. Equal to the requested grow except where the mark is
    /// flush with the image edge, where widening is a no-op that proves
    /// nothing.
    widened: (u32, u32),
    /// The entry expected a rect and the pipeline produced one, so AC-1's
    /// comparison actually happened on it. AC-1's vacuity guard counts these.
    compared: bool,
    /// The pipeline could not read or write the file at all (AC-3).
    failed: bool,
}

/// Score one run against the manifest, with every expected rect first grown by
/// `grow_cols` pixels on the **left and right**.
///
/// `grow_cols` is 0 for the real measurement; AC-1's negative control is this
/// same function with [`CONTROL_GROW_PX`], which is the point - the control
/// exercises the *comparison under test*, not a copy of it that could drift.
fn score(runs: &[Run], slack: u32, grow_cols: u32) -> Vec<Scored> {
    runs.iter()
        .map(|run| {
            let Run {
                entry,
                dims,
                outcome,
            } = run;
            let (expected_rect, widened) = match &entry.expect {
                Expect::Rect(r) => {
                    let (wide, grown) = widen_cols(*r, grow_cols, *dims);
                    (Some(wide), grown)
                }
                Expect::Flag => (None, (0, 0)),
            };
            let expected = match &expected_rect {
                Some(r) => format!("crop {},{} {}x{}", r.x, r.y, r.w, r.h),
                None => "flag".to_string(),
            };
            let got = match outcome {
                Outcome::Cropped { rect, .. } => {
                    format!("crop {},{} {}x{}", rect.x, rect.y, rect.w, rect.h)
                }
                Outcome::Flagged { reason, .. } => format!("flag {reason:?}"),
                Outcome::Failed { error } => format!("FAILED {error}"),
            };

            let mut clip = Vec::new();
            let mut columns_outside = None;
            let mut delta = None;
            let mut compared = false;
            let mut failed = false;
            let miss = match (&expected_rect, outcome) {
                (Some(exp), Outcome::Cropped { rect, .. }) => {
                    compared = true;
                    delta = Some(overshoot(rect, exp));
                    clip = clipped_sides(rect, exp);
                    if clip.is_empty() {
                        columns_outside = outside_column_band(rect, exp, slack);
                        columns_outside.clone()
                    } else {
                        Some(format!("CLIPPED on the {}", clip.join("; ")))
                    }
                }
                (Some(_), Outcome::Flagged { reason, .. }) => {
                    Some(format!("expected a crop, got flag {reason:?}"))
                }
                (None, Outcome::Flagged { .. }) => None,
                (None, Outcome::Cropped { rect, .. }) => Some(format!(
                    "expected a flag, got crop {},{} {}x{}",
                    rect.x, rect.y, rect.w, rect.h
                )),
                (_, Outcome::Failed { error }) => {
                    failed = true;
                    Some(format!("could not be processed: {error}"))
                }
            };

            Scored {
                name: entry.name(),
                expected,
                got,
                miss,
                clip,
                columns_outside,
                delta,
                widened,
                compared,
                failed,
            }
        })
        .collect()
}

/// `rect` widened by `grow` pixels on the left and on the right, clamped to
/// the image, with how much each side actually moved.
///
/// The clamp matters for AC-1's control and only for AC-1's control: widening
/// a mark past the image's own edge would produce a rectangle **no** crop of
/// that image could contain, so an unclamped control would fire on geometry
/// rather than on the detector and look exactly like a real one. Where the
/// dimensions could not be read, the right side does not grow - conservative
/// in the direction that makes the control harder to satisfy, never easier.
fn widen_cols(rect: Rect, grow: u32, dims: Option<(u32, u32)>) -> (Rect, (u32, u32)) {
    let left = grow.min(rect.x);
    let room_right = dims.map_or(0, |(w, _)| w.saturating_sub(rect.x + rect.w));
    let right = grow.min(room_right);
    (
        Rect {
            x: rect.x - left,
            w: rect.w + left + right,
            ..rect
        },
        (left, right),
    )
}

// --- The table AC-2 asks for ------------------------------------------------

/// File, expected, got, right/miss, reason - in manifest order, with the
/// **row-axis deltas printed alongside and never asserted on**.
///
/// AC-2 as amended asks for exactly that: the row numbers are MC-032's to
/// move, and the way a parked axis stays honest is that its deficit is on
/// screen every time the column axis passes.
fn table(rows: &[Scored]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<30} {:<24} {:<29} {:>13} {:>13}  {:<6} reason",
        "file", "expected", "got", "cols(l,r)", "rows(t,b)*", "verdict"
    );
    for row in rows {
        let (cols, rows_txt) = match row.delta {
            Some((top, bottom, left, right)) => (
                format!("{left:>+5},{right:>+5}"),
                format!("{top:>+5},{bottom:>+5}"),
            ),
            None => ("-".to_string(), "-".to_string()),
        };
        let _ = writeln!(
            out,
            "{:<30} {:<24} {:<29} {:>13} {:>13}  {:<6} {}",
            row.name,
            row.expected,
            row.got,
            cols,
            rows_txt,
            // MC-062: a known miss is still shown, and marked as left out of
            // AC-2's bar.
            if row.miss.is_none() {
                "right"
            } else if KNOWN_MISSES.contains(&row.name.as_str()) {
                "KNOWN"
            } else {
                "MISS"
            },
            row.miss.as_deref().unwrap_or(""),
        );
    }

    let mut col_worst: Vec<i64> = Vec::new();
    let mut row_worst: Vec<i64> = Vec::new();
    for row in rows {
        if let Some((top, bottom, left, right)) = row.delta {
            col_worst.push(left.max(right));
            row_worst.push(top.max(bottom));
        }
    }
    col_worst.sort_unstable();
    row_worst.sort_unstable();
    let _ = writeln!(
        out,
        "\n* the row deltas are printed and NOT asserted on: MC-019's AC-2 was \
         narrowed to the column axis by its `## Amendments`, and MC-032 carries \
         the rows.\n  worst column overshoot per entry, sorted: {col_worst:?}\
         \n  worst row    overshoot per entry, sorted: {row_worst:?}"
    );
    out
}

/// Print the table for `--nocapture` runs and hand it back for the assertion
/// message, which is where it actually has to be.
fn report(rows: &[Scored]) -> String {
    let rendered = table(rows);
    println!("{rendered}");
    rendered
}

/// One corpus pass at `Tuning::default()`, scored at the settled slack with no
/// control widening, plus the rendered table.
///
/// The `TempDir` is returned so the caller keeps it alive: dropping it deletes
/// the outputs, and AC-3 is a statement about writes that actually happened.
fn default_pass() -> (TempDir, Tuning, Vec<Scored>, String) {
    let out = TempDir::new().expect("a temp dir for the outputs");
    let tuning = Tuning::default();
    let runs = run_corpus(out.path(), &tuning);
    let rows = score(&runs, slack(&tuning), 0);
    let rendered = report(&rows);
    (out, tuning, rows, rendered)
}

// --- AC-1 -------------------------------------------------------------------

/// AC-1: no crop may cut into the artwork the user marked.
///
/// The one failure the brief calls unacceptable: "leftover border is a minor
/// miss; cut-into art is a failure". One violation fails, and names the file
/// and the side.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn no_corpus_crop_cuts_into_the_artwork_its_manifest_entry_marked() {
    let (_out, _tuning, rows, rendered) = default_pass();

    // Vacuity guard. "Zero clips" is trivially true of a detector that flags
    // everything, and AC-1 would then pass while measuring nothing at all.
    let compared = rows.iter().filter(|r| r.compared).count();
    assert!(
        compared > 0,
        "AC-1 compared no entries at all: not one corpus screenshot with an \
         expected rect came back `Cropped`, so \"zero clips\" is vacuous.\n\n{rendered}"
    );

    // MC-064: the known clips, each held to its measured crop.
    let moved = mc064_crops_moved(&rows, &KNOWN_CLIPS);
    assert!(
        moved.is_empty(),
        "MC-064, MC-072, MC-079, MC-081: each known clip (KNOWN_CLIPS; the user's rulings of 2026-09-30 and 2026-10-02, MC-079 AC-3, MC-081 AC-4) must \
         still be cropped to exactly its pin in MC064_CROPS - if a fix moves one, \
         take it off the list and let AC-1 judge it:\n{}\n\n{rendered}",
        moved.join("\n")
    );

    let clipped: Vec<String> = rows
        .iter()
        .filter(|r| !r.clip.is_empty())
        .filter(|r| !KNOWN_CLIPS.contains(&r.name.as_str()))
        .map(|r| format!("  {}: {}", r.name, r.clip.join("; ")))
        .collect();

    assert!(
        clipped.is_empty(),
        "AC-1: {} of the {compared} compared crops cut into the artwork their \
         manifest entry marked, starting with {}. A clip here is either a \
         detector fault or a mis-marked rectangle, and only the person who drew \
         it can say which - open the file, decide, and record it in the story's \
         `## Notes`. Never edit the manifest to make this pass.\n\n{}\n\n{rendered}",
        clipped.len(),
        clipped.first().map_or("-", String::as_str).trim_start(),
        clipped.join("\n")
    );
}

/// AC-1's negative control on the metric, on the column axis at 1 px.
///
/// "Zero clips" is satisfiable by a detector that returns the whole image every
/// time, so a green AC-1 on its own proves nothing about whether the comparison
/// is tight enough to notice anything. Widening every expected rect on the
/// **left and right** by [`CONTROL_GROW_PX`] and re-running the *same*
/// containment check must make at least one entry fail; if it does not, every
/// crop clears its marked columns by more than a pixel and AC-1's green is an
/// artefact of slack rather than a measurement of accuracy.
///
/// On this tree it fires on exactly one entry, `2025-10-14 23_30_20.png`. The
/// assertion is the criterion's - *at least one* - and the identity is in the
/// message and in the story's handoff, because pinning the identity here would
/// make a detector that got *better* on that file look like a broken control.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn widening_every_expected_rect_one_pixel_on_the_columns_makes_the_zero_clip_check_fail() {
    let out = TempDir::new().expect("a temp dir for the outputs");
    let tuning = Tuning::default();
    let runs = run_corpus(out.path(), &tuning);
    let rows = score(&runs, slack(&tuning), CONTROL_GROW_PX);

    // MC-064: a known clip clips before any widening, so it would make this
    // control fire whatever the comparison did; it is left out.
    let fired: Vec<&str> = rows
        .iter()
        .filter(|r| !r.clip.is_empty())
        .filter(|r| !KNOWN_CLIPS.contains(&r.name.as_str()))
        .map(|r| r.name.as_str())
        .collect();
    let compared = rows.iter().filter(|r| r.compared).count();

    // How much room each crop has beside its marked columns. This is the
    // number that explains a control that does not fire, so it goes in the
    // message rather than being left for somebody to go and measure.
    let mut headroom: Vec<(i64, String)> = rows
        .iter()
        .filter_map(|r| {
            r.delta
                .map(|(_, _, left, right)| (left.min(right), r.name.clone()))
        })
        .collect();
    headroom.sort();
    let listing: Vec<String> = headroom
        .iter()
        .map(|(gap, name)| format!("  {gap:>6} px  {name}"))
        .collect();
    let no_ops = rows
        .iter()
        .filter(|r| r.compared && (r.widened.0 < CONTROL_GROW_PX || r.widened.1 < CONTROL_GROW_PX))
        .count();

    println!("AC-1's control at {CONTROL_GROW_PX} px on the columns fired on {fired:?}");

    assert!(
        !fired.is_empty(),
        "AC-1's control did not fire: widening every expected rect by \
         {CONTROL_GROW_PX} px on the left and right made 0 of {compared} \
         compared crops fail the containment check, so AC-1's \"zero clips\" \
         cannot tell a tight crop from one that returned most of the image. \
         Every crop clears its marked columns by more than {CONTROL_GROW_PX} px. \
         Column headroom (the smaller of expected.x - crop.x and \
         crop.right - expected.right), smallest first:\n{}\n\n({no_ops} marks \
         are flush with an image edge, where widening is a no-op and proves \
         nothing.)",
        listing.join("\n")
    );
}

// --- AC-2 -------------------------------------------------------------------

/// AC-2: at least nine corpus screenshots in ten are right on the column axis.
///
/// Right means: an expected rect came back as a `Cropped` that contains it on
/// all four sides (AC-1) and whose **left and right** edges sit inside the
/// mark's left and right grown by `margin_px + 8`; an expected flag came back
/// `Flagged`. Everything else is a miss, including a flag where a crop was
/// wanted and a crop where a flag was wanted.
///
/// The row deltas are in the table this prints and nothing here asserts on
/// them. That is MC-019's `## Amendments` doing its job in the open.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn at_least_nine_corpus_screenshots_in_ten_are_right_on_the_column_axis() {
    let (_out, tuning, rows, rendered) = default_pass();
    let band = slack(&tuning);

    // MC-062: the known misses, exact in both directions. Every listed entry
    // must be a `tuning` entry here, and still a miss; `left` is measured.
    let known_and_missed: Vec<&str> = KNOWN_MISSES
        .into_iter()
        .filter(|name| rows.iter().any(|r| r.name == *name && r.miss.is_some()))
        .collect();
    assert_eq!(
        known_and_missed,
        KNOWN_MISSES.to_vec(),
        "AC-2's known misses (KNOWN_MISSES, the user's rulings of 2026-09-30, \
         MC-062) must each still be a `tuning` entry and still a miss. One absent \
         from `left` is gone from the corpus or is right now: take it off the \
         list, and it counts toward the bar again.\n\n{rendered}"
    );
    // MC-064: its four known misses are held to their measured crops too.
    let moved = mc064_crops_moved(&rows, &KNOWN_MISSES);
    assert!(
        moved.is_empty(),
        "MC-064: each of its four known misses must still be cropped to exactly its \
         pin in MC064_CROPS - if a fix moves one, take it off KNOWN_MISSES and let \
         the bar count it:\n{}\n\n{rendered}",
        moved.join("\n")
    );

    let counted: Vec<&Scored> = rows
        .iter()
        .filter(|r| !KNOWN_MISSES.contains(&r.name.as_str()))
        .collect();
    let total = counted.len();
    assert!(
        total >= MIN_ENTRIES,
        "AC-2 needs at least {MIN_ENTRIES} entries for a percentage to mean \
         anything; the corpus loaded {total} besides the known misses"
    );

    let right = counted.iter().filter(|r| r.miss.is_none()).count();
    let misses: Vec<String> = counted
        .iter()
        .filter_map(|r| r.miss.as_ref().map(|why| format!("  {}: {why}", r.name)))
        .collect();

    let fraction = right as f64 / total as f64;

    assert!(
        fraction >= RIGHT_FRACTION,
        "AC-2: {right} of {total} corpus screenshots are right on the column \
         axis ({:.1}%), below the {:.0}% bar. A crop is right when it contains \
         its expected rect on all four sides and its left and right edges sit \
         inside the mark's, grown by {band} px (margin_px {} + \
         {SLACK_OVER_MARGIN}); a flag is right when the entry asked to be left \
         alone. The {} known misses (KNOWN_MISSES: {KNOWN_MISSES:?}) are left \
         out of both counts, by the user's rulings of 2026-09-30; they are still \
         in the table. The row deltas in the table are NOT part of this count - \
         they are MC-032's.\n\nMisses counted:\n{}\n\n{rendered}",
        fraction * 100.0,
        RIGHT_FRACTION * 100.0,
        tuning.margin_px,
        KNOWN_MISSES.len(),
        misses.join("\n")
    );
    println!(
        "AC-2: {right} of {total} right ({:.1}%), {} known misses left out",
        fraction * 100.0,
        KNOWN_MISSES.len()
    );
}

/// AC-2's negative control on the metric: the column window is tight enough
/// that the corpus is not all inside it.
///
/// A 90% bar measured through a window wide enough to admit everything is a
/// bar that cannot be failed, and `margin_px + 8` is only a *measurement*
/// while some entry sits outside it. On this tree that entry is `Screenshot
/// (93).jpg`, at +20 px on the right against a slack of 11.
///
/// This is the one assertion here that a *better* detector could turn red. It
/// is written that way deliberately and the failure message says so: closing
/// the last column miss is a real change to what this story characterises, and
/// it belongs in a story with the user's decision on it, not in a quiet edit to
/// the window.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn the_column_window_is_narrower_than_the_corpus_so_the_ninety_percent_can_be_missed() {
    let (_out, tuning, rows, rendered) = default_pass();
    let band = slack(&tuning);

    // MC-064: a known miss is outside the window by a ruling, not by the
    // comparison (`2025-03-16 22_47_44.png` by 1381 px), so it would make this
    // control fire on any window at all; known misses are left out.
    let outside: Vec<String> = rows
        .iter()
        .filter(|r| !KNOWN_MISSES.contains(&r.name.as_str()))
        .filter_map(|r| {
            r.columns_outside
                .as_ref()
                .map(|why| format!("  {}: {why}", r.name))
        })
        .collect();
    let compared = rows.iter().filter(|r| r.compared).count();

    println!(
        "AC-2's control: {} of {compared} compared crops are outside the \
              {band} px column window:\n{}",
        outside.len(),
        outside.join("\n")
    );

    assert!(
        !outside.is_empty(),
        "AC-2's control did not fire: all {compared} compared crops sit inside \
         the {band} px column window (margin_px {} + {SLACK_OVER_MARGIN}), so \
         the 90% bar above is measured through a window nothing can fall out \
         of and would pass on any detector at all. Either the window was \
         widened - which AC-2 forbids, and `margin_px` cannot buy, see this \
         file's module docs - or the detector genuinely closed the last column \
         miss, which is a change to what MC-019 characterises and needs the \
         user's decision in a story, not an edit here.\n\n{rendered}",
        tuning.margin_px
    );
}

// --- AC-3 -------------------------------------------------------------------

/// AC-3: every corpus file is readable and writable - nothing comes back
/// `Failed`.
///
/// Separate from AC-2, which would also count a failure as a miss, because the
/// two say different things. A miss is a detector that was wrong; a failure is
/// a file the engine could not open or an output it could not write, and that
/// is a broken checkout or a broken codec, not a tuning result.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn every_corpus_file_is_read_and_its_output_written_without_failing() {
    let (_out, _tuning, rows, rendered) = default_pass();

    let failures: Vec<String> = rows
        .iter()
        .filter(|r| r.failed)
        .map(|r| format!("  {}: {}", r.name, r.got))
        .collect();

    assert!(
        failures.is_empty(),
        "AC-3: {} of {} corpus files could not be read, decoded or written, \
         starting with {}:\n{}\n\n{rendered}",
        failures.len(),
        rows.len(),
        failures.first().map_or("-", String::as_str).trim_start(),
        failures.join("\n")
    );
}
