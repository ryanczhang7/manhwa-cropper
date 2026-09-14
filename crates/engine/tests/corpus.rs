//! MC-019, AC-1 to AC-3: the detector meets the accuracy bar on the corpus.
//!
//! The product brief's section 7, expressed as a test. Every entry of the
//! calibration corpus (MC-018) is put through the real pipeline -
//! [`process_file`], `Tuning::default()`, no stubs and no shortcuts - and the
//! outcomes are measured against the rectangles a person drew by hand:
//!
//! * **AC-1, zero clips.** No cropped rect may cut into its expected rect.
//! * **AC-2, nine in ten.** At least 90% of entries need no manual fix.
//! * **AC-3.** Nothing fails: every corpus file is readable and writable.
//!
//! This is the first thing in the project that can *contradict* the detector.
//! Every other suite measures it against fixtures whose author already knew
//! what the answer should be; these twenty-eight are screenshots the user took
//! for their own reading, marked before any of this ran.
//!
//! # Running it
//!
//! `#[ignore]`d, so `cargo test --workspace` (the `unit` gate, debug) never
//! runs it. The `integration` gate does, in release - and MC-019 makes that
//! gate *required* through the story's `required_gates`:
//!
//! ```text
//! cargo test --workspace --release -- --ignored
//! ```
//!
//! To run it alone, with the per-file table on stdout:
//!
//! ```text
//! cargo test --workspace --release --test corpus -- --ignored --nocapture
//! ```
//!
//! **Release is not optional**, for the same reason `tests/perf.rs` says so:
//! decoding 21 MB of screenshots in an unoptimised build measures the profile.
//!
//! # The table has to survive capture
//!
//! AC-2 requires a per-file table, and the gate does not pass `--nocapture`, so
//! a `println!` would be swallowed on exactly the run whose output somebody
//! needs. Every assertion here therefore carries [`table`]'s output *in its own
//! message*. It is printed as well, for `--nocapture` runs; the printed copy is
//! a convenience and the message is the contract.
//!
//! # What is settled, what is mechanical, what is the oracle
//!
//! * **Settled**, read out of the acceptance criteria and never re-derived:
//!   the 90% bar ([`RIGHT_FRACTION`]), the `margin_px + 8` slack
//!   ([`SLACK_OVER_MARGIN`], 11 px at today's `margin_px` of 3), and the
//!   containment predicate itself ([`clipped_sides`]). None of these is a
//!   number this test is free to choose; if one is wrong it goes back to the
//!   product owner under `## Amendments`.
//! * **Mechanical**: AC-3, and the arithmetic of the two counts.
//! * **The oracle is the corpus.** `manifest.json` is the answer key, and
//!   nothing here re-derives it. It can still be *wrong*: a reported clip may
//!   be a mis-marked rectangle rather than a detector fault, which is why AC-1
//!   names the file and the offending side rather than only counting. That
//!   finding goes to the user, who opens the image and decides; the manifest is
//!   never edited to make this test pass. MC-019's `## Model guidance` is
//!   explicit about it.
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
//! the band by *k* at the same time: the left check `rect.x + slack >= e.x` has
//! `rect.x` fall by *k* and `slack` rise by *k*, and the right check
//! `rect.x + rect.w <= e.x + e.w + slack` has both sides rise by *k*. The
//! comparison is invariant. `margin_px` cannot be turned up to buy AC-2.
//!
//! # What this corpus cannot catch
//!
//! Recorded so nobody reads a pass here as more than it is. MC-018's handoff
//! established that all 7 `"flag"` entries are the non-2560x1440 images and all
//! 21 rect entries are 2560x1440 desktop captures. The correlation is causal -
//! an already-cropped panel image has no chrome to find - but it means a
//! detector keying on something size-correlated would pass every assertion
//! below, and there is no full-screen capture here that should be flagged and
//! no mobile-sized screenshot that should be cropped. The tags
//! `diagonal-gutter` and `overhang-text` exist but are not yet applied to any
//! entry, so nothing here may depend on them; [`table`] tallies misses by tag
//! anyway, so the moment they are applied the clustering shows up for free.

// Included directly rather than through `common/mod.rs`. The app crate reaches
// that module with `#[path]`, so declaring the loader there would compile it -
// and its `serde` dependency - into test binaries that have no use for either.
// MC-018's test plan has the error it produced.
#[path = "common/corpus.rs"]
mod corpus;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use corpus::{CorpusEntry, Expect};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};
use tempfile::TempDir;

// --- The settled numbers ----------------------------------------------------

/// AC-2's bar, from the brief's section 7: nine in ten need no manual fix.
/// Settled - if this is wrong it goes back to the product owner, never down.
const RIGHT_FRACTION: f64 = 0.90;

/// AC-2's slack over `Tuning::margin_px`: a crop may be loose by the margin the
/// detector deliberately adds plus eight pixels, and no more. Settled. At
/// today's `margin_px` of 3 the band is 11 px on every side.
const SLACK_OVER_MARGIN: u32 = 8;

/// How far AC-1's negative control widens every expected rect on the top
/// before re-running the containment check.
///
/// AC-1 sets this by measurement, not by choice: start at 1 px, and "if it does
/// not [fail on at least one entry], the detector's margin is hiding the
/// comparison and the control must move to 4 px". Measured out-of-band in RED
/// against `Tuning::default()`: **1 px made 0 entries fail, and so did 4 px**,
/// because the detector's crops are 97 to 306 px above their expected top, not
/// `margin_px` above it. 4 px is what AC-1's ladder ends at, so 4 px is what is
/// written here, and the residual - that the ladder does not reach today's
/// detector either - is in the story's `## Handoff: RED -> GREEN` for the
/// orchestrator, not resolved quietly here.
const CONTROL_GROW_PX: u32 = 4;

/// MC-018 AC-1's floor, read out here as a vacuity guard: a percentage over a
/// handful of entries would not mean anything.
const MIN_ENTRIES: usize = 20;

// --- One pass over the corpus -----------------------------------------------

/// One corpus entry and what the real pipeline did with it.
struct Run {
    entry: CorpusEntry,
    outcome: Outcome,
}

/// Every corpus entry through [`process_file`], in manifest order (MC-018
/// AC-5 guarantees that order, and a table whose rows move between runs is one
/// nobody can diff).
///
/// Each output goes to its own name inside `out`, because `process_file` takes
/// the whole output path rather than a directory (MC-010 plans the name; this
/// test is not the planner and only needs somewhere writable).
fn run_corpus(out: &Path, tuning: &Tuning) -> Vec<Run> {
    corpus::load()
        .into_iter()
        .map(|entry| {
            let dest = out.join(entry.name());
            let outcome = process_file(&entry.path, &dest, tuning).outcome;
            Run { entry, outcome }
        })
        .collect()
}

/// AC-2's slack, in pixels, for the tuning a run was given.
fn slack(tuning: &Tuning) -> u32 {
    tuning.margin_px + SLACK_OVER_MARGIN
}

// --- The two predicates AC-1 and AC-2 turn on -------------------------------

/// The sides on which `rect` fails to contain `expected` - AC-1's predicate,
/// read out of the criterion and not re-derived: `rect.x <= e.x`,
/// `rect.y <= e.y`, `rect.x + rect.w >= e.x + e.w`, `rect.y + rect.h >= e.y +
/// e.h`.
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

/// How far `rect` sticks out beyond `expected` grown by `slack` on every side -
/// AC-2's "needs no manual fix" predicate. `None` means it is inside the band.
fn outside_band(rect: &Rect, expected: &Rect, slack: u32) -> Option<String> {
    let mut sides = Vec::new();
    if rect.x + slack < expected.x {
        sides.push(format!("left by {} px", expected.x - rect.x - slack));
    }
    if rect.y + slack < expected.y {
        sides.push(format!("top by {} px", expected.y - rect.y - slack));
    }
    if rect.x + rect.w > expected.x + expected.w + slack {
        sides.push(format!(
            "right by {} px",
            rect.x + rect.w - (expected.x + expected.w) - slack
        ));
    }
    if rect.y + rect.h > expected.y + expected.h + slack {
        sides.push(format!(
            "bottom by {} px",
            rect.y + rect.h - (expected.y + expected.h) - slack
        ));
    }
    if sides.is_empty() {
        None
    } else {
        Some(format!(
            "loose beyond the {slack} px band: {}",
            sides.join(", ")
        ))
    }
}

// --- Scoring ----------------------------------------------------------------

/// One row of AC-2's table.
struct Scored {
    name: String,
    tags: Vec<String>,
    expected: String,
    got: String,
    /// `None` when the entry is right; otherwise why it is a miss.
    miss: Option<String>,
    /// The sides AC-1 says were cut into, if any. A clip is always a miss too.
    clip: Vec<String>,
    /// The entry expected a rect and the pipeline produced one, so AC-1's
    /// comparison actually happened on it. AC-1's vacuity guard counts these.
    compared: bool,
    /// The pipeline could not read or write the file at all (AC-3).
    failed: bool,
    /// Which of [`DEFERRED_TAGS`] this entry carries.
    deferred: Vec<&'static str>,
}

/// The two tags MC-018 defined and deliberately did not apply to any entry
/// yet. Nothing here may depend on them - a corpus where they are absent has
/// to read the same as one where they are present - but [`table`] takes a
/// census so that the day the user applies them, a cluster of misses on
/// `diagonal-gutter` is visible without anyone changing this file. MC-018's
/// handoff calls that the next diagnostic step if the failures look
/// unexplained.
const DEFERRED_TAGS: [&str; 2] = ["diagonal-gutter", "overhang-text"];

/// Score one run against the manifest, with every expected rect first grown by
/// `grow_top` pixels on the top.
///
/// `grow_top` is 0 for the real measurement; AC-1's negative control is the
/// same function with [`CONTROL_GROW_PX`], which is the point - the control
/// exercises the *comparison under test*, not a copy of it that could drift.
fn score(runs: &[Run], slack: u32, grow_top: u32) -> Vec<Scored> {
    runs.iter()
        .map(|Run { entry, outcome }| {
            let expected_rect = match &entry.expect {
                Expect::Rect(r) => Some(widen_top(*r, grow_top)),
                Expect::Flag => None,
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
            let mut compared = false;
            let mut failed = false;
            let miss = match (&expected_rect, outcome) {
                (Some(exp), Outcome::Cropped { rect, .. }) => {
                    compared = true;
                    clip = clipped_sides(rect, exp);
                    if clip.is_empty() {
                        outside_band(rect, exp, slack)
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
                tags: entry.tags.clone(),
                expected,
                got,
                miss,
                clip,
                compared,
                failed,
                deferred: DEFERRED_TAGS
                    .iter()
                    .filter(|tag| entry.has_tag(tag))
                    .copied()
                    .collect(),
            }
        })
        .collect()
}

/// `rect` with its top edge raised by `grow` pixels, clamped at the image's
/// top. Growing a rect that already starts at `y = 0` is a no-op, and AC-1's
/// control reports how many of those there were rather than counting them as
/// evidence.
fn widen_top(rect: Rect, grow: u32) -> Rect {
    let grow = grow.min(rect.y);
    Rect {
        y: rect.y - grow,
        h: rect.h + grow,
        ..rect
    }
}

// --- The table AC-2 asks for ------------------------------------------------

/// File, expected, got, right/miss, reason - in manifest order, with a tally of
/// misses by tag underneath.
///
/// The tally is not an assertion and nothing turns on it. It exists because the
/// difference between "the constants need tuning" and "the detector needs a
/// capability it does not have" usually shows up as failures clustering on one
/// tag, and MC-018 added `diagonal-gutter` and `overhang-text` for exactly that
/// reading.
fn table(rows: &[Scored]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<34} {:<24} {:<30} {:<6} reason",
        "file", "expected", "got", "verdict"
    );
    for row in rows {
        let _ = writeln!(
            out,
            "{:<34} {:<24} {:<30} {:<6} {}  [{}]",
            row.name,
            row.expected,
            row.got,
            if row.miss.is_none() { "right" } else { "MISS" },
            row.miss.as_deref().unwrap_or(""),
            row.tags.join(",")
        );
    }

    let mut by_tag: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for row in rows {
        for tag in &row.tags {
            let slot = by_tag.entry(tag.as_str()).or_insert((0, 0));
            slot.0 += 1;
            if row.miss.is_some() {
                slot.1 += 1;
            }
        }
    }
    let _ = writeln!(out, "\nmisses by tag:");
    for (tag, (total, missed)) in &by_tag {
        let _ = writeln!(out, "  {tag:<18} {missed}/{total}");
    }

    for tag in DEFERRED_TAGS {
        let carrying: Vec<&Scored> = rows.iter().filter(|r| r.deferred.contains(&tag)).collect();
        let missed = carrying.iter().filter(|r| r.miss.is_some()).count();
        let _ = writeln!(
            out,
            "  {tag:<18} {missed}/{}{}",
            carrying.len(),
            if carrying.is_empty() {
                "   (MC-018 defined this tag and has not applied it to any entry yet)"
            } else {
                ""
            }
        );
    }
    out
}

/// Print the table for `--nocapture` runs and hand it back for the assertion
/// message, which is where it actually has to be.
fn report(rows: &[Scored]) -> String {
    let rendered = table(rows);
    println!("{rendered}");
    rendered
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
    let out = TempDir::new().expect("a temp dir for the outputs");
    let tuning = Tuning::default();
    let runs = run_corpus(out.path(), &tuning);
    let rows = score(&runs, slack(&tuning), 0);
    let rendered = report(&rows);

    // Vacuity guard. "Zero clips" is trivially true of a detector that flags
    // everything, and AC-1 would then pass while measuring nothing at all.
    let compared = rows.iter().filter(|r| r.compared).count();
    assert!(
        compared > 0,
        "AC-1 compared no entries at all: not one corpus screenshot with an \
         expected rect came back `Cropped`, so \"zero clips\" is vacuous.\n\n{rendered}"
    );

    let clipped: Vec<String> = rows
        .iter()
        .filter(|r| !r.clip.is_empty())
        .map(|r| format!("  {}: {}", r.name, r.clip.join("; ")))
        .collect();

    assert!(
        clipped.is_empty(),
        "AC-1: {} of the {compared} compared crops cut into the artwork their \
         manifest entry marked. A clip here is either a detector fault or a \
         mis-marked rectangle, and only the person who drew it can say which - \
         open the file, decide, and record it in the story's `## Notes`. Never \
         edit the manifest to make this pass.\n\n{}\n\n{rendered}",
        clipped.len(),
        clipped.join("\n")
    );
}

/// AC-1's negative control on the metric.
///
/// "Zero clips" is satisfiable by a detector that returns the whole image every
/// time, so a green AC-1 on its own proves nothing about whether the comparison
/// is tight enough to notice anything. Widening every expected rect on the top
/// by [`CONTROL_GROW_PX`] and re-running the *same* containment check must make
/// at least one entry fail; if it does not, every crop clears its expected top
/// by more than [`CONTROL_GROW_PX`] pixels and AC-1's green is an artefact of
/// slack rather than a measurement of accuracy.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn widening_every_expected_rect_on_the_top_makes_the_zero_clip_check_fail() {
    let out = TempDir::new().expect("a temp dir for the outputs");
    let tuning = Tuning::default();
    let runs = run_corpus(out.path(), &tuning);
    let rows = score(&runs, slack(&tuning), CONTROL_GROW_PX);

    let fired: Vec<&Scored> = rows.iter().filter(|r| !r.clip.is_empty()).collect();

    // How much room each crop has above its expected top. This is the number
    // that explains a control that does not fire, so it goes in the message.
    let mut headroom: Vec<(i64, String)> = runs
        .iter()
        .filter_map(|Run { entry, outcome }| match (&entry.expect, outcome) {
            (Expect::Rect(exp), Outcome::Cropped { rect, .. }) => {
                Some((i64::from(exp.y) - i64::from(rect.y), entry.name()))
            }
            _ => None,
        })
        .collect();
    headroom.sort();
    let at_top = runs
        .iter()
        .filter(|r| matches!(&r.entry.expect, Expect::Rect(e) if e.y == 0))
        .count();
    let listing: Vec<String> = headroom
        .iter()
        .map(|(gap, name)| format!("  {gap:>6} px  {name}"))
        .collect();

    assert!(
        !fired.is_empty(),
        "AC-1's control did not fire: widening every expected rect by \
         {CONTROL_GROW_PX} px on the top made 0 of {} compared crops fail the \
         containment check, so AC-1's \"zero clips\" cannot tell a tight crop \
         from one that returned most of the image. Every crop clears its \
         expected top by more than {CONTROL_GROW_PX} px. Headroom \
         (expected.y - crop.y), smallest first:\n{}\n\n({at_top} expected rects \
         sit at y=0, where widening is a no-op and proves nothing.)",
        headroom.len(),
        listing.join("\n")
    );
}

// --- AC-2 -------------------------------------------------------------------

/// AC-2: at least nine corpus screenshots in ten need no manual fix.
///
/// Right means: an expected rect came back as a `Cropped` that contains it
/// (AC-1) and sits inside it grown by `margin_px + 8` on every side; an
/// expected flag came back `Flagged`. Everything else is a miss, including a
/// flag where a crop was wanted and a crop where a flag was wanted.
#[test]
#[ignore = "reads the 21 MB calibration corpus; the integration gate runs it in release"]
fn at_least_nine_corpus_screenshots_in_ten_need_no_manual_fix() {
    let out = TempDir::new().expect("a temp dir for the outputs");
    let tuning = Tuning::default();
    let band = slack(&tuning);
    let runs = run_corpus(out.path(), &tuning);
    let rows = score(&runs, band, 0);
    let rendered = report(&rows);

    let total = rows.len();
    assert!(
        total >= MIN_ENTRIES,
        "AC-2 needs at least {MIN_ENTRIES} entries for a percentage to mean \
         anything; the corpus loaded {total}"
    );

    let right = rows.iter().filter(|r| r.miss.is_none()).count();
    let misses: Vec<String> = rows
        .iter()
        .filter_map(|r| r.miss.as_ref().map(|why| format!("  {}: {why}", r.name)))
        .collect();

    #[allow(clippy::cast_precision_loss)]
    let fraction = right as f64 / total as f64;

    assert!(
        fraction >= RIGHT_FRACTION,
        "AC-2: {right} of {total} corpus screenshots are right ({:.1}%), below \
         the {:.0}% bar. A crop is right when it contains its expected rect and \
         sits inside it grown by {band} px (margin_px {} + {SLACK_OVER_MARGIN}); \
         a flag is right when the entry asked to be left alone.\n\nMisses:\n{}\n\n{rendered}",
        fraction * 100.0,
        RIGHT_FRACTION * 100.0,
        tuning.margin_px,
        misses.join("\n")
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
    let out = TempDir::new().expect("a temp dir for the outputs");
    let tuning = Tuning::default();
    let runs = run_corpus(out.path(), &tuning);
    let rows = score(&runs, slack(&tuning), 0);
    let rendered = report(&rows);

    let failures: Vec<String> = rows
        .iter()
        .filter(|r| r.failed)
        .map(|r| format!("  {}: {}", r.name, r.got))
        .collect();

    assert!(
        failures.is_empty(),
        "AC-3: {} of {} corpus files could not be read, decoded or written:\n{}\n\n{rendered}",
        failures.len(),
        rows.len(),
        failures.join("\n")
    );
}

// --- AC-4 -------------------------------------------------------------------

/// The tuning table in `docs/wiki/architecture.md`, as `constant -> the first
/// number in its Default cell`.
///
/// The cells are prose - "10 (per-channel, 0..255)", "0.30 of the image's
/// height (horizontal strips) or width (vertical strips)" - so the value is the
/// leading run of digits and dots and the rest is commentary. Nothing else in
/// the cell is constrained, which is deliberate: the table earns its keep by
/// explaining the constants, and a parser that demanded a bare number would
/// make it worse documentation to satisfy a test.
fn architecture_tuning_table() -> BTreeMap<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("wiki")
        .join("architecture.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));

    let mut rows = BTreeMap::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.starts_with("| Constant | Default |") {
            in_table = true;
            continue;
        }
        if in_table {
            if !line.starts_with('|') {
                break;
            }
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            // "", constant, default, tuned by, ""
            if cells.len() < 4 {
                continue;
            }
            let name = cells[1].trim_matches('`');
            if name.is_empty() || name.starts_with("---") {
                continue;
            }
            let value: String = cells[2]
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            rows.insert(name.to_string(), value);
        }
    }
    rows
}

/// AC-4, the half of it a test can hold: `architecture.md`'s tuning table says
/// exactly what `Tuning::default()` says.
///
/// Not `#[ignore]`d, and it reads a file rather than the corpus, so it costs
/// nothing and runs in the `unit` gate on every commit. That is the point -
/// MC-019 is the story that may change these constants, and a table that
/// silently stops matching the code is a document that teaches the next agent
/// the wrong numbers. The architecture document calls them "settled for RED
/// (read them out; do not calibrate)", which only means anything while it is
/// true.
///
/// The other half of AC-4 - that `## Notes` carries old and new values with
/// AC-1/AC-2 figures either side - is prose about a decision and is checked at
/// review, not here.
#[test]
fn the_architecture_tuning_table_says_what_tuning_default_says() {
    let t = Tuning::default();
    let documented = architecture_tuning_table();

    let expected: [(&str, String); 9] = [
        ("uniform_tolerance", t.uniform_tolerance.to_string()),
        ("edge_threshold", t.edge_threshold.to_string()),
        ("min_content_stddev", t.min_content_stddev.to_string()),
        ("chrome_flat_fraction", t.chrome_flat_fraction.to_string()),
        ("chrome_max_extent", t.chrome_max_extent.to_string()),
        ("ambiguity_band", t.ambiguity_band.to_string()),
        ("min_content_fraction", t.min_content_fraction.to_string()),
        ("min_content_side", t.min_content_side.to_string()),
        ("margin_px", t.margin_px.to_string()),
    ];

    let mut wrong = Vec::new();
    for (name, code) in &expected {
        match documented.get(*name) {
            None => wrong.push(format!(
                "  {name}: no row in architecture.md's tuning table (the code says {code})"
            )),
            // The documented cell is written the way a person reads it - "0.30"
            // where the code's `0.3f32` prints as "0.3", "12" where `12.0f32`
            // prints as "12" - so the comparison is numeric, not textual.
            Some(doc) => match doc.parse::<f64>() {
                Ok(n) if same_number(n, code) => {}
                Ok(n) => wrong.push(format!(
                    "  {name}: architecture.md says {n}, Tuning::default() says {code}"
                )),
                Err(_) => wrong.push(format!(
                    "  {name}: architecture.md's Default cell starts with {doc:?}, \
                     which is not a number; the code says {code}"
                )),
            },
        }
    }

    assert!(
        wrong.is_empty(),
        "AC-4: docs/wiki/architecture.md's tuning table and `Tuning::default()` \
         disagree on {} of the {} constants. MC-019 is allowed to change a \
         default - with the corpus as the evidence - and the table has to move \
         with it, or the next agent reads a settled number that is not the one \
         compiled in.\n{}",
        wrong.len(),
        expected.len(),
        wrong.join("\n")
    );
}

/// Whether the number parsed out of the document is the number the code holds.
///
/// `code` arrives as the code value's own `to_string`, which is exact for the
/// integer fields and shortest-round-trip for the floats, so parsing it back
/// and comparing is an equality on the values rather than on their spelling.
fn same_number(documented: f64, code: &str) -> bool {
    code.parse::<f64>()
        .is_ok_and(|c| (documented - c).abs() < f64::EPSILON)
}
