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

## Follow-up, agreed 2026-09-13

**Run `/audit-mutations` once MC-016 closes this epic**, and point it first at
the window's look-only rules. MC-015's test surface is the AccessKit tree,
which carries text and roles but no colour, so every rule in
`components.md` that is *only* a fill, a stroke or a text colour - the drop
zone's `drop-hover-fill` and 2 px `primary` border, the disabled greys, the
focus ring - is pinned by nothing. That is not a suspicion: MC-015's `## Notes`
records a mutation that makes the drop zone paint the hover look during a run
and leaves all 15 of its tests green.

Deferred rather than done inside MC-015 because closing it means image
snapshot tests, which need a GPU renderer feature and are out of MC-015's
scope by its own `## Out of scope`. The user's decision was to finish the epic
first; this note exists so the next agent does not have to rediscover the gap.

**Done, 2026-09-13.** The audit ran and is at
`docs/wiki/audits/app-window-2026-09-13.md`: 151 mutants over
`crates/app/src/`, 76 survived, 72 of them in `gui.rs`, while `lib.rs` killed
every viable mutant. The premise above was confirmed rather than merely
restated. The work it found is **EPIC-06**, not this epic - MC-021 to MC-024
live there, and this epic stays closed on the goal it delivered.
