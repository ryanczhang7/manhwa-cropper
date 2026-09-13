//! The egui shell: one window, four stacked regions, painted from a [`Model`].
//!
//! Immediate-mode paint code, excluded from line coverage by the coverage gate
//! (`docs/wiki/stack.md`, "Gates") and checked headlessly with `egui_kittest`
//! instead (`crates/app/tests/gui.rs`), which drives [`paint`] for every window
//! state in `docs/wiki/design/components.md` and reads the AccessKit tree.
//!
//! # Where the values come from
//!
//! Every colour, size, radius and gap below is a token from
//! `docs/wiki/design/tokens.md`, named after the token; the column of regions
//! is `docs/wiki/design/layout.md`; which region shows what in which state is
//! `docs/wiki/design/components.md`.
//!
//! # Where the words come from
//!
//! Nowhere in this file. The string table is `docs/wiki/design/voice.md` and it
//! lives in `lib.rs`; this module asks [`dropzone_text`], [`path_text`],
//! [`progress_text`], [`result_line`] and [`rows`] what to show and paints the
//! answer. The two exceptions are not strings from that table: `ELLIPSIS` is
//! `layout.md`'s truncation mark, which the painter adds to a path that does
//! not fit, and [`crate::ROW_SEPARATOR`] is reached through `lib.rs` precisely
//! so that the separator is still defined in one place.
//!
//! # Theme
//!
//! [`install_style`] registers a palette for *both* themes and never chooses
//! between them: eframe's default `ThemePreference::System` plus
//! `RawInput::system_theme` is what makes the window follow Windows. A
//! `set_theme` here would pin the shipped window to one palette for good.

use std::collections::BTreeMap;
use std::f32::consts::{FRAC_PI_2, PI};

use eframe::egui::{
    self, Align, Button, Color32, CornerRadius, Direction, FontFamily, FontId, Label, Layout,
    Margin, Pos2, Rect, Response, RichText, Shadow, Stroke, StrokeKind, TextStyle, TextWrapMode,
    Theme, Ui, Vec2, Visuals, WidgetInfo, WidgetType,
    style::{Selection, WidgetVisuals, Widgets},
    text::{LayoutJob, TextFormat},
};

use crate::{
    AppState, FOLDER_BUTTON, Model, ROW_SEPARATOR, dropzone_text, path_text, progress_text,
    result_line, rows, window_title,
};

// --- Spacing, sizes, radii and borders (tokens.md) ---------------------------

/// `space-window`: padding inside the window on all four sides.
const SPACE_WINDOW: i8 = 16;
/// `space-stack`: vertical gap between the four stacked regions.
const SPACE_STACK: f32 = 8.0;
/// `space-inline`: horizontal gap between the path label and the button.
const SPACE_INLINE: f32 = 8.0;
/// `space-control-x`: button horizontal padding.
const SPACE_CONTROL_X: f32 = 12.0;
/// `space-control-y`: button vertical padding.
const SPACE_CONTROL_Y: f32 = 6.0;
/// The 4 px between the progress bar and its count (`layout.md`, status slot).
const SPACE_TIGHT: f32 = 4.0;

/// `size-control`: height of every interactive control and of the folder row.
const SIZE_CONTROL: f32 = 32.0;
/// `size-row`: height of one flagged-list row.
const SIZE_ROW: f32 = 24.0;
/// `size-dropzone`: fixed height of the drop zone.
const SIZE_DROPZONE: f32 = 120.0;
/// `size-status`: fixed height of the status slot, so the list never jumps.
const SIZE_STATUS: f32 = 40.0;
/// `size-progress`: progress bar height.
const SIZE_PROGRESS: f32 = 4.0;
/// The folder button is never narrower than this (`components.md`).
const MIN_BUTTON_WIDTH: f32 = 120.0;

/// `radius-control`: button and progress bar ends.
const RADIUS_CONTROL: u8 = 4;
/// `radius-zone`: the drop zone.
const RADIUS_ZONE: u8 = 8;
/// `radius-focus`: the focus ring, control radius plus the ring's offset.
const RADIUS_FOCUS: u8 = 6;

/// `stroke-control`: 1 px boundary of an enabled control and of the drop zone.
const STROKE_CONTROL: f32 = 1.0;
/// `stroke-hover`: 2 px solid boundary while files are dragged over the window,
/// and the width of the focus ring.
const STROKE_HOVER: f32 = 2.0;
/// The drop zone's dashed boundary: 6 px dash, 4 px gap (`tokens.md`).
const DASH_LENGTH: f32 = 6.0;
/// The gap between two dashes of that boundary.
const DASH_GAP: f32 = 4.0;

/// Type scale (`tokens.md`), in egui points.
const SIZE_BODY: f32 = 14.0;
/// `button` is the same size as `body`; the scale has no weight axis.
const SIZE_BUTTON: f32 = 14.0;
/// `small`: the progress count under the bar.
const SIZE_SMALL: f32 = 12.0;
/// `heading`: reserved, the window has no headings in v1.
const SIZE_HEADING: f32 = 20.0;
/// `mono`: reserved, unused in v1.
const SIZE_MONO: f32 = 13.0;

/// Default inner size of the window, logical px (`layout.md`).
const WINDOW_SIZE: [f32; 2] = [520.0, 440.0];
/// Minimum inner size the OS will allow, logical px (`layout.md`).
const WINDOW_MIN_SIZE: [f32; 2] = [400.0, 320.0];

/// The single character a truncated path is cut with (`layout.md`,
/// `…\Pictures\cropped`). Not a string from `voice.md`'s table - it is the
/// painter's own mark for text that did not fit - but the same U+2026 that
/// table insists on everywhere else.
const ELLIPSIS: char = '…';

// --- Colour roles (tokens.md) ------------------------------------------------

/// One theme's value for every colour role the window uses.
///
/// `primary-contrast`, `warning` and `success` are reserved in `tokens.md`;
/// only `warning` appears here, because it has an egui field
/// (`visuals.warn_fg_color`) that would otherwise keep egui's own value.
#[derive(Clone, Copy)]
struct Palette {
    surface: Color32,
    surface_raised: Color32,
    text: Color32,
    text_muted: Color32,
    text_disabled: Color32,
    border: Color32,
    border_strong: Color32,
    primary: Color32,
    drop_hover_fill: Color32,
    control: Color32,
    control_hover: Color32,
    control_active: Color32,
    control_disabled: Color32,
    danger: Color32,
    warning: Color32,
}

/// The reference palette.
const LIGHT: Palette = Palette {
    surface: Color32::from_rgb(0xF3, 0xF3, 0xF3),
    surface_raised: Color32::from_rgb(0xFB, 0xFB, 0xFB),
    text: Color32::from_rgb(0x1B, 0x1B, 0x1B),
    text_muted: Color32::from_rgb(0x5D, 0x5D, 0x5D),
    text_disabled: Color32::from_rgb(0x9B, 0x9B, 0x9B),
    border: Color32::from_rgb(0xD9, 0xD9, 0xD9),
    border_strong: Color32::from_rgb(0x76, 0x76, 0x76),
    primary: Color32::from_rgb(0x00, 0x67, 0xC0),
    drop_hover_fill: Color32::from_rgb(0xE0, 0xEE, 0xFA),
    control: Color32::from_rgb(0xFB, 0xFB, 0xFB),
    control_hover: Color32::from_rgb(0xF0, 0xF0, 0xF0),
    control_active: Color32::from_rgb(0xE5, 0xE5, 0xE5),
    control_disabled: Color32::from_rgb(0xF5, 0xF5, 0xF5),
    danger: Color32::from_rgb(0xC4, 0x2B, 0x1C),
    warning: Color32::from_rgb(0x9D, 0x5D, 0x00),
};

/// Dark overrides every role; no role exists in dark only.
const DARK: Palette = Palette {
    surface: Color32::from_rgb(0x20, 0x20, 0x20),
    surface_raised: Color32::from_rgb(0x2B, 0x2B, 0x2B),
    text: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    text_muted: Color32::from_rgb(0xC5, 0xC5, 0xC5),
    text_disabled: Color32::from_rgb(0x6D, 0x6D, 0x6D),
    border: Color32::from_rgb(0x3F, 0x3F, 0x3F),
    border_strong: Color32::from_rgb(0x9E, 0x9E, 0x9E),
    primary: Color32::from_rgb(0x4C, 0xC2, 0xFF),
    drop_hover_fill: Color32::from_rgb(0x17, 0x32, 0x47),
    control: Color32::from_rgb(0x2D, 0x2D, 0x2D),
    control_hover: Color32::from_rgb(0x38, 0x38, 0x38),
    control_active: Color32::from_rgb(0x27, 0x27, 0x27),
    control_disabled: Color32::from_rgb(0x2A, 0x2A, 0x2A),
    danger: Color32::from_rgb(0xFF, 0x99, 0xA4),
    warning: Color32::from_rgb(0xFC, 0xE1, 0x00),
};

/// The palette for the theme egui is currently painting in.
const fn palette(dark_mode: bool) -> Palette {
    if dark_mode { DARK } else { LIGHT }
}

// --- The window --------------------------------------------------------------

/// What the OS is asked for when the window opens (`layout.md`, "Window").
#[must_use]
pub fn viewport() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title(window_title())
        .with_inner_size(WINDOW_SIZE)
        .with_min_inner_size(WINDOW_MIN_SIZE)
}

/// Apply `tokens.md` to `ctx`, once, at startup.
///
/// Both themes are registered and neither is selected: the preference stays at
/// eframe's `ThemePreference::System`, so the palette follows Windows on the
/// next frame after it changes (`tokens.md`, "Theme").
pub fn install_style(ctx: &egui::Context) {
    ctx.set_visuals_of(Theme::Light, visuals(Theme::Light));
    ctx.set_visuals_of(Theme::Dark, visuals(Theme::Dark));
    ctx.all_styles_mut(|style| {
        style.text_styles = text_styles();
        style.animation_time = 0.0;
        // The button is the only focusable widget in the window
        // (`accessibility.md`, "Focus"), so no label may take focus or a
        // selection.
        style.interaction.selectable_labels = false;
        let spacing = &mut style.spacing;
        spacing.item_spacing = egui::vec2(SPACE_INLINE, SPACE_STACK);
        spacing.button_padding = egui::vec2(SPACE_CONTROL_X, SPACE_CONTROL_Y);
        spacing.interact_size.y = SIZE_CONTROL;
    });
}

/// The type scale, as egui text styles.
fn text_styles() -> BTreeMap<TextStyle, FontId> {
    BTreeMap::from([
        (TextStyle::Body, FontId::proportional(SIZE_BODY)),
        (TextStyle::Button, FontId::proportional(SIZE_BUTTON)),
        (TextStyle::Small, FontId::proportional(SIZE_SMALL)),
        (TextStyle::Heading, FontId::proportional(SIZE_HEADING)),
        (
            TextStyle::Monospace,
            FontId::new(SIZE_MONO, FontFamily::Monospace),
        ),
    ])
}

/// One theme's `Visuals`: egui's own as the base, every field `tokens.md`
/// names overridden.
fn visuals(theme: Theme) -> Visuals {
    let palette = palette(theme == Theme::Dark);
    let mut visuals = match theme {
        Theme::Dark => Visuals::dark(),
        Theme::Light => Visuals::light(),
    };
    visuals.panel_fill = palette.surface;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.surface_raised;
    visuals.override_text_color = Some(palette.text);
    visuals.error_fg_color = palette.danger;
    visuals.warn_fg_color = palette.warning;
    visuals.selection = Selection {
        bg_fill: palette.primary,
        stroke: Stroke::new(STROKE_CONTROL, palette.primary),
    };
    visuals.widgets = widgets(palette);
    visuals.window_stroke = Stroke::new(STROKE_CONTROL, palette.border_strong);
    visuals.window_corner_radius = CornerRadius::same(RADIUS_CONTROL);
    visuals.menu_corner_radius = CornerRadius::same(RADIUS_CONTROL);
    // The window is one flat surface.
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    // A disabled control already has its own three tokens (`control-disabled`,
    // `border`, `text-disabled`); fading them as well would paint neither.
    visuals.disabled_alpha = 1.0;
    visuals
}

/// The widget palette: the folder button at rest, hovered and pressed.
fn widgets(palette: Palette) -> Widgets {
    let widget = |fill: Color32, boundary: Color32, text: Color32| WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(STROKE_CONTROL, boundary),
        corner_radius: CornerRadius::same(RADIUS_CONTROL),
        fg_stroke: Stroke::new(STROKE_CONTROL, text),
        // Nothing grows on hover or press; state changes are colour only.
        expansion: 0.0,
    };
    Widgets {
        noninteractive: widget(palette.surface, palette.border, palette.text),
        inactive: widget(palette.control, palette.border_strong, palette.text),
        hovered: widget(palette.control_hover, palette.border_strong, palette.text),
        active: widget(palette.control_active, palette.border_strong, palette.text),
        open: widget(palette.control, palette.border_strong, palette.text),
    }
}

// --- Painting one frame ------------------------------------------------------

/// Paint one frame of the window into `ui`.
///
/// `hovering` is true while files are dragged over the window
/// (`layout.md`, "Drop target"). Nothing here reads input or returns an event:
/// the window's four regions are a function of the model and that one flag.
pub fn paint(ui: &mut Ui, model: &Model, hovering: bool) {
    let palette = palette(ui.visuals().dark_mode);
    egui::Frame::NONE
        .inner_margin(Margin::same(SPACE_WINDOW))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(SPACE_INLINE, SPACE_STACK);
            let width = ui.available_width();
            drop_zone(ui, width, model, hovering, palette);
            folder_row(ui, width, model, palette);
            status_slot(ui, width, model, palette);
            flagged_list(ui, model, palette);
        });
}

/// Whether a run is under way, which every region reads.
fn is_processing(model: &Model) -> bool {
    matches!(model.state, AppState::Processing { .. })
}

/// Region 1: the drop zone. A status display shaped like a target - never
/// interactive, never focusable (`components.md`).
fn drop_zone(ui: &mut Ui, width: f32, model: &Model, hovering: bool, palette: Palette) {
    let size = egui::vec2(width, SIZE_DROPZONE);
    ui.allocate_ui_with_layout(
        size,
        Layout::centered_and_justified(Direction::TopDown),
        |ui| {
            ui.set_min_size(size);
            let rect = ui.max_rect();
            // A run beats a hovering drag, because a drop made during one is
            // ignored: the disabled look is the signal the design wants.
            let (fill, stroke, dashed, color) = if is_processing(model) {
                (
                    palette.surface,
                    Stroke::new(STROKE_CONTROL, palette.border),
                    true,
                    palette.text_disabled,
                )
            } else if hovering {
                (
                    palette.drop_hover_fill,
                    Stroke::new(STROKE_HOVER, palette.primary),
                    false,
                    palette.text,
                )
            } else {
                (
                    palette.surface_raised,
                    Stroke::new(STROKE_CONTROL, palette.border_strong),
                    true,
                    palette.text_muted,
                )
            };
            let radius = CornerRadius::same(RADIUS_ZONE);
            ui.painter().rect_filled(rect, radius, fill);
            if dashed {
                // Inset by half the width so the boundary is not half clipped
                // by the rect it draws.
                let outline = rounded_rect_outline(
                    rect.shrink(stroke.width / 2.0),
                    f32::from(RADIUS_ZONE) - stroke.width / 2.0,
                );
                ui.painter().extend(egui::Shape::dashed_line(
                    &outline,
                    stroke,
                    DASH_LENGTH,
                    DASH_GAP,
                ));
            } else {
                ui.painter()
                    .rect_stroke(rect, radius, stroke, StrokeKind::Inside);
            }
            ui.add(Label::new(
                RichText::new(dropzone_text(model, hovering)).color(color),
            ));
        },
    );
}

/// Region 2: the path label on the left, the folder button right-aligned.
fn folder_row(ui: &mut Ui, width: f32, model: &Model, palette: Palette) {
    let size = egui::vec2(width, SIZE_CONTROL);
    ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
        ui.set_min_size(size);
        folder_button(ui, !is_processing(model), palette);
        // The label takes every point the button did not need.
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let color = if is_processing(model) {
                palette.text_disabled
            } else if model.output_dir.is_some() {
                palette.text
            } else {
                palette.text_muted
            };
            path_label(ui, &path_text(model), color);
        });
    });
}

/// The one focusable widget in the window.
///
/// `add_enabled(false, ..)` rather than a grey of our own: AccessKit is told
/// the node is disabled and the button leaves the Tab order
/// (`accessibility.md`, "Keyboard path").
fn folder_button(ui: &mut Ui, enabled: bool, palette: Palette) -> Response {
    let response = ui
        .scope(|ui| {
            if !enabled {
                // A disabled widget paints from `noninteractive`; give that
                // slot the three disabled tokens for as long as this button.
                let visuals = ui.visuals_mut();
                visuals.widgets.noninteractive.bg_fill = palette.control_disabled;
                visuals.widgets.noninteractive.weak_bg_fill = palette.control_disabled;
                visuals.widgets.noninteractive.bg_stroke =
                    Stroke::new(STROKE_CONTROL, palette.border);
                visuals.widgets.noninteractive.fg_stroke =
                    Stroke::new(STROKE_CONTROL, palette.text_disabled);
                visuals.override_text_color = Some(palette.text_disabled);
            }
            ui.add_enabled(
                enabled,
                Button::new(FOLDER_BUTTON).min_size(egui::vec2(MIN_BUTTON_WIDTH, SIZE_CONTROL)),
            )
        })
        .inner;
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.expand(STROKE_HOVER),
            CornerRadius::same(RADIUS_FOCUS),
            Stroke::new(STROKE_HOVER, palette.primary),
            StrokeKind::Middle,
        );
    }
    response
}

/// The chosen folder, or `folder.none`.
///
/// Truncated from the left so the innermost folder stays readable, with the
/// **whole** path as the accessible name: a screen reader reads where the
/// files went even when the window is too narrow to show it (`layout.md`,
/// "Output folder row"; `accessibility.md`, "Names").
fn path_label(ui: &mut Ui, full: &str, color: Color32) {
    let shown = truncate_left(ui, full, ui.available_width());
    let response =
        ui.add(Label::new(RichText::new(shown).color(color)).wrap_mode(TextWrapMode::Extend));
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), full));
}

/// Region 3: the fixed 40 px slot, holding exactly one thing - or nothing.
fn status_slot(ui: &mut Ui, width: f32, model: &Model, palette: Palette) {
    let size = egui::vec2(width, SIZE_STATUS);
    ui.allocate_ui_with_layout(size, Layout::top_down(Align::Min), |ui| {
        ui.set_min_size(size);
        match &model.state {
            // Idle shows nothing at all: the slot is allocated so that the
            // list below does not jump when a run starts.
            AppState::Idle => {}
            AppState::Processing { done, total } => {
                ui.spacing_mut().item_spacing.y = SPACE_TIGHT;
                let content = SIZE_PROGRESS + SPACE_TIGHT + ui.text_style_height(&TextStyle::Small);
                ui.add_space(((SIZE_STATUS - content) / 2.0).max(0.0));
                progress_bar(ui, *done, *total, palette);
                ui.add(Label::new(
                    RichText::new(progress_text(*done, *total))
                        .text_style(TextStyle::Small)
                        .color(palette.text_muted),
                ));
            }
            AppState::Done { summary } => {
                centred_line(ui, &result_line(summary), palette.text);
            }
            AppState::Error { message } => {
                // `danger`, and a sentence pair: colour is never the only
                // carrier (`accessibility.md`).
                centred_line(ui, message, palette.danger);
            }
        }
    });
}

/// One line of `body` text, vertically centred in the status slot.
fn centred_line(ui: &mut Ui, text: &str, color: Color32) {
    let content = ui.text_style_height(&TextStyle::Body);
    ui.add_space(((SIZE_STATUS - content) / 2.0).max(0.0));
    ui.add(Label::new(RichText::new(text).color(color)).wrap_mode(TextWrapMode::Truncate));
}

/// The 4 px bar, full width, no percentage text.
///
/// Deliberately unnamed: the only way to name it at this egui version is
/// `ProgressBar::text`, which renders that text inside the bar and adds a
/// second node with the count's name. The `progress.count` Label beneath it is
/// the accessible surface (`components.md`, and the story's Model guidance).
fn progress_bar(ui: &mut Ui, done: u32, total: u32, palette: Palette) {
    let fraction = if total == 0 {
        0.0
    } else {
        done as f32 / total as f32
    };
    ui.scope(|ui| {
        // The bar draws its track with `extreme_bg_color`; the token for a
        // progress track is `border`, and only this widget may see it.
        ui.visuals_mut().extreme_bg_color = palette.border;
        ui.add(
            egui::ProgressBar::new(fraction)
                .desired_height(SIZE_PROGRESS)
                .corner_radius(CornerRadius::same(RADIUS_CONTROL))
                .fill(palette.primary)
                .animate(false),
        );
    });
}

/// Region 4: every flagged and failed file of the run just finished.
///
/// Present in every state and empty in most of them: with no rows it paints
/// nothing at all - no border, no placeholder - so Idle and a clean run look
/// the same (`layout.md`, "Flagged list").
fn flagged_list(ui: &mut Ui, model: &Model, palette: Palette) {
    let row_texts = match &model.state {
        AppState::Done { summary } => rows(summary),
        AppState::Idle | AppState::Processing { .. } | AppState::Error { .. } => Vec::new(),
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in &row_texts {
                flagged_row(ui, row, palette);
            }
        });
}

/// One row: the file name, the separator and the reason word, as a single
/// [`LayoutJob`] so the whole row is exactly one accessible node.
fn flagged_row(ui: &mut Ui, row: &str, palette: Palette) {
    let size = egui::vec2(ui.available_width(), SIZE_ROW);
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_size(size);
        // The last separator, not the first: a reason word never contains one,
        // a file name conceivably could.
        let (name, reason) = row.rsplit_once(ROW_SEPARATOR).unwrap_or((row, ""));
        let font = TextStyle::Body.resolve(ui.style());
        let tail = format!("{ROW_SEPARATOR}{reason}");
        // The name is what gives way when the row is too narrow; the reason
        // word stays visible (`components.md`, "overflow").
        let budget = ui.available_width() - text_width(ui, &tail, &font);
        let name = truncate_right(ui, name, budget);

        let mut job = LayoutJob::default();
        job.append(&name, 0.0, format(&font, palette.text));
        job.append(ROW_SEPARATOR, 0.0, format(&font, palette.text));
        job.append(reason, 0.0, format(&font, palette.text_muted));
        let response = ui.add(Label::new(job).wrap_mode(TextWrapMode::Extend));
        // Truncated or not, the row is named by everything it says.
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, ui.is_enabled(), row));
    });
}

/// One section of a row's [`LayoutJob`].
fn format(font: &FontId, color: Color32) -> TextFormat {
    TextFormat {
        font_id: font.clone(),
        color,
        ..Default::default()
    }
}

// --- Text that does not fit --------------------------------------------------

/// How wide `text` is in `font`, in points.
fn text_width(ui: &Ui, text: &str, font: &FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::PLACEHOLDER)
        .rect
        .width()
}

/// `text` with as much of its **head** dropped as it takes to fit `available`,
/// marked with one [`ELLIPSIS`]. `…\Pictures\cropped`.
fn truncate_left(ui: &Ui, text: &str, available: f32) -> String {
    let font = TextStyle::Body.resolve(ui.style());
    if text_width(ui, text, &font) <= available {
        return text.to_owned();
    }
    for (index, _) in text.char_indices().skip(1) {
        let candidate = format!("{ELLIPSIS}{}", &text[index..]);
        if text_width(ui, &candidate, &font) <= available {
            return candidate;
        }
    }
    ELLIPSIS.to_string()
}

/// `text` with as much of its **tail** dropped as it takes to fit `available`,
/// marked with one [`ELLIPSIS`].
fn truncate_right(ui: &Ui, text: &str, available: f32) -> String {
    let font = TextStyle::Body.resolve(ui.style());
    if text_width(ui, text, &font) <= available {
        return text.to_owned();
    }
    let mut end = text.len();
    while let Some(index) = text[..end].char_indices().next_back().map(|(i, _)| i) {
        let candidate = format!("{}{ELLIPSIS}", &text[..index]);
        if text_width(ui, &candidate, &font) <= available {
            return candidate;
        }
        end = index;
    }
    ELLIPSIS.to_string()
}

// --- Shapes ------------------------------------------------------------------

/// The outline of a rounded rectangle as one closed polyline, so that
/// `Shape::dashed_line` can dash it end to end with one phase.
fn rounded_rect_outline(rect: Rect, radius: f32) -> Vec<Pos2> {
    /// Segments per quarter-circle corner. Six is smooth at radius 8 and keeps
    /// the dash pattern from crawling.
    const SEGMENTS: usize = 6;

    let radius = radius.clamp(0.0, rect.width().min(rect.height()) / 2.0);
    // Clockwise in screen space (y grows downward), starting where the top
    // edge leaves the top-left corner.
    let corners = [
        (
            egui::pos2(rect.max.x - radius, rect.min.y + radius),
            -FRAC_PI_2,
        ),
        (egui::pos2(rect.max.x - radius, rect.max.y - radius), 0.0),
        (
            egui::pos2(rect.min.x + radius, rect.max.y - radius),
            FRAC_PI_2,
        ),
        (egui::pos2(rect.min.x + radius, rect.min.y + radius), PI),
    ];
    let start = egui::pos2(rect.min.x + radius, rect.min.y);
    let mut points = vec![start];
    for (centre, from) in corners {
        for step in 0..=SEGMENTS {
            let angle = from + FRAC_PI_2 * (step as f32 / SEGMENTS as f32);
            points.push(centre + radius * Vec2::angled(angle));
        }
    }
    points.push(start);
    points
}

// --- The eframe application --------------------------------------------------

/// The eframe application: the style, and the model the window paints.
///
/// MC-015 paints; MC-016 wires the events. Until it does, the model is the one
/// a launch leaves behind and nothing changes it.
#[derive(Debug)]
pub struct CropperApp {
    model: Model,
}

impl CropperApp {
    /// Install the style once, and start from the settings on disk.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>, model: Model) -> Self {
        install_style(&cc.egui_ctx);
        Self { model }
    }
}

impl eframe::App for CropperApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(ui.visuals().panel_fill))
            .show(ui, |ui| paint(ui, &self.model, hovering));
    }
}
