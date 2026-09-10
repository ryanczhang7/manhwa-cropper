//! MC-002, AC-1 to AC-4: the built exe, run headlessly, copies one PNG into
//! the output folder and reports through its exit code and the files on
//! disk. Nothing here reads stdout or stderr: the exe is a Windows GUI
//! subsystem binary (`docs/wiki/architecture.md`, decision 10) and those
//! streams may be unattached.
//!
//! Every test spawns `target/.../manhwa-cropper.exe` via
//! `CARGO_BIN_EXE_manhwa-cropper`, which cargo sets for integration tests of
//! a crate with a binary target, and polls it with a hard timeout: an exe
//! that ignores its arguments and opens a window would otherwise hang the
//! suite forever. Scratch directories live under the OS temp dir, never in
//! the repository tree, and are removed when the test ends, pass or fail.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const EXE: &str = env!("CARGO_BIN_EXE_manhwa-cropper");

/// Longer than any honest run of this exe by two orders of magnitude, and
/// short enough that a RED suite of window-opening runs still finishes.
const TIMEOUT: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(20);

// --- Spawning with a timeout ------------------------------------------------

#[derive(Debug)]
enum Outcome {
    Exited(ExitStatus),
    TimedOut,
}

/// Run the exe with `args`, working directory `cwd`, no attached stdio.
/// Returns the exit status, or `TimedOut` after killing a process that is
/// still alive at `TIMEOUT` (which means it opened a window instead of
/// handling its arguments).
fn run_exe(args: &[&str], cwd: &Path) -> Outcome {
    let mut child = Command::new(EXE)
        .args(args)
        .current_dir(cwd)
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

/// The exit code the exe reported, as the assertion-friendly number.
/// Panics with a bug-report-shaped message on timeout or a signal-style exit.
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
        let root = std::env::temp_dir().join(format!("mc002-{}-{}", std::process::id(), test_name));
        // A previous run that was killed hard may have left it behind.
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

    /// Write `bytes` at `rel` (creating parents) and return its path.
    fn file(&self, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = self.path(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, bytes).unwrap_or_else(|e| panic!("cannot write {}: {e}", p.display()));
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

/// Names of the entries directly inside `dir`, sorted. Empty for a directory
/// that does not exist, so "nothing was written" reads the same either way.
fn entries(dir: &Path) -> Vec<String> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = rd
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// --- A PNG that is not trivial ---------------------------------------------

/// The real 8-byte PNG signature followed by ~600 bytes of varied content:
/// an IHDR-shaped chunk header and a deterministic pseudo-random body that
/// contains zeros, 0xFF and every kind of byte in between. The exe does not
/// decode it in this story; the point is that a byte-for-byte copy of it is
/// distinguishable from a truncated, padded or re-encoded one.
fn sample_png() -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&[0, 0, 0, 13]);
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&[0, 0, 0, 7, 0, 0, 0, 5, 8, 2, 0, 0, 0]);
    let mut x: u32 = 0x2545_F491;
    for _ in 0..600 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        bytes.push((x & 0xFF) as u8);
    }
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(b"IEND");
    bytes.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]);
    bytes
}

// --- AC-1 -----------------------------------------------------------------

#[test]
fn copies_one_png_byte_for_byte_into_an_existing_out_dir_and_exits_0() {
    let scratch = Scratch::new("ac1-copy");
    let png = sample_png();
    let input = scratch.file("in/a.png", &png);
    let out = scratch.dir("out");

    let outcome = run_exe(&["--no-gui", "--out", s(&out), s(&input)], &scratch.root);

    assert_eq!(exit_code(outcome, "AC-1 copy"), 0, "exit code");
    let copy = out.join("a.png");
    assert!(copy.is_file(), "expected {} to exist", copy.display());
    let got = fs::read(&copy).unwrap();
    assert_eq!(got.len(), png.len(), "copied file length");
    assert!(got == png, "copied bytes differ from the input");
    assert_eq!(
        entries(&out),
        vec!["a.png".to_string()],
        "out dir holds only the copy"
    );
    assert!(
        fs::read(&input).unwrap() == png,
        "the input must be left untouched"
    );
}

#[test]
fn a_relative_input_path_resolves_against_the_working_directory() {
    let scratch = Scratch::new("ac1-relative");
    let png = sample_png();
    scratch.file("shot.png", &png);
    scratch.dir("out");

    let outcome = run_exe(&["--no-gui", "--out", "out", "shot.png"], &scratch.root);

    assert_eq!(exit_code(outcome, "AC-1 relative"), 0, "exit code");
    let copy = scratch.path("out/shot.png");
    assert!(
        fs::read(&copy).map(|b| b == png).unwrap_or(false),
        "expected an exact copy at {}",
        copy.display()
    );
}

// --- AC-2 -----------------------------------------------------------------

#[test]
fn creates_a_missing_out_dir_then_copies_and_exits_0() {
    let scratch = Scratch::new("ac2-create");
    let png = sample_png();
    let input = scratch.file("in/a.png", &png);
    let out = scratch.path("not-yet-there");
    assert!(!out.exists(), "precondition: out dir must not exist");

    let outcome = run_exe(&["--no-gui", "--out", s(&out), s(&input)], &scratch.root);

    assert_eq!(exit_code(outcome, "AC-2 create out dir"), 0, "exit code");
    assert!(
        out.is_dir(),
        "expected {} to have been created",
        out.display()
    );
    assert_eq!(
        entries(&out),
        vec!["a.png".to_string()],
        "out dir holds only the copy"
    );
    assert!(
        fs::read(out.join("a.png")).unwrap() == png,
        "copied bytes differ from the input"
    );
}

// --- AC-3 -----------------------------------------------------------------

#[test]
fn a_missing_input_writes_nothing_and_exits_1() {
    let scratch = Scratch::new("ac3-missing-input");
    let out = scratch.dir("out");
    let missing = scratch.path("in/does-not-exist.png");
    assert!(!missing.exists(), "precondition: input must not exist");

    let outcome = run_exe(&["--no-gui", "--out", s(&out), s(&missing)], &scratch.root);

    assert_eq!(exit_code(outcome, "AC-3 missing input"), 1, "exit code");
    assert_eq!(
        entries(&out),
        Vec::<String>::new(),
        "nothing may be written to out"
    );
}

// --- AC-4 -----------------------------------------------------------------

#[test]
fn an_unknown_flag_writes_nothing_and_exits_2() {
    let scratch = Scratch::new("ac4-unknown-flag");
    let input = scratch.file("in/a.png", &sample_png());
    let out = scratch.dir("out");

    let outcome = run_exe(
        &["--no-gui", "--out", s(&out), "--bogus", s(&input)],
        &scratch.root,
    );

    assert_eq!(exit_code(outcome, "AC-4 unknown flag"), 2, "exit code");
    assert_eq!(
        entries(&out),
        Vec::<String>::new(),
        "nothing may be written to out"
    );
}

#[test]
fn an_unknown_flag_exits_2_even_without_no_gui() {
    let scratch = Scratch::new("ac4-unknown-flag-no-headless");

    let outcome = run_exe(&["--bogus"], &scratch.root);

    assert_eq!(
        exit_code(outcome, "AC-4 unknown flag, GUI path"),
        2,
        "exit code"
    );
    assert_eq!(
        entries(&scratch.root),
        Vec::<String>::new(),
        "nothing may be written"
    );
}

#[test]
fn out_with_no_value_writes_nothing_and_exits_2() {
    let scratch = Scratch::new("ac4-out-missing-value");
    let input = scratch.file("in/a.png", &sample_png());

    let outcome = run_exe(&["--no-gui", s(&input), "--out"], &scratch.root);

    assert_eq!(
        exit_code(outcome, "AC-4 --out without a value"),
        2,
        "exit code"
    );
    assert_eq!(
        entries(&scratch.root),
        vec!["in".to_string()],
        "only the pre-existing input dir may be present"
    );
}
