//! The one piece of state that outlives a run: the chosen output folder
//! (MC-013, `docs/wiki/architecture.md` decision 6 and "Where state lives").
//!
//! [`Settings`] is written as `settings.json` under [`Settings::config_dir`],
//! which is `MANHWA_CROPPER_CONFIG_DIR` when that variable is set and the
//! `directories` layout otherwise. There is exactly one setting in v1
//! (decision 11), so this module is small on purpose.
//!
//! # Reading never fails
//!
//! [`load`](Settings::load) returns a `Settings`, not a `Result`: a missing
//! file is a first run, and an unreadable one is a file the app cannot use
//! either way. Both are the defaults, and neither touches the file - a corrupt
//! `settings.json` survives untouched until the next
//! [`save`](Settings::save), which overwrites it whole. The app never fails
//! to start because of its settings.
//!
//! Writing is the opposite: [`save`](Settings::save) answers with
//! `io::Result`, because a folder the user picked and that was silently not
//! remembered is a bug they would only find on the next launch.
//!
//! # Why the directory is also a parameter
//!
//! [`load_from`](Settings::load_from) and [`save_to`](Settings::save_to) are
//! handed the directory and read no environment at all;
//! [`load`](Settings::load) and [`save`](Settings::save) are one-line wrappers
//! that ask [`config_dir`](Settings::config_dir) for it. A test binary is one
//! process with one environment, so a test that has to move
//! `MANHWA_CROPPER_CONFIG_DIR` cannot run beside one that reads it - the pair
//! taking the directory as an argument is what keeps most of MC-013's suite
//! (and MC-012's and MC-016's after it) free of that lock.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// The variable that replaces the `directories` layout outright. Tests always
/// set it, so that nothing in the suite writes to the real config folder.
const CONFIG_DIR_ENV: &str = "MANHWA_CROPPER_CONFIG_DIR";

/// The file, directly inside the config directory.
const SETTINGS_FILE: &str = "settings.json";

/// The application name `directories` builds the layout from. Passed as the
/// application with an empty qualifier and organisation, which on Windows
/// gives `%APPDATA%\manhwa-cropper\config`.
const APPLICATION: &str = "manhwa-cropper";

/// Everything the app remembers between runs: the folder the user last chose
/// to write crops into, or `None` if they never have.
///
/// The path is recorded, not validated - a folder can be deleted or a drive
/// unplugged between two runs, and answering that question is the caller's
/// job at the moment it wants to write.
///
/// `Clone` is for MC-014's `Command::SaveSettings`, which carries a copy of
/// the settings out of the view-model for someone else to write.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct Settings {
    /// Where crops go, once a folder has been chosen.
    ///
    /// Serialised as a present, possibly-null key: `None` is
    /// `{"output_dir": null}` and not an empty object, because MC-016 reads
    /// the key by name.
    pub output_dir: Option<PathBuf>,
}

impl Settings {
    /// Where `settings.json` lives: `MANHWA_CROPPER_CONFIG_DIR` when it is
    /// set, and the `directories` layout otherwise.
    ///
    /// A question about a path, not a visit to it - nothing here creates
    /// anything, and nothing here is cached, so a caller that changes the
    /// variable sees the new answer on the next call.
    #[must_use]
    pub fn config_dir() -> PathBuf {
        if let Some(named) = env::var_os(CONFIG_DIR_ENV) {
            return PathBuf::from(named);
        }
        // The `None` arm needs a machine with no home directory at all, which
        // is why it is one expression: a relative folder of the same shape, so
        // that the app degrades to writing beside itself rather than panicking
        // on a launch that has nothing to do with settings.
        ProjectDirs::from("", "", APPLICATION).map_or_else(
            || PathBuf::from(APPLICATION).join("config"),
            |dirs| dirs.config_dir().to_path_buf(),
        )
    }

    /// The settings in [`config_dir`](Self::config_dir), or the defaults.
    #[must_use]
    pub fn load() -> Self {
        Self::load_from(&Self::config_dir())
    }

    /// Write these settings into [`config_dir`](Self::config_dir), creating
    /// it if it is not there yet.
    ///
    /// # Errors
    ///
    /// If the config directory cannot be created or the file cannot be
    /// written.
    pub fn save(&self) -> io::Result<()> {
        self.save_to(&Self::config_dir())
    }

    /// `dir/settings.json`, or the defaults when it is missing or unreadable.
    ///
    /// Reads no environment: `dir` is the whole answer to "where".
    #[must_use]
    pub fn load_from(dir: &Path) -> Self {
        fs::read(dir.join(SETTINGS_FILE))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Write these settings to `dir/settings.json`, creating every missing
    /// level of `dir` first - a real first run has neither
    /// `manhwa-cropper` nor `config` under `%APPDATA%`.
    ///
    /// Reads no environment: `dir` is the whole answer to "where". The file is
    /// replaced whole, so a corrupt one is gone rather than appended to, and
    /// no temporary file is left beside it.
    ///
    /// # Errors
    ///
    /// If `dir` cannot be created - something in the way is a file, say - or
    /// the file cannot be written.
    pub fn save_to(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)?;
        let document = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        fs::write(dir.join(SETTINGS_FILE), document)
    }
}
