//! MC-013, AC-1 to AC-5: the output folder is remembered between runs.
//!
//! `settings::Settings { output_dir: Option<PathBuf> }` lives in
//! `<config_dir>/settings.json`, where `config_dir` is
//! `MANHWA_CROPPER_CONFIG_DIR` when the variable is set and the
//! `directories` layout otherwise (`docs/wiki/architecture.md` decision 6 and
//! "Where state lives"). A missing or corrupt file is the defaults: the app
//! never fails to start over its settings.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! Per the story's `## Model guidance`, which partitions these criteria:
//!
//! * **Mechanical**: all five. Every expected value below is an exact path, an
//!   exact directory listing, an exact JSON document or an exact `Settings`
//!   value. There is no threshold anywhere and nothing to tune.
//! * **Settled elsewhere, read out rather than re-derived**: the key name
//!   `output_dir` and the file name `settings.json`. MC-016 AC-1 asserts that
//!   "`settings.json` under `MANHWA_CROPPER_CONFIG_DIR` holds
//!   `output_dir == p`" after the folder picker runs, so both are already a
//!   contract with a story that is not written yet.
//! * **Measured**: the `directories` layout on this machine, which AC-5 pins
//!   the tail of. `ProjectDirs::from("", "", "manhwa-cropper").config_dir()`
//!   was run against `directories` 6.0.0 during RED and answered
//!   `C:\Users\ryanc\AppData\Roaming\manhwa-cropper\config`. Also measured:
//!   that each of the five [`CORRUPT_FILES`] really is unreadable, and the
//!   exact JSON `serde` writes for `Some` and for `None`. The numbers are in
//!   the story's `## Handoff: RED -> GREEN`.
//!
//! # The environment variable, and why most of this file never touches it
//!
//! Rust runs a test binary's tests on threads of one process, and one process
//! has one environment. In edition 2024 `std::env::set_var` is `unsafe` for
//! exactly that reason. So the split the story's `## Model guidance` asks for:
//!
//! * The **substance** of AC-1 to AC-4 is pinned through `load_from(dir)` and
//!   `save_to(dir)`, which are handed the directory and read no environment at
//!   all. Those tests are the ones that say what persistence *does*, and they
//!   are free to run concurrently with everything else.
//! * The **wiring** - that `load`/`save` really do go through `config_dir()`,
//!   and that `config_dir()` really does read the variable - needs the process
//!   environment, so the three tests that need it take [`ENV_LOCK`] first.
//!   The set-it cases and AC-5's unset case take the *same* lock, so no test
//!   here can observe another test's value. [`ScopedConfigDirEnv`] restores
//!   the previous value from `Drop`, which runs while a failing test unwinds,
//!   and the lock is taken with `unwrap_or_else(PoisonError::into_inner)` so
//!   that one panicking test does not turn into four misleading ones.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Deserialize;
use serde_json::{Value, json};

use cropper_engine::settings::Settings;

// --- Harness ----------------------------------------------------------------

/// The one file this story writes, under the config directory.
const SETTINGS_FILE: &str = "settings.json";

/// The variable that replaces the `directories` layout, so that no test - and
/// no MC-012 or MC-016 test after it - ever writes to the real config folder.
const CONFIG_DIR_ENV: &str = "MANHWA_CROPPER_CONFIG_DIR";

/// The bytes of the AC-4 blocker: a real file, with the config directory asked
/// for *underneath* it, so the directory cannot be created.
const BLOCKER: &[u8] =
    b"MC-013 AC-4: a file, not a folder. Nothing may turn this into a directory.";

/// Five files that a `Settings` cannot be read out of. The first is the exact
/// text AC-3 names; the rest are the ways a settings file really goes wrong -
/// a zero-length file from a crash, a value of the wrong shape, a hand edit
/// that put the wrong type in, and a write that was cut off part way.
///
/// *Control, measured outside the test framework in RED* (see the handoff):
/// all five are `Err` when deserialised into a struct with one
/// `Option<PathBuf>` field, and `{}`, `{"output_dir": null}` and
/// `{"output_dir": "<path>"}` are `Ok`. Without the first half AC-3 would be
/// asserting that valid files load as the defaults, which is the opposite of
/// what it says.
const CORRUPT_FILES: [(&str, &str); 5] = [
    ("the criterion's own text", "{not json"),
    ("an empty file", ""),
    ("a bare null", "null"),
    ("the right key, the wrong type", r#"{"output_dir": 42}"#),
    (
        "a write that was cut off",
        r#"{"output_dir": "D:/pictures/out""#,
    ),
];

/// The lock every test that reads or writes [`CONFIG_DIR_ENV`] holds. One
/// process, one environment: AC-5 asks what the config directory is when the
/// variable is *unset*, which is only a question if no other test can set it
/// halfway through.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// A struct that mirrors the shape `Settings` is declared with, used by
/// [`the_five_corrupt_files_are_all_unreadable_as_settings`] to check the
/// control without going through the module under test.
#[derive(Deserialize, Debug)]
struct MirrorSettings {
    output_dir: Option<PathBuf>,
}

/// Holds [`ENV_LOCK`] and puts [`CONFIG_DIR_ENV`] back the way it was.
///
/// `Drop` runs before the fields are dropped, so the variable is restored
/// while the lock is still held - and it runs during a panicking test's
/// unwind, so a failed assertion cannot leak a config directory into the next
/// test that takes the lock.
struct ScopedConfigDirEnv {
    previous: Option<OsString>,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for ScopedConfigDirEnv {
    fn drop(&mut self) {
        // SAFETY: `_lock` is still held, so no other test in this binary is
        // reading or writing the environment right now.
        unsafe { write_config_dir_env(self.previous.as_deref()) }
    }
}

/// Set [`CONFIG_DIR_ENV`] to `value`, or remove it when `value` is `None`.
///
/// # Safety
///
/// The caller must hold [`ENV_LOCK`]. `std::env::set_var` and
/// `std::env::remove_var` are unsound if another thread is reading the
/// environment at the same time, which is the whole reason this file has a
/// lock.
unsafe fn write_config_dir_env(value: Option<&OsStr>) {
    match value {
        Some(dir) => unsafe { env::set_var(CONFIG_DIR_ENV, dir) },
        None => unsafe { env::remove_var(CONFIG_DIR_ENV) },
    }
}

/// Point [`CONFIG_DIR_ENV`] at `value` - or unset it, for `None` - until the
/// returned guard is dropped. Bind it to a name; dropping it immediately puts
/// the variable straight back.
#[must_use]
fn with_config_dir(value: Option<&Path>) -> ScopedConfigDirEnv {
    let lock = ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    let previous = env::var_os(CONFIG_DIR_ENV);
    // SAFETY: the lock is held for as long as the returned guard lives, and
    // every test in this binary that touches the environment takes it first.
    unsafe { write_config_dir_env(value.map(Path::as_os_str)) }
    ScopedConfigDirEnv {
        previous,
        _lock: lock,
    }
}

/// A temp directory, an **empty** config directory inside it, and the folder a
/// user has picked as their output.
///
/// The chosen folder is deliberately *not* under the config directory, so that
/// "the config directory holds exactly `settings.json`" says something, and it
/// deliberately does not exist on disk: settings record a path, they do not
/// validate one.
fn workspace() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let config = tmp.path().join("config");
    fs::create_dir(&config).expect("an empty config dir");
    let chosen = tmp.path().join("pictures").join("cropped");
    (tmp, config, chosen)
}

/// The file names in `dir`, sorted - a failure message that reads like a
/// directory listing rather than a set of absolute temp paths.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
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

/// `dir/settings.json` parsed as JSON, so the assertions pin the shape and the
/// key names rather than the whitespace - pretty-printed or not is GREEN's
/// choice.
fn json_at(dir: &Path) -> Value {
    let path = dir.join(SETTINGS_FILE);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|err| {
        panic!(
            "{} is not JSON at all ({err}); it holds {:?}",
            path.display(),
            String::from_utf8_lossy(&bytes)
        )
    })
}

/// A path as JSON carries it: `PathBuf`'s own `Serialize` writes the string.
fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// --- The control ------------------------------------------------------------

/// AC-3 says a corrupt file loads as the defaults. That is only a demand if
/// the files really are corrupt - if one of them happened to parse, AC-3 would
/// be asserting that a *valid* settings file is thrown away, and the test
/// would pass while the product lost the user's folder.
///
/// So: all five fail, and a well-formed document does not. One direction alone
/// would prove only that `serde_json` rejects things.
#[test]
fn the_five_corrupt_files_are_all_unreadable_as_settings() {
    let measured: Vec<(&str, bool)> = CORRUPT_FILES
        .into_iter()
        .map(|(label, text)| {
            let parsed = serde_json::from_str::<MirrorSettings>(text).map(|s| s.output_dir);
            (label, parsed.is_err())
        })
        .collect();
    let expected: Vec<(&str, bool)> = CORRUPT_FILES.into_iter().map(|(l, _)| (l, true)).collect();

    assert_eq!(
        measured, expected,
        "every row of CORRUPT_FILES must be unreadable as a struct with one \
         Option<PathBuf> field, or the AC-3 test below is asserting that a \
         readable file is discarded. Each row is (label, deserialising it \
         failed)"
    );

    assert_eq!(
        serde_json::from_str::<MirrorSettings>(r#"{"output_dir": "D:/pictures/out"}"#)
            .map(|s| s.output_dir)
            .ok(),
        Some(Some(PathBuf::from("D:/pictures/out"))),
        "and the other direction: a well-formed document IS read, so the five \
         rows above fail because of what they are and not because nothing \
         parses"
    );
}

// --- AC-1: what was saved is what is loaded ---------------------------------

#[test]
fn a_saved_output_folder_is_read_back_unchanged_from_settings_json() {
    let (_tmp, config, chosen) = workspace();
    let saved = Settings {
        output_dir: Some(chosen.clone()),
    };

    saved
        .save_to(&config)
        .expect("AC-1: saving into an existing, writable config dir succeeds");

    assert_eq!(
        entries(&config),
        [SETTINGS_FILE],
        "AC-1: the file is <dir>/settings.json - that exact name, directly in \
         the config directory, and it is the only thing save wrote"
    );
    assert_eq!(
        Settings::load_from(&config),
        saved,
        "AC-1: what load reads back is what save was given, path for path"
    );
    assert_eq!(
        json_at(&config),
        json!({ "output_dir": path_string(&chosen) }),
        "AC-1: and the document is one object with one key, `output_dir`, \
         holding the path as a string. MC-016 AC-1 reads this key by name, so \
         it is a contract and not an implementation detail"
    );
}

#[test]
fn saving_the_defaults_writes_a_null_output_folder_and_reads_back_as_the_defaults() {
    let (_tmp, config, _chosen) = workspace();
    let saved = Settings::default();

    saved
        .save_to(&config)
        .expect("AC-1: saving the defaults succeeds too");

    assert_eq!(
        json_at(&config),
        json!({ "output_dir": null }),
        "AC-1: `None` is written as a present, null key - an omitted key is a \
         different document, and MC-016 reads the key by name. No \
         `skip_serializing_if`"
    );
    assert_eq!(
        Settings::load_from(&config),
        Settings::default(),
        "AC-1: and it reads back as the defaults, so the empty case round-trips \
         like any other"
    );
}

#[test]
fn save_and_load_go_through_the_config_dir_the_environment_variable_names() {
    let (_tmp, config, chosen) = workspace();
    let _env = with_config_dir(Some(&config));
    let saved = Settings {
        output_dir: Some(chosen),
    };

    saved
        .save()
        .expect("AC-1: save() writes under the config dir the variable names");

    assert_eq!(
        Settings::config_dir(),
        config,
        "AC-1: with MANHWA_CROPPER_CONFIG_DIR set, that directory IS the config \
         directory"
    );
    assert_eq!(
        entries(&config),
        [SETTINGS_FILE],
        "AC-1: so save() - the no-argument one the app calls - put \
         settings.json there and nowhere else. A save() that ignored \
         config_dir() would leave this directory empty and write into the real \
         %APPDATA% folder"
    );
    assert_eq!(
        Settings::load(),
        saved,
        "AC-1: and load() reads the same file back. These two are the wrappers \
         MC-014 and MC-016 call; load_from/save_to above pin what they wrap"
    );
}

// --- AC-2: no settings file is the defaults ---------------------------------

#[test]
fn a_config_directory_with_no_settings_file_loads_the_defaults() {
    let (_tmp, config, _chosen) = workspace();

    let loaded = Settings::load_from(&config);

    assert_eq!(
        loaded,
        Settings::default(),
        "AC-2: an empty config directory is a first run, and a first run is the \
         defaults - not an error, not a panic"
    );
    assert_eq!(
        loaded.output_dir, None,
        "AC-2: and the defaults are no output folder at all, which is what \
         MC-012's headless fallback tests for"
    );
    assert_eq!(
        entries(&config),
        Vec::<String>::new(),
        "AC-2: loading creates nothing - not an empty settings.json written \
         out helpfully, not a lock file. Only save writes"
    );
}

#[test]
fn a_config_directory_that_does_not_exist_loads_the_defaults_and_is_not_created() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let config = tmp.path().join("never").join("created");
    assert!(
        !config.exists(),
        "the fixture is wrong: this test needs a config dir that does not exist"
    );

    let loaded = Settings::load_from(&config);

    assert_eq!(
        loaded,
        Settings::default(),
        "AC-2: the very first run has no config directory either, and that is \
         still the defaults"
    );
    assert!(
        !config.exists(),
        "AC-2: and loading did not create it; creating it is save's job (AC-4)"
    );
}

// --- AC-3: a corrupt settings file is the defaults --------------------------

#[test]
fn a_corrupt_settings_file_loads_the_defaults_and_is_left_exactly_as_it_was() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut measured = Vec::new();
    let mut expected = Vec::new();

    for (index, (label, text)) in CORRUPT_FILES.into_iter().enumerate() {
        let config = tmp.path().join(format!("case-{index}"));
        fs::create_dir(&config).expect("a config dir for this case");
        let file = config.join(SETTINGS_FILE);
        fs::write(&file, text).expect("the corrupt settings file");

        let loaded = Settings::load_from(&config);

        measured.push((
            label,
            loaded,
            fs::read_to_string(&file).ok(),
            entries(&config),
        ));
        expected.push((
            label,
            Settings::default(),
            Some(String::from(text)),
            vec![String::from(SETTINGS_FILE)],
        ));
    }

    assert_eq!(
        measured, expected,
        "AC-3: an unreadable settings.json is the defaults and nothing else \
         happens - load does not return an error, does not panic, does not \
         rewrite the file with something valid, does not delete it and does not \
         leave a .bak beside it. Each row is (label, what loaded, the file's \
         text afterwards, the config dir's listing)"
    );
}

#[test]
fn the_next_save_replaces_a_corrupt_settings_file() {
    let (_tmp, config, chosen) = workspace();
    fs::write(config.join(SETTINGS_FILE), "{not json").expect("the corrupt settings file");
    let saved = Settings {
        output_dir: Some(chosen.clone()),
    };

    saved
        .save_to(&config)
        .expect("AC-3: saving over a corrupt file succeeds");

    assert_eq!(
        json_at(&config),
        json!({ "output_dir": path_string(&chosen) }),
        "AC-3: the corrupt text survives only until the next save, which \
         overwrites it whole - no appending to it, no leaving a fragment of the \
         old text past the end of the new"
    );
    assert_eq!(
        Settings::load_from(&config),
        saved,
        "AC-3: and from then on the folder is remembered again"
    );
    assert_eq!(
        entries(&config),
        [SETTINGS_FILE],
        "AC-3: with no temporary file left behind next to it"
    );
}

// --- AC-4: save creates the config directory --------------------------------

#[test]
fn save_creates_a_config_directory_that_does_not_exist_yet() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    // TWO levels missing, because that is what a real first run faces:
    // %APPDATA%\manhwa-cropper\config has neither `manhwa-cropper` nor
    // `config` until something makes them.
    let config = tmp.path().join("manhwa-cropper").join("config");
    let chosen = tmp.path().join("pictures").join("cropped");
    assert!(
        !config.exists(),
        "the fixture is wrong: AC-4 needs a config dir that does not exist"
    );
    let saved = Settings {
        output_dir: Some(chosen),
    };

    saved.save_to(&config).expect(
        "AC-4: save creates the config directory it needs, every missing level \
         of it - fs::create_dir_all, not fs::create_dir, which fails here",
    );

    assert!(
        config.is_dir(),
        "AC-4: the config directory now exists; {} is still missing",
        config.display()
    );
    assert_eq!(
        entries(&config),
        [SETTINGS_FILE],
        "AC-4: and the file was written into it"
    );
    assert_eq!(
        Settings::load_from(&config),
        saved,
        "AC-4: and reads back, so the directory was created before the write \
         rather than instead of it"
    );
}

#[test]
fn save_reports_an_error_when_the_config_directory_cannot_be_created() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let blocker = tmp.path().join("blocker");
    fs::write(&blocker, BLOCKER).expect("the blocker file");
    let config = blocker.join("config");
    let saved = Settings {
        output_dir: Some(tmp.path().join("cropped")),
    };

    let result = saved.save_to(&config);

    assert!(
        result.is_err(),
        "AC-4: nothing can be created under a path that is a file, and save \
         says so with an Err rather than swallowing it and reporting success. \
         Its io::Result is a real answer, not decoration; got {result:?}"
    );
    assert_eq!(
        fs::read(&blocker).expect("the blocker file is still readable"),
        BLOCKER,
        "AC-4: and the file that was in the way was not clobbered on the way \
         through"
    );
}

// --- AC-5: where the config directory is ------------------------------------

#[test]
fn with_the_variable_unset_the_config_dir_is_the_directories_layout() {
    let _env = with_config_dir(None);

    let dir = Settings::config_dir();

    let mut tail: Vec<String> = dir
        .components()
        .rev()
        .take(2)
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    tail.reverse();

    assert_eq!(
        tail,
        ["manhwa-cropper", "config"],
        "AC-5: with MANHWA_CROPPER_CONFIG_DIR unset the answer is the \
         `directories` layout, whose last two components are the application \
         name and then `config`. Measured against directories 6.0.0 during \
         MC-013 RED: ProjectDirs::from(\"\", \"\", \"manhwa-cropper\") gives \
         C:\\Users\\<user>\\AppData\\Roaming\\manhwa-cropper\\config. Not \
         %LOCALAPPDATA%, not a bare application folder with no `config` inside \
         it, and not the temp directory some other test was using. The whole \
         path was {}",
        dir.display()
    );
}

#[test]
fn the_variable_replaces_the_config_dir_and_asking_where_it_is_does_not_create_it() {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let named = tmp.path().join("somewhere").join("else");
    let _env = with_config_dir(Some(&named));

    let dir = Settings::config_dir();

    assert_eq!(
        dir, named,
        "AC-5: MANHWA_CROPPER_CONFIG_DIR replaces the `directories` layout \
         outright - it is not joined onto it, not consulted only when \
         ProjectDirs fails. This is what keeps this file, MC-012 and MC-016 off \
         the real %APPDATA% folder"
    );
    assert!(
        !dir.exists(),
        "AC-5: and asking where the config directory is is a question about a \
         path, not a visit to it - config_dir() creates nothing. Only save does \
         (AC-4). {} was created just by asking",
        dir.display()
    );
}
