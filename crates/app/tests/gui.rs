//! MC-015: what `gui::paint` puts in the accessibility tree for every window
//! state, what `gui::install_style` does to the palette, and what
//! `gui::viewport` asks the OS for.
//!
//! # How these tests read the window
//!
//! `egui_kittest` renders a frame headlessly and exposes the AccessKit tree
//! that eframe would hand Narrator. Every assertion here is a query against
//! that tree by the roles and names in `docs/wiki/design/accessibility.md` -
//! never a pixel, never a rect, never a colour of a painted shape. That is
//! deliberate: the design's second carrier for every meaning is *text*
//! (`accessibility.md`, "Colour is never the only carrier"), so text is what a
//! test can hold the painter to.
//!
//! # Why every string below is a literal
//!
//! The expected strings are copied character for character out of
//! `docs/wiki/design/voice.md`'s frozen table, not read back from
//! `manhwa_cropper`'s own `dropzone_text` / `path_text` / `result_line`.
//! `assert_eq!(painted, dropzone_text(&model, false))` would pass for any
//! table at all, including an empty one; it asserts that the painter and the
//! view-model agree, which is not what the criteria say. MC-014's own suite
//! (`tests/model.rs`) pins the functions against the same literals, so the two
//! files meet at `voice.md` and nowhere else.
//!
//! # Why the theme test drives `RawInput::system_theme`
//!
//! `tokens.md` says the window follows the system theme only, through
//! `ThemePreference::System`. `HarnessBuilder` forces a fixed preference
//! (`Harness::from_builder` calls `ctx.set_theme(Theme::Dark)`), so the theme
//! test restores `ThemePreference::System` itself - standing in for eframe's
//! default, which the harness does not reproduce. That call belongs to the
//! test, not to `install_style`: the design note forbids the painter from
//! touching `theme_preference`.

use std::path::PathBuf;

use cropper_core::{FlagReason, Rect};
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Flag, Outcome};
use eframe::egui;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{By, NodeT, Queryable};
use manhwa_cropper::{AppState, Model, gui};

// --- The frozen strings (docs/wiki/design/voice.md) --------------------------

/// `dropzone.no_folder`.
const DROPZONE_NO_FOLDER: &str = "Choose an output folder, then drop images here";
/// `dropzone.ready`.
const DROPZONE_READY: &str = "Drop images here";
/// `dropzone.hover`.
const DROPZONE_HOVER: &str = "Release to crop";
/// `dropzone.busy`. One U+2026, never three dots.
const DROPZONE_BUSY: &str = "Cropping…";
/// `folder.none`.
const FOLDER_NONE: &str = "No output folder chosen";
/// `folder.button`. One U+2026.
const FOLDER_BUTTON: &str = "Choose folder…";
/// `error.no_folder`.
const ERROR_NO_FOLDER: &str = "No output folder chosen. Choose one, then drop the files again.";
/// `error.run_failed` with `{detail}` = `the disk is full`.
const ERROR_RUN_FAILED: &str = "Cropping stopped: the disk is full. Drop the files again.";

/// The output folder AC-2 names. A Windows path, shown unquoted, backslashes
/// and all.
const OUT_DIR: &str = r"D:\shots\out";

// --- The frozen numbers (docs/wiki/design/layout.md, tokens.md) --------------

/// `window.title`.
const WINDOW_TITLE: &str = "Manhwa Cropper";
/// Default inner size, logical px.
const DEFAULT_SIZE: [f32; 2] = [520.0, 440.0];
/// Minimum inner size, logical px.
const MIN_SIZE: [f32; 2] = [400.0, 320.0];
/// `surface`, light: `#F3F3F3`.
const SURFACE_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xF3, 0xF3, 0xF3);
/// `surface`, dark: `#202020`.
const SURFACE_DARK: egui::Color32 = egui::Color32::from_rgb(0x20, 0x20, 0x20);

// --- Building a window to look at --------------------------------------------

/// Render one frame of `model` at the window's default size and hand back the
/// tree.
///
/// 520 x 440 because `layout.md` sizes the flagged list from it; a narrower
/// harness would give the list fewer rows than the design says it has and
/// would make a row-count assertion a measurement of the test's own window.
fn painted(model: &Model, hovering: bool) -> Harness<'_> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(DEFAULT_SIZE[0], DEFAULT_SIZE[1]))
        .build_ui(move |ui| gui::paint(ui, model, hovering));
    harness.run();
    harness
}

fn model(state: AppState, output_dir: Option<&str>) -> Model {
    Model {
        state,
        output_dir: output_dir.map(PathBuf::from),
    }
}

/// An input file somewhere that is not the output folder, so a row asserting a
/// file *name* fails if the painter shows a whole path.
fn input(name: &str) -> PathBuf {
    PathBuf::from(format!(r"C:\Users\ryan\Downloads\{name}"))
}

fn cropped(name: &str) -> FileResult {
    FileResult {
        input: input(name),
        outcome: Outcome::Cropped {
            rect: Rect {
                x: 0,
                y: 0,
                w: 800,
                h: 1200,
            },
            output: PathBuf::from(OUT_DIR).join(name),
        },
    }
}

fn flagged(name: &str, reason: Flag) -> FileResult {
    FileResult {
        input: input(name),
        outcome: Outcome::Flagged {
            reason,
            output: PathBuf::from(OUT_DIR).join(name),
        },
    }
}

fn not_written(name: &str) -> FileResult {
    FileResult {
        input: input(name),
        outcome: Outcome::Failed {
            error: "The disk is full.".to_owned(),
        },
    }
}

/// AC-4's run: ten files, every one cropped.
fn all_cropped() -> RunSummary {
    RunSummary {
        results: (0..10).map(|i| cropped(&format!("page_{i}.png"))).collect(),
    }
}

/// AC-5's run: 7 cropped, 2 flagged, 1 not written, with the three rows
/// **interleaved** among the cropped files - none of them first, none of them
/// last, never two adjacent - so that a painter which collects the rows by
/// walking a contiguous slice, or which drops the head or the tail of the
/// list, fails here.
///
/// Their relative order is the one AC-5 fixes: `a.png`, `b.gif`, `c.png`.
/// That order is also "every flagged file, then the failed one", so this
/// fixture alone cannot tell `RunSummary.results` order apart from a painter
/// that groups flagged before failed. MC-014's `tests/model.rs` already can -
/// its `mixed_summary` puts a *failed* file between two flagged ones and pins
/// `rows()` against it - and the painter gets its rows from `rows()`, so the
/// grouping mutation dies one layer down. Noted in the story's Test plan.
fn mixed() -> RunSummary {
    RunSummary {
        results: vec![
            cropped("page_0.png"),
            flagged("a.png", Flag::Detector(FlagReason::Uniform)),
            cropped("page_2.png"),
            flagged("b.gif", Flag::Unsupported),
            cropped("page_3.png"),
            cropped("page_5.png"),
            not_written("c.png"),
            cropped("page_7.png"),
            cropped("page_8.png"),
            cropped("page_9.png"),
        ],
    }
}

// --- Reading the tree --------------------------------------------------------

/// How many Label nodes carry exactly `text` as their accessible name.
///
/// Exact, and role-qualified: `accessibility.md` puts every string of the
/// window on a Label, and a role-blind query would confuse the progress bar
/// (role ProgressIndicator) with the count beneath it if the painter ever
/// named both.
fn labels_named<'a>(harness: &'a Harness<'_>, text: &'a str) -> usize {
    harness
        .query_all_by_role_and_label(Role::Label, text)
        .count()
}

/// How many Label nodes contain `needle` anywhere in their accessible name.
fn labels_containing<'a>(harness: &'a Harness<'_>, needle: &'a str) -> usize {
    harness
        .query_all(By::new().role(Role::Label).label_contains(needle))
        .count()
}

/// The `Choose folder…` button, or a panic naming the query that missed.
fn folder_button<'a>(harness: &'a Harness<'a>) -> egui_kittest::Node<'a> {
    harness.get_by_role_and_label(Role::Button, FOLDER_BUTTON)
}

/// Every flagged-list row's text, in the order the tree holds them.
///
/// Reads `Node::value` because AccessKit stores a `Role::Label` node's name
/// there. Selecting rows by the em dash is what makes a painter that splits a
/// row into two Labels fail here rather than pass: `a.png` on its own does not
/// contain ` — `, so a two-Label row is either invisible to this query or
/// reads as ` — Blank`, and neither equals the row the design specifies.
fn row_texts(harness: &Harness<'_>) -> Vec<String> {
    harness
        .query_all(By::new().role(Role::Label).label_contains(" — "))
        .filter_map(|node| node.accesskit_node().value())
        .collect()
}

/// What the status slot and the list hold, counted by the queries the absence
/// criteria use.
///
/// The four fields are the four things AC-1 and AC-4 say are *not* there. They
/// are counted by one helper so that the absence assertions and their positive
/// twins (`the_absence_queries_find_each_node_in_the_state_that_must_have_it`)
/// are literally the same query and not two that merely look alike.
#[derive(Debug, PartialEq, Eq)]
struct Census {
    /// Nodes with role ProgressIndicator.
    progress_bars: usize,
    /// Labels reading `{done} of {total}`.
    progress_counts: usize,
    /// Labels reading `{cropped} cropped, {flagged} flagged`.
    result_lines: usize,
    /// Labels reading `{filename} — {reason}`.
    rows: usize,
}

fn census(harness: &Harness<'_>) -> Census {
    Census {
        progress_bars: harness.query_all_by_role(Role::ProgressIndicator).count(),
        progress_counts: labels_containing(harness, " of "),
        result_lines: labels_containing(harness, " cropped, "),
        rows: labels_containing(harness, " — "),
    }
}

// --- AC-1: idle, no folder ---------------------------------------------------

#[test]
fn idle_with_no_folder_shows_the_instruction_and_an_enabled_button() {
    let model = model(AppState::Idle, None);
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, DROPZONE_NO_FOLDER),
        1,
        "the drop zone must show the product's one instruction when no folder is chosen"
    );
    assert_eq!(
        labels_named(&harness, FOLDER_NONE),
        1,
        "the path label must read `No output folder chosen` when no folder is chosen"
    );
    assert!(
        !folder_button(&harness).accesskit_node().is_disabled(),
        "the `Choose folder…` button is the way out of this state, so it must be enabled"
    );
}

#[test]
fn idle_with_no_folder_has_an_empty_status_slot_and_no_list_rows() {
    let model = model(AppState::Idle, None);
    let harness = painted(&model, false);

    assert_eq!(
        census(&harness),
        Census {
            progress_bars: 0,
            progress_counts: 0,
            result_lines: 0,
            rows: 0,
        },
        "Idle shows nothing in the status slot and paints no list rows"
    );
}

// --- AC-2: idle, folder chosen, and the hover overlay ------------------------

#[test]
fn idle_with_a_folder_reads_drop_images_here_and_shows_the_path() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        1,
        "with a folder chosen the drop zone drops the instruction and reads `Drop images here`"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_NO_FOLDER),
        0,
        "the instruction must be gone once a folder is chosen"
    );
    assert_eq!(
        labels_named(&harness, OUT_DIR),
        1,
        "the path label shows the chosen folder as Windows writes it, unquoted"
    );
    assert_eq!(
        labels_named(&harness, FOLDER_NONE),
        0,
        "`No output folder chosen` must not survive a folder being chosen"
    );
}

#[test]
fn hovering_files_over_a_chosen_folder_reads_release_to_crop() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, true);

    assert_eq!(
        labels_named(&harness, DROPZONE_HOVER),
        1,
        "a drag over the window must promise what a release will do"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        0,
        "`Drop images here` is replaced while files hover, not shown alongside"
    );
}

#[test]
fn hovering_files_with_no_folder_keeps_the_instruction() {
    let model = model(AppState::Idle, None);
    let harness = painted(&model, true);

    assert_eq!(
        labels_named(&harness, DROPZONE_NO_FOLDER),
        1,
        "with no folder a drop cannot be honoured, so the zone keeps telling the user to \
         choose one"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_HOVER),
        0,
        "`Release to crop` would promise a crop the model refuses without a folder"
    );
}

#[test]
fn a_long_output_path_keeps_its_whole_path_as_the_accessible_name() {
    let long = r"C:\Users\ryan\Pictures\manhwa\2026\september\cropped-output-folder";
    let model = model(AppState::Idle, Some(long));
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, long),
        1,
        "the path label truncates on screen but its accessible name is the full path, so a \
         screen reader and a test can both still find it"
    );
}

// --- AC-3: processing --------------------------------------------------------

#[test]
fn processing_reads_cropping_shows_the_count_and_disables_the_button() {
    let model = model(AppState::Processing { done: 3, total: 10 }, Some(OUT_DIR));
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, DROPZONE_BUSY),
        1,
        "a run under way must say so in the drop zone: `Cropping…`, one U+2026"
    );
    assert_eq!(
        labels_named(&harness, "3 of 10"),
        1,
        "the progress count is the tested surface for how far the run has got"
    );
    assert!(
        folder_button(&harness).accesskit_node().is_disabled(),
        "the button leaves the Tab order during a run, which AccessKit reports as disabled"
    );
}

#[test]
fn processing_ignores_a_hovering_drag() {
    let model = model(AppState::Processing { done: 3, total: 10 }, Some(OUT_DIR));
    let harness = painted(&model, true);

    assert_eq!(
        labels_named(&harness, DROPZONE_BUSY),
        1,
        "a drop during a run is ignored, so the zone keeps reading `Cropping…` under hover"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_HOVER),
        0,
        "`Release to crop` during a run would promise something the view-model discards"
    );
}

// --- AC-4: done, nothing flagged ---------------------------------------------

#[test]
fn done_with_nothing_flagged_shows_the_result_line_and_no_list_rows() {
    let model = model(
        AppState::Done {
            summary: all_cropped(),
        },
        Some(OUT_DIR),
    );
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, "10 cropped, 0 flagged"),
        1,
        "ten cropped files read `10 cropped, 0 flagged` - digits, zeros shown, no `0 failed`"
    );
    assert_eq!(
        census(&harness).rows,
        0,
        "nothing was flagged or lost, so the list paints nothing at all"
    );
}

// --- AC-5: done with flags ---------------------------------------------------

#[test]
fn done_with_flags_shows_the_failed_suffix_and_three_rows_in_summary_order() {
    let model = model(AppState::Done { summary: mixed() }, Some(OUT_DIR));
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, "7 cropped, 2 flagged, 1 failed"),
        1,
        "a run that lost a file appends `, 1 failed` to the result line"
    );
    assert_eq!(
        row_texts(&harness),
        vec![
            "a.png — Blank".to_owned(),
            "b.gif — Unsupported format".to_owned(),
            "c.png — Not written".to_owned(),
        ],
        "exactly the flagged and failed files, each one node, in the summary's own order"
    );
}

// --- AC-6: error -------------------------------------------------------------

#[test]
fn an_error_with_a_folder_shows_the_message_and_the_ready_drop_zone() {
    let model = model(
        AppState::Error {
            message: ERROR_RUN_FAILED.to_owned(),
        },
        Some(OUT_DIR),
    );
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, ERROR_RUN_FAILED),
        1,
        "the message is the whole explanation, so it must be in the tree word for word"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_READY),
        1,
        "an error lives in the status slot; the zone returns to the text for the folder state"
    );
    assert_eq!(
        labels_named(&harness, OUT_DIR),
        1,
        "the chosen folder survives an error"
    );
}

#[test]
fn an_error_with_no_folder_shows_the_message_and_the_instruction() {
    let model = model(
        AppState::Error {
            message: ERROR_NO_FOLDER.to_owned(),
        },
        None,
    );
    let harness = painted(&model, false);

    assert_eq!(
        labels_named(&harness, ERROR_NO_FOLDER),
        1,
        "the message is the whole explanation, so it must be in the tree word for word"
    );
    assert_eq!(
        labels_named(&harness, DROPZONE_NO_FOLDER),
        1,
        "with no folder the zone shows the instruction, error or no error"
    );
    assert_eq!(
        labels_named(&harness, FOLDER_NONE),
        1,
        "the path label still reads `No output folder chosen`, distinct from the message"
    );
}

// --- AC-7: the palette follows the system theme ------------------------------

#[test]
fn the_window_follows_the_system_theme() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let mut harness = painted(&model, false);

    gui::install_style(&harness.ctx);
    // eframe leaves the preference at `ThemePreference::System`; kittest's
    // builder does not, so the test puts it back. See the module docs.
    harness.ctx.set_theme(egui::ThemePreference::System);

    harness.input_mut().system_theme = Some(egui::Theme::Dark);
    harness.step();
    let visuals = harness.ctx.global_style().visuals.clone();
    assert!(
        visuals.dark_mode,
        "a dark system theme must put egui in dark mode"
    );
    assert_eq!(
        visuals.panel_fill, SURFACE_DARK,
        "the window background is the dark `surface` token, #202020"
    );
    assert_eq!(
        visuals.window_fill, visuals.panel_fill,
        "`surface` is one role: window fill and panel fill are the same colour"
    );

    harness.input_mut().system_theme = Some(egui::Theme::Light);
    harness.step();
    let visuals = harness.ctx.global_style().visuals.clone();
    assert!(
        !visuals.dark_mode,
        "a light system theme must put egui in light mode on the next frame"
    );
    assert_eq!(
        visuals.panel_fill, SURFACE_LIGHT,
        "the window background is the light `surface` token, #F3F3F3"
    );
    assert_eq!(
        visuals.window_fill, visuals.panel_fill,
        "`surface` is one role: window fill and panel fill are the same colour"
    );
}

// --- AC-8: the window the OS is asked for ------------------------------------

#[test]
fn the_viewport_asks_for_the_window_the_layout_specifies() {
    let viewport = gui::viewport();

    assert_eq!(
        viewport.title.as_deref(),
        Some(WINDOW_TITLE),
        "the native title bar carries the product name"
    );
    assert_eq!(
        viewport.inner_size,
        Some(egui::vec2(DEFAULT_SIZE[0], DEFAULT_SIZE[1])),
        "the window opens at 520 x 440, where the flagged list is eight rows"
    );
    assert_eq!(
        viewport.min_inner_size,
        Some(egui::vec2(MIN_SIZE[0], MIN_SIZE[1])),
        "the OS refuses smaller than 400 x 320, where the list is still three rows"
    );
}

// --- The negative control for every absence asserted above -------------------

#[test]
fn the_absence_queries_find_each_node_in_the_state_that_must_have_it() {
    // `idle_with_no_folder_has_an_empty_status_slot_and_no_list_rows` and
    // `done_with_nothing_flagged_shows_the_result_line_and_no_list_rows`
    // assert that four things are missing. A query that can never find its
    // node would satisfy both while proving nothing, so each of the four is
    // run again here against the window state the design says must contain
    // it. Expected: 1, 1, 1, 3.
    let processing = model(AppState::Processing { done: 3, total: 10 }, Some(OUT_DIR));
    let harness = painted(&processing, false);
    let counted = census(&harness);
    assert_eq!(
        counted.progress_bars, 1,
        "the ProgressIndicator query must find the bar while a run is under way, or AC-1's \
         `no progress bar in Idle` means nothing"
    );
    assert_eq!(
        counted.progress_counts, 1,
        "the `{{done}} of {{total}}` query must find the count while a run is under way, or \
         AC-1's `no progress count in Idle` means nothing"
    );

    let done = model(AppState::Done { summary: mixed() }, Some(OUT_DIR));
    let harness = painted(&done, false);
    let counted = census(&harness);
    assert_eq!(
        counted.result_lines, 1,
        "the result-line query must find the line in Done, or AC-1's `no result line in \
         Idle` means nothing"
    );
    assert_eq!(
        counted.rows, 3,
        "the row query must find all three rows in Done-with-flags, or AC-1's and AC-4's \
         `no list row` mean nothing"
    );
}

// --- MC-023 / AC-4: where the bar's fill fraction is computed ----------------
//
// The AccessKit tree cannot answer this one, and that is settled rather than
// discovered: it carries roles and names, not colours and not a bar's fill,
// and `progress_bar` is *deliberately* unnamed - its own doc comment and
// `components.md` say the `{done} of {total}` Label beneath it is the
// accessible surface. So no `By`/`Role` query in this file can see a fraction,
// and the audit's Decided-2 settles that no story should invent one.
//
// The claim is therefore made against the painter's own source, read at
// compile time. It is a coarse instrument and it is the right one here: what
// AC-4 asks is precisely a question about *where the code is*, not about what
// a rendered frame looks like.

/// The painter's source, read at compile time. `include_str!` resolves
/// relative to this file, so this is `crates/app/src/gui.rs`.
const PAINTER_SOURCE: &str = include_str!("../src/gui.rs");

/// The arithmetic `gui.rs` may not contain anywhere: the bar's fill fraction
/// is the view-model's to compute (`lib.rs::progress_fraction`).
///
/// The patterns name `done` and `total` rather than division in general,
/// because `gui.rs` has one legitimate f32 division - `step as f32 /
/// SEGMENTS as f32` in the corner-arc helper - and a rule that banned that
/// would have to be weakened the first time it fired.
const FRACTION_ARITHMETIC: &[&str] = &[
    "done as f32",
    "total as f32",
    "total == 0",
    "total != 0",
    "/ total",
    "done /",
];

/// The body of a top-level `fn`, from its signature to the closing brace in
/// column zero. Sound for rustfmt-formatted Rust, which is what the `format`
/// gate keeps this file as. CRLF is normalised first so the test reads the
/// same on a checkout that ignored `.gitattributes`.
fn body_of(source: &str, signature_prefix: &str) -> String {
    let source = source.replace("\r\n", "\n");
    let start = source
        .find(signature_prefix)
        .unwrap_or_else(|| panic!("gui.rs has no `{signature_prefix}`"));
    let rest = &source[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("`{signature_prefix}` has no closing brace in column zero"));
    rest[..end].to_owned()
}

/// AC-4, first half: the fraction the painter hands `egui::ProgressBar` is the
/// value the view-model returned.
#[test]
fn the_painter_takes_the_bars_fill_fraction_from_the_view_model() {
    let body = body_of(PAINTER_SOURCE, "fn progress_bar(");

    assert!(
        body.contains("progress_fraction(done, total)"),
        "`progress_bar` never calls the view-model's `progress_fraction(done, total)`:\n{body}"
    );

    let passed_straight_in = body.contains("ProgressBar::new(progress_fraction(done, total))");
    let bound_then_passed = body.contains("let fraction = progress_fraction(done, total);")
        && body.contains("ProgressBar::new(fraction)");
    assert!(
        passed_straight_in || bound_then_passed,
        "the value `progress_bar` hands `egui::ProgressBar::new` is not the one \
         `progress_fraction` returned:\n{body}"
    );
}

/// AC-4, second half: and it is computed nowhere else in `gui.rs`.
#[test]
fn the_painter_computes_no_fill_fraction_of_its_own() {
    let found: Vec<&str> = FRACTION_ARITHMETIC
        .iter()
        .copied()
        .filter(|pattern| PAINTER_SOURCE.contains(pattern))
        .collect();

    assert!(
        found.is_empty(),
        "gui.rs still works the bar's fill out for itself; it contains {found:?}"
    );
}

/// The negative control for the test above.
///
/// `the_painter_computes_no_fill_fraction_of_its_own` passes by finding
/// nothing - which is also exactly what a list of patterns that can never
/// match anything does. So the same list is run here against the two lines the
/// audit found surviving, written out verbatim: `gui.rs:516`'s `if total == 0`
/// and `gui.rs:519`'s `done as f32 / total as f32`. If someone reintroduced
/// them, these are the four patterns that would catch them.
#[test]
fn the_fraction_arithmetic_patterns_catch_the_lines_they_forbid() {
    let reintroduced = concat!(
        "    let fraction = if total == 0 {\n",
        "        0.0\n",
        "    } else {\n",
        "        done as f32 / total as f32\n",
        "    };\n",
    );

    let caught: Vec<&str> = FRACTION_ARITHMETIC
        .iter()
        .copied()
        .filter(|pattern| reintroduced.contains(pattern))
        .collect();

    assert_eq!(
        caught,
        ["done as f32", "total as f32", "total == 0", "/ total"],
        "the forbidden-arithmetic list must catch the painter's own surviving mutants; \
         against the lines it exists to forbid it caught {caught:?}"
    );
}
