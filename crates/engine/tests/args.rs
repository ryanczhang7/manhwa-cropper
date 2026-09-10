//! MC-002, AC-5: `cropper_engine::args::parse` turns `argv` (without the
//! program name) into an `Invocation`, or an `ArgError` naming the flag that
//! was wrong. Everything here is mechanical: exact struct values, exact error
//! variants. See the story's `## Test plan` for what is deliberately left
//! unconstrained.

use std::fmt::Debug;
use std::path::PathBuf;

use cropper_engine::args::{ArgError, Invocation, parse};

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

// --- AC-5 as written ---------------------------------------------------------

#[test]
fn no_gui_out_and_two_inputs_parse_into_the_full_invocation_in_order() {
    let got = parse(["--no-gui", "--out", "D:/o", "a.png", "b.png"]);
    assert_eq!(
        got,
        Ok(Invocation {
            inputs: paths(&["a.png", "b.png"]),
            out_dir: Some(PathBuf::from("D:/o")),
            no_gui: true,
            summary_path: None,
        })
    );
}

#[test]
fn summary_flag_sets_summary_path_and_leaves_the_rest_default() {
    let got = parse(["--summary", "s.json", "a.png"]);
    assert_eq!(
        got,
        Ok(Invocation {
            inputs: paths(&["a.png"]),
            out_dir: None,
            no_gui: false,
            summary_path: Some(PathBuf::from("s.json")),
        })
    );
}

#[test]
fn out_with_no_value_is_a_missing_value_error_naming_the_flag() {
    assert_eq!(
        parse(["--out"]),
        Err(ArgError::MissingValue("--out".to_string()))
    );
}

// --- Edges the criteria imply -----------------------------------------------

#[test]
fn summary_with_no_value_is_a_missing_value_error_naming_the_flag() {
    assert_eq!(
        parse(["a.png", "--summary"]),
        Err(ArgError::MissingValue("--summary".to_string()))
    );
}

#[test]
fn an_unknown_double_dash_flag_is_an_unknown_flag_error_naming_it() {
    assert_eq!(
        parse(["--bogus"]),
        Err(ArgError::UnknownFlag("--bogus".to_string()))
    );
}

#[test]
fn an_unknown_flag_after_valid_arguments_is_still_an_error() {
    assert_eq!(
        parse(["--no-gui", "a.png", "--bogus"]),
        Err(ArgError::UnknownFlag("--bogus".to_string()))
    );
}

#[test]
fn empty_argv_is_the_default_invocation_with_nothing_set() {
    let got = parse(Vec::<String>::new());
    assert_eq!(got, Ok(Invocation::default()));
    assert_eq!(
        Invocation::default(),
        Invocation {
            inputs: Vec::new(),
            out_dir: None,
            no_gui: false,
            summary_path: None,
        }
    );
}

#[test]
fn no_gui_may_appear_anywhere_in_argv() {
    let expected = Ok(Invocation {
        inputs: paths(&["a.png"]),
        out_dir: Some(PathBuf::from("o")),
        no_gui: true,
        summary_path: None,
    });
    assert_eq!(parse(["--no-gui", "--out", "o", "a.png"]), expected);
    assert_eq!(parse(["--out", "o", "--no-gui", "a.png"]), expected);
    assert_eq!(parse(["--out", "o", "a.png", "--no-gui"]), expected);
}

#[test]
fn a_second_out_value_overrides_the_first() {
    let got = parse(["--out", "first", "--out", "second", "a.png"]);
    assert_eq!(
        got,
        Ok(Invocation {
            inputs: paths(&["a.png"]),
            out_dir: Some(PathBuf::from("second")),
            no_gui: false,
            summary_path: None,
        })
    );
}

#[test]
fn a_bare_dash_and_anything_not_starting_with_two_dashes_are_inputs() {
    let got = parse(["-", "photo.png", "C:/shots/y.png", "a--b.png"]);
    assert_eq!(
        got,
        Ok(Invocation {
            inputs: paths(&["-", "photo.png", "C:/shots/y.png", "a--b.png"]),
            out_dir: None,
            no_gui: false,
            summary_path: None,
        })
    );
}

#[test]
fn one_input_and_no_flags_is_just_that_input() {
    assert_eq!(
        parse(["only.png"]),
        Ok(Invocation {
            inputs: paths(&["only.png"]),
            ..Invocation::default()
        })
    );
}

// --- The signature the exe relies on ----------------------------------------

/// `main.rs` feeds `std::env::args().skip(1)` (an iterator of `String`) and
/// tests feed string literals, so `parse` takes any `IntoIterator` whose items
/// are `AsRef<str>`. `["--no-gui"].iter()` yields `&&str`, which is `AsRef<str>`
/// but not `Into<String>`: this test is what pins the bound.
#[test]
fn parse_accepts_any_iterator_of_str_like_items() {
    let literals = ["--no-gui"];
    let from_borrowed = parse(literals.iter());
    let from_owned = parse(vec![String::from("--no-gui")]);
    let from_env_shape = parse(std::iter::once(String::from("--no-gui")));
    let expected = Ok(Invocation {
        no_gui: true,
        ..Invocation::default()
    });
    assert_eq!(from_borrowed, expected);
    assert_eq!(from_owned, expected);
    assert_eq!(from_env_shape, expected);
}

fn assert_is_cloneable_value_type<T: Debug + Clone + PartialEq + Eq>(_: &T) {}
fn assert_is_comparable<T: Debug + PartialEq + Eq>(_: &T) {}

#[test]
fn invocation_and_arg_error_are_plain_comparable_value_types() {
    let inv = Invocation::default();
    assert_is_cloneable_value_type(&inv);
    assert_eq!(inv.clone(), inv);

    let err = ArgError::UnknownFlag("--x".to_string());
    assert_is_comparable(&err);
    assert_ne!(err, ArgError::MissingValue("--x".to_string()));
}
