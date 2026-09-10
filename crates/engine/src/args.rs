//! Command-line parsing: `argv` (without the program name) into an
//! [`Invocation`], or an [`ArgError`] naming the flag that was wrong
//! (`docs/wiki/architecture.md`, "cropper-engine", `args`).
//!
//! Hand-rolled on purpose: three flags do not justify a dependency
//! (`docs/wiki/stack.md`). Rules: `--no-gui` may appear anywhere; `--out <v>`
//! and `--summary <v>` take the next item verbatim, a later `--out` wins; any
//! other item starting with `--` is unknown; everything else - including `-`
//! and `a--b.png` - is an input, kept in argv order.

use std::path::PathBuf;

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Invocation {
    /// Input files, in argv order.
    pub inputs: Vec<PathBuf>,
    /// The `--out` folder, if given (the last one wins).
    pub out_dir: Option<PathBuf>,
    /// `--no-gui`: run headlessly and exit instead of opening the window.
    pub no_gui: bool,
    /// The `--summary` file, if given.
    pub summary_path: Option<PathBuf>,
}

/// Why `argv` could not be parsed. Each variant carries the flag as typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgError {
    /// An item starting with `--` that is not a flag this exe knows.
    UnknownFlag(String),
    /// A flag that takes a value was the last item.
    MissingValue(String),
}

/// Parse `argv`, which must exclude the program name (`main.rs` feeds
/// `std::env::args().skip(1)`).
///
/// # Errors
///
/// [`ArgError::UnknownFlag`] for any `--` item that is not `--no-gui`,
/// `--out` or `--summary`; [`ArgError::MissingValue`] when `--out` or
/// `--summary` has no item after it.
pub fn parse<I, S>(argv: I) -> Result<Invocation, ArgError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut inv = Invocation::default();
    let mut items = argv.into_iter();
    while let Some(item) = items.next() {
        let item = item.as_ref();
        match item {
            "--no-gui" => inv.no_gui = true,
            "--out" => inv.out_dir = Some(value_after(item, &mut items)?),
            "--summary" => inv.summary_path = Some(value_after(item, &mut items)?),
            other if other.starts_with("--") => {
                return Err(ArgError::UnknownFlag(other.to_string()));
            }
            other => inv.inputs.push(PathBuf::from(other)),
        }
    }
    Ok(inv)
}

/// The item following `flag`, verbatim, or `MissingValue(flag)`.
fn value_after<S: AsRef<str>>(
    flag: &str,
    items: &mut impl Iterator<Item = S>,
) -> Result<PathBuf, ArgError> {
    items
        .next()
        .map(|v| PathBuf::from(v.as_ref()))
        .ok_or_else(|| ArgError::MissingValue(flag.to_string()))
}
