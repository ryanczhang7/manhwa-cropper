//! Process entry. No console window on launch or on Explorer "Send to"
//! (`docs/wiki/stack.md`, "Release shape"), so nothing here may rely on
//! stdout; the exit code and the files written are the contract.

#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]

use eframe::egui;
use manhwa_cropper::gui::CropperApp;
use manhwa_cropper::window_title;

fn main() -> eframe::Result {
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
