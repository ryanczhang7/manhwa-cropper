//! MC-021: the colours the window actually *paints*.
//!
//! # The surface these tests read
//!
//! `egui_kittest`'s `Harness::output()` hands back the `egui::FullOutput` of
//! the last frame, and its `shapes` are the **untessellated** paint list. MC-022
//! read `Shape::Text`'s galley out of it to pin what the window *says*
//! (`crates/app/tests/truncation.rs`); the same list carries `Shape::Rect` with
//! `fill` and `stroke` as values, and every `Shape::Text` carries its galley's
//! per-section `format.color` and `format.font_id`. So the colour a user would
//! see is readable as a number, with no renderer, no image and no pixel
//! comparison.
//!
//! That is the gap `docs/wiki/audits/app-window-2026-09-13.md`, evidence E-3,
//! left open. `folder_button` calls `add_enabled(enabled, ..)`, which is the
//! part AccessKit can see, and then paints a colour that nothing reads back:
//! `delete !` in `folder_button` and `widgets -> Widgets::default()` both leave
//! all 26 assertions in `tests/gui.rs` and `tests/shell.rs` green while turning
//! the window a different colour.
//!
//! # Why this is a separate file from `tests/gui.rs`
//!
//! `tests/gui.rs` states its own contract in its first paragraph: "Every
//! assertion here is a query against that tree - never a pixel, never a rect,
//! never a colour of a painted shape." Every assertion *here* is a colour of a
//! painted shape, so they do not belong under that sentence. MC-022 made the
//! same call and wrote `tests/truncation.rs`; nothing in `tests/gui.rs` is
//! changed by this story.
//!
//! # Why every colour below is a literal
//!
//! The expected values are copied out of `docs/wiki/design/tokens.md`'s
//! "Colour roles" table and `components.md`'s component tables, never read back
//! from `gui.rs`'s own `LIGHT` / `DARK` constants - which are private, and
//! which would make every assertion here say only that the painter agrees with
//! itself. `tests/gui.rs` states the same convention in its module docs and
//! spells `SURFACE_LIGHT` out as a literal. The two files meet at `tokens.md`
//! and nowhere else.
//!
//! # What is deliberately not here
//!
//! No geometry. Positions, sizes, radii and centring are MC-029 and MC-030
//! (`## Out of scope`); a rect's position is used below to *identify* the
//! button, never asserted. No hover or pressed state: a headless frame has no
//! pointer. No font metric: a colour is a value and does not move when a font
//! package updates, and the only font assertion here is the `body` `FontId`,
//! which is a declared style rather than a measurement.

use std::ops::Range;
use std::path::PathBuf;

use cropper_core::FlagReason;
use cropper_engine::batch::RunSummary;
use cropper_engine::process::{FileResult, Flag, Outcome};
use eframe::egui;
use eframe::egui::{Color32, FontFamily, FontId, Pos2};
use egui_kittest::Harness;
use manhwa_cropper::{AppState, Model, gui};

// --- The frozen colours (docs/wiki/design/tokens.md, "Colour roles") ---------
//
// Hex, fully opaque, character for character out of the table's Light and Dark
// columns. Only the roles this story's criteria name are here.

/// `text`, dark column: all text unless a role says otherwise.
const TEXT_DARK: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF);
/// `text`, light column.
const TEXT_LIGHT: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);

/// `text-muted`, dark column: the flag reason word, among others.
const TEXT_MUTED_DARK: Color32 = Color32::from_rgb(0xC5, 0xC5, 0xC5);

/// `text-disabled`, dark column: text of a disabled control.
const TEXT_DISABLED_DARK: Color32 = Color32::from_rgb(0x6D, 0x6D, 0x6D);

/// `border`, dark column: "the disabled button's boundary; never the boundary
/// of an enabled control".
const BORDER_DARK: Color32 = Color32::from_rgb(0x3F, 0x3F, 0x3F);

/// `border-strong`, dark column: boundary of every enabled control.
const BORDER_STRONG_DARK: Color32 = Color32::from_rgb(0x9E, 0x9E, 0x9E);
/// `border-strong`, light column.
const BORDER_STRONG_LIGHT: Color32 = Color32::from_rgb(0x76, 0x76, 0x76);

/// `control`, dark column: button at rest.
const CONTROL_DARK: Color32 = Color32::from_rgb(0x2D, 0x2D, 0x2D);
/// `control`, light column.
const CONTROL_LIGHT: Color32 = Color32::from_rgb(0xFB, 0xFB, 0xFB);

/// `control-disabled`, dark column: button fill when disabled.
const CONTROL_DISABLED_DARK: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x2A);

/// `stroke-control` (`tokens.md`, "Radii, borders, elevation"): every control
/// boundary in this window is 1 px, enabled or disabled.
const STROKE_CONTROL: f32 = 1.0;

/// The `body` step of `tokens.md`'s type scale: 14 pt, egui's proportional
/// family. `components.md` gives the flagged row `body`.
fn body_font() -> FontId {
    FontId::new(14.0, FontFamily::Proportional)
}

// --- What egui paints if our palette is discarded ----------------------------
//
// AC-3's second clause. `Widgets::default()` is `Widgets::dark()`
// (`egui-0.36.2/src/style.rs:1772`), whose `inactive` slot - the one a button
// at rest reads - is `weak_bg_fill: Color32::from_gray(60)` and
// `bg_stroke: Default::default()`, i.e. a zero-width transparent stroke
// (`style.rs:1691`). Those are the values the audit's `269` survivor
// (`replace widgets -> Widgets with Default::default()`) would put on screen.
// They are spelled out here so a reader can see for themselves that they are
// not `control` (#2D2D2D) and not `border-strong` (#9E9E9E): without that, the
// criterion would be about any palette rather than about ours.

/// `Widgets::default().inactive.weak_bg_fill` - `Color32::from_gray(60)`.
const EGUI_DEFAULT_BUTTON_FILL: Color32 = Color32::from_rgb(60, 60, 60);
/// `Widgets::default().inactive.bg_stroke.color` - `Stroke::default()`.
const EGUI_DEFAULT_BUTTON_BOUNDARY: Color32 = Color32::TRANSPARENT;
/// `Widgets::default().inactive.bg_stroke.width` - `Stroke::default()`.
const EGUI_DEFAULT_BUTTON_BOUNDARY_WIDTH: f32 = 0.0;

// --- The frozen strings (docs/wiki/design/voice.md) --------------------------

/// `folder.button`. One U+2026, never three dots.
const FOLDER_BUTTON: &str = "Choose folder…";

/// A short output path that fits the folder row at the default window size, so
/// nothing here depends on truncation (which is MC-022's story, not this one).
const OUT_DIR: &str = r"D:\shots\out";

/// The one flagged row AC-4 reads, as `list.row` builds it.
const FLAGGED_ROW: &str = "b.gif — Blank";
/// `reason.uniform`: the tail of [`FLAGGED_ROW`] that carries `text-muted`.
const FLAGGED_REASON: &str = "Blank";
/// The file name at the head of [`FLAGGED_ROW`], which carries `text`.
const FLAGGED_NAME: &str = "b.gif";

/// Default inner size, logical px (`layout.md`). The same window every other
/// test in this crate paints.
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
/// palette: without it the frame is painted from egui's own `Visuals` and every
/// colour below would be measuring egui rather than `tokens.md`. It is
/// idempotent and registers *both* themes, which is what lets [`look_under`]
/// switch between them.
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

/// A finished run with exactly one flagged row, for AC-4.
fn one_flagged_row() -> RunSummary {
    RunSummary {
        results: vec![FileResult {
            input: PathBuf::from(format!(r"C:\Users\ryan\Downloads\{FLAGGED_NAME}")),
            outcome: Outcome::Flagged {
                reason: Flag::Detector(FlagReason::Uniform),
                output: PathBuf::from(OUT_DIR).join(FLAGGED_NAME),
            },
        }],
    }
}

// --- Reading the paint list --------------------------------------------------

/// One `Shape::Rect` the painter emitted: the two values a fill and a boundary
/// are made of, and the rect itself - used only to say *which* shape this is.
#[derive(Clone, Debug)]
struct PaintedRect {
    rect: egui::Rect,
    fill: Color32,
    boundary: Color32,
    boundary_width: f32,
}

/// One section of a painted galley: the bytes it covers, and how it looks.
#[derive(Clone, Debug)]
struct PaintedSection {
    bytes: Range<usize>,
    colour: Color32,
    font: FontId,
}

/// One `Shape::Text` the painter emitted.
#[derive(Clone, Debug)]
struct PaintedText {
    pos: Pos2,
    text: String,
    sections: Vec<PaintedSection>,
}

/// One shape, in paint order. Everything that is neither a rect nor text is
/// dropped: this story is about fills, boundaries and text colour.
#[derive(Clone, Debug)]
enum Painted {
    Rect(PaintedRect),
    Text(PaintedText),
}

/// Every rect and every string the last frame painted, in paint order.
///
/// `Shape::Vec` is walked because egui nests shapes: a button inside a frame
/// inside a panel arrives as a tree, and a shape only reachable through the
/// nesting would silently vanish from a flat read. Same walk MC-022 wrote in
/// `truncation.rs`'s `painted_texts`, widened from text to rects.
///
/// A `TextShape`'s `override_text_color`, when set, is what the tessellator
/// paints *instead of* every section colour, so it is folded in here: this
/// function answers "what colour did the user see", not "what is in the job".
fn paint_list(harness: &Harness<'_>) -> Vec<Painted> {
    fn walk(shape: &egui::Shape, out: &mut Vec<Painted>) {
        match shape {
            egui::Shape::Rect(rect) => out.push(Painted::Rect(PaintedRect {
                rect: rect.rect,
                fill: rect.fill,
                boundary: rect.stroke.color,
                boundary_width: rect.stroke.width,
            })),
            egui::Shape::Text(text) => out.push(Painted::Text(PaintedText {
                pos: text.pos,
                text: text.galley.text().to_owned(),
                sections: text
                    .galley
                    .job
                    .sections
                    .iter()
                    .map(|section| PaintedSection {
                        bytes: section.byte_range.start.0..section.byte_range.end.0,
                        colour: text.override_text_color.unwrap_or(section.format.color),
                        font: section.format.font_id.clone(),
                    })
                    .collect(),
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

/// The one painted string equal to `wanted`.
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
        "exactly one painted string must read {wanted:?}, and {} do. Everything painted: {list:#?}",
        found.len()
    );
    found.remove(0)
}

/// The innermost painted rect containing `pos`: the one a user would see under
/// that point.
///
/// **This is how the button's rect is told apart from every other rect in the
/// frame**, and it is deliberately semantic: the button is the rect its own
/// caption is painted inside. An index into the paint list would pin paint
/// order and break on any refactor that is not a defect (`MC-021`, "## Model
/// guidance"). MC-029 (the drop zone's border) and MC-030 (the centring
/// arithmetic) want the same helper; nothing here is specific to the button.
///
/// "Contains" alone is not enough, and that is the cost. A frame is nested: the
/// panel background `[[8,8]-[512,432]]` is painted under everything and
/// contains every caption in the window, so the idle frame has **three** rects
/// containing the button's caption. The innermost - smallest by area - is the
/// one painted last and on top, and area is order-free in a way "the last one
/// in the list" is not.
///
/// Two guards keep that from silently answering the wrong question: the
/// smallest must be the *unique* smallest, and every other candidate must
/// contain it. A control that painted no background of its own would leave the
/// two equal window-sized rects tied for smallest and fail the first, rather
/// than quietly reporting the panel's fill as the control's.
///
/// The other cost is that this needs a point known to be inside the shape,
/// which for a control means it must paint text. A control with no caption
/// cannot be found this way; the drop zone, whose text sits inside it, can.
#[track_caller]
fn innermost_rect_containing(list: &[Painted], pos: Pos2) -> PaintedRect {
    let area = |rect: egui::Rect| rect.width() * rect.height();
    let candidates: Vec<PaintedRect> = list
        .iter()
        .filter_map(|painted| match painted {
            Painted::Rect(rect) if rect.rect.contains(pos) => Some(rect.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !candidates.is_empty(),
        "no painted rect contains {pos:?}: the shape a caption sits inside is how this suite \
         names a control, so none means the control painted no background at all. Everything \
         painted: {list:#?}"
    );

    let smallest = candidates
        .iter()
        .map(|candidate| area(candidate.rect))
        .fold(f32::INFINITY, f32::min);
    let mut innermost: Vec<PaintedRect> = candidates
        .iter()
        .filter(|candidate| area(candidate.rect) == smallest)
        .cloned()
        .collect();
    assert_eq!(
        innermost.len(),
        1,
        "exactly one painted rect containing {pos:?} must be the innermost, and {} are tied at \
         {smallest} sq pt: the identification is ambiguous, which is what it looks like when the \
         control painted no background of its own. Candidates: {candidates:#?}",
        innermost.len()
    );
    let innermost = innermost.remove(0);

    for candidate in &candidates {
        assert!(
            candidate.rect.contains_rect(innermost.rect),
            "the rects containing {pos:?} must be nested, innermost last, for \"the one on top\" \
             to be well defined; {:?} overlaps {:?} without containing it. Candidates: \
             {candidates:#?}",
            candidate.rect,
            innermost.rect
        );
    }
    innermost
}

/// The section of `text` covering byte `at`.
///
/// By byte offset rather than by index, because `LayoutJob::append` merges a
/// run into the previous section when the format is identical
/// (`epaint-0.36.2/src/text/text_layout_types.rs:207`): `flagged_row` appends
/// three runs and the galley carries two sections, since the separator is in
/// the same `text` as the name. An index would be asserting that merge.
#[track_caller]
fn section_at(text: &PaintedText, at: usize) -> PaintedSection {
    text.sections
        .iter()
        .find(|section| section.bytes.contains(&at))
        .cloned()
        .unwrap_or_else(|| {
            panic!(
                "no section of the painted string {:?} covers byte {at}; its sections are {:#?}",
                text.text, text.sections
            )
        })
}

// --- The look of the one button this window has ------------------------------

/// The three values `components.md`'s button table gives a state: fill,
/// boundary and text. Compared as a whole in AC-5, where "the two themes must
/// differ" is a statement about the look and not about any one colour.
#[derive(Clone, Debug, PartialEq)]
struct ButtonLook {
    fill: Color32,
    boundary: Color32,
    boundary_width: f32,
    caption: Color32,
}

/// What the "Choose folder…" button looks like in the last painted frame.
#[track_caller]
fn button_look(harness: &Harness<'_>) -> ButtonLook {
    let list = paint_list(harness);
    let caption = painted_text(&list, FOLDER_BUTTON);
    assert_eq!(
        caption.sections.len(),
        1,
        "the button's caption is one run of text in one style, so it has exactly one section; \
         got {:#?}",
        caption.sections
    );
    let rect = innermost_rect_containing(&list, caption.pos);
    ButtonLook {
        fill: rect.fill,
        boundary: rect.boundary,
        boundary_width: rect.boundary_width,
        caption: caption.sections[0].colour,
    }
}

/// The button's look with the system theme set to `theme`.
///
/// `HarnessBuilder` forces a fixed preference - `Harness::from_builder` calls
/// `ctx.set_theme(Theme::Dark)` - so this puts `ThemePreference::System` back,
/// standing in for eframe's default, which the harness does not reproduce.
/// Same recipe as `tests/gui.rs`'s `the_window_follows_the_system_theme`, and
/// it belongs to the test: the design note forbids the painter from touching
/// `theme_preference`.
#[track_caller]
fn look_under(model: &Model, theme: egui::Theme) -> ButtonLook {
    let mut harness = painted(model);
    harness.ctx.set_theme(egui::ThemePreference::System);
    harness.input_mut().system_theme = Some(theme);
    harness.run();
    assert_eq!(
        harness.ctx.global_style().visuals.dark_mode,
        theme == egui::Theme::Dark,
        "the frame read below must actually have been painted under {theme:?}; if the harness \
         ignored `system_theme` then both of AC-5's frames are the same frame and the criterion \
         proves nothing"
    );
    button_look(&harness)
}

// --- Reporting -------------------------------------------------------------
//
// Every criterion below gathers its disagreements and reports them together,
// so one run says everything that is wrong with the look rather than the first
// thing. AC-2 in particular expects more than one.

fn expect_colour(problems: &mut Vec<String>, what: &str, got: Color32, want: Color32, role: &str) {
    if got != want {
        problems.push(format!(
            "{what} is painted {got:?}, but `{role}` is {want:?} (`tokens.md`, \"Colour roles\")"
        ));
    }
}

fn expect_width(problems: &mut Vec<String>, what: &str, got: f32, want: f32) {
    if got != want {
        problems.push(format!(
            "{what} is {got} px wide, but `stroke-control` is {want} px (`tokens.md`, \"Radii, \
             borders, elevation\")"
        ));
    }
}

fn expect_font(problems: &mut Vec<String>, what: &str, got: &FontId, want: &FontId) {
    if got != want {
        problems.push(format!(
            "{what} is painted in {got:?}, but the `body` step is {want:?} (`tokens.md`, \"Type \
             scale\")"
        ));
    }
}

#[track_caller]
fn report(problems: Vec<String>) {
    assert!(
        problems.is_empty(),
        "the window paints {} thing(s) the design does not say:\n  - {}",
        problems.len(),
        problems.join("\n  - ")
    );
}

// --- AC-1: the enabled button's caption --------------------------------------

#[test]
fn the_enabled_folder_buttons_caption_is_painted_in_text_not_in_text_disabled() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let look = button_look(&painted(&model));

    let mut problems = Vec::new();
    expect_colour(
        &mut problems,
        "the enabled button's caption",
        look.caption,
        TEXT_DARK,
        "text",
    );
    if look.caption == TEXT_DISABLED_DARK {
        problems.push(format!(
            "the enabled button's caption is painted in `text-disabled` ({TEXT_DISABLED_DARK:?}): \
             a button the user can click is wearing the disabled look"
        ));
    }
    report(problems);
}

// --- AC-2: the disabled button's three tokens --------------------------------

#[test]
fn a_disabled_folder_button_is_painted_with_the_three_tokens_a_disabled_control_has() {
    let model = model(AppState::Processing { done: 1, total: 4 }, Some(OUT_DIR));
    let look = button_look(&painted(&model));

    // `components.md`, "Choose folder… button", the `disabled (processing)`
    // row: fill `control-disabled`, boundary 1 px `border`, text
    // `text-disabled`. All three, because `delete !` in `folder_button` swaps
    // the enabled and disabled looks wholesale and a suite that pinned one of
    // them would pass on half the defect.
    let mut problems = Vec::new();
    expect_colour(
        &mut problems,
        "the disabled button's fill",
        look.fill,
        CONTROL_DISABLED_DARK,
        "control-disabled",
    );
    expect_colour(
        &mut problems,
        "the disabled button's boundary",
        look.boundary,
        BORDER_DARK,
        "border",
    );
    expect_width(
        &mut problems,
        "the disabled button's boundary",
        look.boundary_width,
        STROKE_CONTROL,
    );
    expect_colour(
        &mut problems,
        "the disabled button's caption",
        look.caption,
        TEXT_DISABLED_DARK,
        "text-disabled",
    );
    report(problems);
}

// --- AC-3: the enabled button is painted from our palette --------------------

#[test]
fn the_enabled_folder_button_is_painted_from_our_palette_and_not_eguis_own() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let look = button_look(&painted(&model));

    let mut problems = Vec::new();
    expect_colour(
        &mut problems,
        "the enabled button's fill",
        look.fill,
        CONTROL_DARK,
        "control",
    );
    expect_colour(
        &mut problems,
        "the enabled button's boundary",
        look.boundary,
        BORDER_STRONG_DARK,
        "border-strong",
    );
    expect_width(
        &mut problems,
        "the enabled button's boundary",
        look.boundary_width,
        STROKE_CONTROL,
    );

    // The clause that makes this criterion about *our* palette rather than
    // about any palette at all.
    if look.fill == EGUI_DEFAULT_BUTTON_FILL {
        problems.push(format!(
            "the enabled button's fill is {EGUI_DEFAULT_BUTTON_FILL:?}, which is exactly what \
             `Widgets::default()` puts in `inactive.weak_bg_fill`: the window is painting egui's \
             palette, not `tokens.md`'s"
        ));
    }
    if look.boundary == EGUI_DEFAULT_BUTTON_BOUNDARY
        && look.boundary_width == EGUI_DEFAULT_BUTTON_BOUNDARY_WIDTH
    {
        problems.push(format!(
            "the enabled button has no boundary at all ({EGUI_DEFAULT_BUTTON_BOUNDARY_WIDTH} px, \
             {EGUI_DEFAULT_BUTTON_BOUNDARY:?}), which is exactly what `Widgets::default()` puts \
             in `inactive.bg_stroke`: the window is painting egui's palette, not `tokens.md`'s"
        ));
    }
    report(problems);
}

// --- AC-4: the flagged row keeps two colours in one galley -------------------

#[test]
fn a_flagged_rows_file_name_is_text_and_its_reason_word_is_text_muted_in_body() {
    let model = model(
        AppState::Done {
            summary: one_flagged_row(),
        },
        Some(OUT_DIR),
    );
    let harness = painted(&model);
    let row = painted_text(&paint_list(&harness), FLAGGED_ROW);

    // The whole row fits, so these byte offsets address the row as written:
    // the name at the head, the reason word at the tail.
    let name = section_at(&row, 0);
    let reason = section_at(&row, FLAGGED_ROW.len() - FLAGGED_REASON.len());

    let mut problems = Vec::new();
    expect_colour(
        &mut problems,
        "the flagged row's file name",
        name.colour,
        TEXT_DARK,
        "text",
    );
    expect_colour(
        &mut problems,
        "the flagged row's reason word",
        reason.colour,
        TEXT_MUTED_DARK,
        "text-muted",
    );
    expect_font(
        &mut problems,
        "the flagged row's file name",
        &name.font,
        &body_font(),
    );
    expect_font(
        &mut problems,
        "the flagged row's reason word",
        &reason.font,
        &body_font(),
    );
    if name.bytes == reason.bytes {
        problems.push(format!(
            "the whole row {FLAGGED_ROW:?} is one section, so the file name and the reason word \
             cannot be carrying different colours: `components.md` builds the row as one \
             `LayoutJob` precisely so the two keep their own roles inside one accessible node"
        ));
    }
    report(problems);
}

// --- AC-5: each theme paints its own column ----------------------------------

#[test]
fn the_folder_button_paints_its_own_column_of_tokens_under_light_and_under_dark() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let dark = look_under(&model, egui::Theme::Dark);
    let light = look_under(&model, egui::Theme::Light);

    let mut problems = Vec::new();
    expect_colour(
        &mut problems,
        "under the dark theme the button's fill",
        dark.fill,
        CONTROL_DARK,
        "control, dark column",
    );
    expect_colour(
        &mut problems,
        "under the dark theme the button's boundary",
        dark.boundary,
        BORDER_STRONG_DARK,
        "border-strong, dark column",
    );
    expect_colour(
        &mut problems,
        "under the dark theme the button's caption",
        dark.caption,
        TEXT_DARK,
        "text, dark column",
    );
    expect_colour(
        &mut problems,
        "under the light theme the button's fill",
        light.fill,
        CONTROL_LIGHT,
        "control, light column",
    );
    expect_colour(
        &mut problems,
        "under the light theme the button's boundary",
        light.boundary,
        BORDER_STRONG_LIGHT,
        "border-strong, light column",
    );
    expect_colour(
        &mut problems,
        "under the light theme the button's caption",
        light.caption,
        TEXT_LIGHT,
        "text, light column",
    );

    // The control. A harness that silently painted one theme twice would
    // satisfy AC-1 to AC-4 and prove nothing about theming, so the two looks
    // being equal has to fail here.
    if dark == light {
        problems.push(format!(
            "the light frame and the dark frame painted the identical look {dark:?}: `tokens.md` \
             gives every role a value in each column and dark overrides every role, so two equal \
             frames mean the theme was never switched"
        ));
    }
    report(problems);
}
