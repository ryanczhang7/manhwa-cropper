//! The calibration corpus: real screenshots with hand-marked expected crops
//! (MC-018), and the loader MC-019 measures the detector against.
//!
//! Everything else under `common/` *generates* fixtures. This module does the
//! opposite - it reads files a person chose and rectangles a person drew, and
//! that difference is the whole point of the corpus. A synthetic scene can
//! only confirm the detector does what its author thought; these twenty-eight
//! are the first thing in the project that can contradict it.
//!
//! # The manifest is the oracle, and it can be wrong
//!
//! `fixtures/corpus/manifest.json` carries one entry per file: either a
//! rectangle, or `"flag"` for a screenshot that should be left alone. Nothing
//! here validates that a rectangle is *correct* - no code can. What
//! `tests/corpus_manifest.rs` checks is that every entry is well formed and
//! lands inside its image; whether the rectangle is the right rectangle is a
//! judgement only the person who drew it can make, and MC-019's guidance says
//! so explicitly: a reported "clip" may be a mis-marked rect rather than a
//! detector fault.
//!
//! # Why the paths are built, not stored
//!
//! A manifest entry names a bare file, never a path. The corpus is one flat
//! directory by construction, so a name is the whole address - and a manifest
//! that could reach out of `fixtures/corpus/` would be a manifest that could
//! point anywhere on the machine running the tests.

use std::fs;
use std::path::{Path, PathBuf};

use cropper_core::Rect;
use serde::Deserialize;

/// The manifest's file name inside [`dir`].
pub const MANIFEST_FILE: &str = "manifest.json";

/// What a corpus entry claims the right answer is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    /// The tightest axis-aligned rectangle containing all the artwork,
    /// including any non-flat overhang into the gutter (MC-018's geometric
    /// marking rule).
    Rect(Rect),
    /// The screenshot should be left alone: all art, or too blank to call.
    Flag,
}

/// One corpus screenshot and the answer a person recorded for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusEntry {
    /// Absolute path to the image inside [`dir`].
    pub path: PathBuf,
    /// What the detector is expected to do with it.
    pub expect: Expect,
    /// Free-form tags. The nine MC-018 AC-3 lists must be covered across the
    /// corpus; extras (`diagonal-gutter`, `overhang-text`) are valid and exist
    /// so MC-019 can report on those cases separately.
    pub tags: Vec<String>,
}

impl CorpusEntry {
    /// The bare file name, which is what a manifest entry and a failure
    /// message both name.
    #[must_use]
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Whether this entry carries `tag`.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// `fixtures/corpus/`, from this crate's manifest directory.
///
/// `CARGO_MANIFEST_DIR` is `crates/engine`, so the repository root is two
/// levels up. Built rather than searched for, so a test run from anywhere
/// finds the same directory.
#[must_use]
pub fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("corpus")
}

/// Every entry in `manifest.json`, **in manifest order** (MC-018 AC-5).
///
/// Order is part of the contract: MC-019 prints a per-file table, and a table
/// whose rows move between runs is one nobody can diff.
///
/// # Panics
///
/// If the manifest is missing, unparseable, or an `expect` string is anything
/// but `"flag"`. A corpus that cannot be read is a broken checkout, not a test
/// failure to be reported politely - every message below says which file and
/// what was wrong with it.
#[must_use]
pub fn load() -> Vec<CorpusEntry> {
    let root = dir();
    let path = root.join(MANIFEST_FILE);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let parsed: Manifest = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("parsing {}: {err}", path.display()));

    parsed
        .entries
        .into_iter()
        .map(|raw| {
            let expect = match raw.expect {
                RawExpect::Word(word) if word == "flag" => Expect::Flag,
                RawExpect::Word(word) => panic!(
                    "{}: entry {:?} has expect {word:?}; the only string form is \"flag\"",
                    path.display(),
                    raw.file
                ),
                RawExpect::Rect { x, y, w, h } => Expect::Rect(Rect { x, y, w, h }),
            };
            CorpusEntry {
                path: root.join(&raw.file),
                expect,
                tags: raw.tags,
            }
        })
        .collect()
}

// --- The manifest on the wire -----------------------------------------------

#[derive(Deserialize)]
struct Manifest {
    entries: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawEntry {
    file: String,
    expect: RawExpect,
    tags: Vec<String>,
}

/// `"flag"` or `{ x, y, w, h }`. Untagged because that is the shape a person
/// writes by hand, and the shape the annotation tool emits.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawExpect {
    Word(String),
    Rect { x: u32, y: u32, w: u32, h: u32 },
}
