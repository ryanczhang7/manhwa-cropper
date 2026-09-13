//! MC-001: proves the test runner discovers `crates/app/tests/` and that the
//! app library is measured by coverage. The window title is a product
//! decision, so it is pinned here against the literal in
//! `docs/wiki/design/voice.md` rather than against the constant.
//!
//! MC-001's second test - `painted_window_shows_the_title_label_headlessly`,
//! which asserted that `gui::paint` renders a heading carrying the app title -
//! was **retired by MC-015**, not broken by it. The design has no heading
//! (`tokens.md`'s type scale: "heading: reserved; the window has no headings
//! in v1") and `components.md`'s inventory has no such component; the title's
//! home is the native title bar, which `accessibility.md` puts on the Window
//! node. `the_viewport_asks_for_the_window_the_layout_specifies` in
//! `tests/gui.rs` is what covers it now, and that file is where every
//! headless kittest assertion about the painted window lives.

use manhwa_cropper::{APP_TITLE, window_title};

#[test]
fn window_title_is_the_product_name() {
    assert_eq!(window_title(), "Manhwa Cropper");
    assert_eq!(window_title(), APP_TITLE);
}
