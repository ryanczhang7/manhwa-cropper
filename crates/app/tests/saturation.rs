//! MC-024 AC-3: the `usize` -> `u32` conversion at the runner's boundary,
//! at and past the edge of what the window's type can hold.
//!
//! # Why this cannot be reached through the window
//!
//! `shell::count` is the conversion `ThreadRunner` applies to the engine's
//! `batch::Progress` before it becomes an `Event::Progress` the window can
//! print (`lib.rs`'s module docs give the reason the boundary is there, and
//! MC-014 pinned it). `tests/progress.rs` drives it through a real run, which
//! is the only way to prove it is *wired*; what a real run cannot do is hand
//! it a count that does not fit a `u32`, because that would take a drop of
//! four billion files. The saturating branch `shell.rs` documents is
//! therefore only reachable by calling the function, and that is what this
//! file does.
//!
//! It is a separate binary from `tests/progress.rs` on purpose. Until
//! `shell::count` is public this file does not compile, and a test target
//! that does not compile takes every assertion in its binary with it. Keeping
//! the run-observing tests in their own target is what let RED watch *their*
//! assertions fail (and pass) while this one was still an `E0603`.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled elsewhere**: that this conversion saturates rather than
//!   panicking. `shell.rs`'s doc comment on `count` says so, for the same
//!   reason `Model::files_dropped` saturates - a number the window only
//!   prints must never be able to end the process. This file does not revisit
//!   that decision; it exercises it.
//! * **Mechanical**: every value below. `u32::MAX`, `u32::MAX as usize`,
//!   `u32::MAX as usize + 1`, `usize::MAX`, `0` and `1` are the boundary and
//!   its neighbours, named by MC-024 AC-3, and each expected result follows
//!   from `u32::try_from`.
//! * **Measured**: nothing is an expectation here. Every value *was*
//!   evaluated in RED against a copy of `count`'s body driven outside the
//!   test framework, because this file could not compile at the time and so
//!   none of its assertions had ever run. The numbers are in the story's
//!   `## Handoff: RED -> GREEN`; confirming them against the shipped
//!   `shell::count` is GREEN's job.
//!
//! # "Does not panic" is asserted by arriving
//!
//! AC-3 asks that the conversion not panic. There is no assertion for that
//! and there does not need to be one: `count` is called on the test thread,
//! so an `unwrap` where the `unwrap_or` should be aborts the test at the call
//! and names itself in the failure. The `assert_eq!` on the next line is
//! reached only by a call that returned.

use manhwa_cropper::shell::count;

/// The largest count that fits the window's type, as the engine counts.
const AT_MAX: usize = u32::MAX as usize;

/// One more file than the window's type can hold - AC-3's input.
///
/// `checked_add` rather than `+ 1` so that the one thing a 32-bit target fails
/// on is the control test's `const` assert, which explains itself, instead of
/// a const-evaluation overflow here that says nothing about why.
fn over_max() -> usize {
    AT_MAX.checked_add(1).unwrap_or(AT_MAX)
}

/// A large count that still fits, so that "it saturates" is not mistaken for
/// "it saturates everything big".
const LARGE_BUT_FITTING: usize = 4_000_000_000;

// --- The control that makes the rest of this file mean something -------------

#[test]
fn the_two_boundary_counts_are_distinct_before_any_conversion() {
    const {
        assert!(
            usize::BITS > u32::BITS,
            "this target's `usize` is no wider than `u32`, so a count larger than `u32::MAX` \
             cannot be expressed and nothing in this file exercises the saturating branch it \
             claims to: every AC-3 assertion in this file is vacuous on such a target and must \
             not be read as passing"
        )
    };
    assert_ne!(
        AT_MAX,
        over_max(),
        "`u32::MAX as usize` and `u32::MAX as usize + 1` came out equal, so the second is \
         not in fact past the boundary and the saturation assertion below would hold for a \
         conversion that saturates nothing"
    );
}

// --- AC-3: past the boundary ------------------------------------------------

#[test]
fn a_count_too_large_for_the_window_saturates_instead_of_panicking() {
    assert_eq!(
        count(over_max()),
        u32::MAX,
        "one file more than `u32::MAX` must be shown as `u32::MAX`, not wrapped and not \
         panicked on: this is a number the window only prints"
    );
    assert_eq!(
        count(usize::MAX),
        u32::MAX,
        "the largest count a `usize` can express must saturate the same way - saturation \
         is the branch, not a special case of the value one past the edge"
    );
}

// --- AC-3's controls: at the boundary and below it ---------------------------

#[test]
fn a_count_the_window_can_hold_is_converted_unchanged() {
    assert_eq!(
        count(AT_MAX),
        u32::MAX,
        "`u32::MAX` files fits the window's type exactly, so it must arrive unchanged - a \
         conversion that saturated one file too early would be indistinguishable from a \
         correct one if only the value past the edge were checked"
    );
    assert_eq!(
        count(LARGE_BUT_FITTING),
        4_000_000_000,
        "a count in the billions that still fits must not be clamped: `count` saturates at \
         the type's edge, not at some threshold of its own"
    );
    assert_eq!(count(1), 1, "one file is one file");
    assert_eq!(
        count(0),
        0,
        "an empty count converts to an empty count, and does not panic on the way"
    );
}
