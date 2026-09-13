//! MC-012, AC-1 to AC-6: the built exe, run with `--no-gui`, runs the MC-011
//! batch over its inputs, writes the JSON summary where `--summary` says, and
//! exits with 0, 1 or 2.
//!
//! Nothing here reads stdout or stderr: the exe is a Windows GUI-subsystem
//! binary (`docs/wiki/architecture.md`, decision 10) and those streams may be
//! unattached. Exit codes and files on disk are the whole contract.
//!
//! # Why this is a second file beside `cli.rs`, and not more tests in it
//!
//! `cli.rs` is MC-002's, and it is finished and frozen. The deciding reason is
//! its `run_exe`: it spawns the exe with the *test process's* environment, and
//! every spawn in this story must set `MANHWA_CROPPER_CONFIG_DIR` on the child
//! or the run reads - and, for AC-5 and AC-6, could write - the developer's
//! real `%APPDATA%\manhwa-cropper\config`. That has already happened twice in
//! this project. Making the config directory a *mandatory parameter* of the
//! spawn helper is the fix, and it cannot be applied to a frozen file. A
//! second binary also keeps this story's image fixtures out of MC-002's.
//!
//! The cost is that the spawn-with-timeout harness, `Scratch` and `entries`
//! are duplicated from `cli.rs`, near-verbatim. That is accepted: the
//! alternative is editing a frozen test file, which the rules forbid.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! Per the story's `## Model guidance`, which partitions every criterion as
//! **mechanical**:
//!
//! * **Settled elsewhere, read out rather than re-derived**: the summary's
//!   JSON shape, which is MC-011 AC-5's contract - six keys on every result,
//!   the nulls written and not omitted, `outcome` one of `cropped`, `flagged`,
//!   `failed`. The three exit codes (decision 10). The `cropped` fallback
//!   folder and the precedence around it (decision 6). None of it is
//!   re-decided here.
//! * **Mechanical**: every assertion. Exit codes, directory listings, exact
//!   JSON documents and exact file bytes.
//! * **Measured**: the crop rect the fixture scene yields, `Rect { x: 12,
//!   y: 19, w: 126, h: 96 }`, which is [`common::crop_rect`] evaluated at
//!   `Tuning::default()` and is re-derived here rather than written as
//!   literals; the 126x96 output dimensions that follow from it; and the
//!   operating system's wording for a missing file, which is read out of the
//!   summary rather than pinned. The numbers are in the story's
//!   `## Handoff: RED -> GREEN`.
//!
//! # Why the fixtures are the engine's and not new ones
//!
//! AC-1 says *supported images*, and it is satisfiable by three files that
//! are junk: 600 pseudo-random bytes behind a PNG signature decode to nothing,
//! which is `Flag::DecodeFailed`, which is **flagged**, which counts towards
//! `cropped + flagged == 3`. MC-002's `sample_png` is exactly such a file. So
//! the fixtures here are the MC-008 screenshot scene - four uniform borders, a
//! chrome band and textured art - rendered into all three containers the
//! engine handles, and the assertions below demand `"outcome": "cropped"` with
//! the rect, not a total. Reaching the generator with `#[path]` rather than
//! copying it is what makes "these files really crop" a fact this crate
//! inherits instead of a claim it has to re-establish: `formats.rs` already
//! pins that all three containers of this scene crop to the same rect.

#[path = "../../engine/tests/common/mod.rs"]
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use cropper_engine::settings::Settings;
use serde_json::{Value, json};

const EXE: &str = env!("CARGO_BIN_EXE_manhwa-cropper");

/// Longer than any honest run of this exe by two orders of magnitude, and
/// short enough that a suite of window-opening runs still finishes. Three
/// 147x120 images are microseconds of work; the cost of a case here is
/// process start, which leaves the whole budget intact even under the
/// `coverage` gate's instrumentation.
const TIMEOUT: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(20);

/// The variable `settings::config_dir` reads. Every spawn below sets it, on
/// the child, to a directory inside that test's own scratch space.
const CONFIG_DIR_ENV: &str = "MANHWA_CROPPER_CONFIG_DIR";

/// The settings file's name inside the config directory (MC-013).
const SETTINGS_FILE: &str = "settings.json";

/// The fallback output folder's name (`docs/wiki/architecture.md` decision 6).
const FALLBACK: &str = "cropped";

// --- Spawning with a timeout ------------------------------------------------

#[derive(Debug)]
enum Outcome {
    Exited(ExitStatus),
    TimedOut,
}

/// Run the exe with `args`, working directory `cwd`, `MANHWA_CROPPER_CONFIG_DIR`
/// set to `config_dir`, and no attached stdio.
///
/// `config_dir` is a parameter and not an option on purpose: a spawn that
/// forgets it inherits the developer's real `%APPDATA%\manhwa-cropper\config`,
/// which AC-5 and AC-6 would then read and could write. There is no way to
/// call this function without saying where the settings live.
fn run_exe(args: &[&str], cwd: &Path, config_dir: &Path) -> Outcome {
    let mut child = Command::new(EXE)
        .args(args)
        .current_dir(cwd)
        .env(CONFIG_DIR_ENV, config_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("could not spawn {EXE}: {e}"));

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Outcome::Exited(status),
            Ok(None) => {}
            Err(e) => panic!("waiting on {EXE} failed: {e}"),
        }
        if started.elapsed() >= TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Outcome::TimedOut;
        }
        thread::sleep(POLL);
    }
}

/// The exit code the exe reported. Panics with a bug-report-shaped message on
/// a timeout, which is what an exe that opened a window instead of running the
/// batch looks like from here.
fn exit_code(outcome: Outcome, what: &str) -> i32 {
    match outcome {
        Outcome::Exited(status) => status
            .code()
            .unwrap_or_else(|| panic!("{what}: exe ended without an exit code ({status:?})")),
        Outcome::TimedOut => panic!(
            "{what}: exe was still running after {TIMEOUT:?} - it did not act on \
             its arguments and exit (killed)"
        ),
    }
}

// --- Scratch directories ----------------------------------------------------

/// A unique directory under the OS temp dir, removed on drop (including
/// during a panic unwind, so a failing test does not leak it).
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(test_name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("mc012-{}-{}", std::process::id(), test_name));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)
            .unwrap_or_else(|e| panic!("cannot create scratch dir {}: {e}", root.display()));
        Self { root }
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    /// Create `rel` as a directory and return its path.
    fn dir(&self, rel: &str) -> PathBuf {
        let p = self.path(rel);
        fs::create_dir_all(&p).unwrap_or_else(|e| panic!("cannot create {}: {e}", p.display()));
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn s(p: &Path) -> &str {
    p.to_str().expect("scratch paths are valid UTF-8")
}

/// A path exactly as the summary writes it: `to_string_lossy`, so a Windows
/// path keeps its backslashes and the JSON comparison is over the same string
/// the engine produced (MC-011 AC-5).
fn ps(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// Names of the entries directly inside `dir`, sorted. Empty for a directory
/// that does not exist, so "nothing was written" reads the same either way.
fn entries(dir: &Path) -> Vec<String> {
    let Ok(read) = fs::read_dir(dir) else {
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

// --- Fixtures ---------------------------------------------------------------

/// The three supported images AC-1 and AC-2 are about, written into `dir` as
/// `a.png`, `b.jpg` and `c.webp`: one screenshot scene in the three containers
/// the engine handles. All three crop, and all three crop to [`rect`].
fn supported_images(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let plane = common::screenshot();
    let a = dir.join("a.png");
    common::write_grey8(&plane, &a);
    let b = dir.join("b.jpg");
    common::write_jpeg_q95(&plane, &b);
    let c = dir.join("c.webp");
    common::write_webp(&plane, &c);
    (a, b, c)
}

/// The rect the fixture scene is cropped to, derived from the scene's geometry
/// and `Tuning::default().margin_px` rather than written out, so a tuning
/// change moves the expectation instead of breaking it. Measured in RED as
/// `Rect { x: 12, y: 19, w: 126, h: 96 }`.
fn rect() -> Value {
    let r = common::crop_rect();
    json!({ "x": r.x, "y": r.y, "w": r.w, "h": r.h })
}

/// The `results` entry a cropped input earns: six keys, the nulls present
/// (MC-011 AC-5).
fn cropped_entry(input: &Path, output: &Path) -> Value {
    json!({
        "input": ps(input),
        "outcome": "cropped",
        "output": ps(output),
        "rect": rect(),
        "reason": null,
        "error": null,
    })
}

/// The summary document at `path`, parsed. Panics naming the path when it is
/// missing, which is what "`--summary` was ignored" looks like.
fn summary_at(path: &Path) -> Value {
    let bytes =
        fs::read(path).unwrap_or_else(|e| panic!("expected a summary at {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!(
            "the summary at {} must be JSON: {e}; it holds {:?}",
            path.display(),
            String::from_utf8_lossy(&bytes)
        )
    })
}

/// The dimensions of the image at `path`, decoded.
fn dimensions(path: &Path) -> (u32, u32) {
    image::GenericImageView::dimensions(&common::decode(path))
}

/// Write a settings file into `config` remembering `dir`, and return the bytes
/// it now holds - the exact document AC-5 and AC-6 insist is unchanged
/// afterwards.
fn remember(config: &Path, dir: &Path) -> Vec<u8> {
    Settings {
        output_dir: Some(dir.to_path_buf()),
    }
    .save_to(config)
    .expect("the settings fixture can be written");
    fs::read(config.join(SETTINGS_FILE)).expect("the settings fixture is readable")
}

// --- AC-1 -------------------------------------------------------------------

#[test]
fn three_supported_images_are_cropped_into_out_and_the_summary_records_all_three() {
    let scratch = Scratch::new("ac1");
    let config = scratch.dir("config");
    let src = scratch.dir("in");
    let (a, b, c) = supported_images(&src);
    let out = scratch.path("out");
    let summary = scratch.path("summary.json");

    let outcome = run_exe(
        &[
            "--no-gui",
            "--out",
            s(&out),
            "--summary",
            s(&summary),
            s(&a),
            s(&b),
            s(&c),
        ],
        &scratch.root,
        &config,
    );

    assert_eq!(exit_code(outcome, "AC-1"), 0, "AC-1: exit code");
    assert_eq!(
        entries(&out),
        ["a.png", "b.jpg", "c.webp"],
        "AC-1: all three outputs exist in --out, under the names they came in \
         with, and nothing else is there"
    );
    assert_eq!(
        (
            dimensions(&out.join("a.png")),
            dimensions(&out.join("b.jpg")),
            dimensions(&out.join("c.webp"))
        ),
        ((126, 96), (126, 96), (126, 96)),
        "AC-1: each output is the CROP of the 147x120 scene, not a copy of it. \
         A batch that flagged all three - which junk bytes behind a real PNG \
         signature would do - copies them at their original size and still \
         satisfies `cropped + flagged == 3`"
    );
    assert_eq!(
        summary_at(&summary),
        json!({
            "cropped": 3,
            "flagged": 0,
            "failed": 0,
            "results": [
                cropped_entry(&a, &out.join("a.png")),
                cropped_entry(&b, &out.join("b.jpg")),
                cropped_entry(&c, &out.join("c.webp")),
            ],
        }),
        "AC-1: `--summary` holds the MC-011 JSON document for this run - the \
         three counts, one result per input in input order, six keys on each \
         with the nulls written out, and `outcome` is \"cropped\" for every \
         one of the three. `cropped + flagged == 3` is the criterion's \
         wording; three CROPS is what makes it mean anything"
    );
}

// --- AC-2 -------------------------------------------------------------------

#[test]
fn a_missing_input_is_one_failed_result_the_others_still_crop_and_the_exit_code_is_1() {
    let scratch = Scratch::new("ac2");
    let config = scratch.dir("config");
    let src = scratch.dir("in");
    let (a, _b, c) = supported_images(&src);
    let gone = src.join("missing.png");
    assert!(!gone.exists(), "precondition: the input must not exist");
    let out = scratch.path("out");
    let summary = scratch.path("summary.json");

    let outcome = run_exe(
        &[
            "--no-gui",
            "--out",
            s(&out),
            "--summary",
            s(&summary),
            s(&a),
            s(&gone),
            s(&c),
        ],
        &scratch.root,
        &config,
    );

    assert_eq!(
        exit_code(outcome, "AC-2"),
        1,
        "AC-2: one input that produced no output at all is exit 1 - not 0 \
         because most of the run worked, and not 2, which is a command line \
         this exe could not act on"
    );
    assert_eq!(
        entries(&out),
        ["a.png", "c.webp"],
        "AC-2: one file's failure never stops the others, and the one that \
         failed leaves nothing behind - no empty missing.png"
    );

    let document = summary_at(&summary);
    // The operating system's words, not this program's: read out, so that
    // everything around it can be pinned exactly.
    let error = document["results"][1]["error"].clone();
    assert!(
        error.as_str().is_some_and(|e| !e.trim().is_empty()),
        "AC-2: the failed result carries a non-empty error string, not null \
         and not an empty one; it held {error}"
    );
    assert_eq!(
        document,
        json!({
            "cropped": 2,
            "flagged": 0,
            "failed": 1,
            "results": [
                cropped_entry(&a, &out.join("a.png")),
                {
                    "input": ps(&gone),
                    "outcome": "failed",
                    "output": null,
                    "rect": null,
                    "reason": null,
                    "error": error,
                },
                cropped_entry(&c, &out.join("c.webp")),
            ],
        }),
        "AC-2: exactly one `failed`, in the position the input was given in, \
         with a null `output` - and the other two are still full `cropped` \
         entries. A run that stopped at the missing file would report fewer \
         than three results"
    );
}

// --- AC-3 -------------------------------------------------------------------

#[test]
fn no_input_paths_writes_nothing_at_all_and_exits_2() {
    let scratch = Scratch::new("ac3");
    let config = scratch.dir("config");
    let out = scratch.path("out");
    let summary = scratch.path("summary.json");

    let outcome = run_exe(
        &["--no-gui", "--out", s(&out), "--summary", s(&summary)],
        &scratch.root,
        &config,
    );

    assert_eq!(
        exit_code(outcome, "AC-3"),
        2,
        "AC-3: `--no-gui` with nothing to do is a command line this exe cannot \
         act on, which is 2 - not 0 for a run of zero files that trivially \
         succeeded"
    );
    assert!(
        !out.exists(),
        "AC-3: nothing is written, and that includes the output folder. A run \
         handed an empty input list would create it - `batch::run` calls \
         `create_dir_all` before it looks at the inputs - so the folder's \
         absence is what proves the batch never started"
    );
    assert!(
        !summary.exists(),
        "AC-3: and no summary either. An empty run would write a perfectly \
         valid `{{\"cropped\":0,\"flagged\":0,\"failed\":0,\"results\":[]}}`"
    );
    assert_eq!(
        entries(&scratch.root),
        ["config"],
        "AC-3: nothing appeared in the working directory either"
    );
    assert_eq!(
        entries(&config),
        Vec::<String>::new(),
        "AC-3: and a run that never happened remembers nothing"
    );
}

#[test]
fn no_input_paths_and_no_out_folder_creates_no_fallback_and_exits_2() {
    let scratch = Scratch::new("ac3-bare");
    let config = scratch.dir("config");

    let outcome = run_exe(&["--no-gui"], &scratch.root, &config);

    assert_eq!(
        exit_code(outcome, "AC-3 bare"),
        2,
        "AC-3: `--no-gui` on its own is still a command line with nothing to do"
    );
    assert_eq!(
        entries(&scratch.root),
        ["config"],
        "AC-3: with no input there is no `first input` to put a `cropped` \
         folder beside, and the exe must not invent one in its working \
         directory. This is the case that catches an implementation that \
         resolves the output folder before it checks it has work to do"
    );
    assert_eq!(
        entries(&config),
        Vec::<String>::new(),
        "AC-3: and nothing was remembered"
    );
}

// --- AC-4 -------------------------------------------------------------------

#[test]
fn with_no_out_flag_the_remembered_folder_receives_the_crop() {
    let scratch = Scratch::new("ac4");
    let config = scratch.dir("config");
    let remembered = scratch.path("remembered");
    let before = remember(&config, &remembered);
    let src = scratch.dir("in");
    let (a, _b, _c) = supported_images(&src);

    let outcome = run_exe(&["--no-gui", s(&a)], &scratch.root, &config);

    assert_eq!(exit_code(outcome, "AC-4"), 0, "AC-4: exit code");
    assert_eq!(
        entries(&remembered),
        ["a.png"],
        "AC-4: with no `--out`, the folder the user last chose in the window is \
         where the crop goes - created if it is not there. This is the brief's \
         tenth run: Send-to, no flags, no interaction"
    );
    assert_eq!(
        dimensions(&remembered.join("a.png")),
        (126, 96),
        "AC-4: and it is the crop that went there, not a copy"
    );
    assert_eq!(
        entries(&src),
        ["a.png", "b.jpg", "c.webp"],
        "AC-4: the `cropped` fallback is for a user who never chose a folder. \
         With one remembered, nothing may appear beside the input"
    );
    assert_eq!(
        fs::read(config.join(SETTINGS_FILE)).expect("the settings file survives"),
        before,
        "AC-4: reading the remembered folder does not rewrite it"
    );
}

// --- AC-5 -------------------------------------------------------------------

#[test]
fn with_no_out_flag_and_no_settings_file_a_cropped_folder_is_made_beside_the_input() {
    let scratch = Scratch::new("ac5");
    let config = scratch.dir("config");
    assert_eq!(
        entries(&config),
        Vec::<String>::new(),
        "precondition: no settings file has ever been written"
    );
    let src = scratch.dir("in");
    let (a, _b, _c) = supported_images(&src);

    let outcome = run_exe(&["--no-gui", s(&a)], &scratch.root, &config);

    assert_eq!(exit_code(outcome, "AC-5"), 0, "AC-5: exit code");
    assert_eq!(
        entries(&src),
        ["a.png", "b.jpg", "c.webp", FALLBACK],
        "AC-5: a first run that was never given a folder makes `cropped` beside \
         the FIRST INPUT and uses it (decision 6). The folder did not exist \
         before this run"
    );
    assert_eq!(
        entries(&src.join(FALLBACK)),
        ["a.png"],
        "AC-5: and the one input asked for is the one thing in it"
    );
    assert_eq!(
        dimensions(&src.join(FALLBACK).join("a.png")),
        (126, 96),
        "AC-5: the crop, not a copy - the fallback folder is a destination for \
         the real run, not a consolation prize"
    );
    assert_eq!(
        entries(&config),
        Vec::<String>::new(),
        "AC-5: the fallback is NOT remembered as a choice. No settings.json is \
         written, so the next run - and the window, which reads the same key - \
         still knows the user has never picked a folder. A run that saved it \
         would silently turn one Send-to into a permanent setting"
    );
    assert_eq!(
        entries(&scratch.root),
        ["config", "in"],
        "AC-5: and nothing was created in the working directory - `beside the \
         first input` is the input's folder, not the cwd"
    );
}

// --- AC-6 -------------------------------------------------------------------

#[test]
fn an_out_folder_on_the_command_line_does_not_become_the_remembered_folder() {
    let scratch = Scratch::new("ac6");
    let config = scratch.dir("config");
    let remembered = scratch.path("remembered");
    let before = remember(&config, &remembered);
    let src = scratch.dir("in");
    let (a, _b, _c) = supported_images(&src);
    let out = scratch.path("out");

    let outcome = run_exe(
        &["--no-gui", "--out", s(&out), s(&a)],
        &scratch.root,
        &config,
    );

    assert_eq!(exit_code(outcome, "AC-6"), 0, "AC-6: exit code");
    assert_eq!(
        entries(&out),
        ["a.png"],
        "AC-6: the command line's folder is where this run wrote"
    );
    assert_eq!(
        dimensions(&out.join("a.png")),
        (126, 96),
        "AC-6: `when the run completes` - the batch really ran and really \
         cropped, so what follows about the settings is a statement about a \
         completed run and not about an exe that did nothing"
    );
    assert_eq!(
        fs::read(config.join(SETTINGS_FILE)).expect("the settings file survives"),
        before,
        "AC-6: and settings.json still holds its previous value, byte for byte. \
         A command-line folder is a one-off - a script or a Send-to invocation \
         must not silently change what the window will offer next time"
    );
    assert_eq!(
        entries(&remembered),
        Vec::<String>::new(),
        "AC-6: nothing went to the remembered folder, and `--out` did not cause \
         it to be created"
    );
    assert_eq!(
        entries(&config),
        [SETTINGS_FILE],
        "AC-6: and the config directory gained nothing"
    );
}
