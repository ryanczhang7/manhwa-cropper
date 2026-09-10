//! Process entry. No console window on launch or on Explorer "Send to"
//! (`docs/wiki/stack.md`, "Release shape"), so nothing here may rely on
//! stdout; the exit code and the files written are the contract
//! (`docs/wiki/architecture.md`, "manhwa-cropper"): 0 when every input was
//! written, 1 when one could not be, 2 on bad arguments. This file only
//! dispatches; the parsing and the copying live in `cropper-engine`, where
//! they are measured (`main.rs` is excluded from line coverage).

#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]

use std::process::ExitCode;

use cropper_engine::args::{self, Invocation};
use cropper_engine::copy::copy_all;
use eframe::egui;
use manhwa_cropper::gui::CropperApp;
use manhwa_cropper::window_title;

/// Exit code for a run in which every input was written.
const EXIT_OK: u8 = 0;
/// Exit code for a run in which at least one input could not be written.
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

/// The `--no-gui` path: copy every input into `--out` and exit. `--out` is
/// mandatory until MC-012 supplies the fallback folder; without it this is
/// a bad command line (2), and nothing is written.
fn headless(inv: &Invocation) -> ExitCode {
    let Some(out_dir) = inv.out_dir.as_deref() else {
        return ExitCode::from(EXIT_BAD_ARGS);
    };
    if copy_all(&inv.inputs, out_dir) {
        ExitCode::from(EXIT_OK)
    } else {
        ExitCode::from(EXIT_WRITE_FAILED)
    }
}

/// Open the window as MC-001 did; blocks until it is closed.
fn open_window() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(window_title())
            .with_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        window_title(),
        options,
        Box::new(|_cc| Ok(Box::new(CropperApp))),
    )
}
