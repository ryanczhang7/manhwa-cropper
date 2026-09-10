# Layout

One window, one column, four stacked regions. Nothing else ever opens
except the native folder dialog.

## Window

| Property | Value |
|---|---|
| Title | `Manhwa Cropper` (`window.title`) |
| Default inner size | 520 x 440 logical px (`ViewportBuilder::with_inner_size`) |
| Minimum inner size | 400 x 320 logical px (`with_min_inner_size`); the OS refuses smaller |
| Maximum | none; resizable in both axes |
| Decorations | native Windows title bar and frame; no custom chrome |
| Position | OS default on every launch; not remembered (eframe persistence off - the only files the app writes are the output folder's contents and `settings.json`, per `architecture.md`) |
| Icon | none in v1 (the exe's default) |
| Scaling | egui points; Windows display scaling (100-200 %) applies on top and is not compensated for |

## The column

Top to bottom, inside `space-window` (16 px) padding, separated by
`space-stack` (8 px). Every region is the full inner width.

| # | Region | Height | Grows with the window? |
|---|---|---|---|
| 1 | Drop zone | `size-dropzone` = 120 | width only |
| 2 | Output folder row | `size-control` = 32 | width only |
| 3 | Status slot | `size-status` = 40 | width only |
| 4 | Flagged list | remaining height | width and height |

At the default size the list is 440 - 16 - 120 - 8 - 32 - 8 - 40 - 8 - 16 =
192 px, which is 8 rows. At the minimum size it is 72 px, 3 rows.

### Output folder row

A horizontal row: the path label takes all width not needed by the button;
the button is right-aligned at its natural width (text plus 2 x 12 px
padding, never narrower than 120 px). The path label truncates from the
**left** with a single `…` when the path is wider than its space, so the
innermost folder stays visible: `…\Pictures\cropped`. The full path is the
label's accessible name.

### Status slot

A fixed 40 px slot so the list below never jumps when a run starts or
ends. Content is vertically centred and left-aligned. When it holds the
progress bar, the bar (4 px, full width) sits above the count label
(`small`), 4 px apart. When it holds text (result line or error message),
the text may wrap to two lines; anything beyond is truncated with `…`.

### Flagged list

`egui::ScrollArea::vertical()` with `auto_shrink([false, false])`, filling
the remaining height, `size-row` (24 px) per row, egui's default scrollbar.
When the rows exceed the height the region scrolls; **the window never
grows to fit**. When the list is empty the region is present but paints
nothing (no border, no placeholder), so the window looks the same in Idle
as in Done with nothing flagged.

## Reflow

There are no breakpoints. Between the minimum and any larger size the only
things that change are the width of every region and the height of the
list. Nothing hides, reorders or collapses. Below 400 x 320 the OS does not
let the window go, so no size exists at which a control is unreachable.

## Drop target

The whole window is the drop target, not only the drop zone; the zone is
the visual affordance. `ctx.input(|i| i.raw.hovered_files)` non-empty puts
the window in drop-hover; `dropped_files` non-empty raises
`Event::FilesDropped` with every dropped path, in the order Windows
supplies them.
