# Is a panel gutter locatable from pixels?

MC-028, a spike. The deliverable is a measured answer, not code. Nothing in
`crates/` changed.

A **panel gutter** is the white or black space between panels, running
horizontally across the strip, with speech bubbles routinely sitting on one
(`architecture.md`, "Two kinds of blank space"). The question is whether the
top and bottom of the corpus's hand-marked page rects can be placed from the
pixels, well enough for MC-019's accuracy bar.

## 1. The verdict: a reasoned negative

**No rule tried reaches the bar.** Seven rule shapes were scored across
**264 parameterisations**; the best reaches **4 of 21 with 6 clips**, the
best clip-free rule reaches **1 of 21**, and cherry-picking the best
parameterisation **per file** — which no single rule can do — reaches only
**7 of 21**. MC-019 needs 20 of 21 with zero clips, and MC-026's existing
baselines are 5 of 21 (mean absolute deviation), 8 of 21 (flat fraction) and
0 of 21 with 17 to 20 clips (a search over the flat-fraction profile). **Nothing
here beats 8 of 21**, and it is reported as not beating it.

The five best, with both of MC-019's numbers, two-axis through the columns
MC-027 leaves and the rest of `decide`:

| # | Rule | Right (MC-019 AC-2) | Clips (AC-1) | Flag entries cropped | Metric control |
|---|---|---|---|---|---|
| 1 | `LastBandStart`, flat ≥ 0.40, bridge 4, tone tol 10 | **4 / 21** | **6** | 1 / 7 (no regression) | holds |
| 2 | `LastGutterEnd`, flat ≥ 0.40, bridge 1, tone tol 24 | 4 / 21 | 6 | 4 / 7 (**breaks 3**) | holds |
| 3 | `DeepestGutterEnd`, flat ≥ 0.60, bridge 4, tone tol 10 | 4 / 21 | 7 | 1 / 7 | holds |
| 4 | `LastGutterEnd`, flat ≥ 0.60, bridge 1, tone tol 10 | 3 / 21 | 1 | 2 / 7 (**breaks 1**) | holds |
| 5 | `LastBandStart`, flat ≥ 0.90, bridge 8, tone tol 10 | 1 / 21 | **0** | 1 / 7 | holds |

Rules 1–4 fail on AC-1 before AC-2 is reached: MC-019 AC-1 ranks a clip above
every other defect (MC-005 decision 13), and a rule that clips 6 of 21 has
failed however many entries it otherwise places. Rule 5 is the only clip-free
member of the family and it places one edge pair in 21. Section 7 says why each
fails, mechanically.

**This negative is the useful outcome.** It stops a third feature story from
guessing at the row axis, and it sends one number back to the user: see
section 9.

## 2. What was already settled, read out rather than re-derived

From MC-026's `## Notes`, by reference:

- **Finding 4 — the panel gutter is separable.** Per-row flat fraction (the
  share of a row's pixels within `uniform_tolerance` of that row's *median*)
  has median **0.201** inside a panel and **0.984** in the gutter band just
  outside the mark; one threshold at 0.40 classifies **61 of 63 bands**
  correctly.
- **Finding 5 — it is not locatable by a search over that profile.** Threshold
  × minimum gutter depth × selector, columns given for free: **0 of 21 right,
  17 to 20 clips in every combination.** The failure mode is *too many*
  boundaries, not none: panels contain flat full-width rows a single row
  profile cannot tell from a gutter.
- **Finding 1b — mean absolute deviation is the wrong statistic here** (5 of 21)
  and the earlier flat-fraction attempt reached **8 of 21**.

Those three numbers — **5, 8 and 0 of 21** — are the baselines every rule below
is compared against. None of them was re-measured as a discovery.

## 3. The instrument, and its validation

Before measuring anything new, MC-026 finding 4 was reproduced from its
definition in this spike's own harness, over `codec::to_luma`'s plane rather
than MC-026's float BT.601 rounding — two implementations rather than one run
twice:

```
flat fraction: inside a panel 0.201   in the gutter band outside 0.984
best single threshold: 0.40 classifies 61/63 bands correctly
```

Identical to the recorded values, and the per-file table agrees with MC-026's
own binary to three decimals on all 63 bands (`Screenshot (93)`: 0.390 / 0.618 /
0.545; `Screenshot (103)`: 0.208 / 0.247 / 0.964).

**One correction to MC-026's prose, not to its conclusion.** MC-026's `## Notes`
names the two misclassified bands as "`Screenshot (103)` below (0.247 …) and
`Screenshot (93)` above (0.545, borderline throughout)". At a threshold of 0.40,
0.545 is on the *correct* side, so `Screenshot (93) above` is classified right;
the second failure is **`2026-01-05 13_49_39.png` above, at 0.294**. Both this
harness and MC-026's own `gutter` binary, re-run, agree. `Screenshot (93)` is
genuinely borderline — its panel interior reads 0.390 against the 0.40
threshold — which is what that sentence was reaching for. 61 of 63 stands.

## 4. The row axis is the whole of the remaining error

Measured first, because it sizes everything else. At `Tuning::default()` on this
tree, over the 21 marked entries:

| What | MC-019 AC-2 right | AC-1 clips |
|---|---|---|
| the shipped detector, both axes | **0 / 21** | 0 |
| the shipped detector, **rows replaced by the mark's own rows** | **20 / 21** | 0 |

With 6 of the 7 `"expect": "flag"` entries answered `Flagged`, a correct row
placement alone would put MC-019 AC-2 at **26 of 28 = 92.9 %**, clear of its
90 % bar, with zero clips. The one remaining column-axis miss is
`Screenshot (93).jpg`, right edge +20 px against a slack of 11 — MC-027's and
MC-019's business, not this spike's.

So the row axis carries the entire deficit, and it is large: the rect the
detector produces is **97 to 306 px too tall at the top** and **16 to 310 px too
tall at the bottom** on every one of the 21 marked entries.

## 5. The evidence

### 5a. What is actually at the 42 marked row edges

The starting point no earlier row measurement had. MC-026 findings 1, 1b and 5
profiled rows over the *marked* columns and over the *whole image's* rows; this
profiles them over the **page column MC-027 locates**, inside the rect the real
pipeline produces — which is finding 7's move applied to the other axis.

The structure at each edge was printed rather than searched over blind
(`edges.rs`). Three shapes appear, and each is common:

1. **the edge is the end of the leading flat region** — browser chrome above,
   art below, nothing between. `2025-08-05 00_11_13.webp` (chrome to row 115,
   mark 114), `Screenshot (2744).jpg` (133 / 134), `Screenshot (3187).png`
   (137 / 142);
2. **the edge is a tone change inside one flat region** — chrome grey gives way
   to page white and the panel begins tens or hundreds of rows lower.
   `2025-10-14 23_30_20.png`: rows 40–167 flat at tone 37–59, rows 167–313 flat
   at tone 249–255, art from 313, **mark 171**. The mark is 4 px past the tone
   change and 142 px above the art;
3. **the edge is nowhere in particular inside a uniform band.**
   `2025-10-20 15_37_25.png`: one flat band, rows 167–279, tone 248–255, flat
   fraction 1.00 — **mark 228**, 61 rows inside it. The only thing distinguishing
   row 228 is that rows 215–226 carry a handful of dark pixels (row minimum 8 to
   53) and rows 227 onward are pure 255.

Shapes 2 and 3 are why every "widest / nearest-centre / holds-centre band"
selector clips: the flat region *straddles* the mark, so any selector that keeps
the region cuts below the mark and any selector that drops it cuts far above.

### 5b. The rule family, and its scores

Seven rule shapes, each parameterised by a flat threshold, a bridge depth (how
short a non-flat run may be and still be crossed) and a tone tolerance. All are
given the **page's own flat tone**, established once per image as the modal row
median over the flat rows in 8-level bins — which is candidate direction 1, and
the context finding 5's search did not have.

| Selector | Places the edge at | Best right / clips |
|---|---|---|
| `FlatEnd` | the end of the leading flat region | 1 / 21, 1 clip |
| `LastBandStart` | the start of the last tone band of that region (the chrome-to-page transition) | 4 / 21, 6 clips |
| `LastGutterEnd` | the end of the last **page-toned** band inside it | 4 / 21, 6 clips |
| `LastGutterStart` | the start of that same band | 0 / 21, 0 clips |
| `FirstArt` | MC-025's rule over the page column: the first row whose deviation reaches `min_line_spread` | 0 / 21, 1 clip |
| `DeepestGutterEnd` | the end of the deepest page-toned band in the leading half | 4 / 21, 7 clips |
| `margins` (section 5d) | the first row whose page margins are flat | 0 / 21, 0 clips |

216 parameterisations of the first six, 48 of the seventh. **The bottom edge is
where it breaks.** Scored per edge:

| | best rule | edges placed | of which clips |
|---|---|---|---|
| top edge | `DeepestGutterEnd`, 0.40 / 1 / 24 | **13 / 21** | 6 |
| bottom edge | `LastGutterEnd`, 0.60 / 4 / 24 | **7 / 21** | 1 |

and no rule is best at both: the two columns of that table are maximised by
different rules, so a single rule scores 4 of 21 jointly at best. The per-file
detail under the best top-edge rule shows the shape of it — thirteen top edges
inside the window, and the same rule 252, 497 and 544 px wrong at the bottom on
`00_11_13`, `67` and `13_45_59`.

### 5c. The ceiling of the whole family: 7 of 21

Cherry-picking the best of all 216 parameterisations **per file**, which no
single rule can do, places both edges correctly on **7 of 21**. Fourteen entries
are reachable by *no* member: `23_29_06`, `23_30_20`, `15_37_25`, `13_33_41`,
`13_45_59`, `13_49_39`, `70`, `75`, `93`, `103`, `2582`, `2744`, `3187`, `3538`.

This is the strongest statement in the document, because it bounds the family
rather than sampling it: **no selector over these six placements can reach
MC-019's bar, whatever selector is invented.** 7 of 21 also does not beat
MC-026's 8 of 21.

### 5d. Candidate direction 3, column structure: 0 of 21

Every rule above reads the page column's own pixels. This one reads the pixels
*beside* it: a row inside the page's vertical extent has flat page margin on
both sides of the page column, while a row of browser chrome or taskbar is
painted across the whole width and has none. 3 margin widths × 4 thresholds ×
4 bridge depths:

```
best: margins k 16  th 0.80  bridge  1 -> 0/21 with 0 clips
  2025-08-05 00_11_13.webp     top    24 vs 114      -90   bottom  1440 vs 1330    +110
  2025-10-14 23_30_20.png      top    40 vs 171     -131   bottom  1440 vs 1379     +61
  2026-01-05 13_33_41.png      top     4 vs 233     -229   bottom  1438 vs 1293    +145
```

**0 of 21 at every parameter, and the mechanism is visible in the offsets**: the
page column is narrower than the browser's own toolbars, so the pixels beside it
are flat *everywhere* in the rect, chrome rows included. The signal does not
discriminate at all. Direction 3 is measured and it fails.

### 5e. Why: the features are there, and they are not selectable

One level above any rule family. A rule that places an edge has to place it
where the pixels change, so the distance from each marked edge to the nearest
change of **sixteen** kinds was measured — flat fraction crossing 0.40 / 0.60 /
0.80 / 0.90 / 0.95 / 0.99, row median moving by more than 4 / 10 / 24, deviation
crossing 2.0 / 8.0 / 24.0, the row becoming pure, the row becoming free of
content, and the adjacent-row gradient crossing `edge_threshold`.

**40 of 42 marked edges lie within MC-019's 11 px window of a change of some
kind, and 30 of 42 lie exactly on one.** The two exceptions are
`Screenshot (103).jpg` bottom (nearest change 65 px away) and
`Screenshot (70).jpg` bottom (12 px).

So the marks are not arbitrary. The problem is **which** change:

| Feature | the marked top edge is the *k*-th change from the top | rival candidates per rect |
|---|---|---|
| flat ≥ 0.40 | k = 1 … 16 (median 3) | 2 … 80 |
| flat ≥ 0.90 | k = 3 … 11 (median 5) | 5 … 26 |
| flat ≥ 0.99 | k = 3 … 19 (median 5) | 3 … 51 |
| median moves > 10 | k = 5 … 35 (median 7) | 14 … 150 |
| deviation ≥ 8.0 | k = 3 … 9 (median 5) | 6 … 16 |
| gradient ≥ 24 | k = 6 … 30 (median 11) | 8 … 124 |

Every feature kind is present at the right row **and at five to a hundred and
fifty wrong rows**, and the ordinal of the right one is not constant on any of
them. That kills "take the k-th change" as well as the six selectors scored, and
it is the precise sense in which the panel gutter is separable (finding 4) and
not locatable (finding 5): *separating* gutter from panel is a per-row question
the flat fraction answers well; *locating* the page is a choice among dozens of
correct separations, and nothing in the profile ranks them.

### 5f. And the oracle itself carries two conventions

The same question asked of the manifest rather than of a rule. For each marked
edge, the maximal homogeneous band containing it was grown — rows flat at
0.90 or above, each within `uniform_tolerance` of the previous — and the edge's
depth inside it measured. A mark `k` rows inside such a band cannot be placed
closer than `k` by any rule, because nothing distinguishes row `k` from row
`k − 1`.

```
uniform gutter the marked rects CONTAIN, over the 42 edges: min 0, median 0, max 50
edges where that depth exceeds MC-019's 11 px window: 6 of 42
all values: [0 × 30, 1, 1, 2, 2, 2, 3, 12, 22, 23, 36, 41, 50]
```

Thirty of the 42 edges sit exactly on a change — tight against the art, as
`architecture.md` describes. Six sit 12 to 50 rows inside a homogeneous gutter
band, and the band outside them runs to 97 rows (`Screenshot (67).png` top) and
170 (`Screenshot (2698).jpg` bottom). Two entries make the point on their own:
`2025-10-14 23_30_20.png` and `2025-10-20 15_37_25.png` have the *same*
structure — browser chrome to row 167, then flat white — and are marked at
**171** and **228**.

That is a 57 px disagreement between two marks of the same pixel feature,
against an 11 px window. It is not a criticism of the marking: a person drawing
a box around a page in a uniform white gutter has no reason to prefer one row
over another, and both boxes are correct as *pages*. It does mean the manifest
does not define the row edges to 11 px, and no rule can be fitted to a target
that is not defined that finely.

## 6. The negative controls (AC-4)

Both controls AC-4 names were run against every parameterisation.

**The metric control — a panel containing a flat full-width row**, the defect
finding 5 names. A synthetic 400 × 600 page: gutter at tone 250 in rows 0–60 and
540–600, textured art between, and a **flat full-width block at the same tone in
rows 200–230, inside the panel**. A rule holds iff it places neither edge at
that block. It **fires for 6 of the 216 parameterisations** — every
`DeepestGutterEnd` at a flat threshold of 0.40 — so the control discriminates
rather than passing vacuously. The five rules in section 1 all hold on it.

**The corpus control — the seven `"expect": "flag"` entries**, which must not
become croppable. Six of the seven are `Flagged` on this tree;
`2025-02-27 22_46_15.png` is already `Cropped` and is the entry MC-026's AC-4
names `NOT_THIS_STORYS_TO_FIX`, so **1 of 7 cropped is the no-regression
baseline** and any rule reporting 2 or more has broken one. Rules 2 and 4 in
section 1 do: `LastGutterEnd` at flat ≥ 0.40 / tone tol 24 makes **four** of the
seven croppable. That is an independent reason to reject them, and it is the
reason a rule cannot be salvaged by widening its tone tolerance.

## 7. Why each of the five best fails

1. **`LastBandStart` 0.40 / 4 / 10 — 4 of 21, 6 clips.** Right where the mark
   sits at the chrome-to-page tone change (shape 2): `00_11_13` (+0, +3),
   `00_11_27` (−4, +5), `67` (+3, +4), `2708` (−2, +2). Its six clips are the
   two shapes the tone change cannot see: a mark that sits *inside* the page
   gutter band (`15_37_25` top +52, `13_45_59` top +4, `1661` top +4) and a
   trailing flat region that starts far above the mark (`3538` bottom −88,
   `13_49_39` bottom −80, `2698` bottom −5). Three of those six are clips by 4
   or 5 px, which is the whole margin between "right" and "worst defect in the
   ranking".
2. **`LastGutterEnd` 0.40 / 1 / 24 — 4 of 21, 6 clips, and it breaks 3 flag
   entries.** Widening the tone tolerance to 24 is what lets it bridge chrome
   greys into the page tone; that same widening makes an all-art image look like
   a page with margins, which is what the flag entries catch.
3. **`DeepestGutterEnd` 0.60 / 4 / 10 — 4 of 21, 7 clips.** The deepest
   page-toned band is the panel's own sky or dark field on a third of the
   corpus. This is finding 5's failure mode returning under a new name, now
   with the page tone known: knowing the tone does not help when the panel's
   interior *is* that tone.
4. **`LastGutterEnd` 0.60 / 1 / 10 — 3 of 21, 1 clip.** The best trade in the
   family and still nowhere near: right on `1661`, `2698` and `2708`, and its
   single clip is `Screenshot (3538).png`, where the trailing flat white starts
   87 rows above the mark.
5. **`LastBandStart` 0.90 / 8 / 10 — 1 of 21, 0 clips.** Clip-free because at a
   0.90 threshold the leading flat region usually has one band and the rule
   declines to speak; a rule that stays silent does not clip and does not crop
   either. It is the honest shape of "no rule": the only clip-free member is the
   one that does nothing.

## 8. Reproducing every number here (AC-5)

The harness is **outside the repository**, as MC-025's and MC-026's were, so no
phase and no committed test depends on it:

```
C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\
  14e270c4-a323-4bc1-9b58-3cbdb69acba0\scratchpad\gutterspike
```

A cargo crate with path dependencies on `cropper-core` and `cropper-engine`.
Every statistic is written from its definition rather than by calling a
crate-private helper; the two deliberate exceptions are `codec::to_luma`, so the
plane measured is the plane the detector sees, and the composition of the
crate's own public stages in `pipeline_rect`, because the question is what a
rule sees when wired into that pipeline. MC-019's two numbers are computed over
the **final, margin-expanded, clamped** rect, so no convention about
`margin_px` can drift.

| Command | What it produced |
|---|---|
| `cargo run --bin repro` | section 3, finding 4 reproduced and the one correction |
| `cargo run --bin look` | section 4, the 0 / 21 and 20 / 21, and the pipeline rect per entry |
| `cargo run --bin profile [threshold]` | every flat row run per entry, with depth and tone |
| `cargo run --bin edges` | section 5a, the banded structure at all 42 marked edges |
| `cargo run --bin dump -- "<file>" [y0] [y1]` | one entry's banded rows |
| `cargo run --bin raw -- "<file>" <y0> <y1>` | one entry's unbanded per-row statistics (the `15_37_25` reading) |
| `cargo run --bin rules` | section 1 and 5b, all 216 parameterisations two-axis with both numbers, the flag control and the metric control |
| `cargo run --bin peredge` | section 5b's per-edge table and its per-file detail |
| `cargo run --bin ceil` | section 5c, the 7 of 21 ceiling, and the metric control's 6 of 216 |
| `cargo run --bin detail -- <Selector> <th> <bridge> <ttol>` | section 7, one named rule per file with both numbers |
| `cargo run --bin marginrow` | section 5d, column structure, 0 of 21 |
| `cargo run --bin feature` | section 5e, distance to the nearest change of sixteen kinds |
| `cargo run --bin ordinal` | section 5e's ordinal table |
| `cargo run --bin interior` | section 5f, the gutter depth inside each mark |
| `cargo run --bin bar` | section 9, the slack sweep and the per-entry row error |

MC-026's harness, whose `gutter` binary was re-run for the correction in
section 3, is at the path its own `## Notes` records.

## 9. What has to move (AC-6)

No implementation story is recommended. **One of MC-019's numbers has to move,
and the measurement says which.**

- **Not the 90 %.** Section 4: with the row axis correct, MC-019 AC-2 reads
  26 of 28 = 92.9 % with zero clips. The bar is reachable; it is the row
  placement that is not.
- **Not the 11 px tolerance, for the same reason it cannot help.** Sweeping the
  slack with the detector as it stands:

  ```
   slack      shipped   clips          of 28
      11         0/21       0         21.4%
     100         1/21       0         25.0%
     200        13/21       0         67.9%
     300        20/21       0         92.9%
  ```

  The slack would have to rise from **11 px to 294 px** for the current detector
  to clear 90 %. On a 1440 px screen that is a crop a fifth of the screen too
  tall — "needing no manual fix" would have stopped meaning anything, and
  MC-019 AC-2 would be measuring nothing. Note the detector never clips: every
  error is in the too-large direction, so the *failure mode is benign* and only
  the tolerance is violated.
- **So it is the manifest**, and the question is narrow and answerable. Section
  5f: 30 of the 42 marked edges sit exactly on a pixel change, and 6 sit 12 to
  50 rows inside a uniform gutter band, with two entries of identical structure
  marked 57 px apart. **The question for the user is which of the two
  conventions the marks mean**, on the vertical axis only:

  - *tight* — the page's row edges are where the **art** begins and ends, so a
    mark inside a uniform gutter band should be moved to the art; or
  - *loose* — the row edges are where the **browser chrome ends and the taskbar
    begins**, so the gutter above and below the panel is wanted in the crop.

  Either is a coherent product answer and each makes a *different* rule
  measurable. Under *tight*, section 5b's `FlatEnd` and `LastGutterEnd`
  selectors become scoreable against a consistent target instead of a mixed one.
  Under *loose*, the target is the page's vertical extent and section 5d's
  column-structure signal is the one to revisit with a stronger margin
  definition than "flat beside the column".

  **What the user would need to decide:** six images, opened with the
  homogeneous band drawn on. `2025-10-20 15_37_25.png` and
  `2025-10-14 23_30_20.png` first, because they are the identical-structure pair
  marked 57 px apart; then the four other entries carrying the six deep-inside
  edges — `2026-01-05 13_49_39.png` (36 rows inside at the top, 41 at the
  bottom), `Screenshot (2582).jpg` (22), `Screenshot (3538).png` (23) and
  `Screenshot (103).jpg` (50). That is an hour with the images, not another
  spike.

The recorded form of the answer is an **MC-019 `## Amendments` entry**: which
acceptance criterion moves, what it said, what it says now, who approved it and
why. MC-019's `## Model guidance` already anticipates exactly this — "the corpus
can be wrong: a 'clip' on one file may be a mis-marked rect" — and names the
orchestrator and the user as the ones who decide.

Until that is decided, **no third story should attempt the row axis.** Two have
now been measured wrong after the fact (MC-025's row locator moved the corpus by
zero entries; MC-026's interior-band premise reached 5 of 21), and this spike is
the third attempt, measured *before* shipping.

## 10. `Screenshot (103).jpg`, as instructed

It did not drive the design and here is what it does. Its marked rows are
302–1146 inside a pipeline rect of 18–1439. Above the mark, rows 220–383 are
flat at tone 0–13 — the black region the user has said is to be treated as a
gutter — and the mark sits **56 rows inside its outside half and 50 rows inside
its inside half**, the deepest such case in the corpus. Below the mark, the
white diagonal gutter reads 0.247 on finding 4's flat-fraction test (the
threshold is 0.40), and it is the **one edge of the 42 with no pixel change of
any of the sixteen kinds within 65 px**. Both halves of the file are
unlocatable by everything measured here, which is consistent with the user's
ruling that it is not a priority and that including the region above the mark is
acceptable. It is one of the 14 entries no member of the rule family reaches, so
excluding it changes no number in this document by more than one entry.

## 11. Scope

No production or test code was written. `Tuning::default()`, the corpus and the
manifest are untouched — the manifest question in section 9 is put to the user,
not applied. The column axis is taken as MC-027 leaves it, and every row number
here was measured through those columns; section 4 records what they are worth
on their own. Colour and chroma were not revisited (ruled out in MC-025's
`## Context`).
