//! MC-069, AC-2 to AC-4: the screenshots the app called `Ambiguous` in real
//! use are cropped, and contain the page the user marked.
//!
//! The user ran the app on `Downloads\mahwa panels` (2026-10-01) and it left
//! 14 screenshots uncropped as `Ambiguous`. MC-069 adds them to the corpus as
//! `tuning` entries (`corpus_manifest.rs` pins the files, the marks and the
//! tags) and ships one rule: a strip that is *nearly* chrome makes the
//! decision `Ambiguous` only if the final crop includes at least one of its
//! pixels.
//!
//! On 13 of them the close call is the browser scrollbar, `x 2545..2560`,
//! and the crop `detect` returns lies wholly left of it - so after MC-069
//! each must be `Cropped`, and the crop must contain the user's mark. On
//! `main` each is `Flagged Detector(Ambiguous)`, at both margins, and the
//! tables below print that per file: that is AC-2's reproduction and the
//! reason these tests are red there.
//!
//! The 14th, `Screenshot (2705).png`, is the named known exception (AC-3):
//! its close call is the page's own bottom rows, inside the crop, so the rule
//! leaves it flagged, and MC-070 fixes it. **MC-070 AC-1** turns its pin
//! here round: it is `Cropped` at both margins and contains its mark.
//!
//! AC-4's one measured exception is here too: `2025-07-17 14_20_23.png`, MC-056's
//! known `Ambiguous` entry, whose close call is the same scrollbar. It crops
//! now and must contain its mark.
//!
//! "Both margins" is MC-069's: `margin_px` 0 (the default) and 3.
//!
//! `#[ignore]`d like every corpus suite; the `integration` gate runs it.
//!
//! ```text
//! cargo test -p cropper-engine --release --test corpus_reported_ambiguous -- --ignored --nocapture
//! ```
//!
//! **Held-out discipline.** `tuning` entries only, by name.

// Included directly rather than through `common/mod.rs`, for the reason
// `tests/corpus_manifest.rs` gives at the same line.
#[path = "common/corpus.rs"]
mod corpus;

use corpus::{CorpusEntry, Expect, Split};
use cropper_core::{Rect, Tuning};
use cropper_engine::{Outcome, process_file};

/// MC-069 AC-2 and AC-3: the 13 of the 14 whose close call is the browser
/// scrollbar, outside the crop, in the order the user listed them (`a01` to
/// `a13`). Their marks are the manifest's, which `corpus_manifest.rs` pins to
/// the user's frozen table (`Screenshot (42).png` as amended).
const THE_THIRTEEN: [&str; 13] = [
    "Screenshot (14).png",
    "Screenshot (19).png",
    "Screenshot (20).png",
    "Screenshot (23).png",
    "Screenshot (42).png",
    "Screenshot (48).png",
    "Screenshot (49).png",
    "Screenshot (50).png",
    "Screenshot (51).png",
    "Screenshot (52).png",
    "Screenshot (53).png",
    "Screenshot (57).png",
    "Screenshot (58).png",
];

/// MC-069 AC-3's named known exception (`a14`), whose close call lies inside
/// its crop: `Flag(Ambiguous)` at both margins until MC-070, whose AC-1 crops
/// it.
const SCREENSHOT_2705: &str = "Screenshot (2705).png";

/// MC-069 AC-4's one measured exception: MC-056's known `Ambiguous` entry,
/// whose close call is the same scrollbar (flat fraction 0.849273), outside
/// its crop.
const MC056_SCROLLBAR_ENTRY: &str = "2025-07-17 14_20_23.png";

/// Both margins, as MC-069 defines them: 0 and 3. The 0 is written out rather
/// than read from `Tuning::default()` so the pair cannot collapse to one
/// margin if the default ever moves to 3.
fn both_margins() -> [Tuning; 2] {
    [
        Tuning {
            margin_px: 0,
            ..Tuning::default()
        },
        Tuning {
            margin_px: 3,
            ..Tuning::default()
        },
    ]
}

/// The `tuning` entry named `name`, with its marked rect. Panics if the
/// corpus does not carry it so: `corpus_manifest.rs` is where that is judged,
/// and a test here that skipped a missing entry would pass on fewer files.
fn marked_tuning(name: &str) -> (CorpusEntry, Rect) {
    let entry = corpus::load()
        .into_iter()
        .find(|entry| entry.name() == name)
        .unwrap_or_else(|| panic!("{name} is not in the corpus manifest"));
    assert_eq!(
        entry.split,
        Split::Tuning,
        "{name} must be a `tuning` entry (MC-069 AC-1)"
    );
    let Expect::Rect(mark) = entry.expect else {
        panic!("{name} must carry a marked rect (MC-069 AC-1)");
    };
    (entry, mark)
}

/// Whether `outer` contains `inner` entirely - MC-026 AC-5's predicate, the
/// definition of "not a clip".
fn contains(outer: Rect, inner: Rect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.w >= inner.x + inner.w
        && outer.y + outer.h >= inner.y + inner.h
}

/// `process_file`'s outcome for `entry` at `t`.
fn outcome(entry: &CorpusEntry, tmp: &tempfile::TempDir, t: &Tuning) -> Outcome {
    let output = tmp.path().join(entry.name());
    process_file(&entry.path, &output, t).outcome
}

fn shown(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Cropped { rect, .. } => {
            format!("Cropped {},{} {}x{}", rect.x, rect.y, rect.w, rect.h)
        }
        Outcome::Flagged { reason, .. } => format!("Flagged {reason:?}"),
        Outcome::Failed { error } => format!("Failed {error}"),
    }
}

fn shown_rect(r: Rect) -> String {
    format!("{},{} {}x{}", r.x, r.y, r.w, r.h)
}

/// Print a table under `--nocapture` and return it for the assertion
/// message, so the same rows are visible whether the test passes or fails.
fn table(title: &str, rows: &[String]) -> String {
    let mut out = format!("{title}\n");
    for row in rows {
        out.push_str(row);
        out.push('\n');
    }
    println!("{out}");
    out
}

/// Every way `names` fail "`Cropped` at both margins, containing the mark",
/// one row per `(file, margin)`, and the printed table of all of them.
fn cropped_containing_the_mark(names: &[&str]) -> (Vec<String>, String) {
    let tmp = tempfile::tempdir().expect("a temp dir");
    let mut rows = Vec::new();
    let mut wrong = Vec::new();
    for &name in names {
        let (entry, mark) = marked_tuning(name);
        for t in both_margins() {
            let got = outcome(&entry, &tmp, &t);
            let verdict = match &got {
                Outcome::Cropped { rect, .. } if contains(*rect, mark) => "ok",
                Outcome::Cropped { .. } => {
                    wrong.push(format!(
                        "{name} at margin_px {}: {} CLIPS the mark {}",
                        t.margin_px,
                        shown(&got),
                        shown_rect(mark)
                    ));
                    "CLIPS"
                }
                _ => {
                    wrong.push(format!(
                        "{name} at margin_px {}: {}, not Cropped",
                        t.margin_px,
                        shown(&got)
                    ));
                    "NOT CROPPED"
                }
            };
            rows.push(format!(
                "{name:<26} margin {}  mark {:<20} {:<32} {verdict}",
                t.margin_px,
                shown_rect(mark),
                shown(&got)
            ));
        }
    }
    let printed = table("file                       margin  mark / outcome", &rows);
    (wrong, printed)
}

// --- AC-2 and AC-3: the 13 are cropped and contain their marks ---------------

/// MC-069 AC-2 and AC-3. Each of the 13 is `Cropped` at margin 0 and at
/// margin 3, and the crop contains the user's mark.
///
/// **Red on `main`**, for exactly AC-2's reason: every one of the 26
/// `(file, margin)` pairs is `Flagged Detector(Ambiguous)`, and the table in
/// the message says so per file. After MC-069 the crops are the rects
/// `detect` already returns on `main` - `corpus_tuning_crops_unmoved.rs` pins
/// them - which equal the mark on 12 and contain it on `(48)`.
#[test]
#[ignore = "integration: decodes the corpus"]
fn the_thirteen_screenshots_the_app_called_ambiguous_are_cropped_and_contain_their_marks() {
    let (wrong, printed) = cropped_containing_the_mark(&THE_THIRTEEN);
    assert!(
        wrong.is_empty(),
        "MC-069 AC-2/AC-3: the 13 screenshots the user's run answered `Ambiguous` on, \
         whose only close call is the browser scrollbar at x 2545..2560 that the crop \
         lies wholly left of, must each be Cropped at margin_px 0 and 3 and contain \
         the page the user marked. A close call on a strip the crop excludes changes \
         nothing in the answer. {} of {} fail:\n{}\n\n{printed}",
        wrong.len(),
        2 * THE_THIRTEEN.len(),
        wrong.join("\n")
    );
}

/// MC-070 AC-1: `Screenshot (2705).png`, MC-069's named known exception
/// until MC-070, is `Cropped` at margin 0 and at margin 3, and the crop
/// contains the user's mark, read from the manifest: `1073,133 399x1259`
/// since MC-070's `## Amendments` (the user, "Mark ends at 1471"; it was
/// `400x1259`).
///
/// **Red on `main`** (`eb54767`), where it is `Flagged Detector(Ambiguous)`
/// at both: its bottom strip, the page's own white rows between two dark
/// margins, is called a close call inside the crop (cause A). `main`'s
/// `detect` rect is already the mark.
///
/// Until MC-070 this place held the opposite pin, that `(2705)` stays
/// flagged, which also guarded MC-069's rule against "drop the close call
/// wholesale". That guard now lives in `crates/core/tests/decide_page_bottom_rows.rs`
/// (two edge-to-edge controls that must still flag) and in MC-069's own
/// `decide_near_strip_outside_crop.rs` control.
#[test]
#[ignore = "integration: decodes the corpus"]
fn screenshot_2705_is_cropped_and_contains_its_mark_at_both_margins() {
    let (wrong, printed) = cropped_containing_the_mark(&[SCREENSHOT_2705]);
    assert!(
        wrong.is_empty(),
        "MC-070 AC-1: {SCREENSHOT_2705}'s bottom strip is the page's own white rows \
         between two dark margins, not a close call; it must be Cropped at margin_px 0 and 3 and contain the page the user marked:\n{}\
         \n\n{printed}",
        wrong.join("\n")
    );
}

// --- AC-4: MC-056's known exception crops now --------------------------------

/// MC-069 AC-4's one measured exception. `2025-07-17 14_20_23.png` was
/// MC-056's known `Ambiguous` entry; its close call is the same browser
/// scrollbar, outside its crop, so it is `Cropped` at both margins now and
/// must contain its mark. **Red on `main`**, where it is `Flagged
/// Detector(Ambiguous)` at both.
#[test]
#[ignore = "integration: decodes the corpus"]
fn mc056s_known_ambiguous_entry_is_cropped_and_contains_its_mark() {
    let (wrong, printed) = cropped_containing_the_mark(&[MC056_SCROLLBAR_ENTRY]);
    assert!(
        wrong.is_empty(),
        "MC-069 AC-4: {MC056_SCROLLBAR_ENTRY}'s close call is the browser scrollbar, \
         which its crop lies wholly left of, so it leaves KNOWN_AMBIGUOUS and must be \
         Cropped at margin_px 0 and 3, containing its mark:\n{}\n\n{printed}",
        wrong.join("\n")
    );
}
