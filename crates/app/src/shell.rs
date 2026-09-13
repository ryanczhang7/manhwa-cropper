//! The wiring: input to [`Event`], [`Command`] to effect, a worker thread's
//! progress back to a painted frame.
//!
//! [`Model`] is pure and [`gui`] only paints, so something has to read egui's
//! input, carry out the commands the model answers with, and keep the channel
//! a run reports on. That is [`Shell`], and it is the one place in this crate
//! where the three meet.
//!
//! # Why the dialog and the thread are traits
//!
//! A modal native dialog and a worker thread are the two things a headless
//! test cannot have. They are therefore [`FolderPicker`] and [`BatchRunner`],
//! injected by value, and `crates/app/tests/shell.rs` drives the whole shell
//! with fakes that record what they were asked for. [`ThreadRunner`] is the
//! real runner and lives here because a test can run it for real; the real
//! `rfd` picker lives in [`gui`], which the `coverage` gate ignores, because
//! nothing can ever execute a modal dialog under a test.
//!
//! # The order inside a frame
//!
//! Input is read and the channel is drained **before** anything is painted,
//! so a drop and a `Progress` each move the window on in the frame they
//! arrive in rather than the one after. The one thing that has to happen
//! after the paint is the button's activation, because the activation *is*
//! the paint's answer; when it changes the model the shell asks for one more
//! frame so that the new folder reaches the screen.
//!
//! While a run is under way the shell calls
//! [`request_repaint`](egui::Context::request_repaint) every frame
//! (`docs/wiki/architecture.md`, "Threading"): egui paints on demand, and
//! without it the count would only advance when the user moved the mouse.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use cropper_engine::args::Invocation;
use cropper_engine::batch::{self, Progress};
use cropper_engine::settings::Settings;
use cropper_engine::{Tuning, resolve_out_dir};
use eframe::egui;

use crate::{AppState, Command, Event, FOLDER_DIALOG_TITLE, Model, gui};

/// The native "choose a folder" dialog, as the shell needs it.
///
/// `start_in` is the folder the dialog opens in - the one already chosen,
/// when there is one (`docs/wiki/design/components.md`). Blocking: the window
/// does not paint again until the user answers, and [`None`] is Cancel.
pub trait FolderPicker {
    /// Ask the user for a folder. [`None`] if they cancelled.
    fn pick(&self, title: &str, start_in: Option<&Path>) -> Option<PathBuf>;
}

/// Somewhere to run a batch that is not the GUI thread.
///
/// `start` returns at once; everything it has to say about the run it says by
/// sending [`Event`]s down `tx`, which the shell drains each frame. Dropping
/// `tx` is how a run says it has nothing more to report.
pub trait BatchRunner {
    /// Begin a run over `inputs` into `out_dir`, reporting on `tx`.
    fn start(&self, inputs: Vec<PathBuf>, out_dir: PathBuf, tx: Sender<Event>);
}

/// The window's state and the two collaborators that give its commands effect.
///
/// Generic rather than boxed: there is exactly one instantiation in the
/// shipped exe (`Shell<RfdPicker, ThreadRunner>`) and one per test, so a
/// vtable would buy nothing.
pub struct Shell<P, R> {
    /// Everything the window paints.
    model: Model,
    /// Opens the folder dialog.
    picker: P,
    /// Runs a batch somewhere that is not this thread.
    runner: R,
    /// The running batch's end of the channel, or [`None`] when no run has
    /// ever been started. Held for exactly as long as the shell wants to hear
    /// from a run.
    progress: Option<Receiver<Event>>,
}

impl<P: FolderPicker, R: BatchRunner> Shell<P, R> {
    /// A shell over `model`, with no run under way.
    #[must_use]
    pub fn new(model: Model, picker: P, runner: R) -> Self {
        Self {
            model,
            picker,
            runner,
            progress: None,
        }
    }

    /// A shell as a launch leaves it (`docs/wiki/architecture.md` decision 6).
    ///
    /// With files on the command line the run starts here, before the first
    /// frame, into the folder [`resolve_out_dir`] chooses - and the model is
    /// built from **that** folder rather than from `settings`, so the path
    /// label says where the files are going from the very first frame even
    /// when the folder is the `cropped` fallback. The fallback is shown and
    /// not remembered: nothing here saves anything, because nothing here was
    /// chosen by the user (see [`Command::SaveSettings`]).
    ///
    /// With no files there is nothing to resolve and nothing to run: the
    /// window opens on whatever `settings` remembered, which may be nothing.
    #[must_use]
    pub fn from_invocation(inv: &Invocation, settings: Settings, picker: P, runner: R) -> Self {
        if inv.inputs.is_empty() {
            return Self::new(Model::new(settings), picker, runner);
        }
        let resolved = resolve_out_dir(inv, &settings);
        let mut shell = Self::new(
            Model::new(Settings {
                output_dir: Some(resolved),
            }),
            picker,
            runner,
        );
        shell.dispatch(Event::FilesDropped(inv.inputs.clone()));
        shell
    }

    /// What the window is showing. The tests' one window onto the shell, and
    /// the app's one source for the paint.
    #[must_use]
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// One frame: read, drain, paint, act.
    ///
    /// `ctx` and `ui` are the same frame's context and root ui; both are
    /// taken because the first is where input and repaints live and the
    /// second is where the paint goes.
    pub fn frame(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let (hovering, dropped) = read_files(ctx);
        if !dropped.is_empty() {
            self.dispatch(Event::FilesDropped(dropped));
        }
        self.drain();

        let folder_button = gui::paint_interactive(ui, &self.model, hovering);

        // After the paint, because a click is what the paint answers with.
        // `add_enabled(false, ..)` is what keeps this silent during a run:
        // a disabled button is out of the Tab order and reports no click, by
        // mouse or by keyboard (`docs/wiki/design/components.md`).
        if folder_button.clicked()
            && let Some(folder) = self
                .picker
                .pick(FOLDER_DIALOG_TITLE, self.model.output_dir.as_deref())
        {
            self.dispatch(Event::FolderChosen(folder));
            // This frame is already painted, so the folder the user just
            // chose needs one more to reach the screen.
            ctx.request_repaint();
        }

        if matches!(self.model.state, AppState::Processing { .. }) {
            ctx.request_repaint();
        }
    }

    /// Everything the worker has said since the last frame, applied in order.
    ///
    /// Collected before it is applied because applying borrows the shell, and
    /// drained rather than blocked on: a frame is never held up by a run.
    fn drain(&mut self) {
        let Some(progress) = &self.progress else {
            return;
        };
        let events: Vec<Event> = progress.try_iter().collect();
        for event in events {
            self.dispatch(event);
        }
    }

    /// Apply `event` to the model and carry out whatever it asks for.
    fn dispatch(&mut self, event: Event) {
        for command in self.model.handle(event) {
            self.perform(command);
        }
    }

    /// Give one [`Command`] effect. The only two effects this window has.
    fn perform(&mut self, command: Command) {
        match command {
            Command::StartBatch { inputs, out_dir } => {
                // A channel per run: the previous receiver goes with the
                // previous run, so a late message from it cannot be read as
                // this one's.
                let (tx, rx) = mpsc::channel();
                self.progress = Some(rx);
                self.runner.start(inputs, out_dir, tx);
            }
            // Deliberately dropped. There is no console to report to, and
            // `voice.md` has no string for a settings file that could not be
            // written: the folder is still chosen and the run will still go
            // there, it just will not be remembered next time.
            Command::SaveSettings(settings) => {
                let _ = settings.save();
            }
        }
    }
}

/// Whether files are being dragged over the window, and any that were dropped
/// on it this frame, in the order the OS listed them.
fn read_files(ctx: &egui::Context) -> (bool, Vec<PathBuf>) {
    ctx.input(|input| {
        (
            !input.raw.hovered_files.is_empty(),
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect(),
        )
    })
}

/// The real runner: one `std::thread` per run, reporting on the channel.
///
/// A unit struct because a run needs nothing but its arguments; the tuning is
/// the engine's default, which is the only one this product exposes.
#[derive(Debug, Default)]
pub struct ThreadRunner;

impl ThreadRunner {
    /// A runner. Stateless, so every one of these is the same runner.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl BatchRunner for ThreadRunner {
    /// Spawn the run and return immediately.
    ///
    /// Every send is allowed to fail: a window closed mid-run drops the
    /// receiver, and a worker shouting at a closed window is not an error -
    /// it is the ordinary way a run outlives what asked for it.
    fn start(&self, inputs: Vec<PathBuf>, out_dir: PathBuf, tx: Sender<Event>) {
        thread::spawn(move || {
            let progress = tx.clone();
            let summary = batch::run(
                &inputs,
                &out_dir,
                &Tuning::default(),
                &|Progress { done, total }| {
                    let _ = progress.send(Event::Progress {
                        done: count(done),
                        total: count(total),
                    });
                },
            );
            let _ = tx.send(Event::Finished(summary));
        });
    }
}

/// The engine counts files in `usize` and the window prints them in `u32`;
/// this is where the two meet. Saturating rather than panicking, for the same
/// reason [`Model`] saturates: a number the window only prints must never be
/// able to end the process.
fn count(files: usize) -> u32 {
    u32::try_from(files).unwrap_or(u32::MAX)
}
