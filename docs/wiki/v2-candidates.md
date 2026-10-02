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
| After MC-048, is a reader-agnostic rule needed to remove the reader's own header and footer? (bar: `EPIC-07`'s containment + absence, not MC-019's window) | `reader-furniture-search.md` (MC-050) | not on the tuning set. MC-048's crop alone has **0 clips on 21 of 21 and no reader furniture on 19 of 21**, which meets the bar. The 2 misses are both `kunmanga`'s same-tone header, and cutting it clips `Screenshot (67).png`'s mark by 3 rows. No rule family was scored (AC-2 early stop), so this is **not** evidence that a furniture rule exists. The open test is the held-out score |
| Does the crop meet `EPIC-07`'s bar on the held-out set? (containment + absence, 0 clips and at least 18 of 19) | `held-out-score.md` (MC-051) | **no — 16 of 19, with 0 clips on 19 of 19**, at `d2876f5` (2026-09-29, the one held-out run, furniture ruled blind by the user). The misses are `kunmanga`'s same-tone header, a one-row `rolia-scans` header, and a flagged `xbato` page. **Held-out is spent**: any later run over those 26 entries is a re-score, never `EPIC-07`'s held-out score, and a fresh score needs new screenshots |
| Does the crop after MC-065 to MC-070 meet `EPIC-07`'s bar on 15 fresh screenshots from four known readers? (containment + absence, 0 clips and at least 14 of 15; the scrollbar counts as browser furniture) | `held-out-score.md` (MC-071) | **no: 2 clips of 15, and 11 of 15 meet the bar**, at `4539e12` (2026-10-02, the one run on MC-068's draw, with the bottom bars ruled blind where a mark stopped short). Both clips are on the **column axis**: `n02` is cut on both sides by a too-narrow, full-height crop, and `n05` loses one column on the left. The other two misses keep browser chrome (`n06` full height, `n13` 93 rows). The viewport stage declined on 3 of 15. **Four known readers, mid-chapter, dark pages only**: the set holds no unseen reader, no screenshot that should be left alone, no light page and no `w-network` page. **Spent**: any later run over the 15 is a re-score. The four failures go to `tuning` in a follow-up chore |
| Does the same crop meet `EPIC-07`'s bar on 25 fresh screenshots from known readers? (containment + absence, 0 clips and at least 23 of 25; the browser's horizontal scrollbar counts as browser furniture) | `held-out-score.md` (MC-063) | **no: 3 clips of 25, and 21 of 25 meet the bar**, at `1438b2b` (the same detector as `d2876f5`; 2026-09-30, the one run on MC-062's fresh draw, with the bottom bars ruled blind where a mark stopped short). All 3 clips are on the **column axis**: two right-edge cuts of 2 and 6 columns, and one crop of a different screen region. There are no row clips. The fourth miss is a crop too wide that keeps the scrollbar. **Known readers only**: the set holds no unseen reader and no screenshot that should be left alone. **Spent**: any later run over the 25 is a re-score. The four failures go to `tuning` in MC-064 and are fixed in MC-065 |
| With `kunmanga` withheld from fitting, does a reader-agnostic rule remove its same-tone site header without clipping? (0 clips on 28 and at least 26 of 28 furniture-free, in-sample and summed over leave-one-reader-out folds) | `site-header-search.md` (MC-059) | **no.** Two families (a full-width line above the art, 72 members; a band of small marks ending at the art, 54). In the `kunmanga` fold, **0** tied-best members of either family beat today's crop on `kunmanga`. In-sample, A never fires (the menu line reads as art to a row-share test), and B clears 3 of 4 headers (27 of 28) but always cuts `2025-08-05 00_11_13.webp` by 1 row. Even with that row read as border, the fold still fails. Today's crop: 24 of 28; ceiling 28 of 28 after MC-060 corrected `(67)`'s mark. The tuning set has one same-tone-header reader, so the next step needs **new captures of such sites**, not a wider sweep |

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
