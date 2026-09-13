//! Process entry. No console window on launch or on Explorer "Send to"
//! (`docs/wiki/stack.md`, "Release shape"), so nothing here may rely on
//! stdout; the exit code and the files written are the contract
//! (`docs/wiki/architecture.md`, "manhwa-cropper" and decision 10): 0 when
//! every input was cropped or flagged, 1 when one of them produced no output
//! at all, 2 on a command line this exe cannot act on. This file only
//! dispatches; the parsing, the folder rule and the run itself live in
//! `cropper-engine`, where they are measured - the `coverage` gate ignores
//! `main.rs` and `gui.rs`, so a decision left here is a decision nothing
//! judges.

#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use cropper_engine::args::{self, Invocation};
use cropper_engine::batch;
use cropper_engine::settings::Settings;
use cropper_engine::{Tuning, resolve_out_dir};

use manhwa_cropper::gui::{self, CropperApp};
use manhwa_cropper::{Model, window_title};

/// Exit code for a run in which every input was cropped or flagged.
const EXIT_OK: u8 = 0;
/// Exit code for a run in which at least one input produced no output.
const EXIT_WRITE_FAILED: u8 = 1;
/// Exit code for a command line that could not be parsed or acted on.
const EXIT_BAD_ARGS: u8 = 2;

fn main() -> ExitCode {
    let Ok(inv) = args::parse(std::env::args().skip(1)) else {
        return ExitCode::from(EXIT_BAD_ARGS);
    };
    if inv.no_gui {
        return headless(&inv);
    }
    match open_window() {
        Ok(()) => ExitCode::from(EXIT_OK),
        Err(_) => ExitCode::from(EXIT_WRITE_FAILED),
    }
}

/// The `--no-gui` path (MC-012): run the batch over every input into the
/// folder [`resolve_out_dir`] chooses, write the JSON summary if `--summary`
/// asked for one, and exit.
///
/// `--no-gui` with no inputs is a command line with nothing to do, and it is
/// rejected *before* a folder is resolved or a run is started: an empty batch
/// would otherwise create the output folder and report a perfectly valid run
/// of zero files (AC-3).
fn headless(inv: &Invocation) -> ExitCode {
    if inv.inputs.is_empty() {
        return ExitCode::from(EXIT_BAD_ARGS);
    }
    // Loaded only when it can change the answer: `--out` wins outright
    // (AC-6), and reading the config folder on every scripted run that named
    // its own destination is a question nobody asked.
    let settings = if inv.out_dir.is_some() {
        Settings::default()
    } else {
        Settings::load()
    };
    // Resolving does not save: a `--out` folder is a one-off and the fallback
    // is not a choice, so nothing here writes settings.json (AC-5, AC-6).
    let out_dir = resolve_out_dir(inv, &settings);
    let summary = batch::run(&inv.inputs, &out_dir, &Tuning::default(), &|_progress| {});
    if let Some(path) = &inv.summary_path {
        // Dropped on purpose: the exit code reports what became of the
        // *inputs*, and an unwritable summary path changes none of that. With
        // no console there is nowhere to say more.
        let _ = batch::write_summary(&summary, path);
    }
    if summary.failed() == 0 {
        ExitCode::from(EXIT_OK)
    } else {
        ExitCode::from(EXIT_WRITE_FAILED)
    }
}

/// Open the window; blocks until it is closed.
///
/// The size, the minimum and the title are `gui::viewport()`, which is what
/// `docs/wiki/design/layout.md` specifies and what MC-015's AC-8 pins. The
/// model starts from the settings on disk, so a remembered folder is on screen
/// from the first frame; MC-016 wires the events that change it.
fn open_window() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: gui::viewport(),
        ..Default::default()
    };
    let model = Model::new(Settings::load());
    eframe::run_native(
        window_title(),
        options,
        Box::new(move |cc| Ok(Box::new(CropperApp::new(cc, model)))),
    )
}
