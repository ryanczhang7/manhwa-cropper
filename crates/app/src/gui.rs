//! The egui shell. Immediate-mode paint code, excluded from line coverage by
//! the coverage gate (`docs/wiki/stack.md`, "Gates") and checked headlessly
//! with `egui_kittest` instead (`crates/app/tests/window.rs`).

use eframe::egui;

use crate::window_title;

/// Paints one frame of the window into `ui`. Split out from
/// [`CropperApp::update`] so the kittest harness can drive it without a
/// native window.
pub fn paint(ui: &mut egui::Ui) {
    ui.heading(window_title());
}

/// The eframe application. MC-001 has no state; MC-014 adds the view-model.
#[derive(Debug, Default)]
pub struct CropperApp;

impl eframe::App for CropperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, paint);
    }
}
