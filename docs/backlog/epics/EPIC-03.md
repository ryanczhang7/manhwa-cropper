---
id: EPIC-03
title: Files go in from Explorer and come out cropped in the chosen folder
status: todo
stories: [MC-008, MC-009, MC-010, MC-020, MC-011, MC-012, MC-013]
---

## Goal

The user selects screenshots in Explorer, right-clicks, "Send to", and finds
them cropped in their output folder with the same names and formats: PNG
pixel-identical inside the crop, JPEG and WebP re-saved at maximum quality,
flagged files copied untouched, nothing ever overwritten, a summary of what
happened, and the output folder remembered from last time. All of it works
headlessly, before there is a window.

## Why now

The tenth-run flow in the brief ("Send to, no interaction") needs none of the
window. Building the engine crate to completion first gives the window
(EPIC-04) a single `batch::run` to call and keeps every file-level behaviour
testable in a temp directory.

## Done when

`manhwa-cropper.exe --no-gui a.png b.jpg c.webp` (no `--out`) writes cropped
files into the remembered folder, or into `cropped\` beside `a.png` when no
folder was ever chosen, with ` (2)` suffixes where names clash, and
`--summary s.json` records per-file outcomes; exit code 0.

## Stories

1. MC-008 - PNG is cropped losslessly at original resolution
2. MC-009 - JPEG and WebP are cropped and re-saved at maximum quality
3. MC-010 - Output names never overwrite an existing file
4. MC-020 - Output names never collide by letter case alone
5. MC-011 - A batch runs every file and reports a run summary
6. MC-012 - Command-line file paths run headlessly for Send to
7. MC-013 - The output folder is remembered between runs

MC-020 is out of numeric order on purpose: it was filed after MC-010 shipped,
and it belongs before MC-011 because MC-011 is where a batch first makes the
defect reachable. The order is enforced by `MC-011.depends_on`, not by the
numbering - see MC-020's `## Notes` for why an in-sequence id would have been
unsafe.

## Deliberately not in this epic

The window and any progress display, creating the Send-to shortcut for the
user, performance (EPIC-05), recursion into folders, and any format not named
in the brief.
