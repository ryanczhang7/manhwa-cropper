//! MC-020, AC-1 to AC-6: output names never collide by letter case alone.
//!
//! MC-010 made `naming::plan_outputs` answer one *currently-free* path per
//! input. It asks two questions to decide "free", and on Windows the two
//! disagree: `Path::exists` asks a case-insensitive filesystem, while the
//! `HashSet<PathBuf>` holding the batch's own claims compares bytes. So
//! `x/a.png` and `y/A.PNG` are planned as two names and written as one file,
//! and one of the user's outputs is gone with no error anywhere.
//!
//! This file pins the fix. It is the whole of MC-020 and nothing in
//! `tests/naming.rs` - MC-010's target, whose criteria stay frozen and whose
//! assertions are untouched - changes.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! Per the story's `## Model guidance`, which partitions these criteria:
//!
//! * **Settled by measurement, and read out here rather than re-derived**:
//!   which name pairs this NTFS volume treats as one file. The story records
//!   five measurements taken on 2026-09-12;
//!   [`names_that_differ_only_by_case_are_one_file_on_this_volume`] re-takes
//!   them inside the suite, so that every threshold-free count below is
//!   anchored to the filesystem the product ships on rather than to an
//!   assumption about it. All five reproduced in RED.
//! * **Mechanical**: `plan_outputs`'s signature and the ` (n)` suffix format,
//!   both unchanged from MC-010 and both pinned rather than restated - MC-010's
//!   own criteria are not re-asserted here.
//! * **Oracle-free**: nothing. Every expected value in this file is a count of
//!   files or an exact name. There is no threshold anywhere and nothing to
//!   tune.
//!
//! # Why the characters are written as escapes
//!
//! `\u{c4}` rather than a literal `A`-with-diaeresis. This story is about
//! **case**, and composed-against-decomposed normalisation is explicitly out
//! of scope; an escape pins the exact code point, so a test that fails cannot
//! be failing because an editor normalised the source file. It also keeps this
//! file ASCII on disk.
//!
//! # The controls, and what they are for
//!
//! Two tests call nothing from `naming`. They exist because AC-1, AC-3 and
//! AC-6 assert that a name *is taken* when the byte comparison says it is
//! free, which is only a demand if the filesystem really does conflate the two
//! names - on a case-sensitive volume every criterion here would be asking for
//! a gratuitous suffix. [`names_that_differ_only_by_case_are_one_file_on_this_volume`]
//! measures exactly that, including the `\u{df}`/`SS` pair that does **not**
//! conflate, so the control fires in both directions rather than only
//! confirming what it hopes.
//! [`a_pathbuf_set_compares_bytes_while_the_disk_does_not`] measures the
//! defect's two halves side by side: the set says free, the disk says taken.

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
const OCCUPIED: &[u8] = b"MC-020: a file the user already has. Nothing may overwrite it.";

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
/// (`x/a.png`, `y/A.PNG`). Nothing is created on disk: `plan_outputs` is a
/// function of the inputs' *names* and of what is in `out_dir`.
fn from_folder(tmp: &Path, folder: &str, name: &str) -> PathBuf {
    tmp.join(folder).join(name)
}

/// The same, but with the file actually created by `write` - AC-2 is the only
/// test here that needs its inputs to be real images.
fn source_file(tmp: &Path, folder: &str, name: &str, write: impl FnOnce(&Path)) -> PathBuf {
    let path = from_folder(tmp, folder, name);
    fs::create_dir_all(path.parent().expect("a source folder")).expect("a source folder");
    write(&path);
    path
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

/// The measurement every criterion in this story stands on, re-taken here.
///
/// Four pairs that differ only by case are **one** file on this volume, and
/// the second write destroys the first; one pair that Unicode case folding
/// says is different is **two** files. Without the fifth row this control
/// could not tell "the volume conflates names" from "the volume conflates
/// everything", and without the three non-ASCII rows an ASCII-only fold would
/// look adequate.
///
/// The story's `## Model guidance` settles these by measurement and asks RED
/// to read them out rather than re-derive them. All five reproduced exactly,
/// including the surviving name in each pair.
#[test]
fn names_that_differ_only_by_case_are_one_file_on_this_volume() {
    let tmp = tempfile::tempdir().expect("a temp dir");

    // (label and directory, first name written, second name written,
    //  files the pair leaves behind, the name that survives)
    let cases: [(&str, &str, &str, usize, &str); 5] = [
        ("ascii", "Ae.png", "AE.png", 1, "Ae.png"),
        // U+00C4 / U+00E4, A-with-diaeresis and its lower case.
        ("latin1", "\u{c4}.png", "\u{e4}.png", 1, "\u{c4}.png"),
        // U+0414 / U+0434, Cyrillic De and its lower case.
        ("cyrillic", "\u{414}.png", "\u{434}.png", 1, "\u{414}.png"),
        ("dotted-i", "I.png", "i.png", 1, "I.png"),
        // U+00DF, sharp s. Its upper case is "SS", but its LOWER case is
        // itself - which is why the fold must be "lower-case and compare"
        // and not "upper-case and compare".
        ("sharp-s", "\u{df}.png", "SS.png", 2, "\u{df}.png"),
    ];

    let mut measured = Vec::new();
    let mut expected = Vec::new();
    for (label, first, second, files, survivor) in cases {
        let dir = tmp.path().join(label);
        fs::create_dir(&dir).expect("a directory for this pair");
        fs::write(dir.join(first), b"FIRST FILE").expect("the first name is writable");
        let second_already_taken = dir.join(second).exists();
        fs::write(dir.join(second), b"SECOND FILE, LONGER").expect("the second name is writable");
        let listing = entries(&dir);
        let first_survived = fs::read(dir.join(first)).expect("the first name still reads")
            == b"FIRST FILE".as_slice();

        measured.push((
            label,
            listing.len(),
            second_already_taken,
            first_survived,
            listing.contains(&String::from(survivor)),
        ));
        // One file means the second write landed on the first: the name was
        // already taken, the first file's bytes are gone, and the name that
        // survives is the one written first. Two files means none of that.
        expected.push((label, files, files == 1, files == 2, true));
    }

    assert_eq!(
        measured, expected,
        "the volume this product ships on must conflate names that differ only \
         by case, or AC-1, AC-3 and AC-6 are asking for a gratuitous suffix; \
         and it must NOT conflate \u{df} with SS, or the fold could be \
         anything. Each row is (pair, files left, the second name was already \
         taken, the first file's bytes survived, the expected name survived)"
    );
}

/// Rust's case folding must agree with the volume on the one pair that
/// separates a lower-case fold from an upper-case one.
///
/// `to_uppercase("\u{df}")` is `"SS"`, so an implementation that upper-cased
/// to compare would call `\u{df}.png` and `SS.png` the same name - two files
/// on this volume, per the control above. `to_lowercase` leaves `\u{df}`
/// alone and agrees.
#[test]
fn rust_case_folding_agrees_with_the_volume_on_sharp_s() {
    assert_eq!(
        (
            "\u{df}".to_lowercase(),
            "SS".to_lowercase(),
            "\u{df}".to_uppercase(),
        ),
        (
            String::from("\u{df}"),
            String::from("ss"),
            String::from("SS"),
        ),
        "lower-casing keeps sharp s distinct from SS, which is what the volume \
         does; upper-casing conflates them, which is what it does not"
    );
}

/// The defect itself, measured with no `naming` involved: the batch's own
/// memory and the disk answer "is this name taken?" differently.
#[test]
fn a_pathbuf_set_compares_bytes_while_the_disk_does_not() {
    let (_tmp, out) = workspace();
    let lower = out.join("a.png");
    let upper = out.join("A.PNG");

    let mut claimed: HashSet<PathBuf> = HashSet::new();
    claimed.insert(lower.clone());
    let set_says_taken = claimed.contains(&upper);

    fs::write(&lower, OCCUPIED).expect("a file in the output dir");
    let disk_says_taken = upper.exists();

    assert_eq!(
        (set_says_taken, disk_says_taken),
        (false, true),
        "MC-010's planner asks both questions and believes either answer. The \
         set says `A.PNG` is free while the disk says it is taken, so the \
         in-batch half is the broken half and the disk half (AC-5) is not"
    );
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn two_inputs_differing_only_by_case_are_planned_apart() {
    let (tmp, out) = workspace();
    let inputs = vec![
        from_folder(tmp.path(), "x", "a.png"),
        from_folder(tmp.path(), "y", "A.PNG"),
    ];

    let planned = plan_outputs(&out, &inputs);

    assert_eq!(
        file_names(&planned),
        vec!["a.png", "A (2).PNG"],
        "AC-1: the second input's name differs from the first only by case, so \
         it is suffixed - and its own case survives on both sides of the \
         suffix. Not `a (2).png`, not `A (2).png`, and above all not `A.PNG`, \
         which is the same file as `a.png` here"
    );
    assert_eq!(
        planned,
        vec![out.join("a.png"), out.join("A (2).PNG")],
        "AC-1: and those are the whole paths, under out_dir"
    );
    assert_eq!(
        entries(&out),
        Vec::<String>::new(),
        "AC-1: a plan is not a write - out_dir is still empty afterwards"
    );
}

// --- AC-2 -------------------------------------------------------------------

/// The reproduction, end to end: two inputs whose names differ only by case,
/// written through `process_file` to the paths the plan gave them.
///
/// The two inputs are deliberately **different images** - one the screenshot
/// the detector crops, one the uniform image it flags and copies byte for
/// byte. If both wrote the same bytes, "the first call's output is unchanged"
/// would hold however the second call behaved, and the third element of the
/// tuple below is what refuses to let that pass unnoticed. It also puts both
/// of `process_file`'s writers on the planned path.
///
/// *Control, measured in RED against the shipped planner*: this scenario
/// leaves **one** file, holding the second call's bytes - `(1, false, true)`.
#[test]
fn two_inputs_differing_only_by_case_leave_two_files_on_disk() {
    let (tmp, out) = workspace();
    let cropped_input = source_file(tmp.path(), "x", "a.png", |path| {
        common::write_rgba8(&common::screenshot(), path);
    });
    let copied_input = source_file(tmp.path(), "y", "A.PNG", |path| {
        common::write_marked_uniform_png(&common::uniform(), path);
    });

    let planned = plan_outputs(&out, &[cropped_input.clone(), copied_input.clone()]);

    let first = process_file(&cropped_input, &planned[0], &Tuning::default());
    let after_first = fs::read(&planned[0]).expect("the first call wrote its output");
    let second = process_file(&copied_input, &planned[1], &Tuning::default());

    for (run, result) in [("first", &first), ("second", &second)] {
        assert!(
            !matches!(result.outcome, Outcome::Failed { .. }),
            "AC-2: the {run} call must write a file, or the count below is \
             about a failure and not about naming: {:?}",
            result.outcome
        );
    }

    let observed = (
        entries(&out).len(),
        fs::read(&planned[0]).ok().as_deref() == Some(after_first.as_slice()),
        fs::read(&planned[1]).ok().as_deref() != Some(after_first.as_slice()),
    );
    assert_eq!(
        observed,
        (2, true, true),
        "AC-2: (files in out_dir, the first call's bytes survived the second, \
         the two calls wrote different bytes). Control: under MC-010's planner \
         this is (1, false, true) - one file, holding the second call's bytes, \
         and the first output destroyed. The plan was {:?}; out_dir now holds \
         {:?}",
        file_names(&planned),
        entries(&out)
    );
}

// --- AC-3 -------------------------------------------------------------------

#[test]
fn one_counter_is_shared_across_every_case_variant_of_a_name() {
    let (tmp, out) = workspace();
    let inputs = vec![
        from_folder(tmp.path(), "x", "a.png"),
        from_folder(tmp.path(), "y", "A.PNG"),
        from_folder(tmp.path(), "z", "A.png"),
    ];

    let planned = plan_outputs(&out, &inputs);

    assert_eq!(
        file_names(&planned),
        vec!["a.png", "A (2).PNG", "A (3).png"],
        "AC-3: one counter across the case variants, not one per variant. A \
         counter per variant gives `A (2).PNG` and `A (2).png`, which are one \
         file on this volume - the same bug one suffix further along"
    );

    // The plan's whole point, stated as files rather than as strings: three
    // inputs must be able to become three files here.
    for path in &planned {
        fs::write(path, OCCUPIED).expect("every planned name is writable");
    }
    assert_eq!(
        entries(&out).len(),
        3,
        "AC-3: three inputs, three files - got {:?}",
        entries(&out)
    );
}

// --- AC-4 -------------------------------------------------------------------

/// The guard against the obvious wrong fix: fold the name to compare it, then
/// write the *folded* name. That passes AC-1, AC-2, AC-3 and AC-5 and renames
/// every one of the user's files on the way past.
#[test]
fn the_planned_name_is_byte_identical_to_the_name_the_input_already_has() {
    let (tmp, out) = workspace();
    let input = from_folder(tmp.path(), "x", "Shot.PNG");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        file_names(&planned),
        vec!["Shot.PNG"],
        "AC-4: folding case is for COMPARISON only. The name that gets written \
         is the input's own, byte for byte: not `shot.png` (the whole name \
         folded), not `Shot.png` (the extension folded), not `shot.PNG`"
    );
    assert_eq!(
        planned,
        vec![out.join("Shot.PNG")],
        "AC-4: and that is the whole path, under out_dir"
    );
}

// --- AC-5 -------------------------------------------------------------------

/// The disk half, which MC-010 already gets right through `Path::exists`.
/// Pinned so that fixing the in-batch half cannot regress it.
#[test]
fn a_name_taken_on_disk_is_found_whatever_its_case() {
    let (tmp, out) = workspace();
    let taken = occupy(&out, "a.png");
    let input = from_folder(tmp.path(), "y", "A.PNG");

    let planned = plan_outputs(&out, std::slice::from_ref(&input));

    assert_eq!(
        file_names(&planned),
        vec!["A (2).PNG"],
        "AC-5: `out_dir/a.png` and `out_dir/A.PNG` are one file on this volume, \
         so the name is taken and the plan must suffix it - planning `A.PNG` \
         would overwrite the file that is already there"
    );
    assert_eq!(
        planned,
        vec![out.join("A (2).PNG")],
        "AC-5: and that is the whole path, under out_dir"
    );
    assert_eq!(
        fs::read(&taken).expect("the file that was already in out_dir"),
        OCCUPIED,
        "AC-5: planning does not touch the file it found"
    );
}

// --- AC-6 -------------------------------------------------------------------

/// An ASCII-only fold would ship a known hole: U+00C4 and U+00E4 are one file
/// on this volume, as are U+0414 and U+0434 - both measured by the control at
/// the top of this file.
#[test]
fn the_fold_reaches_past_ascii() {
    let (tmp, out) = workspace();
    let inputs = vec![
        from_folder(tmp.path(), "x", "\u{c4}.png"),
        from_folder(tmp.path(), "y", "\u{e4}.png"),
    ];

    let planned = plan_outputs(&out, &inputs);

    assert_eq!(
        file_names(&planned),
        vec!["\u{c4}.png", "\u{e4} (2).png"],
        "AC-6: U+00C4 and U+00E4 differ only by case and are one file here, so \
         the second is suffixed; and the second keeps its own lower-case \
         letter, exactly as AC-4 requires"
    );
    assert_eq!(
        planned,
        vec![out.join("\u{c4}.png"), out.join("\u{e4} (2).png")],
        "AC-6: and those are the whole paths, under out_dir"
    );
}
