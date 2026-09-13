---
id: EPIC-06
title: The window's look and its numbers are pinned by tests
status: todo
stories: [MC-021, MC-022, MC-023, MC-024]
---

## Goal

Every rule in `docs/wiki/design/components.md` that is *only* a fill, a
stroke, a text colour or an offset is verified by a test that fails when it
breaks, and the two numbers the window computes for itself - the progress
bar's fill fraction and the count a real run reports - are compared as values
rather than trusted. When this epic is done, changing the drop zone's border
to nothing, the enabled button's caption to grey, or the progress fraction to
a constant makes the suite go red.

## Why now

EPIC-04 delivered the window and its goal is met; this epic is about what the
tests of that window could not see. The gap is measured, not suspected:
`docs/wiki/audits/app-window-2026-09-13.md` ran 151 mutants over
`crates/app/src/` and **76 survived** - 72 of them in `gui.rs` - while
`lib.rs` killed every viable mutant it was given. The cause is structural
rather than careless: MC-015's test surface is the AccessKit tree, which
carries text and roles but no colour and no geometry, so a painter mutation
is invisible to every assertion that exists. Closing it needs image snapshot
tests on a GPU renderer feature, which MC-015 put out of scope by name.

The audit's `## Decided` section is settled and these stories implement it
without reopening it. Its `## Evidence` is not settled: a story depending on
a number there verifies it first.

## Done when

The ~55 look-only survivors in `gui.rs` are killed by image snapshots that a
**required** gate runs; the truncation functions are observable without a
renderer or the cluster has been handed to the snapshot story deliberately;
the progress fraction lives in `lib.rs` where a unit test can read it; and a
real `ThreadRunner` run is observed reporting a `Progress` count rather than
only its final state.

## Stories

1. MC-021 - The window's painted look is pinned by image snapshots (the ~55 look-only survivors)
2. MC-022 - The path label and the flagged row show the text they truncated (the eleven truncation survivors)
3. MC-023 - The progress bar's fill fraction is the view-model's number
4. MC-024 - A real run reports its progress counts

## Deliberately not in this epic

`crates/core` and `crates/engine`, which this audit did not look at and says
so - their mutation state is unknown and a separate pass is the honest way to
learn it. The three survivor groups the audit accepted as gaps rather than
defects: `RfdPicker::pick` and `main::open_window`, unreachable by
construction under any headless test, and `CropperApp::ui`, four lines of
delegation whose cost to pin is out of proportion (audit, Decided-4 and
Decided-5). Enabling the workspace-wide `mutation` gate, whose runtime was
never measured.

## Carried over, unfiled

`main.rs:93 delete field viewport` survives: with the field deleted the exe
opens untitled and default-sized, because MC-015's AC-8 pins `gui::viewport()`
as a *value* but nothing pins that `open_window` passes it to eframe. The
audit (Decided-6) judged it cheap to close - extract the `NativeOptions`
construction into a pure function - but did not file it, being one line in a
file with no other pending work. Promote it into a story if MC-021 or MC-023
ends up touching `main.rs` anyway.
