//! MC-014: the pure view-model in `crates/app/src/lib.rs`.
//!
//! Every rule in `docs/wiki/design/components.md`, "Rules the view-model
//! owns", and every string in the frozen table of `docs/wiki/design/voice.md`.
//! No egui, no filesystem, no threads: effects leave `Model::handle` as
//! `Command` values, so a test reads them the way MC-016's runner will.
//!
//! Every expected string in this file is a literal copied out of `voice.md`,
//! never a constant re-read from the module under test - a test that asserts
//! `dropzone_text(..) == manhwa_cropper::DROPZONE_READY` asserts nothing.
//! `Cropping…` carries U+2026 and the row separator is space U+2014 space;
//! `the_frozen_literals_in_this_file_carry_voices_codepoints` is the control
//! that says so without touching the module under test.

use std::path::PathBuf;

use cropper_core::{FlagReason, Rect};
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Flag, Outcome};
use cropper_engine::settings::Settings;
use manhwa_cropper::{
    AppState, Command, Event, Model, dropzone_text, path_text, progress_text, result_line,
    row_text, rows,
};

// --- Fixtures ---------------------------------------------------------------

/// The output folder a test's user has chosen, as Windows spells it.
fn out() -> PathBuf {
    PathBuf::from("C:\\Users\\ryan\\Pictures\\cropped")
}

/// An input file in a folder that is not the output folder, so that any test
/// asserting a file *name* fails if a whole path is used instead.
fn input(name: &str) -> PathBuf {
    PathBuf::from(format!("C:\\Users\\ryan\\Downloads\\{name}"))
}

/// `n` distinct dropped files.
fn inputs(n: usize) -> Vec<PathBuf> {
    (0..n).map(|i| input(&format!("page_{i}.png"))).collect()
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
            output: out().join(name),
        },
    }
}

fn flagged(name: &str, reason: Flag) -> FileResult {
    FileResult {
        input: input(name),
        outcome: Outcome::Flagged {
            reason,
            output: out().join(name),
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

fn summary(results: Vec<FileResult>) -> RunSummary {
    RunSummary { results }
}

/// 7 cropped, 2 flagged, 1 not written - `voice.md`'s worked example - with
/// the three kinds interleaved so that `rows`' ordering is actually tested and
/// not incidentally satisfied by a filter that happens to preserve order.
fn mixed_summary() -> RunSummary {
    summary(vec![
        cropped("page_0.png"),
        flagged("page_1.png", Flag::Detector(FlagReason::Uniform)),
        cropped("page_2.png"),
        cropped("page_3.png"),
        not_written("page_4.png"),
        cropped("page_5.png"),
        flagged("notes.txt", Flag::Unsupported),
        cropped("page_7.png"),
        cropped("page_8.png"),
        cropped("page_9.png"),
    ])
}

/// 10 cropped, nothing else.
fn clean_summary() -> RunSummary {
    summary((0..10).map(|i| cropped(&format!("page_{i}.png"))).collect())
}

// --- Starting states --------------------------------------------------------
//
// Built by driving events rather than by a struct literal, so the suite pins
// the transitions it needs and leaves `Model`'s construction to GREEN.

/// A named starting state and the way to build a fresh one. A builder rather
/// than a `Model`, so that a table-driven test hands each case an untouched
/// model without the suite pinning `Model: Clone`.
type Start = (&'static str, fn() -> Model);

fn idle_no_folder() -> Model {
    Model::new(Settings { output_dir: None })
}

fn idle_with_folder() -> Model {
    Model::new(Settings {
        output_dir: Some(out()),
    })
}

fn processing() -> Model {
    let mut model = idle_with_folder();
    let _ = model.handle(Event::FilesDropped(inputs(2)));
    model
}

fn done() -> Model {
    let mut model = processing();
    let _ = model.handle(Event::Finished(mixed_summary()));
    model
}

fn error_with_folder() -> Model {
    let mut model = processing();
    let _ = model.handle(Event::Failed("disk full".to_owned()));
    model
}

fn error_no_folder() -> Model {
    let mut model = idle_no_folder();
    let _ = model.handle(Event::FilesDropped(inputs(1)));
    model
}

// --- AC-1: what a launch shows ---------------------------------------------

#[test]
fn a_first_launch_starts_idle_with_no_folder() {
    let model = Model::new(Settings { output_dir: None });

    assert_eq!(model.state, AppState::Idle);
    assert_eq!(model.output_dir, None);
}

#[test]
fn a_remembered_folder_is_shown_from_the_first_frame() {
    let model = Model::new(Settings {
        output_dir: Some(out()),
    });

    assert_eq!(model.state, AppState::Idle);
    assert_eq!(model.output_dir, Some(out()));
    assert_eq!(path_text(&model), "C:\\Users\\ryan\\Pictures\\cropped");
}

// --- AC-2: a drop before a folder is chosen ---------------------------------

#[test]
fn dropping_files_before_choosing_a_folder_asks_for_one_and_starts_nothing() {
    let mut model = idle_no_folder();

    let commands = model.handle(Event::FilesDropped(inputs(1)));

    assert_eq!(
        model.state,
        AppState::Error {
            message: "No output folder chosen. Choose one, then drop the files again.".to_owned()
        }
    );
    assert_eq!(commands, Vec::new());
    assert_eq!(model.output_dir, None);
}

#[test]
fn dropping_files_again_before_choosing_a_folder_repeats_the_same_refusal() {
    let mut model = error_no_folder();

    let commands = model.handle(Event::FilesDropped(inputs(3)));

    assert_eq!(
        model.state,
        AppState::Error {
            message: "No output folder chosen. Choose one, then drop the files again.".to_owned()
        }
    );
    assert_eq!(commands, Vec::new());
}

// --- AC-3: a drop with a folder starts a batch ------------------------------

#[test]
fn dropping_one_file_while_idle_starts_a_batch_into_the_chosen_folder() {
    let mut model = idle_with_folder();

    let commands = model.handle(Event::FilesDropped(inputs(1)));

    assert_eq!(model.state, AppState::Processing { done: 0, total: 1 });
    assert_eq!(
        commands,
        vec![Command::StartBatch {
            inputs: inputs(1),
            out_dir: out(),
        }]
    );
}

#[test]
fn dropping_many_files_after_a_finished_run_starts_a_new_batch() {
    let mut model = done();

    let commands = model.handle(Event::FilesDropped(inputs(4)));

    assert_eq!(model.state, AppState::Processing { done: 0, total: 4 });
    assert_eq!(
        commands,
        vec![Command::StartBatch {
            inputs: inputs(4),
            out_dir: out(),
        }]
    );
}

#[test]
fn dropping_files_after_a_failed_run_starts_a_batch() {
    let mut model = error_with_folder();

    let commands = model.handle(Event::FilesDropped(inputs(2)));

    assert_eq!(model.state, AppState::Processing { done: 0, total: 2 });
    assert_eq!(
        commands,
        vec![Command::StartBatch {
            inputs: inputs(2),
            out_dir: out(),
        }]
    );
}

// --- AC-4: progress and completion ------------------------------------------

#[test]
fn progress_moves_the_count_and_asks_for_nothing() {
    let mut model = processing();

    let commands = model.handle(Event::Progress { done: 3, total: 10 });

    assert_eq!(model.state, AppState::Processing { done: 3, total: 10 });
    assert_eq!(commands, Vec::new());
    assert_eq!(progress_text(3, 10), "3 of 10");
}

#[test]
fn finishing_a_run_shows_its_summary_and_asks_for_nothing() {
    let mut model = processing();

    let commands = model.handle(Event::Finished(mixed_summary()));

    assert_eq!(
        model.state,
        AppState::Done {
            summary: mixed_summary()
        }
    );
    assert_eq!(commands, Vec::new());
}

// --- AC-5: choosing a folder ------------------------------------------------

#[test]
fn choosing_a_folder_while_idle_remembers_it_and_saves_it() {
    let mut model = idle_no_folder();

    let commands = model.handle(Event::FolderChosen(out()));

    assert_eq!(model.state, AppState::Idle);
    assert_eq!(model.output_dir, Some(out()));
    assert_eq!(
        commands,
        vec![Command::SaveSettings(Settings {
            output_dir: Some(out()),
        })]
    );
}

#[test]
fn choosing_a_folder_after_a_run_keeps_the_result_on_screen() {
    let mut model = done();
    let elsewhere = PathBuf::from("D:\\crops");

    let commands = model.handle(Event::FolderChosen(elsewhere.clone()));

    assert_eq!(
        model.state,
        AppState::Done {
            summary: mixed_summary()
        }
    );
    assert_eq!(model.output_dir, Some(elsewhere.clone()));
    assert_eq!(
        commands,
        vec![Command::SaveSettings(Settings {
            output_dir: Some(elsewhere),
        })]
    );
}

#[test]
fn choosing_a_folder_after_an_error_clears_it_back_to_idle_and_saves() {
    let mut model = error_no_folder();

    let commands = model.handle(Event::FolderChosen(out()));

    assert_eq!(model.state, AppState::Idle);
    assert_eq!(model.output_dir, Some(out()));
    assert_eq!(
        commands,
        vec![Command::SaveSettings(Settings {
            output_dir: Some(out()),
        })]
    );
}

#[test]
fn choosing_a_folder_during_a_run_changes_nothing() {
    let mut model = processing();

    let commands = model.handle(Event::FolderChosen(PathBuf::from("D:\\crops")));

    assert_eq!(model.state, AppState::Processing { done: 0, total: 2 });
    assert_eq!(model.output_dir, Some(out()));
    assert_eq!(commands, Vec::new());
}

// --- AC-6: drops that are ignored -------------------------------------------

#[test]
fn dropping_files_during_a_run_is_ignored_silently() {
    let mut model = processing();

    let commands = model.handle(Event::FilesDropped(inputs(5)));

    assert_eq!(model.state, AppState::Processing { done: 0, total: 2 });
    assert_eq!(model.output_dir, Some(out()));
    assert_eq!(commands, Vec::new());
}

#[test]
fn dropping_an_empty_set_of_files_is_ignored_in_every_state() {
    let starts: [Start; 5] = [
        ("idle without a folder", idle_no_folder),
        ("idle with a folder", idle_with_folder),
        ("processing", processing),
        ("done", done),
        ("error", error_with_folder),
    ];

    let mut wrong: Vec<String> = Vec::new();
    for (name, build) in starts {
        let mut model = build();
        let before_state = model.state.clone();
        let before_dir = model.output_dir.clone();

        let commands = model.handle(Event::FilesDropped(Vec::new()));

        if model.state != before_state {
            wrong.push(format!("{name}: state became {:?}", model.state));
        }
        if model.output_dir != before_dir {
            wrong.push(format!("{name}: folder became {:?}", model.output_dir));
        }
        if !commands.is_empty() {
            wrong.push(format!("{name}: returned {commands:?}"));
        }
    }

    assert!(
        wrong.is_empty(),
        "an empty drop must change nothing anywhere: {wrong:?}"
    );
}

// --- AC-7: a run that stopped -----------------------------------------------

#[test]
fn a_failed_run_names_the_detail_and_invites_another_drop() {
    let mut model = processing();

    let _ = model.handle(Event::Failed("disk full".to_owned()));

    assert_eq!(
        model.state,
        AppState::Error {
            message: "Cropping stopped: disk full. Drop the files again.".to_owned()
        }
    );
}

#[test]
fn a_detail_that_already_ends_in_a_full_stop_does_not_get_a_second_one() {
    let mut model = processing();

    let _ = model.handle(Event::Failed("disk full.".to_owned()));

    assert_eq!(
        model.state,
        AppState::Error {
            message: "Cropping stopped: disk full. Drop the files again.".to_owned()
        }
    );
}

// --- AC-8: the result line --------------------------------------------------

#[test]
fn a_run_with_nothing_to_report_states_the_two_counts_only() {
    assert_eq!(result_line(&clean_summary()), "10 cropped, 0 flagged");
}

#[test]
fn a_run_that_lost_a_file_appends_the_failed_count() {
    assert_eq!(
        result_line(&mixed_summary()),
        "7 cropped, 2 flagged, 1 failed"
    );
}

#[test]
fn a_run_of_no_files_reports_zeroes_and_lists_nothing() {
    let empty = summary(Vec::new());

    assert_eq!(result_line(&empty), "0 cropped, 0 flagged");
    assert_eq!(rows(&empty), Vec::<String>::new());
}

// --- AC-9: the flagged list -------------------------------------------------

#[test]
fn every_flag_reason_has_its_own_word_in_a_row() {
    let cases: Vec<(FileResult, &str)> = vec![
        (
            flagged("a.png", Flag::Detector(FlagReason::Uniform)),
            "a.png — Blank",
        ),
        (
            flagged("b.png", Flag::Detector(FlagReason::NoBorderFound)),
            "b.png — No border",
        ),
        (
            flagged("c.png", Flag::Detector(FlagReason::LowContent)),
            "c.png — Low content",
        ),
        (
            flagged("d.png", Flag::Detector(FlagReason::Ambiguous)),
            "d.png — Ambiguous",
        ),
        (
            flagged("notes.txt", Flag::Unsupported),
            "notes.txt — Unsupported format",
        ),
        (
            flagged("shot.jpg", Flag::DecodeFailed("truncated".to_owned())),
            "shot.jpg — Unreadable",
        ),
        (not_written("shot.jpg"), "shot.jpg — Not written"),
    ];

    let wrong: Vec<String> = cases
        .iter()
        .filter(|(result, want)| row_text(result) != **want)
        .map(|(result, want)| format!("want {want:?}, got {:?}", row_text(result)))
        .collect();

    assert!(wrong.is_empty(), "wrong reason words: {wrong:?}");
}

#[test]
fn a_row_names_the_file_with_its_extension_not_the_folder_it_came_from() {
    let deep = FileResult {
        input: PathBuf::from("C:\\Users\\ryan\\Downloads\\manhwa\\ch 12\\page_03.png"),
        outcome: Outcome::Flagged {
            reason: Flag::Detector(FlagReason::NoBorderFound),
            output: out().join("page_03.png"),
        },
    };

    assert_eq!(row_text(&deep), "page_03.png — No border");
}

#[test]
fn the_list_holds_only_flagged_and_failed_files_in_input_order() {
    assert_eq!(
        rows(&mixed_summary()),
        vec![
            "page_1.png — Blank".to_owned(),
            "page_4.png — Not written".to_owned(),
            "notes.txt — Unsupported format".to_owned(),
        ]
    );
}

#[test]
fn the_list_is_empty_when_every_file_was_cropped() {
    assert_eq!(rows(&clean_summary()), Vec::<String>::new());
}

// --- AC-10: the strings the window paints -----------------------------------

#[test]
fn the_drop_zone_asks_for_a_folder_first_whether_files_hover_or_not() {
    let starts: [Start; 2] = [
        ("idle without a folder", idle_no_folder),
        ("error without a folder", error_no_folder),
    ];

    let mut wrong: Vec<String> = Vec::new();
    for (name, build) in starts {
        for hovering in [false, true] {
            let model = build();
            let got = dropzone_text(&model, hovering);
            if got != "Choose an output folder, then drop images here" {
                wrong.push(format!("{name}, hovering={hovering}: got {got:?}"));
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "with no folder the hover flag is ignored: {wrong:?}"
    );
}

#[test]
fn the_drop_zone_invites_a_drop_once_a_folder_is_known() {
    let starts: [Start; 3] = [
        ("idle", idle_with_folder),
        ("done", done),
        ("error", error_with_folder),
    ];

    let mut wrong: Vec<String> = Vec::new();
    for (name, build) in starts {
        let model = build();
        let got = dropzone_text(&model, false);
        if got != "Drop images here" {
            wrong.push(format!("{name}: got {got:?}"));
        }
    }

    assert!(wrong.is_empty(), "resting drop-zone text: {wrong:?}");
}

#[test]
fn the_drop_zone_confirms_the_release_while_files_hover_over_it() {
    let starts: [Start; 3] = [
        ("idle", idle_with_folder),
        ("done", done),
        ("error", error_with_folder),
    ];

    let mut wrong: Vec<String> = Vec::new();
    for (name, build) in starts {
        let model = build();
        let got = dropzone_text(&model, true);
        if got != "Release to crop" {
            wrong.push(format!("{name}: got {got:?}"));
        }
    }

    assert!(wrong.is_empty(), "drop-hover text: {wrong:?}");
}

#[test]
fn the_drop_zone_reports_that_it_is_busy_while_cropping() {
    let model = processing();

    assert_eq!(dropzone_text(&model, false), "Cropping…");
    assert_eq!(dropzone_text(&model, true), "Cropping…");
}

/// `Processing` with no output folder is the one cell of AC-10's table with no
/// text, because AC-3 says a batch cannot start without a folder. Rather than
/// invent a string for a state the user cannot reach, this pins the
/// unreachability itself: no event other than `FolderChosen` may put a
/// folderless model into `Processing` or ask for a batch.
#[test]
fn no_event_can_start_a_run_while_no_folder_is_known() {
    let starts: [Start; 2] = [
        ("idle without a folder", idle_no_folder),
        ("error without a folder", error_no_folder),
    ];
    let events = [
        Event::FilesDropped(inputs(2)),
        Event::FilesDropped(Vec::new()),
        Event::Progress { done: 1, total: 2 },
        Event::Finished(mixed_summary()),
        Event::Failed("disk full".to_owned()),
    ];

    let mut wrong: Vec<String> = Vec::new();
    for (name, build) in starts {
        for event in &events {
            let mut model = build();
            let commands = model.handle(event.clone());

            if matches!(model.state, AppState::Processing { .. }) {
                wrong.push(format!("{name} + {event:?} reached Processing"));
            }
            if commands
                .iter()
                .any(|command| matches!(command, Command::StartBatch { .. }))
            {
                wrong.push(format!("{name} + {event:?} asked for a batch"));
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "a run must never start without an output folder: {wrong:?}"
    );
}

#[test]
fn the_path_label_shows_the_whole_windows_path_unquoted() {
    let model = Model::new(Settings {
        output_dir: Some(PathBuf::from(
            "C:\\Users\\ryan\\Pictures\\manhwa\\ch 12\\cropped",
        )),
    });

    assert_eq!(
        path_text(&model),
        "C:\\Users\\ryan\\Pictures\\manhwa\\ch 12\\cropped"
    );
}

#[test]
fn the_path_label_says_so_when_no_folder_is_chosen() {
    assert_eq!(path_text(&idle_no_folder()), "No output folder chosen");
}

#[test]
fn progress_reads_as_a_count_out_of_a_total() {
    assert_eq!(progress_text(0, 10), "0 of 10");
    assert_eq!(progress_text(3, 10), "3 of 10");
    assert_eq!(progress_text(10, 10), "10 of 10");
}

// --- Codepoints: the two characters voice.md names explicitly ---------------

#[test]
fn the_busy_text_ends_in_one_ellipsis_character_and_not_three_dots() {
    let busy = dropzone_text(&processing(), false);

    assert_eq!(busy.chars().count(), 9, "{busy:?} is not 9 characters");
    assert_eq!(busy.len(), 11, "{busy:?} is not 11 bytes");
    assert_eq!(busy.chars().next_back(), Some('\u{2026}'));
    assert!(!busy.contains("..."), "{busy:?} uses three dots");
    assert!(!busy.contains('.'), "{busy:?} contains a full stop");
}

#[test]
fn a_row_is_separated_by_an_em_dash_and_not_a_hyphen() {
    let row = row_text(&flagged(
        "page_03.png",
        Flag::Detector(FlagReason::NoBorderFound),
    ));

    assert_eq!(row.chars().count(), 23, "{row:?} is not 23 characters");
    assert_eq!(row.len(), 25, "{row:?} is not 25 bytes");
    assert!(row.contains('\u{2014}'), "{row:?} has no em dash");
    assert!(!row.contains(" - "), "{row:?} uses a hyphen");
    assert!(!row.contains(" \u{2013} "), "{row:?} uses an en dash");
}

/// The control for every string assertion above. It touches nothing from the
/// module under test: it measures the literals *this file* is typed with, so
/// that a suite which goes green is known to have compared against the bytes
/// `voice.md` froze rather than against a lookalike pasted from a terminal.
/// Measured outside the test framework during RED (see `## Handoff`), because
/// the suite fails at import there and no assertion in it runs.
#[test]
fn the_frozen_literals_in_this_file_carry_voices_codepoints() {
    let busy = "Cropping…";
    assert_eq!(busy.chars().count(), 9);
    assert_eq!(busy.len(), 11);
    assert_eq!(busy.chars().next_back(), Some('\u{2026}'));
    assert!(!busy.contains('.'));

    let separator = " — ";
    assert_eq!(separator, " \u{2014} ");
    assert_eq!(separator.chars().count(), 3);
    assert_eq!(separator.len(), 5);
    assert!(!separator.contains('-'));

    let row = "page_03.png — No border";
    assert_eq!(row.chars().count(), 23);
    assert_eq!(row.len(), 25);
    assert!(row.contains('\u{2014}'));
    assert!(!row.contains(" - "));
}
