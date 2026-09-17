# Is a page row edge locatable from the panel's own art edge?

MC-034, a spike. The deliverable is a measured answer, not code. Nothing in
`crates/` changed.

MC-028 asked whether a **panel gutter** is locatable from pixels and came back
a reasoned negative. MC-031 asked the same of **full-width chrome** and came
back a second one. Both searched for the *gutter* and tested a candidate row by
*flatness*. MC-032's survey of all 21 marked entries points out that three of
the four causes of a missing flat gutter defeat a flatness test by construction
while the panel's own edge stays plainly visible to a reader, and MC-033 then
made "one page's own artwork" the corpus's official marking rule
(`docs/wiki/corpus.md`). This document asks the remaining question: can the
marks be placed from the **panel's own art edge**?

## 1. The verdict: a reasoned negative, and a sharp correction

**No rule tried reaches the bar, and it is the bottom edge that closes it.**
Two families were scored across **780 parameterisations**. The best single rule
reaches **8 of 21 with 9 clips**; the best **clip-free** rule reaches **0 of
21**; and cherry-picking the best parameterisation **per file** — which no
single rule can do — reaches only **10 of 21**. MC-019 needs 20 of 21 with zero
clips. **This does not beat 8 of 21.**

But the ceiling decomposes in a way no earlier document could see, and this is
the finding worth carrying forward:

| Edge | Reachable by *something* in 780 parameterisations |
|---|---|
| **top** | **20 of 21** |
| **bottom** | **11 of 21** |
| both (the ceiling) | 10 of 21 |

**The top edge is not the problem and has not been for some time.** Twenty of
the twenty-one top marks are placeable, and one *single* rule places 13 of them.
The entire deficit is the bottom edge. EPIC-05's amendment — "MC-028 and MC-031
closed both pixel-level directions" — is therefore **too strong in a specific,
checkable way**, and section 7 says what should replace it.

The second correction is to MC-032's cause 2, and it goes the *opposite* way
from what MC-034's own `## Context` predicted. See section 2.

## 2. The reachability ceiling (AC-1)

### 2a. What a panel-edge rule can and cannot be blocked by

MC-032's `## Notes` partitions the 21 marked entries into four causes of a
missing flat gutter. Those causes are **settled** — they came from opening
every entry with the user — and are read out here, never re-derived. What is
*not* settled is which of them blocks a rule that looks for the panel's art
edge rather than for an empty gutter, because MC-032 was describing why
*flatness* failed.

| # | Cause | Entries MC-032 names | Blocks an art-edge rule? |
|---|---|---|---|
| 1 | a neighbouring bubble crosses the gutter | `2630`, `1661`, `13_33_41`, `23_30_20`, `13_45_59`, `70`, `2708` — 7 | **no.** The edge is visible. It is a *hazard* — over-reach — which control (a) tests |
| 2 | the mark is the browser viewport edge | top edges of `00_11_13`, `00_11_27`, `2744`, `3187`, `75`, `3538`, `23_29_06` — 7 | **measured: no.** See 2b |
| 3 | diagonal gutter | `93` top, `15_37_25` bottom — 2 | partly. The mark is one answer in a band (`corpus.md`); MC-019 scores it exactly |
| 4 | dark art on a dark gutter | `103`, `2582` — 2 | in principle no; **measured: yes for `103`** (2c) |

MC-033 supplies two edge-level rulings the per-entry lists do not:
`Screenshot (2744)` **bottom** (a caption box straddles it, and is outside the
rectangle) and `Screenshot (2630)` **top**.

### 2b. Cause 2 does not block, and this contradicts MC-034's own prior

MC-032 says of the seven cause-2 top edges: "There is no panel boundary in the
image to find. Only this cause makes the task ill-posed." Read as a statement
about *reachability*, that is not what the pixels do. Because the art begins at
the cut line, a rule seeking the outermost wide art row **lands on the mark
anyway**:

| Entry | coverage family, dTop | spread family, dTop |
|---|---|---|
| `2025-08-05 00_11_13.webp` | +2 | +2 |
| `2025-08-05 00_11_27.webp` | +6 | +6 |
| `2025-10-14 23_29_06.png` | +9 | +24 |
| `Screenshot (75).png` | +7 | +7 |
| `Screenshot (2744).jpg` | +4 | +4 |
| `Screenshot (3187).png` | +8 | +53 |
| `Screenshot (3538).png` | −2 (clip) | −1 (clip) |

Six of the seven land inside MC-019's 11 px window on at least one family, four
on both. This **independently reproduces** MC-031 section 7, which placed
`00_11_13` (+2, +7) and `00_11_27` (+6, +10) with a completely different family
— so that result was not a coincidence of where the browser cut.

**MC-032 is not wrong about the images; it is right that the panel is cut.** The
two statements are reconciled, not in conflict: those entries are ill-posed as
*panel-boundary detection* and well-posed as *art-extent detection*, and the
mark happens to be the same row. Nothing in the eye survey is disturbed, and no
image was re-opened to say this — the disagreement was never about what the
images show.

The consequence is that **cause 2 excludes nothing**, so the ceiling cannot be
derived by subtracting it. AC-1 requires the ceiling both ways, and here they
are: **14 of 21** under MC-032's reading (21 minus the seven cause-2 entries)
and **21 of 21** under the permissive one. The second is the vacuous answer
AC-1's control rejects, and the first is not supported by the table above. So
the ceiling is taken **by measurement** instead, in 2c.

### 2c. The measured ceiling: 10 of 21, with the eleven named

For each entry, is there *any* parameterisation of *either* family placing both
edges inside the window without clipping? No single rule can cherry-pick per
file, so this is an upper bound the family cannot reach.

```
file                             both    topOK    botOK
2025-08-05 00_11_13.webp          yes      yes      yes
2025-08-05 00_11_27.webp          yes      yes      yes
2025-10-14 23_29_06.png           yes      yes      yes
2025-10-14 23_30_20.png            NO      yes       NO
2025-10-20 15_37_25.png            NO      yes       NO
2026-01-05 13_33_41.png            NO      yes       NO
2026-01-05 13_45_59.png            NO      yes       NO
2026-01-05 13_49_39.png            NO      yes      yes
Screenshot (67).png               yes      yes      yes
Screenshot (70).jpg                NO      yes       NO
Screenshot (75).png                NO      yes       NO
Screenshot (93).jpg                NO      yes       NO
Screenshot (103).jpg               NO       NO       NO
Screenshot (1661).png             yes      yes      yes
Screenshot (2582).jpg             yes      yes      yes
Screenshot (2630).jpg             yes      yes      yes
Screenshot (2698).jpg             yes      yes      yes
Screenshot (2708).jpg             yes      yes      yes
Screenshot (2744).jpg              NO      yes       NO
Screenshot (3187).png              NO      yes       NO
Screenshot (3538).png             yes      yes      yes

cherry-picked per-file ceiling : 10 of 21
  top edge reachable           : 20 of 21
  bottom edge reachable        : 11 of 21
```

The eleven no parameterisation of either family places: `23_30_20`,
`15_37_25`, `13_33_41`, `13_45_59`, `13_49_39`, `70`, `75`, `93`, `103`,
`2744`, `3187`. **Ten of the eleven fail on the bottom edge alone** — their top
is reachable. The single entry whose *top* is unreachable is
`Screenshot (103).jpg`, a cause-4 dark-on-dark entry.

**10 of 21 is below MC-019's 20 of 21 with a factor of two to spare.** No
tuning closes that.

### 2d. MC-032's flatness instrument does not reproduce, so nothing here rests on it

The obvious way to spread MC-032's per-*entry* causes onto the 42 edges MC-019
scores is its own flatness instrument ("a row is flat if fewer than 3 pixels in
the marked columns deviate more than 40 luma from the row median; a mark is
bounded if ≥ 6 consecutive flat rows fall inside its ±11 px window").
Re-implemented from that description it gives **run@top ≥ 6 for 14 of 21** where
MC-032 records 16 of 21, and **run@bottom = 0 for 14 of 21** where MC-032
records 13 of 21.

That is not a defect to tune away. It is MC-032's own warning — "the partition
is simply not robust to the flatness definition, and no argument should rest on
it" — reconfirmed on a second independent implementation. The instrument was
therefore **abandoned**, not calibrated to its target, and the ceiling in 2c is
measured against the corpus directly instead.

## 3. The two families, and both of MC-019's numbers (AC-2)

Both are anchored to the **page column** `flat::page_column` already finds
(MC-027, 20 of 21 on the column axis) and scan **outward from the page's own
vertical centre**, stopping at the first run of `gap` non-page rows. Nothing in
either refers to the image edges, to chrome, or to any full-width structure.

### 3a. Why this is not MC-031's `anchored` under a new name

`chrome-row-search.md`'s `anchored` family is itself an art scan — "anchored,
chrome th 1.00 / k 64, **art `flat < 0.90`, run 8**" — over 672
parameterisations, best 5 of 21 with 5 clips. Two structural differences, both
required by MC-034 AC-2:

- **Keyed to the page, not to chrome.** `anchored` starts at the browser's
  full-width UI and scans inward; every one of its four ranked rules fires its
  own control (c2) by skipping the chrome block entirely. These start at the
  page column's centre and scan outward.
- **A stopping rule for *which* art edge is the page's.** `gap` is the smallest
  gutter that ends the page: below it an interruption is a gutter *inside* the
  page and the scan continues through it. That is `corpus.md`'s marking rule
  made numeric, and it is the parameter `anchored` — written before that rule
  existed — has no equivalent of.

### 3b. Family 1, spread: 240 parameterisations, best 6 of 21 with 4 clips

A row carries art when its mean absolute deviation over the page column is at
least `art_th`. Best member `art_th 6.0, gap 12, backoff 0`; clip-free best
**0 of 21**; per-file ceiling 7 of 21.

### 3c. Family 2, coverage: 540 parameterisations, best 8 of 21 with 9 clips

**The family `corpus.md`'s marking rule actually implies**, and the one no
earlier spike had a reason to try. "A neighbouring panel's overhang, or a
caption box sitting in the gutter, does not [extend the rectangle]." An overhang
is geometrically **narrow**; a row of the page's own art is **wide**. So the
discriminator is not *is this row flat* — which is what MC-028, MC-031 and
MC-032 all asked, and a bubble makes a row non-flat, which is exactly why they
failed — but **how much of the row's width is not background**.

`cover(y)` is the share of the page column's pixels at row `y` deviating from
that row's own median by more than `dev`; `min_cover` is the threshold.

```
 rows  clips  flags   top   bot    dev  cover  gap  backoff
    8      9      3    12     9     20   0.10    4        0
    7      8      2    12    10     20   0.05    2        0
    7      8      2    12     9     20   0.05    4        0
    7      9      3    12     8     20   0.10    8        0
    6      6      2    10     7     20   0.02    2        0

parameterisations : 540
best rows         : 8 of 21
best CLIP-FREE    : 0 of 21
best TOP edge     : 13 of 21
best BOTTOM edge  : 10 of 21
```

The per-entry table for the best member is in section 6's `detail` command. Its
failures are almost all one of two shapes, both bottom-edge:

- **the page's art runs continuously into the taskbar**, so there is no gutter
  to stop at — `13_45_59` (art 364…1440), `13_49_39` (art 331…1440);
- **the mark lies past an internal gutter the scan stops at** — `2708` (stops
  1305, mark 1322), `3538` (stops 1314, mark 1330), `2698` (stops 1080, mark
  1085). This one matters and section 7 turns it into a question.

The two `diagonal-gutter` entries are scored exactly as MC-019 scores them.
`corpus.md`'s band tolerance is *not* folded into any number above; had it been,
`15_37_25` and `93` would both read differently, and that is precisely why the
band is reported separately and never in a headline.

## 4. The negative controls, each shown firing (AC-3)

### (a) A neighbour's overhang — new to this spike

MC-028's and MC-031's controls all test *under*-reaching. Under `corpus.md`'s
marking rule the characteristic failure of an art-edge rule is **over**-reaching
onto something outside the rectangle, and nothing existing catches it. A
synthetic page occupies rows 200…800; a narrow caption box (80 of 400 columns,
20 % of the width) sits in the gutter below at 820…880. The right bottom edge is
**800**.

```
(a) neighbour's overhang, coverage family
    control FIRES on 45 of 540 parameterisations
    first firing: cover dev 20 min_cover 0.02 gap 24 backoff 0 -> bottom 880 (right answer 800)
(a) neighbour's overhang, panel family
    control FIRES on 60 of 240 parameterisations
    first firing: panel art_th 4.0 gap 24 backoff 0 -> bottom 880 (right answer 800)
```

It fires, and it **discriminates**: 8.3 % of the coverage family over-reaches
against 25 % of the spread family. That is direct evidence `min_cover` does the
job the marking rule asks of it, and it is the one result in this document that
argues *for* the coverage family rather than against it.

### (b) The seven `"expect": "flag"` entries

```
(b) the 7 flag entries
    shipped pipeline crops 1 of 7 (the no-regression baseline)
```

The best coverage rule crops **3 of 7**, and the best spread rule **2 of 7**.
Both regress against the baseline, so the control fires on both. This is the
same defect MC-031's rule 3 was rejected for independently of its clips.

### (c) A flat full-width row inside the panel

MC-026 finding 5's defect, which MC-028 and MC-031 both ran. A synthetic page
occupies 200…1000 with a flat full-width band at 600…640.

```
(c) flat full-width row inside the panel, coverage family
    control FIRES on 540 of 540 parameterisations
    first firing: cover dev 20 min_cover 0.02 gap 2 backoff 0 -> rows 200..600 (right answer 200..1000)
```

It fires on **every** member, and the reason is structural rather than a bad
choice of grid: the band is 40 rows and the largest `gap` swept is 24, so every
member splits the page there. Surviving it needs `gap` above the widest flat
full-width row a real page contains — and `gap` above 12 is already off the top
of 3c's ranking, because real *gutters* are smaller than that. **The control and
the corpus pull `gap` in opposite directions**, which is an independent reason
this family cannot be tuned into the bar.

## 5. The verdict against both yardsticks (AC-4)

**A reasoned negative.** No rule tried reaches MC-019's bar of 20 of 21 with
zero clips, and none can: the measured ceiling is 10 of 21.

Against AC-1's ceiling: the best single rule reaches 8 of 21 against a ceiling
of 10 of 21, so the family is close to its own ceiling and the ceiling is the
binding constraint. Better tuning inside this family is worth at most two
entries.

Against the five baselines:

| Source | Score | Clips |
|---|---|---|
| MC-026 finding 1 (mean absolute deviation) | 5 of 21 | — |
| MC-026 finding 1b (flat fraction) | **8 of 21** | — |
| MC-026 finding 5 (search over the flat-fraction profile) | 0 of 21 | 17–20 |
| MC-028 (panel gutter, 264 parameterisations) | 4 of 21 | 6 |
| MC-031 `anchored` (672 parameterisations) | 5 of 21 | 5 |
| **MC-034 coverage (540)** | **8 of 21** | **9** |
| **MC-034 spread (240)** | 6 of 21 | 4 |

**It does not beat 8 of 21.** It ties it on the headline and loses on the number
that ranks above it: MC-005 decision 13 puts a clip above every other defect,
and the clip-free best of this family is **0 of 21**, worse than MC-028's 1 of
21. The spread family's 6 of 21 with 4 clips is the best *clip* count anyone has
reported on the row axis, and 6 of 21 is still not 8.

The timebox in MC-034's `## Model guidance` fired here: ceiling below 20 of 21,
two families scored, neither beating 8 of 21.

## 6. Reproducing every number (AC-5)

The harness is **outside the repository**, so no phase, gate or committed test
depends on it, as MC-025's, MC-026's, MC-028's and MC-031's were:

```
C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\
  d2462a4d-1798-4ec1-a131-529a1297daa3\scratchpad\mc034
```

A cargo crate with `[workspace]` in its `Cargo.toml` and path dependencies on
`cropper-core` and `cropper-engine`, composing only their public stages. The
rect every number is computed over is the **final, margin-expanded, clamped**
one, as MC-028 section 5b and MC-031 section 8 do, so no `margin_px` convention
drifts between documents.

| Command | What it produced |
|---|---|
| `cargo run --bin base` | section 1's shipped baseline: rows 0 of 21, clips 0, overshoot +97…+306 top and +16…+310 bottom |
| `cargo run --bin flatrun` | 2d: MC-032's instrument re-implemented, 14 of 21 and 14 of 21 |
| `cargo run --bin profile [minlen]` | the art/gutter run structure per entry behind 3c's two failure shapes |
| `cargo run --bin sweep` | 3b: all 240 spread parameterisations, both numbers, flag control, ceiling |
| `cargo run --bin sweep2` | 3c: all 540 coverage parameterisations, the same, plus the per-edge split |
| `cargo run --bin detail -- cover <dev> <min_cover> <gap> <backoff>` | 2b and 3c: one named rule per file, with each entry's cause |
| `cargo run --bin detail -- panel <art_th> <gap> <backoff>` | the same for a spread rule |
| `cargo run --bin ceiling` | 2c: 780 parameterisations, the per-file ceiling and the per-edge reachability |
| `cargo run --bin controls` | section 4: all three controls and their firing counts |

Numbers read out of MC-026, MC-028, MC-031, MC-032 or `corpus.md` cite the
section they come from and are **not** re-measured. MC-028's and MC-031's
harnesses are at the paths their own documents record; nothing here re-runs
them.

## 7. What this means for MC-032 (AC-6)

### Recommendation: keep MC-032 parked — but change the reason

Unparking is not supportable. The ceiling is 10 of 21 against a bar of 20 of 21,
and no tuning inside either family closes a gap of that size.

**But MC-032's `## Context` is now wrong in a checkable way and should be
corrected whatever happens to the story.** It asserts, following EPIC-05's
amendment, that "both pixel-level directions are now closed". Three things
replace that:

1. **The top edge is not closed. It is nearly solved.** 20 of 21 top marks are
   placeable, and a single rule places 13 of 21. No document before this one
   reports a per-edge *reachability* number, as opposed to a per-edge score.
2. **The whole deficit is the bottom edge**, at 11 of 21 reachable. MC-032's
   survey said this from the flatness side ("run@bottom is 0 for 13 of 21");
   this is the same conclusion from an instrument that does not depend on
   flatness, which matters because 2d shows the flatness one does not reproduce.
3. **Cause 2 is not a blocker** (2b). The seven browser-cut entries are among
   the easiest, not the hardest.

### The question for the user, which is not another spike

Three entries fail in a way that is **about what the mark means**, not about
the pixels. In each, the rule stops at a gutter that is plainly there, and the
user's bottom mark is *below* it:

| Entry | rule stops at | user's bottom mark | gutter it stopped at |
|---|---|---|---|
| `Screenshot (2708).jpg` | 1305 | **1322** | 1305…1352 |
| `Screenshot (3538).png` | 1314 | **1330** | 1314…1404 |
| `Screenshot (2698).jpg` | 1080 | **1085** | 1082…1254 |

If the marked rectangle is meant to span **more than one panel** — to include
content past an internal gutter — then "stop at the first gutter" can never be
right, and the family in section 3 is mis-specified rather than merely weak.
`corpus.md`'s marking rule settles whose *overhang* counts but is silent on
whether a mark may span a gutter.

**Open the three entries above and say whether the bottom mark is meant to be
below that gutter.** If yes, a rule that seeks the *last* wide art row before
the taskbar — rather than the first gutter — is a different family and is
untested; that would be a reason to run one more spike. If no, these three are
mis-marked and MC-019's oracle changes, which is the user's call and nobody
else's (`corpus.md`: "the mark is as likely to be the thing that is wrong as
the detector").

A third option the measurements support and this document does not recommend:
**MC-019's bar could be split again, by edge rather than by axis**, since the
top edge is at 20 of 21 reachable and the bottom is not. That is a product
decision about what "nine in ten need no manual fix" means, and MC-019's
criteria are frozen and DONE, so it goes to the user rather than into a story.
