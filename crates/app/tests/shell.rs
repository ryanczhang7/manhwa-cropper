//! MC-016: the shell that turns input into events, events into effects, and a
//! worker thread's progress back into painted frames.
//!
//! # What this file drives, and why it is not `tests/gui.rs`
//!
//! `tests/gui.rs` is MC-015's and is frozen: it drives [`gui::paint`] over a
//! model the test hands it and asks what the accessibility tree holds. Nothing
//! in it can press a button, because `paint` takes a model and returns
//! nothing. This file drives `Shell::frame` instead - the one function that
//! reads `egui`'s input, applies MC-014's `Model::handle`, carries out the
//! commands it answers with, and paints the result - and asks two questions of
//! every criterion: *what did the shell ask the outside world to do*, and
//! *what does the window say afterwards*.
//!
//! The first question is answered by fakes. `FolderPicker` and `BatchRunner`
//! are the two things the shell cannot do itself in a test - a modal native
//! dialog and a worker thread - so both are traits, and the fakes here record
//! every call with its arguments. The second is answered exactly as MC-015
//! answers it: a query against the AccessKit tree for a string copied
//! character for character out of `docs/wiki/design/voice.md`.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! Per the story's `## Model guidance`, every criterion is **mechanical**.
//!
//! * **Settled elsewhere, read out rather than re-derived**: every string
//!   (`voice.md`'s frozen table, including `folder.dialog_title` =
//!   `Choose output folder`); the `cropped` fallback folder and the
//!   precedence around it (`architecture.md` decision 6, implemented by
//!   `resolve_out_dir` and pinned by MC-012); that a fallback is shown and
//!   not remembered (the same decision, and MC-014's `Command::SaveSettings`
//!   doc); the states MC-014's `Model` moves between.
//! * **Mechanical**: every assertion below. An exact call record, an exact
//!   `AppState`, an exact path, an exact directory listing, or a string from
//!   the frozen table.
//! * **Measured**: nothing is an expectation here. AC-7's 10 s bound is a
//!   failure bound and not a measurement; the real run it bounds took 5-6 ms
//!   of wall time when it was measured in RED, which is in the story's
//!   `## Test plan`.
//!
//! # Why every test takes the environment lock
//!
//! `Command::SaveSettings` is carried out by `Settings::save`, which reads
//! `MANHWA_CROPPER_CONFIG_DIR`. That variable is process-wide, and this
//! binary is one process. Every test here builds a `Shell`, and a `Shell` is
//! the thing that can write `settings.json`; a test that did not own the
//! variable would let a defective shell write into the developer's real
//! `%APPDATA%\manhwa-cropper\config` (which this project has done twice
//! before - see `tests/headless.rs`) or, worse, into the scratch directory of
//! whichever test *does* own it, making that test's "the settings file was
//! not written" assertion depend on the scheduler. So the guard is taken by
//! every test, pointed at that test's own scratch directory, and restored on
//! drop. The lock serialises this binary; it is eleven fast tests and one
//! that takes a few milliseconds.

#[path = "../../engine/tests/common/mod.rs"]
mod common;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};
use std::{env, fs, thread};

use cropper_core::Rect;
use cropper_engine::args::Invocation;
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Outcome};
use cropper_engine::settings::Settings;
use eframe::egui;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use manhwa_cropper::shell::{BatchRunner, FolderPicker, Shell, ThreadRunner};
use manhwa_cropper::{AppState, Event, Model};

// --- The frozen strings (docs/wiki/design/voice.md) --------------------------

/// `folder.dialog_title` - the one argument the native dialog is given.
const DIALOG_TITLE: &str = "Choose output folder";
/// `folder.button`. One U+2026.
const FOLDER_BUTTON: &str = "Choose folder…";
/// `folder.none`.
const FOLDER_NONE: &str = "No output folder chosen";
/// `dropzone.no_folder`.
const DROPZONE_NO_FOLDER: &str = "Choose an output folder, then drop images here";
/// `dropzone.ready`.
const DROPZONE_READY: &str = "Drop images here";
/// `dropzone.hover`.
const DROPZONE_HOVER: &str = "Release to crop";
/// `dropzone.busy`. One U+2026, never three dots.
const DROPZONE_BUSY: &str = "Cropping…";

// --- The frozen numbers and names --------------------------------------------

/// The window the design specifies (`layout.md`), so the tree this file reads
/// is the tree of the window that ships.
const WINDOW_SIZE: [f32; 2] = [520.0, 440.0];

/// The variable `Settings::config_dir` reads.
const CONFIG_DIR_ENV: &str = "MANHWA_CROPPER_CONFIG_DIR";

/// The settings file's name inside the config directory (MC-013).
const SETTINGS_FILE: &str = "settings.json";

/// The fallback output folder's name (`architecture.md` decision 6).
const FALLBACK: &str = "cropped";

/// How long AC-7's real run may take before the test calls it a failure.
///
/// A bound, not a wait: the poll below leaves the moment the state stops being
/// `Processing`, and the measured run is three orders of magnitude inside
/// this. Reaching it means the shell never drained the worker's channel, or
/// the worker never finished.
const RUN_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the poll sleeps between frames. Short enough that the test costs
/// what the run costs, long enough not to spin.
const POLL: Duration = Duration::from_millis(5);

// --- Owning MANHWA_CROPPER_CONFIG_DIR ----------------------------------------

/// The lock every test in this binary holds. One process, one environment;
/// see the module docs for why "every" and not "the ones that assert about
/// `settings.json`".
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds [`ENV_LOCK`] and puts [`CONFIG_DIR_ENV`] back the way it was.
///
/// `Drop` runs while the lock is still held, and during a panicking test's
/// unwind, so a failed assertion cannot leak a config directory into the next
/// test.
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

/// Set [`CONFIG_DIR_ENV`] to `value`, or remove it when `value` is `None`.
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

/// A unique directory under the OS temp dir, removed on drop (including
/// during a panic unwind, so a failing test does not leak it).
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(test_name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("mc016-{}-{test_name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)
            .unwrap_or_else(|e| panic!("cannot create scratch dir {}: {e}", root.display()));
        Self { root }
    }

    /// A path inside the scratch space. Nothing is created: a folder the user
    /// picks is a path that is recorded, not one that is visited.
    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// Create `name` as a directory and return its path.
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.path(name);
        fs::create_dir_all(&p).unwrap_or_else(|e| panic!("cannot create {}: {e}", p.display()));
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A scratch directory, an empty config directory inside it, and ownership of
/// `MANHWA_CROPPER_CONFIG_DIR` pointing at that config directory.
///
/// Field order is drop order: the scratch goes first, then the variable is put
/// back, then the lock is released.
struct Fixture {
    scratch: Scratch,
    config: PathBuf,
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
        config,
        _env: ScopedConfigDirEnv {
            previous,
            _lock: lock,
        },
    }
}

// --- Reading the settings file -----------------------------------------------

/// Whether `settings.json` exists in `config` at all.
///
/// The one query behind both halves of the story's settings criteria: AC-1 and
/// AC-4 call it and expect opposite answers, so neither absence can be an
/// assertion that could never have been true.
fn settings_written(config: &Path) -> bool {
    config.join(SETTINGS_FILE).is_file()
}

/// The folder `settings.json` in `config` remembers, read back through the
/// engine's own reader. `None` for a file that is missing or unreadable.
fn remembered_folder(config: &Path) -> Option<PathBuf> {
    Settings::load_from(config).output_dir
}

// --- The fake folder picker --------------------------------------------------

/// One call to [`FolderPicker::pick`], with both of its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PickerCall {
    /// The dialog's title.
    title: String,
    /// The folder the dialog was asked to start in.
    start_in: Option<PathBuf>,
}

/// The shared record of what the shell asked the dialog for.
#[derive(Debug, Default, Clone)]
struct PickerLog(Arc<Mutex<Vec<PickerCall>>>);

impl PickerLog {
    fn record(&self, title: &str, start_in: Option<&Path>) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(PickerCall {
                title: title.to_owned(),
                start_in: start_in.map(Path::to_path_buf),
            });
    }

    fn calls(&self) -> Vec<PickerCall> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// A dialog that never opens: it records the call and answers with whatever
/// the test decided the user would do.
struct FakePicker {
    answer: Option<PathBuf>,
    log: PickerLog,
}

impl FolderPicker for FakePicker {
    fn pick(&self, title: &str, start_in: Option<&Path>) -> Option<PathBuf> {
        self.log.record(title, start_in);
        self.answer.clone()
    }
}

/// A picker that answers `answer` - `None` being Cancel - and the log of what
/// it was asked.
fn picker(answer: Option<&Path>) -> (FakePicker, PickerLog) {
    let log = PickerLog::default();
    (
        FakePicker {
            answer: answer.map(Path::to_path_buf),
            log: log.clone(),
        },
        log,
    )
}

// --- The fake batch runner ---------------------------------------------------

/// One call to [`BatchRunner::start`], with the two arguments that say what
/// the run is.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RunCall {
    /// The files the run was given, in drop order.
    inputs: Vec<PathBuf>,
    /// Where its output goes.
    out_dir: PathBuf,
}

/// The shared record of every run the shell started, and the channels it
/// handed over - which is what lets a test play the worker thread.
#[derive(Debug, Default, Clone)]
struct RunLog {
    calls: Arc<Mutex<Vec<RunCall>>>,
    senders: Arc<Mutex<Vec<Sender<Event>>>>,
}

impl RunLog {
    fn record(&self, inputs: Vec<PathBuf>, out_dir: PathBuf, tx: Sender<Event>) {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(RunCall { inputs, out_dir });
        self.senders
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(tx);
    }

    fn calls(&self) -> Vec<RunCall> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Say what a worker thread would say, on the channel of the most recent
    /// run. Panics naming the cause if the shell dropped the receiver, which
    /// is what "the window stopped listening to its own run" looks like.
    fn report(&self, event: Event) {
        let senders = self.senders.lock().unwrap_or_else(PoisonError::into_inner);
        let tx = senders
            .last()
            .expect("no run was started, so there is no channel to report on");
        tx.send(event)
            .expect("the shell must keep the receiver of a running batch alive");
    }
}

/// A runner that starts nothing. It records the call and keeps the channel, so
/// the test decides when - and whether - the run appears to make progress.
struct FakeRunner {
    log: RunLog,
}

impl BatchRunner for FakeRunner {
    fn start(&self, inputs: Vec<PathBuf>, out_dir: PathBuf, tx: Sender<Event>) {
        self.log.record(inputs, out_dir, tx);
    }
}

fn runner() -> (FakeRunner, RunLog) {
    let log = RunLog::default();
    (FakeRunner { log: log.clone() }, log)
}

// --- Driving the window ------------------------------------------------------

/// A file the window is told was dropped on it.
///
/// egui 0.36 lets an integration own the file handle: `RawInput::dropped_files`
/// is a `Vec<DroppedFileHandle>` over the public `egui::DroppedFile` trait, so
/// a test can inject a drop through the same input the shipped window reads
/// rather than through a back door on the shell. That is the stronger route,
/// and it is the one the story's AC-3 asked the handoff to record.
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
///
/// The style is deliberately not installed: `CropperApp::new` does that once
/// in the shipped window, and nothing this file reads depends on it (measured
/// in RED - the button is the first Tab stop either way).
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

/// Drag `paths` over the window, and keep them there until they are cleared.
fn hover_files<S>(harness: &mut Harness<'_, S>, paths: &[PathBuf]) {
    harness.input_mut().hovered_files = paths
        .iter()
        .map(|p| egui::HoveredFile {
            path: Some(p.clone()),
            mime: String::new(),
        })
        .collect();
}

// --- Reading the window ------------------------------------------------------

/// How many Label nodes carry exactly `text` as their accessible name. The
/// same query MC-015's suite reads the window with.
fn labels_named<S>(harness: &Harness<'_, S>, text: &str) -> usize {
    harness
        .query_all_by_role_and_label(Role::Label, text)
        .count()
}

/// The model the shell is holding.
fn model_of<'a, P, R>(harness: &'a Harness<'_, Shell<P, R>>) -> &'a Model
where
    P: FolderPicker,
    R: BatchRunner,
{
    harness.state().model()
}

/// Names of the entries directly inside `dir`, sorted. Empty for a directory
/// that does not exist, so "nothing was written" reads the same either way.
fn entries(dir: &Path) -> Vec<String> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .map(|entry| {
            entry
                .expect("a readable directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

// --- Fixtures ----------------------------------------------------------------

/// A window that already knows where crops go, as a launch with a remembered
/// folder leaves it.
fn folder_model(out: &Path) -> Model {
    Model::new(Settings {
        output_dir: Some(out.to_path_buf()),
    })
}

/// A GUI launch: file paths, no `--no-gui`, and whatever `--out` said.
fn invocation(inputs: Vec<PathBuf>, out_dir: Option<PathBuf>) -> Invocation {
    Invocation {
        inputs,
        out_dir,
        no_gui: false,
        summary_path: None,
    }
}

/// The summary a two-file run that cropped both would end with.
fn both_cropped(out: &Path) -> RunSummary {
    RunSummary {
        results: ["a.png", "b.png"]
            .iter()
            .map(|name| FileResult {
                input: PathBuf::from(name),
                outcome: Outcome::Cropped {
                    rect: Rect {
                        x: 0,
                        y: 0,
                        w: 800,
                        h: 1200,
                    },
                    output: out.join(name),
                },
            })
            .collect(),
    }
}

// --- AC-1: the button opens the dialog, and the answer is kept ---------------

#[test]
fn clicking_choose_folder_opens_the_dialog_shows_the_folder_and_remembers_it() {
    let fx = fixture("ac1-click");
    let chosen = fx.scratch.path("pictures");
    let (picker, picks) = picker(Some(&chosen));
    let (runner, runs) = runner();
    let mut harness = window(Shell::new(Model::new(Settings::default()), picker, runner));

    assert!(
        !settings_written(&fx.config),
        "nothing is saved before the user has chosen anything"
    );

    harness
        .get_by_role_and_label(Role::Button, FOLDER_BUTTON)
        .click();
    harness.run();

    assert_eq!(
        picks.calls(),
        vec![PickerCall {
            title: DIALOG_TITLE.to_owned(),
            start_in: None,
        }],
        "the button opens the dialog exactly once, titled `Choose output folder`, and with \
         no starting directory because no folder is known yet"
    );
    assert_eq!(
        labels_named(&harness, &chosen.display().to_string()),
        1,
        "the path label shows the folder the user just chose"
    );
    assert_eq!(
        labels_named(&harness, FOLDER_NONE),
        0,
        "`No output folder chosen` must not survive a folder being chosen"
    );
    assert!(
        settings_written(&fx.config),
        "a folder the user chose is remembered, so settings.json exists afterwards"
    );
    assert_eq!(
        remembered_folder(&fx.config),
        Some(chosen.clone()),
        "settings.json holds the chosen folder and not some other path"
    );
    assert_eq!(
        model_of(&harness).output_dir,
        Some(chosen),
        "the model carries the chosen folder from here on"
    );
    assert_eq!(
        runs.calls(),
        Vec::new(),
        "choosing a folder starts no run - there is nothing to run yet"
    );
}

#[test]
fn tab_then_enter_opens_the_dialog_without_a_mouse() {
    let fx = fixture("ac1-keyboard");
    let chosen = fx.scratch.path("pictures");
    let (picker, picks) = picker(Some(&chosen));
    let (runner, _runs) = runner();
    let mut harness = window(Shell::new(Model::new(Settings::default()), picker, runner));

    harness.key_press(egui::Key::Tab);
    harness.run();
    assert!(
        harness
            .get_by_role_and_label(Role::Button, FOLDER_BUTTON)
            .is_focused(),
        "the button is the window's only focus stop, so the first Tab must land on it"
    );
    assert_eq!(
        picks.calls(),
        Vec::new(),
        "reaching the button does not open the dialog"
    );

    harness.key_press(egui::Key::Enter);
    harness.run();

    assert_eq!(
        picks.calls(),
        vec![PickerCall {
            title: DIALOG_TITLE.to_owned(),
            start_in: None,
        }],
        "Enter on the focused button opens the same dialog a click does"
    );
    assert_eq!(
        labels_named(&harness, &chosen.display().to_string()),
        1,
        "the keyboard route ends in the same painted folder as the mouse route"
    );
    assert_eq!(
        remembered_folder(&fx.config),
        Some(chosen),
        "the keyboard route remembers the folder just as the mouse route does"
    );
}

#[test]
fn the_dialog_starts_in_the_folder_that_is_already_chosen() {
    let fx = fixture("ac1-start-in");
    let first = fx.scratch.path("first");
    let second = fx.scratch.path("second");
    let (picker, picks) = picker(Some(&second));
    let (runner, _runs) = runner();
    let mut harness = window(Shell::new(folder_model(&first), picker, runner));

    harness
        .get_by_role_and_label(Role::Button, FOLDER_BUTTON)
        .click();
    harness.run();

    assert_eq!(
        picks.calls(),
        vec![PickerCall {
            title: DIALOG_TITLE.to_owned(),
            start_in: Some(first),
        }],
        "with a folder known, the dialog opens in it rather than wherever Windows last was"
    );
    assert_eq!(
        labels_named(&harness, &second.display().to_string()),
        1,
        "the newly chosen folder replaces the old one in the path label"
    );
    assert_eq!(
        remembered_folder(&fx.config),
        Some(second),
        "the newly chosen folder replaces the old one on disk"
    );
}

// --- AC-2: Cancel ------------------------------------------------------------

#[test]
fn cancelling_the_dialog_changes_nothing_and_leaves_focus_on_the_button() {
    let fx = fixture("ac2-cancel");
    let (picker, picks) = picker(None);
    let (runner, runs) = runner();
    let before = Model::new(Settings::default());
    let mut harness = window(Shell::new(before.clone(), picker, runner));

    harness.key_press(egui::Key::Tab);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();

    assert_eq!(
        picks.calls().len(),
        1,
        "the dialog did open - a Cancel is the picker's answer, not a call that never happened"
    );
    assert_eq!(
        model_of(&harness),
        &before,
        "Cancel changes no part of the model: not the state, not the folder"
    );
    assert_eq!(
        labels_named(&harness, FOLDER_NONE),
        1,
        "the path label still reads `No output folder chosen`"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_NO_FOLDER),
        1,
        "the drop zone still asks for a folder; Cancel puts no message on screen"
    );
    assert!(
        !settings_written(&fx.config),
        "a cancelled dialog writes no settings file at all"
    );
    assert!(
        harness
            .get_by_role_and_label(Role::Button, FOLDER_BUTTON)
            .is_focused(),
        "focus returns to the button the dialog was opened from, and is not surrendered"
    );
    assert_eq!(runs.calls(), Vec::new(), "a cancelled dialog starts no run");
}

// --- AC-3: a drop starts a run, and the run reports back ---------------------

#[test]
fn dropping_two_files_starts_a_run_over_exactly_those_files_and_that_folder() {
    let fx = fixture("ac3-start");
    let out = fx.scratch.path("out");
    let a = fx.scratch.path("a.png");
    let b = fx.scratch.path("b.png");
    let (picker, picks) = picker(None);
    let (runner, runs) = runner();
    let mut harness = window(Shell::new(folder_model(&out), picker, runner));

    drop_files(&mut harness, &[a.clone(), b.clone()]);
    harness.step();

    assert_eq!(
        runs.calls(),
        vec![RunCall {
            inputs: vec![a, b],
            out_dir: out,
        }],
        "one run, over both dropped files in the order they were dropped, into the chosen \
         folder"
    );
    assert_eq!(
        model_of(&harness).state,
        AppState::Processing { done: 0, total: 2 },
        "the first frame after a drop already counts the files, and none of them is done"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_BUSY),
        1,
        "the drop zone says a run is under way"
    );
    assert_eq!(
        labels_named(&harness, "0 of 2"),
        1,
        "the count starts at `0 of 2` on the frame the drop landed"
    );
    assert_eq!(
        picks.calls(),
        Vec::new(),
        "a drop never opens the folder dialog"
    );
}

#[test]
fn progress_and_then_a_summary_each_move_the_window_on_in_one_frame() {
    let fx = fixture("ac3-progress");
    let out = fx.scratch.path("out");
    let (picker, _picks) = picker(None);
    let (runner, runs) = runner();
    let mut harness = window(Shell::new(folder_model(&out), picker, runner));

    drop_files(
        &mut harness,
        &[fx.scratch.path("a.png"), fx.scratch.path("b.png")],
    );
    harness.step();

    runs.report(Event::Progress { done: 1, total: 2 });
    harness.step();

    assert_eq!(
        labels_named(&harness, "1 of 2"),
        1,
        "one Progress message advances the count on the very next frame"
    );
    assert_eq!(
        labels_named(&harness, "0 of 2"),
        0,
        "the new count replaces the old one; there is no tween and no second count"
    );
    assert_eq!(
        labels_named(&harness, "2 cropped, 0 flagged"),
        0,
        "the run is not over, so no result line is shown"
    );

    runs.report(Event::Finished(both_cropped(&out)));
    harness.step();

    assert_eq!(
        labels_named(&harness, "2 cropped, 0 flagged"),
        1,
        "the summary swaps the status slot to the result line in one frame"
    );
    assert_eq!(
        labels_named(&harness, "1 of 2"),
        0,
        "the progress count is gone once the run has ended"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        1,
        "with the run over the drop zone invites the next one"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_BUSY),
        0,
        "`Cropping…` must not outlive the run"
    );
}

// --- The disabled button (Design notes, "Disabled during processing") --------

#[test]
fn the_button_opens_nothing_while_a_run_is_under_way() {
    let fx = fixture("disabled-button");
    let out = fx.scratch.path("out");
    let other = fx.scratch.path("somewhere-else");
    // An answer that would be visible if it were ever asked for: a picker
    // called here would change the folder, so this test fails twice over.
    let (picker, picks) = picker(Some(&other));
    let (runner, runs) = runner();
    let mut harness = window(Shell::new(folder_model(&out), picker, runner));

    drop_files(
        &mut harness,
        &[fx.scratch.path("a.png"), fx.scratch.path("b.png")],
    );
    harness.step();
    assert_eq!(
        model_of(&harness).state,
        AppState::Processing { done: 0, total: 2 },
        "the run must really be under way, or this test asserts nothing"
    );

    harness
        .get_by_role_and_label(Role::Button, FOLDER_BUTTON)
        .click();
    harness.step();
    harness.key_press(egui::Key::Tab);
    harness.step();
    harness.key_press(egui::Key::Enter);
    harness.step();

    assert_eq!(
        picks.calls(),
        Vec::new(),
        "a disabled button opens no dialog, by mouse or by keyboard: it is out of the Tab \
         order and it ignores a click"
    );
    assert_eq!(
        model_of(&harness).output_dir,
        Some(out),
        "the folder the running batch is writing into is untouched"
    );
    assert_eq!(runs.calls().len(), 1, "nothing here starts a second run");
}

// --- Hover (Design notes, "Drops") -------------------------------------------

#[test]
fn files_dragged_over_the_window_promise_the_crop_until_they_leave() {
    let fx = fixture("hover");
    let out = fx.scratch.path("out");
    let (picker, _picks) = picker(None);
    let (runner, runs) = runner();
    let mut harness = window(Shell::new(folder_model(&out), picker, runner));

    harness.step();
    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        1,
        "with nothing over the window the zone reads `Drop images here`"
    );

    hover_files(&mut harness, &[fx.scratch.path("a.png")]);
    harness.step();
    assert_eq!(
        labels_named(&harness, DROPZONE_HOVER),
        1,
        "a drag over the window promises what releasing will do"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        0,
        "`Drop images here` is replaced while files hover, not shown alongside"
    );

    harness.input_mut().hovered_files.clear();
    harness.step();
    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        1,
        "the promise is withdrawn the moment the drag leaves the window"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_HOVER),
        0,
        "`Release to crop` must not survive the drag"
    );
    assert_eq!(
        runs.calls(),
        Vec::new(),
        "hovering is not dropping: nothing has been run"
    );
}

// --- AC-4, AC-5, AC-6: the Send-to launch ------------------------------------

#[test]
fn a_launch_with_files_and_no_remembered_folder_runs_into_the_cropped_fallback() {
    let fx = fixture("ac4-fallback");
    let shots = fx.scratch.dir("shots");
    let a = shots.join("a.png");
    let b = shots.join("b.png");
    let (picker, picks) = picker(None);
    let (runner, runs) = runner();
    let inv = invocation(vec![a.clone(), b.clone()], None);
    let mut harness = window(Shell::from_invocation(
        &inv,
        Settings::default(),
        picker,
        runner,
    ));
    harness.step();

    assert_eq!(
        runs.calls(),
        vec![RunCall {
            inputs: vec![a, b],
            out_dir: shots.join(FALLBACK),
        }],
        "one run, started once however many frames are painted, into a `cropped` folder \
         beside the first input"
    );
    assert_eq!(
        model_of(&harness).state,
        AppState::Processing { done: 0, total: 2 },
        "a Send-to launch is already processing on its first frame"
    );
    assert_eq!(
        labels_named(&harness, &format!(r"{}\{FALLBACK}", shots.display())),
        1,
        "the path label shows where the files are actually going, as a Windows path"
    );
    assert!(
        !settings_written(&fx.config),
        "the fallback is shown, not remembered: a one-off folder must not become the \
         remembered one"
    );
    assert_eq!(
        picks.calls(),
        Vec::new(),
        "a Send-to launch never opens the dialog by itself"
    );
}

#[test]
fn a_launch_with_files_and_a_remembered_folder_runs_into_that_folder() {
    let fx = fixture("ac5-remembered");
    let shots = fx.scratch.dir("shots");
    let a = shots.join("a.png");
    let b = shots.join("b.png");
    let remembered = fx.scratch.path("remembered");
    let (picker, _picks) = picker(None);
    let (runner, runs) = runner();
    let inv = invocation(vec![a.clone(), b.clone()], None);
    let settings = Settings {
        output_dir: Some(remembered.clone()),
    };
    let mut harness = window(Shell::from_invocation(&inv, settings, picker, runner));
    harness.step();

    assert_eq!(
        runs.calls(),
        vec![RunCall {
            inputs: vec![a, b],
            out_dir: remembered.clone(),
        }],
        "the remembered folder wins over the fallback"
    );
    assert_eq!(
        labels_named(&harness, &remembered.display().to_string()),
        1,
        "the path label shows the remembered folder"
    );
    assert_eq!(
        labels_named(&harness, &shots.join(FALLBACK).display().to_string()),
        0,
        "the fallback is never shown when a folder is remembered"
    );
    assert!(
        !settings_written(&fx.config),
        "a folder that merely arrived in the settings is not written back"
    );
}

#[test]
fn a_launch_with_no_files_starts_nothing_and_waits() {
    let fx = fixture("ac6-no-files");
    let (picker, picks) = picker(None);
    let (runner, runs) = runner();
    let inv = invocation(Vec::new(), None);
    let mut harness = window(Shell::from_invocation(
        &inv,
        Settings::default(),
        picker,
        runner,
    ));
    harness.step();

    assert_eq!(
        runs.calls(),
        Vec::new(),
        "a launch with nothing to do starts no run"
    );
    assert_eq!(
        model_of(&harness).state,
        AppState::Idle,
        "an ordinary launch opens an idle window"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_NO_FOLDER),
        1,
        "with no folder and nothing running, the window shows its one instruction"
    );
    assert_eq!(
        picks.calls(),
        Vec::new(),
        "a launch never opens the dialog by itself"
    );
    assert!(
        !settings_written(&fx.config),
        "opening the window writes nothing"
    );
}

// --- AC-7: the real runner, a real thread, real files ------------------------

#[test]
fn a_real_run_of_two_screenshots_writes_both_crops_and_reports_them() {
    let fx = fixture("ac7-real");
    let inbox = fx.scratch.dir("inbox");
    let out = fx.scratch.dir("out");
    let plane = common::screenshot();
    let a = inbox.join("a.png");
    let b = inbox.join("b.png");
    common::write_grey8(&plane, &a);
    common::write_grey8(&plane, &b);

    let (picker, picks) = picker(None);
    let mut harness = window(Shell::new(folder_model(&out), picker, ThreadRunner::new()));

    drop_files(&mut harness, &[a, b]);
    harness.step();
    assert_eq!(
        model_of(&harness).state,
        AppState::Processing { done: 0, total: 2 },
        "the drop starts the real run, which the window reports before it can have finished"
    );

    let started = Instant::now();
    while matches!(model_of(&harness).state, AppState::Processing { .. }) {
        assert!(
            started.elapsed() < RUN_TIMEOUT,
            "the window was still `Processing` after {RUN_TIMEOUT:?}: either the worker \
             never finished, or the shell never drained its channel"
        );
        thread::sleep(POLL);
        harness.step();
    }

    assert_eq!(
        entries(&out),
        vec!["a.png".to_owned(), "b.png".to_owned()],
        "both crops are in the folder the window said they were going to"
    );
    assert_eq!(
        labels_named(&harness, "2 cropped, 0 flagged"),
        1,
        "the window reports the run the engine actually did"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_BUSY),
        0,
        "the drop zone stops saying `Cropping…` when the run is over"
    );
    assert_eq!(
        labels_named(&harness, &out.display().to_string()),
        1,
        "the path label still shows where the files went"
    );
    assert_eq!(picks.calls(), Vec::new(), "a real run opens no dialog");
}
