# Components

The inventory of the single window, each component's states, and the
window-level states that compose them. Every visible string is named by a
key; the frozen text for every key is the table in `voice.md`, and the text
is repeated here so this file reads on its own. Tests assert the strings
exactly. Tokens are from `tokens.md`.

Conventions:

- **n/a** in a state cell means the component is not interactive and renders
  its default in that state. A test may assert that it does not change.
- "Shown" means present in the accessibility tree with the given text.
  "Hidden" means not painted and not in the tree.

## Window-level states

`AppState` (`crates/app/src/lib.rs`) has four variants; the window has seven
looks because Idle splits on whether a folder is known and drop-hover
overlays any of them.

| Window state | `AppState` | Drop zone | Folder row | Status slot | Flagged list |
|---|---|---|---|---|---|
| idle, no folder | `Idle`, `output_dir == None` | `dropzone.no_folder` | `folder.none` + button enabled | empty | hidden |
| idle, folder chosen | `Idle`, `output_dir == Some` | `dropzone.ready` | path + button enabled | empty | hidden |
| drop-hover | any, `hovered_files` non-empty | hover look; text `dropzone.hover` if a folder is chosen, else `dropzone.no_folder` | unchanged | unchanged | unchanged |
| processing | `Processing{done,total}` | `dropzone.busy`, disabled look | path + button **disabled** | progress bar + `progress.count` | hidden |
| done | `Done{summary}`, no flagged, no failed | `dropzone.ready` | path + button enabled | `result.line` | hidden (empty) |
| done with flags | `Done{summary}`, any flagged or failed | `dropzone.ready` | path + button enabled | `result.line` (+ `result.failed_suffix`) | one row per flagged or failed file |
| error | `Error{message}` | `dropzone.ready` or `dropzone.no_folder` by folder | path or `folder.none` + button enabled | message in `danger` | hidden |

Rules the view-model owns (they are what `AppState::handle` tests assert):

- `FilesDropped` with `output_dir == None` moves to `Error{error.no_folder}`;
  nothing is processed, nothing is written.
- `FilesDropped` while `Processing` is **ignored**: no state change, no
  message. The drop zone's disabled look and `dropzone.busy` text are the
  only signal. (Queueing is out of scope for v1; if the PO wants it, it is
  a story.)
- `FolderChosen` from any non-processing state keeps that state's slot
  contents (a result line survives choosing a new folder; an error is
  cleared to Idle, since the folder was the usual cause).
- `Finished` replaces any earlier summary; `Failed{message}` moves to
  `Error{error.run_failed}` with the engine's detail interpolated.
- The output folder shown is whatever `Settings.output_dir` resolved to,
  including the Send-to fallback folder (`architecture.md`, decision 6), so
  the user always sees where files went.

## Drop zone

A rounded (`radius-zone`) rectangle, `size-dropzone` tall, full width,
one centred line of `body` text. Not interactive: no click, no focus; the
whole window accepts the drop (`layout.md`). It is a status display shaped
like a target.

| State | Look | Text |
|---|---|---|
| default (no folder) | fill `surface-raised`, dashed 1 px `border-strong`, text `text-muted` | `dropzone.no_folder`: **Choose an output folder, then drop images here** |
| default (folder chosen) | same | `dropzone.ready`: **Drop images here** |
| hover (mouse only) | n/a - identical to default | unchanged |
| focus-visible | n/a - never focusable | unchanged |
| active | n/a | unchanged |
| drop-hover (files dragged over the window) | fill `drop-hover-fill`, solid 2 px `primary` border, text `text` | `dropzone.hover`: **Release to crop** when a folder is chosen; `dropzone.no_folder` when none |
| disabled (processing) | fill `surface`, dashed 1 px `border`, text `text-disabled` | `dropzone.busy`: **Cropping…** |
| loading | = disabled | |
| empty | = default (no folder); this **is** the window's empty state and its text is the instruction | |
| error | n/a - errors show in the status slot, the zone returns to its default for the current folder | |

The empty state tells the user the two things they must do, in order. It is
the only instruction in the product.

## Output folder row

Two widgets on one 32 px row.

### Path label

`body` text, single line, left-truncated (`layout.md`). Not interactive.

| State | Look | Text |
|---|---|---|
| default (folder chosen) | `text` | the full path as Windows displays it, e.g. `C:\Users\ryan\Pictures\cropped`; accessible name = full path |
| empty (no folder) | `text-muted` | `folder.none`: **No output folder chosen** |
| hover / focus-visible / active | n/a | |
| disabled (processing) | `text-disabled` | unchanged |
| loading | = disabled | |
| error | n/a | |

### "Choose folder…" button

`egui::Button`, `button` text, `size-control` tall, right-aligned, minimum
width 120. The only focusable widget in the window. On activation it opens
`rfd::FileDialog::new().set_title(folder.dialog_title).pick_folder()`,
starting in the current output folder when one is known. The dialog is
modal and native; Cancel raises no event, a choice raises
`Event::FolderChosen(path)`.

| State | Fill | Boundary | Text |
|---|---|---|---|
| default | `control` | 1 px `border-strong`, `radius-control` | `folder.button`: **Choose folder…** in `text` |
| hover | `control-hover` | same | same |
| focus-visible | as default (or hover) | plus the 2 px `focus` ring at 2 px offset, `radius-focus` (`accessibility.md`) | same |
| active (pressed) | `control-active` | same | same |
| disabled (processing) | `control-disabled` | 1 px `border` | `text-disabled`; clicks and Enter/Space do nothing; not in the Tab order |
| loading | n/a - the dialog is modal; the window does not paint until it closes | |
| empty | n/a | | |
| error | n/a - a folder that cannot be written is reported in the status slot at drop time, not on the button | | |

Accessible: role Button, name `Choose folder…`. Activates on click, Enter
and Space.

## Status slot

A 40 px region that shows exactly one of: nothing, the progress bar, the
result line, an error message. Not interactive.

### Progress bar

`egui::ProgressBar`, `size-progress` (4 px) tall, full width, no percentage
text, `animate(false)`, with a `small` count label beneath it.

| State | Look | Text |
|---|---|---|
| default (processing) | track `border`, fill `primary` from the left, width `done / total`; `radius-control` ends | `progress.count`: **{done} of {total}**, `text-muted`, e.g. `3 of 10`; `0 of 10` on the first frame |
| hover / focus-visible / active | n/a | |
| disabled | n/a | |
| loading | this component is the loading state | |
| empty | hidden (any state other than Processing) | |
| error | hidden; the error message takes the slot | |

Accessible: role ProgressIndicator with numeric value `done`, min 0, max
`total`, name `progress.count`. If the egui version does not expose the
value, the count label is the tested surface.

### Result line

`body` text, `text`, one `egui::Label`.

| State | Text |
|---|---|
| default (done) | `result.line`: **{cropped} cropped, {flagged} flagged**, e.g. `12 cropped, 0 flagged`; when any file failed, append `result.failed_suffix`: **, {failed} failed**, e.g. `10 cropped, 1 flagged, 1 failed` |
| empty | hidden (Idle, Processing, Error) |
| hover / focus-visible / active / disabled / loading / error | n/a |

The counts are always all shown, including zeros, except `failed`, which
appears only when non-zero. Digits, no words for numbers, no plural forms
(the adjectives do not inflect).

### Message line

`body` text in `danger`, one `egui::Label`, up to two lines.

| State | Text |
|---|---|
| error | one of `error.no_folder`, `error.folder_unwritable`, `error.run_failed` (`voice.md`) |
| empty | hidden (every non-error state) |
| everything else | n/a |

The message is a full sentence pair; colour is not its only carrier.

## Flagged list

`egui::ScrollArea::vertical()` filling the remaining height, one
`egui::Label` per row, `size-row` tall, `body`, not selectable, not
clickable. Rows are in `RunSummary.results` order, containing every result
whose outcome is `Flagged` or `Failed`.

| State | Look | Text |
|---|---|---|
| default | filename in `text`, then ` — ` (space, em dash, space), then the reason word in `text-muted`, built as one `LayoutJob` so the row is one accessible node | `list.row`: **{filename} — {reason}**, e.g. `IMG_0412.png — Blank`; `{filename}` is the file name component of the input path, not the full path; `{reason}` is the `reason.*` word from `voice.md` |
| overflow | rows beyond the height scroll; the window does not grow (`layout.md`); a filename wider than the row truncates at the end with `…` and the reason word stays visible | |
| hover / focus-visible / active | n/a | |
| disabled | n/a | |
| loading | hidden while Processing | |
| empty | hidden (paints nothing) in Idle, Processing, Error, and Done with zero flagged and zero failed | |
| error | n/a | |

Accessible: each row is a Label whose name is the full row text, so a test
can query `IMG_0412.png — Blank` and find exactly one node. There is no
list header; the result line above already gives the count.

## Native folder dialog

Not painted by egui. `rfd` opens the Windows `IFileDialog` folder picker
with title `folder.dialog_title`: **Choose output folder**. Nothing else in
the dialog is ours to specify.
