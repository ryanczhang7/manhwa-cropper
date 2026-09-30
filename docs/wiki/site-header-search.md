# Is a same-tone site header locatable by a rule that works on any reader?

MC-059, a spike under `EPIC-07`. Written 2026-09-30 against `main` at
`25bf5d7` (MC-058 done; MC-056 has moved `h09`, `h11` and `h21` to tuning).
The question: does a rule with **no per-site knowledge** remove `kunmanga`'s
same-tone site header, the miss [MC-051](held-out-score.md) §6 names, when its
parameters are chosen **without seeing any `kunmanga` entry**, and without
clipping or misfiring on the other five readers?

Every number here is over the **marked `tuning` entries** and nothing else.
The harness drops every entry whose `split` is not `tuning`, or whose `expect`
is not a rectangle, before a file is named or decoded, and asserts that 29
remain. No held-out entry was opened, decoded, listed or re-scored (§7).

**This document was written in order.** §2 and §3 (the oracle, the pass line
and the selection rule) were committed before any crop was computed or any
rule scored; later sections were appended after. The git history of this file
is the evidence.

## 1. The verdict: a reasoned negative

**No rule family in this spike finds `kunmanga`'s same-tone header when
`kunmanga` is withheld from its fitting.** The tuning set therefore cannot
justify a reader-agnostic site-header rule, whatever a rule scores with
`kunmanga` in view.

| | In-sample, over 28 | Summed over the six reader folds | `kunmanga` fold (fitted without it) |
|---|---|---|---|
| Identity (today's crop) | 0 clips, **24** free | — | 0 of 4 |
| Family A, full-width line | 0 clips, **24** free (no member ever moves a crop) | 0 clips, 24 | **0 of 4** |
| Family B, small-mark band | 0 clips, **24** free | **1 clip**, 24 | **0 of 4** |
| Ceiling | 0 clips, **28** | | |
| Pass line | 0 clips, **26** | 0 clips, 26 | more than identity's 0 |

Both controls fire: the memorising control scores 27 of 28 in-sample and 0 of
4 on the `kunmanga` fold; the blind offset is 1 row and scores 24.

**Why, by entry (§7):**

1. **Family A never fires.** Its art test is a share of the row, and
   `kunmanga`'s menu line ("ROMANCE MANHUA ..." on rows 185..192 of `(70)`)
   covers up to 41 % of the column, so at every `F` up to 0.30 the menu reads
   as art, and no line qualifies above it. That is a row statistic failing the
   way MC-032 recorded.
2. **Family B finds the header, but not on 0 clips.** 24 of its 54 members
   clear at least 2 of the 4 `kunmanga` entries, and the 12 with `τ` = 40
   clear 3 (27 of 28 overall, one short of the ceiling: `(68)`'s art rises to
   the header's line). Every one of the 24 also
   cuts `2025-08-05 00_11_13.webp` by 1 row: that crop still holds the browser
   bar (MC-048's named WebP limitation), the rule removes the bar down to row
   115, and the mark starts at 114, the bar's own bottom border (flat, luma 48).
3. **Even with that row set aside, the `kunmanga` fold fails.** A what-if,
   reading that mark's top as 115, is **not** a score and changes nothing in the
   oracle. Under it, Family B meets the pass line in-sample (0 clips, 27), and
   the `kunmanga` fold still selects a member that finds nothing: **0 of its
   tied-best members** beat identity on `kunmanga`, in either tie-break order.
   The toongod fold also clips. With no same-tone header in the fitting set,
   nothing rewards a member for firing, and step 3 of the selection rule
   penalises the ones that do, because they trim the WebPs' browser bar as
   plain page.

**What that establishes, and what it does not.** It establishes that the 24
non-`kunmanga` tuning entries carry no signal that selects a header rule:
the only fold that tests *finding* a header finds none, for either family.
It does **not** show that no reader-agnostic rule exists; two families were
tried, as the timebox allows, and the sweep was not widened after scoring. And
the one in-sample success needs a mark question answered first (the WebP's
row 114), which this spike does not decide.

**The decision this forces is the user's** (§8): more captures of sites with
same-tone headers before any rule is fitted, or opening per-site matching on
top of a general rule (the brief allows it only there, MC-041), or learned
detection.

## 2. The furniture oracle (AC-1)

All 29 marked tuning entries, each with a ruled site-header end and site-footer
start. "none" means no site bar on that side: MC-050's round 2 stores it as
`null`; MC-051's and MC-059's pages store four lines, and a bar that is not
there was saved as a line on the browser bar's (or taskbar's) own row, a bar of
zero height. The harness normalises both to "none". Mark rows are
`manifest.json`'s `expect`, `y .. y + h`.

**Sources.** 21 from MC-050's round 2 (Artifact
https://claude.ai/artifact/JxL6XQMUtWKrdjru1umiVp, collection `rulings2`,
ruled 2026-09-24; the records do not store a user id, and the page was private
to its owner, the user). 3 from MC-051's frozen rulings (Artifact
https://claude.ai/artifact/1cTq9VKYMjD3jGbJc3Cvqy, collection `rulings2`,
ruled 2026-09-29 by `u_kwpZVcE6UJhWvzvk33yzGw`, the user). 5 ruled for this
story (Artifact https://claude.ai/artifact/CR2iXD7kXcRukUoRTzLNU9, collection
`rulings-mc059`, 2026-09-30, by `u_kwpZVcE6UJhWvzvk33yzGw`). All were read
back with `ArtifactData`, never copied by hand.

The MC-059 page drew only the user's own lines: no crop, no viewport, no mark
and no proposal, and every line started unplaced. `Screenshot (73).png` was
first saved at 294, one row below its mark's top and one below its two
siblings; it was put back to the user in plain words **without a proposed
value**, and they re-checked it on the page and saved 293 (document version 3,
14:44 UTC). The other four are unchanged from their first save.

| # | File | Reader | Mark rows | Site header ends | Site footer starts | Source (doc) | Ruled at | Note |
|---|---|---|---|---|---|---|---|---|
| 1 | `2025-08-05 00_11_13.webp` | rolia-scans | 114..1330 | none | none | MC-050 round 2 (`e00`) | 2026-09-24 |  |
| 2 | `2025-08-05 00_11_27.webp` | rolia-scans | 118..1343 | none | none | MC-050 round 2 (`e01`) | 2026-09-24 |  |
| 3 | `2025-10-14 23_29_06.png` | demonicrevolution | 188..1326 | none | none | MC-050 round 2 (`e02`) | 2026-09-24 |  |
| 4 | `2025-10-14 23_30_20.png` | demonicrevolution | 171..1379 | none | none | MC-050 round 2 (`e03`) | 2026-09-24 |  |
| 5 | `2025-10-20 15_37_25.png` | toongod | 228..1302 | none | none | MC-050 round 2 (`e04`) | 2026-09-24 |  |
| 6 | `2026-01-05 13_33_41.png` | toongod | 233..1293 | none | none | MC-050 round 2 (`e05`) | 2026-09-24 |  |
| 7 | `2026-01-05 13_45_59.png` | demonicrevolution | 204..1371 | none | none | MC-050 round 2 (`e06`) | 2026-09-24 |  |
| 8 | `2026-01-05 13_49_39.png` | demonicrevolution | 286..1396 | none | none | MC-050 round 2 (`e07`) | 2026-09-24 |  |
| 9 | `Screenshot (67).png` | kunmanga | 293..1388 | 293 | none | MC-050 round 2 (`e08`) | 2026-09-24 |  |
| 10 | `Screenshot (70).jpg` | kunmanga | 298..1310 | 293 | none | MC-050 round 2 (`e09`) | 2026-09-24 |  |
| 11 | `Screenshot (75).png` | toongod | 171..1379 | none | none | MC-050 round 2 (`e10`) | 2026-09-24 |  |
| 12 | `Screenshot (93).jpg` | toongod | 212..1379 | none | none | MC-050 round 2 (`e11`) | 2026-09-24 |  |
| 13 | `Screenshot (103).jpg` | toongod | 302..1146 | none | none | MC-050 round 2 (`e12`) | 2026-09-24 |  |
| 14 | `Screenshot (1661).png` | toongod | 192..1277 | none | none | MC-050 round 2 (`e13`) | 2026-09-24 |  |
| 15 | `Screenshot (2582).jpg` | toongod | 216..1224 | none | none | MC-050 round 2 (`e14`) | 2026-09-24 |  |
| 16 | `Screenshot (2630).jpg` | toongod | 224..1113 | none | none | MC-050 round 2 (`e15`) | 2026-09-24 |  |
| 17 | `Screenshot (2698).jpg` | toongod | 343..1085 | none | none | MC-050 round 2 (`e16`) | 2026-09-24 |  |
| 18 | `Screenshot (2708).jpg` | toongod | 220..1322 | none | none | MC-050 round 2 (`e17`) | 2026-09-24 |  |
| 19 | `Screenshot (2744).jpg` | w-network | 134..1298 | none | none | MC-050 round 2 (`e18`) | 2026-09-24 |  |
| 20 | `Screenshot (3187).png` | w-network | 142..1379 | none | none | MC-050 round 2 (`e19`) | 2026-09-24 |  |
| 21 | `Screenshot (3538).png` | toongod | 171..1330 | none | none | MC-050 round 2 (`e20`) | 2026-09-24 |  |
| 22 | `2025-03-06 01_22_45.png` | toongod | 118..1362 | none | none | MC-059 (`n01`) | 2026-09-30 |  |
| 23 | `2025-03-07 00_58_06.png` | toongod | 362..1290 | none | none | MC-059 (`n02`) | 2026-09-30 |  |
| 24 | `2025-07-17 14_20_23.png` | xbato | 171..1382 | none | none | MC-051 frozen (`h09`) | 2026-09-29 | out of the scored set |
| 25 | `2025-07-17 14_41_58.png` | xbato | 273..1369 | none | none | MC-059 (`n03`) | 2026-09-30 |  |
| 26 | `2025-07-17 14_55_10.png` | xbato | 171..1395 | none | none | MC-059 (`n04`) | 2026-09-30 |  |
| 27 | `2025-08-03 11_27_49.png` | rolia-scans | 118..1385 | 116 | none | MC-051 frozen (`h11`) | 2026-09-29 | one-row line under the browser bar (115..116) |
| 28 | `Screenshot (68).png` | kunmanga | 392..1336 | 319 | none | MC-051 frozen (`h21`) | 2026-09-29 |  |
| 29 | `Screenshot (73).png` | kunmanga | 293..1379 | 293 | none | MC-059 (`n05`) | 2026-09-30 |  |

**Counts.** 29 rows; **28 scored** (the detector declines `2025-07-17
14_20_23.png`, `Ambiguous`, so no furniture rule can change its crop; it is
reported on its own line and counts toward nothing). Per reader, scored:
`toongod` 13, `demonicrevolution` 4, `kunmanga` 4, `rolia-scans` 3, `xbato`
2, `w-network` 2.

**A site header is ruled on 5 entries:** all four `kunmanga`,
`Screenshot (67)` (293), `(68)` (319), `(70)` (293) and `(73)` (293), and
`2025-08-03 11_27_49.png` (`rolia-scans`), the one-row line at 115..116. No
entry has a site footer.

**`Screenshot (67).png`'s conflict is resolved by [MC-060](../backlog/stories/MC-060.md).**
Its header is ruled to end at 293, and its mark used to start at 290. §5
shows why that could not stand, and the user had the mark moved to 293
(merged 2026-09-30, PR #73). The table above reads the manifest after
MC-060, and no entry now has a ruled header end below its mark's top.

**The refusal check fires.** With `n05.json` withheld from `oracle/`:

```
REFUSING TO SCORE: no furniture ruling for 1 of 29: ["Screenshot (73).png"]
exit 2
```

Restored, the same binary prints 29 rows and exits 0.

## 3. Before any score: the tests, the pass line and the selection rule

**Crop.** `main`'s crop at `Tuning::default()`: `cropper_core::decide` on
`cropper_engine::codec::to_luma` of the decoded file; a flagged file's crop is
the whole image (it is copied unchanged). Rows `top .. bottom`, bottom
exclusive.

**Clip.** Rows of the mark outside the crop: `max(0, top - mark.top) +
max(0, mark.bottom - bottom)`. An entry clips when that is above 0.

**Furniture-free** (MC-051's exact test, reader part only): with `T` the ruled
site-header end (the crop's own top where "none") and `B` the ruled
site-footer start (the crop's own bottom where "none"), `top >= T` and
`bottom <= B`, exact to the row. Browser chrome and the taskbar are MC-048's
and are not scored here.

**Two counts, one pass line (open question 3).** The one-row `rolia-scans`
line (115..116 on `2025-08-03 11_27_49.png`) is counted both ways:
**"set aside"** treats it as "none"; **"with"** requires the crop to start at
or below 116. The pass line applies to **set aside**. Both are reported
everywhere a furniture-free count is.

**The pass line:** the brief's 9 in 10, rounded up, over 28:
**0 clips on 28 of 28 (absolute) and furniture-free on at least 26 of 28**
(`ceil(0.9 × 28) = ceil(25.2) = 26`). No open question's answer moved the
denominator, so it stands as the story wrote it.

**The selection rule (AC-3), for every family, applied to whatever set a
member is fitted on:**

1. **Zero clips first.** A member that clips any entry in the fitting set is
   not eligible.
2. **Then most furniture-free** on the fitting set (the "set aside" count).
3. **Then fewest rows of plain page trimmed:** the sum, over fitting entries
   ruled "no site header", of rows the rule removes from the top of `main`'s
   crop (and likewise at the bottom over entries ruled "no site footer").
4. **Then the first member in the family's declared sweep order**, which each
   family states with its definition, before it is scored.

**Out, by construction:** any per-site constant or lookup; a reader or file
name anywhere in a rule; colour (the pixels are the luma `detect` already
reads); any learned model.

**What AC-2 expects, to be measured and not assumed:** identity 23 to 25 of
28 furniture-free, 0 clips; zero-clip ceiling 27 of 28 (`Screenshot (67)` the
one miss). *(Written before MC-060. After it, the ceiling's expected miss is
gone; §4 has both measurements.)*

## 4. The baseline and the ceiling (AC-2), and the scorer's controls (AC-5)

`main` at `24e00c1` (MC-060 merged; the crop code is `25bf5d7`'s, and MC-060
changed only the manifest), `Tuning::default()`. Output of `baseline` (after
`crops`), verbatim. "free" is `yes`/`NO`, or `y/N` where the two counts
differ ("set aside" / "with"); "decision" carries the crop's columns.

```
== AC-2 (a): identity rule - main's crop, no new stage ==
file                         reader                    mark        crop    hdr   clip   free decision      
2025-08-05 00_11_13.webp     rolia-scans          114..1330     18..1440       -      0    yes crop x 953..1593
2025-08-05 00_11_27.webp     rolia-scans          118..1343     18..1440       -      0    yes crop x 1006..1539
2025-10-14 23_29_06.png      demonicrevolution    188..1326    167..1400       -      0    yes crop x 1003..1543
2025-10-14 23_30_20.png      demonicrevolution    171..1379    167..1400       -      0    yes crop x 1033..1513
2025-10-20 15_37_25.png      toongod              228..1302    167..1400       -      0    yes crop x 1007..1539
2026-01-05 13_33_41.png      toongod              233..1293    167..1400       -      0    yes crop x 1074..1472
2026-01-05 13_45_59.png      demonicrevolution    204..1371    167..1400       -      0    yes crop x 1039..1506
2026-01-05 13_49_39.png      demonicrevolution    286..1396    167..1400       -      0    yes crop x 1039..1506
Screenshot (67).png          kunmanga             293..1388    167..1392     293      0     NO crop x 1010..1535
Screenshot (70).jpg          kunmanga             298..1310    167..1392     293      0     NO crop x 1010..1535
Screenshot (75).png          toongod              171..1379    167..1392       -      0    yes crop x 1073..1473
Screenshot (93).jpg          toongod              212..1379    167..1392       -      0    yes crop x 1139..1406
Screenshot (103).jpg         toongod              302..1146    167..1392       -      0    yes crop x 1073..1473
Screenshot (1661).png        toongod              192..1277    133..1392       -      0    yes crop x 1073..1473
Screenshot (2582).jpg        toongod              216..1224    133..1392       -      0    yes crop x 1006..1539
Screenshot (2630).jpg        toongod              224..1113    133..1392       -      0    yes crop x 955..1591
Screenshot (2698).jpg        toongod              343..1085    133..1392       -      0    yes crop x 953..1592
Screenshot (2708).jpg        toongod              220..1322    133..1392       -      0    yes crop x 1074..1473
Screenshot (2744).jpg        w-network            134..1298    133..1392       -      0    yes crop x 948..1596
Screenshot (3187).png        w-network            142..1379    137..1392       -      0    yes crop x 984..1560
Screenshot (3538).png        toongod              171..1330    137..1392       -      0    yes crop x 975..1571
2025-03-06 01_22_45.png      toongod              118..1362    115..1374       -      0    yes crop x 643..1176
2025-03-07 00_58_06.png      toongod              362..1290    115..1399       -      0    yes crop x 670..1150
2025-07-17 14_41_58.png      xbato                273..1369    167..1400       -      0    yes crop x 928..1618
2025-07-17 14_55_10.png      xbato                171..1395    167..1400       -      0    yes crop x 928..1618
2025-08-03 11_27_49.png      rolia-scans          118..1385    115..1400     116      0    y/N crop x 1006..1539
Screenshot (68).png          kunmanga             392..1336    167..1392     319      0     NO crop x 958..1587
Screenshot (73).png          kunmanga             293..1379    167..1392     293      0     NO crop x 1003..1543
identity: clips on 0/28; furniture-free 24/28 (set aside), 23/28 (with)
pass line (0 clips on 28, >= 26 free, set aside): not met
per reader: entries, zero-clip, free (set aside), free (with)
  demonicrevolution   4  4  4  4
  kunmanga            4  4  0  0
  rolia-scans         3  3  3  2
  toongod            13 13 13 13
  w-network           2  2  2  2
  xbato               2  2  2  2
out of scope: 2025-07-17 14_20_23.png crop 0..1440 flag Ambiguous, clip 0 (counts toward nothing)

== AC-2 (b): the ceiling - crop exactly to the ruled rows ==
ceiling: exact cut clips 0/28; zero-clip ceiling furniture-free 28/28 (set aside), 28/28 (with)
per reader: entries, ceiling free (set aside), ceiling free (with)
  demonicrevolution   4  4  4
  kunmanga            4  4  4
  rolia-scans         3  3  3
  toongod            13 13 13
  w-network           2  2  2
  xbato               2  2  2

== AC-5 control 1: ruled rows moved 10 rows inward (the clip check is live) ==
clips on 11/28 (must be >= 1): FIRES ["2025-10-14 23_30_20.png", "2026-01-05 13_49_39.png", "Screenshot (67).png", "Screenshot (70).jpg", "Screenshot (75).png", "Screenshot (2744).jpg", "Screenshot (3187).png", "2025-03-06 01_22_45.png", "2025-07-17 14_55_10.png", "2025-08-03 11_27_49.png", "Screenshot (73).png"]

== AC-5 control 2: the identity rule reports furniture exactly where a ruled site bar is inside today's crop ==
ruled bar inside today's crop: ["Screenshot (67).png", "Screenshot (70).jpg", "2025-08-03 11_27_49.png", "Screenshot (68).png", "Screenshot (73).png"]
identity reports furniture (with): ["Screenshot (67).png", "Screenshot (70).jpg", "2025-08-03 11_27_49.png", "Screenshot (68).png", "Screenshot (73).png"]
FIRES (same entries, non-empty)
```

- **Identity (a):** 0 clips on 28; furniture-free **24 of 28** set aside,
  **23 of 28** with. Inside the predicted 23 to 25. The misses are all four
  `kunmanga` entries, and, "with", the `rolia-scans` line. **The pass line is
  not met, so the spike does not stop here.**
- **Ceiling (b):** the exact cut clips **0 of 28**; the zero-clip ceiling is
  **28 of 28** both ways.
- **AC-5, both controls fire.** Moving the ruled rows 10 inward clips 11 of
  28. The identity rule reports furniture on exactly the 5 entries whose ruled
  site bar lies inside today's crop.
- `2025-07-17 14_20_23.png`: flagged `Ambiguous`, crop is the whole image,
  0 clips; counts toward nothing.
- **Blind offset, read off this table:** the largest fixed trim below the
  crop's top that keeps 0 clips is **1 row** (`Screenshot (2744).jpg`, crop
  133, mark 134). It reaches no `kunmanga` header (the nearest needs 126 rows).

**Before MC-060** (the run §5 was read from, `(67)` marked from 290): the same
table, except `(67)`'s mark was 290..1388, and the ceiling line read `exact cut
clips 1/28` (`(67)`, 3 rows), `zero-clip ceiling furniture-free 27/28` both
ways, `kunmanga 4 3 3`. Identity and both controls were identical. The crops
did not change: MC-060 edited no source.

## 5. Before any family: `Screenshot (67)`'s old mark made the pass line unreachable

The pixel reading below was done before any rule family was defined or
scored. It changes what AC-2's numbers mean.

**The `kunmanga` header ends the same way on every entry.** A full-width
line of luma about 235 closes it (rows 292..293 on `(67)`, `(70)`, `(73)`;
317..318 on `(68)`), and on `(67)`, `(70)` and `(73)` the art starts on row
293 inside the column (`rows` dump; `compare`, column deviation from white).

**`(67)` and `(73)` are byte-identical over the full width on rows 281..292**
(both PNG; `compare "(67)" "(73)" 284 296 1012 1535`):

```
  290  col-dev (67)   0 | (73)   0   full-width max|a-b|   0, pixels >8: 0
  291  col-dev (67)   0 | (73)   0   full-width max|a-b|   0, pixels >8: 0
  292  col-dev (67)  20 | (73)  20   full-width max|a-b|   0, pixels >8: 0
  293  col-dev (67) 192 | (73) 199   full-width max|a-b| 175, pixels >8: 458
```

Over rows 167..292 they differ only on rows 194..219 and 246..280, the menu
text and the chapter selector. `(70)` (JPEG) matches `(67)` to within 9 luma
levels on rows 284..292.

**What that forces.** The user ruled `(73)`'s header to end at 293, so rows
290..292 are header there, and a furniture-free crop of `(73)` must start at
exactly 293 (its mark starts at 293). `(67)`'s mark claims the same rows,
290..292, which hold no art (column deviation 0 on 290..291, the header's
line on 292), so a zero-clip crop of `(67)` must start at or above 290. A
rule that reads where the header ends sees the same rows on both entries and
the art beginning on the same row, so it gives both the same top. Then:

- if it removes the header on `(70)` or `(73)`, it cuts rows 290..292 from
  `(67)`'s mark: **a clip, and zero clips is absolute**;
- if it does not, the most it can add to the identity rule's 24 is `(68)`:
  **25 of 28, under the 26 the pass line needs**.

Only a rule keyed to `(73)`'s menu text, or to the art's own content, could
split the two. Neither is a furniture rule, and the first is per-file
knowledge. **So with `(67)`'s mark as it is, no reader-agnostic rule can meet
the pass line in-sample, and AC-6's positive verdict is out of reach before
any family is scored.** Open question 2 was answered on the premise that
`(67)` costs one miss that the pass line allows for. The premise is false:
it costs the three entries that share its header.

**Decision (the user, 2026-09-30).** Put in plain words: the art box starts
at 290, the art at 293, and rows 290..292 are blank page plus the thin grey
line under the menu, the same pixels as on `(73)`, where they marked them as
header. The options were fixing the box first, keeping it and finishing now
(a negative for this reason alone), or leaving `(67)` out of the count (an
amendment to the scored set). The user chose **"Fix the box first"**. That is
[MC-060](../backlog/stories/MC-060.md), a chore that moves the mark's top to
293 by `corpus.md`'s marking rule. This spike depends on it. When it merges,
§4 is re-measured against the corrected mark, and no family is scored before
then. §3 (the pass line and the selection rule) does not change.

## 6. The two families, defined before either is scored (AC-3)

Both read only the luma image and `main`'s crop (rows `t .. b`, columns
`x0 .. x1`), and only ever move the crop's **top** down. Neither names a reader
or file, holds a per-site constant, reads colour, or learns. Common terms:

- **`bg`**: the median luma of row `t` (the crop's first row) over the image's
  full width.
- **window**: rows `t .. min(t + D, b)`.

**What the pixels showed while choosing them** (all four `kunmanga` entries,
the pages of the other readers in §4's table): the site header is a full-width
band of small marks (menu words, buttons, a chapter selector) closed by a
full-width line one or two rows thick (luma 235 or 229 on white); the art begins
on or just below that line and lies inside the column. On `(68)` a stroke of
the art rises above its mark and touches the line (row 318; the ruled header
end is 319, the mark 392). Mid-chapter pages of the other readers show nothing
full-width between the crop's top and the art.

### Family A: a full-width line above the art

- **Line row** `y`: over the full width, at least `S` of the pixels lie within
  10 of the row's own median, and that median differs from `bg` by at least
  `δ`.
- **Art row** `y`: at least `F` of the pixels in the crop's columns differ
  from `bg` by more than 24.
- `a` = the first art row in the window (the window's end when there is none).
- **Candidate lines**: maximal runs of line rows, 1 to 3 rows long, starting
  at `t + 1` or lower and before `a`.
- **New top** = `min(e + 1, a)` for the lowest candidate run `s .. e`; `t`
  when there is no candidate.
- **Sweep, 72 members:** `D` ∈ {400, 250, 150}, `S` ∈ {0.90, 0.95, 0.98},
  `δ` ∈ {8, 16}, `F` ∈ {0.30, 0.20, 0.10, 0.05}. **Declared order** (the
  selection rule's step 4) is that nesting, `D` outermost, each list in the
  order written: deepest search, loosest line and art tests first.

### Family B: a band of small marks that ends at the art

- **Foreground**: pixels in the window differing from `bg` by more than `τ`.
  8-connected components over the full width; components under 4 pixels are
  ignored.
- **Art component**: one that reaches into the crop's columns and is at least
  `H` rows tall (within the window).
- `a` = the topmost row of any art component.
- **Band evidence**: the number of non-art components lying wholly above `a`.
- **New top** = `a` when an art component exists, `a > t`, and the band
  evidence is at least `N`; otherwise `t`.
- **Sweep, 54 members:** `D` ∈ {400, 250, 150}, `N` ∈ {1, 3, 6}, `H` ∈ {96,
  48}, `τ` ∈ {40, 24, 16}. **Declared order**: that nesting, `D` outermost,
  each list in the order written: deepest search, least evidence, tallest
  art threshold first.

### Why the declared order matters, and what is reported beside it

Fitted on a set with no site header, as in the `kunmanga` fold, many members
can tie on steps 1 to 3: 0 clips, every entry furniture-free, 0 rows trimmed.
The fold's answer then rests on step 4 alone. The order above is chosen to
prefer members that search deepest on the weakest evidence the other readers
tolerate, since that is what "fitted without positives" can mean. So the
fold's verdict does not depend on that choice alone, each fold also reports,
**order-free**, how many of its tied-best members are furniture-free on the
held-out reader, and what the **reverse** order would have selected. Only the
declared order counts toward AC-6.

### The controls (AC-4)

- **Memorising:** per reader, one fixed number of rows trimmed below the
  crop's top, chosen by the selection rule over `0 ..= 300` on that reader's
  own entries. A reader it has no height for is trimmed by 0.
- **Blind offset:** one fixed trim on every entry, chosen by the selection rule
  over `0 ..= 300` on the fitting set. Expected in-sample: 1 row.

## 7. The scores (AC-3, AC-4), verbatim

Output of `families` at `main` `24e00c1`, run 2026-09-30 after §6 was
committed (`75701f4`). Only the top edge ever moves; no family moves the
bottom, so "per edge" is the top.

```

==================== Family A: a full-width line above the art (72 members) ====================
in-sample selected: A { d: 400, s: 0.9, delta: 8, f: 0.3 } (72 tied on steps 1-3)
in-sample: clips 0/28, free 24/28 (set aside), 23/28 (with), plain-page rows trimmed 0
pass line in-sample: not met
per reader (entries, clips, free aside, free with, trimmed):
  kunmanga            4  0  0  0    0
  toongod            13  0 13 13    0
  demonicrevolution   4  0  4  4    0
  rolia-scans         3  0  3  2    0
  xbato               2  0  2  2    0
  w-network           2  0  2  2    0
per edge: top moved on 0 entries; bottom edge never moved (no family moves it)
per entry, where the selected member moved the top or missed:
  Screenshot (67).png          kunmanga           crop top  167 ->  167  header   293  mark top  293  clip 0  free NO
  Screenshot (70).jpg          kunmanga           crop top  167 ->  167  header   293  mark top  298  clip 0  free NO
  Screenshot (68).png          kunmanga           crop top  167 ->  167  header   319  mark top  392  clip 0  free NO
  Screenshot (73).png          kunmanga           crop top  167 ->  167  header   293  mark top  293  clip 0  free NO

-- leave one reader out (Family A: a full-width line above the art) --
fold kunmanga           selected A { d: 400, s: 0.9, delta: 8, f: 0.3 }
    held-out reader: clips 0/4, free 0/4 (set aside), 0/4 (with), plain-page rows trimmed 0
    identity on it: free 0/4
    order-free: 72 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select A { d: 150, s: 0.98, delta: 16, f: 0.05 }: clips 0, free 0/4
fold toongod            selected A { d: 400, s: 0.9, delta: 8, f: 0.3 }
    held-out reader: clips 0/13, free 13/13 (set aside), 13/13 (with), plain-page rows trimmed 0
    identity on it: free 13/13
    order-free: 72 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select A { d: 150, s: 0.98, delta: 16, f: 0.05 }: clips 0, free 13/13
fold demonicrevolution  selected A { d: 400, s: 0.9, delta: 8, f: 0.3 }
    held-out reader: clips 0/4, free 4/4 (set aside), 4/4 (with), plain-page rows trimmed 0
    identity on it: free 4/4
    order-free: 72 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select A { d: 150, s: 0.98, delta: 16, f: 0.05 }: clips 0, free 4/4
fold rolia-scans        selected A { d: 400, s: 0.9, delta: 8, f: 0.3 }
    held-out reader: clips 0/3, free 3/3 (set aside), 2/3 (with), plain-page rows trimmed 0
    identity on it: free 3/3
    order-free: 72 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select A { d: 150, s: 0.98, delta: 16, f: 0.05 }: clips 0, free 3/3
fold xbato              selected A { d: 400, s: 0.9, delta: 8, f: 0.3 }
    held-out reader: clips 0/2, free 2/2 (set aside), 2/2 (with), plain-page rows trimmed 0
    identity on it: free 2/2
    order-free: 72 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select A { d: 150, s: 0.98, delta: 16, f: 0.05 }: clips 0, free 2/2
fold w-network          selected A { d: 400, s: 0.9, delta: 8, f: 0.3 }
    held-out reader: clips 0/2, free 2/2 (set aside), 2/2 (with), plain-page rows trimmed 0
    identity on it: free 2/2
    order-free: 72 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select A { d: 150, s: 0.98, delta: 16, f: 0.05 }: clips 0, free 2/2
sum over folds: clips 0/28, free 24/28 (set aside), 23/28 (with), plain-page rows trimmed 0
pass line summed over folds: not met

-- failure causes (Family A) --
members free on >= 1 kunmanga entry: 0 of 72
members free per kunmanga entry: ["Screenshot (67).png 0", "Screenshot (70).jpg 0", "Screenshot (68).png 0", "Screenshot (73).png 0"]

==================== Family B: a band of small marks that ends at the art (54 members) ====================
in-sample selected: B { d: 150, n: 6, h: 96, tau: 40 } (2 tied on steps 1-3)
in-sample: clips 0/28, free 24/28 (set aside), 23/28 (with), plain-page rows trimmed 0
pass line in-sample: not met
per reader (entries, clips, free aside, free with, trimmed):
  kunmanga            4  0  0  0    0
  toongod            13  0 13 13    0
  demonicrevolution   4  0  4  4    0
  rolia-scans         3  0  3  2    0
  xbato               2  0  2  2    0
  w-network           2  0  2  2    0
per edge: top moved on 0 entries; bottom edge never moved (no family moves it)
per entry, where the selected member moved the top or missed:
  Screenshot (67).png          kunmanga           crop top  167 ->  167  header   293  mark top  293  clip 0  free NO
  Screenshot (70).jpg          kunmanga           crop top  167 ->  167  header   293  mark top  298  clip 0  free NO
  Screenshot (68).png          kunmanga           crop top  167 ->  167  header   319  mark top  392  clip 0  free NO
  Screenshot (73).png          kunmanga           crop top  167 ->  167  header   293  mark top  293  clip 0  free NO

-- leave one reader out (Family B: a band of small marks that ends at the art) --
fold kunmanga           selected B { d: 150, n: 6, h: 96, tau: 40 }
    held-out reader: clips 0/4, free 0/4 (set aside), 0/4 (with), plain-page rows trimmed 0
    identity on it: free 0/4
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 150, n: 6, h: 96, tau: 24 }: clips 0, free 0/4
fold toongod            selected B { d: 150, n: 6, h: 96, tau: 40 }
    held-out reader: clips 0/13, free 13/13 (set aside), 13/13 (with), plain-page rows trimmed 0
    identity on it: free 13/13
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 150, n: 6, h: 96, tau: 24 }: clips 0, free 13/13
fold demonicrevolution  selected B { d: 150, n: 3, h: 96, tau: 40 }
    held-out reader: clips 0/4, free 4/4 (set aside), 4/4 (with), plain-page rows trimmed 8
    identity on it: free 4/4
    order-free: 3 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 150, n: 6, h: 96, tau: 24 }: clips 0, free 4/4
fold rolia-scans        selected B { d: 400, n: 6, h: 48, tau: 40 }
    held-out reader: clips 1/3, free 3/3 (set aside), 2/3 (with), plain-page rows trimmed 194
    identity on it: free 3/3
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 2 clip it
    reverse order would select B { d: 250, n: 6, h: 48, tau: 40 }: clips 1, free 3/3
fold xbato              selected B { d: 150, n: 6, h: 96, tau: 40 }
    held-out reader: clips 0/2, free 2/2 (set aside), 2/2 (with), plain-page rows trimmed 0
    identity on it: free 2/2
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 150, n: 6, h: 96, tau: 24 }: clips 0, free 2/2
fold w-network          selected B { d: 150, n: 6, h: 96, tau: 40 }
    held-out reader: clips 0/2, free 2/2 (set aside), 2/2 (with), plain-page rows trimmed 0
    identity on it: free 2/2
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 150, n: 6, h: 96, tau: 24 }: clips 0, free 2/2
sum over folds: clips 1/28, free 24/28 (set aside), 23/28 (with), plain-page rows trimmed 202
pass line summed over folds: not met

-- failure causes (Family B) --
members free on >= 1 kunmanga entry: 24 of 54
  B { d: 400, n: 1, h: 96, tau: 40 }: kunmanga free 3/4; over 28: clips 5, free 27, trimmed 767; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 32 (top 244 mark 212)", "Screenshot (1661).png clip 5 (top 197 mark 192)", "Screenshot (3538).png clip 4 (top 175 mark 171)", "2025-08-03 11_27_49.png clip 193 (top 311 mark 118)"]
  B { d: 400, n: 1, h: 96, tau: 24 }: kunmanga free 2/4; over 28: clips 4, free 26, trimmed 544; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 6 (top 218 mark 212)", "Screenshot (1661).png clip 5 (top 197 mark 192)", "Screenshot (3538).png clip 4 (top 175 mark 171)"]
  B { d: 400, n: 1, h: 48, tau: 40 }: kunmanga free 3/4; over 28: clips 3, free 27, trimmed 473; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (3538).png clip 4 (top 175 mark 171)", "2025-08-03 11_27_49.png clip 193 (top 311 mark 118)"]
  B { d: 400, n: 1, h: 48, tau: 24 }: kunmanga free 2/4; over 28: clips 2, free 26, trimmed 276; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (3538).png clip 4 (top 175 mark 171)"]
  B { d: 400, n: 3, h: 96, tau: 40 }: kunmanga free 3/4; over 28: clips 2, free 27, trimmed 432; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 32 (top 244 mark 212)"]
  B { d: 400, n: 3, h: 96, tau: 24 }: kunmanga free 2/4; over 28: clips 2, free 26, trimmed 405; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 6 (top 218 mark 212)"]
  B { d: 400, n: 3, h: 48, tau: 40 }: kunmanga free 3/4; over 28: clips 1, free 27, trimmed 202; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 400, n: 3, h: 48, tau: 24 }: kunmanga free 2/4; over 28: clips 1, free 26, trimmed 201; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 400, n: 6, h: 96, tau: 40 }: kunmanga free 3/4; over 28: clips 2, free 27, trimmed 339; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 32 (top 244 mark 212)"]
  B { d: 400, n: 6, h: 96, tau: 24 }: kunmanga free 2/4; over 28: clips 1, free 26, trimmed 194; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 400, n: 6, h: 48, tau: 40 }: kunmanga free 3/4; over 28: clips 1, free 27, trimmed 194; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 400, n: 6, h: 48, tau: 24 }: kunmanga free 2/4; over 28: clips 1, free 26, trimmed 194; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 250, n: 1, h: 96, tau: 40 }: kunmanga free 3/4; over 28: clips 4, free 27, trimmed 571; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 32 (top 244 mark 212)", "Screenshot (1661).png clip 5 (top 197 mark 192)", "Screenshot (3538).png clip 4 (top 175 mark 171)"]
  B { d: 250, n: 1, h: 96, tau: 24 }: kunmanga free 2/4; over 28: clips 4, free 26, trimmed 544; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 6 (top 218 mark 212)", "Screenshot (1661).png clip 5 (top 197 mark 192)", "Screenshot (3538).png clip 4 (top 175 mark 171)"]
  B { d: 250, n: 1, h: 48, tau: 40 }: kunmanga free 3/4; over 28: clips 3, free 27, trimmed 473; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (3538).png clip 4 (top 175 mark 171)", "2025-08-03 11_27_49.png clip 193 (top 311 mark 118)"]
  B { d: 250, n: 1, h: 48, tau: 24 }: kunmanga free 2/4; over 28: clips 2, free 26, trimmed 276; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (3538).png clip 4 (top 175 mark 171)"]
  B { d: 250, n: 3, h: 96, tau: 40 }: kunmanga free 3/4; over 28: clips 2, free 27, trimmed 432; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 32 (top 244 mark 212)"]
  B { d: 250, n: 3, h: 96, tau: 24 }: kunmanga free 2/4; over 28: clips 2, free 26, trimmed 405; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 6 (top 218 mark 212)"]
  B { d: 250, n: 3, h: 48, tau: 40 }: kunmanga free 3/4; over 28: clips 1, free 27, trimmed 202; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 250, n: 3, h: 48, tau: 24 }: kunmanga free 2/4; over 28: clips 1, free 26, trimmed 201; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 250, n: 6, h: 96, tau: 40 }: kunmanga free 3/4; over 28: clips 2, free 27, trimmed 339; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)", "Screenshot (93).jpg clip 32 (top 244 mark 212)"]
  B { d: 250, n: 6, h: 96, tau: 24 }: kunmanga free 2/4; over 28: clips 1, free 26, trimmed 194; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 250, n: 6, h: 48, tau: 40 }: kunmanga free 3/4; over 28: clips 1, free 27, trimmed 194; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
  B { d: 250, n: 6, h: 48, tau: 24 }: kunmanga free 2/4; over 28: clips 1, free 26, trimmed 194; clipped: ["2025-08-05 00_11_13.webp clip 1 (top 115 mark 114)"]
members free per kunmanga entry: ["Screenshot (67).png 24", "Screenshot (70).jpg 12", "Screenshot (68).png 0", "Screenshot (73).png 24"]

==================== controls (AC-4) ====================
memorising heights: {"demonicrevolution": 0, "kunmanga": 126, "rolia-scans": 0, "toongod": 0, "w-network": 0, "xbato": 0}
memorising in-sample: clips 0/28, free 27/28 (set aside), 26/28 (with), plain-page rows trimmed 0
memorising, kunmanga fold (no height for an unseen reader, trims 0): clips 0, free 0/4 -> FAILS the fold (no more than identity's 0)
blind offset (largest zero-clip trim, AC-4): 1 rows; in-sample: clips 0/28, free 24/28 (set aside), 24/28 (with), plain-page rows trimmed 24
  (the selection rule alone would pick 0 rows)
  one row more (2 rows) clips: ["Screenshot (2744).jpg"]
blind offset vs pass line: FAILS
```

**How to read the folds.** "selected" is the member the selection rule
(§3) picks on the other five readers. "order-free" counts that fold's
tied-best members and how many of them would have beaten identity on the
held-out reader, so it does not depend on the declared order. "reverse
order" is what the opposite order would have picked. In the **`kunmanga`
fold**, both families' tied-best members (72 for A, 2 for B) all leave
`kunmanga`'s header in place: **0 of them** beat identity.

**The what-if** (`MC059_WHATIF_WEBP115=1 families`, the WebP's mark top read
as 115; **not a score**, oracle unchanged), Family B only:

```
WHAT-IF ONLY: 2025-08-05 00_11_13.webp mark top read as 115, not the manifest's 114
==================== Family B: a band of small marks that ends at the art (54 members) ====================
in-sample selected: B { d: 400, n: 6, h: 48, tau: 40 } (2 tied on steps 1-3)
in-sample: clips 0/28, free 27/28 (set aside), 26/28 (with), plain-page rows trimmed 194
pass line in-sample: MET
fold kunmanga           selected B { d: 150, n: 6, h: 96, tau: 40 }
    held-out reader: clips 0/4, free 0/4 (set aside), 0/4 (with), plain-page rows trimmed 0
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 150, n: 6, h: 96, tau: 24 }: clips 0, free 0/4
fold toongod            selected B { d: 400, n: 6, h: 96, tau: 40 }
    held-out reader: clips 1/13, free 13/13 (set aside), 13/13 (with), plain-page rows trimmed 145
    order-free: 4 tied-best members; 0 of them beat identity on this reader, 2 clip it
    reverse order would select B { d: 250, n: 6, h: 48, tau: 40 }: clips 0, free 13/13
fold demonicrevolution  selected B { d: 400, n: 3, h: 48, tau: 40 }
    held-out reader: clips 0/4, free 4/4 (set aside), 4/4 (with), plain-page rows trimmed 8
    order-free: 4 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 250, n: 6, h: 48, tau: 40 }: clips 0, free 4/4
fold rolia-scans        selected B { d: 400, n: 6, h: 48, tau: 40 }
    held-out reader: clips 0/3, free 3/3 (set aside), 2/3 (with), plain-page rows trimmed 194
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 250, n: 6, h: 48, tau: 40 }: clips 0, free 3/3
fold xbato              selected B { d: 400, n: 6, h: 48, tau: 40 }
    held-out reader: clips 0/2, free 2/2 (set aside), 2/2 (with), plain-page rows trimmed 0
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 250, n: 6, h: 48, tau: 40 }: clips 0, free 2/2
fold w-network          selected B { d: 400, n: 6, h: 48, tau: 40 }
    held-out reader: clips 0/2, free 2/2 (set aside), 2/2 (with), plain-page rows trimmed 0
    order-free: 2 tied-best members; 0 of them beat identity on this reader, 0 clip it
    reverse order would select B { d: 250, n: 6, h: 48, tau: 40 }: clips 0, free 2/2
sum over folds: clips 1/28, free 24/28 (set aside), 23/28 (with), plain-page rows trimmed 347
pass line summed over folds: not met
```

**The WebP's row 114** (`rowflat "00_11_13" 108 120 958 1589`): rows 108..113
are the browser bar (luma 59..61), row 114 is its bottom border (luma 48,
share 1.00 over the mark's columns), and the page and art begin on row 115
(share 0.23). The mark includes the border row. That is the same shape as
`(67)`'s old mark (§5), but here it is **not** what decides the verdict (the
what-if above), so it is recorded and not put to the user as a blocker.

## 8. The decision it forces (AC-6)

This spike does not choose. The options, each with what it would take:

1. **More captures of sites with same-tone headers, then fit.** The fold
   that tests finding a header has one reader in it. Screenshots from the top
   of a chapter on other sites (the user has already asked for these, open
   question 5) are what a fitting set needs before any rule can be selected
   without seeing its test reader. This is the same collection `EPIC-07`'s
   owed fresh held-out score needs (an MC-037-shaped story).
2. **Per-site matching on top of a general rule** (MC-041, parked). The brief
   allows it only as an optimisation over a rule that works without it. This
   spike shows no such general rule exists yet on this evidence, so this
   option has nothing to sit on top of.
3. **Learned detection.** Out of the brief's scope today (no learned model,
   decision 3); named for completeness.

Separately, and not blocking: **`2025-08-05 00_11_13.webp`'s mark includes
the browser bar's bottom border (row 114)**. Whether to correct it is a
`corpus.md` question of the MC-060 kind, for the user, if a later story needs
it.

**No feature story is sketched**: AC-6's positive branch was not reached.

## 9. Reproducing every number here (AC-7)

The harness lives **outside the repository**:
`C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\063e672c-416d-4a62-a34c-6480896b3e28\scratchpad\mc059\harness\`,
a Cargo crate depending on this repo's `cropper-core` and `cropper-engine` by
path, so it measures whatever is checked out (`main` at `24e00c1` for §4 and
§7; `25bf5d7` before MC-060). Its `oracle/` holds the 29 rulings as read back
with `ArtifactData`.

```bash
cargo build --release
./target/release/oracle                         # §2's table; exits 2 naming any entry without a ruling
./target/release/crops                          # main's crop on the 29 -> out/crops.json
./target/release/baseline                       # §4: identity, ceiling, AC-5 controls
./target/release/families                       # §7: both families, the folds, AC-4's controls
MC059_WHATIF_WEBP115=1 ./target/release/families  # §7's what-if (not a score)
./target/release/rows|rowflat|compare|strip|zoom ...   # the pixel viewers §5 and §7 quote
```

Source hashes (SHA-256, first 16 hex) at the run:

```
b2c7f1fc6951ddcb Cargo.toml
837f11bee0d546b3 Cargo.lock
29e44bf6bae64194 src/lib.rs
59448b1fbf22cf11 src/bin/baseline.rs
cd16d50dcd7b23bd src/bin/compare.rs
068a965122d01e32 src/bin/crops.rs
4b0b2e4501e049ad src/bin/families.rs
28531b4e0d752f65 src/bin/oracle.rs
478fc8f1c1d03dda src/bin/rowflat.rs
bae7b007360bcfd4 src/bin/rows.rs
2bfe9f3c23d2d3bc src/bin/strip.rs
22ac096edfaf0108 src/bin/zoom.rs
```

**Temp gets wiped.** If the folder is gone, §2's table is the whole oracle,
§3 and §6 define every test and rule exactly, and the three ruling pages hold
the answers (§2).

**No held-out entry was opened.** `tuning_marked()` in `src/lib.rs` drops
every manifest entry whose `split` is not `"tuning"` (or whose `expect` is not
a rectangle) before reading its `file` field, and asserts 29 remain. Every
binary reaches images only through it; the pixel viewers look entries up
among those 29 and panic on any other name.

## 10. Scope

Not done here, by the story's `## Out of scope`: the `Ambiguous` flag on
`2025-07-17 14_20_23.png`; any held-out run; per-site matching, learned
detection, colour; the column axis; site footers (none is ruled); the WebPs'
browser bar (MC-048's limitation, which §7 shows is also where Family B
clips); any code in `crates/`. The one fixture change the spike needed,
`(67)`'s mark, was its own chore (MC-060), at the user's request.
