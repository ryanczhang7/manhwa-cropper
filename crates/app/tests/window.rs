//! MC-001: proves the test runner discovers `crates/app/tests/`, that the
//! app library is measured by coverage, and that `egui_kittest` can drive the
//! paint function without a display (AC-5).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use manhwa_cropper::{APP_TITLE, gui, window_title};

#[test]
fn window_title_is_the_product_name() {
    assert_eq!(window_title(), "Manhwa Cropper");
    assert_eq!(window_title(), APP_TITLE);
}

#[test]
fn painted_window_shows_the_title_label_headlessly() {
    let mut harness = Harness::new_ui(gui::paint);
    harness.run();
    // Panics with a clear message if no node carries this label.
    harness.get_by_label(APP_TITLE);
}
