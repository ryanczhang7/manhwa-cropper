---
id: EPIC-08
title: The crop's sides hug the artwork
status: planned
stories: [MC-053, MC-049]
---

## Goal

A cropped screenshot has **no page background beside the artwork** on the left
or the right: no white band on a light reader, no black band on a dark one. The
user never sees a clipped panel — zero clips stays absolute, exactly as in
every other epic.

## Why now

**The user's words, 2026-09-23:** *"I consider this a failure currently. Not
good enough for v1. The cropping from the side isnt tight enough and I still
see the page on the right and left side."*

That reverses two recorded positions, and this epic is where the reversal
lives:

- `architecture.md` decision 5 — *"A leftover 3 px border is the brief's
  accepted cost"* — and `product-brief.md` §4 item 2's *"leaving a few pixels
  of border is acceptable"*. The user now says it is not.
- `EPIC-07`'s *"The column axis, settled at 20 of 21 and not reopened."* Twenty
  of twenty-one inside MC-019's 11 px window was a true number that concealed
  what the user sees: the window admits a crop that carries a band of page
  background, and every one of the twenty-one does — the margin puts three
  flat columns on each side of every crop.

**Why a new epic and not an amendment of `EPIC-07`.** `EPIC-07` says of itself
*"This epic changes where the crop rectangle's top and bottom edges land, and
nothing else"*, and its bar on the row axis is containment plus absence of
furniture, with the marks demoted to a never-clip oracle. The column axis is
different in kind: the marks there **are** still an accuracy target (MC-019's
window), the defect is a margin constant rather than a missing signal, and the
fix is small and measured. Folding it into `EPIC-07` would give one epic two
axes with two different bars and a scope sentence that is false. `EPIC-07`'s
"deliberately not" list gains a pointer here instead of losing its line.

**It is cheap, which is also why it is its own epic.** A probe on 2026-09-23
(MC-049 `## Notes`) found the column locator already sitting exactly on the
boundary between page background and textured page on every marked tuning
entry; the band the user sees is the 3 px margin added afterwards. The fix is
the margin and seven marks that contain page background against the marking
rule — not a new detector.

## Done when

On every marked `tuning` entry, no column of the crop outside the mark is page
background, and no crop clips its mark; the held-out set shows zero clips in one
aggregate run (MC-049 Open question 4).

*2026-09-24.* The held-out set is now 21 marked entries, because MC-053 moves
three into `tuning`. On `main` after MC-048, the aggregate run also reports
three **row** clips and two more entries not cropped. Those belong to the row
axis and to `EPIC-07`, not here. Where the page's textured region runs a few
columns past the mark, those columns may remain unless the user rules
otherwise.

*Later on 2026-09-24.* The user answered where the row clips go: *"Fix
story, top priority"*. That is [MC-052](../stories/MC-052.md), under
`EPIC-07`, and it runs **before** this epic's stories. It moves two more
held-out entries to `tuning` and corrects one held-out mark. So after MC-052
and MC-053 the held-out set has **19** marked entries. That is one below
`MIN_HELD_OUT_MARKED`, and MC-053 Open question 6 asks what to do about it.
MC-053's held-out check (its AC-5) now requires zero clips of **any** kind,
so it also holds MC-052's row bar.

## Stories

**Order: MC-052 (`EPIC-07`) → MC-053 → MC-049**, enforced by `depends_on`
(MC-053 on MC-052, MC-049 on MC-053). The chain is serial because all three
edit `fixtures/corpus/manifest.json`, `corpus.md` and the same pinned
constants. Each also changes the counts the next one pins.

- **Prerequisite, not this epic's:
  [MC-052](../stories/MC-052.md)** (`EPIC-07`), a *fix*: the viewport stage
  reads only the reader's own window. It is the user's top priority of
  2026-09-24. It removes the held-out row clips MC-048 shipped, which MC-053
  would otherwise have had to leave standing.

0. **[MC-053](../stories/MC-053.md)**, a *fix*: the page column locator keeps
   dark, low-texture art. MC-049's held-out check found it on 2026-09-24. v1
   already cut such art by 2–6 px at margin 3, and at margin 0 the cut shows at
   full depth. This story moves the three affected entries to `tuning`,
   corrects two held-out marks, and changes the page column's boundary rule
   without letting page background back in. MC-049 depends on it and is parked
   at GATES until it is DONE.
1. **[MC-049](../stories/MC-049.md)** — *fix*: the crop's side edges carry no
   page background. Margin to 0 (Open question 1), seven marks corrected with
   the user (Open question 2), zero clips.
2. *Conditional, not written:* if the user rules, after seeing MC-049's output,
   that the 1–7 textured columns outside the mark are also "page" (MC-049 Open
   question 3), a story to move the locator inward to the art's own edge. It
   would need its own evidence first; do not write it before that ruling.

## Deliberately not in this epic

- **The row axis** — `EPIC-07`, [MC-048](../stories/MC-048.md),
  [MC-050](../stories/MC-050.md).
- **`Screenshot (93).jpg`'s right edge**, the diagonal-gutter entry and
  MC-019's accepted column miss.
- **Re-marking the corpus.** Correcting a mark that contains flat page
  background against `corpus.md`'s marking rule is the MC-027 precedent and a
  per-entry ruling by the user; it is not the re-marking the user declined on
  2026-09-17, which was about the row convention.
- Any change to the column locator in MC-049.
