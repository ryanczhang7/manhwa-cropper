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

## 1. The verdict

*Pending: written last.*

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
| 9 | `Screenshot (67).png` | kunmanga | 290..1388 | 293 | none | MC-050 round 2 (`e08`) | 2026-09-24 | header end 293 is below the mark top 290 |
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

**`Screenshot (67).png` is the one conflict left:** its header is ruled to end
at 293 and its mark starts at 290. Under zero clips a rule may not remove rows
290..292, so that entry cannot be furniture-free by the exact test. The user
chose (open question 2, 2026-09-30) to leave both as they are: it counts as a
miss, and the pass line allows for it.

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
one miss).

## 4. The baseline and the ceiling (AC-2), and the scorer's controls (AC-5)

`main` at `25bf5d7`, `Tuning::default()`. Output of `baseline` (after
`crops`), verbatim; "free" is `yes`/`NO`, or `y/N` where the two counts
differ ("set aside" / "with").

```
== AC-2 (a): identity rule - main's crop, no new stage ==
file                         reader                    mark        crop    hdr   clip   free decision      
2025-08-05 00_11_13.webp     rolia-scans          114..1330     18..1440       -      0    yes crop          
2025-08-05 00_11_27.webp     rolia-scans          118..1343     18..1440       -      0    yes crop          
2025-10-14 23_29_06.png      demonicrevolution    188..1326    167..1400       -      0    yes crop          
2025-10-14 23_30_20.png      demonicrevolution    171..1379    167..1400       -      0    yes crop          
2025-10-20 15_37_25.png      toongod              228..1302    167..1400       -      0    yes crop          
2026-01-05 13_33_41.png      toongod              233..1293    167..1400       -      0    yes crop          
2026-01-05 13_45_59.png      demonicrevolution    204..1371    167..1400       -      0    yes crop          
2026-01-05 13_49_39.png      demonicrevolution    286..1396    167..1400       -      0    yes crop          
Screenshot (67).png          kunmanga             290..1388    167..1392     293      0     NO crop          
Screenshot (70).jpg          kunmanga             298..1310    167..1392     293      0     NO crop          
Screenshot (75).png          toongod              171..1379    167..1392       -      0    yes crop          
Screenshot (93).jpg          toongod              212..1379    167..1392       -      0    yes crop          
Screenshot (103).jpg         toongod              302..1146    167..1392       -      0    yes crop          
Screenshot (1661).png        toongod              192..1277    133..1392       -      0    yes crop          
Screenshot (2582).jpg        toongod              216..1224    133..1392       -      0    yes crop          
Screenshot (2630).jpg        toongod              224..1113    133..1392       -      0    yes crop          
Screenshot (2698).jpg        toongod              343..1085    133..1392       -      0    yes crop          
Screenshot (2708).jpg        toongod              220..1322    133..1392       -      0    yes crop          
Screenshot (2744).jpg        w-network            134..1298    133..1392       -      0    yes crop          
Screenshot (3187).png        w-network            142..1379    137..1392       -      0    yes crop          
Screenshot (3538).png        toongod              171..1330    137..1392       -      0    yes crop          
2025-03-06 01_22_45.png      toongod              118..1362    115..1374       -      0    yes crop          
2025-03-07 00_58_06.png      toongod              362..1290    115..1399       -      0    yes crop          
2025-07-17 14_41_58.png      xbato                273..1369    167..1400       -      0    yes crop          
2025-07-17 14_55_10.png      xbato                171..1395    167..1400       -      0    yes crop          
2025-08-03 11_27_49.png      rolia-scans          118..1385    115..1400     116      0    y/N crop          
Screenshot (68).png          kunmanga             392..1336    167..1392     319      0     NO crop          
Screenshot (73).png          kunmanga             293..1379    167..1392     293      0     NO crop          
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
  exact cut clips Screenshot (67).png: 293..1392 vs mark 290..1388 (3 rows)
ceiling: exact cut clips 1/28; zero-clip ceiling furniture-free 27/28 (set aside), 27/28 (with)
per reader: entries, ceiling free (set aside), ceiling free (with)
  demonicrevolution   4  4  4
  kunmanga            4  3  3
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
- **Ceiling (b):** the exact cut clips **1 of 28** (`Screenshot (67)`, 3
  rows); the zero-clip ceiling is **27 of 28** both ways. As predicted.
- **AC-5, both controls fire.** Moving the ruled rows 10 inward clips 11 of
  28. The identity rule reports furniture on exactly the 5 entries whose ruled
  site bar lies inside today's crop.
- `2025-07-17 14_20_23.png`: flagged `Ambiguous`, crop is the whole image,
  0 clips; counts toward nothing.
- **Blind offset, read off this table:** the largest fixed trim below the
  crop's top that keeps 0 clips is **1 row** (`Screenshot (2744).jpg`, crop
  133, mark 134). It reaches no `kunmanga` header (the nearest needs 126 rows).

## 5. Before any family: `Screenshot (67)` makes the pass line unreachable

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
