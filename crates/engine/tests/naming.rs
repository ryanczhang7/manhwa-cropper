//! MC-010, AC-1 to AC-6: output names never overwrite an existing file.
//!
//! `naming::plan_outputs(out_dir, &inputs)` answers one distinct,
//! currently-free path per input, in input order, accounting both for what is
//! already in `out_dir` and for duplicates inside the batch itself. On a
//! collision the name grows Explorer's suffix - `a (2).png`, `a (3).png`:
//! one space, ASCII parentheses, ASCII digits, between the stem and the
//! extension. `process_file` then writes to the path it is given and decides
//! no names of its own.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**: the ` (n)` convention itself and the fact that `n` starts at
//!   2 - `docs/wiki/architecture.md` decision 7, read out here rather than
//!   re-derived. Every criterion in this story is mechanical, per the story's
//!   `## Model guidance`; there is no threshold anywhere in this file and
//!   nothing to tune.
//! * **Mechanical**: the exact strings. `Shot.PNG` keeps its case on both
//!   sides of the suffix (`Shot (2).PNG`), a name with no extension takes the
//!   suffix at the end (`notes (2)`), and the planned paths are compared as
//!   whole `PathBuf`s, which on Windows compare case-sensitively for a file
//!   name (measured in RED).
//! * **Measured**: the controls below, and the one thing this story leaves
//!   alone - two inputs differing only in case. Both are in the story's
//!   `## Handoff: RED -> GREEN`.
//!
//! # Why the planner is a separate function and not a step inside the writer
//!
//! MC-011 runs the batch on the rayon pool, so two files that would collide
//! are named at the same instant. "Look for a free name, then write it" is a
//! race in that setting: both workers see `a (2).png` free and one of them
//! loses its file. Planning every name up front, sequentially, before the
//! first byte is written is what makes the parallel batch safe, and it is why
//! `plan_outputs` takes the whole batch rather than one input.
//!
//! # The controls, and what they are for
//!
//! Two tests here call nothing from `naming` at all.
//! [`the_naive_plan_collides_in_every_scenario_these_criteria_pin`] measures
//! what `out_dir.join(file_name)` - the implementation MC-008 shipped - would
//! answer in the AC-2, AC-3 and AC-4 scenarios, so that those criteria are
//! known to be asking for something rather than restating what already
//! happens. [`two_writes_to_one_path_leave_one_file`] is AC-6's: writing the
//! same input twice under one name leaves a single file, so "both output files
//! exist" is a real demand and not an accident of the filesystem.

mod common;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use cropper_core::Tuning;
use cropper_engine::naming::plan_outputs;
use cropper_engine::{Outcome, process_file};

// --- Harness ----------------------------------------------------------------

/// The contents of a file that is already in the output folder when the plan
/// runs. Distinctive, so that a test which found it overwritten says so with
/// the bytes rather than with a length.
const OCCUPIED: &[u8] = b"MC-010: a file the user already has. Nothing may overwrite it.";

/// A temp directory and an existing output directory inside it.
fn workspace() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let out = tmp.path().join("out");
    fs::create_dir(&out).expect("an output dir");
    (tmp, out)
}

/// Put `name` in the output folder, as a previous run or another program
/// would have left it. Returns the path.
fn occupy(out: &Path, name: &str) -> PathBuf {
    let path = out.join(name);
    fs::write(&path, OCCUPIED).expect("a file already in the output dir");
    path
}

/// An input path under its own source folder, as the criteria describe them
/// (`x/a.png`, `y/a.png`).
///
/// Nothing is created on disk: `plan_outputs` is a function of the inputs'
/// *names* and of what is in `out_dir`, and the criteria pass it bare paths.
fn from_folder(tmp: &Path, folder: &str, name: &str) -> PathBuf {
    tmp.join(folder).join(name)
}

/// The file names in `dir`, sorted - a failure message that reads like a
/// directory listing instead of a set of absolute temp paths.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("the output dir is readable")
        .map(|entry| {
            entry
                .expect("a directory entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// The file names of `paths`, in order, for the same reason.
fn file_names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| {
            path.file_name()
                .expect("a planned path has a file name")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

// --- The controls -----------------------------------------------------------

/// What the criteria would be worth against the implementation MC-008 shipped.
///
/// `process_file` joins the input's file name onto `out_dir` and writes there.
/// That answer satisfies AC-1 and nothing else, so without this control a
/// reader cannot tell whether AC-2 to AC-4 demand work or describe what
/// already happens. Measured in RED, outside the test framework (the file did
/// not compile then), and recorded in the story's handoff: the naive path
/// already exists in AC-2's and AC-3's scenarios, and AC-4's two inputs get
/// **one** path between them rather than two.
#[test]
fn the_naive_plan_collides_in_every_scenario_these_criteria_pin() {
    let (tmp, out) = workspace();
    let naive = |input: &Path| out.join(input.file_name().expect("a file name"));

    // AC-2: one file already there.
    let taken = occupy(&out, "a.png");
    let input = from_folder(tmp.path(), "x", "a.png");
    assert_eq!(
        naive(&input),
        taken,
        "AC-2's control: the naive plan is the file that is already there"
    );
    assert!(
        naive(&input).exists(),
        "AC-2's control: writing to the naive path would overwrite a file on disk"
    );

    // AC-3: that file and its ` (2)`.
    let taken_two = occupy(&out, "a (2).png");
    assert!(
        taken.exists() && taken_two.exists(),
        "AC-3's control: both a.png and a (2).png are on disk, so both are unusable"
    );

    // AC-4: two inputs, one name, nothing on disk to notice it.
    let first = from_folder(tmp.path(), "x", "a.png");
    let second = from_folder(tmp.path(), "y", "a.png");
    assert_ne!(
        first, second,
        "AC-4's control: the two inputs really are different files"
    );
    assert_eq!(
        naive(&first),
        naive(&second),
        "AC-4's control: the naive plan gives two inputs the same output path, \
         so the second write destroys the first"
    );
    let distinct: HashSet<PathBuf> = [naive(&first), naive(&second)].into_iter().collect();
    assert_eq!(
        distinct.len(),
        1,
        "AC-4's control: two inputs, one planned path"
    );
}

/// AC-6's control: two writes to one path leave one file.
///
/// AC-6 asks that both outputs exist with distinct names. That is only a
/// demand if the alternative loses a file, and it does - the filesystem
/// replaces the first file's contents in place and the directory still holds a
/// single entry. Measured in RED: one entry, holding the second write's bytes.
#[test]
fn two_writes_to_one_path_leave_one_file() {
    let (_tmp, out) = workspace();
    let target = out.join("a.png");

    fs::write(&target, OCCUPIED).expect("the first write");
    let after_first = fs::read(&target).expect("the first write is readable");
    fs::write(&target, b"a second run of the same file, re-encoded").expect("the second write");

    assert_eq!(
        entries(&out),
        vec![String::from("a.png")],
        "the control: two writes to one path leave one file, not two"
    );
    assert_ne!(
        fs::read(&target).expect("the second write is readable"),
        after_first,
        "the control: the first file's bytes are gone - this is the loss AC-6 forbids"
    );
}

// --- The shape of a plan ----------------------------------------------------

/// One path per input, in the order the inputs were given: the batch pairs
/// them up positionally, so a planner that sorted or de-duplicated its answer
/// would hand a file the wrong name.
#[test]
fn every_input_gets_one_planned_path_in_input_order() {
    let (tmp, out) = workspace();
    let inputs = vec![
        from_folder(tmp.path(), "x", "a.png"),
        from_folder(tmp.path(), "y", "b.jpg"),
        from_folder(tmp.path(), "z", "c.webp"),
    ];

    let planned = plan_outputs(&out, &inputs);

    assert_eq!(
        planned.len(),
        inputs.len(),
        "one planned path per input; got {:?}",
        file_names(&planned)
    );
    assert_eq!(
        planned,
        vec![out.join("a.png"), out.join("b.jpg"), out.join("c.webp")],
        "three free names are planned unchanged, in input order; got {:?}",
        file_names(&planned)
    );
}

/// The empty batch. `plan_outputs` is called by MC-011's runner before it
/// knows whether there is anything to do.
#[test]
fn an_empty_batch_plans_no_outputs() {
    let (_tmp, out) = workspace();
    assert_eq!(
        plan_outputs(&out, &[]),
        Vec::<PathBuf>::new(),
        "no inputs, no planned paths"
    );
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn a_name_nothing_else_claims_is_planned_unchanged() {
    let (tmp, out) = workspace();
    let input = from_folder(tmp.path(), "x", "a.png");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        planned,
        vec![out.join("a.png")],
        "AC-1: an empty output folder leaves the input's own name free"
    );
    assert!(
        !planned[0].exists(),
        "AC-1: a planned path is a free one - {} must not already exist",
        planned[0].display()
    );
}

/// The zero case of "what is already in `out_dir`": the folder itself is not
/// there yet. MC-006's decision 6 creates the output folder on demand, so the
/// planner is asked about a directory that does not exist and every name in it
/// is free.
#[test]
fn an_output_folder_that_does_not_exist_yet_leaves_every_name_free() {
    let (tmp, _out) = workspace();
    let missing = tmp.path().join("not").join("yet");
    let inputs = vec![
        from_folder(tmp.path(), "x", "a.png"),
        from_folder(tmp.path(), "y", "b.png"),
    ];

    let planned = plan_outputs(&missing, &inputs);

    assert_eq!(
        planned,
        vec![missing.join("a.png"), missing.join("b.png")],
        "AC-1: a missing output folder contains nothing, so nothing collides"
    );
}

// --- AC-2 -------------------------------------------------------------------

#[test]
fn a_name_already_on_disk_is_planned_as_two() {
    let (tmp, out) = workspace();
    let taken = occupy(&out, "a.png");
    let input = from_folder(tmp.path(), "x", "a.png");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        planned,
        vec![out.join("a (2).png")],
        "AC-2: a.png is taken, so the plan is `a (2).png` - one space, \
         parentheses, the digit 2, before the extension"
    );
    assert!(
        !planned[0].exists(),
        "AC-2: {} must be free, not merely different",
        planned[0].display()
    );
    assert_eq!(
        fs::read(&taken).expect("the file that was already there"),
        OCCUPIED,
        "AC-2: planning a name must not touch the file that holds it"
    );
}

// --- AC-3 -------------------------------------------------------------------

#[test]
fn a_name_and_its_two_both_on_disk_are_planned_as_three() {
    let (tmp, out) = workspace();
    let taken = occupy(&out, "a.png");
    let taken_two = occupy(&out, "a (2).png");
    let input = from_folder(tmp.path(), "x", "a.png");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        planned,
        vec![out.join("a (3).png")],
        "AC-3: a.png and `a (2).png` are both taken, so the counter goes on to 3"
    );
    assert_eq!(
        entries(&out),
        vec![String::from("a (2).png"), String::from("a.png")],
        "AC-3: planning writes nothing"
    );
    assert_eq!(
        (
            fs::read(&taken).expect("a.png"),
            fs::read(&taken_two).expect("a (2).png")
        ),
        (OCCUPIED.to_vec(), OCCUPIED.to_vec()),
        "AC-3: neither file that was already there is touched"
    );
}

// --- AC-4 -------------------------------------------------------------------

#[test]
fn two_inputs_with_one_name_in_the_same_batch_are_planned_apart() {
    let (tmp, out) = workspace();
    let inputs = vec![
        from_folder(tmp.path(), "x", "a.png"),
        from_folder(tmp.path(), "y", "a.png"),
    ];

    let planned = plan_outputs(&out, &inputs);

    assert_eq!(
        planned,
        vec![out.join("a.png"), out.join("a (2).png")],
        "AC-4: the duplicate is resolved from the batch alone - nothing is on \
         disk to reveal it; got {:?}",
        file_names(&planned)
    );
    assert_eq!(
        entries(&out),
        Vec::<String>::new(),
        "AC-4: the output folder is still empty - a plan is not a write, and \
         a planner that reserved names by touching files would be visible here"
    );
}

/// Many, and both halves of the rule at once: five inputs with one name
/// between them, over an output folder that already holds two of that name.
/// The counter has to carry on from what is on disk *and* keep moving within
/// the batch, and every off-by-one in it shows up as a different list.
#[test]
fn five_inputs_with_one_name_continue_past_what_is_already_on_disk() {
    let (tmp, out) = workspace();
    occupy(&out, "a.png");
    occupy(&out, "a (2).png");
    let inputs: Vec<PathBuf> = (0..5)
        .map(|i| from_folder(tmp.path(), &format!("src{i}"), "a.png"))
        .collect();

    let planned = plan_outputs(&out, &inputs);

    assert_eq!(
        file_names(&planned),
        vec![
            "a (3).png",
            "a (4).png",
            "a (5).png",
            "a (6).png",
            "a (7).png"
        ],
        "AC-3 and AC-4 together: the counter starts after the files on disk \
         and advances once per input"
    );
    let distinct: HashSet<&PathBuf> = planned.iter().collect();
    assert_eq!(
        distinct.len(),
        planned.len(),
        "AC-4: every planned path is distinct; got {:?}",
        file_names(&planned)
    );
    let occupied: Vec<&PathBuf> = planned.iter().filter(|path| path.exists()).collect();
    assert!(
        occupied.is_empty(),
        "AC-4: every planned path must be free; these are not: {occupied:?}"
    );
}

// --- AC-5 -------------------------------------------------------------------

#[test]
fn a_suffixed_name_keeps_the_case_of_its_stem_and_its_extension() {
    let (tmp, out) = workspace();
    occupy(&out, "Shot.PNG");
    let input = from_folder(tmp.path(), "x", "Shot.PNG");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        file_names(&planned),
        vec!["Shot (2).PNG"],
        "AC-5: the stem's case and the extension's case both survive - not \
         `shot (2).png`, not `Shot (2).png`"
    );
    assert_eq!(
        planned,
        vec![out.join("Shot (2).PNG")],
        "AC-5: and that is the whole path, under out_dir"
    );
}

#[test]
fn a_colliding_name_with_no_extension_takes_the_suffix_at_the_end() {
    let (tmp, out) = workspace();
    occupy(&out, "notes");
    let input = from_folder(tmp.path(), "x", "notes");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        file_names(&planned),
        vec!["notes (2)"],
        "AC-5: with no extension the suffix ends the name - not `notes (2).` \
         and not `notes.(2)`"
    );
}

// --- AC-6 -------------------------------------------------------------------

/// The planner and the writer together, on the crop path: the same screenshot
/// twice, each written to the path the plan gave it.
#[test]
fn the_same_input_processed_twice_under_planned_names_leaves_both_files() {
    let (tmp, out) = workspace();
    let input = tmp.path().join("shot.png");
    common::write_rgba8(&common::screenshot(), &input);
    let planned = plan_outputs(&out, &[input.clone(), input.clone()]);
    assert_eq!(
        planned,
        vec![out.join("shot.png"), out.join("shot (2).png")],
        "AC-6: the two runs are planned apart before either one writes"
    );

    let first = process_file(&input, &planned[0], &Tuning::default());
    let first_bytes = fs::read(&planned[0]).expect("the first run wrote its file");
    let second = process_file(&input, &planned[1], &Tuning::default());

    for (run, result, expected) in [
        ("first", first, planned[0].clone()),
        ("second", second, planned[1].clone()),
    ] {
        match result.outcome {
            Outcome::Cropped { rect, output } => {
                assert_eq!(
                    rect,
                    common::crop_rect(),
                    "AC-6: the {run} run must crop the scene to the detector's rect"
                );
                assert_eq!(
                    output, expected,
                    "AC-6: the {run} run reports the path it was given to write"
                );
            }
            other => panic!("AC-6: the {run} run must crop this screenshot, not {other:?}"),
        }
    }
    assert_eq!(
        entries(&out),
        vec![String::from("shot (2).png"), String::from("shot.png")],
        "AC-6: both outputs exist, with distinct names, and nothing else was written"
    );
    assert_eq!(
        fs::read(&planned[0]).expect("the first output is still there"),
        first_bytes,
        "AC-6: the second run leaves the first run's file exactly as it was"
    );
}

/// The same thing on the copy path. A flagged file is copied byte for byte
/// (MC-008 AC-5, decision 8), and that copy goes to the planned path too - the
/// two writers inside `process_file` must both honour it, and only one of them
/// is exercised by the test above.
#[test]
fn a_flagged_input_processed_twice_under_planned_names_leaves_both_copies() {
    let (tmp, out) = workspace();
    let input = tmp.path().join("flat.png");
    common::write_marked_uniform_png(&common::uniform(), &input);
    let source = fs::read(&input).expect("the fixture was just written");
    let planned = plan_outputs(&out, &[input.clone(), input.clone()]);
    assert_eq!(
        planned,
        vec![out.join("flat.png"), out.join("flat (2).png")],
        "AC-6: a flagged file is planned exactly like a cropped one"
    );

    let first = process_file(&input, &planned[0], &Tuning::default());
    let second = process_file(&input, &planned[1], &Tuning::default());

    for (run, result, expected) in [
        ("first", first, planned[0].clone()),
        ("second", second, planned[1].clone()),
    ] {
        match result.outcome {
            Outcome::Flagged { output, .. } => assert_eq!(
                output, expected,
                "AC-6: the {run} run copies to the path it was given"
            ),
            other => panic!("AC-6: the uniform fixture must be flagged, not {other:?}"),
        }
    }
    assert_eq!(
        entries(&out),
        vec![String::from("flat (2).png"), String::from("flat.png")],
        "AC-6: two copies, distinct names"
    );
    assert_eq!(
        (
            fs::read(&planned[0]).expect("the first copy"),
            fs::read(&planned[1]).expect("the second copy")
        ),
        (source.clone(), source),
        "AC-6: both copies are the input's own bytes - the second run neither \
         re-encoded the first nor truncated it"
    );
}
