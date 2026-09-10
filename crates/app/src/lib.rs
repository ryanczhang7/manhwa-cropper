//! `manhwa-cropper`: the exe's library half.
//!
//! `lib.rs` is the view-model (pure, fully tested, no egui types), `gui.rs`
//! is the egui paint over it, and `main.rs` is the process entry
//! (`docs/wiki/architecture.md`, "manhwa-cropper"). MC-001 ships only the
//! window title and the paint function that shows it; the state machine is
//! MC-014.

#![forbid(unsafe_code)]

pub mod gui;

/// The window title, and the one label the walking-skeleton window shows.
pub const APP_TITLE: &str = "Manhwa Cropper";

/// Title for the native window. A function rather than a bare constant so
/// the app crate's library has a measured line from the start.
#[must_use]
pub fn window_title() -> &'static str {
    APP_TITLE
}
