# Deferred to v2

What v1 gave up, what it would take to get it back, and — the part that saves
the most time — **what has already been measured and must not be re-run**.

> **v2 has since been planned: [`EPIC-07`](../backlog/epics/EPIC-07.md).** Read
> this file for what is already answered, then that epic for what is being
> done about it. Two things it settles that this file predates: the corpus
> marks stay **tight** (MC-031's Option C is not taken), and chrome removal is
> **not** shippable as its own criterion (Option B was put to the user and
> declined, because the site's own navigation must go too).

This file exists because six stories measured one question and all six answered
it the same way. Their documents are scattered across the wiki under names that
do not obviously belong together, and someone opening v2 cold would re-run them.

## The one thing v1 gave up

**Row-axis accuracy.** `architecture.md` **decision 14**, the user's decision of
2026-09-17, is what owns this; read it before anything here.

| Axis | v1 behaviour |
|---|---|
| column (left, right) | **20 of 21** corpus entries inside MC-019's 11 px window |
| row (top, bottom) | **0 of 21**; overshoots every edge outward by **97 to 310 px**, *never clipping* |

The row behaviour is loose but safe, and safe is the part that matters: a
leftover border is the brief's accepted cost (decision 5), while a clip is what
decision 13 ranks above every other defect. v1 ships a wide crop rather than a
wrong one.

## What the user deferred, on 2026-09-17

- **More data.** The corpus is 21 marked entries and 7 flag entries
  (`fixtures/corpus/`, `corpus.md`). Every number below is tuned against, and
  scored on, those same 21 — so they are optimistic, and a larger corpus is the
  first thing any v2 attempt needs.
- **New signal classes.** Both of the obvious ones are excluded from v1 rather
  than untried: **learning- or model-based detection**, which EPIC-05 rules out
  by name, and **colour and chroma**, ruled out in MC-025 `## Context`.

## Already measured — do not re-run these

Every one is a *reasoned negative* with a reproducible harness, not an
abandoned attempt. The bar throughout is MC-019's: **20 of 21 inside an 11 px
window, with zero clips.**

| Asked | Where | Answer |
|---|---|---|
| Is a panel gutter locatable from pixels? | `panel-gutter-search.md` (MC-028) | no |
| Are the page row edges locatable from full-width chrome? | `chrome-row-search.md` (MC-031) | no |
| Are they locatable from the panel's own art edge? | `panel-edge-search.md` (MC-034) | no — two families, 780 parameterisations, best **8 of 21 with 9 clips**, per-file ceiling **10 of 21** |
| Were those numbers unfairly low, given the gutter-crossing band? | `gutter-band-rescore.md` (MC-035) | no — 387 clipped scorings retire, and **no member of either family is clip-free at all** (0 of 780), at the ruled band width or twice it |
| Would splitting the bar by edge help? | MC-032 `## Closed` | no — best single rule **14 of 21** top edges *and it clips*; **0 of 780** avoid a top clip |
| Does 2D region structure locate the edge where 1D projections cannot? | `region-row-search.md` (MC-038) | no — two families, 1120 parameterisations, best **8 of 21 with 6 clips**, ceiling **10 of 21**. The 2D property is real (it separates a bubble row from a marked edge on `2708` and `2630` where `cover(y)` cannot) and too rare to build on: **8 of 42 edges against `cover(y)`'s 6 of 42** |

Two instruments were also tried and abandoned, and both are worth knowing about
because they look attractive from a standing start:

- **Flatness of a row** as the gutter discriminator. MC-028, MC-031 and MC-032
  all asked it and all failed, for one reason: a speech bubble makes a row
  non-flat, so a gutter containing one reads as content. MC-034 replaced it with
  *how much of the row's width is not background*, which is the right question
  and still does not reach the bar.
- **MC-032's own flatness partition** ("5 of 21 entries are bounded by a flat
  gutter on both sides"). MC-034 section 2d re-implemented it from its own
  description and got different numbers. It is a property of an instrument, not
  of the corpus; **no argument should rest on it**, and MC-032 says so itself.
- **Connected components of the *ink*.** MC-038 section 3a: the ink of a comic
  page percolates, so a page, its browser chrome and everything below it come
  back as one component — 390,035 px spanning rows 133…1391 on
  `Screenshot (2630)`. Its ceiling is 6 of 21, below every 1D family. Label the
  **background** instead, or put a morphological opening in front of the
  labelling; MC-038 did the former and did not try the latter.

## What a v2 attempt has to beat

Not the 8 of 21. **Zero clips at 20 of 21**, on a corpus bigger than the one
every number above was tuned on. The shape of the v1 failure is the useful
part: the rules that rarely clip are the ones that overshoot so far they place
nothing, and the rules that place anything clip repeatedly. There is no middle,
and no band width creates one. A v2 signal has to break that trade-off, not
move along it.

Reopening means editing `architecture.md` decision 14 first, as that decision
says.
