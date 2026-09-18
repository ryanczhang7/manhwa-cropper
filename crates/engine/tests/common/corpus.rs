//! The calibration corpus: real screenshots with hand-marked expected crops
//! (MC-018), and the loader MC-019 measures the detector against.
//!
//! Everything else under `common/` *generates* fixtures. This module does the
//! opposite - it reads files a person chose and rectangles a person drew, and
//! that difference is the whole point of the corpus. A synthetic scene can
//! only confirm the detector does what its author thought; these twenty-eight
//! are the first thing in the project that can contradict it.
//!
//! # The rule the rectangles were drawn to lives on one page
//!
//! **`docs/wiki/corpus.md`** (MC-033) — the marking rule, the diagonal-gutter
//! tolerance, and the tag vocabulary. It is cited here rather than restated,
//! because this comment used to restate it and the restatement drifted: it
//! said "including any non-flat overhang into the gutter", which reads two
//! marks the user has confirmed as correct (`Screenshot (2744).jpg`,
//! `Screenshot (2630).jpg`) as manifest bugs. Two copies of a rule is one copy
//! more than anybody keeps up to date. Add to the page, not to this comment.
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
//!
//! MC-026 adds a second `#[path]`-including target, `tests/corpus.rs`, and
//! Cargo compiles this file separately into each of them - so a helper used by
//! one is dead code in the other (`has_tag` is `corpus_manifest.rs`'s alone).
//! That is the same situation `crates/core/tests/common/mod.rs` documents at
//! greater length, and it has the same answer: the lint stopped being a guard
//! the moment the second target existed. Not an invitation to leave dead
//! helpers behind - add one in the story that first uses it.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use cropper_core::Rect;
use serde::Deserialize;

/// The manifest's file name inside [`dir`].
pub const MANIFEST_FILE: &str = "manifest.json";

/// What a corpus entry claims the right answer is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    /// The tightest axis-aligned rectangle containing one page's own artwork.
    /// The marking rule is on `docs/wiki/corpus.md`, including whose overhang
    /// counts and whose does not.
    Rect(Rect),
    /// The screenshot should be left alone: all art, or too blank to call.
    Flag,
}

/// Which half of the corpus an entry belongs to (MC-036).
///
/// The two values and the rule that gives them meaning are on
/// `docs/wiki/corpus.md`, under `## The tuning / held-out split`, and
/// `tests/corpus_manifest.rs` parses them off that page rather than repeating
/// them - the same arrangement the tag vocabulary has, for the same reason.
///
/// The distinction is methodological, not mechanical: nothing here can enforce
/// that a held-out entry was really scored once. What it can do is make the
/// claim explicit per entry, so that flipping one is a visible edit to a
/// tracked file rather than a decision nobody wrote down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    /// May be looked at, measured against and fitted to without limit. Every
    /// v1 threshold was chosen against these.
    Tuning,
    /// Scored once, at the end, and never tuned against.
    HeldOut,
}

impl Split {
    /// The string a manifest entry carries, which is also what the
    /// `<!-- split-vocabulary -->` table on `docs/wiki/corpus.md` lists.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Split::Tuning => "tuning",
            Split::HeldOut => "held-out",
        }
    }
}

/// One corpus screenshot and the answer a person recorded for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusEntry {
    /// Absolute path to the image inside [`dir`].
    pub path: PathBuf,
    /// What the detector is expected to do with it.
    pub expect: Expect,
    /// Which half of the corpus this entry is in (MC-036). Required on every
    /// manifest entry: an entry with no `split` is an entry whose contamination
    /// status nobody has stated, and a default would state it silently.
    pub split: Split,
    /// Tags from the vocabulary on `docs/wiki/corpus.md`, which
    /// `tests/corpus_manifest.rs` parses off that page and checks every entry
    /// against. The nine MC-018 AC-3 lists must additionally be *covered*
    /// across the corpus; `overhang-text` is documented and deliberately
    /// carried by nothing.
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
/// If the manifest is missing, unparseable, an `expect` string is anything but
/// `"flag"`, or a `split` is absent or not one of the two documented values
/// (MC-036). A corpus that cannot be read is a broken checkout, not a test
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
            let raw_split = raw.split.as_deref().unwrap_or_else(|| {
                panic!(
                    "{}: entry {:?} has no \"split\". Every entry must say which \
                     half of the corpus it is in - {:?} or {:?} - because an \
                     entry whose contamination status nobody stated is one no \
                     accuracy number can be trusted against. See \
                     docs/wiki/corpus.md, \"The tuning / held-out split\"",
                    path.display(),
                    raw.file,
                    Split::Tuning.as_str(),
                    Split::HeldOut.as_str(),
                )
            });
            let split = match raw_split {
                "tuning" => Split::Tuning,
                "held-out" => Split::HeldOut,
                other => panic!(
                    "{}: entry {:?} has split {other:?}; the only values are \
                     {:?} and {:?}, documented on docs/wiki/corpus.md under \
                     \"The tuning / held-out split\"",
                    path.display(),
                    raw.file,
                    Split::Tuning.as_str(),
                    Split::HeldOut.as_str(),
                ),
            };
            CorpusEntry {
                path: root.join(&raw.file),
                expect,
                split,
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
    /// Required in practice - [`load`] panics on `None`. Modelled as an
    /// `Option` rather than a bare `String` so that the panic can name the
    /// *entry*: serde's own missing-field error names the manifest and a byte
    /// offset, and "missing field `split` at line 61 column 7" is a worse
    /// message than "Screenshot (93).jpg has no split" for the person who has
    /// just added a screenshot and forgotten one.
    ///
    /// There is deliberately no `#[serde(default)]`. A default would decide an
    /// entry's contamination status silently, which is the one thing this
    /// field exists to stop.
    split: Option<String>,
}

/// `"flag"` or `{ x, y, w, h }`. Untagged because that is the shape a person
/// writes by hand, and the shape the annotation tool emits.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawExpect {
    Word(String),
    Rect { x: u32, y: u32, w: u32, h: u32 },
}
