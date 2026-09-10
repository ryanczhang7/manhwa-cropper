//! The walking-skeleton output path (MC-002): copy an input's bytes
//! unchanged into the output folder under the input's own file name.
//! Nothing is decoded. Collision handling belongs to `naming` (a later
//! story); the summary and the batch runner belong to `batch` (MC-011).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Copy `input` byte-for-byte to `out_dir/<input file name>`, creating
/// `out_dir` (and missing parents) first. Returns the path written.
///
/// The input is read in full before anything is created, so an unreadable
/// input leaves `out_dir` untouched when it already exists.
///
/// # Errors
///
/// Any I/O error from reading the input, creating the directory or writing
/// the copy; `InvalidInput` when `input` has no file name (e.g. `..`).
pub fn copy_unchanged(input: &Path, out_dir: &Path) -> io::Result<PathBuf> {
    let name = input.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} has no file name", input.display()),
        )
    })?;
    let bytes = fs::read(input)?;
    fs::create_dir_all(out_dir)?;
    let target = out_dir.join(name);
    fs::write(&target, bytes)?;
    Ok(target)
}

/// Copy every input with [`copy_unchanged`]; `true` when all of them were
/// written. Every input is attempted - a failure does not stop the ones
/// after it. This is the `--no-gui` walking skeleton: exit 0 when it
/// returns `true`, 1 otherwise (`docs/wiki/architecture.md`,
/// "manhwa-cropper").
#[must_use]
pub fn copy_all(inputs: &[PathBuf], out_dir: &Path) -> bool {
    let failed = inputs
        .iter()
        .filter(|input| copy_unchanged(input, out_dir).is_err())
        .count();
    failed == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes() -> Vec<u8> {
        (0..=255u8).cycle().take(641).collect()
    }

    #[test]
    fn copies_bytes_under_the_input_file_name_into_an_existing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("a.png");
        fs::write(&input, bytes()).unwrap();
        let out = tmp.path().join("out");
        fs::create_dir(&out).unwrap();

        let written = copy_unchanged(&input, &out).unwrap();

        assert_eq!(written, out.join("a.png"));
        assert_eq!(fs::read(&written).unwrap(), bytes());
        assert_eq!(fs::read(&input).unwrap(), bytes());
    }

    #[test]
    fn creates_a_missing_out_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("a.png");
        fs::write(&input, bytes()).unwrap();
        let out = tmp.path().join("not").join("yet");

        let written = copy_unchanged(&input, &out).unwrap();

        assert!(out.is_dir());
        assert_eq!(fs::read(written).unwrap(), bytes());
    }

    #[test]
    fn a_missing_input_is_not_found_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        fs::create_dir(&out).unwrap();
        let missing = tmp.path().join("nope.png");

        let err = copy_unchanged(&missing, &out).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(fs::read_dir(&out).unwrap().count(), 0);
    }

    #[test]
    fn an_input_with_no_file_name_is_invalid_input() {
        let tmp = tempfile::tempdir().unwrap();
        let err = copy_unchanged(Path::new(".."), tmp.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn a_write_failure_is_reported_as_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let input = tmp.path().join("a.png");
        fs::write(&input, bytes()).unwrap();
        // The target path is occupied by a directory, so the write fails.
        let out = tmp.path().join("out");
        fs::create_dir_all(out.join("a.png")).unwrap();

        assert!(copy_unchanged(&input, &out).is_err());
    }

    #[test]
    fn copy_all_is_true_only_when_every_input_was_written() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a.png");
        let b = tmp.path().join("b.png");
        fs::write(&a, bytes()).unwrap();
        fs::write(&b, b"bb").unwrap();
        let missing = tmp.path().join("missing.png");
        let out = tmp.path().join("out");

        assert!(copy_all(&[a.clone(), b.clone()], &out));
        assert_eq!(fs::read(out.join("b.png")).unwrap(), b"bb");
        assert!(copy_all(&[], &out));

        // A failure in the middle does not stop the inputs after it.
        let out2 = tmp.path().join("out2");
        assert!(!copy_all(&[a, missing, b], &out2));
        assert_eq!(fs::read(out2.join("a.png")).unwrap(), bytes());
        assert_eq!(fs::read(out2.join("b.png")).unwrap(), b"bb");
    }
}
