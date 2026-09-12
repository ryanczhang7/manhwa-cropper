//! Output names, decided before anything is written (MC-010, MC-020).
//!
//! [`plan_outputs`] answers one distinct, currently-free path per input, in
//! input order, accounting both for what is already in the output folder and
//! for duplicates inside the batch itself. On a collision the name grows
//! Explorer's suffix - `a (2).png`, then `a (3).png`: one space, ASCII
//! parentheses, ASCII digits, between the stem and the extension
//! (`docs/wiki/architecture.md` decision 7). Nothing here reads, writes or
//! creates a file; the only question it asks the filesystem is whether a path
//! is taken.
//!
//! "Distinct" means distinct *to the filesystem*, which on Windows ignores
//! letter case: `a.png` and `A.PNG` are one file, so planning them as two
//! names would write one and destroy the other. Names are therefore compared
//! [`fold`]ed, on both halves of the question - the disk's and the batch's own
//! - while the name that gets written stays the input's own bytes (MC-020).
//!
//! # Why the whole batch is planned at once, and up front
//!
//! MC-011 runs the batch on the rayon pool, so two files that would collide
//! are written at the same instant. "Look for a free name, then write it"
//! inside a worker is a race: both workers see `a (2).png` free, both write
//! it, and one of the user's files is gone with no error anywhere. Planning
//! every name first, sequentially, in one pass, is what makes the parallel
//! writes safe - which is why this is a function over the whole batch rather
//! than a check inside [`process_file`](crate::process_file), and why keeping
//! it that way matters more than it looks.
//!
//! # What it does not decide
//!
//! It does not create the output folder - a plan is not a write, and the
//! folder may legitimately not exist yet (decision 6 creates it on demand).
//! The writers in [`process`](crate::process) create the parent of the path
//! they are handed, which is where an unwritable folder becomes an
//! [`Outcome::Failed`](crate::Outcome::Failed) the run summary can report.
//! It also does not require its inputs to exist: it is a function of their
//! *names* and of what is in `out_dir`.

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The first suffix a collision takes. Explorer's convention, and decision
/// 7's: the file that gets the plain name is the first one, so the second is
/// ` (2)` and never ` (1)`.
const FIRST_SUFFIX: u32 = 2;

/// What an input with no file name of its own - `..`, or a bare root - is
/// planned under, so that the plan stays one path per input, positionally.
/// Such an input cannot be processed at all ([`process_file`] reports
/// `Failed` before it writes anything), so this name is a placeholder that
/// keeps the batch aligned rather than a name a user will see.
///
/// [`process_file`]: crate::process_file
const NAMELESS: &str = "output";

/// One free path in `out_dir` for each of `inputs`, in input order.
///
/// A path is free when nothing is on disk at it and nothing earlier in this
/// same batch has been given it. The first input to want a name gets it
/// unchanged; every later claim on that name takes the next ` (n)` that is
/// still free, counting from 2 and skipping whatever is already on disk. The
/// stem's case and the extension's case are the input's own, untouched.
///
/// **Taken is decided without regard to letter case**, on both halves of that
/// question: the disk is asked through [`Path::exists`], which is
/// case-insensitive on the volume this ships on, and the batch's own claims
/// are kept under a [`fold`]ed key for the same reason. So `x/a.png` and
/// `y/A.PNG` are planned as `a.png` and `A (2).PNG`, which are two files,
/// rather than as two names that are one file (MC-020). The fold decides
/// *comparison* only; it never reaches the name that gets written.
///
/// The returned vector has exactly one entry per input, in the same order, so
/// a caller may zip it against `inputs`.
#[must_use]
pub fn plan_outputs(out_dir: &Path, inputs: &[PathBuf]) -> Vec<PathBuf> {
    let mut planned = Vec::with_capacity(inputs.len());
    // What this batch has already handed out. The disk cannot answer for it:
    // nothing is written until every name is planned, which is the whole
    // point (see the module docs). Held as `fold`ed file names - every
    // candidate is in `out_dir`, so the folded name is the whole of what
    // distinguishes one from another, and folding is what keeps `a.png` and
    // `A.PNG` from being handed out as two names for one file (MC-020).
    let mut claimed: HashSet<String> = HashSet::with_capacity(inputs.len());
    for input in inputs {
        let name = input.file_name().unwrap_or(OsStr::new(NAMELESS));
        let mut candidate = out_dir.join(name);
        let mut key = fold(name);
        let mut n = FIRST_SUFFIX;
        while claimed.contains(&key) || candidate.exists() {
            let next = suffixed(name, n);
            key = fold(&next);
            candidate = out_dir.join(next);
            n += 1;
        }
        claimed.insert(key);
        planned.push(candidate);
    }
    planned
}

/// How two names are compared for "is this one taken?", and the only place
/// case is touched: the answer is a key for [`plan_outputs`]'s set and is
/// never written anywhere. The name that reaches the disk is always the
/// input's own bytes (MC-020 AC-4).
///
/// Lower-case, not upper-case. They differ on exactly one pair that matters -
/// `\u{df}` upper-cases to `SS`, while it lower-cases to itself - and this
/// volume keeps `\u{df}.png` and `SS.png` as two files, which lower-casing
/// agrees with and upper-casing would not.
///
/// [`to_string_lossy`](OsStr::to_string_lossy) rather than a fallible decode:
/// `plan_outputs` returns a `Vec` and has nowhere to report an error, so a
/// name this program cannot decode must still get a key. Two such names may
/// fold together and cost one unnecessary suffix, which is the safe direction
/// - a spurious suffix is cosmetic, a missed collision loses a file.
fn fold(name: &OsStr) -> String {
    name.to_string_lossy().to_lowercase()
}

/// `name` with ` (n)` between its stem and its extension: `a.png` and 2 give
/// `a (2).png`, `Shot.PNG` gives `Shot (2).PNG`, and `notes` - which has no
/// extension - gives `notes (2)`, with the suffix at the end and no stray
/// dot.
///
/// Built as an [`OsString`] rather than through a `String`, so a name this
/// program cannot decode is still copied through exactly as it arrived.
fn suffixed(name: &OsStr, n: u32) -> OsString {
    let path = Path::new(name);
    // `file_stem` is the whole name when there is no extension, and `..` is
    // the only shape with no stem at all - it never reaches here through
    // `file_name`, and if it ever does it keeps its own text.
    let mut suffixed = OsString::from(path.file_stem().unwrap_or(name));
    suffixed.push(format!(" ({n})"));
    if let Some(extension) = path.extension() {
        suffixed.push(".");
        suffixed.push(extension);
    }
    suffixed
}
