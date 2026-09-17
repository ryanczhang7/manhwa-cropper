---
id: EPIC-06
title: The window's look and its numbers are pinned by tests
status: done
stories: [MC-021, MC-022, MC-023, MC-024, MC-029, MC-030]
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
is invisible to every assertion that exists.

The audit's `## Decided` section is settled and these stories implement it
without reopening it. Its `## Evidence` is not settled: a story depending on
a number there verifies it first.

## The instrument changed on 2026-09-15, and the epic was re-planned around it

This epic was written believing that closing the look-only gap "needs image
snapshot tests on a GPU renderer feature, which MC-015 put out of scope by
name". **MC-022 falsified that**, and the correction is worth stating because
three stories were sized against the old belief.

There is a third surface between the accessibility tree and a rendered image:
`egui_kittest`'s `Harness::output()` returns the frame's **untessellated** paint
list, where a `Shape::Text` still carries the `Galley` egui laid out, a
`Shape::Rect` carries its `fill`, `stroke` and `corner_radius`, and a dashed
border arrives as one `Shape::LineSegment` per dash. MC-022 killed eleven
survivors through it with **no production change, no renderer, no `snapshot` or
`wgpu` feature and no reference images to review** - and the tests run under the
required `unit` gate by construction, which is what the old MC-021 needed a
whole acceptance criterion to force.

The audit's **Decided-2 stands exactly as written**: an *AccessKit* assertion
cannot kill a colour, and none is proposed. It simply did not consider a third
option. Its **E-7** - that `egui_kittest` 0.36.2 declares the `snapshot` and
`wgpu` features - was read off a manifest, nothing was ever compiled, and no
story in this epic now depends on it.

So MC-021 was re-planned from seven looks and two themes down to the colour
survivors alone, and the geometry it used to carry became MC-029 (the dashed
border) and MC-030 (the centring arithmetic). Three cycles rather than one, each
demonstrable on its own, and the audit's own priority order - colours wholesale
first, offsets of a few points last - is the order they are filed in.

## Done when

The ~55 look-only survivors in `gui.rs` are killed by tests a **required** gate
runs, or classified as equivalent with the argument written down (E-4 already
found one that no test can kill, and MC-029 works through the rest); the
truncation functions are observable without a renderer **(done: MC-022)**; the
progress fraction lives in `lib.rs` where a unit test can read it **(done:
MC-023)**; and a real `ThreadRunner` run is observed reporting a `Progress`
count rather than only its final state.

## Stories

1. MC-021 - The window's painted colours are pinned by the paint list (the palette, the enabled/disabled button, the row's text style)
2. MC-022 - The path label and the flagged row show the text they truncated (the eleven truncation survivors) - **done**
3. MC-023 - The progress bar's fill fraction is the view-model's number - **done**
4. MC-024 - A real run reports its progress counts
5. MC-029 - The drop zone's dashed border is pinned by the shape it paints (E-4's thirty, and the equivalence inspection)
6. MC-030 - The status slot and the flagged row place their text where the design says (E-3's offsets; the lowest-value cluster, and filed as such)

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
