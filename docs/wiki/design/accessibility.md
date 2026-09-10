# Accessibility floor

Written once so tests can assert it. Every user-facing story in the app
crate is measured against this file.

## Contrast

Computed with the WCAG 2.x relative-luminance formula against the surfaces
the text or boundary actually sits on. Floor: 4.5:1 for text, 3:1 for the
boundary of an interactive control and for large text. Nothing in the
window is large text, so 4.5:1 applies to every string.

| Pair | Light | Dark | Floor |
|---|---|---|---|
| `text` on `surface` | 15.52 | 16.29 | 4.5 |
| `text-muted` on `surface` | 5.93 | 9.44 | 4.5 |
| `text-muted` on `surface-raised` (drop zone, list) | 6.36 | 8.20 | 4.5 |
| `text` on `drop-hover-fill` | 14.58 | 13.26 | 4.5 |
| `text` on `control-hover` | 15.11 | 11.73 | 4.5 |
| `text` on `control-active` | 13.67 | 14.94 | 4.5 |
| `danger` on `surface` (error message) | 5.10 | 8.03 | 4.5 |
| `border-strong` on `surface` (button boundary) | 4.09 | 6.08 | 3 |
| `border-strong` on `surface-raised` | 4.39 | 5.14 | 3 |
| `primary` on `surface` (focus ring, progress fill) | 5.11 | 8.12 | 3 |
| `primary` on `drop-hover-fill` (drop-hover border) | 4.80 | 6.61 | 3 |
| `primary-contrast` on `primary` (reserved) | 5.67 | 10.47 | 4.5 |
| `warning` on `surface` (reserved) | 4.73 | 12.34 | 4.5 |
| `success` on `surface` (reserved) | 4.90 | 8.03 | 4.5 |

`text-disabled` is exempt (WCAG exempts inactive controls) and is paired
with a control that also ignores input, so the state is not carried by
colour alone.

Changing any value in `tokens.md` means recomputing this table; a pair that
drops below its floor is a design defect, not a tuning choice.

## Focus

- Exactly one widget takes focus in v1: the `Choose folder…` button. When
  `response.has_focus()` is true, `gui.rs` paints a 2 px `focus` ring at the
  button rect expanded by 2 px on every side, corner radius `radius-focus`
  (6), on top of whatever fill the button has. The ring's contrast is the
  `primary` on `surface` row above.
- egui does not distinguish keyboard focus from click focus. The ring shows
  whenever the button holds egui focus; a ring after a mouse click is
  accepted, as Windows itself does for several controls.
- Focus is never trapped. The native folder dialog is modal, owned by the
  window, and returns focus to the button when it closes.
- Nothing has focus on launch. The first Tab focuses the button.

## Keyboard path

Everything that can be done with a mouse can be done from the keyboard,
and files have a keyboard route that does not need the window at all.

| Action | Keys |
|---|---|
| Reach the folder button | `Tab` (and `Shift+Tab`; it is the only stop, so both land on it) |
| Open the folder dialog | `Enter` or `Space` while the button has focus |
| Inside the dialog | Windows' own keyboard handling; `Esc` cancels with no event |
| Give the app files | Explorer: select files, `Shift+F10` or the menu key, "Send to", Manhwa Cropper. This is the keyboard route and the tenth-run route in the brief; the drop zone is never the only way in |
| Scroll the flagged list | mouse wheel over the list; `Down`/`Up` by one row (24 px), `PageDown`/`PageUp` by the visible height, `Home`/`End` to the ends, whenever the button does not have focus. `gui.rs` reads these from `ctx.input` and sets the `ScrollArea` offset |
| Close | `Alt+F4` (native) |

The processing state removes the button from the Tab order (it is
disabled); no other control exists, so Tab does nothing until the run ends.

## Targets

| Control | Size |
|---|---|
| `Choose folder…` button | 32 px tall, at least 120 px wide (floor: 24 px) |
| Drop target | the whole window, at least 400 x 320 |
| List rows | not interactive; 24 px is for reading rhythm, not hit testing |

No touch target guidance applies; the brief's platform is a desktop PC.

## Names and the accessibility tree

eframe enables AccessKit (egui 0.32 default feature). The tree must
contain, with these roles and names, so that `egui_kittest` can query them
and Narrator can read them:

| Node | Role | Name |
|---|---|---|
| window | Window | `Manhwa Cropper` |
| drop zone text | Label | the current `dropzone.*` string |
| path label | Label | the full output path, or `folder.none` |
| folder button | Button | `Choose folder…` |
| progress bar | ProgressIndicator | `progress.count`; numeric value `done`, max `total`, when egui exposes it |
| progress count | Label | `progress.count` |
| result line | Label | `result.line` (+ `result.failed_suffix`) |
| message line | Label | the `error.*` string |
| each flagged row | Label | `list.row` text, e.g. `IMG_0412.png — Blank` |

A test that cannot find one of these by name fails; that is the point. No
node has a tooltip, and no name is an icon or an abbreviation.

Live updates: the result line and message line replace the slot's previous
content in one frame. AccessKit's live-region property is set to `Polite`
on those two labels where egui exposes it; where it does not, the presence
of the label by name is the tested surface.

## Colour is never the only carrier

| Meaning | Colour | Second carrier |
|---|---|---|
| files can be dropped now | `primary` border | text changes to `dropzone.hover` (when a folder is chosen) |
| an error | `danger` text | the text is a sentence saying what happened and what to do |
| a file was flagged | `text-muted` reason word | the reason is a word, and the file is listed at all |
| the button is disabled | `text-disabled` | the button ignores input and leaves the Tab order; the drop zone says `Cropping…` |
| progress | `primary` fill | `{done} of {total}` in text |

## Motion

Nothing animates except the progress fill's width, which tracks a count.
There is no transform, fade or parallax to remove under a reduced-motion
preference, so none is queried.

## Zoom

Windows display scaling at 100-200 % is supported because the layout is in
egui points and the list scrolls; at 200 % on a 1080p display the default
window still fits. Text is never rendered as an image.

## Reviewing a built screen

When the app runs: launch it, press Tab and confirm the ring; press Enter
and confirm the dialog; cancel it and confirm the state is unchanged; drag a
file over and confirm the zone changes before release; drop with no folder
and read the message; switch Windows to dark and confirm the palette; resize
to the minimum and confirm three list rows scroll. Report each miss as a
diff against `components.md` and hand code changes to the Lead PO as a
story.
