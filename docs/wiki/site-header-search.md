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
