# Voice

## Tone

Utilitarian and quiet. The window is a tool the user glances at, not a page
they read. When a run works, the only thing they should register is a
number. Rules:

- Say what is true now, in as few words as fit. No greetings, no praise, no
  "successfully".
- One instruction exists in the whole product (the empty drop zone). It
  reads as a sentence fragment, not a tip.
- No exclamation marks anywhere. No emoji, no icons standing in for words.
- No tooltips. Every string is visible or it does not exist.
- Numbers are digits. Nouns after a count do not inflect (`1 cropped`,
  `0 flagged`), which is why the counts use adjectives.
- Labels and short states have no trailing full stop. Error messages are
  sentences and do.
- The ellipsis is the single character `…` (U+2026), never three dots. The
  list separator is ` — ` (space, em dash U+2014, space).
- Paths are shown as Windows shows them, backslashes and all, never quoted.

## Error-message style

Two sentences, in this order, each one clause:

1. **What happened**, stated plainly, naming the thing involved (the path,
   the count) when there is one.
2. **What to do next**, as an imperative.

No apologies, no "oops", no error codes, no stack traces. The message is
the whole explanation; nothing is logged elsewhere.

## Flag reasons

The word the user sees for each reason in `architecture.md`. Each is at
most two words and describes the picture, not the algorithm.

| Engine value | Key | Shown |
|---|---|---|
| `Flag::Detector(FlagReason::Uniform)` | `reason.uniform` | `Blank` |
| `Flag::Detector(FlagReason::NoBorderFound)` | `reason.no_border` | `No border` |
| `Flag::Detector(FlagReason::LowContent)` | `reason.low_content` | `Low content` |
| `Flag::Detector(FlagReason::Ambiguous)` | `reason.ambiguous` | `Ambiguous` |
| `Flag::Unsupported` | `reason.unsupported` | `Unsupported format` |
| `Flag::DecodeFailed(_)` | `reason.decode_failed` | `Unreadable` |
| `Outcome::Failed { .. }` | `reason.failed` | `Not written` |

`Blank`, `No border`, `Low content`, `Ambiguous`, `Unsupported format` and
`Unreadable` files were copied unchanged to the output folder. `Not
written` files were not; that word is the only place the distinction shows,
and the result line's `failed` count backs it up.

## The frozen string table

Every string the window can show. Keys are the names the view-model and the
tests use; `{placeholders}` are filled at runtime, everything else is
literal. Changing a string means changing this table and every test that
cites it, under a story.

| Key | Text |
|---|---|
| `window.title` | `Manhwa Cropper` |
| `dropzone.no_folder` | `Choose an output folder, then drop images here` |
| `dropzone.ready` | `Drop images here` |
| `dropzone.hover` | `Release to crop` |
| `dropzone.busy` | `Cropping…` |
| `folder.none` | `No output folder chosen` |
| `folder.button` | `Choose folder…` |
| `folder.dialog_title` | `Choose output folder` |
| `progress.count` | `{done} of {total}` |
| `result.line` | `{cropped} cropped, {flagged} flagged` |
| `result.failed_suffix` | `, {failed} failed` |
| `list.row` | `{filename} — {reason}` |
| `reason.uniform` | `Blank` |
| `reason.no_border` | `No border` |
| `reason.low_content` | `Low content` |
| `reason.ambiguous` | `Ambiguous` |
| `reason.unsupported` | `Unsupported format` |
| `reason.decode_failed` | `Unreadable` |
| `reason.failed` | `Not written` |
| `error.no_folder` | `No output folder chosen. Choose one, then drop the files again.` |
| `error.folder_unwritable` | `Cannot write to {path}. Choose a different output folder.` | Reserved: no producer in v1 (PO decision 2026-09-10, see MC-014 Notes); an unwritable folder surfaces as `error.run_failed`. |
| `error.run_failed` | `Cropping stopped: {detail}. Drop the files again.` |

Placeholder rules:

- `{done}`, `{total}`, `{cropped}`, `{flagged}`, `{failed}`: unsigned
  integers as digits, no separators.
- `{filename}`: the final path component of the input file, with its
  extension, exactly as on disk.
- `{reason}`: one of the seven `reason.*` values.
- `{path}`: the output folder as a full Windows path.
- `{detail}`: the engine's error string with any trailing full stop
  removed, otherwise verbatim.
- `result.failed_suffix` is appended to `result.line` only when `{failed}`
  is greater than zero: `12 cropped, 0 flagged` and
  `10 cropped, 1 flagged, 1 failed` are both valid; `12 cropped, 0 flagged,
  0 failed` is not.

Worked examples a test can copy:

| Situation | Exact text |
|---|---|
| Fresh install, first launch | drop zone `Choose an output folder, then drop images here`; path label `No output folder chosen` |
| Third of ten files done | `3 of 10` |
| Ten files, all cropped | `10 cropped, 0 flagged` |
| Ten files, eight cropped, two flagged | `8 cropped, 2 flagged` |
| Ten files: 7 cropped, 2 flagged, 1 not written | `7 cropped, 2 flagged, 1 failed` |
| A flagged all-art screenshot `page_03.png` | row `page_03.png — No border` |
| A dropped `notes.txt` | row `notes.txt — Unsupported format` |
| A truncated JPEG `shot.jpg` | row `shot.jpg — Unreadable` |
| Output disk full for `shot.jpg` | row `shot.jpg — Not written` and the line ends `, 1 failed` |
| Drop before choosing a folder | `No output folder chosen. Choose one, then drop the files again.` |
| Folder on an unplugged drive `E:\cropped` | `Cannot write to E:\cropped. Choose a different output folder.` |
