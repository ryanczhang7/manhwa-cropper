//! MC-011, AC-1 to AC-6: a batch runs every file and reports a run summary.
//!
//! `batch::run(&inputs, out_dir, &tuning, progress)` plans one output name per
//! input with `naming::plan_outputs` (MC-010, MC-020), processes the files on
//! the rayon pool (`docs/wiki/architecture.md` decision 12), keeps the results
//! in **input** order, never lets one file's failure stop the others, and
//! answers with a `RunSummary` whose JSON shape MC-012 writes to disk and
//! MC-015 renders.
//!
//! AC-6 was amended on 2026-09-12, from a wall-time comparison to an
//! observation that the work happened on more than one thread. The old
//! wording could not be satisfied under the `coverage` gate by any correct
//! implementation, and was a 20-80% control where it did run; the
//! reproduction, the numbers and the approval are in the story's
//! `## Amendments`.
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

use std::collections::HashSet;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread::ThreadId;

use cropper_core::{FlagReason, Luma, Tuning};
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

// --- AC-6: the batch is parallel --------------------------------------------
//
// Amended on 2026-09-12 from a wall-time comparison to a thread observation;
// the reproduction and the reasons are in the story's `## Amendments`. In
// short: `coverage` runs this suite under `cargo llvm-cov`, whose counters are
// process-global and uninstrumented for concurrency, so a parallel batch pays
// an order of magnitude more for them than a sequential one and the old
// comparison inverted. A `ThreadId` is not a duration and instrumentation
// cannot change how many of them there were.

/// How many inputs AC-6 hands the batch.
const AC6_INPUTS: usize = 32;

/// How much bigger AC-6's fixture is than the MC-008 scene.
///
/// Not a threshold and not a performance target: it is how much work each of
/// the 32 items carries, and the only thing it buys is that the pool has a
/// reason to spread them rather than let one worker drain the queue.
///
/// **1x was clean too** - 50 runs of this file at each scale, no failure at
/// either - so this is margin and not a fix for a measured flake. It is here
/// because 50 clean runs do not exclude a one-in-fifty rate, the failure mode
/// it insures against is likelier the cheaper an item is, and 3x buys ~4.6 ms
/// of real work per item against 1x's ~1 ms for 0.05 s of test time. If that
/// ever needs revisiting, revisit it with a measurement; do not lower it to
/// make something pass.
const AC6_SCALE: u32 = 3;

/// The MC-008 screenshot scene at `k` times its size, nearest neighbour: the
/// same four borders, chrome band and art, every pixel a `k x k` block, and
/// still `Cropped` at `Tuning::default()`.
fn upscaled(k: u32) -> Luma {
    let base = common::screenshot();
    let (width, height) = (base.width * k, base.height * k);
    let mut data = vec![0u8; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            data[(y * width + x) as usize] = base.data[((y / k) * base.width + (x / k)) as usize];
        }
    }
    Luma {
        width,
        height,
        data,
    }
}

#[test]
fn the_thirty_two_files_are_processed_on_more_than_one_thread() {
    // The one skip this story authorises, and it is the criterion's own: on a
    // single-core machine there is no second thread to observe and nothing to
    // assert. It is conditioned on the core count and on nothing else - if
    // this assertion fails on a machine with two or more cores, that is a real
    // failure and not a reason to widen this branch.
    let cores = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    if cores < 2 {
        eprintln!(
            "MC-011 AC-6 skipped: available_parallelism() reports {cores} core, and the \
             criterion is checked only on a machine with more than one. Nothing about \
             the batch was asserted here."
        );
        return;
    }

    let (_tmp, src, out) = workspace();
    let plane = upscaled(AC6_SCALE);
    let inputs: Vec<PathBuf> = (0..AC6_INPUTS)
        .map(|i| {
            let path = src.join(format!("s{i:02}.png"));
            common::write_grey8(&plane, &path);
            path
        })
        .collect();

    // The progress callback is the only hook a caller has inside the run, so
    // it is where the thread is read. AC-3 makes the calls serial - they are
    // made under one lock, in order - but serialising *when* they happen does
    // not move them: each one is still made by the worker that finished the
    // file. What this records, then, is the set of threads the work was done
    // on.
    let seen: Mutex<HashSet<ThreadId>> = Mutex::new(HashSet::new());
    let summary = run(&inputs, &out, &Tuning::default(), &|_: Progress| {
        seen.lock()
            .expect("the thread log is not poisoned")
            .insert(std::thread::current().id());
    });
    let seen = seen.into_inner().expect("the thread log is not poisoned");

    // A fixture guard, not a criterion: 32 items that all failed in a
    // microsecond would give the pool no reason to spread them, and the
    // assertion below would be measuring nothing.
    assert_eq!(
        summary.results.len(),
        AC6_INPUTS,
        "AC-6: {AC6_INPUTS} inputs in"
    );
    assert_eq!(
        summary.failed(),
        0,
        "the fixture is wrong: AC-6 needs {AC6_INPUTS} inputs that are really \
         cropped, so that each item carries real work. A batch of failures \
         costs a microsecond each and gives the pool no reason to spread them"
    );

    assert!(
        seen.len() >= 2,
        "AC-6: {AC6_INPUTS} files were processed on {} thread(s) - {seen:?} - so \
         this batch is not parallel (decision 12). Two things produce exactly \
         one thread here, and the difference matters: a `run` that iterates \
         the inputs itself instead of handing them to the pool, or a `run` \
         that is parallel but reports progress from the calling thread after \
         the pool has finished. The second also breaks what AC-3's callback is \
         for - MC-015's progress bar has to move while the run is happening, \
         not once at the end",
        seen.len()
    );
}
