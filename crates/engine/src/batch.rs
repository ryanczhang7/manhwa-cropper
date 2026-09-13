//! Many files in, one summary out (MC-011): the whole of a run.
//!
//! [`run`] is the engine's top-level entry - the thing the GUI (MC-015) and
//! the headless CLI (MC-012) call once per user action. It does four things
//! and delegates everything else:
//!
//! 1. plans one output path per input with
//!    [`naming::plan_outputs`](crate::naming::plan_outputs) - **once, before
//!    anything is written**, because the writes below happen at the same
//!    instant and "find a free name, then write it" inside a worker loses a
//!    file (MC-010, MC-020);
//! 2. creates `out_dir`, or leaves the failure to be reported per file;
//! 3. hands the pairs to the rayon pool
//!    (`docs/wiki/architecture.md` decision 12) and lets
//!    [`process_file`](crate::process_file) do the per-file work;
//! 4. reports progress as each file completes, and collects the results **in
//!    input order**.
//!
//! Nothing here can fail: `process_file` reports rather than unwinds, so one
//! unreadable file is one [`Outcome::Failed`](crate::Outcome::Failed) in the
//! middle of a summary that still describes every other input (AC-2, AC-4).
//!
//! # Why progress is counted under a lock, and called under the same one
//!
//! [`Progress`] is a count out of a total, never a file name, so the bar
//! MC-015 draws never has to explain which file is where (decision 12). The
//! count is a `Mutex<usize>` rather than an `AtomicUsize`, and the callback is
//! invoked *while that lock is held*, which is the difference between a
//! sequence a caller can rely on and one that is usually right: with
//! `fetch_add` and an unsynchronised call, two workers can take 1 and 2 and
//! then call in the order 2, 1. Holding the lock across the call makes the
//! observed sequence `1..=n` by construction (AC-3). The callback is cheap by
//! contract - it moves a progress bar - so the lock costs a batch nothing it
//! can measure.
//!
//! # Why the worker makes the call, and not this thread afterwards
//!
//! The callback is the only hook a caller has *inside* a run. Collecting every
//! result and then firing all `n` calls from here would produce a
//! byte-identical log - `(1, n) .. (n, n)` either way - and a progress bar
//! that fills instantly when the run is already over. So each call is made by
//! the worker that finished the file, at the moment it finished it, which is
//! what "called once per completed file" means (MC-011 AC-3, AC-6).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use cropper_core::{FlagReason, Rect, Tuning};
use rayon::prelude::*;
use serde::{Serialize, Serializer};

use crate::naming;
use crate::process::{FileResult, Flag, Outcome, process_file};

/// How far along a run is: `done` files of `total` are finished.
///
/// A count and not a name, per `docs/wiki/architecture.md` decision 12. One
/// of these is handed to the callback once per completed file, with `done`
/// running `1..=total` (MC-011 AC-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// How many files have finished, counting from 1.
    pub done: usize,
    /// How many files the run was given, fixed for the whole run.
    pub total: usize,
}

/// What became of a whole run: one [`FileResult`] per input, in input order.
///
/// The counts are derived rather than stored - there is one place a result's
/// outcome is decided, and it is [`process_file`] - and the JSON the
/// `Serialize` impl writes is the contract MC-012 saves to disk and MC-015
/// renders.
#[derive(Debug)]
pub struct RunSummary {
    /// One result per input, in the order the inputs were given.
    pub results: Vec<FileResult>,
}

impl RunSummary {
    /// How many inputs were cropped.
    #[must_use]
    pub fn cropped(&self) -> usize {
        self.count(|outcome| matches!(outcome, Outcome::Cropped { .. }))
    }

    /// How many inputs were flagged for a person to look at. A flag is an
    /// answer, not a failure: a flagged file reached the output folder
    /// unchanged.
    #[must_use]
    pub fn flagged(&self) -> usize {
        self.count(|outcome| matches!(outcome, Outcome::Flagged { .. }))
    }

    /// How many inputs produced no output at all.
    #[must_use]
    pub fn failed(&self) -> usize {
        self.count(|outcome| matches!(outcome, Outcome::Failed { .. }))
    }

    /// The flagged inputs' **file names**, in input order: one row each in
    /// MC-015's result list, and the list MC-012 prints.
    ///
    /// Names rather than whole paths, because the user chose these files and
    /// knows where they came from; the whole path is in `results`.
    #[must_use]
    pub fn flagged_names(&self) -> Vec<String> {
        self.results
            .iter()
            .filter(|result| matches!(result.outcome, Outcome::Flagged { .. }))
            .map(|result| file_name(&result.input))
            .collect()
    }

    /// How many results' outcomes satisfy `is`.
    fn count(&self, is: impl Fn(&Outcome) -> bool) -> usize {
        self.results
            .iter()
            .filter(|result| is(&result.outcome))
            .count()
    }
}

/// Process every input into `out_dir` and report what became of each.
///
/// The results are one per input, in input order, however the pool happened
/// to schedule them. One file's failure never stops another: every input is
/// attempted and every input gets a result. `progress` is called once per
/// completed file - including a failed one, which is a file that completed -
/// by the worker that finished it, with `done` running `1..=inputs.len()`.
///
/// `out_dir` is created if it does not exist. If it cannot be created, that
/// is not an error `run` can report - it reports per file - so every result
/// is [`Failed`](Outcome::Failed) with the reason the writer saw.
#[must_use]
pub fn run(
    inputs: &[PathBuf],
    out_dir: &Path,
    tuning: &Tuning,
    progress: &(dyn Fn(Progress) + Sync),
) -> RunSummary {
    // Every name, planned in one sequential pass before any worker starts:
    // see the module docs, and `naming`'s.
    let outputs = naming::plan_outputs(out_dir, inputs);
    // Decision 6: the output folder is created on demand. The error is
    // deliberately dropped - `run` has nowhere to report it and nothing to
    // report it *about*, since the same failure reaches each writer and
    // becomes that file's own `Outcome::Failed` (AC-4). The writers create it
    // too, so this call is the criterion's "created when the run starts" and
    // not the only chance.
    let _ = fs::create_dir_all(out_dir);

    let total = inputs.len();
    let done = Mutex::new(0usize);
    RunSummary {
        // `zip` over two indexed parallel iterators, collected into a `Vec`:
        // rayon puts each result back at its own index, so the order is the
        // inputs' however the pool scheduled them (AC-1).
        results: inputs
            .par_iter()
            .zip(outputs.par_iter())
            .map(|(input, output)| {
                let result = process_file(input, output, tuning);
                // Increment and report under one lock, on the worker that did
                // the work. Both halves of that are load-bearing; the module
                // docs say why. A poisoned counter means the caller's own
                // callback panicked on another worker, which rayon is already
                // carrying back to them - recovering here keeps that first
                // panic as the one they see.
                let mut done = done.lock().unwrap_or_else(PoisonError::into_inner);
                *done += 1;
                progress(Progress { done: *done, total });
                result
            })
            .collect(),
    }
}

/// Write `summary` to `path` as the JSON document below - what `--summary`
/// asks for (MC-012 AC-1, AC-2).
///
/// Serialised with `to_vec_pretty`, the way `settings.json` is: the file is a
/// script's input and a person's, and neither is served by one long line. The
/// file is replaced whole, and `path` is taken exactly as given - a summary
/// is one named file, not a name this function gets to choose.
///
/// # Errors
///
/// If the document cannot be serialised, or `path` cannot be written -
/// a folder that is not there, say.
pub fn write_summary(summary: &RunSummary, path: &Path) -> io::Result<()> {
    let document = serde_json::to_vec_pretty(summary).map_err(io::Error::other)?;
    fs::write(path, document)
}

/// `path`'s file name, or the whole path when it has none (`..`, a bare
/// root). Lossy rather than fallible: a name this program cannot decode is
/// still a name the user has to be shown.
fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

// --- The wire shape (MC-011 AC-5) -------------------------------------------
//
// Flatter than the types, and deliberately so: `Outcome` carries an `output`
// on two variants and an `error` on the third, while the JSON carries all six
// keys on every entry with the ones that do not apply written as `null`. An
// omitted key is a different shape from a null one, and MC-015 renders a
// table - so a derive on `Outcome` would not do, and the shadow structs below
// are what the summary is really serialised through.

impl Serialize for RunSummary {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SummaryJson {
            cropped: self.cropped(),
            flagged: self.flagged(),
            failed: self.failed(),
            results: self.results.iter().map(ResultJson::from).collect(),
        }
        .serialize(serializer)
    }
}

/// A whole run on the wire.
#[derive(Serialize)]
struct SummaryJson<'a> {
    cropped: usize,
    flagged: usize,
    failed: usize,
    results: Vec<ResultJson<'a>>,
}

/// One result on the wire: six keys, always.
///
/// Paths are serialised [lossily](std::ffi::OsStr::to_string_lossy) rather
/// than through `Path`'s own `Serialize`, which fails on a name that is not
/// valid Unicode. A summary that cannot be written because one of the user's
/// files is named oddly would lose the report of every other file with it.
#[derive(Serialize)]
struct ResultJson<'a> {
    input: std::borrow::Cow<'a, str>,
    outcome: &'static str,
    output: Option<std::borrow::Cow<'a, str>>,
    rect: Option<Rect>,
    reason: Option<ReasonName<'a>>,
    error: Option<&'a str>,
}

impl<'a> From<&'a FileResult> for ResultJson<'a> {
    fn from(result: &'a FileResult) -> Self {
        let (outcome, output, rect, reason, error) = match &result.outcome {
            Outcome::Cropped { rect, output } => ("cropped", Some(output), Some(*rect), None, None),
            Outcome::Flagged { reason, output } => (
                "flagged",
                Some(output),
                None,
                Some(ReasonName::from(reason)),
                None,
            ),
            Outcome::Failed { error } => ("failed", None, None, None, Some(error.as_str())),
        };
        Self {
            input: result.input.to_string_lossy(),
            outcome,
            output: output.map(|path| path.to_string_lossy()),
            rect,
            reason,
            error,
        }
    }
}

/// `reason` on the wire: the flag's variant name, and nothing else.
///
/// The engine's [`Flag`] is one level deeper than this JSON is -
/// `Detector` wraps the detector's own reason and `DecodeFailed` carries the
/// decoder's message - so a derive on `Flag` would write `{"Uniform": null}`
/// or `{"DecodeFailed": "..."}` where the criterion asks for `"Uniform"` and
/// `"DecodeFailed"`. Untagged is what discards that outer level: a detector
/// reason serialises as `FlagReason`'s own derive, which for a unit variant
/// is the bare name.
#[derive(Serialize)]
#[serde(untagged)]
enum ReasonName<'a> {
    /// The detector's four, named by `cropper-core`.
    Detector(&'a FlagReason),
    /// The engine's two, whose payload the wire shape drops.
    Engine(&'static str),
}

impl<'a> From<&'a Flag> for ReasonName<'a> {
    fn from(flag: &'a Flag) -> Self {
        match flag {
            Flag::Detector(reason) => Self::Detector(reason),
            Flag::Unsupported => Self::Engine("Unsupported"),
            Flag::DecodeFailed(_) => Self::Engine("DecodeFailed"),
        }
    }
}
