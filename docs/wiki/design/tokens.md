# Tokens

Values for the one window Manhwa Cropper has. egui does not draw Win32
widgets; "native look" (brief, section 8) is honoured as: the native title
bar and window frame, the real Windows folder dialog via `rfd`, the system
light/dark theme followed, and Windows 11 proportions and restraint - 4 px
control radius, 32 px controls, 16 px window padding, one accent, no shadows.
Everything below is applied by `crates/app/src/gui.rs` on top of egui's
`Visuals::light()` / `Visuals::dark()`; nothing is left at egui's default
where a value is given here.

## Theme

- The window follows the system theme only: eframe `ThemePreference::System`.
  There is no toggle, no override, nothing persisted. If Windows switches
  theme while the app is open, the next frame paints in the new theme.
- Light is the reference palette. Dark overrides every role listed; no role
  exists in dark only.

## Colour roles

Hex, fully opaque. `egui field` says where the value lands so the mapping is
not re-invented per story; roles with no egui field are used directly by the
painter in `gui.rs`.

| Role | Light | Dark | egui field / use |
|---|---|---|---|
| `surface` | `#F3F3F3` | `#202020` | `visuals.panel_fill`, `visuals.window_fill`; the window background |
| `surface-raised` | `#FBFBFB` | `#2B2B2B` | `visuals.extreme_bg_color`; drop-zone fill, list background |
| `text` | `#1B1B1B` | `#FFFFFF` | `visuals.override_text_color`; all text unless a role below says otherwise |
| `text-muted` | `#5D5D5D` | `#C5C5C5` | secondary text: drop-zone text, `folder.none`, progress count, flag reason word |
| `text-disabled` | `#9B9B9B` | `#6D6D6D` | text of a disabled control |
| `border` | `#D9D9D9` | `#3F3F3F` | the progress track and the disabled button's boundary; never the boundary of an enabled control |
| `border-strong` | `#767676` | `#9E9E9E` | `widgets.inactive.bg_stroke`; boundary of every enabled control and of the drop zone |
| `primary` | `#0067C0` | `#4CC2FF` | `visuals.selection.bg_fill` (progress fill), drop-hover border, focus ring |
| `primary-contrast` | `#FFFFFF` | `#000000` | text on `primary` (reserved; no filled-primary control exists in v1) |
| `drop-hover-fill` | `#E0EEFA` | `#173247` | drop-zone fill while files are dragged over the window |
| `control` | `#FBFBFB` | `#2D2D2D` | `widgets.inactive.bg_fill`; button at rest |
| `control-hover` | `#F0F0F0` | `#383838` | `widgets.hovered.bg_fill` |
| `control-active` | `#E5E5E5` | `#272727` | `widgets.active.bg_fill`; button while pressed |
| `control-disabled` | `#F5F5F5` | `#2A2A2A` | button fill when disabled |
| `danger` | `#C42B1C` | `#FF99A4` | `visuals.error_fg_color`; error message text |
| `warning` | `#9D5D00` | `#FCE100` | `visuals.warn_fg_color`; reserved, unused in v1 |
| `success` | `#0F7B0F` | `#6CCB5F` | reserved, unused in v1 |
| `focus` | = `primary` | = `primary` | the 2 px focus ring (`accessibility.md`) |

Measured contrast for every text-on-surface and boundary pair is in
`accessibility.md`; the lowest is `border-strong` on `surface` in light at
4.09:1, above the 3:1 boundary floor.

The reason word in the flagged list is `text-muted`, not `warning`: the word
carries the meaning, and a run with twelve amber rows would shout at a user
the brief says should never need to read the window.

## Type scale

egui's bundled default proportional font (Ubuntu-Light in egui 0.32). It
ships in one weight, so the scale has no weight axis; emphasis is by size
and colour role only. Sizes are egui points (logical px; Windows display
scaling multiplies them). Line height is the font's own; row heights are
fixed by spacing below rather than by line height.

| Step | egui `TextStyle` | Size | Used for |
|---|---|---|---|
| body | `Body` | 14 | drop-zone text, folder path, result line, list rows, messages |
| button | `Button` | 14 | `folder.button` |
| small | `Small` | 12 | progress count under the bar |
| heading | `Heading` | 20 | reserved; the window has no headings in v1 |
| mono | `Monospace` | 13 | reserved; unused in v1 |

Set once at startup through `ctx.set_style` / `style.text_styles`.

## Spacing

4 px base, ramp 4 / 8 / 12 / 16 / 24 / 32. Nothing else.

| Token | Value | Where |
|---|---|---|
| `space-window` | 16 | padding inside the window on all four sides (`Frame::inner_margin`) |
| `space-stack` | 8 | vertical gap between the four stacked regions (`spacing.item_spacing.y`) |
| `space-inline` | 8 | horizontal gap between the path label and the button (`spacing.item_spacing.x`) |
| `space-control-x` | 12 | button horizontal padding (`spacing.button_padding.x`) |
| `space-control-y` | 6 | button vertical padding (`spacing.button_padding.y`); with 14 pt text this yields a 32 px tall button |
| `size-control` | 32 | height of every interactive control and of the folder row |
| `size-row` | 24 | height of one flagged-list row |
| `size-dropzone` | 120 | fixed height of the drop zone |
| `size-status` | 40 | fixed height of the status slot |
| `size-progress` | 4 | progress bar height |

## Radii, borders, elevation

| Token | Value | Where |
|---|---|---|
| `radius-control` | 4 | button, progress bar ends (`widgets.*.corner_radius`) |
| `radius-zone` | 8 | drop zone |
| `radius-focus` | 6 | focus ring (control radius + ring offset) |
| `stroke-control` | 1 px `border-strong` | button boundary; drop-zone boundary, dashed (6 px dash, 4 px gap, `Shape::dashed_line`) |
| `stroke-hover` | 2 px `primary`, solid | drop zone while files are dragged over the window |
| elevation | none | `visuals.window_shadow` and `visuals.popup_shadow` are `Shadow::NONE`; the window is one flat surface |

## Motion

The progress bar is the only thing that moves.

| Token | Value |
|---|---|
| progress fill | width = `done / total` of the track, set directly each frame; no tween, no easing, `ProgressBar::animate(false)` (no shimmer) |
| everything else | `style.animation_time = 0.0`; egui's hover and press fades are off, state changes are instant |
| reduced motion | nothing to reduce: no transform, no fade, no indeterminate spinner. The Windows "Animation effects" setting is not queried because no animation depends on it |
