//! MC-022: what the folder row and the flagged list actually *show* when the
//! text is wider than the space they have.
//!
//! # The surface these tests read
//!
//! `egui_kittest`'s `Harness::output()` hands back the `egui::FullOutput` of
//! the last frame, and its `shapes` are the **untessellated** paint list -
//! every `Shape::Text` still carries the `Galley` egui laid out, so
//! `galley.text()` is the string the user would read off the screen and
//! `galley.rect.width()` is how wide it is in points. No renderer, no image,
//! no pixel comparison: this is the painter's own output, read back as text.
//!
//! That is the gap `docs/wiki/audits/app-window-2026-09-13.md`, evidence E-6,
//! left open. `path_label` paints the *truncated* string and then overwrites
//! the AccessKit node's value with the **full** path, so the accessibility
//! tree - the only surface `tests/gui.rs` reads - is true whatever
//! `truncate_left` returned, including `""` and `"xyzzy"`. `flagged_row` does
//! the same for rows. Reading the galley is what tells the two apart.
//!
//! # Why this is a separate file from `tests/gui.rs`
//!
//! `tests/gui.rs` states its own contract in its first paragraph: "Every
//! assertion here is a query against that tree - never a pixel, never a rect,
//! never a colour of a painted shape." These assertions are about a rect and a
//! galley, so they do not belong under that sentence. Nothing in `tests/gui.rs`
//! is changed by this story;
//! `a_long_output_path_keeps_its_whole_path_as_the_accessible_name` stays
//! exactly as MC-015 wrote it, and AC-3 below is a second, independent pin on
//! the same behaviour taken from the same frame as the truncation assertion,
//! so a change made for AC-1 cannot quietly cost it.
//!
//! # Why widths are compared as ratios, never as character counts
//!
//! `docs/backlog/stories/MC-022.md`, "## Out of scope", forbids
//! font-metric-dependent assertions - an exact character count at an exact
//! width would break the day a font package updates. So the shape of the
//! result is what is asserted: which end carries the U+2026, that the survivor
//! is a genuine head or tail of the original, and that the shown text is
//! *substantially* narrower than the text it stands for. That last one is
//! measured against the **same string painted in a window wide enough to hold
//! it**, so both sides of the comparison move together when the font changes.
//! See [`MUCH_NARROWER`] for the threshold and the numbers measured under it.

use std::path::PathBuf;

use cropper_core::FlagReason;
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Flag, Outcome};
use eframe::egui;
use eframe::egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use manhwa_cropper::{AppState, Model, gui};

// --- The frozen strings and sizes --------------------------------------------

/// `layout.md`'s truncation mark. One U+2026, never three dots.
const ELLIPSIS: char = '…';
/// `dropzone.ready`.
const DROPZONE_READY: &str = "Drop images here";
/// `folder.button`. Carries a U+2026 of its own, which is why the path label is
/// identified by elimination rather than by looking for the mark.
const FOLDER_BUTTON: &str = "Choose folder…";

/// Default inner size, logical px (`layout.md`).
const DEFAULT_SIZE: [f32; 2] = [520.0, 440.0];
/// A window far wider than any string below, used only as the reference frame
/// for "how wide would this text be if nothing gave way".
const WIDE_SIZE: [f32; 2] = [3000.0, 440.0];

/// A path that fits the folder row at the default size, for AC-2.
const SHORT_PATH: &str = r"D:\shots\out";

/// A path that does not fit the folder row at the default size, for AC-1.
///
/// Deliberately far too wide - 894.7 pt against the label's ~356 pt - so that
/// a truncation which dropped only one character is still wildly over, and so
/// that no plausible font change makes it fit.
const LONG_PATH: &str = r"C:\Users\ryan\Pictures\manhwa\2026\september\week-38\archive\second-pass\rechecked\by-hand\approved\for-upload\batch-07\overflow\final\cropped";

/// The innermost folder of [`LONG_PATH`]: what `layout.md` says must survive.
const LEAF: &str = r"\cropped";

/// A file name that does not fit one flagged row at the default size, for AC-4.
const LONG_NAME: &str = "chapter-118-a-very-long-scanlation-file-name-that-will-not-fit-in-the-flagged-list-no-matter-how-wide-the-user-drags-the-window-page-0412.png";

/// `list.row`'s separator - space, em dash, space - and `reason.uniform`: the
/// part of a row that never gives way (`components.md`, "overflow").
const BLANK_TAIL: &str = " — Blank";

/// A file name that fits one flagged row, and the whole row it makes.
const SHORT_NAME: &str = "b.gif";
/// `reason.unsupported`.
const SHORT_ROW: &str = "b.gif — Unsupported format";

/// How much narrower the shown text must be than the text it stands for.
///
/// Measured in RED, at the default window size, as `shown / whole`:
///
/// | What | shown | whole | ratio |
/// |---|---|---|---|
/// | [`LONG_PATH`] in the folder row | 340.6 | 894.7 | 0.381 |
/// | [`LONG_NAME`]'s row in the list | 468.4 | 878.4 | 0.533 |
/// | a truncation that dropped one character | ~886 | 894.7 | ~0.99 |
///
/// 0.75 sits between the worst real case (0.533) and the mutant (0.99) with
/// room on both sides, and it is a *ratio of two widths in the same font*, so a
/// font update moves the numerator and the denominator together.
const MUCH_NARROWER: f32 = 0.75;

// --- Building a window to look at --------------------------------------------

fn model(state: AppState, output_dir: Option<&str>) -> Model {
    Model {
        state,
        output_dir: output_dir.map(PathBuf::from),
    }
}

/// Render one frame of `model` at `size` and hand back the harness.
fn painted<'a>(model: &'a Model, size: [f32; 2]) -> Harness<'a> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(size[0], size[1]))
        .build_ui(move |ui| gui::paint(ui, model, false));
    harness.run();
    harness
}

/// One string the painter put on the screen, and how wide it is in points.
#[derive(Debug)]
struct Painted {
    text: String,
    width: f32,
}

/// Every string the last frame painted, in paint order.
///
/// `Shape::Vec` is walked because egui nests shapes: a label inside a scroll
/// area inside a panel arrives as a tree, and a row that was only reachable
/// through the nesting would silently vanish from a flat read.
fn painted_texts(harness: &Harness<'_>) -> Vec<Painted> {
    fn walk(shape: &egui::Shape, out: &mut Vec<Painted>) {
        match shape {
            egui::Shape::Text(text) => out.push(Painted {
                text: text.galley.text().to_owned(),
                width: text.galley.rect.width(),
            }),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    walk(shape, out);
                }
            }
            _ => {}
        }
    }

    let mut out = Vec::new();
    for clipped in &harness.output().shapes {
        walk(&clipped.shape, &mut out);
    }
    out
}

/// What the folder row shows, in an Idle window whose only other strings are
/// the two frozen literals above.
///
/// By elimination rather than by pattern: a query for "the text containing a
/// U+2026" would find `Choose folder…` too, and a query for "the text that
/// looks like a path" would pass for a painter that showed no path at all.
fn path_label_text(harness: &Harness<'_>) -> Painted {
    let mut rest: Vec<Painted> = painted_texts(harness)
        .into_iter()
        .filter(|painted| painted.text != DROPZONE_READY && painted.text != FOLDER_BUTTON)
        .collect();
    assert_eq!(
        rest.len(),
        1,
        "an Idle window with a folder paints the drop zone, the button and the path label and \
         nothing else; the path label is whichever string is left, and there must be exactly one \
         of it. Everything painted: {:#?}",
        painted_texts(harness)
    );
    rest.remove(0)
}

/// The one painted row ending in `tail`.
fn row_text(harness: &Harness<'_>, tail: &str) -> Painted {
    let mut rows: Vec<Painted> = painted_texts(harness)
        .into_iter()
        .filter(|painted| painted.text.ends_with(tail))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "exactly one painted row must end in {tail:?} - the reason word is the part of a row \
         that never gives way (`components.md`, \"overflow\"). Everything painted: {:#?}",
        painted_texts(harness)
    );
    rows.remove(0)
}

/// How many Label nodes carry exactly `text` as their accessible name.
fn labels_named(harness: &Harness<'_>, text: &str) -> usize {
    harness
        .query_all_by_role_and_label(Role::Label, text)
        .count()
}

// --- The checks, as values -----------------------------------------------
//
// Every assertion below goes through one of these four functions, and
// `the_checks_reject_what_each_e6_survivor_would_paint` feeds the same four
// the strings each mutant in the audit's E-6 cluster would have painted. They
// are the same code in both places on purpose: a control that exercised a
// look-alike check would prove nothing about the checks the criteria use.

/// `shown` is `full` with its **head** dropped and one [`ELLIPSIS`] in its
/// place, and `leaf` still visible (`layout.md`, "Output folder row").
fn head_dropped(shown: &str, full: &str, leaf: &str) -> Result<(), String> {
    if shown == full {
        return Err(format!(
            "nothing was dropped: the row showed the whole path {shown:?}, which does not fit"
        ));
    }
    let Some(tail) = shown.strip_prefix(ELLIPSIS) else {
        return Err(format!(
            "the shown text must begin with U+2026 - the path gives way at its head so the \
             innermost folder stays readable - but it is {shown:?}"
        ));
    };
    if tail.contains(ELLIPSIS) {
        return Err(format!(
            "one U+2026 marks a truncation, not several; {shown:?} has {}",
            shown.matches(ELLIPSIS).count()
        ));
    }
    if tail.is_empty() {
        return Err("the shown text is a bare U+2026: the whole path was dropped".to_owned());
    }
    if !full.ends_with(tail) {
        return Err(format!(
            "what is shown after the U+2026 must be the end of the real path, but {tail:?} is not \
             a tail of {full:?}"
        ));
    }
    if !tail.ends_with(leaf) {
        return Err(format!(
            "the innermost folder {leaf:?} is the part that must survive, and {shown:?} does not \
             end with it"
        ));
    }
    Ok(())
}

/// `shown` is `full`, untouched: nothing was dropped and no mark was added.
///
/// The control for [`head_dropped`] and [`tail_dropped`]. A truncation that
/// always fired would satisfy both of those and be wrong.
fn nothing_dropped(shown: &str, full: &str) -> Result<(), String> {
    if shown.contains(ELLIPSIS) {
        return Err(format!(
            "text that fits its space is shown unchanged, with no U+2026 anywhere; got {shown:?}"
        ));
    }
    if shown != full {
        return Err(format!(
            "text that fits its space is shown exactly as it is: wanted {full:?}, got {shown:?}"
        ));
    }
    Ok(())
}

/// `shown` is `name` with its **tail** dropped and one [`ELLIPSIS`] in its
/// place, followed by the whole of `tail` (`components.md`, "overflow").
fn tail_dropped(shown: &str, name: &str, tail: &str) -> Result<(), String> {
    if !shown.ends_with(tail) {
        return Err(format!(
            "the separator and the reason word never give way, so the row must end with {tail:?}; \
             got {shown:?}"
        ));
    }
    let head = shown.strip_suffix(tail).unwrap_or_default();
    if head == name {
        return Err(format!(
            "nothing was dropped: the row showed the whole file name {name:?}, which does not fit"
        ));
    }
    let Some(stem) = head.strip_suffix(ELLIPSIS) else {
        return Err(format!(
            "the file name is what carries the U+2026, so the part before {tail:?} must end with \
             it; got {head:?}"
        ));
    };
    if shown.matches(ELLIPSIS).count() != 1 {
        return Err(format!(
            "one U+2026 marks a truncation, not several; {shown:?} has {}",
            shown.matches(ELLIPSIS).count()
        ));
    }
    if stem.is_empty() {
        return Err("the whole file name was dropped, leaving only the reason".to_owned());
    }
    if !name.starts_with(stem) {
        return Err(format!(
            "what is shown before the U+2026 must be the start of the real file name, but \
             {stem:?} is not a head of {name:?}"
        ));
    }
    Ok(())
}

/// The shown text is substantially narrower than the same text painted where it
/// all fits - i.e. the truncation did the job it exists for.
///
/// See [`MUCH_NARROWER`] for why this is a ratio and what was measured.
fn much_narrower(shown: f32, whole: f32) -> Result<(), String> {
    let ratio = shown / whole;
    if ratio > MUCH_NARROWER {
        return Err(format!(
            "the shown text is {shown} pt wide against {whole} pt for the whole string, a ratio \
             of {ratio:.3}: that is not a truncation, it is the whole thing minus a character or \
             two, and it will not fit the space the row has"
        ));
    }
    Ok(())
}

/// Panic with the check's own message, which reads as the bug report.
#[track_caller]
fn check(verdict: Result<(), String>) {
    if let Err(message) = verdict {
        panic!("{message}");
    }
}

// --- Fixtures ----------------------------------------------------------------

fn flagged(name: &str, reason: Flag) -> FileResult {
    FileResult {
        input: PathBuf::from(format!(r"C:\Users\ryan\Downloads\{name}")),
        outcome: Outcome::Flagged {
            reason,
            output: PathBuf::from(SHORT_PATH).join(name),
        },
    }
}

/// A finished run with one row too wide for the list and one that fits.
///
/// Both in the same frame on purpose: the row that fits is the control for the
/// row that does not, exactly as AC-2 is the control for AC-1.
fn one_long_row_and_one_short() -> RunSummary {
    RunSummary {
        results: vec![
            flagged(LONG_NAME, Flag::Detector(FlagReason::Uniform)),
            flagged(SHORT_NAME, Flag::Unsupported),
        ],
    }
}

// --- AC-1: a path too wide for the folder row --------------------------------

#[test]
fn a_path_too_wide_for_the_folder_row_drops_its_head_and_keeps_its_leaf() {
    let model = model(AppState::Idle, Some(LONG_PATH));
    let shown = path_label_text(&painted(&model, DEFAULT_SIZE));
    let whole = path_label_text(&painted(&model, WIDE_SIZE));

    // The shown text first, so that a painter which shows the wrong thing
    // fails on the criterion rather than on the reference frame. The wide
    // window is only here to say how wide the whole path would have been.
    check(head_dropped(&shown.text, LONG_PATH, LEAF));
    check(nothing_dropped(&whole.text, LONG_PATH));
    check(much_narrower(shown.width, whole.width));
}

// --- AC-2: a path that fits, the control for AC-1 ----------------------------

#[test]
fn a_path_that_fits_the_folder_row_is_shown_whole() {
    let model = model(AppState::Idle, Some(SHORT_PATH));
    let shown = path_label_text(&painted(&model, DEFAULT_SIZE));

    check(nothing_dropped(&shown.text, SHORT_PATH));
}

// --- AC-3: the accessible name is still the whole path -----------------------

#[test]
fn a_truncated_path_still_reads_its_whole_self_to_a_screen_reader() {
    let model = model(AppState::Idle, Some(LONG_PATH));
    let harness = painted(&model, DEFAULT_SIZE);

    assert_eq!(
        labels_named(&harness, LONG_PATH),
        1,
        "the folder row's accessible name is the whole path, truncated on screen or not \
         (`accessibility.md`, \"Names\"): a screen reader must still say where the files went"
    );
    // Taken from the same frame as the name above, so the pair says what AC-3
    // is for: the name survived *while* the shown text was cut.
    check(head_dropped(
        &path_label_text(&harness).text,
        LONG_PATH,
        LEAF,
    ));
}

// --- AC-4: a flagged row too wide for the list -------------------------------

#[test]
fn a_flagged_row_too_wide_for_the_list_drops_the_file_name_not_the_reason() {
    let model = model(
        AppState::Done {
            summary: one_long_row_and_one_short(),
        },
        Some(SHORT_PATH),
    );
    let shown = row_text(&painted(&model, DEFAULT_SIZE), BLANK_TAIL);
    let whole = row_text(&painted(&model, WIDE_SIZE), BLANK_TAIL);

    // Shown first, reference frame second: see the AC-1 test.
    check(tail_dropped(&shown.text, LONG_NAME, BLANK_TAIL));
    check(nothing_dropped(
        &whole.text,
        &format!("{LONG_NAME}{BLANK_TAIL}"),
    ));
    check(much_narrower(shown.width, whole.width));
}

#[test]
fn a_flagged_row_that_fits_the_list_is_shown_whole() {
    let model = model(
        AppState::Done {
            summary: one_long_row_and_one_short(),
        },
        Some(SHORT_PATH),
    );
    let shown = row_text(&painted(&model, DEFAULT_SIZE), "Unsupported format");

    check(nothing_dropped(&shown.text, SHORT_ROW));
}

// --- AC-5: the checks against every survivor in the audit's E-6 --------------

/// The negative control for this whole file.
///
/// Every check above runs, in RED, against code that already works, so each
/// one is green the moment it is written and would stay green if it asserted
/// nothing at all. This test is what earns them: it feeds the same four checks
/// the string each of E-6's eleven survivors would have painted, and requires
/// every one to be rejected - and the real string to be accepted, so that a
/// check which simply refused everything would fail here.
///
/// The two mutations AC-5 names by line are the first and sixth rows of the
/// path table. Confirming these predictions against the *actually mutated*
/// painter is GREEN's job (`## Gate probes`); RED may not write `gui.rs`.
#[test]
fn the_checks_reject_what_each_e6_survivor_would_paint() {
    let whole_row = format!("{LONG_NAME}{BLANK_TAIL}");

    // (what the painter would show, why it is wrong) for the folder row with
    // LONG_PATH, which does not fit.
    let head_rejects: Vec<(String, &str)> = vec![
        (
            LONG_PATH.to_owned(),
            "text_width -> 0.0 / 1.0 / -1.0, and truncate_left's first <= flipped to >: the \
             early return fires and the whole path is painted",
        ),
        (
            String::new(),
            "replace truncate_left -> String with String::new()",
        ),
        (
            "xyzzy".to_owned(),
            "replace truncate_left -> String with \"xyzzy\".into()",
        ),
        (
            ELLIPSIS.to_string(),
            "a truncation that dropped the path and left only the mark",
        ),
        (
            format!("{}{ELLIPSIS}", &LONG_PATH[..20]),
            "the wrong end gave way: the leaf is what the design keeps",
        ),
        (
            format!("{ELLIPSIS}{}", &LONG_PATH[10..30]),
            "a middle slice is not a tail of the real path",
        ),
    ];
    for (shown, why) in &head_rejects {
        assert!(
            head_dropped(shown, LONG_PATH, LEAF).is_err(),
            "AC-1's check accepted {shown:?}, which is what the painter shows when: {why}"
        );
    }
    check(head_dropped(
        &format!("{ELLIPSIS}{}", &LONG_PATH[LONG_PATH.len() - 30..]),
        LONG_PATH,
        LEAF,
    ));

    // The folder row with SHORT_PATH, which fits. This is where
    // truncate_left's first `<=` flipped to `>` shows up as a truncation that
    // should never have fired.
    let fits_rejects: Vec<(String, &str)> = vec![
        (
            format!("{ELLIPSIS}{}", &SHORT_PATH[1..]),
            "truncate_left's first <= flipped to >: the early return is skipped and a path that \
             fits is truncated anyway",
        ),
        (
            "xyzzy".to_owned(),
            "replace truncate_left -> String with \"xyzzy\".into()",
        ),
        (
            String::new(),
            "replace truncate_left -> String with String::new()",
        ),
    ];
    for (shown, why) in &fits_rejects {
        assert!(
            nothing_dropped(shown, SHORT_PATH).is_err(),
            "AC-2's check accepted {shown:?}, which is what the painter shows when: {why}"
        );
    }
    check(nothing_dropped(SHORT_PATH, SHORT_PATH));

    // A flagged row whose file name does not fit.
    let tail_rejects: Vec<(String, &str)> = vec![
        (
            whole_row.clone(),
            "text_width -> 0.0 / 1.0 / -1.0, and truncate_right's first <= flipped to >: the \
             early return fires and the whole name is painted",
        ),
        (
            BLANK_TAIL.to_owned(),
            "replace truncate_right -> String with String::new()",
        ),
        (
            format!("xyzzy{BLANK_TAIL}"),
            "replace truncate_right -> String with \"xyzzy\".into()",
        ),
        (
            format!(
                "{ELLIPSIS}{}{BLANK_TAIL}",
                &LONG_NAME[LONG_NAME.len() - 20..]
            ),
            "the wrong end gave way: components.md keeps the start of the name",
        ),
        (
            format!("{}{ELLIPSIS}", &LONG_NAME[..40]),
            "the separator and the reason word were dropped instead of the name",
        ),
    ];
    for (shown, why) in &tail_rejects {
        assert!(
            tail_dropped(shown, LONG_NAME, BLANK_TAIL).is_err(),
            "AC-4's check accepted {shown:?}, which is what the painter shows when: {why}"
        );
    }
    check(tail_dropped(
        &format!("{}{ELLIPSIS}{BLANK_TAIL}", &LONG_NAME[..40]),
        LONG_NAME,
        BLANK_TAIL,
    ));

    // A flagged row that fits, the control for the one that does not.
    assert!(
        nothing_dropped(&format!("{ELLIPSIS}{}", &SHORT_ROW[1..]), SHORT_ROW).is_err(),
        "AC-4's control accepted a row that fits but was truncated anyway, which is what \
         truncate_right's first <= flipped to > paints"
    );
    check(nothing_dropped(SHORT_ROW, SHORT_ROW));

    // The two loop comparisons - `truncate_left`'s and `truncate_right`'s
    // second `<=` flipped to `>` - return the first candidate they try, which
    // is the whole string minus one character plus the mark. Those pass every
    // check above: they carry the right mark at the right end and the survivor
    // really is a head or a tail. Only the width catches them, which is why
    // `much_narrower` exists. The numbers are the ratios measured in RED.
    assert!(
        much_narrower(886.0, 894.7).is_err(),
        "the width check accepted a `…` + all-but-one-character result, which is what either \
         loop's <= flipped to > paints: it is not narrow enough to fit anything"
    );
    check(much_narrower(340.6, 894.7));
    check(much_narrower(468.4, 878.4));
}
