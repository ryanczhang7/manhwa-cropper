//! `manhwa-cropper`: the exe's library half.
//!
//! `lib.rs` is the view-model (pure, fully tested, no egui types), `gui.rs`
//! is the egui paint over it, and `main.rs` is the process entry
//! (`docs/wiki/architecture.md`, "manhwa-cropper"). MC-001 shipped the window
//! title and the paint function that shows it; MC-014 adds the state machine
//! it paints: [`Model`], the [`Event`]s the window feeds it, the [`Command`]s
//! it asks for back, and every string the window can show.
//!
//! # Why the effects leave as data
//!
//! [`Model::handle`] starts no thread, opens no dialog and touches no file. A
//! drop that should begin a run answers with
//! [`Command::StartBatch`](Command::StartBatch) and a folder the user picked
//! answers with [`Command::SaveSettings`](Command::SaveSettings); MC-016's
//! runner is what carries either of them out. That is what lets the whole of
//! this module be tested by comparing values - `crates/app/tests/model.rs`
//! drives every rule below without a temporary directory, a channel or a
//! window - and it is why the module compiles without egui in scope.
//!
//! # Why every string lives here and not in the painter
//!
//! The text the window shows is a product decision, frozen key by key in
//! `docs/wiki/design/voice.md`; painting it is a layout decision. So the six
//! free functions below own the table, and MC-015 paints what they return
//! without typing a literal of its own. The ellipsis in `Cropping…` is one
//! character (U+2026) and the row separator is space, em dash (U+2014),
//! space, because `voice.md` says so in those words.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use cropper_core::FlagReason;
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Flag, Outcome};
use cropper_engine::settings::Settings;

pub mod gui;
pub mod shell;

/// The window title, and the one label the walking-skeleton window shows.
pub const APP_TITLE: &str = "Manhwa Cropper";

// --- The frozen strings (docs/wiki/design/voice.md) --------------------------
//
// One constant per key in the frozen table, named after the key. They are
// private: a caller outside this crate is meant to ask a function what to
// show, not to assemble a sentence itself, and the test suite deliberately
// compares against literals of its own rather than against these.

/// `dropzone.no_folder` - the product's only instruction.
const DROPZONE_NO_FOLDER: &str = "Choose an output folder, then drop images here";

/// `dropzone.ready`.
const DROPZONE_READY: &str = "Drop images here";

/// `dropzone.hover`.
const DROPZONE_HOVER: &str = "Release to crop";

/// `dropzone.busy`. One U+2026, never three dots.
const DROPZONE_BUSY: &str = "Cropping…";

/// `folder.none`.
const FOLDER_NONE: &str = "No output folder chosen";

/// `folder.button`. One U+2026. No function returns it - it is a caption, not
/// a state - so [`gui`] reads the constant, which keeps the table here.
pub(crate) const FOLDER_BUTTON: &str = "Choose folder…";

/// `folder.dialog_title` - the one argument the native dialog is given. No
/// function returns it either: [`shell`] hands it to the picker exactly as it
/// stands, so the table still lives here and not in the wiring.
pub(crate) const FOLDER_DIALOG_TITLE: &str = "Choose output folder";

/// The separator of `list.row`: space, em dash (U+2014), space.
///
/// Named because it is used three times: [`row_text`] builds a row with it and
/// [`gui`] splits a row on it to give the file name and the reason word their
/// own colours, without either of them typing the characters out.
pub(crate) const ROW_SEPARATOR: &str = " — ";

/// `error.no_folder`.
const ERROR_NO_FOLDER: &str = "No output folder chosen. Choose one, then drop the files again.";

/// `reason.uniform`.
const REASON_UNIFORM: &str = "Blank";

/// `reason.no_border`.
const REASON_NO_BORDER: &str = "No border";

/// `reason.low_content`.
const REASON_LOW_CONTENT: &str = "Low content";

/// `reason.ambiguous`.
const REASON_AMBIGUOUS: &str = "Ambiguous";

/// `reason.unsupported`.
const REASON_UNSUPPORTED: &str = "Unsupported format";

/// `reason.decode_failed`.
const REASON_DECODE_FAILED: &str = "Unreadable";

/// `reason.failed` - the one word that says the file did not reach the output
/// folder at all.
const REASON_FAILED: &str = "Not written";

// --- The state machine -------------------------------------------------------

/// What the window is doing, and therefore what its one content slot shows
/// (`docs/wiki/design/components.md`, "Rules the view-model owns").
///
/// The chosen output folder is **not** here: it survives every one of these
/// states and lives on [`Model`] beside them, so that choosing a folder in
/// the middle of a finished run's summary does not have to rebuild the
/// summary.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Nothing has happened yet, or the last thing that happened has been
    /// cleared. The drop zone is the whole message.
    Idle,
    /// A run is under way: `done` of `total` files are finished.
    ///
    /// `u32` rather than the engine's `usize`: this is a number the window
    /// prints, and MC-016's channel adapter converts once at the boundary
    /// rather than leaking the engine's type into the paint.
    Processing {
        /// How many files have finished.
        done: u32,
        /// How many files the run was given.
        total: u32,
    },
    /// A run finished; its summary is what the slot shows until the next drop.
    Done {
        /// What became of every file in the run that just ended.
        summary: RunSummary,
    },
    /// Something stopped, and the message is the whole explanation - nothing
    /// is logged elsewhere (`voice.md`, "Error-message style").
    Error {
        /// The sentence pair the user reads, already interpolated.
        message: String,
    },
}

/// Something that happened to the window, from the user or from a run.
///
/// Deliberately not the engine's types: [`Progress`](Event::Progress) counts
/// in `u32` where `batch::Progress` counts in `usize`, because this is the
/// number the window prints and MC-016 owns the conversion.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Files were dropped on the window. Possibly none, if the drop carried
    /// nothing this app can use.
    FilesDropped(Vec<PathBuf>),
    /// The user picked an output folder in the dialog.
    FolderChosen(PathBuf),
    /// One more file of the running batch finished.
    Progress {
        /// How many files have finished.
        done: u32,
        /// How many files the run was given.
        total: u32,
    },
    /// The running batch ended and here is what became of every file.
    Finished(RunSummary),
    /// The running batch stopped without a summary; the payload is the
    /// engine's detail, which `error.run_failed` interpolates.
    Failed(String),
}

/// Something the view-model wants done that it will not do itself.
///
/// Every effect in the window is one of these two, returned from
/// [`Model::handle`] as a value. Nothing here has happened yet when it is
/// returned.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Run the engine over `inputs`, writing into `out_dir`.
    StartBatch {
        /// The dropped files, in the order they were dropped.
        inputs: Vec<PathBuf>,
        /// The folder the user chose.
        out_dir: PathBuf,
    },
    /// Persist the settings, because the folder in them just changed.
    ///
    /// Only a folder the *user* chose produces this. A folder that merely
    /// arrived in [`Model::new`] - a remembered one, or MC-016's Send-to
    /// fallback - is not saved back, which is what keeps a one-off fallback
    /// from becoming the remembered folder.
    SaveSettings(Settings),
}

/// The whole of the window's state: one variant of [`AppState`] and the
/// output folder that outlives it.
///
/// Public fields because they are the contract: MC-015 reads them to decide
/// what to paint, and the tests read them to decide whether a rule holds.
/// Nothing here is derived or cached, so there is no way for the two to
/// disagree.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// What the window is doing.
    pub state: AppState,
    /// Where crops go, once a folder is known.
    pub output_dir: Option<PathBuf>,
}

impl Model {
    /// A window as a launch leaves it: [`Idle`](AppState::Idle), showing
    /// whatever folder `settings` resolved to.
    ///
    /// Takes the settings by value because this is the end of their journey -
    /// the app loads them once and hands them over. A folder here is shown
    /// from the first frame and **not** saved back; see
    /// [`Command::SaveSettings`].
    #[must_use]
    pub fn new(settings: Settings) -> Self {
        Self {
            state: AppState::Idle,
            output_dir: settings.output_dir,
        }
    }

    /// Apply `event` and answer with what should happen next.
    ///
    /// The returned commands are the only effects: an empty vector means the
    /// window has nothing to ask anyone for, which is the answer to every
    /// event that is ignored.
    #[must_use]
    pub fn handle(&mut self, event: Event) -> Vec<Command> {
        match event {
            Event::FilesDropped(inputs) => self.files_dropped(inputs),
            Event::FolderChosen(folder) => self.folder_chosen(folder),
            // Progress, Finished and Failed describe the run that is under
            // way, so outside `Processing` they describe a run that is over:
            // a late message from a batch whose result the user has already
            // read, or that a `Failed` already closed. Ignoring a stale event
            // is this window's house style - it is what a drop during a run
            // gets - and it is the only choice here that cannot overwrite
            // what the user is currently looking at. In particular a second
            // `Finished` cannot replace the summary of the run that is on
            // screen: `Finished` replaces an earlier summary by ending a
            // *newer run*, never by arriving twice.
            Event::Progress { done, total } => {
                if self.is_processing() {
                    self.state = AppState::Processing { done, total };
                }
                Vec::new()
            }
            Event::Finished(summary) => {
                if self.is_processing() {
                    self.state = AppState::Done { summary };
                }
                Vec::new()
            }
            Event::Failed(detail) => {
                if self.is_processing() {
                    self.state = AppState::Error {
                        message: run_failed(&detail),
                    };
                }
                Vec::new()
            }
        }
    }

    /// Files were dropped: start a batch, refuse for want of a folder, or
    /// ignore the drop.
    ///
    /// A drop during a run is ignored in silence - no state change and no
    /// message - because the disabled drop zone and its `dropzone.busy` text
    /// are the only signal the design wants (ratified designer decision 1;
    /// queueing is a separate story). A drop of nothing at all is ignored
    /// everywhere for the same reason: a drag that carried no usable file is
    /// not a request, and refusing it would put an error on screen for an
    /// action the user did not take.
    fn files_dropped(&mut self, inputs: Vec<PathBuf>) -> Vec<Command> {
        if inputs.is_empty() || self.is_processing() {
            return Vec::new();
        }
        let Some(out_dir) = self.output_dir.clone() else {
            self.state = AppState::Error {
                message: ERROR_NO_FOLDER.to_owned(),
            };
            return Vec::new();
        };
        self.state = AppState::Processing {
            done: 0,
            // A drop of more than four billion files is not a drop; saturating
            // rather than panicking keeps a number the window only prints from
            // ever being able to end the process.
            total: u32::try_from(inputs.len()).unwrap_or(u32::MAX),
        };
        vec![Command::StartBatch { inputs, out_dir }]
    }

    /// The user picked a folder: remember it, save it, and leave the slot
    /// alone - except for an error, which the new folder has probably just
    /// fixed.
    ///
    /// During a run the choice is dropped entirely: the batch already has its
    /// output folder, and saving a different one mid-run would remember a
    /// folder the run did not use.
    fn folder_chosen(&mut self, folder: PathBuf) -> Vec<Command> {
        if self.is_processing() {
            return Vec::new();
        }
        if matches!(self.state, AppState::Error { .. }) {
            self.state = AppState::Idle;
        }
        self.output_dir = Some(folder.clone());
        vec![Command::SaveSettings(Settings {
            output_dir: Some(folder),
        })]
    }

    /// Whether a run is under way. The guard on every rule that has one.
    fn is_processing(&self) -> bool {
        matches!(self.state, AppState::Processing { .. })
    }
}

// --- The strings the window shows --------------------------------------------

/// `result.line`, with `result.failed_suffix` appended only when a file was
/// lost.
///
/// The counts come from the summary's own derived counters rather than from a
/// count taken here: there is one place a result's outcome is decided, and it
/// is the engine's `process_file`.
#[must_use]
pub fn result_line(summary: &RunSummary) -> String {
    let mut line = format!(
        "{} cropped, {} flagged",
        summary.cropped(),
        summary.flagged()
    );
    let failed = summary.failed();
    if failed > 0 {
        line.push_str(&format!(", {failed} failed"));
    }
    line
}

/// `list.row` for one result: its file name, a space, an em dash, a space, and
/// the word for why it is in the list.
#[must_use]
pub fn row_text(result: &FileResult) -> String {
    format!(
        "{}{ROW_SEPARATOR}{}",
        file_name(&result.input),
        reason_word(&result.outcome)
    )
}

/// The rows of the result list: every flagged and failed file, in input order,
/// and nothing else.
///
/// A cropped file is the ordinary case and has no row - the result line's
/// count is the whole of what the user is told about it.
#[must_use]
pub fn rows(summary: &RunSummary) -> Vec<String> {
    summary
        .results
        .iter()
        .filter(|result| !matches!(result.outcome, Outcome::Cropped { .. }))
        .map(row_text)
        .collect()
}

/// The drop zone's one line of text.
///
/// The order of the arms is the rule: a run in progress beats a hovering drag,
/// because a drop made during a run is ignored and `Release to crop` would
/// promise otherwise; and a missing folder beats both, because choosing one is
/// what the user must do first. `Processing` with no folder is unreachable -
/// [`Model::handle`] cannot start a batch without a folder - and falls
/// out as `dropzone.busy`, which is what the state says is true rather than an
/// invented string for a cell `voice.md` does not have.
#[must_use]
pub fn dropzone_text(model: &Model, hovering: bool) -> &'static str {
    match (&model.state, &model.output_dir, hovering) {
        (AppState::Processing { .. }, _, _) => DROPZONE_BUSY,
        (_, None, _) => DROPZONE_NO_FOLDER,
        (_, Some(_), true) => DROPZONE_HOVER,
        (_, Some(_), false) => DROPZONE_READY,
    }
}

/// The path label: the whole folder as Windows displays it, or `folder.none`.
///
/// Unquoted and untruncated. Fitting a long path into a narrow label is the
/// painter's problem (MC-015 truncates on the left, so the leaf stays
/// readable), and a truncation done here would be one the accessible name
/// could not undo.
#[must_use]
pub fn path_text(model: &Model) -> String {
    model
        .output_dir
        .as_ref()
        .map_or_else(|| FOLDER_NONE.to_owned(), |dir| dir.display().to_string())
}

/// `progress.count`: how far a run has got.
#[must_use]
pub fn progress_text(done: u32, total: u32) -> String {
    format!("{done} of {total}")
}

/// The progress bar's fill: `components.md` gives its width as `done / total`.
///
/// `progress_text`'s twin - the same two numbers read as a length instead of
/// as a count - and it lives here rather than in the painter so it can be
/// compared as a value (the audit's Decided-3). Answers `0.0` for a run of no
/// files, because `0 / 0` is not a length a bar can have, and never more than
/// `1.0`, because a late or duplicated `Progress` may carry `done > total` and
/// the window must not paint past its own end.
#[must_use]
pub fn progress_fraction(done: u32, total: u32) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (done as f32 / total as f32).min(1.0)
}

/// The window title, and the one label the walking-skeleton window shows.
#[must_use]
pub fn window_title() -> &'static str {
    APP_TITLE
}

/// `error.run_failed`, with `voice.md`'s `{detail}` rule applied: one trailing
/// full stop is removed, so the engine's `disk full.` does not read
/// `…disk full.. Drop…`.
fn run_failed(detail: &str) -> String {
    let detail = detail.strip_suffix('.').unwrap_or(detail);
    format!("Cropping stopped: {detail}. Drop the files again.")
}

/// The word `list.row` shows for an outcome.
///
/// A cropped file shares the `reason.failed` arm because it has no word of its
/// own: `rows` filters cropped results out, so the only way to reach it is to
/// ask for the row of a file that has none, and `voice.md` has no string to
/// answer with. Folding it in keeps the alternative - inventing a seventh
/// reason word, which is a design decision this story has no mandate to make -
/// out of the product.
fn reason_word(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Flagged { reason, .. } => match reason {
            Flag::Detector(FlagReason::Uniform) => REASON_UNIFORM,
            Flag::Detector(FlagReason::NoBorderFound) => REASON_NO_BORDER,
            Flag::Detector(FlagReason::LowContent) => REASON_LOW_CONTENT,
            Flag::Detector(FlagReason::Ambiguous) => REASON_AMBIGUOUS,
            Flag::Unsupported => REASON_UNSUPPORTED,
            Flag::DecodeFailed(_) => REASON_DECODE_FAILED,
        },
        Outcome::Cropped { .. } | Outcome::Failed { .. } => REASON_FAILED,
    }
}

/// `path`'s file name, or the whole path when it has none (`..`, a bare root).
///
/// Lossy rather than fallible, and the same rule as the engine's own
/// `batch::file_name`: a name this program cannot decode is still a name the
/// user has to be shown.
fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}
