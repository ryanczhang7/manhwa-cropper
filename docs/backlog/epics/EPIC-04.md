---
id: EPIC-04
title: One small window: drop, pick a folder, watch it finish
status: todo
stories: [MC-014, MC-015, MC-016]
---

## Goal

The first-run flow from the brief: launch, see a drop zone, pick an output
folder once, drag screenshots on, watch a progress bar, read "N cropped, M
flagged" and the flagged names. Launched from Send-to, the window opens
already working and ends on the summary. It follows the system light/dark
theme and looks like it belongs on Windows 11.

## Why now

The engine is complete after EPIC-03; the window is a view over it. Doing the
view-model as its own story first keeps every state transition testable
without egui, and the egui story then only has to paint and wire.

## Done when

Running the exe with no arguments opens the window designed in
`docs/wiki/design/`; dropping files onto it crops them into the chosen folder
and shows the summary; running it with file arguments does the same without
a click. `egui_kittest` finds every state's text exactly as `voice.md`
freezes it.

## Stories

1. MC-014 - Window state machine drives idle, processing and done
2. MC-015 - The window shows drop zone, folder picker, progress and summary (paints every state)
3. MC-016 - The window wires folder picker, drops and launch arguments to the engine

## Deliberately not in this epic

Thumbnails, preview or adjustment, a settings pane, theme customisation,
cancelling a run in progress, a system tray icon, and remembering window size
or position.
