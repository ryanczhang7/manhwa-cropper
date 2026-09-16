//! MC-024: what a **real** run tells the window about itself while it is
//! still running.
//!
//! # The gap this file closes
//!
//! `docs/wiki/audits/app-window-2026-09-13.md`, evidence E-5, found that
//! `shell.rs` has exactly two surviving mutants and that both are in
//! `fn count` - the `usize` -> `u32` conversion `ThreadRunner` applies to the
//! engine's `batch::Progress` on its way to `Event::Progress`. Nothing
//! executes it: the eleven fake-runner tests in `tests/shell.rs` push
//! `Event::Progress` down the channel with counts of their own, and the one
//! test that uses a real `ThreadRunner`
//! (`a_real_run_of_two_screenshots_writes_both_crops_and_reports_them`) polls
//! until the model *leaves* `Processing` and only then asserts. It never
//! observes a `Progress` at all, so `count -> 0` - a motionless `0 of 0` for
//! the whole of every real run, including the Explorer "Send to" launch -
//! survives it.
//!
//! So this file does the one thing that suite does not: it starts a real
//! `ThreadRunner` over real files and **reads the counts the window showed
//! while the run was under way**.
//!
//! # Why this is a separate file from `tests/shell.rs`
//!
//! Three reasons, in order of weight.
//!
//! 1. `tests/shell.rs` is MC-016's and states its own contract in its header,
//!    including "**Measured**: nothing is an expectation here". Half of this
//!    file *is* a measurement - [`FILES`] is a number chosen from a timing run
//!    and defended by one - so it does not belong under that sentence.
//!    `tests/truncation.rs` (MC-022) is the precedent for a second file with a
//!    contract of its own.
//! 2. A test binary is its own process, and `MANHWA_CROPPER_CONFIG_DIR` is
//!    process-wide. Keeping this file's three long runs out of MC-016's
//!    process keeps MC-016's eleven fast tests from queueing behind them on
//!    that binary's environment lock.
//! 3. Nothing in `tests/shell.rs` changes. Its two-file real run stays exactly
//!    as MC-016 wrote it.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled elsewhere, read out rather than re-derived**: that the engine
//!   calls its progress callback once per completed file with `done` running
//!   `1..=total` (`batch::run`'s doc, MC-011 AC-3); that `Shell::drain`
//!   dispatches every buffered event of a frame in that frame; that
//!   `Model::files_dropped` sets `Processing { done: 0, total: N }` from its
//!   own `u32::try_from` before any `Progress` can arrive (MC-014, audit E-2).
//!   That last one is why [`assert_a_frame_showed_the_running_batch`] demands
//!   `done > 0` as well as `total == N`.
//! * **Mechanical**: every assertion. Each is a predicate over the counts the
//!   window actually showed, in frame order.
//! * **Measured**: [`FILES`] and [`POLL`], and nothing else. The numbers and
//!   the reasoning are on [`FILES`]; the story's
//!   `## Handoff: RED -> GREEN` carries the full table.
//!
//! # Why the test that matters here is green on arrival, and what earns it
//!
//! `count` is not new: it ships today and it is correct, so the run below
//! reports its counts correctly today and the assertions pass the first time
//! they are run. A test that has never been observed to fail is not a test
//! (`.claude/harness/rules.md`), so two things earn these assertions their
//! place.
//!
//! * **The two negative controls at the bottom of this file.** They drive the
//!   same window and the same observation loop with [`MutantRunner`] - which
//!   is `ThreadRunner`'s body with `shell::count` replaced by a substitute -
//!   and watch the *same assertion functions the real test calls* reject the
//!   result. `count -> 0` and `count -> 1` are the audit's two survivors, so
//!   the controls are the survivors, injected at the runner instead of at the
//!   function, where a test may reach them.
//! * **MC-024 AC-4, in GREEN.** The controls prove the assertions
//!   discriminate; they cannot prove `shell::count` is the thing wired into
//!   the path. That is what AC-4's real mutation of `crates/app/src/shell.rs`
//!   proves, and it is recorded in the story's `## Gate probes`.
//!
//! A control that never observed a converted count would reject the trace for
//! the wrong reason - an empty trace fails the same assertion - so each
//! control first asserts that the mutated count *was* observed. Without that
//! line the pair is decoration.
//!
//! # Why every test here takes the environment lock
//!
//! Two process-wide resources, one lock.
//!
//! * `MANHWA_CROPPER_CONFIG_DIR`, for the reason `tests/shell.rs` gives at
//!   length: every test here builds a `Shell`, and a `Shell` is the thing that
//!   can write `settings.json`. Nothing below chooses a folder, so nothing
//!   below *should* write one; the guard is containment rather than an
//!   assertion, and it is what keeps a defect from reaching the developer's
//!   real `%APPDATA%`.
//! * The panic hook, which [`failure_of`] silences while it watches an
//!   assertion fail. A hook is global to the process, and a hook swapped in
//!   under one test would swallow another test's panic message.

#[path = "../../engine/tests/common/mod.rs"]
mod common;

use std::any::Any;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::panic::{self, UnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};
use std::{env, fs, thread};

use cropper_engine::Tuning;
use cropper_engine::batch::{self, Progress, RunSummary};
use cropper_engine::settings::Settings;
use eframe::egui;
use egui_kittest::Harness;
use manhwa_cropper::shell::{BatchRunner, FolderPicker, Shell, ThreadRunner};
use manhwa_cropper::{AppState, Event, Model};

// --- The measured numbers ----------------------------------------------------

/// How many images the run below is given.
///
/// **This number is a measurement, not a taste.** `Shell::drain` applies every
/// event buffered since the last frame *in* that frame, so a run that finishes
/// between two polls collapses its whole progress history and its `Finished`
/// into one frame and no count from the running batch is ever seen. What
/// decides whether that happens is the **gap**: the wall time between the
/// first `Progress` reaching the channel and `Finished` reaching it. Only a
/// stall of the GUI thread longer than the whole gap can lose the
/// observation, so the gap - not the number of polls - is the margin.
///
/// The gap is roughly `run time * (1 - cores / FILES)`: rayon finishes files
/// in waves of one per core, and with `FILES <= cores` every `Progress` and
/// the `Finished` land in a single wave. That is exactly why MC-016's
/// two-file run never sees one. So `FILES` has to be several times the core
/// count, and it was chosen by measuring rather than by arithmetic.
///
/// Measured on this machine (Windows, 12 logical cores, `profile.test`
/// `opt-level = 2`), 20 consecutive runs at each size, counting frames whose
/// count satisfied [`assert_a_frame_showed_the_running_batch`]:
///
/// | files | qualifying frames | gap (ms) | run (ms) | setup (ms) |
/// |---|---|---|---|---|
/// | 24 | 1..5 | 5.7..30.7 | 14.1..36.6 | 10.4..19.1 |
/// | 48 | 4..7 | 22.6..40.8 | 28.5..46.8 | 21.5..36.5 |
/// | **96** | **5..13** | **45.9..80.9** | **56.1..97.2** | 43.9..775.1 |
/// | 144 | 7..13 | 72.6..104.1 | 79.9..111.9 | 71.1..785.3 |
///
/// Those two setup maxima are outliers - one run in twenty at each size -
/// and they are filesystem jitter while writing the images, not the run.
/// Setup happens before the drop, so it costs wall time and buys no margin;
/// the typical figure is 44-95 ms. It is the reason not to go past 96.
///
/// 96 is the smallest of those whose worst gap is an order of magnitude
/// clear of a Windows scheduling quantum: losing the observation would take a
/// 45 ms stall of the GUI thread. 48 halves the cost and halves the margin to
/// 22 ms, which is one bad quantum; 144 doubles the setup for a margin
/// nothing needs. Zero qualifying frames occurred in **0 of 80** measured
/// runs across all four sizes.
///
/// The `coverage` gate runs this instrumented, and instrumentation makes the
/// margin **larger**, not smaller: at 96 files, 20 runs under
/// `cargo llvm-cov` gave 9..30 qualifying frames, a 139.8..194.1 ms gap and a
/// 164.3..220.1 ms run. A CI runner with fewer cores moves it the same way,
/// because fewer cores means more waves. The instrumented run is the slow
/// case for wall time and the safe case for the observation.
///
/// Do not lower this number to save a tenth of a second. The seconds are in
/// the setup, the safety is in the gap, and the two move together.
const FILES: usize = 96;

/// [`FILES`] as the window counts them. One definition, cast once.
const FILES_SHOWN: u32 = FILES as u32;

/// How long the observation loop sleeps between frames.
///
/// The same 5 ms `tests/shell.rs` polls its two-file run with. At [`FILES`]
/// this is 9 or more polls inside the measured worst-case gap; the margin is
/// the gap, and this only has to be comfortably shorter than it.
const POLL: Duration = Duration::from_millis(5);

/// How long a run may take before the test calls it a failure.
///
/// A bound, not a wait: the loop leaves the moment the state stops being
/// `Processing`. Measured at 56..97 ms plain and 164..220 ms instrumented on
/// 12 cores; a 2-core CI runner under instrumentation projects to about 2 s,
/// so this leaves an order of magnitude. Reaching it means the shell never
/// drained the worker's channel, or the worker never finished.
const RUN_TIMEOUT: Duration = Duration::from_secs(30);

/// The window the design specifies (`layout.md`).
const WINDOW_SIZE: [f32; 2] = [520.0, 440.0];

/// The variable `Settings::config_dir` reads.
const CONFIG_DIR_ENV: &str = "MANHWA_CROPPER_CONFIG_DIR";

// --- The two sentences the controls check for --------------------------------

/// The heart of AC-1's failure message, named so that the negative controls
/// can assert that the assertion they watched fail was *this* one.
const NO_COUNT_FROM_THE_RUN: &str = "no frame showed a count from the running batch";

/// The heart of AC-2's "the bar never moved" failure message, for the same
/// reason.
const THE_BAR_NEVER_MOVED: &str = "the last count before the summary was";

// --- What the window showed --------------------------------------------------

/// One `AppState::Processing` count, as one frame showed it.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Count {
    /// How many files the window said were finished.
    done: u32,
    /// How many files the window said the run was given.
    total: u32,
}

impl fmt::Debug for Count {
    /// `12/96`, so that a whole trace fits in a failure message.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.done, self.total)
    }
}

/// Everything one observed run has to say for itself.
struct Observed {
    /// The count the window showed in every frame it was still `Processing`,
    /// in frame order. The first is always the one `Model::files_dropped`
    /// set, because the drop's frame precedes any `Progress`.
    counts: Vec<Count>,
    /// The summary the run ended with.
    summary: RunSummary,
}

/// Step `harness` until the run ends, recording what the window showed.
///
/// Polls rather than blocks, because the shell only drains its channel inside
/// a frame: not stepping is not waiting, it is refusing to listen.
fn observe_run<P, R>(harness: &mut Harness<'static, Shell<P, R>>) -> Observed
where
    P: FolderPicker,
    R: BatchRunner,
{
    let started = Instant::now();
    let mut counts = Vec::new();
    loop {
        match &harness.state().model().state {
            AppState::Processing { done, total } => counts.push(Count {
                done: *done,
                total: *total,
            }),
            AppState::Done { summary } => {
                return Observed {
                    counts,
                    summary: summary.clone(),
                };
            }
            other => panic!("the run ended in {other:?} instead of a summary"),
        }
        assert!(
            started.elapsed() < RUN_TIMEOUT,
            "the window was still `Processing` after {RUN_TIMEOUT:?}: either the worker \
             never finished, or the shell never drained its channel. Counts so far: {counts:?}"
        );
        thread::sleep(POLL);
        harness.step();
    }
}

// --- The criteria, as two functions the controls can also call ---------------

/// **AC-1** - some frame showed a count that came from the running batch.
///
/// "From the running batch" is `total == files` **and** `done > 0`, and the
/// second half is not decoration. `Model::files_dropped` sets
/// `Processing { done: 0, total: N }` from a `u32::try_from` of its own,
/// before the runner has been started and long before any `Progress` can
/// arrive; the audit (E-2) already pinned that conversion, and it is not the
/// one this story is about. A frame showing `total == N` alone is therefore
/// satisfied by a window that never received a single `Progress`, and
/// `done > 0` is the only thing in the count that tells the two apart -
/// `batch::run` calls its callback with `done` running `1..=total`, so every
/// count the engine produces has `done >= 1` and the pre-run frame is the
/// only one that can show zero.
fn assert_a_frame_showed_the_running_batch(counts: &[Count], files: u32) {
    let from_the_run = counts
        .iter()
        .filter(|count| count.done > 0 && count.total == files)
        .count();
    assert!(
        from_the_run > 0,
        "{NO_COUNT_FROM_THE_RUN}: no frame showed `done > 0` of {files} while the run was \
         under way, so the window never reported a count the engine produced. Counts \
         shown, in frame order: {counts:?}"
    );
}

/// **AC-2** - every count the window showed made sense, and the last one
/// before the summary was not still sitting on zero.
///
/// Phrased over what was observed rather than over an exact sequence: which
/// intermediate counts a frame catches depends on the scheduler, so an
/// assertion that demanded a particular one would be flaky by construction.
/// The first count in the trace is the one the drop set, and it is
/// deliberately included in both halves - it is what the window showed, and
/// the strict reading is the one with teeth.
fn assert_every_count_made_sense(counts: &[Count]) {
    assert!(
        !counts.is_empty(),
        "the window was never seen `Processing` at all, so there is no count to judge: \
         the run finished inside the frame that started it"
    );
    let impossible: Vec<&Count> = counts.iter().filter(|c| c.done > c.total).collect();
    assert!(
        impossible.is_empty(),
        "the window showed {} count(s) of more files than the run was given: {impossible:?}. \
         Counts shown, in frame order: {counts:?}",
        impossible.len()
    );
    let last = counts.last().expect("a non-empty trace");
    assert!(
        last.done > 0,
        "{THE_BAR_NEVER_MOVED} {last:?}: the window's count never moved off zero for the \
         whole run, and then snapped straight to the summary. Counts shown, in frame \
         order: {counts:?}"
    );
}

// --- AC-1 and AC-2: the real runner, real files, real frames -----------------

#[test]
fn a_real_run_shows_a_moving_count_of_the_whole_batch_before_it_summarises() {
    let fx = fixture("real-run");
    let (inputs, out) = batch_of(&fx.scratch, FILES);
    let mut harness = window(Shell::new(
        folder_model(&out),
        NeverPicker,
        ThreadRunner::new(),
    ));

    drop_files(&mut harness, &inputs);
    harness.step();
    assert_eq!(
        harness.state().model().state,
        AppState::Processing {
            done: 0,
            total: FILES_SHOWN
        },
        "the drop starts the real run, which the window reports before it can have finished"
    );

    let observed = observe_run(&mut harness);

    assert_a_frame_showed_the_running_batch(&observed.counts, FILES_SHOWN);
    assert_every_count_made_sense(&observed.counts);
    assert_eq!(
        observed.summary.cropped(),
        FILES,
        "the run those counts described has to be the run that happened: every one of the \
         {FILES} images is the screenshot scene, and every one of them crops"
    );
}

// --- The negative controls: the audit's two survivors, at the runner ---------

#[test]
fn a_conversion_that_always_answers_zero_is_caught_by_both_assertions() {
    let fx = fixture("mutant-zero");
    let (inputs, out) = batch_of(&fx.scratch, FILES);
    let mut harness = window(Shell::new(
        folder_model(&out),
        NeverPicker,
        MutantRunner::answering(|_| 0),
    ));
    drop_files(&mut harness, &inputs);
    harness.step();
    let observed = observe_run(&mut harness);

    assert!(
        observed.counts.contains(&Count { done: 0, total: 0 }),
        "this control never observed a converted count at all, so it proves nothing about \
         the assertions below - an empty run would fail them for the wrong reason. Counts \
         shown: {:?}",
        observed.counts
    );

    let ac1 = failure_of(|| assert_a_frame_showed_the_running_batch(&observed.counts, FILES_SHOWN))
        .expect("AC-1's assertion must reject a run that only ever reported `0 of 0`");
    assert!(
        ac1.contains(NO_COUNT_FROM_THE_RUN),
        "AC-1's assertion failed, but not with the message this control expects.\n  \
         wanted a message containing: {NO_COUNT_FROM_THE_RUN}\n  got: {ac1}"
    );

    let ac2 = failure_of(|| assert_every_count_made_sense(&observed.counts))
        .expect("AC-2's assertion must reject a run whose last count before the summary was 0");
    assert!(
        ac2.contains(THE_BAR_NEVER_MOVED),
        "AC-2's assertion failed, but not with the message this control expects.\n  \
         wanted a message containing: {THE_BAR_NEVER_MOVED}\n  got: {ac2}"
    );
}

#[test]
fn a_conversion_that_always_answers_one_is_caught_by_the_total_and_not_by_the_bar() {
    let fx = fixture("mutant-one");
    let (inputs, out) = batch_of(&fx.scratch, FILES);
    let mut harness = window(Shell::new(
        folder_model(&out),
        NeverPicker,
        MutantRunner::answering(|_| 1),
    ));
    drop_files(&mut harness, &inputs);
    harness.step();
    let observed = observe_run(&mut harness);

    assert!(
        observed.counts.contains(&Count { done: 1, total: 1 }),
        "this control never observed a converted count at all, so it proves nothing about \
         the assertions below. Counts shown: {:?}",
        observed.counts
    );

    let ac1 = failure_of(|| assert_a_frame_showed_the_running_batch(&observed.counts, FILES_SHOWN))
        .expect("AC-1's assertion must reject a run that only ever reported `1 of 1`");
    assert!(
        ac1.contains(NO_COUNT_FROM_THE_RUN),
        "AC-1's assertion failed, but not with the message this control expects.\n  \
         wanted a message containing: {NO_COUNT_FROM_THE_RUN}\n  got: {ac1}"
    );

    // And this is the half that says why AC-1 is phrased the way it is: a
    // window stuck on `1 of 1` moves, counts within its total, and ends with
    // a non-zero count, so AC-2 alone has nothing to complain about. Only
    // AC-1's `total == FILES` catches it.
    assert!(
        failure_of(|| assert_every_count_made_sense(&observed.counts)).is_none(),
        "AC-2 was expected to accept `1 of 1` - it is a count that moved and never exceeded \
         its total. If AC-2 now rejects it, this control has stopped documenting why AC-1 \
         must demand `total == {FILES_SHOWN}` as well as `done > 0`, and the reasoning in \
         both needs re-reading"
    );
}

// --- Watching an assertion fail on purpose -----------------------------------

/// The message `assertion` panicked with, or [`None`] if it passed.
///
/// The panic hook is silenced for the duration so that a control's deliberate
/// red does not print a stack trace that reads like a failure. The hook is
/// process-wide, which is the second reason every test in this file holds
/// [`ENV_LOCK`].
fn failure_of(assertion: impl FnOnce() + UnwindSafe) -> Option<String> {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let outcome = panic::catch_unwind(assertion);
    panic::set_hook(previous);
    // `&*payload` and not `&payload`: a `Box<dyn Any + Send>` is itself
    // `Any`, so `&payload` hands `message_of` a `dyn Any` that *is the box*
    // and every downcast below misses. Observed in RED - the controls
    // reported "a panic carrying no message" for an assertion that had in
    // fact failed with the right one.
    outcome.err().map(|payload| message_of(&*payload))
}

/// A panic payload as the string the test runner would have printed.
fn message_of(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic carrying no message".to_owned())
}

// --- The runner that is `ThreadRunner` with one function replaced ------------

/// `ThreadRunner`'s body with `shell::count` replaced by `substitute`.
///
/// The audit's two survivors are mutations of `count`, and a test may not
/// mutate the source it is testing. Injecting the substitute at the runner is
/// as close as a test can get: everything else - the real engine, the real
/// thread, the real channel, the real `Shell::drain` - is the shipped path.
/// What this proves is that the assertions above discriminate. That they
/// discriminate *on `shell::count`* is AC-4's job, in GREEN.
struct MutantRunner {
    /// Stands where `shell::count` stands.
    substitute: fn(usize) -> u32,
}

impl MutantRunner {
    /// A runner whose conversion is `substitute`.
    fn answering(substitute: fn(usize) -> u32) -> Self {
        Self { substitute }
    }
}

impl BatchRunner for MutantRunner {
    fn start(&self, inputs: Vec<PathBuf>, out_dir: PathBuf, tx: Sender<Event>) {
        let substitute = self.substitute;
        thread::spawn(move || {
            let progress = tx.clone();
            let summary = batch::run(
                &inputs,
                &out_dir,
                &Tuning::default(),
                &|Progress { done, total }| {
                    let _ = progress.send(Event::Progress {
                        done: substitute(done),
                        total: substitute(total),
                    });
                },
            );
            let _ = tx.send(Event::Finished(summary));
        });
    }
}

// --- A dialog that must never open -------------------------------------------

/// A picker that fails the test if it is ever asked.
///
/// Nothing here clicks anything, and the folder button is disabled for the
/// whole of a run, so a call means the shell opened a modal dialog on its own.
struct NeverPicker;

impl FolderPicker for NeverPicker {
    fn pick(&self, title: &str, start_in: Option<&Path>) -> Option<PathBuf> {
        panic!("the shell opened the folder dialog during a run: {title:?}, {start_in:?}");
    }
}

// --- Owning the process-wide state -------------------------------------------

/// The lock every test in this binary holds: one process, one environment and
/// one panic hook. See the module docs.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds [`ENV_LOCK`] and puts [`CONFIG_DIR_ENV`] back the way it was, during
/// a panicking test's unwind as well as an ordinary return.
struct ScopedConfigDirEnv {
    previous: Option<OsString>,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for ScopedConfigDirEnv {
    fn drop(&mut self) {
        // SAFETY: `_lock` is still held, so no other test in this binary is
        // reading or writing the environment right now.
        unsafe { write_config_dir_env(self.previous.as_deref()) }
    }
}

/// Set [`CONFIG_DIR_ENV`] to `value`, or remove it when `value` is [`None`].
///
/// # Safety
///
/// The caller must hold [`ENV_LOCK`]. `std::env::set_var` and
/// `std::env::remove_var` are unsound if another thread is reading the
/// environment at the same time.
unsafe fn write_config_dir_env(value: Option<&OsStr>) {
    match value {
        Some(dir) => unsafe { env::set_var(CONFIG_DIR_ENV, dir) },
        None => unsafe { env::remove_var(CONFIG_DIR_ENV) },
    }
}

// --- Scratch directories -----------------------------------------------------

/// A unique directory under the OS temp dir, removed on drop - including
/// during a panic unwind, so a failing test does not leave 96 files behind.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(test_name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("mc024-{}-{test_name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)
            .unwrap_or_else(|e| panic!("cannot create scratch dir {}: {e}", root.display()));
        Self { root }
    }

    /// Create `name` as a directory and return its path.
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.root.join(name);
        fs::create_dir_all(&p).unwrap_or_else(|e| panic!("cannot create {}: {e}", p.display()));
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A scratch directory and ownership of the process-wide state.
///
/// Field order is drop order: the scratch goes first, then the variable is put
/// back, then the lock is released.
struct Fixture {
    scratch: Scratch,
    _env: ScopedConfigDirEnv,
}

fn fixture(test_name: &str) -> Fixture {
    let scratch = Scratch::new(test_name);
    let config = scratch.dir("config");
    let lock = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let previous = env::var_os(CONFIG_DIR_ENV);
    // SAFETY: the lock is held for as long as the guard lives, and every test
    // in this binary takes it before touching the environment.
    unsafe { write_config_dir_env(Some(config.as_os_str())) }
    Fixture {
        scratch,
        _env: ScopedConfigDirEnv {
            previous,
            _lock: lock,
        },
    }
}

/// `n` real PNG files of the engine's screenshot scene, and the folder the
/// window will be told to write crops into.
///
/// One scene, written `n` times under `n` names: `naming::plan_outputs` gives
/// each its own output, and every one of them crops, so the run's summary is
/// `n` cropped and nothing else. The scene is generated once - it is the same
/// plane every time, and generating it 96 times would be most of the setup.
fn batch_of(scratch: &Scratch, n: usize) -> (Vec<PathBuf>, PathBuf) {
    let inbox = scratch.dir("inbox");
    let out = scratch.dir("out");
    let plane = common::screenshot();
    let inputs = (0..n)
        .map(|i| {
            let path = inbox.join(format!("f{i:03}.png"));
            common::write_grey8(&plane, &path);
            path
        })
        .collect();
    (inputs, out)
}

// --- Driving the window ------------------------------------------------------

/// A file the window is told was dropped on it, through the same
/// `RawInput::dropped_files` the shipped window reads.
#[derive(Debug)]
struct DroppedPath(PathBuf);

impl egui::DroppedFile for DroppedPath {
    fn path(&self) -> &Path {
        &self.0
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        fs::read(&self.0).map_err(|e| e.to_string())
    }
}

/// A window over `shell`, at the size the design specifies.
fn window<P, R>(shell: Shell<P, R>) -> Harness<'static, Shell<P, R>>
where
    P: FolderPicker + 'static,
    R: BatchRunner + 'static,
{
    Harness::builder()
        .with_size(egui::vec2(WINDOW_SIZE[0], WINDOW_SIZE[1]))
        .build_ui_state(
            |ui, shell: &mut Shell<P, R>| {
                let ctx = ui.ctx().clone();
                shell.frame(&ctx, ui);
            },
            shell,
        )
}

/// Hand `paths` to the next frame as a drop, in this order.
fn drop_files<S>(harness: &mut Harness<'_, S>, paths: &[PathBuf]) {
    harness.input_mut().dropped_files = paths
        .iter()
        .map(|p| Arc::new(DroppedPath(p.clone())) as egui::DroppedFileHandle)
        .collect();
}

/// A window that already knows where crops go, as a launch with a remembered
/// folder leaves it.
fn folder_model(out: &Path) -> Model {
    Model::new(Settings {
        output_dir: Some(out.to_path_buf()),
    })
}
