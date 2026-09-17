# Are the page row edges locatable from full-width chrome?

MC-031, a spike. The deliverable is a measured answer, not code. Nothing in
`crates/` changed, and neither did `fixtures/corpus/manifest.json` or
`Tuning::default()`.

**Browser and OS chrome** is fixed-position, full-screen-width UI: tab strips,
the URL bar, a bookmarks bar, the Windows taskbar. It is not a **panel gutter**
(`architecture.md`, "Two kinds of blank space"), which is the white or black
space between two panels and which MC-028 searched and closed. The question
here is whether the top and bottom of the corpus's hand-marked page rects can be
placed from the chrome instead, well enough for MC-019's accuracy bar.

## 1. The verdict: a reasoned negative

**No rule tried reaches the bar, and the reason is structural rather than a
matter of tuning.**

Twelve rule shapes were scored across **1312 parameterisations** (plus every
pairing of a top rule with a bottom rule, 1312 × 1312). The best single rule
reaches **5 of 21 with 5 clips**; the best clip-free rule reaches **1 of 21**;
the best *hybrid* — a different selector for each edge, which is still one rule
and is a freedom MC-028's family did not have — reaches **6 of 21**. MC-019
needs 20 of 21 with zero clips, and MC-026's baselines are 5 of 21 (mean
absolute deviation), 8 of 21 (flat fraction) and 0 of 21 with 17 to 20 clips.
**Nothing here beats 8 of 21**, and it is reported as not beating it.

| # | Rule | Right (MC-019 AC-2) | Clips (AC-1) | Top edge | Bottom edge | (b) flag entries cropped | (a) flat row | (c) no chrome |
|---|---|---|---|---|---|---|---|---|
| 1 | anchored, chrome th 1.00 / k 64, art `flat < 0.90`, run 8, backoff 0 | **5 / 21** | 5 | 10 / 21 | 7 / 21 | 1 / 7 (no regression) | holds | **fails** |
| 2 | anchored, th 1.00 / k 64, art `flat < 0.90`, run 8, backoff 4 | 4 / 21 | 4 | 12 / 21 | 6 / 21 | 1 / 7 | **fails** | **fails** |
| 3 | anchored, th 1.00 / k 64, art `flat < 0.60`, run 2, backoff 0 | 4 / 21 | 9 | 12 / 21 | 8 / 21 | 3 / 7 (**breaks 2**) | holds | **fails** |
| 4 | `OuterRuns` on the margin, th 1.00, run 16, k 64 | 2 / 21 | 2 | 10 / 21 | 4 / 21 | 1 / 7 | holds | holds |
| 5 | anchored, th 0.98 / k full, art `dev ≥ 4.0`, run 8, backoff 11 | 1 / 21 | **0** | 6 / 21 | 3 / 21 | 1 / 7 | **fails** | **fails** |
| 6 | the **perfect chrome oracle** — the browser viewport boundary itself | **0 / 21** | **0** | 4 / 21 | 2 / 21 | — | holds | holds |

The last three columns are AC-4's negative controls, each shown firing in
section 6. **Only rules 4 and 6 hold all three, and they are the two lowest
scorers**: every rule that places more edges does so by reaching past what the
chrome signal knows, and pays for it on a control.

Rules 1–4 fail on AC-1 before AC-2 is reached: MC-019 AC-1 ranks a clip above
every other defect (MC-005 decision 13). Rule 5 is the honest clip-free member
and it places one entry in 21. Section 7 says why each fails, mechanically.

**Row 6 is the whole answer and it is not a rule's failure.** Section 4 measures
what the chrome signal locates: the **browser viewport** — the row the chrome
ends at and the row the taskbar begins at. It locates it very well: one
unambiguous candidate per edge on 15 of 21 entries, never clipping, 19 of 19
located edges lying outside the mark. And the viewport is **not what the manifest
marks**. Scored against the marks it is **0 of 21**. No rule keyed to chrome can
do better than the thing it is keyed to, so the family's ceiling is set before
any threshold is chosen.

**What is left over is panel gutter, and that is MC-028's closed question.**
Section 4's outputs were written and looked at: under the chrome rule the worst
entry in the corpus, `Screenshot (2698).jpg` (+210 / +307), carries **no browser
chrome and no taskbar at all** — only white panel gutter above the panel and the
next panel's speech bubble below. The two spikes therefore compose into a
complete negative on the row axis: MC-028 found *too many* candidate rows and
nothing to rank them; MC-031 finds *exactly one* candidate row per edge, ranked
unambiguously, and it is the wrong row.

**This negative is the useful outcome.** It stops a fourth attempt at the row
axis and it sends MC-019 back to the user with both pixel-level directions
closed. Section 9 says what that leaves, and it includes one genuinely new
option that MC-028's evidence could not support.

## 2. What was already settled, read out rather than re-derived

The six items in MC-031's `## Context`, by reference:

1. **MC-026 finding 4** — per-row flat fraction separates a panel row from a
   gutter row: median 0.201 inside a panel, 0.984 in the gutter band, one
   threshold at 0.40 classifying 61 of 63 bands.
2. **MC-026 finding 5** — and it does not locate the page: **0 of 21, 17 to 20
   clips** over every threshold × depth × selector.
3. **MC-026 findings 1 and 1b** — **5 of 21** (mean absolute deviation) and
   **8 of 21** (the earlier flat-fraction attempt). These are two of the three
   baselines below.
4. **MC-028 section 5f, the ordinal finding** — 40 of 42 marked edges are within
   11 px of a change of one of sixteen kinds and 30 of 42 sit exactly on one,
   but each kind changes at 2 to 150 other rows and the right one's ordinal runs
   k = 1…16, 3…11, 5…35, 6…30. Separating is easy; ranking is what nothing has
   managed. **Section 5d confronts this directly.**
5. **MC-019 `## Notes` measurement 3** — "the marked edge sits at an art
   boundary" is vacuous (78.6 % of marks within 0 px, 99.4 % of all other rows
   likewise) and is withdrawn. Not re-derived; its lesson — run the control
   before believing the statistic — is applied in sections 3 and 5d.
6. **MC-019 `## Notes` measurement 2** — MC-028's "identical-structure pair
   marked 57 px apart" does not hold. **The manifest is not treated as broken
   and is not edited.**

Also read out and not re-opened: MC-019's hit window (`margin_px + 8`), its
zero-clip rule, the 11 px tolerance and the manifest, all fixed by the user's
decision of 2026-09-16; MC-005 decision 13; MC-027's columns, taken as they are.

The three **baselines** every rule below is compared against are **5 of 21**,
**8 of 21**, and **0 of 21 with 17 to 20 clips**. For calibration, MC-028
section 4's figure, reproduced in section 3: with the row axis correct and
MC-027's columns as they are, MC-019 AC-2 reads **26 of 28 = 92.9 %**.

## 3. The instrument, and its validation

### 3a. The baseline reproduces exactly

`cargo run --bin base`, the shipped detector at `Tuning::default()` over all 28
entries, through this harness's own composition of the crates' public stages:

```
SHIPPED  : 0/21 right, 0 clips, 6/7 flag entries Flagged
           flag entries cropped: ["2025-02-27 22_46_15.png"]
ROWS FROM THE MARK, columns from the pipeline: 20/21 right, 0 clips
worst ROW overshoot: top +97 … +306, bottom +16 … +310, on every one of the 21
```

Identical to MC-019 `## Notes` measurement 1 and to MC-028 section 4. The row
axis carries the entire deficit; the columns are solved.

### 3b. The signal: the pixels beside the page column, over the FULL image width

For each row, the share of the pixels **outside** the page column MC-027 locates
— over the whole image width, not the pipeline rect — that lie within
`uniform_tolerance` of the page background tone. The background tone is
established once per image as the modal per-row margin median in 8-level bins.
A row of browser chrome or taskbar is painted edge to edge and has no flat
margin; a row inside the reader page has flat margin on both sides.

### 3c. Is chrome separable from page? Yes, and better than the panel gutter is

MC-026 finding 4's question, asked of this instrument, with ground truth that
comes from no rule under test: **rows 0…100 are browser chrome on every one of
the 21** (the earliest chrome end measured anywhere in this document is 115), and
**the rows strictly inside the hand-marked rect are page on every one of the
21**. `cargo run --bin separate full`:

```
margin bg-flat fraction, over all 21 entries:
  browser chrome rows (0..100)   n=  2100  median 0.000
  page rows (inside the mark)    n= 22913  median 0.993
  best single threshold 0.815 classifies 24939/25013 rows = 99.70%
```

0.000 against 0.993, and one threshold gets 99.70 % of 25 013 rows right. That is
a cleaner separation than MC-026 finding 4's 0.201 / 0.984 at 61 of 63.

**This corrects MC-028 section 5d's stated mechanism, not its score.** Section 5d
scores the same direction at 0 of 21 and explains it: "the page column is
narrower than the browser's toolbars, so the pixels beside it are flat
*everywhere* in the rect, chrome rows included. The signal does not discriminate
at all." The second sentence is wrong. Measured over the full image width the
signal discriminates almost perfectly. MC-028 profiled those pixels *inside the
pipeline rect*, which on this tree is 2 to 10 px wider than the column, so there
were almost no pixels to read — exactly the gap MC-031 exists to close. The
direction was untested, it is now tested, and it fails for a different reason.

## 4. What the chrome signal locates — and what the manifest marks

`cargo run --bin ceiling`, the "perfect chrome oracle" section: the located
viewport boundary taken as the answer, with no rule error at all.

```
file                       chromeEnd  taskbar    dTop    dBot
2025-08-05 00_11_13.webp   declines            (see 5e)
2025-08-05 00_11_27.webp   declines
2025-10-14 23_29_06.png         167     1400     +21     +74
2025-10-14 23_30_20.png         167     1400      +4     +21
2025-10-20 15_37_25.png         167     1400     +61     +98
2026-01-05 13_33_41.png         167     1400     +66    +107
2026-01-05 13_45_59.png         167     1400     +37     +29
2026-01-05 13_49_39.png         167     1400    +119      +4
Screenshot (67).png             167     1392    +123      +4
Screenshot (70).jpg             167     1392    +131     +82
Screenshot (75).png             167     1392      +4     +13
Screenshot (93).jpg             167     1392     +45     +13
Screenshot (103).jpg            167     1392    +135    +246
Screenshot (1661).png           133     1392     +59    +115
Screenshot (2582).jpg           133     1392     +83    +168
Screenshot (2630).jpg           133     1392     +91    +279
Screenshot (2698).jpg           133     1392    +210    +307
Screenshot (2708).jpg           133     1392     +87     +70
Screenshot (2744).jpg           133     1392      +1     +94
Screenshot (3187).png           137     1392      +5     +13
Screenshot (3538).png           137     1392     +34     +62

viewport boundary as the answer: 0/21 right, 0 clips; top edge 4/21, bottom 2/21
```

Read the two located columns, not the offsets. **The chrome end takes four
values across nineteen entries — 133, 137, 167 and (at a narrower margin, 5e)
115 — and the taskbar two, 1392 and 1400.** Those are browser and Windows
geometry, constant per session, and they cross-validate against the only two
independent readings on record: MC-019 `## Notes` measurement 2 puts both
2025-10-14 / 2025-10-20 files' chrome end at row 166 (here: the first page row is
167), and MC-028 section 5a puts `2025-08-05 00_11_13.webp`'s at 115 (here: 115).

The **marks** run 114…343 at the top and 1085…1396 at the bottom. They are not
browser geometry. They are where a person drew a box round a **panel**, which is
what MC-026's correction records the user confirming: "the content outside a
marked rect *vertically* is not artwork. It is panel gutter."

So the chrome signal answers a different question from the one the manifest
asks, and the residual between the two answers is panel gutter. **This was
checked by looking, not only by arithmetic.** `cargo run --bin cutouts` writes
the crop the chrome rule produces; the corpus's worst entry,
`Screenshot (2698).jpg` at +210 / +307, comes out carrying **no browser chrome
and no taskbar** — a white panel gutter above the panel and the next panel's
speech-bubble below. `2025-10-20 15_37_25.png` (+61 / +98) likewise: a bubble
tail from the panel above, and the next panel's bubble below. The defect MC-019
`## Notes` measurement 4 identified — every output carrying browser or OS chrome
— is entirely removed by this signal. What replaces it is MC-028's problem.

## 5. The evidence

### 5a. Rule family 1 — the chrome boundary as the answer: best 2 of 21, 2 clips

`cargo run --bin score`. Four selectors × two statistics × 5 thresholds × 4
minimum run lengths × 4 margin geometries = **640 parameterisations**, every one
scored two-axis through MC-027's columns and the rest of `decide`, over the
final margin-expanded clamped rect.

| Selector | Places the edge at | What ranks its candidates |
|---|---|---|
| `OuterRuns` | start of the first page-like run / end of the last | distance from the image edge |
| `ChromeStrip` | end of the leading non-page-like run / start of the trailing one | adjacency to the image edge, bridging short blips |
| `LongestRun` | the bounds of the single longest page-like run | run length |
| `CentreRun` | the bounds of the run holding the image's vertical midpoint | holding the centre |

Each also in a `SelfMed` variant, which asks whether the row's margin is flat
about *its own* median rather than about the page background tone — the variant
that survives a wallpaper or a gradient.

```
best by right          : 2/21 right, 2 clips   [OuterRuns/Bg th 1.00 run 16 gap 0 k 64]
best clip-free         : 0/21 right, 0 clips   (248 of 640 clip nothing)
best top edge alone    : 10/21          best bottom edge alone: 4/21
family ceiling, cherry-picking the best parameterisation PER FILE: 2/21
```

The ceiling line is the strongest statement about this family, because it bounds
it rather than sampling it: **no selector over the chrome boundary can reach
3 of 21, whatever selector is invented.**

### 5b. Rule family 2 — the chrome boundary as an *anchor*: best 5 of 21, 5 clips

The interesting family, and the one built specifically to answer MC-028 section
5f. If the ordinal of the correct change is the problem, a chrome boundary fixes
the origin: "the first art row **below** the chrome" is ordinal 1 by
construction. MC-028's `FirstArt` (section 5b) counted from the *rect's* edge,
which is inside the chrome, so it stopped on the chrome itself; counted from the
chrome's far side it is a different rule.

`cargo run --bin anchored`. Anchor threshold × anchor margin width × 7 art
statistics (MC-025's `dev ≥ {2, 4, 8}` and MC-026 finding 4's `flat < {0.40,
0.60, 0.90, 0.99}`) × 4 minimum art run lengths × 4 outward backoffs =
**448 parameterisations**.

```
best by right          : 5/21 right, 5 clips   [anchor th 1.00 k 64, art flat<0.90, run 8, backoff 0]
best clip-free         : 1/21 right, 0 clips   (40 of 448 clip nothing)
best top edge alone    : 13/21         best bottom edge alone: 8/21
family ceiling, cherry-picked PER FILE: top 17/21, bottom 13/21, BOTH by one member 9/21
```

The anchor helps and it is not enough. It lifts the best single rule from 2 to 5
and the per-file ceiling from 2 to 9, and every rule above 1 of 21 clips.

### 5c. The ceiling of everything measured here

`cargo run --bin ceiling`, over both families at once — **1312 parameterisations**
— and over all 1312 × 1312 pairings of a top rule with a bottom rule:

```
best SINGLE rule                                     : 5/21 right, 5 clips
best TOP edge over everything                        : 13/21
best BOTTOM edge over everything                     :  8/21
best HYBRID (one top rule x one bottom rule)         :  6/21
the perfect chrome oracle (section 4)                :  0/21 right, 0 clips
```

The hybrid number matters because it is a freedom MC-028's family did not have —
MC-028 section 5b records that "no rule is best at both" edges and treats that as
a limit; here it is lifted, and 6 of 21 is what lifting it buys. It still does
not beat 8 of 21.

**Against the three baselines, explicitly.** Best single rule 5 of 21 *ties*
MC-026 finding 1's 5 of 21 and does not beat it, while clipping 5 where finding
1's measurement was a ceiling with the other axis given free. Best hybrid 6 of
21 does not beat finding 1b's 8 of 21. Best clip-free 1 of 21 does not beat
either. Against MC-026 finding 5's 0 of 21 with 17 to 20 clips, every rule here
is better on clips — the chrome signal does not cut inside panels — and that is
the only baseline anything here beats.

### 5d. The ordinal finding, confronted head-on (AC-3, item 4)

MC-028 section 5f's obstacle, asked of this instrument. At the threshold section
3c measured (0.815), `cargo run --bin separate full`:

```
crossings per image, sorted: [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,4,4,5,6,10,11]
the correct crossing's ordinal: k = 1 at the top and k = 2 at the bottom on 15 of 21
distance from a marked edge to the nearest crossing:
  [1,1,2,2,3,4,4,4,4,4,5, 13,13,13,21,21,29,34,37,45,59,61,62,66,66,70,74,82,
   83,87,91,94,98,107,115,119,135,168,210,246,279,307]
marked edges within MC-019's 11 px of ANY crossing: 11/42
```

**This signal has no ordinal problem.** Fifteen of the twenty-one images offer
exactly **two** candidate rows in the whole image, and the ordinal of the
"correct" one is a constant k = 1 / k = 2. Against MC-028's k = 1…16, 3…11,
5…35, 6…30 with 2 to 150 rivals, that is the ranking MC-028 section 5f says
nothing has managed — so the rule this document proposes *does* say what ranks
its candidates, and the answer is "distance from the image edge, of which there
is only one".

And it does not help, because the ranked candidate is the wrong row: only
**11 of 42** marked edges are within 11 px of *any* crossing, against MC-028's
40 of 42 within 11 px of some feature. The two findings are mirror images. MC-028
had every marked edge sitting on a feature and dozens of rivals; MC-031 has no
rivals and two thirds of the marked edges sitting on no feature at all. **A rule
that ranks perfectly among candidates that do not include the answer is not
closer to the answer than one that cannot rank.**

**The control on this statistic** — MC-019 `## Notes` measurement 3's lesson — is
built into the measurement rather than run after it. Measurement 3 was withdrawn
because 99.4 % of *ordinary* rows scored as well as the marks. Here the statistic
is explicitly shown to be **constant across page rows** (section 3c: median 0.993
inside the mark, and 15 of 21 images contain no crossing whatsoever between the
two viewport edges). That is the reason the distances above are large; it is
stated as the signal's limitation, not hidden behind a favourable percentage.

### 5e. Where the instrument itself is weak

Two entries, `2025-08-05 00_11_13.webp` and `2025-08-05 00_11_27.webp`, have a
margin whose median wanders (31, 59, 36, 37…) instead of holding one tone, so at
a threshold of 0.98 or above over the full margin width the rule **declines to
speak** (`cargo run --bin dump -- "2025-08-05 00_11_13.webp" 0 200`). Reading a
narrower margin — 64 px each side — recovers them and puts the chrome end at 115,
matching MC-028 section 5a's independent reading of the same file. Both
geometries are in the family and both are scored; neither changes the verdict.
The narrow margin is also what makes rule 4's top edge reach 10 of 21 while the
full margin reaches 4 — it is reading the page's own light/dark boundary beside
the column rather than chrome, which is why it also clips.

### 5f. A correction to this story's own premise

MC-031's `## Context`, quoting MC-019 `## Notes` measurement 5, says "a naive
bottom scan lands on the taskbar row at 1399 or 1439 repeatedly and puts 2 of 21
within 11 px of the mark — and a naive top scan is 0 of 21". Measured here
(`cargo run --bin ceiling`) the taskbar rows are **1392 and 1400**, not 1439 —
1439 is the shipped rect's own bottom, three to forty-eight rows *inside* the
taskbar — and the top scan is **4 of 21**, not 0, while the bottom is 2 of 21 as
recorded. The premise understated the top edge and overstated the bottom. It does
not change the answer: jointly the boundary is 0 of 21.

## 6. The negative controls (AC-4), and each is shown to fire

`cargo run --bin controls`. All three are run against every parameterisation of
both families, and each is demonstrated firing rather than passing vacuously.

**(a) A panel containing a flat full-width row** — MC-026 finding 5's defect. A
synthetic 1000 × 600 page: browser chrome in rows 0–40 and a taskbar in 560–600,
both full width; a page column at x 400–600; gutter at tone 250; **a flat
full-width block at that same tone in rows 100–140, inside the panel**; faint
low-contrast art in rows 70–100 and 500–530 and real art between. The page is
rows 40–560 and the panel is rows 70–530. A rule holds iff it places neither
edge in 100–140.

```
chrome-only family : control fires on   0 of 640 parameterisations (all 640 spoke)
anchored family    : control fires on 432 of 672 parameterisations
  first firing     : anchored th 0.90 k full art dev>=4 run 1 backoff 4 -> rows 136..504
```

The control fires on two thirds of the anchored family, so it discriminates. The
chrome-only family is **immune by construction** — it reads only the margin and
never looks inside the panel — and that is reported as a property, not as a pass.

**(b) The seven `"expect": "flag"` entries.** Six are `Flagged` on this tree;
`2025-02-27 22_46_15.png` is already `Cropped`, so **1 of 7 is the no-regression
baseline** and 2 or more is a rejection.

```
chrome-only family : fires on   0 of 640; worst 1/7
anchored family    : fires on 280 of 672; worst 5/7  [anchored th 0.90 k full, art flat<0.40, run 1, backoff 0]
```

The control fires, hard, on 280 anchored parameterisations. It is the
independent reason rule 3 in section 1 is rejected (3 of 7) even before its nine
clips are counted.

**(c) A page with no browser chrome at all.** Two forms, because the first turns
out not to discriminate.

*c1, the plain form*: the same synthetic with no chrome and no taskbar — flat
margins, art in the column, nothing at either image edge. Neither family fires
(0 of 640, 0 of 672). Reported as what it is: the pipeline's own uniform trim has
already placed those rows, so the control **cannot** fire and passes vacuously.

*c2, the form that discriminates*: still no browser chrome and no taskbar, but
the reader site's **own full-width banner** sits inside the page at rows 40–80
and the panel's top 40 rows are a pale low-contrast sky. The whole image is page;
the correct answer is the rect unchanged; anything else is an invented edge.

```
chrome-only family : control fires on 400 of 640
  first firing     : OuterRuns/Bg th 0.90 run 48 gap 0 k full -> rows 80..600
anchored family    : control fires on 576 of 672
  first firing     : anchored th 0.90 k full art dev>=4 run 1 backoff 0 -> rows 40..600
```

The control fires on both families. The seven `"expect": "flag"` corpus entries
are the real-world instance of the same control and they are (b).

**Each of the six named rules of section 1, individually**, against (a) and
(c2) — measured rather than asserted, at the end of `cargo run --bin controls`:

```
1 anchored 1.00/64 flat<0.90 run 8 backoff 0        (a) holds rows 140..500  (c2) FIRES rows 40..600
2 anchored 1.00/64 flat<0.90 run 8 backoff 4        (a) FIRES rows 136..504  (c2) FIRES rows 36..600
3 anchored 1.00/64 flat<0.60 run 2 backoff 0        (a) holds rows 140..500  (c2) FIRES rows 40..600
4 OuterRuns/Bg th 1.00 run 16 k 64                  (a) holds rows  40..560  (c2) holds rows  0..600
5 anchored 0.98/full dev>=4 run 8 backoff 11        (a) FIRES rows 129..511  (c2) FIRES rows 29..600
6 ChromeStrip/Bg th 0.90 run 16 k full (the oracle) (a) holds rows  40..560  (c2) holds rows  0..600
```

**Only rules 4 and 6 — the two pure chrome rules — hold both controls**, and
they are the two lowest scorers, at 2 of 21 and 0 of 21. Every rule that scores
better fails at least one control: rules 2 and 5 cut into the flat full-width
block in (a), and all four anchored rules invent an edge in (c2) by skipping the
pale low-contrast sky at the top of the panel — which is the precise sense in
which an art scan anchored to chrome is no longer keyed to chrome. Rule 3
additionally fails (b) at 3 of 7. This is the clearest single statement of the
trade in this family: **the scores above 2 of 21 are bought with control
failures, not with better placement.**

Note that rules 4 and 6 place the page exactly right on the synthetic in (a) —
rows 40…560, the page between the chrome and the taskbar — which is the
synthetic's own confirmation of section 4: the chrome signal locates the page,
and the corpus marks the panel inside it.

## 7. Why each of the six in section 1 fails

`cargo run --bin detail -- anchored 1.00 64 5 8 0`, and the analogous commands.

1. **anchored `flat < 0.90`, run 8, backoff 0 — 5 of 21, 5 clips.** Right where
   the panel starts within a few rows of the chrome or of the page's own tone
   boundary: `00_11_13` (+2, +7), `00_11_27` (+6, +10), `2630` (+2, +2), `2698`
   (+5, +0), `2708` (+5, +5). Its five clips are the two shapes an art scan
   cannot see: a mark that sits *inside* a gutter the scan runs straight through
   (`23_30_20` −15, `15_37_25` −49, `13_45_59` −1) and a panel whose far end is
   beyond a second gutter the scan stops at (`13_49_39` bottom −38, `3538`
   bottom −98). Two of the five clip by 1 and 15 px, which is the whole margin
   between "right" and the worst defect in the ranking.
2. **the same at backoff 4 — 4 of 21, 4 clips.** Backing off outward converts one
   clip into a loose entry and loses one hit. The trade is monotone across the
   family and never reaches both goals: `cargo run --bin anchored` shows every
   backoff of 8 or 11 clip-free member scoring 1 of 21.
3. **anchored `flat < 0.60`, run 2 — 4 of 21, 9 clips, and it crops 3 of the 7
   flag entries.** A looser art threshold finds the panel edge more often and
   also finds it inside every all-art image, which is what the flag control
   catches. Rejected on section 6 (b) independently of its clips, and it fails
   control (c2) as well.
4. **`OuterRuns` on the margin, th 1.00, k 64 — 2 of 21, 2 clips.** The best pure
   chrome rule, and it is only this good because a 64 px margin is reading the
   page beside the column rather than chrome (5e). Its two clips, `15_37_25`
   (−52) and `3538` (−3), are exactly where that page-side reading disagrees
   with the viewport.
5. **anchored `dev ≥ 4.0`, run 8, backoff 11 — 1 of 21, 0 clips.** Clip-free
   because a strict art definition plus an 11 px outward backoff makes the rule
   place its edges above and below everything it is unsure of. As in MC-028
   section 7, the only clip-free member of the family is the one that barely
   speaks.
6. **the perfect chrome oracle — 0 of 21, 0 clips.** It fails for the reason
   section 4 gives and no other: it locates the browser viewport correctly and the
   manifest marks a panel. Its top edge is 1 to 210 px above the mark and its
   bottom 4 to 307 px below, and every one of those pixels is panel gutter. It
   is the one rule here that fails for a stated, structural reason rather than a
   tuning one, and it is the reason the family has a ceiling.

## 8. Reproducing every number here (AC-5)

The harness is **outside the repository**, as MC-025's, MC-026's and MC-028's
were, so no phase, gate or committed test depends on it:

```
C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\
  ec112bbb-5092-4e7b-a32c-d576e34a9ebb\scratchpad\mc031
```

A cargo crate with path dependencies on `cropper-core` and `cropper-engine`, with
`[workspace]` in its `Cargo.toml` so it does not join the repo workspace. Every
statistic is written from its definition rather than by calling a crate-private
helper; the two deliberate exceptions are `codec::to_luma`, so the plane measured
is the plane the detector sees, and the composition of the crate's public stages
in `stages()`, because the question is what a rule sees when wired into that
pipeline. MC-019's two numbers are computed over the **final, margin-expanded,
clamped** rect (`finalise` + `judge` in `src/lib.rs`), as MC-028 section 5b does,
so no `margin_px` convention can drift between documents.

| Command | What it produced |
|---|---|
| `cargo run --bin base` | section 3a: the shipped baseline, the 20 of 21 row oracle, the per-entry pipeline rect |
| `cargo run --bin separate full` | sections 3c and 5d: separability at 99.70 %, the crossings-per-image table, the 11 of 42 |
| `cargo run --bin mprofile [th] [gap] [minrun]` | the page-like run structure of all 21 over the full image height |
| `cargo run --bin dump -- "<file>" [y0] [y1] [k]` | one entry's banded per-row margin and column statistics (5e) |
| `cargo run --bin chrome [th] [k]` | section 4's located-boundary table at one parameterisation |
| `cargo run --bin score` | section 5a: all 640 chrome-only parameterisations, both numbers, per-edge, flag control, family ceiling |
| `cargo run --bin anchored` | section 5b: all 448 anchored parameterisations, the same |
| `cargo run --bin ceiling` | sections 4, 5c and 5f: 1312 parameterisations, the hybrid pairing, the perfect chrome oracle |
| `cargo run --bin controls` | section 6: all three negative controls and their firing counts |
| `cargo run --bin detail -- anchored <th> <k\|full> <art 0..6> <run> <backoff>` | section 7, one named rule per file with both numbers |
| `cargo run --bin detail -- chrome <Selector> <th> <minrun> <k\|full>` | the same for a family-1 rule |
| `cargo run --bin cutouts` | section 4's images, written to `out/` and looked at |

MC-028's and MC-026's harnesses are at the paths their own documents record;
nothing here re-runs them.

## 9. What it would take for MC-019 to be completable (AC-6)

**No implementation story is recommended for the row axis.** Both pixel-level
directions are now closed: MC-028 closed the panel gutter, and this document
closes full-width chrome. The two failures are complementary (section 5d), and
between them they cover every signal a single row profile of a reader screenshot
carries.

What MC-019 needs is a decision, and the measurements narrow it to four options.

### Option A — split MC-019 by axis

AC-1 (zero clips) and AC-3 pass today. The column axis is at 20 of 21 within
10 px (MC-027). Only AC-2 fails, and only because of the rows. Splitting gives
a story that can be finished now — zero clips, the columns, and the `Tuning`
constants AC-4 records — and leaves the row accuracy as its own story with its
own bar and its own evidence. The `integration` gate is unchanged and still the
gate that fails if either half breaks.

### Option B — descope the row *accuracy* and ship the chrome rule as a *quality* criterion

This is the new option, and MC-028's evidence could not support it. The chrome
rule is not accurate against the marks, but it is **reliable against a different,
testable criterion**: it never clips (0 clips at every conservative
parameterisation), its located boundary lies outside the mark on 19 of 19
entries where it speaks, and section 4's outputs carry **no browser chrome and no
taskbar**. That is exactly the defect MC-019 `## Notes` measurement 4 raised as
the reason the tolerance must not move.

So a story could assert "no output contains browser or OS chrome" instead of
"every output is within 11 px", and that assertion is measurable against the same
corpus without touching the manifest: the located viewport rows are 133/137/167
and 1392/1400, the crop's rows must lie inside them, and the two webp entries
that decline must be handled (5e) or excluded with a named reason. MC-019's 90 %
bar would then move to a later epic, under `## Amendments` with the user.

### Option C — re-mark the manifest's row edges to the page's vertical extent

MC-028 section 9 put two conventions to the user — *tight* (the art) and *loose*
(the browser chrome to the taskbar) — and the user declined to re-mark, choosing
this spike instead. **The evidence has changed and it is worth putting back
once.** Under *loose*, the target is precisely what section 4 measures: one
candidate row per edge, ranked k = 1 and k = 2, 0 clips, and a single threshold
that classifies 99.70 % of rows. A rule exists under that convention; section 4
is it. Under *tight*, nothing measured in either spike reaches the bar.

This is **not** a recommendation to edit the manifest, and nothing here edits it.
It is the observation that the manifest and the chrome signal answer different
questions, and that only one of the two questions has an answer.

### Option D — a signal class outside v1

Layout or semantic segmentation of the strip. Out of scope for v1 (`Out of
scope`, and MC-025's `## Context` already ruled out colour and chroma), and not
costed here.

### What the user would need to decide

**Four images and ten minutes**, not another spike. `cargo run --bin cutouts`
writes the crop the chrome rule produces for `Screenshot (2698).jpg` (the worst
entry, +210 / +307), `2025-10-20 15_37_25.png` (+61 / +98), `Screenshot (67).png`
(+123 / +4) and `Screenshot (2744).jpg` (+1 / +94). The question is one question:
**is a crop that contains the whole page with the browser chrome and the taskbar
removed, but with panel gutter above and below the panel, acceptable output?**

- If **yes**, Option B or C follows and MC-019's AC-2 moves under `## Amendments`.
- If **no**, Option A follows: MC-019 is split, the row half is parked, and the
  row axis stays open until a signal outside both spikes is proposed.

Either way the recorded form is an **MC-019 `## Amendments` entry** — which AC
moves, what it said, what it says now, who approved it and why — exactly as
MC-019's `## Model guidance` anticipates.

**Until that is decided, no fourth story should attempt the row axis.** Three
have now been measured wrong after the fact or before shipping: MC-025's row
locator moved the corpus by zero entries, MC-026's interior-band premise reached
5 of 21, MC-028's panel-gutter family reached 4 of 21 with 6 clips, and this
one's chrome family reaches 5 of 21 with 5 clips and 0 of 21 at its own ceiling.

## 10. Scope

No production or test code was written; the measurement harness is outside the
repository (section 8). `Tuning::default()`, the corpus and
`fixtures/corpus/manifest.json` are untouched — the manifest question in section
9 is put to the user, not applied, and MC-019 `## Notes` measurement 2 is taken as
settling that the manifest is not broken. The 11 px tolerance is read out, not
re-opened. The column axis is taken as MC-027 leaves it and every row number here
was measured through those columns. MC-028's panel-gutter family was not re-run.
Colour and chroma were not revisited. The only files this story writes inside the
repository are this document and MC-031's `## Notes`.
