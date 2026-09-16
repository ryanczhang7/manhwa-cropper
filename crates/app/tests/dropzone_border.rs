//! MC-029: the geometry of the drop zone's dashed border.
//!
//! # The surface these tests read
//!
//! `egui_kittest`'s `Harness::output()` hands back the `egui::FullOutput` of
//! the last frame, and its `shapes` are the **untessellated** paint list.
//! MC-022 read `Shape::Text` out of it to pin what the window *says*
//! (`tests/truncation.rs`) and MC-021 read `Shape::Rect` to pin what colour it
//! is (`tests/palette.rs`). Neither observed a third kind, and this story
//! needed one: `drop_zone` builds the boundary as a polyline and hands it to
//! `egui::Shape::dashed_line`, which emits `Shape::LineSegment` per dash
//! (`epaint-0.36.2/src/shapes/shape.rs`, `dashes_from_line`).
//!
//! RED's first job was to confirm those segments really arrive, rather than
//! being tessellated away or nested somewhere the walk cannot see. They do:
//! the idle 520x440 frame carries 134 `Shape::LineSegment`s, flat in
//! `output().shapes` (`Painter::extend` pushes them individually, not wrapped
//! in a `Shape::Vec`), each `w1 #9E_9E_9E_FF`. So the border is readable as
//! coordinates, with no renderer, no image and no `snapshot` / `wgpu` feature.
//!
//! # Why this is a separate file
//!
//! `tests/gui.rs` states its contract in its first paragraph: "Every assertion
//! here is a query against that tree - never a pixel, never a rect, never a
//! colour of a painted shape." Every assertion here is a *rect* of a painted
//! shape, so they do not belong under that sentence. `tests/palette.rs` is
//! about colour and says so in its own docs, and MC-021's "What is deliberately
//! not asserted" hands geometry to this story; mixing the two would leave
//! neither file describable in a sentence. Nothing in either file is modified.
//!
//! # Why every expected number below is a literal
//!
//! The dash, the gap, the stroke, the radius and the colours are copied out of
//! `docs/wiki/design/tokens.md` and `components.md`, never read back from
//! `gui.rs`'s own private constants - which would make every assertion here
//! say only that the painter agrees with itself. `tests/gui.rs` and
//! `tests/palette.rs` state the same convention. The two derived quantities,
//! the 0.5 pt inset and the 7.5 pt boundary radius, are the arithmetic **the
//! criteria themselves state** ("inside it by half the stroke width",
//! "`radius-zone` less half the stroke width"), not a reading of the painter.
//!
//! # What is deliberately not asserted
//!
//! Not the number of dashes, not the coordinates of any particular one, and
//! not the seam where the closed polyline's dash pattern fails to come out
//! even. All three are quantisation artefacts of `dashes_from_line` and will
//! move for reasons that are not defects (MC-029, `## Out of scope`). Not
//! colour beyond the one stroke colour AC-1 names, because a dash in the wrong
//! colour is not the boundary AC-1 is about; the rest is MC-021. Not the
//! centring of the zone's caption, which is MC-030.

use std::f32::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Stroke, Vec2};
use egui_kittest::Harness;
use manhwa_cropper::{AppState, Model, gui};

// --- The frozen numbers (docs/wiki/design/tokens.md, components.md) ----------

/// `stroke-control` (`tokens.md`, "Radii, borders, elevation"): 1 px
/// `border-strong`, and for the drop zone "dashed (6 px dash, 4 px gap,
/// `Shape::dashed_line`)".
const STROKE_CONTROL: f32 = 1.0;
/// `stroke-hover`: "2 px `primary`, solid ... drop zone while files are
/// dragged over the window".
const STROKE_HOVER: f32 = 2.0;
/// The dash of `stroke-control`'s drop-zone row.
const DASH_LENGTH: f32 = 6.0;
/// The gap of `stroke-control`'s drop-zone row.
const DASH_GAP: f32 = 4.0;
/// `radius-zone`: 8, "drop zone".
const RADIUS_ZONE: f32 = 8.0;

/// `border-strong`, dark column (`tokens.md`, "Colour roles"): "boundary of
/// every enabled control **and of the drop zone**".
const BORDER_STRONG_DARK: Color32 = Color32::from_rgb(0x9E, 0x9E, 0x9E);
/// `primary`, dark column: "drop-hover border".
const PRIMARY_DARK: Color32 = Color32::from_rgb(0x4C, 0xC2, 0xFF);

// --- The two derived quantities the criteria state --------------------------

/// AC-2: "the boundary sits inside it by **half the stroke width** on every
/// side". Half of `stroke-control`.
const INSET: f32 = STROKE_CONTROL / 2.0;

/// AC-3: "the inset rect at radius `radius-zone` **less half the stroke
/// width**". 8 - 0.5.
const BOUNDARY_RADIUS: f32 = RADIUS_ZONE - INSET;

/// The share of the boundary a 6-on, 4-off pattern paints: 6/10.
const PAINTED_SHARE: f32 = DASH_LENGTH / (DASH_LENGTH + DASH_GAP);

// --- The tolerances, and where each one comes from ---------------------------

/// **AC-2, and it is deliberately far tighter than the criterion's 0.5 pt
/// ceiling.**
///
/// The extent of the painted dashes on each side is *exact* arithmetic, not an
/// approximation. `dashes_from_line` places every endpoint at
/// `start + vector * t`; on the left edge `vector.x` is zero, so every point on
/// that edge has `x` bit-identical to the inset rect's `min.x`, and the same
/// holds for the other three straight edges. The corner points are strictly
/// inside those four extremes. So the extent is not "close to" the inset rect,
/// it *is* the inset rect, and the only slack needed is for the f32 subtraction
/// in the comparison itself.
///
/// Choosing the 0.5 pt ceiling instead would have been a trap: the
/// `stroke.width % 2.0` mutant insets by 1.0 rather than 0.5, a displacement of
/// exactly 0.5 pt, which would sit *on* the boundary of the assertion. At
/// 1e-3 it misses by a factor of 500.
const EXTENT_TOLERANCE: f32 = 1e-3;

/// Chords per quarter circle in the polyline the painter dashes. This is a
/// fact about the *construction*, which is what AC-3 says to derive the
/// tolerance from - not a design token, and not a value any assertion below
/// compares against.
const SEGMENTS_PER_CORNER: f32 = 6.0;

/// **AC-3.** A dash endpoint on a corner does not lie on the true arc: it lies
/// on one of the six chords that approximate it, which is *inside* the arc.
/// Each chord subtends `90 / 6 = 15` degrees, and the deepest a chord dips
/// below its arc is the sagitta at its midpoint, `r * (1 - cos(7.5 degrees))`.
/// At `r = 7.5` that is 0.0642 pt. The `+ 1e-3` is slack for f32 arithmetic in
/// the distance function, nothing more.
///
/// The discriminator this has to stay well under is the square corner AC-3's
/// second clause rejects: the inset rect's own corner is `r * (sqrt(2) - 1)` =
/// **3.107 pt** from the rounded boundary, 48 times this tolerance. The test
/// `the_boundary_checks_reject_...` measures both numbers rather than trusting
/// this comment.
///
/// **The shipped border sits at 98 % of this bound** - its furthest endpoint is
/// 0.06416 pt out against a limit of 0.06516 - and that is the derivation
/// working, not luck. The sagitta *is* the maximum a chord can dip below its
/// arc, and a dash endpoint lands near a chord's midpoint sooner or later over
/// 24 chords, so the measured worst case converges on the analytic bound from
/// below. The only headroom needed is f32 error in the distance function, which
/// is around 1e-5, and the `+ 1e-3` covers it a hundred times over. Widening
/// this "for safety" would cost real discrimination: the thinnest mutant this
/// file has to catch, `f32::from(RADIUS_ZONE) - stroke.width % 2.0` (a radius of
/// 7.0 instead of 7.5), is only 0.207 pt out - 3.2x this tolerance, and inside
/// a lazier one.
fn boundary_tolerance() -> f32 {
    BOUNDARY_RADIUS * (1.0 - (FRAC_PI_2 / (2.0 * SEGMENTS_PER_CORNER)).cos()) + 1e-3
}

/// **AC-5**, "within 10 %", read as a relative tolerance on both clauses: the
/// painted share must be within 10 % of 6/10 (so 0.54 to 0.66) and each gap
/// within 10 % of 4 pt (so 3.6 to 4.4). A solid outline scores 1.0 and a
/// swapped pattern 0.4; both are several times outside the band.
const MEASUREMENT_TOLERANCE: f32 = 0.10;

/// Two painted segments are parts of **one** dash when they share an endpoint.
///
/// `dashes_from_line` walks the polyline segment by segment and cuts any dash
/// that runs past a polyline vertex in two, pushing `[start_point, end]` and
/// resuming from the same point on the next segment
/// (`epaint-0.36.2/src/shapes/shape.rs:573`). The shared point is bit-identical
/// (`array_windows` hands the same `Pos2` to both iterations), so this only
/// has to be non-zero at all. It is 1e-4 rather than 0.0 so that the join test
/// is about coincidence rather than about float equality; the smallest real gap
/// the design allows is 4 pt, four orders of magnitude away, so nothing can be
/// merged by accident.
const JOIN_TOLERANCE: f32 = 1e-4;

// --- The frozen strings (docs/wiki/design/voice.md, via components.md) -------

/// `dropzone.ready`, the caption of the idle zone once a folder is chosen.
/// Used only to *find* the zone, never asserted - that is MC-022's story.
const ZONE_TEXT_IDLE: &str = "Drop images here";
/// `dropzone.hover`, the caption while files are dragged over the window.
const ZONE_TEXT_HOVER: &str = "Release to crop";

/// An output path short enough that the folder row never truncates, so nothing
/// here depends on MC-022's arithmetic.
const OUT_DIR: &str = "D:/shots/out";

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
/// `install_style` is called inside the closure for the reason `palette.rs`
/// gives: it is what installs the palette, and AC-1 and AC-4 read a colour.
fn painted(model: &Model, hovering: bool) -> Harness<'_> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(DEFAULT_SIZE[0], DEFAULT_SIZE[1]))
        .build_ui(move |ui| {
            gui::install_style(ui.ctx());
            gui::paint(ui, model, hovering);
        });
    harness.run();
    harness
}

// --- Reading the paint list --------------------------------------------------

/// One `Shape::Rect` the painter emitted.
#[derive(Clone, Debug)]
struct PaintedRect {
    rect: Rect,
    boundary: Color32,
    boundary_width: f32,
}

/// One `Shape::Text` the painter emitted. Only `pos` is used, and only to name
/// the zone.
#[derive(Clone, Debug)]
struct PaintedText {
    pos: Pos2,
    text: String,
}

/// One `Shape::LineSegment` the painter emitted: one *piece* of a dash.
#[derive(Clone, Copy, Debug)]
struct PaintedSegment {
    a: Pos2,
    b: Pos2,
    colour: Color32,
    width: f32,
}

impl PaintedSegment {
    fn length(&self) -> f32 {
        (self.b - self.a).length()
    }
}

/// One shape, in paint order.
#[derive(Clone, Debug)]
enum Painted {
    Rect(PaintedRect),
    Text(PaintedText),
    Segment(PaintedSegment),
}

/// Every rect, string and line segment the last frame painted, in paint order.
///
/// This is MC-021's `paint_list` (`tests/palette.rs`) with one arm added.
/// **What extending it cost**, since `## Model guidance` asks: the walk itself
/// cost three lines, because `Shape::LineSegment` is a plain struct variant
/// carrying `[Pos2; 2]` and a `Stroke` and needs no unpacking - unlike
/// `Shape::Text`, whose galley MC-021 had to reach into. What it did *not*
/// generalise is the identification helper: `innermost_rect_containing` is
/// blind to anything that is not a `Shape::Rect`, exactly as MC-021's handoff
/// records, so it finds the zone's *fill* here and contributes nothing to
/// finding the border. The border is identified by geometry instead
/// (`zone_segments`), which is this file's own work.
///
/// `Shape::Vec` is walked because egui nests shapes. In this window the dashes
/// arrive flat - `Painter::extend` pushes each `Shape` into the paint list on
/// its own - but a future frame that nested them must not make the border
/// silently vanish from the read.
fn paint_list(harness: &Harness<'_>) -> Vec<Painted> {
    fn walk(shape: &egui::Shape, out: &mut Vec<Painted>) {
        match shape {
            egui::Shape::Rect(rect) => out.push(Painted::Rect(PaintedRect {
                rect: rect.rect,
                boundary: rect.stroke.color,
                boundary_width: rect.stroke.width,
            })),
            egui::Shape::Text(text) => out.push(Painted::Text(PaintedText {
                pos: text.pos,
                text: text.galley.text().to_owned(),
            })),
            egui::Shape::LineSegment { points, stroke } => {
                out.push(Painted::Segment(PaintedSegment {
                    a: points[0],
                    b: points[1],
                    colour: stroke.color,
                    width: stroke.width,
                }));
            }
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
        "exactly one painted string must read {wanted:?}, and {} do",
        found.len()
    );
    found.remove(0)
}

/// Every painted line segment in the frame.
fn segments(list: &[Painted]) -> Vec<PaintedSegment> {
    list.iter()
        .filter_map(|painted| match painted {
            Painted::Segment(segment) => Some(*segment),
            _ => None,
        })
        .collect()
}

/// The drop zone: its rectangle, and every painted rect that has exactly that
/// geometry.
#[derive(Clone, Debug)]
struct Zone {
    rect: Rect,
    rects: Vec<PaintedRect>,
}

/// Find the drop zone by the caption painted inside it.
///
/// This is MC-021's `innermost_rect_containing` - the rect containing the
/// caption's position that is smallest by area, which is the one a user sees
/// under that point - with **one change forced by this story's frames**. The
/// idle zone paints one rect (its fill) but the drop-hovered zone paints
/// *two* with identical bounds, a fill and a stroke-only sibling
/// (`rect_filled` then `rect_stroke`). MC-021's uniqueness guard fails on that
/// tie with "2 are tied at 56640 sq pt", because it was written to return one
/// *shape*. So this version requires the tie to be unambiguous about
/// **geometry** - every shape tied for smallest must have the same `rect` - and
/// returns all of them. The nesting guard is kept verbatim: every rect
/// containing the caption must contain the zone, so "the one on top" stays a
/// fact about the frame rather than an assumption.
///
/// That is the whole cost of reusing the helper, and it is why `tests/palette.rs`
/// is not edited: MC-021 never paints a hovered frame and does not need it.
#[track_caller]
fn zone(list: &[Painted], caption: &str) -> Zone {
    let area = |rect: Rect| rect.width() * rect.height();
    let pos = painted_text(list, caption).pos;
    let candidates: Vec<PaintedRect> = list
        .iter()
        .filter_map(|painted| match painted {
            Painted::Rect(rect) if rect.rect.contains(pos) => Some(rect.clone()),
            _ => None,
        })
        .collect();
    assert!(
        !candidates.is_empty(),
        "no painted rect contains the drop zone's caption at {pos:?}: the shape the caption sits \
         inside is how this suite names the zone, so none means the zone painted no background \
         at all"
    );

    let smallest = candidates
        .iter()
        .map(|candidate| area(candidate.rect))
        .fold(f32::INFINITY, f32::min);
    let innermost: Vec<PaintedRect> = candidates
        .iter()
        .filter(|candidate| area(candidate.rect) == smallest)
        .cloned()
        .collect();
    let rect = innermost[0].rect;
    for candidate in &innermost {
        assert_eq!(
            candidate.rect, rect,
            "the painted rects tied for innermost around the caption at {pos:?} must all be the \
             drop zone's own rectangle; {:?} and {rect:?} are different shapes of the same area, \
             so the identification is ambiguous. Candidates: {candidates:#?}",
            candidate.rect
        );
    }
    for candidate in &candidates {
        assert!(
            candidate.rect.contains_rect(rect),
            "the rects containing {pos:?} must be nested, innermost last, for \"the one on top\" \
             to be well defined; {:?} overlaps {rect:?} without containing it. Candidates: \
             {candidates:#?}",
            candidate.rect
        );
    }
    Zone {
        rect,
        rects: innermost,
    }
}

/// The painted segments that belong to the drop zone's boundary.
///
/// Identified by geometry, because the border is not a widget and has no
/// caption to sit inside: a segment belongs to the zone when both of its
/// endpoints lie within the zone's rectangle, generously expanded. The guard
/// is what makes this safe - **every** segment in the frame must qualify, so a
/// border painted somewhere else entirely cannot quietly leave this function
/// with an empty list for AC-2, AC-3 and AC-5 to pass vacuously over.
#[track_caller]
fn zone_segments(list: &[Painted], zone: &Zone) -> Vec<PaintedSegment> {
    let reach = zone.rect.expand(STROKE_HOVER + 1.0);
    let all = segments(list);
    for segment in &all {
        assert!(
            reach.contains(segment.a) && reach.contains(segment.b),
            "every line segment this frame paints must belong to the drop zone's boundary, and \
             {:?} -> {:?} lies outside {:?}. Either another region has started painting lines - \
             in which case this filter is no longer sound - or the boundary has moved off the \
             zone entirely.",
            segment.a,
            segment.b,
            zone.rect
        );
    }
    all
}

// --- Measuring the boundary ---------------------------------------------------

/// One whole dash: where it starts, where it ends, and how much line it paints.
///
/// `length` is the sum of its pieces' lengths (the distance along the path),
/// not the straight line from `start` to `end`, because a dash that crosses a
/// corner is bent.
#[derive(Clone, Copy, Debug)]
struct Dash {
    start: Pos2,
    end: Pos2,
    length: f32,
}

/// Glue the painted segments back into dashes.
///
/// **This is the subtlety AC-5 turns on.** The number of `LineSegment`s is not
/// the number of dashes: `dashes_from_line` walks the polyline one straight
/// piece at a time and splits any dash that runs past a vertex into two
/// segments that share an endpoint, so the corners of this border arrive as
/// runs of two to four pieces with a zero gap between them. Measuring "the gap
/// between consecutive segments" would therefore measure a crowd of zeros and
/// report a mean near zero for a border whose gaps are all 4 pt. Segments that
/// share an endpoint are one dash; see [`JOIN_TOLERANCE`].
///
/// The walker also emits genuinely zero-length segments where a dash ends
/// exactly on a vertex (the idle frame has one, at `[488.0 24.5]`). They share
/// an endpoint with the dash they close, so they merge away here rather than
/// inventing a dash of length nothing.
fn dashes(segments: &[PaintedSegment]) -> Vec<Dash> {
    let mut out: Vec<Dash> = Vec::new();
    for segment in segments {
        match out.last_mut() {
            Some(last) if (segment.a - last.end).length() <= JOIN_TOLERANCE => {
                last.end = segment.b;
                last.length += segment.length();
            }
            _ => out.push(Dash {
                start: segment.a,
                end: segment.b,
                length: segment.length(),
            }),
        }
    }
    out
}

/// The gap between each pair of consecutive dashes.
///
/// Consecutive *in paint order*, which is order along the polyline, and
/// deliberately not wrapped around: the closed polyline's dash pattern does not
/// come out even, so the seam where the last dash meets the first is a partial
/// gap (about 1 pt in the idle frame). That is a quantisation artefact of the
/// walker and is out of scope by name.
fn gaps(dashes: &[Dash]) -> Vec<f32> {
    dashes
        .windows(2)
        .map(|pair| (pair[1].start - pair[0].end).length())
        .collect()
}

/// The length of the boundary the dashes are drawn on: the perimeter of a
/// rounded rectangle, from the criteria's own arithmetic.
///
/// Four straight edges shortened by a radius at each end, plus four quarter
/// circles that make one whole one. The painter walks a chord approximation of
/// those arcs, which is 0.14 pt shorter over the whole border - 0.012 % of
/// 1167 pt, and a rounding error against AC-5's 10 % band.
fn boundary_length(zone: &Zone) -> f32 {
    let inset = zone.rect.shrink(INSET);
    2.0 * (inset.width() - 2.0 * BOUNDARY_RADIUS)
        + 2.0 * (inset.height() - 2.0 * BOUNDARY_RADIUS)
        + 2.0 * PI * BOUNDARY_RADIUS
}

/// Signed distance from `point` to the boundary of a rounded rectangle:
/// negative inside, positive outside, zero exactly on it.
///
/// The standard rounded-box distance field. It is used for AC-3, so it is
/// worth saying what it is *not*: it is not a reading of `rounded_rect_outline`
/// and shares no code with it. It describes the ideal shape `components.md`
/// specifies - a rectangle with `radius-zone` corners - and asks how far each
/// painted point is from it.
fn distance_to_rounded_rect(point: Pos2, rect: Rect, radius: f32) -> f32 {
    let from_centre = point - rect.center();
    let q = egui::vec2(
        from_centre.x.abs() - rect.width() / 2.0 + radius,
        from_centre.y.abs() - rect.height() / 2.0 + radius,
    );
    let outside = egui::vec2(q.x.max(0.0), q.y.max(0.0)).length();
    let inside = q.x.max(q.y).min(0.0);
    outside + inside - radius
}

/// Whether `point` falls in the square corner a radius of zero would produce:
/// inside the inset rectangle's corner box of side `radius`, but further from
/// that corner's arc centre than the arc itself.
///
/// This is AC-3's second clause stated as a predicate rather than inferred from
/// the distance above, so that a control can be pointed at it directly.
fn in_square_corner(point: Pos2, rect: Rect, radius: f32) -> bool {
    let centres = [
        egui::pos2(rect.min.x + radius, rect.min.y + radius),
        egui::pos2(rect.max.x - radius, rect.min.y + radius),
        egui::pos2(rect.min.x + radius, rect.max.y - radius),
        egui::pos2(rect.max.x - radius, rect.max.y - radius),
    ];
    centres.iter().any(|centre| {
        let dx = point.x - centre.x;
        let dy = point.y - centre.y;
        // Only the quadrant of this corner that lies away from the rect's
        // middle, which is where a square corner sticks out.
        let away_x = (centre.x - rect.center().x) * dx > 0.0;
        let away_y = (centre.y - rect.center().y) * dy > 0.0;
        away_x
            && away_y
            && dx.abs() <= radius
            && dy.abs() <= radius
            && (dx * dx + dy * dy).sqrt() > radius + boundary_tolerance()
    })
}

// --- The five checks, each stated once so a control can aim at it ------------

/// **AC-1.** More than one separate dash, each stroked 1 px in `border-strong`.
fn separate_dashes_problems(segments: &[PaintedSegment]) -> Vec<String> {
    let mut problems = Vec::new();
    let dashes = dashes(segments);
    if dashes.len() < 2 {
        problems.push(format!(
            "the drop zone's boundary is painted as {} dash(es), but `tokens.md` gives the \
             drop-zone boundary a dashed `stroke-control` (6 px dash, 4 px gap) - a series of \
             separate dashes is more than one",
            dashes.len()
        ));
    }
    let mut wrong_width: Vec<String> = segments
        .iter()
        .filter(|segment| segment.width != STROKE_CONTROL)
        .map(|segment| format!("{}", segment.width))
        .collect();
    wrong_width.sort();
    wrong_width.dedup();
    if !wrong_width.is_empty() {
        problems.push(format!(
            "{} of the boundary's {} painted pieces are stroked {} px, but `stroke-control` is \
             {STROKE_CONTROL} px (`tokens.md`, \"Radii, borders, elevation\")",
            segments
                .iter()
                .filter(|segment| segment.width != STROKE_CONTROL)
                .count(),
            segments.len(),
            wrong_width.join(" / ")
        ));
    }
    let mut wrong_colour: Vec<String> = segments
        .iter()
        .filter(|segment| segment.colour != BORDER_STRONG_DARK)
        .map(|segment| format!("{:?}", segment.colour))
        .collect();
    wrong_colour.sort();
    wrong_colour.dedup();
    if !wrong_colour.is_empty() {
        problems.push(format!(
            "{} of the boundary's {} painted pieces are stroked {}, but `border-strong` is \
             {BORDER_STRONG_DARK:?} (`tokens.md`, \"Colour roles\": the boundary of every enabled \
             control and of the drop zone)",
            segments
                .iter()
                .filter(|segment| segment.colour != BORDER_STRONG_DARK)
                .count(),
            segments.len(),
            wrong_colour.join(" / ")
        ));
    }
    problems
}

/// **AC-2.** The dashes' extent is the zone's rect inset by half the stroke
/// width on every side.
fn inset_problems(segments: &[PaintedSegment], zone: &Zone) -> Vec<String> {
    let mut problems = Vec::new();
    if segments.is_empty() {
        problems.push(
            "the drop zone paints no boundary at all, so there is nothing inset from its rect"
                .to_string(),
        );
        return problems;
    }
    let mut extent = Rect::from_points(&[segments[0].a]);
    for segment in segments {
        extent.extend_with(segment.a);
        extent.extend_with(segment.b);
    }
    let want = zone.rect.shrink(INSET);
    for (side, got, expected) in [
        ("left", extent.min.x, want.min.x),
        ("top", extent.min.y, want.min.y),
        ("right", extent.max.x, want.max.x),
        ("bottom", extent.max.y, want.max.y),
    ] {
        let off = got - expected;
        if off.abs() > EXTENT_TOLERANCE {
            problems.push(format!(
                "the boundary's {side} edge is painted at {got}, but half of `stroke-control` \
                 inside the drop zone's own {side} edge is {expected} - out by {off} pt, and the \
                 inset is exact arithmetic so the tolerance is {EXTENT_TOLERANCE}"
            ));
        }
    }
    problems
}

/// **AC-3.** Every endpoint lies on the rounded boundary, and none in a square
/// corner.
fn boundary_problems(segments: &[PaintedSegment], zone: &Zone) -> Vec<String> {
    let mut problems = Vec::new();
    // AC-3 reads "when **every** dash endpoint is tested". Over an empty set
    // that is vacuously true, and a universally quantified test that reports
    // success having tested nothing is precisely the assertion `rules.md`'s
    // non-negotiables exist to prevent: a border deleted outright would leave
    // this check silent. Nothing was tested, so say so.
    if segments.is_empty() {
        problems.push(
            "the drop zone paints no boundary at all, so there are no painted endpoints to test \
             against the rounded rectangle `components.md` gives it"
                .to_string(),
        );
        return problems;
    }
    let inset = zone.rect.shrink(INSET);
    let tolerance = boundary_tolerance();

    let mut worst: Option<(Pos2, f32)> = None;
    let mut strays = 0usize;
    let mut points = 0usize;
    for point in segments.iter().flat_map(|s| [s.a, s.b]) {
        points += 1;
        let distance = distance_to_rounded_rect(point, inset, BOUNDARY_RADIUS);
        if distance.abs() > tolerance {
            strays += 1;
            if worst.is_none_or(|(_, w)| distance.abs() > w) {
                worst = Some((point, distance.abs()));
            }
        }
    }
    if let Some((point, off)) = worst {
        problems.push(format!(
            "{strays} of the boundary's {points} painted endpoints are not on the rounded \
             rectangle `components.md` gives the drop zone - the inset rect at `radius-zone` less \
             half of `stroke-control`, {BOUNDARY_RADIUS} pt. The furthest, {point:?}, is {off} pt \
             off, against a tolerance of {tolerance} pt (the sagitta of one of the six chords per \
             quarter circle, r * (1 - cos 7.5 deg))"
        ));
    }

    let square: Vec<Pos2> = segments
        .iter()
        .flat_map(|s| [s.a, s.b])
        .filter(|point| in_square_corner(*point, inset, BOUNDARY_RADIUS))
        .collect();
    if !square.is_empty() {
        problems.push(format!(
            "{} of the boundary's painted endpoints lie in the square corner a `radius-zone` of \
             zero would produce, e.g. {:?}; `components.md` gives the drop zone a rounded \
             rectangle",
            square.len(),
            square[0]
        ));
    }
    problems
}

/// **AC-5, first clause.** The dashes paint 6/10 of the boundary.
fn share_problems(segments: &[PaintedSegment], zone: &Zone) -> Vec<String> {
    let painted: f32 = segments.iter().map(PaintedSegment::length).sum();
    let boundary = boundary_length(zone);
    let share = painted / boundary;
    if (share - PAINTED_SHARE).abs() > PAINTED_SHARE * MEASUREMENT_TOLERANCE {
        return vec![format!(
            "the dashes paint {painted} pt of a {boundary} pt boundary, a share of {share}, but \
             `tokens.md` gives the drop zone a 6 px dash and a 4 px gap - {PAINTED_SHARE} of it, \
             within {} %",
            MEASUREMENT_TOLERANCE * 100.0
        )];
    }
    Vec::new()
}

/// **AC-5, second clause.** Consecutive dashes are 4 pt apart.
fn gap_problems(segments: &[PaintedSegment]) -> Vec<String> {
    let dashes = dashes(segments);
    let gaps = gaps(&dashes);
    if gaps.is_empty() {
        return vec![
            "the drop zone's boundary has no gap between consecutive dashes, because it is not \
             painted as more than one dash"
                .to_string(),
        ];
    }
    let worst = gaps
        .iter()
        .copied()
        .max_by(|a, b| (a - DASH_GAP).abs().total_cmp(&(b - DASH_GAP).abs()))
        .unwrap_or(f32::NAN);
    if (worst - DASH_GAP).abs() > DASH_GAP * MEASUREMENT_TOLERANCE {
        let mean = gaps.iter().sum::<f32>() / gaps.len() as f32;
        return vec![format!(
            "the widest disagreement among the boundary's {} gaps is {worst} pt (mean {mean}), \
             but `tokens.md` gives the drop zone a {DASH_GAP} px gap, within {} %",
            gaps.len(),
            MEASUREMENT_TOLERANCE * 100.0
        )];
    }
    Vec::new()
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

// --- AC-1: the idle boundary is a series of dashes ---------------------------

#[test]
fn the_idle_drop_zones_boundary_is_painted_as_separate_one_px_border_strong_dashes() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);
    let list = paint_list(&harness);
    let zone = zone(&list, ZONE_TEXT_IDLE);
    let segments = zone_segments(&list, &zone);

    report(separate_dashes_problems(&segments));
}

// --- AC-2: the boundary is inset by half the stroke width --------------------

#[test]
fn the_dashed_boundary_sits_half_a_stroke_width_inside_the_drop_zones_rect() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);
    let list = paint_list(&harness);
    let zone = zone(&list, ZONE_TEXT_IDLE);
    let segments = zone_segments(&list, &zone);

    report(inset_problems(&segments, &zone));
}

// --- AC-3: every endpoint is on the rounded boundary -------------------------

#[test]
fn every_dash_endpoint_lies_on_the_rounded_boundary_and_none_in_a_square_corner() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);
    let list = paint_list(&harness);
    let zone = zone(&list, ZONE_TEXT_IDLE);
    let segments = zone_segments(&list, &zone);

    report(boundary_problems(&segments, &zone));
}

// --- AC-4: the drop-hovered boundary is one solid rect, not dashes -----------

#[test]
fn a_drop_hovered_zone_is_bounded_by_one_solid_two_px_primary_rect_and_no_dashes() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, true);
    let list = paint_list(&harness);
    let zone = zone(&list, ZONE_TEXT_HOVER);

    let mut problems = Vec::new();

    // Half one: no dashes. `components.md`'s drop-hover row is solid, so the
    // zone must paint no line segments at all - and neither must anything else
    // in this frame, which is the same window the idle test reads 134 of them
    // from.
    let all = segments(&list);
    let near: Vec<&PaintedSegment> = all
        .iter()
        .filter(|segment| {
            let reach = zone.rect.expand(STROKE_HOVER + 1.0);
            reach.contains(segment.a) || reach.contains(segment.b)
        })
        .collect();
    if !near.is_empty() {
        problems.push(format!(
            "the drop-hovered zone paints {} dash piece(s), e.g. {:?} -> {:?}, but \
             `components.md`'s drop-hover row is a solid 2 px `primary` border",
            near.len(),
            near[0].a,
            near[0].b
        ));
    }
    if !all.is_empty() && near.is_empty() {
        problems.push(format!(
            "the drop-hovered frame paints {} line segment(s) away from the drop zone; nothing \
             else in this window paints lines, so the boundary has moved rather than gone solid",
            all.len()
        ));
    }

    // Half two: the solid stroked rect is there, at 2 px `primary`.
    let stroked: Vec<&PaintedRect> = zone
        .rects
        .iter()
        .filter(|rect| rect.boundary_width > 0.0)
        .collect();
    match stroked.as_slice() {
        [only] => {
            if only.boundary_width != STROKE_HOVER {
                problems.push(format!(
                    "the drop-hovered zone's boundary is {} px, but `stroke-hover` is \
                     {STROKE_HOVER} px `primary`, solid (`tokens.md`, \"Radii, borders, \
                     elevation\")",
                    only.boundary_width
                ));
            }
            if only.boundary != PRIMARY_DARK {
                problems.push(format!(
                    "the drop-hovered zone's boundary is painted {:?}, but `primary` is \
                     {PRIMARY_DARK:?} (`tokens.md`, \"Colour roles\": the drop-hover border)",
                    only.boundary
                ));
            }
        }
        other => problems.push(format!(
            "the drop-hovered zone must be bounded by exactly one stroked rect, and {} of its \
             {} painted rects carry a stroke: {other:#?}",
            other.len(),
            zone.rects.len()
        )),
    }

    report(problems);
}

// --- AC-5: 6/10 painted, 4 pt gaps -------------------------------------------

#[test]
fn the_dashes_paint_six_tenths_of_the_boundary_with_four_point_gaps() {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);
    let list = paint_list(&harness);
    let zone = zone(&list, ZONE_TEXT_IDLE);
    let segments = zone_segments(&list, &zone);

    // The three numbers MC-029's AC-5 asks to be recorded, printed so the run
    // itself is the record rather than a comment that can drift.
    let painted_length: f32 = segments.iter().map(PaintedSegment::length).sum();
    let boundary = boundary_length(&zone);
    let dashes = dashes(&segments);
    let gaps = gaps(&dashes);
    let mean_gap = gaps.iter().sum::<f32>() / gaps.len() as f32;
    println!(
        "AC-5 measured: painted {painted_length} pt / boundary {boundary} pt = share {}; \
         {} segments merged into {} dashes; gaps min {} mean {mean_gap} max {}",
        painted_length / boundary,
        segments.len(),
        dashes.len(),
        gaps.iter().copied().fold(f32::INFINITY, f32::min),
        gaps.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    );

    let mut problems = share_problems(&segments, &zone);
    problems.extend(gap_problems(&segments));
    report(problems);
}

// --- The controls -------------------------------------------------------------
//
// Everything below manufactures a boundary that is *wrong in one named way* and
// asserts that the check above rejects it. They exist because this story adds
// tests to a painter that already works: every assertion in the five tests
// above passes the moment it is written, so on its own each one could assert
// nothing at all and nothing would notice (`rules.md`, "Non-negotiables").
//
// The polyline builder below duplicates the *construction* of
// `rounded_rect_outline` on purpose, and is used **only** to build wrong
// boundaries. No assertion above compares anything against it; the five checks
// compare against literals from `tokens.md` and against the ideal rounded
// rectangle `distance_to_rounded_rect` describes. If it were the reference,
// this file would be asserting that the painter agrees with a copy of itself.

/// A rounded-rect polyline built the way the painter builds one, with two
/// knobs: the `radius`, and the direction each corner sweeps.
///
/// `sweep = 1.0` is the shipped construction. `sweep = -1.0` is the mutation
/// AC-6 names second: `from + FRAC_PI_2 * ...` becoming `from - FRAC_PI_2 * ...`,
/// which keeps a closed path but turns each corner inside out.
fn control_outline(rect: Rect, radius: f32, sweep: f32) -> Vec<Pos2> {
    const SEGMENTS: usize = 6;
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
            let angle = from + sweep * FRAC_PI_2 * (step as f32 / SEGMENTS as f32);
            points.push(centre + radius * Vec2::angled(angle));
        }
    }
    points.push(start);
    points
}

/// Dash a control polyline with epaint's own walker, so the controls go through
/// exactly the machinery the real border does.
fn control_dashes(points: &[Pos2], dash: f32, gap: f32) -> Vec<PaintedSegment> {
    egui::Shape::dashed_line(
        points,
        Stroke::new(STROKE_CONTROL, BORDER_STRONG_DARK),
        dash,
        gap,
    )
    .iter()
    .filter_map(|shape| match shape {
        egui::Shape::LineSegment { points, stroke } => Some(PaintedSegment {
            a: points[0],
            b: points[1],
            colour: stroke.color,
            width: stroke.width,
        }),
        _ => None,
    })
    .collect()
}

/// The same polyline painted solid: every piece of it, no gaps.
fn control_solid(points: &[Pos2]) -> Vec<PaintedSegment> {
    points
        .windows(2)
        .map(|pair| PaintedSegment {
            a: pair[0],
            b: pair[1],
            colour: BORDER_STRONG_DARK,
            width: STROKE_CONTROL,
        })
        .collect()
}

/// The drop zone's rectangle, read once from the real frame so the controls are
/// built at the size the real border is.
fn idle_zone() -> Zone {
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);
    zone(&paint_list(&harness), ZONE_TEXT_IDLE)
}

#[test]
fn the_dash_measurement_rejects_a_solid_outline_and_a_swapped_dash_pattern() {
    let zone = idle_zone();
    let inset = zone.rect.shrink(INSET);
    let boundary = boundary_length(&zone);
    let points = control_outline(inset, BOUNDARY_RADIUS, 1.0);

    // Control 1: no dashing at all - the whole boundary painted. AC-5 says this
    // "scores 10/10 and must fail the band".
    let solid = control_solid(&points);
    let solid_share: f32 = solid.iter().map(PaintedSegment::length).sum::<f32>() / boundary;
    println!("control: solid outline scores share {solid_share}");
    assert!(
        solid_share > 0.9,
        "the solid control must paint essentially all of the boundary for this to be the control \
         AC-5 describes; it scored {solid_share}"
    );
    assert!(
        !share_problems(&solid, &zone).is_empty(),
        "a solid outline paints {solid_share} of the boundary and AC-5's band is \
         {PAINTED_SHARE} within {} %, so the share check must reject it. It did not, which means \
         the check would pass on a border with no gaps in it at all.",
        MEASUREMENT_TOLERANCE * 100.0
    );

    // Control 2: `tokens.md`'s two constants swapped - a 4 px dash and a 6 px
    // gap. AC-5 says this "scores 4/10 and must also fail", and its gaps are
    // 6 pt rather than 4.
    let swapped = control_dashes(&points, DASH_GAP, DASH_LENGTH);
    let swapped_share: f32 = swapped.iter().map(PaintedSegment::length).sum::<f32>() / boundary;
    let swapped_gaps = gaps(&dashes(&swapped));
    let swapped_mean = swapped_gaps.iter().sum::<f32>() / swapped_gaps.len() as f32;
    println!("control: swapped 4-on-6-off scores share {swapped_share}, mean gap {swapped_mean}");
    assert!(
        !share_problems(&swapped, &zone).is_empty(),
        "a 4 px dash with a 6 px gap paints {swapped_share} of the boundary, and the share check \
         must reject it; it did not"
    );
    assert!(
        !gap_problems(&swapped).is_empty(),
        "a 4 px dash with a 6 px gap leaves gaps of about {swapped_mean} pt, and the gap check \
         must reject them; it did not"
    );

    // And the merge the gap check depends on is not vacuous: the shipped border
    // arrives as more segments than dashes, so a gap check that skipped the
    // merge would be measuring a crowd of zeros.
    let model = model(AppState::Idle, Some(OUT_DIR));
    let harness = painted(&model, false);
    let list = paint_list(&harness);
    let real = zone_segments(&list, &zone);
    assert!(
        dashes(&real).len() < real.len(),
        "at least one dash of the shipped border must arrive split across a polyline vertex, or \
         the merge in `dashes` is untested and the comment explaining it is wrong: {} segments, \
         {} dashes",
        real.len(),
        dashes(&real).len()
    );
}

#[test]
fn the_boundary_checks_reject_a_wrong_inset_square_corners_and_a_reversed_corner_sweep() {
    let zone = idle_zone();
    let inset = zone.rect.shrink(INSET);

    // Control 3: the inset mutated. `rect.shrink(stroke.width % 2.0)` insets by
    // 1.0 instead of 0.5 - a displacement of exactly 0.5 pt, the largest the
    // criterion's own ceiling would have tolerated.
    let wrong_inset = control_dashes(
        &control_outline(zone.rect.shrink(1.0), BOUNDARY_RADIUS, 1.0),
        DASH_LENGTH,
        DASH_GAP,
    );
    let problems = inset_problems(&wrong_inset, &zone);
    println!(
        "control: inset of 1.0 pt reports {} problem(s)",
        problems.len()
    );
    assert_eq!(
        problems.len(),
        4,
        "a boundary inset by 1.0 pt instead of 0.5 is out on all four sides, so the inset check \
         must name all four; it named {}: {problems:#?}",
        problems.len()
    );

    // Control 4: the square corner AC-3's second clause exists to reject. The
    // inset rect's own corner is r * (sqrt(2) - 1) from the rounded boundary.
    let corner = inset.min;
    let corner_distance = distance_to_rounded_rect(corner, inset, BOUNDARY_RADIUS).abs();
    let tolerance = boundary_tolerance();
    println!(
        "control: square corner {corner:?} is {corner_distance} pt from the rounded boundary, \
         against a tolerance of {tolerance} pt ({}x)",
        corner_distance / tolerance
    );
    assert!(
        corner_distance > 40.0 * tolerance,
        "AC-3's second clause is only a discriminator if the square corner it rejects is far \
         outside the tolerance the round one needs. The corner is {corner_distance} pt out and \
         the tolerance is {tolerance} pt, a factor of {}",
        corner_distance / tolerance
    );
    assert!(
        in_square_corner(corner, inset, BOUNDARY_RADIUS),
        "the square-corner predicate must classify the inset rect's own corner {corner:?} as a \
         square corner, or it classifies nothing and AC-3's second clause is decoration"
    );
    // ... and a point on the true rounded boundary must not be.
    let on_arc = egui::pos2(inset.min.x + BOUNDARY_RADIUS, inset.min.y);
    assert!(
        !in_square_corner(on_arc, inset, BOUNDARY_RADIUS),
        "the square-corner predicate must not classify {on_arc:?}, which is where the top edge \
         leaves the rounded corner, as a square corner"
    );

    let square = control_dashes(&control_outline(inset, 0.0, 1.0), DASH_LENGTH, DASH_GAP);
    let problems = boundary_problems(&square, &zone);
    println!("control: radius 0 reports {} problem(s)", problems.len());
    assert_eq!(
        problems.len(),
        2,
        "a square-cornered boundary must fail both of AC-3's clauses - off the rounded shape, and \
         inside the square corner - and it reported {}: {problems:#?}",
        problems.len()
    );
    assert!(
        problems[1].contains("square corner"),
        "the second problem a square-cornered boundary reports must be the square-corner clause: \
         {problems:#?}"
    );

    // Control 5: AC-6's second mutation, `from - FRAC_PI_2 * ...`. The path
    // stays closed and its length barely moves, so nothing but AC-3 catches it.
    let reversed = control_dashes(
        &control_outline(inset, BOUNDARY_RADIUS, -1.0),
        DASH_LENGTH,
        DASH_GAP,
    );
    let problems = boundary_problems(&reversed, &zone);
    println!(
        "control: reversed corner sweep reports {} problem(s): {problems:#?}",
        problems.len()
    );
    assert!(
        !problems.is_empty(),
        "a boundary whose corners sweep the wrong way must fail AC-3; it did not, which means \
         `replace + with - in rounded_rect_outline` would survive this suite"
    );

    // Control 6: AC-6's first mutation, `rounded_rect_outline -> vec![]`. No
    // outline, so `dashed_line` emits nothing at all.
    assert!(
        !separate_dashes_problems(&[]).is_empty(),
        "a drop zone that paints no boundary at all must fail AC-1; it did not, which means \
         `replace rounded_rect_outline -> Vec<Pos2> with vec![]` would survive this suite"
    );
    assert!(
        !inset_problems(&[], &zone).is_empty(),
        "a drop zone that paints no boundary at all must fail AC-2 rather than pass it vacuously"
    );
    assert!(
        !boundary_problems(&[], &zone).is_empty(),
        "a drop zone that paints no boundary at all must fail AC-3 rather than pass it vacuously. \
         AC-3 reads \"when every dash endpoint is tested\", and over an empty set that is true \
         having tested nothing - so without the guard in `boundary_problems` the test named for \
         it reports success on a border that has been deleted outright"
    );
    assert!(
        !gap_problems(&[]).is_empty(),
        "a drop zone that paints no boundary at all must fail AC-5 rather than pass it vacuously"
    );
}
