//! MC-012, AC-4 to AC-6 at the unit level: where a run's output goes, decided
//! once, in a pure function.
//!
//! `resolve_out_dir(&Invocation, &Settings) -> PathBuf` is the whole of the
//! rule `docs/wiki/architecture.md` decision 6 states: an explicit `--out` is
//! a one-off and wins; otherwise the folder the user last chose is used; and
//! if they never chose one, a `cropped` folder **beside the first input**.
//!
//! The story's `## Model guidance` asks for this function so AC-4 to AC-6 have
//! a unit-level test as well as a spawned-exe one. There is a second reason,
//! and it is the reason the assertions below matter more than the end-to-end
//! ones: the `coverage` gate runs with
//! `--ignore-filename-regex 'crates[/\\]app[/\\]src[/\\](main|gui)\.rs'`, so
//! anything left in `main.rs` is not measured at all. The rule lives here, in
//! the engine, where it is.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled elsewhere, read out rather than re-derived**: the precedence
//!   itself (decision 6) and the fallback folder's name, `cropped`, which the
//!   story's `## Context` states. Nothing here re-decides either.
//! * **Mechanical**: every assertion. Each expected value is an exact
//!   `PathBuf` built from the same pieces the input was built from; there is
//!   no threshold and nothing to tune.
//! * **Measured**: nothing. This file needs no fixture and no I/O - which is
//!   the point of [`resolving_touches_the_file_system_not_at_all`].
//!
//! # Why purity is asserted and not assumed
//!
//! AC-5 says the fallback folder is *created*. It is not created here:
//! `batch::run` already calls `fs::create_dir_all(out_dir)` (MC-011), and a
//! resolver that also created directories would make "where would the output
//! go?" a question nobody could ask without side effects - the window (MC-015,
//! MC-016) wants to display the answer before a run starts.

use std::path::{Path, PathBuf};

use cropper_engine::args::Invocation;
use cropper_engine::resolve_out_dir;
use cropper_engine::settings::Settings;

// --- Harness ----------------------------------------------------------------

/// The fallback folder's name, from the story's `## Context` and
/// `docs/wiki/architecture.md` decision 6.
const FALLBACK: &str = "cropped";

/// A headless invocation over `inputs`, with `--out` given or not.
fn invocation(inputs: &[&str], out_dir: Option<&str>) -> Invocation {
    Invocation {
        inputs: inputs.iter().map(PathBuf::from).collect(),
        out_dir: out_dir.map(PathBuf::from),
        no_gui: true,
        summary_path: None,
    }
}

/// Settings that remember `dir`, or that remember nothing.
fn settings(dir: Option<&str>) -> Settings {
    Settings {
        output_dir: dir.map(PathBuf::from),
    }
}

// --- AC-6: an explicit --out is a one-off and wins --------------------------

#[test]
fn an_explicit_out_folder_wins_over_the_remembered_one() {
    let inv = invocation(&[r"D:\shots\a.png"], Some(r"D:\one-off"));
    let remembered = settings(Some(r"D:\remembered"));

    assert_eq!(
        resolve_out_dir(&inv, &remembered),
        PathBuf::from(r"D:\one-off"),
        "AC-6: `--out` is what the user asked for on this run; the remembered \
         folder is what they chose in the window on some earlier run. A \
         resolver that prefers the settings sends this run's files somewhere \
         the command line did not name"
    );
}

#[test]
fn an_explicit_out_folder_is_used_when_nothing_is_remembered() {
    let inv = invocation(&[r"D:\shots\a.png"], Some(r"D:\one-off"));

    assert_eq!(
        resolve_out_dir(&inv, &settings(None)),
        PathBuf::from(r"D:\one-off"),
        "AC-6: with no remembered folder the explicit one is still the answer, \
         and the `cropped` fallback must not appear - the fallback is for a \
         run that named no folder at all"
    );
}

#[test]
fn an_explicit_relative_out_folder_is_returned_exactly_as_it_was_typed() {
    let inv = invocation(&["shot.png"], Some("out"));

    assert_eq!(
        resolve_out_dir(&inv, &settings(Some(r"D:\remembered"))),
        PathBuf::from("out"),
        "AC-6: a relative `--out` is left relative. The exe resolves it against \
         its working directory when it writes (MC-002 pins that end to end); \
         canonicalising it here would answer with a folder that does not exist \
         yet, which `canonicalize` cannot do anyway"
    );
}

// --- AC-4: otherwise, the remembered folder ---------------------------------

#[test]
fn with_no_out_flag_the_remembered_folder_is_used() {
    let inv = invocation(&[r"D:\shots\a.png"], None);

    assert_eq!(
        resolve_out_dir(&inv, &settings(Some(r"D:\remembered"))),
        PathBuf::from(r"D:\remembered"),
        "AC-4: this is the tenth-run Send-to flow from the brief - no flags, no \
         interaction, and the files land where the user last said they should"
    );
}

#[test]
fn the_remembered_folder_wins_over_the_fallback_beside_the_input() {
    let inv = invocation(&[r"D:\shots\a.png"], None);

    let resolved = resolve_out_dir(&inv, &settings(Some(r"D:\remembered")));

    assert_ne!(
        resolved,
        PathBuf::from(r"D:\shots").join(FALLBACK),
        "AC-4 and AC-5: the `cropped` fallback applies only when no folder was \
         ever chosen. A resolver that always falls back scatters `cropped` \
         folders through the user's screenshot directory while the folder they \
         picked in the window stays empty"
    );
    assert_eq!(resolved, PathBuf::from(r"D:\remembered"), "AC-4");
}

// --- AC-5: and if nothing was ever chosen, `cropped` beside the first input -

#[test]
fn with_nothing_remembered_the_fallback_is_a_cropped_folder_beside_the_input() {
    let inv = invocation(&[r"D:\shots\a.png"], None);

    assert_eq!(
        resolve_out_dir(&inv, &settings(None)),
        PathBuf::from(r"D:\shots").join(FALLBACK),
        "AC-5: decision 6 - Send-to with no folder ever chosen writes into a \
         `cropped` folder next to the first input, so the very first run needs \
         no interaction either"
    );
}

#[test]
fn the_fallback_sits_beside_the_first_input_when_the_inputs_are_in_different_folders() {
    let inv = invocation(
        &[r"D:\shots\a.png", r"E:\elsewhere\b.png", r"D:\other\c.png"],
        None,
    );

    assert_eq!(
        resolve_out_dir(&inv, &settings(None)),
        PathBuf::from(r"D:\shots").join(FALLBACK),
        "AC-5: one run writes into one folder, and it is the first input's - \
         `first`, not `last`. Explorer hands Send-to the selection in the order \
         it displays them, so the first is the one the user clicked"
    );
}

#[test]
fn an_input_with_no_folder_falls_back_to_a_relative_cropped_folder() {
    let inv = invocation(&["shot.png"], None);

    assert_eq!(
        resolve_out_dir(&inv, &settings(None)),
        PathBuf::from(FALLBACK),
        "AC-5: `shot.png` names no folder, so `beside the first input` is the \
         working directory - a bare relative `cropped`, which the writer then \
         resolves against the cwd exactly as it resolves the input. Not an \
         absolute path, and not `./cropped` with an empty component in front \
         of it"
    );
}

#[test]
fn with_no_inputs_at_all_the_fallback_is_still_a_relative_cropped_folder() {
    let inv = invocation(&[], None);

    assert_eq!(
        resolve_out_dir(&inv, &settings(None)),
        PathBuf::from(FALLBACK),
        "Totality, not an acceptance criterion: AC-3 makes an empty input list \
         exit 2 before a folder is ever needed, so nothing reaches this. It is \
         pinned because the function must still be total, and this is the \
         answer the rule degenerates to - the same one an input with no folder \
         gives. It must not panic and must not be an empty path"
    );
}

// --- The function is pure ---------------------------------------------------

#[test]
fn resolving_touches_the_file_system_not_at_all() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let base = tmp.path();
    let input = base.join("in").join("a.png");
    let named = base.join("named-out");

    let fallback = resolve_out_dir(
        &Invocation {
            inputs: vec![input.clone()],
            out_dir: None,
            no_gui: true,
            summary_path: None,
        },
        &Settings::default(),
    );
    let explicit = resolve_out_dir(
        &Invocation {
            inputs: vec![input.clone()],
            out_dir: Some(named.clone()),
            no_gui: true,
            summary_path: None,
        },
        &Settings::default(),
    );

    assert_eq!(fallback, base.join("in").join(FALLBACK), "AC-5");
    assert_eq!(explicit, named, "AC-6");
    assert_eq!(
        entries(base),
        Vec::<String>::new(),
        "resolving is a question, not a visit: it creates no folder, not the \
         fallback and not the named one. AC-5's `the folder having been \
         created` is `batch::run`'s `create_dir_all` (MC-011), which happens \
         when the run starts - and MC-015 wants to show the user where the \
         files will go before they commit to a run"
    );
    assert!(
        !input.exists(),
        "and it does not go looking at the input either - the answer is a \
         question about the path, not about what is on the volume"
    );
}

/// Names of the entries directly inside `dir`, sorted. Empty for a directory
/// that does not exist, so "nothing was created" reads the same either way.
fn entries(dir: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .map(|entry| {
            entry
                .expect("a readable directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}
