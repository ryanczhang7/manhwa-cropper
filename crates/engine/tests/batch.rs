//! MC-011, AC-1 to AC-5: a batch runs every file and reports a run summary.
//!
//! `batch::run(&inputs, out_dir, &tuning, progress)` plans one output name per
//! input with `naming::plan_outputs` (MC-010, MC-020), processes the files on
//! the rayon pool (`docs/wiki/architecture.md` decision 12), keeps the results
//! in **input** order, never lets one file's failure stop the others, and
//! answers with a `RunSummary` whose JSON shape MC-012 writes to disk and
//! MC-015 renders.
//!
//! AC-6 - the timing claim - is not in this file. See the story's
//! `## Test plan`: it cannot be honoured under the `coverage` gate as the
//! criterion is worded, and that is escalated rather than decided here.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: the JSON shape in AC-5, read out of the criterion rather
//!   than re-derived - it is a contract with two stories that are not written
//!   yet, so there is nothing here to tune and every field, including every
//!   `null`, is pinned. Likewise `docs/wiki/architecture.md` decision 7's
//!   ` (2)` suffix, which
//!   [`two_inputs_with_the_same_file_name_both_reach_the_output_folder`]
//!   relies on without re-deciding.
//! * **Mechanical**: everything else. Which outcome each fixture earns was
//!   fixed by MC-008 and MC-009 and is asserted there as well; this file
//!   composes those answers and pins their order, their counts and their
//!   serialisation.
//! * **Measured**: the error strings a failure produces on this volume, and
//!   the completion order the five AC-1 inputs take when they run at the same
//!   time. Both are in the story's `## Handoff: RED -> GREEN`.
//!
//! # Why AC-1's five inputs do not all cost the same
//!
//! "Results in input order" has exactly one interesting failure mode:
//! collecting them in *completion* order. A batch whose inputs all take the
//! same time cannot see it, because the two orders coincide often enough to
//! pass. These five do not: measured sequentially they cost roughly
//! `[744, 704, 899, 312, 250] us` - the three screenshots are three times the
//! `.txt` - and run at the same time they finish in the order
//! `[notes.txt, flat.png, a.png, b.png, c.png]`, which is nothing like the
//! order they were given in.
//! [`results_are_one_per_input_in_input_order_with_the_outcome_each_input_earned`]
//! compares the whole vector, `input` paths included, so a summary built in
//! completion order fails on the first entry.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use cropper_core::{FlagReason, Tuning};
use cropper_engine::batch::{Progress, RunSummary, run};
use cropper_engine::{FileResult, Flag, Outcome};
use serde_json::json;

// --- Harness ----------------------------------------------------------------

/// The bytes the AC-4 blocker file holds: a real file, with the output folder
/// asked for *underneath* it, so the folder cannot be created.
const BLOCKER: &[u8] =
    b"MC-011 AC-4: a file, not a folder. Nothing may turn this into a directory.";

/// A temp directory, an input folder inside it and a path for the output
/// folder that **does not exist yet** - which is the state AC-4 describes and
/// the state a first run is really in.
fn workspace() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let src = tmp.path().join("in");
    fs::create_dir(&src).expect("an input dir");
    let out = tmp.path().join("out");
    (tmp, src, out)
}

/// A croppable screenshot PNG at `dir/name`: the MC-008 scene, greyscale 8.
fn shot(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    common::write_grey8(&common::screenshot(), &path);
    path
}

/// A wholly flat PNG at `dir/name`: what the detector flags `Uniform`.
fn flat(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    common::write_grey8(&common::uniform(), &path);
    path
}

/// A text file at `dir/name`: not an image at all, so `Unsupported`.
fn text(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    common::write_text(&path);
    path
}

/// 200 pseudo-random bytes at `dir/name`: a file that claims a format the
/// engine handles and cannot honour the claim, so `DecodeFailed`.
fn junk(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    common::write_random_bytes(&path);
    path
}

/// Run the batch and throw the progress away.
fn silent(inputs: &[PathBuf], out: &Path) -> RunSummary {
    run(inputs, out, &Tuning::default(), &|_: Progress| {})
}

/// Run the batch and keep every progress call, in the order it arrived.
///
/// The log is behind a `Mutex` because the callback is `Sync` and the pool
/// may call it from any worker; taking the lock is also what makes the
/// recorded order the call order.
fn recorded(inputs: &[PathBuf], out: &Path) -> (RunSummary, Vec<Progress>) {
    let seen = Mutex::new(Vec::new());
    let summary = run(inputs, out, &Tuning::default(), &|p: Progress| {
        seen.lock()
            .expect("the progress log is not poisoned")
            .push(p);
    });
    let seen = seen.into_inner().expect("the progress log is not poisoned");
    (summary, seen)
}

/// The file names in `dir`, sorted - a failure message that reads like a
/// directory listing rather than a set of absolute temp paths.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .map(|entry| {
            entry
                .expect("a directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// A path as the JSON carries it: `PathBuf`'s own `Serialize` writes the
/// string, so this is what AC-5's `input` and `output` must equal.
fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// The `output` path of a result, or a panic naming what was found instead.
fn output_of(result: &FileResult) -> &Path {
    match &result.outcome {
        Outcome::Cropped { output, .. } | Outcome::Flagged { output, .. } => output,
        Outcome::Failed { error } => panic!(
            "{} was expected to produce an output and failed: {error}",
            result.input.display()
        ),
    }
}

// --- AC-1: five inputs, five results, in input order ------------------------

#[test]
fn results_are_one_per_input_in_input_order_with_the_outcome_each_input_earned() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "a.png"),
        shot(&src, "b.png"),
        shot(&src, "c.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
    ];

    let summary = silent(&inputs, &out);

    let cropped = |input: &PathBuf, name: &str| FileResult {
        input: input.clone(),
        outcome: Outcome::Cropped {
            rect: common::crop_rect(),
            output: out.join(name),
        },
    };
    assert_eq!(
        summary.results,
        [
            cropped(&inputs[0], "a.png"),
            cropped(&inputs[1], "b.png"),
            cropped(&inputs[2], "c.png"),
            FileResult {
                input: inputs[3].clone(),
                outcome: Outcome::Flagged {
                    reason: Flag::Detector(FlagReason::Uniform),
                    output: out.join("flat.png"),
                },
            },
            FileResult {
                input: inputs[4].clone(),
                outcome: Outcome::Flagged {
                    reason: Flag::Unsupported,
                    output: out.join("notes.txt"),
                },
            },
        ],
        "AC-1: five inputs give five results in INPUT order - three Cropped, \
         then Flagged(Detector(Uniform)), then Flagged(Unsupported). These five \
         do not cost the same, so a batch that collects results as workers \
         finish returns them roughly reversed (measured: notes.txt, flat.png, \
         a.png, b.png, c.png)"
    );
}

#[test]
fn the_counts_are_three_cropped_two_flagged_and_none_failed() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "a.png"),
        shot(&src, "b.png"),
        shot(&src, "c.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
    ];

    let summary = silent(&inputs, &out);

    assert_eq!(summary.cropped(), 3, "AC-1: three croppable screenshots");
    assert_eq!(
        summary.flagged(),
        2,
        "AC-1: the uniform PNG and the .txt are flagged, not failed - both \
         reached the output folder"
    );
    assert_eq!(
        summary.failed(),
        0,
        "AC-1: nothing failed; a flag is an answer, not a failure"
    );
}

// --- AC-2: one input does not exist -----------------------------------------

#[test]
fn a_missing_input_is_the_only_failure_and_the_files_after_it_are_still_processed() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "a.png"),
        src.join("gone.png"),
        shot(&src, "b.png"),
        flat(&src, "flat.png"),
    ];

    let summary = silent(&inputs, &out);

    match &summary.results[1].outcome {
        Outcome::Failed { error } => assert!(
            !error.trim().is_empty(),
            "AC-2: a Failed carries what went wrong, for the run summary; got \
             an empty string"
        ),
        other => panic!("AC-2: a missing input is Failed, not {other:?}"),
    }
    assert_eq!(
        summary.failed(),
        1,
        "AC-2: exactly one input failed, and it is the one that does not exist"
    );
    assert_eq!(
        summary.cropped(),
        2,
        "AC-2: the failure did not stop the inputs after it - b.png was still \
         cropped"
    );
    assert_eq!(summary.flagged(), 1, "AC-2: flat.png was still flagged");
    assert_eq!(
        entries(&out),
        ["a.png", "b.png", "flat.png"],
        "AC-2: every input that could be processed reached the output folder, \
         and the one that could not left nothing behind"
    );
}

// --- AC-3: progress, once per completed file --------------------------------

#[test]
fn progress_counts_one_to_n_and_every_call_carries_the_batch_total() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "a.png"),
        shot(&src, "b.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
        src.join("gone.png"),
    ];

    let (_summary, seen) = recorded(&inputs, &out);

    let want: Vec<Progress> = (1..=5).map(|done| Progress { done, total: 5 }).collect();
    assert_eq!(
        seen, want,
        "AC-3: five inputs give five calls, each carrying total == 5, with \
         done running 1, 2, 3, 4, 5. One call at the end gives a log of length \
         one; a counter read after the batch gives five calls all reading \
         (5, 5); a callback handed its own index as the total gives \
         (1, 1), (2, 2), ... - none of them equals this"
    );
}

#[test]
fn a_batch_of_one_reports_one_result_and_one_progress_call() {
    let (_tmp, src, out) = workspace();
    let inputs = [flat(&src, "flat.png")];

    let (summary, seen) = recorded(&inputs, &out);

    assert_eq!(summary.results.len(), 1, "AC-3: one input, one result");
    assert_eq!(
        seen,
        [Progress { done: 1, total: 1 }],
        "AC-3: the smallest batch still reports (1 of 1) once - a progress bar \
         that only moves for the second file never moves for a single file"
    );
}

#[test]
fn an_empty_batch_reports_nothing_and_never_calls_progress() {
    let (_tmp, _src, out) = workspace();

    let (summary, seen) = recorded(&[], &out);

    assert!(
        summary.results.is_empty(),
        "AC-3: no inputs, no results; got {:?}",
        summary.results
    );
    assert_eq!(summary.cropped(), 0);
    assert_eq!(summary.flagged(), 0);
    assert_eq!(summary.failed(), 0);
    assert!(
        seen.is_empty(),
        "AC-3: no inputs, no progress calls - not one call reading (0, 0); got \
         {seen:?}"
    );
}

// --- AC-4: the output folder ------------------------------------------------

#[test]
fn an_output_folder_that_does_not_exist_is_created_and_holds_every_output() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "a.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
    ];
    assert!(
        !out.exists(),
        "the fixture is wrong: AC-4 needs an output folder that does not exist"
    );

    let summary = silent(&inputs, &out);

    assert!(
        out.is_dir(),
        "AC-4: the output folder is created; {} is still missing",
        out.display()
    );
    assert_eq!(
        entries(&out),
        ["a.png", "flat.png", "notes.txt"],
        "AC-4: and every output landed in it"
    );
    assert_eq!(summary.failed(), 0, "AC-4: nothing failed on the way");
}

#[test]
fn an_output_folder_that_cannot_be_created_fails_every_result_without_panicking() {
    let (tmp, src, _out) = workspace();
    // A real file, with the output folder asked for underneath it. Nothing can
    // create `blocker/out`, because `blocker` is not a directory.
    let blocker = tmp.path().join("blocker");
    fs::write(&blocker, BLOCKER).expect("the blocker file");
    let out = blocker.join("out");

    let inputs = [
        shot(&src, "a.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
        junk(&src, "junk.png"),
        src.join("gone.png"),
    ];

    // Reaching the assertions at all is the "nothing panics" half: a panic in
    // a worker is resumed on this thread by the pool and fails the test.
    let (summary, seen) = recorded(&inputs, &out);

    assert_eq!(
        summary.results.len(),
        5,
        "AC-4: an unusable output folder still reports one result per input - \
         not an early return with an empty summary"
    );
    let not_failed: Vec<&FileResult> = summary
        .results
        .iter()
        .filter(|r| !matches!(r.outcome, Outcome::Failed { .. }))
        .collect();
    assert!(
        not_failed.is_empty(),
        "AC-4: nothing can be written under a path that is a file, so every \
         result is Failed; these were not: {not_failed:?}"
    );
    assert_eq!(
        (summary.cropped(), summary.flagged(), summary.failed()),
        (0, 0, 5),
        "AC-4: and the counts say so"
    );
    assert_eq!(
        seen.len(),
        5,
        "AC-3 and AC-4: a file that failed is a file that completed, so \
         progress is still reported once for each of the five"
    );
    assert_eq!(
        fs::read(&blocker).expect("the blocker file is still readable"),
        BLOCKER,
        "AC-4: and the file that was in the way was not clobbered on the way \
         through"
    );
}

// --- AC-5: the JSON shape MC-012 writes and MC-015 renders ------------------

#[test]
fn the_summary_serialises_to_the_json_shape_mc012_writes_and_mc015_renders() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "shot.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
        junk(&src, "junk.png"),
        src.join("gone.png"),
    ];

    let summary = silent(&inputs, &out);

    // The one field that is the operating system's words rather than this
    // program's: read it out, so that everything around it can be pinned
    // exactly. That it is a non-empty string is asserted here; what it says
    // is not this story's business.
    let error = match &summary.results[4].outcome {
        Outcome::Failed { error } => error.clone(),
        other => panic!("AC-5: the missing input is Failed, not {other:?}"),
    };
    assert!(
        !error.trim().is_empty(),
        "AC-5: a failed result carries a non-empty error string"
    );

    let rect = common::crop_rect();
    assert_eq!(
        serde_json::to_value(&summary).expect("a RunSummary serialises"),
        json!({
            "cropped": 1,
            "flagged": 3,
            "failed": 1,
            "results": [
                {
                    "input": path_string(&inputs[0]),
                    "outcome": "cropped",
                    "output": path_string(&out.join("shot.png")),
                    "rect": { "x": rect.x, "y": rect.y, "w": rect.w, "h": rect.h },
                    "reason": null,
                    "error": null
                },
                {
                    "input": path_string(&inputs[1]),
                    "outcome": "flagged",
                    "output": path_string(&out.join("flat.png")),
                    "rect": null,
                    "reason": "Uniform",
                    "error": null
                },
                {
                    "input": path_string(&inputs[2]),
                    "outcome": "flagged",
                    "output": path_string(&out.join("notes.txt")),
                    "rect": null,
                    "reason": "Unsupported",
                    "error": null
                },
                {
                    "input": path_string(&inputs[3]),
                    "outcome": "flagged",
                    "output": path_string(&out.join("junk.png")),
                    "rect": null,
                    "reason": "DecodeFailed",
                    "error": null
                },
                {
                    "input": path_string(&inputs[4]),
                    "outcome": "failed",
                    "output": null,
                    "rect": null,
                    "reason": null,
                    "error": error
                }
            ]
        }),
        "AC-5: the shape is a contract with MC-012, which writes it to disk, \
         and MC-015, which renders it. `reason` is the FLAG VARIANT NAME and \
         nothing else: `Flag::Detector(FlagReason::Uniform)` is the string \
         \"Uniform\", not {{\"Detector\": \"Uniform\"}}, and \
         `Flag::DecodeFailed(msg)` is the string \"DecodeFailed\", not \
         {{\"DecodeFailed\": msg}} - the engine's Flag is one level deeper \
         than this JSON is. Every entry carries all six keys, the nulls \
         included: an omitted key is a different shape from a null one"
    );
}

#[test]
fn flagged_names_are_the_flagged_inputs_file_names_in_input_order() {
    let (_tmp, src, out) = workspace();
    let inputs = [
        shot(&src, "shot.png"),
        flat(&src, "flat.png"),
        text(&src, "notes.txt"),
        junk(&src, "junk.png"),
        src.join("gone.png"),
    ];

    let summary = silent(&inputs, &out);

    assert_eq!(
        summary.flagged_names(),
        ["flat.png", "notes.txt", "junk.png"],
        "AC-5: the flagged inputs' FILE NAMES, in input order - not their \
         whole paths, not the cropped one, and not the one that failed. \
         MC-015 puts each of these in a row of the result list"
    );
}

// --- The planner, composed (MC-010, MC-020) ---------------------------------

#[test]
fn two_inputs_with_the_same_file_name_both_reach_the_output_folder() {
    let (tmp, _src, out) = workspace();
    let x = tmp.path().join("x");
    let y = tmp.path().join("y");
    fs::create_dir(&x).expect("folder x");
    fs::create_dir(&y).expect("folder y");
    let inputs = [shot(&x, "a.png"), shot(&y, "a.png")];

    let summary = silent(&inputs, &out);

    assert_eq!(
        [
            output_of(&summary.results[0]),
            output_of(&summary.results[1])
        ],
        [out.join("a.png"), out.join("a (2).png")],
        "MC-011 plans the whole batch's names before it writes any of them \
         (MC-010, decision 7), so two inputs called a.png are two files"
    );
    assert_eq!(
        entries(&out),
        ["a (2).png", "a.png"],
        "and both of them are really on disk - this is the case the parallel \
         writes would lose if each worker chose its own name"
    );
}

// --- Out of scope: a folder is an input that fails ---------------------------

#[test]
fn a_folder_given_as_an_input_is_failed_rather_than_recursed_into() {
    let (_tmp, src, out) = workspace();
    let folder = src.join("pages");
    fs::create_dir(&folder).expect("a folder to hand in as an input");
    shot(&folder, "inside.png");
    let inputs = [shot(&src, "a.png"), folder];

    let summary = silent(&inputs, &out);

    assert!(
        matches!(summary.results[1].outcome, Outcome::Failed { .. }),
        "Out of scope: recursion into folders. A directory handed in as an \
         input is one Failed result, not a second batch; got {:?}",
        summary.results[1].outcome
    );
    assert_eq!(
        summary.results.len(),
        2,
        "the folder contributed one result, not one per file inside it"
    );
    assert_eq!(
        entries(&out),
        ["a.png"],
        "and nothing from inside the folder was written out"
    );
}
