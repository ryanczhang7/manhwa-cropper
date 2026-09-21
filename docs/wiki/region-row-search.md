# Are the page row edges locatable from 2D region structure?

MC-038, a spike. The deliverable is a measured answer, not code. Nothing in
`crates/` changed, and the harness that produced every number below lives
outside the repository (section 7), as MC-025's, MC-026's, MC-028's, MC-031's
and MC-034's did.

MC-028 asked whether a **panel gutter** is locatable from pixels; MC-031 asked
the same of **full-width chrome**; MC-034 asked it of **the panel's own art
edge**. All three came back reasoned negatives, and
[`EPIC-07`](../backlog/epics/EPIC-07.md) says why in a way none of their own
conclusions did:

> each reduced the image to a **1D row statistic** [...] A speech bubble is a
> *small 2D object*; projected onto a row it contaminates the whole row, so a
> gutter containing one arc reads as content. [...] **no row statistic can work
> when the thing that distinguishes gutter from art is spatial extent**.

This document tests that claim. It is the first measurement of connected-region
structure anywhere in this project: `crates/core/src/` is `content.rs`,
`decide.rs`, `edges.rs`, `flat.rs`, `margin.rs` and `trim.rs`, and there is no
connectivity anywhere in it.

## 1. The verdict

**A reasoned negative on the bar, and a sharp answer — in both directions — on
the epic's premise.**

Two families of region rule were scored across **1120 parameterisations**. The
best single rule reaches **8 of 21 with 6 clips**. The best **clip-free** rule
reaches **0 of 21** — no member of either family is clip-free at all.
Cherry-picking the best parameterisation per file, which no single rule can do,
reaches **10 of 21**. MC-019 needs 20 of 21 with zero clips. **This does not
beat 8 of 21**; it ties it.

The epic's premise splits cleanly into a part that is **confirmed** and a part
that is **refuted**, and keeping them apart is the whole value of this spike:

- **Confirmed, in the specific.** There are rows where no per-row scalar can
  tell a marked page edge from a gutter an overhanging bubble sits in, and
  where a 2D region property can. On `Screenshot (2708)`'s top edge, MC-034's
  `cover(y)` reads **0.429** at the marked edge and **0.133** at the overhang
  row — both "content" under every `min_cover` MC-034 swept — while the region
  property reads **0.383** and **0.865**, which is the page and the gutter.
  Section 4 has the four numbers and a second entry. So the family is **not**
  MC-034's coverage family under a new name, and AC-3's control does not fire.
- **Refuted, as a route to the bar.** At corpus scale the separation is rare,
  not general. Over the 42 marked edges, **no threshold on either statistic
  separates**; at its best threshold `cover(y)` gets **6 of 42** edge/overhang
  pairs right and the region property gets **8 of 42**. Two extra pairs is the
  entire measured value of going 2D, and it is why the headline ties instead of
  moving. "No row statistic can work" is true; *and* the 2D statistic tried here
  does not work either, for reasons section 3 makes structural.

Three findings are worth carrying forward past the verdict.

| | |
|---|---|
| **The bottom-edge ceiling rises, 11 → 13 of 21.** | `## Context` finding 1 calls the bottom edge the whole deficit. It is the one yardstick this family improves, and it improves it on **different entries** from MC-034's — `13_49_39` and `Screenshot (70)` become reachable. |
| **Ink connectivity is actively harmful.** | Components of the *ink* percolate: on `Screenshot (2630)` at `dev 20` a single component of 390,035 px spans rows 133…1391, merging browser chrome, the page and everything below it. That family's ceiling is **6 of 21**, well below MC-034's 10. Section 3a. |
| **The same 8 of 21, at two-thirds of the clips.** | MC-034's coverage family reaches 8 of 21 with **9** clips; the gutter-region family reaches 8 of 21 with **6**, and its best clip count anywhere is 5 against MC-034 coverage's 9. It still does not *break* the trade-off `v2-candidates.md` names, and section 5 says so in that file's own terms. |

## 2. The denominator, and the guard (AC-1)

**Every number in this document is measured over the 28 `tuning` entries — 21
marked and 7 `"expect": "flag"`.** The 31 `held-out` entries were not scored,
not once and not "just to see". `docs/wiki/corpus.md` governs, and its rule is
that contamination runs one way.

Counted from `fixtures/corpus/manifest.json` by the harness itself:

```
manifest rows                : 59
tuning                       : 28
  marked                     : 21
  "expect": "flag"           : 7
held-out (never scored here) : 31
```

The 28, in manifest order, are the whole list — no `held-out` name appears in
it:

```
   1. 2025-02-27 22_46_15.png            flag
   2. 2025-03-03 11_06_04.png            flag
   3. 2025-03-03 11_24_19.png            flag
   4. 2025-05-12 22_55_40.png            flag
   5. 2025-05-12 22_58_53.png            flag
   6. 2025-08-05 00_11_13.webp           958 114 631 1216
   7. 2025-08-05 00_11_27.webp           1008 118 528 1225
   8. 2025-10-14 23_29_06.png            1008 188 528 1138
   9. 2025-10-14 23_30_20.png            1040 171 476 1208
  10. 2025-10-20 15_37_25.png            1008 228 528 1074
  11. 2026-01-05 13_33_41.png            1073 233 397 1060
  12. 2026-01-05 13_45_59.png            1040 204 466 1167
  13. 2026-01-05 13_49_39.png            1044 286 459 1110
  14. Screenshot (67).png                1012 290 524 1098
  15. Screenshot (70).jpg                1016 298 520 1012
  16. Screenshot (75).png                1077 171 393 1208
  17. Screenshot (93).jpg                1143 212 246 1167
  18. Screenshot (103).jpg               1077 302 389 844
  19. Screenshot (1661).png              1077 192 393 1085
  20. Screenshot (2582).jpg              1008 216 524 1008
  21. Screenshot (2630).jpg              958 224 635 889
  22. Screenshot (2698).jpg              954 343 631 742
  23. Screenshot (2708).jpg              1073 220 393 1102
  24. Screenshot (2744).jpg              950 134 639 1164
  25. Screenshot (3187).png              987 142 574 1237
  26. Screenshot (3455).png              flag
  27. Screenshot (3465).png              flag
  28. Screenshot (3538).png              975 171 590 1159

admitted entries             : 28
held-out names among them    : 0
```

### 2a. The mechanism: a guard, not a filter

`crates/engine/tests/common/corpus.rs` is not a library, so this harness cannot
import MC-037's `Split::Tuning` filter and had to reconstruct it. A filter that
is merely *applied* looks identical, from a document, to one that was
forgotten — so it was reconstructed as something that cannot be forgotten:

- the pixels are reachable only through a `corpus::Entry`;
- a `corpus::Entry` has private fields and no public constructor;
- the only thing that makes one is `corpus::admit`, which **returns
  `Err(Refusal)` for a `held-out` row before it has built the file path**.

There is no second path to a pixel. Forgetting the filter is not a mistake this
harness can make, because the type that carries a decoded image cannot be
constructed without the refusal having declined to fire.

### 2b. The guard, shown firing

```
$ cargo run --release --bin guard
Handing the harness a held-out entry: "2024-09-09 23_30_01.png"
REFUSED: "2024-09-09 23_30_01.png" is split "held-out". This harness scores the
tuning split only. docs/wiki/corpus.md: "no threshold is chosen, adjusted or
rejected on the strength of a held-out result; no per-file table of held-out
results is read while a rule is still being changed". Nothing was decoded and
nothing was scored.

per-file results produced for it: 0
scores produced for it          : 0
bytes of "2024-09-09 23_30_01.png" decoded           : 0
```

The refusal happens before the path is built, so demonstrating it cannot itself
contaminate: nothing about that entry's pixels was read in order to refuse it.

### 2c. The instrument, calibrated before it was believed

Before any new family was scored, the harness re-scored the **shipped
pipeline** and compared the result with the one `panel-edge-search.md`
section 6 records for the same baseline. If the scoring code did not reproduce
that, nothing it said about a new family would mean anything.

```
$ cargo run --release --bin base
rows inside the 11 px window : 0 of 21
clips                         : 0
top overshoot    : +97…+306
bottom overshoot : +16…+310

recorded by panel-edge-search.md section 6 for the same baseline:
  rows 0 of 21, clips 0, overshoot +97…+306 top and +16…+310 bottom
```

Exact, on all four numbers. A second calibration falls out of control (c) in
section 6: the shipped pipeline crops **1 of 7** flag entries, which is the
no-regression baseline MC-034 records.

One decoding detail the calibration turned up, recorded because the next
harness will hit it: **`2025-05-12 22_55_40.png` is a JPEG**. The shipped
pipeline sniffs the signature (`crates/engine/src/process.rs:179` says why), so
this harness does too. A harness that trusted the extension would score a
different set of files from the one the engine sees.

## 3. The two families

Both are anchored to the page column `flat::page_column` already finds (MC-027,
20 of 21 on the column axis) and replace only its **rows**, which the locator
deliberately never moves. Both scan outward from the page column's own vertical
centre and stop at the first run of `gap` non-page rows — MC-034's stopping
rule, kept identical so the families differ from its coverage family in one
thing only.

**The binarisation is MC-034's `cover(y)` predicate, pixel for pixel**: a pixel
is *ink* iff it deviates from its own row's median, over the page column, by
more than `dev`. So `cover(y)` is exactly `ink(y) / W`, and the only thing
either family adds is **connected structure over those same pixels**. Any
difference in the scores is attributable to connectivity and to nothing else.
That was the point of choosing it.

Connectivity is **8 for ink and 4 for background** — the standard pairing,
without which a set and its complement can both percolate at once.

### 3a. Family 1, ink regions — and why it fails structurally

The 2D property is a **component's bounding-box width**, as a share of the page
column's. The page's own art is a large wide region; a neighbouring panel's
overhang is a small narrow one, which is exactly what `corpus.md`'s marking rule
distinguishes.

It does not survive contact with a real page, and the reason is not tuning.
**The ink mask percolates.** 8-connected site percolation has a threshold near
0.41, and `cover(y)` on a real art row is well above that, so the ink of a page
is one region — together with anything it touches:

```
$ cargo run --release --bin profile -- "Screenshot (2630).jpg" ink 20 256 0.30 8
448 components at dev 20; the 12 largest by area:
     area      w    w/W     y0     y1      h
   390035    636   1.00    133   1391   1259      <- chrome + page + everything below
      713     71   0.11    268    284     17
      549     41   0.06   1236   1263     28
      ...
placed: rows 133 .. 1391 -> final y 130 .. 1394
  dTop +94 (loose) dBot +282 (loose)
```

One component of 390,035 px spans rows 133…1391 at full page-column width. A
single thin bridge of ink — a page border, a scrollbar edge — merges the browser
chrome, the page and the content below it, and "the rows a wide component
occupies" degenerates to "every row with any ink". Its ceiling is **6 of 21,
top 14, bottom 7** — below MC-034 on every one of the three.

This is worth stating as a general result rather than as a failed attempt:
**connected components of the ink are not a usable decomposition of a comic
page**, because the ink of a comic page is connected. Any future attempt in this
direction needs a morphological step (opening, erosion, run-length filtering)
*before* labelling, and this spike did not try one.

### 3b. Family 2, gutter regions — the family the epic's own argument implies

The epic's sentence is about a **gutter** that a bubble contaminates. The dual
construction is therefore the right one: label the **background**, not the ink.

> A gutter containing a speech bubble is not a run of empty pixels on any row —
> but it is still **one connected background region that reaches from the
> page's left edge to its right, around the bubble**.

`gcov(y)` is the share of row `y` belonging to a background component whose
bounding box spans at least `span_w` of the page column's width. A row is a
gutter row iff `gcov(y) ≥ min_g`.

This is the quantity `cover(y)` cannot compute. `cover(y)` counts pixels at row
`y`; `gcov(y)` counts pixels at row `y` **that belong to a region established
across other rows**. Two rows with identical `cover(y)` differ in `gcov(y)`
whenever their background pixels belong to different regions — which is
precisely the bubble-in-a-gutter case, where the background at that row is part
of the page-spanning gutter and the bubble's own interior is not.

### 3c. The grid

| Parameter | Values | Family |
|---|---|---|
| `dev` — the binarisation, MC-034's | 10, 20, 30, 40 | both |
| `min_area` — smallest region that counts | 0, 256 | both |
| `gap` — the run of non-page rows that ends the page, MC-034's | 2, 4, 8, 12, 24 | both |
| `min_w` — a page-own ink region's width share | 0.10 … 0.90, 8 values | ink |
| occupancy by ink rows / by bounding-box span | 2 values | ink |
| `span_w` — a gutter region's width share | 0.80, 0.90, 1.00 | gutter |
| `min_g` — the share of a row a gutter region must cover | 0.50, 0.70, 0.85, 0.95 | gutter |

**640 ink parameterisations and 480 gutter, 1120 in all.**

## 4. The separation demonstration (AC-3)

AC-3 asks whether the 2D property separates a bubble row from a marked edge
where `cover(y)` does not. `cover(y)` is recomputed only on the specific rows
below; `## Out of scope` permits exactly that and forbids re-running MC-034's
540-member sweep, which is not re-run anywhere in this document.

### 4a. Choosing the comparison row without consulting the answer

For each marked edge, the **overhang row** is the row outside the mark by more
than MC-019's own slack of 11 px, within 150 px of it, with the **highest
`cover(y)`**. The rule does not consult the 2D property, which would make the
demonstration circular; it picks the row an overhanging object makes look most
like the page to a per-row scalar; and the 11 px exclusion is MC-019's, not this
spike's — inside the slack a placement is a *hit*, so a row there is not a
misread at all.

That exclusion is load-bearing and was added after a first attempt failed
without it. Without the 11 px guard the rule picked the row **immediately**
next to the mark on six of the seven entries, and separated nothing, because
adjacent rows are alike by construction. It is recorded here because the
measurement looked like a finding and was an artefact.

### 4b. The four numbers, on two of the seven bubble entries

Thresholds: `min_cover` **0.10**, the largest MC-034 swept
(`panel-edge-search.md` 3c); `min_g` as marked. `cover` at `dev 20`, `gcov` at
`dev 10`, `span_w 1.00`.

| Entry / edge | row | `cover` | `gcov` | `cover` says | `gcov` says |
|---|---|---|---|---|---|
| `Screenshot (2708)` top — **marked edge** | 220 | **0.429** | **0.383** | content | page |
| `Screenshot (2708)` top — overhang row | 190 | **0.133** | **0.865** | content | **gutter** |
| `Screenshot (2630)` top — **marked edge** | 224 | **0.456** | **0.541** | content | page |
| `Screenshot (2630)` top — overhang row | 133 | **0.239** | **0.750** | content | **gutter** |

`cover(y)` calls both rows content in both entries, at every `min_cover` MC-034
swept: it **does not separate**. `gcov(y)` calls the marked edge the page and
the overhang row a gutter — at `min_g 0.85` for `2708` and `min_g 0.70` for
`2630`, both values in the grid above: it **does separate**.

Both are top edges. That is not a convenience: `corpus.md` records `2630`
**top** as one of the two cases the 2026-09-17 marking rule was written for
("the neighbour's bubble outside"), so it is the edge where the phenomenon
lives. The bottom edges of all seven were measured too and none separates — on
those the gutter below the mark is genuinely empty, `gcov` is simply
`1 − cover`, and there is nothing for connectivity to do.

### 4c. And the same question at corpus scale, where it fails

A pair of rows on two entries shows the mechanism exists. It does not show a
rule can be built on it. The corpus-scale form of the question is: **is there
any threshold that puts all 42 marked edge rows on the page side and all 42
overhang rows on the gutter side?**

```
$ cargo run --release --bin separate
cover(y), page side is HIGH:
  lowest  at a marked edge : 0.000  (2025-08-05 00_11_13 top)
  highest at an overhang   : 0.998  ((103) top, row 218)
  a separating threshold exists: NO

gcov(y), page side is LOW:
  highest at a marked edge : 1.000  ((3538) bottom)
  lowest  at an overhang   : 0.000  (2025-10-14 23_30_20 bottom, row 1391)
  a separating threshold exists: NO

best achievable, sweeping the threshold at 0.001 over all 42 edges:
  cover(y): 6 of 42 pairs, at min_cover 0.408
  gcov(y) : 8 of 42 pairs, at min_g     0.463
```

Neither statistic is separable, and both are overlapped end to end — each has a
marked edge on the wrong side of every overhang row. **8 of 42 against 6 of 42
is the entire measured value of the 2D property**, and it is why section 5's
headline ties MC-034's rather than beating it. This is the number that decides
the epic's premise, and it decides it against a rule built on this property
while leaving the premise's *diagnosis* of the 1D families intact.

## 5. Both of MC-019's numbers (AC-4)

Every score is over the **final, margin-expanded, clamped** rect, as MC-028
section 5b, MC-031 section 8 and MC-034 section 6 compute it. Only the **row**
edges are judged, because only the rows are what these families place: the
columns come from the shipped locator, settled at 20 of 21 by MC-027/MC-019 and
not reopened here.

### 5a. The ranked sweep

```
$ cargo run --release --bin sweep
=== family ink: 640 parameterisations ===
 rows  clips   unpl   top   bot  rule
    5      3      0    11     6  ink    dev 20 min_w 0.75 area    0 gap 12 rows
    5      3      0    11     6  ink    dev 20 min_w 0.90 area    0 gap 12 rows
    4      1      0    10     5  ink    dev 10 min_w 0.20 area    0 gap 12 rows   (fewest clips)

best rows          : 5 of 21
best CLIP-FREE     : no member of this family is clip-free at all
best TOP edge      : 13 of 21
best BOTTOM edge   : 6 of 21
fewest clips       : 1

=== family gutter: 480 parameterisations ===
 rows  clips   unpl   top   bot  rule
    8      6      0    14     9  gutter dev 10 span_w 1.00 min_g 0.85 area    0 gap  2
    8      7      0    13     9  gutter dev 10 span_w 0.80 min_g 0.85 area    0 gap  2
    7      5      0    13     9  gutter dev 10 span_w 0.80 min_g 0.95 area    0 gap  2   (fewest clips)

best rows          : 8 of 21
best CLIP-FREE     : no member of this family is clip-free at all
best TOP edge      : 14 of 21
best BOTTOM edge   : 9 of 21
fewest clips       : 5
```

### 5b. The per-entry table for the best rule

`gutter dev 10 span_w 1.00 min_g 0.85 area 0 gap 2`, **8 of 21 with 6 clips**:

```
$ cargo run --release --bin detail -- gutter 10 0 1.00 0.85 2
file                                  dTop    dBot     in   clip  band
2025-08-05 00_11_13.webp                +2      +7    yes      -
2025-08-05 00_11_27.webp                +6      +9    yes      -
2025-10-14 23_29_06.png                 +9     +11    yes      -
2025-10-14 23_30_20.png                 +5     +61      -      -
2025-10-20 15_37_25.png                -15     +14      -   CLIP
2026-01-05 13_33_41.png                 +9     +15      -      -
2026-01-05 13_45_59.png               -172     +69      -   CLIP
2026-01-05 13_49_39.png                -46     +44      -   CLIP
Screenshot (67).png                     +0      +6    yes      -
Screenshot (70).jpg                     +8     +17      -      -
Screenshot (75).png                     +7     +16      -      -
Screenshot (93).jpg                    +48    -573      -   CLIP
Screenshot (103).jpg                   -49    +177      -   CLIP
Screenshot (1661).png                   +0      +9    yes      -
Screenshot (2582).jpg                  +86      +6      -      -
Screenshot (2630).jpg                   +3      +5    yes      -
Screenshot (2698).jpg                   +5      +0    yes      -  alt bottom 1080: dBot  +4 -> hit (either)
Screenshot (2708).jpg                   +5      +5    yes      -  alt bottom 1305: dBot +21 -> hit (either)
Screenshot (2744).jpg                   +4     +37      -      -
Screenshot (3187).png                   +8     +16      -      -
Screenshot (3538).png                   -1     -83      -   CLIP  alt bottom 1314: dBot -68 -> CLIP (both)

rows inside the 11 px window : 8 of 21
clips                         : 6
top edge ok                   : 14 of 21
bottom edge ok                : 9 of 21
```

Two things in that table are worth naming. **Five entries miss the window by
14 to 17 px on the bottom** — `15_37_25`, `70`, `75`, `3187`, and `13_33_41` at
+15 — which is just outside an 11 px slack; they are near misses rather than
failures of kind. And **`2698` and `2708` are placed exactly**, both of them
gutter-crossing entries whose bottom mark lies past an internal gutter: MC-034's
family stopped at that gutter on both (`panel-edge-search.md` 3c), and this one
does not.

The best ink rule, `ink dev 20 min_w 0.75 area 0 gap 12`, is **5 of 21 with 3
clips** — its table is behind the `detail` command in section 7.

The two `diagonal-gutter` entries are scored **exactly as MC-019 scores them**.
`corpus.md`'s band tolerance is not folded into any number above. Reported
separately and never in a headline: under MC-035's disjunction predicate the
three gutter-crossing entries give **2 of 3** acceptable bottoms for the best
gutter rule and **3 of 3** for the best ink rule.

### 5c. Against the baselines, in the words AC-4 asks for

| Source | Score | Clips |
|---|---|---|
| MC-026 finding 1b (flat fraction) | **8 of 21** | — |
| MC-028 (panel gutter, 264 parameterisations) | 4 of 21 | 6 |
| MC-031 `anchored` (672) | 5 of 21 | 5 |
| MC-034 coverage (540) | **8 of 21** | **9** |
| MC-034 spread (240) | 6 of 21 | 4 |
| **MC-038 gutter regions (480)** | **8 of 21** | **6** |
| **MC-038 ink regions (640)** | 5 of 21 | 3 |

**It does not beat 8 of 21.** It ties it.

**The clip-free best is 0 of 21** — no member of either family is clip-free at
all, which is MC-035's stronger form holding here too. That is worse than
MC-028's clip-free 1 of 21 and equal to MC-034/MC-035's 0 of 780, and under
MC-005 decision 13, which ranks a clip above every other defect, it is the
number that matters most.

**Does it break the clips-versus-placements trade-off, or move along it?**
`v2-candidates.md` sets the test: a member that both places more *and* clips
less than every prior member breaks it; anything else does not.

```
breaking the trade-off (places more AND clips less than a prior):
  vs MC-034 coverage (8 of 21, 9 clips): none
  vs MC-034 spread   (6 of 21, 4 clips): none
  vs MC-028          (4 of 21, 6 clips): 30 members, best 7 of 21 with 5 clips
```

**It moves along the trade-off. It does not break it.** No member of either
family places more than MC-034's coverage family while clipping less, and none
places more than MC-034's spread family while clipping less. What it does is
strictly better at a *fixed* number of placements — 8 of 21 at 6 clips against
8 of 21 at 9 — and that is a step along the curve, not off it.
`v2-candidates.md`'s sentence survives this spike unchanged.

## 6. The four negative controls, each shown firing (AC-5)

All four fire. Counts are out of the full 1120 unless split by family.

### (a) A neighbour's overhang

MC-034 section 4(a)'s synthetic page, read out and rebuilt to the same numbers:
the page is rows 200…800 over all 400 columns, and a narrow caption box, 80 of
400 columns, sits in the gutter below at 820…880. The right bottom edge is
**800**; reaching 880 is the over-reach.

```
    control FIRES on 64 of 1120 parameterisations (5.7 %)
    first firing: ink dev 10 min_w 0.10 area 0 gap 24 rows -> rows 200..880

ink     n=640   (a)   16    2.5 %
gutter  n=480   (a)   48   10.0 %
```

Against MC-034's measured **45 of 540 (8.3 %)** for coverage and **60 of 240
(25 %)** for spread. The ink family is the best result anyone has reported on
this control — 2.5 % — which is direct evidence that a component's width does
the job `corpus.md`'s marking rule asks of it, and it is the one measurement in
this document that argues *for* ink regions. It is not enough to rescue them:
section 3a's percolation is what decides that family.

The gutter family is **worse** than MC-034's coverage family here, at 10.0 %
against 8.3 %, and the reason is legible: a caption box 80 px wide leaves the
gutter around it still spanning, so a spanning-gutter rule with a low `min_g`
reads the caption's rows as gutter and scans straight through them. The
property that lets it see past a bubble is the same property that lets it see
past a caption box it should stop at.

### (b) A flat full-width row inside the panel

MC-026 finding 5's defect, which MC-028, MC-031 and MC-034 all ran: the page is
200…1000 with a flat full-width band at 600…640.

```
    control FIRES on 1120 of 1120 parameterisations (100.0 %)
    first firing: ink dev 10 min_w 0.10 area 0 gap 2 rows -> rows 200..599
```

It fires on **every** member of both families, exactly as it fired on all 540 of
MC-034's coverage family, and for the same structural reason: the band is 41
rows and the largest `gap` swept is 24. Connectivity does not help, and this is
the clearest case in the document of it not helping — a full-width flat band
separates the art above from the art below into two components under any
connectivity, so there is no region for a rule to follow across it. Surviving
this control still needs `gap` above the widest flat full-width row a real page
contains, and real gutters are smaller than that. **The control and the corpus
still pull `gap` in opposite directions.**

### (c) The seven `"expect": "flag"` tuning entries

```
    shipped pipeline crops 1 of 7 (the no-regression baseline)
    control FIRES on 1024 of 1120 parameterisations (91.4 %)
    worst member crops 6 of 7
    first firing: ink dev 10 min_w 0.30 area 0 gap 2 rows -> crops 2 of 7
```

The baseline reproduces MC-034's 1 of 7 exactly. 91.4 % of members regress
against it, and the worst crops 6 of the 7 pages that must not be cropped at
all. This is the same defect MC-031's rule 3 was rejected for independently of
its clips, and it is a second reason — beyond the clips — that no member here is
shippable.

### (d) A control on the property this spike invents

(a), (b) and (c) are all inherited from MC-034 and all test failures the
*previous* family had. A metric whose only controls test the case it was
designed for is untested, so this one attacks the width rule directly, in the
two shapes that break it. Both inputs are new to this spike.

**(d1) The page's own art is genuinely narrow.** Rows 200…1000, 80 of 400
columns — real art, the page's own, not an overhang. The right answer is
200…1000.

```
    control FIRES on 800 of 1120 parameterisations (71.4 %)
    first firing: ink dev 10 min_w 0.20 area 0 gap 2 rows -> placed NOTHING
    ink 560 of 640     gutter 240 of 480
```

**(d2) The page's own art is genuinely broken into pieces.** Rows 200…979, a
grid of 24 px blocks with 4 px gutters over all 400 columns — a page of
dialogue panels with no large element anywhere. Every component is 24 px wide,
0.06 of the page. The 4 px gutters are bridged by every `gap` of 8 or more, so
for those members the only thing that can fail is the width rule.

```
    control FIRES on 448 of 1120 parameterisations (40.0 %)
    first firing: ink dev 10 min_w 0.10 area 0 gap 2 rows -> rows 592..615
    ink 256 of 640     gutter 192 of 480
```

Both fire, and what they say is the cost of the property section 3 is built on:
**a rule keyed to a wide region discards a page that has none.** Seventy-one per
cent of the grid cannot place a narrow page at all, and the first firing places
*nothing* rather than placing it badly — which under MC-007's flag-and-copy
behaviour is the safe failure, but is still a page the product would not crop.
The corpus has no narrow or fragmented page in it, so this failure mode is
invisible to every score in section 5 and would have shipped unmeasured.

## 7. Reproducing every number (AC-6)

The harness is **outside the repository**, so no phase, gate or committed test
depends on it:

```
C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\
  42467df5-cf72-43f6-a222-dd7931a46f1d\scratchpad\mc038
```

A cargo crate with `[workspace]` in its `Cargo.toml` and path dependencies on
`cropper-core` and `cropper-engine`, composing only their public stages.

| Command | What it produced |
|---|---|
| `cargo run --release --bin guard [file]` | section 2: the denominator, the 28 names, the refusal |
| `cargo run --release --bin base` | 2c: the shipped baseline, for calibration |
| `cargo run --release --bin ceiling` | section 8: 1120 parameterisations, the per-file ceiling and the per-edge reachability |
| `cargo run --release --bin sweep [ink\|gutter]` | 5a: every parameterisation, both of MC-019's numbers, the trade-off test |
| `cargo run --release --bin detail -- gutter <dev> <area> <span_w> <min_g> <gap>` | 5b: one named rule per file, with MC-035's disjunction alongside |
| `cargo run --release --bin detail -- ink <dev> <area> <min_w> <gap> [span]` | the same for an ink rule |
| `cargo run --release --bin separate` | section 4: the four numbers per entry and the 42-edge separability test |
| `cargo run --release --bin controls` | section 6: all four controls and their firing counts |
| `cargo run --release --bin profile -- "<file>" <family> ...` | 3a: the component structure and occupancy runs of one entry |
| `cargo run --release --bin dump -- "<file>" <ylo> <yhi> [step]` | `cover(y)` and `gcov(y)` side by side over a range of rows |

Numbers read out of MC-026, MC-028, MC-031, MC-032, MC-034, MC-035 or
`corpus.md` cite the section they come from and are **not** re-measured. MC-034's
540-member coverage sweep is not re-run anywhere; `cover(y)` is computed only on
the specific rows section 4 names, which is the boundary `## Out of scope`
draws.

## 8. The reachability ceiling (AC-2)

For each entry, is there **any** parameterisation of either family placing that
edge inside MC-019's window — the mark grown by `margin_px + 8` = 11 px —
without clipping? No single rule can cherry-pick per file, so this is an upper
bound the family cannot reach.

```
$ cargo run --release --bin ceiling
region family, 1120 parameterisations

file                                both   topOK   botOK
2025-08-05 00_11_13.webp             yes     yes     yes
2025-08-05 00_11_27.webp             yes     yes     yes
2025-10-14 23_29_06.png              yes     yes     yes
2025-10-14 23_30_20.png               NO     yes      NO
2025-10-20 15_37_25.png               NO      NO     yes
2026-01-05 13_33_41.png               NO     yes      NO
2026-01-05 13_45_59.png               NO     yes     yes
2026-01-05 13_49_39.png              yes     yes     yes
Screenshot (67).png                  yes     yes     yes
Screenshot (70).jpg                  yes     yes     yes
Screenshot (75).png                   NO     yes      NO
Screenshot (93).jpg                   NO      NO      NO
Screenshot (103).jpg                  NO     yes      NO
Screenshot (1661).png                yes     yes     yes
Screenshot (2582).jpg                 NO      NO     yes
Screenshot (2630).jpg                yes     yes     yes
Screenshot (2698).jpg                yes     yes     yes
Screenshot (2708).jpg                yes     yes     yes
Screenshot (2744).jpg                 NO     yes      NO
Screenshot (3187).png                 NO     yes      NO
Screenshot (3538).png                 NO      NO      NO

cherry-picked per-file ceiling : 10 of 21
  top edge reachable           : 17 of 21
  bottom edge reachable        : 13 of 21
```

| | this family | MC-034, read out of `panel-edge-search.md` 2c |
|---|---|---|
| both (the ceiling) | **10 of 21** | 10 of 21 |
| top edge reachable | 17 of 21 | **20 of 21** |
| bottom edge reachable | **13 of 21** | 11 of 21 |

The ink family alone reaches **6 / 14 / 7**; the whole of the improvement over
it is the gutter family, and section 3a says why.

### 8a. It is not a reproduction of MC-034's ceiling, and not the vacuous answer

AC-2 names two controls on this number and neither fires.

**It is not vacuous.** Eleven of the 21 are excluded, so the instrument
discriminates; a ceiling of 21 of 21 would have meant it does not.

**It is not a reproduction of MC-034's ceiling by a different instrument**,
which would have closed the epic's premise outright. The two elevens are
*different*:

| | entries |
|---|---|
| unreachable for both | `23_30_20`, `15_37_25`, `13_33_41`, `13_45_59`, `75`, `93`, `103`, `2744`, `3187` — nine |
| reached here, **not** by MC-034 | `13_49_39`, `Screenshot (70)` |
| reached by MC-034, **not** here | `Screenshot (2582)`, `Screenshot (3538)` |

Two entries each way. The headline 10 of 21 is the same number over a different
set, and the per-edge decomposition is where the two instruments actually
differ: this family gives up three top edges and gains two bottom ones.

**The bottom edge is the one that matters** — `## Context` finding 1, from
`panel-edge-search.md` 2c: ten of MC-034's eleven unreachable entries fail on
the bottom alone, and a family that improves the top improves nothing. This
family improves the bottom, 11 → **13 of 21**, and gives ground on the top,
20 → 17. That is the correct direction on the binding constraint and it is still
**13 of 21 against a bar of 20 of 21**.

### 8b. The timebox, and the judgement made against it

`## Model guidance` sets two conditions for stopping at the ceiling without a
ranked sweep, and on this measurement **they disagree**:

- *"if it does not clear MC-034's both-edges 10 of 21"* — 10 of 21 **ties** and
  does not clear. This clause says stop.
- *"or, more sharply, if the bottom edge does not clear MC-034's 11 of 21"* —
  13 of 21 **clears**. This clause says continue.

The sweep was run, and the reasoning is recorded here rather than left
implicit. The sharper clause is the one `## Context` finding 1 argues is the
real constraint; the ceiling being the same number over a *different* eleven is
materially unlike MC-034's situation and is not something a tie conveys; and by
the time the ceiling existed the region primitive was built, so the ranked
sweep cost one binary and one run rather than the expensive path the timebox was
sized against. A reader who thinks the first clause should have governed loses
sections 5 and 6 and keeps everything else; the verdict in section 1 does not
turn on it.

## 9. The verdict against both yardsticks, and what stories 3 and 4 inherit (AC-7)

### 9a. A reasoned negative

**No rule tried reaches MC-019's bar of 20 of 21 with zero clips, and none in
this family can.**

*Against the ceiling (section 8):* the best single rule reaches 8 of 21 against
a ceiling of 10 of 21, so the family is close to its own ceiling and the ceiling
is the binding constraint — better tuning inside it is worth at most two
entries. The ceiling itself is 10 of 21 against a bar of 20 of 21, a factor of
two.

*Against the baselines (section 5c):* it does not beat 8 of 21, its clip-free
best is 0 of 21, and it moves along the clips-versus-placements trade-off rather
than breaking it.

The shapes tried and why each failed:

1. **Ink regions, 640 parameterisations** — the ink of a comic page percolates,
   so a page and its chrome are one component (section 3a). Ceiling 6 of 21.
   Failed structurally, not by tuning.
2. **Gutter regions, 480 parameterisations** — the property is real and
   demonstrably 2D (section 4b), but the separation it provides holds on 8 of
   42 edges rather than on 42 (section 4c). Best 8 of 21 with 6 clips.

### 9b. EPIC-07 story 3, known site furniture: **run it**

The epic makes story 3 conditional on this one. The recommendation is to **run
it, unchanged in scope**, and this spike strengthens rather than weakens the
case:

- Both cheap pixel-level directions are now closed. Story 2 was the last
  untested idea that needed neither learning nor per-site knowledge, and the
  epic's own framing — "it costs nothing to test" — has been paid and answered.
- The failures are concentrated where site furniture is the explanation.
  Section 5b's five near misses at 14–17 px on the bottom are the taskbar and
  the site's own footer; `13_45_59` and `13_49_39` fail because the page's art
  runs continuously into the taskbar, which no gutter exists to stop at but
  which **is pixel-identical across every screenshot from that machine**.
- Section 6's controls say what a general rule costs: (d1) and (d2) fire on a
  page with no wide element, and (b) fires on every member. A rule that matches
  *known* furniture has none of those failure modes, because it is not inferring
  a page at all.

Two cautions for whoever writes it. The epic is explicit that adopting per-site
special cases is **a product decision needing an amendment to the brief, not an
agent's judgement**. And `corpus.md` records the site tags MC-037 added across
seven readers with counts as low as one; a furniture matcher measured on
`toongod` (10 entries) and `demonicrevolution` (1) is measuring two different
things.

### 9c. Story 4 and the held-out set

**No rule is recommended, so no held-out score is owed and none was taken.** If
a later story does recommend one, `corpus.md` requires it to select
`Split::HeldOut` **once**, deliberately, on a single **frozen** rule, and to say
so in its own file; the unseen-*reader* number rests on five entries across
three readers and is a direction rather than a percentage. **This story does
not do it**, and nothing in this document was chosen, adjusted or rejected on
anything a held-out entry produced.

### 9d. For whoever tries region structure again

Not a recommendation to run another spike — the bar is not reachable from here —
but the three things this one would want a successor to know:

- **Label the background, not the ink.** Section 3a. The ink of a comic page is
  one region; any ink-based attempt needs a morphological opening first, and
  this spike did not try one.
- **The 2D property is real and too rare.** Section 4c is the number to beat:
  8 of 42 against `cover(y)`'s 6 of 42. A successor that cannot move that number
  will tie 8 of 21 again whatever else it changes.
- **Build the narrow-page and fragmented-page controls first.** Section 6(d).
  The corpus contains neither, so both failure modes are invisible to every
  score in this document, and 71.4 % of this grid cannot place a narrow page at
  all.

`docs/wiki/v2-candidates.md` carries a row for this document so the next agent
opening v2 cold does not re-run it.
