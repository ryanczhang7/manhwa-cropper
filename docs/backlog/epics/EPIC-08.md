---
id: EPIC-08
title: The crop's sides hug the artwork
status: planned
stories: [MC-049]
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
aggregate run (MC-049 Open question 4). Where the page's textured region runs a
few columns past the mark, those columns may remain unless the user rules
otherwise.

## Stories

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
