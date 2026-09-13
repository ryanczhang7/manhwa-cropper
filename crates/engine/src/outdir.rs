//! Where a run's output goes, decided once (MC-012, AC-4 to AC-6).
//!
//! The rule is `docs/wiki/architecture.md` decision 6, in precedence order:
//!
//! 1. `--out` on the command line - a one-off, and it wins;
//! 2. otherwise the folder the user last chose in the window
//!    ([`Settings::output_dir`](crate::settings::Settings::output_dir));
//! 3. otherwise a `cropped` folder **beside the first input**, so that the
//!    very first Send-to run needs no interaction either.
//!
//! # Why this is its own module, and not part of `args` or `settings`
//!
//! It reads both and belongs to neither: `args` turns argv into an
//! [`Invocation`] and knows nothing that outlives a run, `settings` is the
//! one thing that does and knows nothing about a command line. The rule that
//! ranks them is a third thing, and the smallest place to put it is here.
//!
//! It lives in the engine rather than in the exe for a reason the `coverage`
//! gate makes concrete: that gate runs with
//! `--ignore-filename-regex 'crates[/\\]app[/\\]src[/\\](main|gui)\.rs'`, so a
//! resolution rule left in `main.rs` would not be measured at all.
//!
//! # Why it creates nothing
//!
//! Resolving is a question about a path, not a visit to it: nothing here
//! touches the file system, and the input need not exist. AC-5's "the folder
//! having been created" is [`batch::run`](crate::batch::run)'s
//! `create_dir_all`, which happens when the run starts - and MC-015 wants to
//! show the user where files will go *before* they commit to a run.

use std::path::PathBuf;

use crate::args::Invocation;
use crate::settings::Settings;

/// The folder made beside the first input when no folder was ever chosen
/// (`docs/wiki/architecture.md` decision 6).
const FALLBACK: &str = "cropped";

/// The folder this run writes into: `--out`, else the remembered folder, else
/// a `cropped` folder beside the first input.
///
/// Total, and pure. An input with no folder component - `shot.png`, as a
/// script's working-directory-relative argument - and an empty input list
/// both degenerate to a bare relative `cropped`, which the writer resolves
/// against the working directory exactly as it resolves the input. (An empty
/// list never reaches a real run: `--no-gui` with no inputs is exit 2 before
/// a folder is needed.)
#[must_use]
pub fn resolve_out_dir(inv: &Invocation, settings: &Settings) -> PathBuf {
    if let Some(one_off) = &inv.out_dir {
        return one_off.clone();
    }
    if let Some(remembered) = &settings.output_dir {
        return remembered.clone();
    }
    // `Path::parent` is `Some("")` for a bare file name, and joining onto an
    // empty path is the bare name again - so `shot.png` gives `cropped` with
    // no leading component, and the `None` arm is only a path that is nothing
    // but a root, plus the empty input list.
    inv.inputs
        .first()
        .and_then(|first| first.parent())
        .map_or_else(|| PathBuf::from(FALLBACK), |dir| dir.join(FALLBACK))
}
