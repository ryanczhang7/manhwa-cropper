//! MC-030: where the status slot and the flagged row put their text.
//!
//! # The surface these tests read
//!
//! `egui_kittest`'s `Harness::output()` hands back the `egui::FullOutput` of
//! the last frame, and its `shapes` are the **untessellated** paint list.
//! MC-022 read `Shape::Text` out of it to pin what the window *says*
//! (`tests/truncation.rs`), MC-021 read `Shape::Rect` to pin what colour it is
//! (`tests/palette.rs`), and MC-029 read `Shape::LineSegment` to pin the drop
//! zone's border (`tests/dropzone_border.rs`). This file reads the *positions*
//! of the first two: a `TextShape` carries `pos` and a galley whose `rect` is
//! the box the glyphs occupy, and a `RectShape` carries its `rect`. No
//! renderer, no image, no pixel comparison.
//!
//! # Why this is a separate file
//!
//! `tests/gui.rs` states its contract in its first paragraph: "Every assertion
//! here is a query against that tree - never a pixel, never a rect, never a
//! colour of a painted shape." Every assertion here is the *rect* of a painted
//! shape. `tests/palette.rs` is about colour and says so; `tests/truncation.rs`
//! is about what the window says and says so; `tests/dropzone_border.rs` is
//! about one widget's boundary and says so, and its own docs hand the status
//! slot's centring to this story. Nothing in any of them is modified.
//!
//! # The problem this file had to solve: the status slot paints no boundary
//!
//! "The space above the line equals the space below it" is a statement about
//! the **slot**, and `status_slot` paints no rect of its own - it is an
//! `allocate_ui_with_layout` with `set_min_size`, which reserves space and
//! draws nothing. So the slot's top and bottom edges are not in the paint
//! list, and MC-030's `## Model guidance` forbids writing them down as
//! absolute y coordinates, because a constant like `192.0` encodes the drop
//! zone's height, the folder row's height, the window margin and every gap
//! between them - none of which this story is about.
//!
//! The slot is therefore **derived from its neighbour above**, the folder
//! button, which does paint a rect ([`status_slot`]):
//!
//! ```text
//! slot.top    = the folder button's painted bottom edge + `space-stack`
//! slot.bottom = slot.top + `size-status`
//! ```
//!
//! `paint` stacks the four regions in one `Ui` whose `item_spacing.y` is
//! `space-stack`, and `layout.md`'s column table gives the folder row
//! `size-control` - the button's own height, which is why the button's bottom
//! edge *is* the row's. Two constants (`space-stack`, `size-status`) and one
//! painted measurement; nothing above the folder row enters it, so moving the
//! drop zone or the window margin cannot break a test that is about none of
//! those things.
//!
//! Three properties made the folder button the anchor rather than the flagged
//! list below:
//!
//! - it is painted in **every** state, so AC-1's `Done` frame and AC-2's
//!   `Processing` frame use the same anchor in the same frame they measure.
//!   The list paints nothing at all while a run is under way, so an anchor
//!   taken from below would have forced AC-2 to compare two different frames;
//! - it sits **above** the slot, so it cannot move when the slot's own
//!   arithmetic is wrong. That is not hypothetical:
//!   `allocate_ui_with_layout` allocates `max(min_size, content)`, so a mutant
//!   that centres a 16 pt line as though it were 56 pt overflows the 40 pt slot
//!   and pushes everything below it **down by 4 pt**. An anchor taken from
//!   below would have followed the defect and reported a smaller error than
//!   the real one;
//! - it is found by the caption painted inside it
//!   ([`innermost_rect_containing`], MC-021's helper verbatim), so the
//!   identification does not depend on a position, which is what these tests
//!   measure.
//!
//! The anchor is checked rather than assumed:
//! `the_status_slot_these_gaps_are_measured_in_is_where_both_its_neighbours_put_it`
//! derives the slot from the button above and confirms the flagged list's own
//! painted box begins exactly `space-stack` below it.
//!
//! # Where the tolerance comes from
//!
//! Not from what the painter happens to produce. Every length in the chain -
//! `size-status`, `space-stack`, `space-tight`, `size-progress`, the button's
//! bottom edge - is exact. Exactly one step is not: **egui reserves a line's
//! unrounded row height and paints a galley whose row height is rounded to a
//! whole physical pixel** (`epaint-0.36.2/src/text/text_layout.rs:971`,
//! `max_row_height = point_scale.round_to_pixel(max_row_height)`). The painter
//! centres the height it reserved; these tests measure the height it painted;
//! the difference between the two is half a pixel at worst. See
//! [`tolerance`] for the arithmetic and the measured numbers.
//!
//! # What is deliberately not asserted
//!
//! No absolute coordinate: every assertion is a relation between two
//! measurements taken from the same frame. No colour and no text style
//! (MC-021). No dash, radius or stroke (MC-029). Not *which* end of a long
//! name gives way or where the U+2026 goes - that is MC-022's, and AC-3 is
//! about the budget, not the cut. Not the idle slot, which paints nothing.

use std::path::PathBuf;

use cropper_core::FlagReason;
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Flag, Outcome};
use eframe::egui;
use eframe::egui::{Pos2, Rect};
use egui_kittest::Harness;
use manhwa_cropper::{AppState, Model, gui};

// --- The frozen numbers (docs/wiki/design/tokens.md, layout.md) --------------

/// `space-stack`: "vertical gap between the four stacked regions"
/// (`tokens.md`, "Spacing"). The gap between the folder row and the slot, and
/// between the slot and the list.
const SPACE_STACK: f32 = 8.0;

/// The gap between the progress bar and the count beneath it. `layout.md`, the
/// status slot: "the bar (4 px, full width) sits above the count label
/// (`small`), **4 px apart**". `tokens.md`'s spacing ramp has 4 as its base
/// step; `gui.rs` calls the constant `SPACE_TIGHT` and MC-030's AC-2 names it
/// `space-tight`, which is not a row in `tokens.md`'s table - the value is
/// nonetheless fixed unambiguously by the sentence above.
const SPACE_TIGHT: f32 = 4.0;

/// `size-status`: "fixed height of the status slot" (`tokens.md`), and
/// `layout.md`'s column table, region 3.
const SIZE_STATUS: f32 = 40.0;

/// `size-control`: "height of every interactive control **and of the folder
/// row**" (`tokens.md`). Used only as a guard on the anchor: it is what makes
/// the folder button's painted bottom edge the folder row's bottom edge.
const SIZE_CONTROL: f32 = 32.0;

/// emath's `GUI_ROUNDING` (`emath-0.36.2/src/gui_rounding.rs:18`): the 1/32 pt
/// grid `Ui::allocate_space` snaps positions and sizes to. Part of
/// [`tolerance`]'s derivation, and never compared against.
const GUI_ROUNDING: f32 = 1.0 / 32.0;

/// MC-030's AC-1: "within a tolerance **no larger than one point**". The
/// criterion's own ceiling, which [`tolerance`] is checked against rather than
/// used as.
const TOLERANCE_CEILING: f32 = 1.0;

// --- The frozen strings (docs/wiki/design/voice.md, via components.md) -------

/// `folder.button`. Names the anchor, and is never asserted here (MC-021).
const FOLDER_BUTTON: &str = "Choose folder…";

/// `dropzone.ready`. Names the content column in a `Done` frame.
const DROPZONE_READY: &str = "Drop images here";

/// `dropzone.busy`. Names the content column in a `Processing` frame.
const DROPZONE_BUSY: &str = "Cropping…";

/// `result.line` for a run that cropped nothing and flagged one file -
/// `result_line`'s "{cropped} cropped, {flagged} flagged". Used to *find* the
/// line, never asserted: MC-015's suite pins the wording.
const RESULT_LINE: &str = "0 cropped, 1 flagged";

/// `progress.count` for 1 of 4 - `progress_text`'s "{done} of {total}". Used
/// to find the count label, never asserted.
const PROGRESS_COUNT: &str = "1 of 4";

/// `list.row`'s separator and `reason.uniform`: the part of a flagged row that
/// never gives way (`components.md`, "overflow").
const BLANK_TAIL: &str = " — Blank";

/// A file name far too long for one row at the default size, so that AC-3's
/// "whose file name is too long for the list" is unambiguously satisfied.
/// MC-022's `LONG_NAME`, so both stories measure the same string.
const LONG_NAME: &str = "chapter-118-a-very-long-scanlation-file-name-that-will-not-fit-in-the-flagged-list-no-matter-how-wide-the-user-drags-the-window-page-0412.png";

/// An output path short enough that the folder row never truncates, so nothing
/// here depends on MC-022's arithmetic.
const OUT_DIR: &str = "D:/shots/out";

/// Default inner size, logical px (`layout.md`). The window every other test in
/// this crate paints.
const DEFAULT_SIZE: [f32; 2] = [520.0, 440.0];

// --- Building a window to look at --------------------------------------------

fn model(state: AppState, output_dir: Option<&str>) -> Model {
    Model {
        state,
        output_dir: output_dir.map(PathBuf::from),
    }
}

/// Render `model` at the default size and hand back the harness.
///
/// `install_style` is called inside the closure because it is what installs the
/// text styles: without it `body` and `small` are egui's own sizes, every line
/// height changes, and the frame measured would not be the window's.
fn painted(model: &Model) -> Harness<'_> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(DEFAULT_SIZE[0], DEFAULT_SIZE[1]))
        .build_ui(move |ui| {
            gui::install_style(ui.ctx());
            gui::paint(ui, model, false);
        });
    harness.run();
    harness
}

/// A finished run with exactly one flagged file.
fn one_flagged(name: &str) -> RunSummary {
    RunSummary {
        results: vec![FileResult {
            input: PathBuf::from("C:/Users/ryan/Downloads").join(name),
            outcome: Outcome::Flagged {
                reason: Flag::Detector(FlagReason::Uniform),
                output: PathBuf::from(OUT_DIR).join(name),
            },
        }],
    }
}

// --- The tolerance ------------------------------------------------------------

/// How far apart the two gaps may be before the content is not centred.
///
/// **Derived from rounding, not from the painter.** Two steps in the chain
/// between "what `status_slot` centred" and "what the paint list shows" are
/// inexact, and nothing else is:
///
/// 1. **Half a physical pixel.** `centred_line` centres
///    `ui.text_style_height(&TextStyle::Body)` - egui's *unrounded* row height
///    for the font - but the galley it then paints has its row height rounded
///    to a whole physical pixel (`epaint-0.36.2/src/text/text_layout.rs:971`,
///    `max_row_height = point_scale.round_to_pixel(max_row_height)`). The gap
///    above minus the gap below is exactly `painted - reserved`, so this
///    contributes at most `0.5 / pixels_per_point`. At the harness's 1 pixel
///    per point that is 0.5 pt, which is why the tolerance is read off the
///    harness rather than written down.
/// 2. **One 1/32 pt grid step.** `Ui::allocate_space` rounds the space above
///    the content to a multiple of [`GUI_ROUNDING`]. A shift of `d` there
///    lengthens one gap and shortens the other, so it moves their difference
///    by `2d`, and `|d|` is at most half a grid step: `2 * (1/64)` = 1/32.
///
/// So `0.5 / ppp + 1/32` = **0.53125 pt** here, inside AC-1's stated ceiling of
/// [`TOLERANCE_CEILING`], which this function asserts rather than assumes.
///
/// What it has to separate, measured in RED at the default size:
///
/// | Frame | gap above | gap below | difference | vs tolerance |
/// |---|---|---|---|---|
/// | `Done`, shipped | 11.9375 | 12.0625 | 0.125 | 0.24x |
/// | `Processing`, shipped | 9.09375 | 8.90625 | 0.1875 | 0.35x |
/// | `Done`, `(SIZE_STATUS + content) / 2` | 28.0625 | -4.0625 | 32.125 | **60x** |
/// | `Processing`, `SIZE_PROGRESS - SPACE_TIGHT + ...` | 13.09375 | 4.90625 | 8.1875 | **15x** |
///
/// Widening it "for safety" would cost that discrimination; the shipped frames
/// already sit at a quarter to a third of it.
fn tolerance(harness: &Harness<'_>) -> f32 {
    let pixels_per_point = harness.ctx.pixels_per_point();
    let tolerance = 0.5 / pixels_per_point + GUI_ROUNDING;
    assert!(
        tolerance <= TOLERANCE_CEILING,
        "the tolerance derived from this harness's {pixels_per_point} pixel(s) per point is \
         {tolerance} pt, and MC-030's AC-1 sets a ceiling of {TOLERANCE_CEILING} pt. The \
         derivation is sound only while a physical pixel is small enough to sit inside the \
         criterion; at this DPI it is not, and the criterion cannot be tested this way."
    );
    tolerance
}

// --- Reading the paint list --------------------------------------------------

/// One `Shape::Rect` the painter emitted. Only its geometry is read here;
/// colour is MC-021's.
#[derive(Clone, Copy, Debug)]
struct PaintedRect {
    rect: Rect,
}

/// One `Shape::Text` the painter emitted.
///
/// `bounds` is the galley's own rect translated to where the shape was
/// painted, which is the box the glyphs occupy on the screen. It is taken
/// from the galley rather than assembled from `pos` and a size because a
/// centred galley (the drop zone's caption) has a negative `rect.min.x`, and
/// `pos + rect` is right in both cases where `pos` and a width is not.
#[derive(Clone, Debug)]
struct PaintedText {
    pos: Pos2,
    text: String,
    bounds: Rect,
}

/// One shape, in paint order. Everything that is neither a rect nor text is
/// dropped: this story is about where two kinds of thing sit.
#[derive(Clone, Debug)]
enum Painted {
    Rect(PaintedRect),
    Text(PaintedText),
}

/// Every rect and every string the last frame painted, in paint order.
///
/// MC-021's `paint_list` (`tests/palette.rs`) with the colours dropped and the
/// galley's rect kept. `Shape::Vec` is walked because egui nests shapes: a
/// label inside a scroll area inside a panel arrives as a tree, and a shape
/// only reachable through the nesting would silently vanish from a flat read.
fn paint_list(harness: &Harness<'_>) -> Vec<Painted> {
    fn walk(shape: &egui::Shape, out: &mut Vec<Painted>) {
        match shape {
            egui::Shape::Rect(rect) => out.push(Painted::Rect(PaintedRect { rect: rect.rect })),
            egui::Shape::Text(text) => out.push(Painted::Text(PaintedText {
                pos: text.pos,
                text: text.galley.text().to_owned(),
                bounds: text.galley.rect.translate(text.pos.to_vec2()),
            })),
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

/// The one painted string equal to `wanted`. MC-021's helper, unchanged.
#[track_caller]
fn painted_text(list: &[Painted], wanted: &str) -> PaintedText {
    let mut found: Vec<PaintedText> = list
        .iter()
        .filter_map(|painted| match painted {
            Painted::Text(text) if text.text == wanted => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "exactly one painted string must read {wanted:?}, and {} do. Everything painted: \
         {list:#?}",
        found.len()
    );
    found.remove(0)
}

/// The one painted string ending in `tail`. MC-022's `row_text`, restated: the
/// reason word is the part of a row that never gives way
/// (`components.md`, "overflow"), so it is how a truncated row is named.
#[track_caller]
fn painted_text_ending_with(list: &[Painted], tail: &str) -> PaintedText {
    let mut found: Vec<PaintedText> = list
        .iter()
        .filter_map(|painted| match painted {
            Painted::Text(text) if text.text.ends_with(tail) => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "exactly one painted string must end in {tail:?}, and {} do. Everything painted: \
         {list:#?}",
        found.len()
    );
    found.remove(0)
}

/// The painted rect a point sits inside: among those containing it, the
/// smallest by area, which is the one painted on top and the one a user sees
/// there.
///
/// MC-021's helper verbatim (`tests/palette.rs`), guards included - the
/// smallest must be the unique smallest, and every candidate must contain it,
/// so "nested, innermost last" is a fact about the frame rather than an
/// assumption. Its cost, which MC-021's handoff records, is that it needs a
/// point known to be inside the shape: here that is always a caption's own
/// position.
#[track_caller]
fn innermost_rect_containing(list: &[Painted], pos: Pos2) -> Rect {
    let area = |rect: Rect| rect.width() * rect.height();
    let candidates: Vec<Rect> = list
        .iter()
        .filter_map(|painted| match painted {
            Painted::Rect(rect) if rect.rect.contains(pos) => Some(rect.rect),
            _ => None,
        })
        .collect();
    assert!(
        !candidates.is_empty(),
        "no painted rect contains {pos:?}: the shape a caption sits inside is how this suite \
         names a region, so none means the region painted no background at all. Everything \
         painted: {list:#?}"
    );

    let smallest = candidates
        .iter()
        .map(|candidate| area(*candidate))
        .fold(f32::INFINITY, f32::min);
    let innermost: Vec<Rect> = candidates
        .iter()
        .copied()
        .filter(|candidate| area(*candidate) == smallest)
        .collect();
    assert_eq!(
        innermost.len(),
        1,
        "exactly one painted rect containing {pos:?} must be the innermost, and {} are tied at \
         {smallest} sq pt: the identification is ambiguous. Candidates: {candidates:#?}",
        innermost.len()
    );
    let innermost = innermost[0];

    for candidate in &candidates {
        assert!(
            candidate.contains_rect(innermost),
            "the rects containing {pos:?} must be nested, innermost last, for \"the one on top\" \
             to be well defined; {candidate:?} overlaps {innermost:?} without containing it. \
             Candidates: {candidates:#?}"
        );
    }
    innermost
}

// --- Naming the two regions this story measures ------------------------------

/// The content column: the full inner width every region is given.
///
/// The drop zone's painted rect. `paint` reads `ui.available_width()` once,
/// inside the `space-window` margin, and hands that one number to the drop
/// zone, the folder row and the status slot; `flagged_list` is shown the same
/// `Ui`. So the drop zone's width **is** the column, and `layout.md`'s column
/// table says as much: "Every region is the full inner width."
fn content_column(list: &[Painted], caption: &str) -> Rect {
    innermost_rect_containing(list, painted_text(list, caption).pos)
}

/// The 40 pt status slot, derived from the folder button above it.
///
/// See this file's module docs for why the anchor is above the slot rather
/// than below it. The guard is what keeps the derivation honest: the button's
/// painted bottom edge is the folder row's bottom edge only because the row is
/// `size-control` tall and the button fills it, so a button that stopped
/// filling its row must say so rather than quietly shift the slot.
#[track_caller]
fn status_slot(list: &[Painted], column: Rect) -> Rect {
    let button = innermost_rect_containing(list, painted_text(list, FOLDER_BUTTON).pos);
    assert!(
        (button.height() - SIZE_CONTROL).abs() <= GUI_ROUNDING,
        "the folder button is painted {} pt tall, but `tokens.md` gives `size-control` - \"the \
         height of every interactive control and of the folder row\" - as {SIZE_CONTROL}. This \
         suite derives the status slot from the button's bottom edge *because* the two are the \
         same edge; if the button no longer fills its row, that derivation is no longer sound \
         and every gap measured below it is measured from the wrong place.",
        button.height()
    );
    let top = button.bottom() + SPACE_STACK;
    Rect::from_min_max(
        Pos2::new(column.left(), top),
        Pos2::new(column.right(), top + SIZE_STATUS),
    )
}

/// Every painted rect that lies inside the status slot.
///
/// How the progress bar is named: it is a pair of rects - the track and the
/// filled part - and it carries no text, so [`innermost_rect_containing`]
/// cannot find it (MC-021's handoff priced exactly this: "a bare progress
/// track could not, and would need a different handle"). Containment in the
/// slot is that handle, and it is a statement about the region rather than
/// about a colour or a position in the paint list.
fn rects_inside(list: &[Painted], slot: Rect) -> Vec<Rect> {
    list.iter()
        .filter_map(|painted| match painted {
            Painted::Rect(rect) if slot.contains_rect(rect.rect) => Some(rect.rect),
            _ => None,
        })
        .collect()
}

// --- The checks, stated once so a control can aim at each one ----------------
//
// Every assertion in this file goes through one of these four functions, and
// `the_placement_checks_reject_...` feeds the same four the numbers each of
// MC-030's named mutants would paint. They are the same code in both places on
// purpose: a control that exercised a look-alike check would prove nothing
// about the checks the criteria use.

/// **AC-1 and AC-2.** The painted content between `top` and `bottom` has equal
/// space above it and below it inside `slot`.
///
/// A relation between two measurements in the same frame: `top` and `bottom`
/// come from the paint list, `slot` is derived from another painted shape, and
/// the two gaps are compared against each other and never against a constant.
fn centred_problems(what: &str, slot: Rect, top: f32, bottom: f32, tolerance: f32) -> Vec<String> {
    // Finiteness as well as order: a caller that folded `min` over an empty
    // set arrives here with infinities, and infinities compare politely with
    // everything. That is how a vacuous check hides.
    if !top.is_finite() || !bottom.is_finite() || bottom <= top {
        return vec![format!(
            "{what} paints nothing to centre: its top is {top} and its bottom is {bottom}, so \
             there is no height to measure gaps from"
        )];
    }
    let above = top - slot.top();
    let below = slot.bottom() - bottom;
    if (above - below).abs() > tolerance {
        return vec![format!(
            "{what} is not centred in the {SIZE_STATUS} pt status slot: {above} pt above it and \
             {below} pt below it, a disagreement of {} pt against a tolerance of {tolerance} pt. \
             The slot runs {} to {} and the painted content {top} to {bottom} \
             (`layout.md`, the status slot: \"Content is vertically centred\")",
            (above - below).abs(),
            slot.top(),
            slot.bottom()
        )];
    }
    Vec::new()
}

/// **AC-2, second clause.** Two stacked things are `want` apart.
fn separation_problems(what: &str, gap: f32, want: f32, tolerance: f32) -> Vec<String> {
    if (gap - want).abs() > tolerance {
        return vec![format!(
            "{what} are {gap} pt apart, but `layout.md` puts them {want} pt apart (the status \
             slot: \"the bar (4 px, full width) sits above the count label (`small`), 4 px \
             apart\"), within {tolerance} pt"
        )];
    }
    Vec::new()
}

/// **AC-2, the block's identity.** The rects the slot paints are the pieces of
/// one progress bar: there is at least one, and they share one vertical extent.
///
/// The empty guard is not decoration. Without it [`progress_block_problems`]
/// folds `min` and `max` over an empty set - the vacuous-check defect MC-029
/// caught in itself. It is a check of its own rather than a branch inside that
/// function so that the control which pins it can call **it** and nothing else:
/// a control aimed at the whole block goes on passing with this guard deleted,
/// because folding over no rects yields infinities that the separation check
/// then objects to. That is a pass for the wrong reason, and it was caught here
/// by probing the guard rather than by reading it (MC-030, `## Regressions`).
fn bar_problems(bar: &[Rect]) -> Vec<String> {
    if bar.is_empty() {
        return vec![
            "the status slot paints no rect of its own while a run is under way, so there is no \
             progress bar to centre: `layout.md` gives the slot \"the bar (4 px, full width)\" \
             above the count"
                .to_owned(),
        ];
    }
    let top = bar.iter().map(Rect::top).fold(f32::INFINITY, f32::min);
    let bottom = bar
        .iter()
        .map(Rect::bottom)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut problems = Vec::new();
    for piece in bar {
        if (piece.top() - top).abs() > GUI_ROUNDING
            || (piece.bottom() - bottom).abs() > GUI_ROUNDING
        {
            problems.push(format!(
                "the {} rect(s) the status slot paints must all be pieces of one bar sharing one \
                 vertical extent, and {piece:?} runs {} to {} where the block as a whole runs \
                 {top} to {bottom}: the shape being measured is not the progress bar",
                bar.len(),
                piece.top(),
                piece.bottom()
            ));
        }
    }
    problems
}

/// **AC-2, both clauses.** The bar and the count are one block, centred in the
/// slot, a `space-tight` apart.
fn progress_block_problems(slot: Rect, bar: &[Rect], count: Rect, tolerance: f32) -> Vec<String> {
    let mut problems = bar_problems(bar);
    if !problems.is_empty() {
        return problems;
    }
    let top = bar.iter().map(Rect::top).fold(f32::INFINITY, f32::min);
    let bottom = bar
        .iter()
        .map(Rect::bottom)
        .fold(f32::NEG_INFINITY, f32::max);
    problems.extend(separation_problems(
        "the progress bar and the count beneath it",
        count.top() - bottom,
        SPACE_TIGHT,
        tolerance,
    ));
    problems.extend(centred_problems(
        "the progress bar and the count beneath it, taken as one block",
        slot,
        top,
        count.bottom(),
        tolerance,
    ));
    problems
}

/// **AC-3.** The whole painted row is no wider than the column the list is
/// given, and sits inside it.
///
/// A mechanical inequality, so no tolerance: the painter chooses the name's
/// length so that the row fits, and "fits" is not a measurement that rounding
/// can flip - it either asked for less than it had or it did not.
///
/// **What `column` is, and why it is sound.** The width the *list* gives a row
/// is not in the paint list: a scroll area's inner width is its own business,
/// and the row's own allocated box is no use because a row too wide for it
/// **grows it** (`allocate_ui_with_layout` allocates `max(min_size, content)`),
/// so a mutant that overflows the list would quietly widen the very bound it
/// was being judged against. The content column cannot move that way. A scroll
/// bar can only make the list *narrower* than the column, never wider, so
/// `row <= column` is implied by the criterion and never implies it: this is a
/// necessary condition, deliberately the conservative half. In the shipped
/// frame the list's own painted box is exactly as wide as the column - 472 pt,
/// no bar shaved - so today the two coincide.
fn fits_column_problems(row: Rect, column: Rect) -> Vec<String> {
    let mut problems = Vec::new();
    if row.width() > column.width() {
        problems.push(format!(
            "the painted row is {} pt wide and the list is given a column {} pt wide, so it is \
             {} pt too wide for the space it has (`components.md`, \"overflow\": the file name \
             is what gives way)",
            row.width(),
            column.width(),
            row.width() - column.width()
        ));
    }
    if row.right() > column.right() || row.left() < column.left() {
        problems.push(format!(
            "the painted row runs {} to {} and the column runs {} to {}, so it spills out of the \
             list",
            row.left(),
            row.right(),
            column.left(),
            column.right()
        ));
    }
    problems
}

/// **The instrument.** The slot derived from the folder button above ends
/// exactly `space-stack` above the flagged list's own painted box.
fn slot_extent_problems(slot: Rect, list_top: f32, tolerance: f32) -> Vec<String> {
    let gap = list_top - slot.bottom();
    if (gap - SPACE_STACK).abs() > tolerance {
        return vec![format!(
            "the status slot derived from the folder button above it ends at {}, and the flagged \
             list's own painted box begins at {list_top} - a gap of {gap} pt where \
             `space-stack` is {SPACE_STACK}. Either the slot is not {SIZE_STATUS} pt tall, or \
             the regions are not `space-stack` apart; either way the slot these tests measure \
             gaps in is not where the window's own layout puts it",
            slot.bottom()
        )];
    }
    Vec::new()
}

/// Panic with the checks' own messages, which read as the bug report.
#[track_caller]
fn report(problems: Vec<String>) {
    assert!(
        problems.is_empty(),
        "the window paints {} thing(s) the design does not say:\n  - {}",
        problems.len(),
        problems.join("\n  - ")
    );
}

// --- AC-1: the result line is centred in the slot -----------------------------

#[test]
fn a_finished_runs_result_line_has_equal_space_above_and_below_it_in_the_status_slot() {
    let model = model(
        AppState::Done {
            summary: one_flagged("b.gif"),
        },
        Some(OUT_DIR),
    );
    let harness = painted(&model);
    let list = paint_list(&harness);
    let column = content_column(&list, DROPZONE_READY);
    let slot = status_slot(&list, column);
    let line = painted_text(&list, RESULT_LINE).bounds;

    report(centred_problems(
        "the result line",
        slot,
        line.top(),
        line.bottom(),
        tolerance(&harness),
    ));
}

// --- AC-2: the bar and the count are centred as one block --------------------

#[test]
fn a_running_jobs_progress_bar_and_count_are_centred_as_one_block_a_space_tight_apart() {
    let model = model(AppState::Processing { done: 1, total: 4 }, Some(OUT_DIR));
    let harness = painted(&model);
    let list = paint_list(&harness);
    let column = content_column(&list, DROPZONE_BUSY);
    let slot = status_slot(&list, column);
    let bar = rects_inside(&list, slot);
    let count = painted_text(&list, PROGRESS_COUNT).bounds;

    report(progress_block_problems(
        slot,
        &bar,
        count,
        tolerance(&harness),
    ));
}

// --- AC-3: a row too long for the list is no wider than the list -------------

#[test]
fn a_flagged_row_too_long_for_the_list_is_painted_no_wider_than_the_column_it_is_given() {
    let model = model(
        AppState::Done {
            summary: one_flagged(LONG_NAME),
        },
        Some(OUT_DIR),
    );
    let harness = painted(&model);
    let list = paint_list(&harness);
    let column = content_column(&list, DROPZONE_READY);
    let row = painted_text_ending_with(&list, BLANK_TAIL);

    // AC-3's "given" - a row whose file name is too long for the list. Without
    // it a painter that showed the name untouched, or a name that had quietly
    // become short enough to fit, would satisfy the inequality below having
    // been asked nothing.
    let whole = format!("{LONG_NAME}{BLANK_TAIL}");
    assert_ne!(
        row.text, whole,
        "AC-3 is about a row whose file name is too long for the list, and this one was painted \
         whole, so nothing was asked of the budget. The row painted was {:?}",
        row.text
    );

    report(fits_column_problems(row.bounds, column));
}

// --- The instrument: the slot is where both its neighbours put it ------------

#[test]
fn the_status_slot_these_gaps_are_measured_in_is_where_both_its_neighbours_put_it() {
    let model = model(
        AppState::Done {
            summary: one_flagged("b.gif"),
        },
        Some(OUT_DIR),
    );
    let harness = painted(&model);
    let list = paint_list(&harness);
    let column = content_column(&list, DROPZONE_READY);
    let slot = status_slot(&list, column);
    // The flagged list's own painted box, named by the row that sits in it.
    let row = painted_text_ending_with(&list, BLANK_TAIL);
    let list_box = innermost_rect_containing(&list, row.pos);

    report(slot_extent_problems(
        slot,
        list_box.top(),
        tolerance(&harness),
    ));
}

// --- The controls ------------------------------------------------------------
//
// Everything above passes on the first run, because MC-030 adds tests to a
// painter that already works. The four checks are therefore fed, here, the
// numbers each defect they exist to catch would produce. Without this the
// suite could assert nothing at all and nothing would notice; with it, a check
// that has stopped checking fails here first.
//
// The mutations themselves are AC-4's, and applying them to `crates/app/src`
// is GREEN's `## Gate probes`: the phase lock freezes the source in RED, and
// the numbers below are the arithmetic those mutations produce, recorded in
// MC-030's handoff so GREEN can confirm each one against the shipped painter.

/// The slot these controls place content in: 40 pt tall, in a 472 pt column,
/// at an origin that is deliberately not the window's - none of the checks
/// reads an absolute coordinate, and a control that used the real one would
/// hide it if one ever did.
fn control_slot() -> Rect {
    Rect::from_min_max(
        Pos2::new(24.0, 1000.0),
        Pos2::new(496.0, 1000.0 + SIZE_STATUS),
    )
}

/// The tolerance the shipped frames are judged by, spelled out so the controls
/// are measured against the same number the criteria are.
const CONTROL_TOLERANCE: f32 = 0.5 + GUI_ROUNDING;

#[test]
fn the_centring_check_rejects_a_flush_line_a_mutated_block_and_that_blocks_mirror() {
    let slot = control_slot();
    // The painted height of the `body` result line and of the `small` count,
    // measured in RED from the shipped frames.
    let line = 16.0_f32;
    let count = 14.0_f32;

    // AC-1's control, in the criterion's own words: "a line painted flush to
    // the top of the slot, which is what `(SIZE_STATUS - content) / 2.0`
    // becoming `* 2.0` or `% 2.0` produces".
    let flush = centred_problems(
        "a line flush to the top",
        slot,
        slot.top(),
        slot.top() + line,
        CONTROL_TOLERANCE,
    );
    assert!(
        !flush.is_empty(),
        "a result line painted flush to the top of the slot leaves {} pt below it and none \
         above, and the centring check must say so",
        SIZE_STATUS - line
    );

    // Its mirror, flush to the bottom: the check must be about the two gaps
    // disagreeing and not about one of them being small.
    let flush_bottom = centred_problems(
        "a line flush to the bottom",
        slot,
        slot.bottom() - line,
        slot.bottom(),
        CONTROL_TOLERANCE,
    );
    assert!(
        !flush_bottom.is_empty(),
        "a result line painted flush to the bottom of the slot is as wrong as one flush to the \
         top, and a check that caught only the first would pass on half the defect"
    );

    // AC-1's named mutation, `(SIZE_STATUS + content) / 2.0`: the line is
    // placed (40 + 16.125) / 2 = 28.0625 down a 40 pt slot and runs 16 pt, so
    // it ends 4.0625 pt *below* the slot.
    let mutant_top = slot.top() + 28.0625;
    let inflated = centred_problems(
        "the `(SIZE_STATUS + content) / 2.0` line",
        slot,
        mutant_top,
        mutant_top + line,
        CONTROL_TOLERANCE,
    );
    assert!(
        !inflated.is_empty(),
        "`(SIZE_STATUS - content) / 2.0` becoming `+` puts the result line 28.0625 pt down a 40 \
         pt slot - 28.0625 above, -4.0625 below - and AC-4 requires at least one assertion in \
         this file to fail on it"
    );

    // AC-2's named mutation, `SIZE_PROGRESS - SPACE_TIGHT + line`: the block
    // is placed (40 - 13.8125) / 2 = 13.09375 down and runs 4 + 4 + 14 = 22 pt.
    let mutant_top = slot.top() + 13.09375;
    let mismeasured = centred_problems(
        "the `SIZE_PROGRESS - SPACE_TIGHT` block",
        slot,
        mutant_top,
        mutant_top + SPACE_TIGHT + SPACE_TIGHT + count,
        CONTROL_TOLERANCE,
    );
    assert!(
        !mismeasured.is_empty(),
        "the first `+` of `SIZE_PROGRESS + SPACE_TIGHT + ...` becoming `-` centres the block as \
         though it were 13.8125 pt tall when it paints 22 - 13.09375 above, 4.90625 below - and \
         AC-4 requires at least one assertion in this file to fail on it"
    );

    // AC-2's stated control, "swap the two gaps". Swapping is a no-op against
    // a check on `|above - below|`, so it is read as the mutant's mirror: the
    // same two gaps the other way up must be rejected too, or the check would
    // be about which gap is larger rather than about them being equal.
    let swapped_top = slot.top() + 4.90625;
    let swapped = centred_problems(
        "the `SIZE_PROGRESS - SPACE_TIGHT` block with its gaps swapped",
        slot,
        swapped_top,
        swapped_top + SPACE_TIGHT + SPACE_TIGHT + count,
        CONTROL_TOLERANCE,
    );
    assert!(
        !swapped.is_empty(),
        "AC-2's control is \"swap the two gaps and the assertion must fail\": a block 4.90625 pt \
         from the top and 13.09375 from the bottom is as far from centred as the mutant it \
         mirrors"
    );

    // And the check must accept what the shipped painter paints, or it would
    // be rejecting everything and proving nothing. These are the measured
    // numbers, not recomputed ones.
    assert!(
        centred_problems(
            "the shipped result line",
            slot,
            slot.top() + 11.9375,
            slot.top() + 11.9375 + line,
            CONTROL_TOLERANCE,
        )
        .is_empty(),
        "the shipped result line sits 11.9375 pt from the top and 12.0625 from the bottom, a \
         disagreement of 0.125 pt, and the check must accept it"
    );
    assert!(
        centred_problems(
            "the shipped progress block",
            slot,
            slot.top() + 9.09375,
            slot.top() + 9.09375 + SPACE_TIGHT + SPACE_TIGHT + count,
            CONTROL_TOLERANCE,
        )
        .is_empty(),
        "the shipped progress block sits 9.09375 pt from the top and 8.90625 from the bottom, a \
         disagreement of 0.1875 pt, and the check must accept it"
    );
}

#[test]
fn the_progress_block_check_rejects_a_missing_bar_a_ragged_bar_and_a_gap_that_is_not_space_tight() {
    let slot = control_slot();
    let bar = vec![
        Rect::from_min_max(
            Pos2::new(24.0, slot.top() + 9.09375),
            Pos2::new(496.0, slot.top() + 13.09375),
        ),
        Rect::from_min_max(
            Pos2::new(24.0, slot.top() + 9.09375),
            Pos2::new(142.0, slot.top() + 13.09375),
        ),
    ];
    let count = |top: f32| Rect::from_min_max(Pos2::new(24.0, top), Pos2::new(54.5, top + 14.0));

    // The shipped arrangement, accepted: a check that rejected everything
    // would satisfy every control below and mean nothing.
    assert!(
        bar_problems(&bar).is_empty(),
        "the shipped bar is two rects sharing one 4 pt vertical extent - the track and the \
         filled part - and must be recognised as one bar"
    );
    assert!(
        progress_block_problems(slot, &bar, count(slot.top() + 17.09375), CONTROL_TOLERANCE)
            .is_empty(),
        "the shipped bar and count - 9.09375 pt down, 4 pt apart, 22 pt of block in a 40 pt slot \
         - must be accepted"
    );

    // A slot that paints no bar at all. AC-2 is universally quantified over
    // the block's pieces, and over none of them it is true having measured
    // nothing; MC-029 shipped exactly that defect and caught it in review.
    //
    // Aimed at `bar_problems` and not at the whole block on purpose. The first
    // draft of this control called `progress_block_problems` and **passed with
    // the empty guard deleted**, because folding `min` over no rects gives an
    // infinity that the separation check then objects to: a pass, for a reason
    // that has nothing to do with the bar being missing. See `## Regressions`.
    assert!(
        !bar_problems(&[]).is_empty(),
        "a status slot that paints no progress bar at all must fail AC-2 rather than pass it \
         vacuously: with no bar there is no block to centre, and a check that reported success \
         there would report success on a bar deleted outright"
    );
    assert!(
        !progress_block_problems(slot, &[], count(slot.top() + 17.09375), CONTROL_TOLERANCE)
            .is_empty(),
        "and the block check must carry that verdict out rather than fold over an empty set"
    );

    // Two rects that are not one bar, aimed at `bar_problems` for the same
    // reason: judged as a whole block this arrangement is also mis-separated,
    // so the composite check would reject it whether or not it noticed the
    // rects disagree.
    let ragged = vec![
        bar[0],
        Rect::from_min_max(
            Pos2::new(24.0, slot.top() + 20.0),
            Pos2::new(142.0, slot.top() + 24.0),
        ),
    ];
    assert!(
        !bar_problems(&ragged).is_empty(),
        "two rects at different heights are not the two pieces of one progress bar, and the \
         check must refuse to take the topmost of them as the block's top edge"
    );

    // `space-tight` replaced by `space-stack`, the neighbouring token. Aimed at
    // `separation_problems`: moving the count 4 pt down also moves the block's
    // bottom edge, so the centring check objects too and a control on the
    // composite would pass with the separation check deleted.
    assert!(
        separation_problems("the control", SPACE_TIGHT, SPACE_TIGHT, CONTROL_TOLERANCE).is_empty(),
        "a gap that is exactly `space-tight` must be accepted"
    );
    assert!(
        !separation_problems("the control", SPACE_STACK, SPACE_TIGHT, CONTROL_TOLERANCE).is_empty(),
        "the bar and the count 8 pt apart instead of 4 must be rejected: `layout.md` puts them 4 \
         px apart, and `space-stack` is the token next to `space-tight` on the ramp"
    );
    assert!(
        !separation_problems("the control", 0.0, SPACE_TIGHT, CONTROL_TOLERANCE).is_empty(),
        "a count painted flush against the bar has lost `space-tight` entirely"
    );
    assert!(
        !progress_block_problems(slot, &bar, count(slot.top() + 21.09375), CONTROL_TOLERANCE)
            .is_empty(),
        "and the block check must carry that verdict out for a count 8 pt below the bar"
    );
}

#[test]
fn the_width_check_rejects_a_row_wider_than_its_column_and_accepts_the_shipped_one() {
    let column = Rect::from_min_max(Pos2::new(24.0, 240.0), Pos2::new(496.0, 264.0));

    // The shipped frame: a truncated row 469 pt wide in a 472 pt column,
    // 3 pt to spare. Measured in RED, not recomputed.
    let shipped = Rect::from_min_max(Pos2::new(24.0, 244.0), Pos2::new(493.0, 260.0));
    assert!(
        fits_column_problems(shipped, column).is_empty(),
        "the shipped row is 469 pt wide in a 472 pt column and must be accepted; a check that \
         rejected it would make every control below meaningless"
    );

    // AC-3's mutation, `available_width() + text_width(tail)`: the budget is
    // inflated by the tail's own 54.75 pt, the name keeps about 109 pt more,
    // and the row is painted 578.1875 pt wide - 106.1875 pt past the column.
    let overflowing = Rect::from_min_max(Pos2::new(24.0, 244.0), Pos2::new(602.1875, 260.0));
    let problems = fits_column_problems(overflowing, column);
    assert_eq!(
        problems.len(),
        2,
        "a row painted 578.1875 pt wide in a 472 pt column is both too wide and spilling out of \
         it, and the check must say both. It said: {problems:#?}"
    );

    // A row the same width as its column is not too wide: the criterion is
    // "no wider than", and a check that read it as "narrower than" would fail
    // the day a name happened to fill the row exactly.
    let exact = Rect::from_min_max(Pos2::new(24.0, 244.0), Pos2::new(496.0, 260.0));
    assert!(
        fits_column_problems(exact, column).is_empty(),
        "AC-3 says \"no wider than the width the list gives it\", so a row exactly as wide as \
         its column fits"
    );
}

#[test]
fn the_slot_extent_check_rejects_a_slot_that_does_not_end_a_space_stack_above_the_list() {
    let slot = control_slot();

    assert!(
        slot_extent_problems(slot, slot.bottom() + SPACE_STACK, CONTROL_TOLERANCE).is_empty(),
        "a list beginning exactly `space-stack` below the slot is what `layout.md`'s column says, \
         and must be accepted"
    );

    // What the `(SIZE_STATUS + content) / 2.0` mutant does to the frame as a
    // whole: the line overflows the 40 pt slot, `allocate_ui_with_layout`
    // allocates `max(min_size, content)` = 44.0625 pt, and everything below is
    // pushed down by 4.0625 pt.
    assert!(
        !slot_extent_problems(
            slot,
            slot.bottom() + SPACE_STACK + 4.0625,
            CONTROL_TOLERANCE
        )
        .is_empty(),
        "a status slot that has grown past `size-status` pushes the flagged list down, and the \
         instrument that derives the slot from the folder button above must notice rather than \
         keep measuring gaps in a slot that is no longer there"
    );

    assert!(
        !slot_extent_problems(slot, slot.bottom(), CONTROL_TOLERANCE).is_empty(),
        "a list beginning flush against the slot has lost `space-stack` entirely"
    );
}
