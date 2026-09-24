# Is the reader's own furniture locatable by a rule that works on any reader?

MC-050, a spike under `EPIC-07` story 4. Written 2026-09-24 against `main` at
`26eddcb`, the tree [MC-048](../backlog/stories/MC-048.md) left: its viewport
stage removes browser chrome and the taskbar as an internal stage of `detect`.
The question was whether a further, reader-agnostic rule is needed to remove
the **reader's own** header, navigation and footer, and whether one exists.

Every number here is over the **21 marked `tuning` entries** and nothing else.
No held-out entry was opened, decoded or listed per file (section 7).

## 1. The verdict: no new rule is needed on the tuning set, and the spike stops at AC-2

Once MC-048's stage has run, **reader furniture is inside the crop on 2 of 21
entries**. Both are `kunmanga`, both are a header, and neither has a footer.
The crop as shipped scores against `EPIC-07`'s bar:

```
identity rule: MC-048's crop, no furniture stage
zero clips on 21/21; no reader furniture on 19/21
bar (AC-6): zero clips on 21/21 and no furniture on >= 19/21 -> MET
```

AC-2 says: "If furniture is absent on nearly all 21 after MC-048, that is the
answer and the spike stops." It is absent on 19, so the spike stopped there. No
rule family was scored (AC-3), and there was nothing to hold out a reader
against (AC-4). Section 5 says what that does and does not establish.

**Two findings qualify the verdict.** Neither is hidden behind the headline
number:

1. **The slack in the bar goes entirely to one reader.** The 2 entries that
   still carry furniture are both of `kunmanga`'s 2 entries. That reader's header
   is painted in the page's own background tone, white on white, so MC-048's
   margin signal reads it as page and keeps it. Any held-out reader that builds
   its header the same way will fail absence the same way, and corpus.md implies
   `kunmanga` has held-out entries of its own.
2. **Removing that header would clip one mark.** On `Screenshot (67).png` the
   user ruled the header's end at row 293. The manifest marks the panel from
   row 290. So the perfect furniture oracle itself clips by 3 rows (section 3).
   A furniture rule could reach 21 of 21 absent only by breaking containment on
   that file, unless the mark or the ruling moves. Containment is absolute in
   `EPIC-07`, so the tuning ceiling for "zero clips *and* no furniture" is
   **20 of 21**, not 21.

## 2. The furniture oracle (AC-1)

There was no furniture boundary in the manifest; a person has to judge it, as
the marks were. The spike proposed rows from the pixels, and the user ruled on
every one with the images open, through the Artifact **Reader Furniture
Rulings** (https://claude.ai/artifact/JxL6XQMUtWKrdjru1umiVp). It showed each
entry's viewport at 50 %, full-resolution top and bottom bands, the proposal,
the app's current crop and the manifest mark.

**Round 1 was withdrawn.** The first version of the page drew the manifest
marks as dashed lines and never said what they were. The user took them for the
app's output and ruled all 21 tightly to the art edge, i.e. with the page
gutter removed, which is the target the 2026-09-21 amendment dropped. The page's
wording caused this, not the user. Round 1 is kept verbatim (collection
`rulings`, and `rulings-round1/` in the harness) and is **not** this oracle. One
answer from that exchange stands as a ruling: on `2025-08-05 00_11_13.webp` the
browser chrome ends at row 114 and the page begins at **115**, which matches
MC-028 §5a.

**Round 2 is the oracle.** Its page stated the rule: the reader site's own bars
only; page gutter stays; browser chrome and the taskbar belong to MC-048; and
the dashed lines are reference, not output. Every value below was ruled by the
**user (ryanczhang7) on 2026-09-24**, collection `rulings2`. "none" means
none visible. All 21 confirmed the proposal.

| # | File | Reader | MC-048 crop rows | Mark rows | Header ends | Footer starts |
|---|---|---|---|---|---|---|
| 0 | `2025-08-05 00_11_13.webp` | rolia-scans | 15..1440 (no viewport) | 114..1330 | none | none |
| 1 | `2025-08-05 00_11_27.webp` | rolia-scans | 15..1440 (no viewport) | 118..1343 | none | none |
| 2 | `2025-10-14 23_29_06.png` | demonicrevolution | 167..1400 | 188..1326 | none | none |
| 3 | `2025-10-14 23_30_20.png` | demonicrevolution | 167..1400 | 171..1379 | none | none |
| 4 | `2025-10-20 15_37_25.png` | toongod | 167..1400 | 228..1302 | none | none |
| 5 | `2026-01-05 13_33_41.png` | toongod | 167..1400 | 233..1293 | none | none |
| 6 | `2026-01-05 13_45_59.png` | demonicrevolution | 167..1400 | 204..1371 | none | none |
| 7 | `2026-01-05 13_49_39.png` | demonicrevolution | 167..1400 | 286..1396 | none | none |
| 8 | `Screenshot (67).png` | kunmanga | 167..1392 | 290..1388 | **293** | none |
| 9 | `Screenshot (70).jpg` | kunmanga | 167..1392 | 298..1310 | **293** | none |
| 10 | `Screenshot (75).png` | toongod | 167..1392 | 171..1379 | none | none |
| 11 | `Screenshot (93).jpg` | toongod | 167..1392 | 212..1379 | none | none |
| 12 | `Screenshot (103).jpg` | toongod | 167..1392 | 302..1146 | none | none |
| 13 | `Screenshot (1661).png` | toongod | 133..1392 | 192..1277 | none | none |
| 14 | `Screenshot (2582).jpg` | toongod | 133..1392 | 216..1224 | none | none |
| 15 | `Screenshot (2630).jpg` | toongod | 133..1392 | 224..1113 | none | none |
| 16 | `Screenshot (2698).jpg` | toongod | 133..1392 | 343..1085 | none | none |
| 17 | `Screenshot (2708).jpg` | toongod | 133..1392 | 220..1322 | none | none |
| 18 | `Screenshot (2744).jpg` | w-network | 133..1392 | 134..1298 | none | none |
| 19 | `Screenshot (3187).png` | w-network | 137..1392 | 142..1379 | none | none |
| 20 | `Screenshot (3538).png` | toongod | 137..1392 | 171..1330 | none | none |

Rows are image rows, and ranges are `top..bottom` with the bottom exclusive.
The MC-048 crop rows equal the located viewport on all 19 entries that have
one. `kunmanga`'s header is its genre menu, sign-in, chapter select and
Prev/Next, above a full-width divider at row 292. Row 293 is the first row
below the divider.

What the oracle does **not** cover:

- On the two WebPs, MC-048 declines to locate a viewport and their crops still
  contain browser chrome and the taskbar (MC-048 AC-3, a named limitation). That
  is not *reader* furniture, so round 2 rules "none". A person looking at those
  two outputs still sees chrome.
- Overlays drawn on the art, such as a scroll-to-top button or a watermark, are
  not furniture (Open question 2).

## 3. The ceiling (AC-2)

The crop cut exactly to the ruled rows, keeping MC-048's crop edge on any side
ruled "none". From `cargo run --release --bin score`:

```
Screenshot (67).png        kunmanga              290   1388    293   1392     3   present
Screenshot (70).jpg        kunmanga              298   1310    293   1392     0   present
(the other 19: top and bottom = MC-048's crop, clip 0, furniture -)
ceiling: 1/21 entries clip; furniture present in MC-048's crop on 2/21
```

Furniture is present on **2 of 21**, and cropping it away clips **1 of 21**.
AC-2 expected 0 clips and said that a clip, if it happened, would itself be the
finding. The oracle and the mark disagree on `Screenshot (67).png`: the mark's
top edge, row 290, lies 3 rows inside the header bar the user ruled. Either the
bar was drawn over the top of the page image (the "CROSS" lettering is cut at
the divider, which suggests the page runs on underneath it) or the mark is 3
rows loose. Deciding which is re-marking, and re-marking is out of scope here.

## 4. The negative controls (AC-5), each shown firing

```
== AC-5 control 1: ruled rows moved 10 rows inward (clip check is live) ==
clips on 7/21 (must be >= 1): FIRES

== AC-5 control 2: the viewport alone (absence check is live) ==
furniture reported on 2/21: ["Screenshot (67).png", "Screenshot (70).jpg"]
matches AC-2's furniture count (2): FIRES
```

The clip check sees a 10-row intrusion on the 7 entries whose mark lies within
10 rows of a crop edge. The absence check reports exactly the entries the
oracle has furniture on. Neither number is vacuous.

## 5. What was not run, and what that leaves open (AC-3, AC-4)

**AC-3, rule families: not run.** The story's timebox runs AC-1 and AC-2 first
and stops at AC-2 when furniture is absent on nearly all 21. Scoring a family
against 2 positive entries from 1 reader would be fitting that reader's header.
A reader-agnostic rule cannot be told apart from a memorised header height on
that evidence.

**AC-4, leave-one-reader-out: vacuous for the identity rule.** It has no
parameters, so no reader's entries can be withheld from choosing them. Per
reader it scores:

```
per reader (entries, zero-clip, furniture-free):
  demonicrevolution   4  4  4
  kunmanga            2  2  0
  rolia-scans         2  2  2
  toongod            11 11 11
  w-network           2  2  2
```

AC-4's memorising control cannot be demonstrated, because no rule was fitted.
It is recorded as not applicable, not as passed.

**So the evidence supports "MC-048 already meets the bar on these 21", and not
"a reader-agnostic furniture rule exists".** Four of the five tuning readers
have no furniture left inside the viewport in these captures. That says as much
about *where the user was in the chapter* when capturing (mid-chapter, with
page content reaching both viewport edges on 19 of 21) as about the readers. A
capture taken at the top or the bottom of a chapter, where the reader's header
or footer is on screen, is under-represented in the tuning set.

## 6. The recommendation (AC-6)

**Verdict: the bar is met on the tuning set with no rule beyond MC-048's**,
within the limits of sections 1 and 5. This spike justifies **no new feature
stage**. The decision it hands on is how the full `EPIC-07` predicate should be
earned on held-out, and it recommends this next story:

> **Score MC-048's crop on the held-out set against containment and absence.**
> *Before* any score is computed, the user rules reader furniture on the marked
> held-out entries as AC-1 did here: the same page and the same round-2 wording,
> but **without the app's crop drawn** (the ruling must not see the output it
> judges) and without the manifest marks (round 1's lesson). Then one run
> reports zero clips (hard) and entries free of reader furniture against the
> 9-in-10 bar. `required_gates: [integration]` for the tuning half. The
> held-out run stays outside the gate, as `corpus.md` rules.

Things that story should expect, given the findings above:

- **Same-tone headers are the likely miss.** `kunmanga`'s white-on-white
  header is the exact case MC-048's margin signal cannot see. If held-out
  misses cluster there, the follow-up is a rule for a same-tone full-width bar
  under the viewport's top edge. That needs positive entries from more than one
  reader to be fitted honestly, which is the evidence this spike lacked.
- **Chapter-boundary captures.** If held-out contains captures from the top or
  bottom of a chapter, footers will appear for the first time.
- **`Screenshot (67).png`'s 3-row disagreement** is a question about the mark,
  not about the crop. It is worth one look from the user before any furniture
  rule is fitted to `kunmanga`.

Outside this recommendation, and unchanged by it: the **side strip** of page
background ([MC-049](../backlog/stories/MC-049.md), `EPIC-08`), and
**chrome on the two WebPs** (MC-048's named limitation).

Opening learned detection, the signal class AC-6's negative arm would have
named, is **not** forced by this result.

## 7. Reproducing every number here (AC-7)

The harness is outside the repository, as MC-031's was:

```
C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\
  e3483d9e-2114-4364-8342-f15fe583769c\scratchpad\mc050
```

A cargo crate with path dependencies on `cropper-core` and `cropper-engine`,
and `[workspace]` in its `Cargo.toml` so that it does not join the repo
workspace. `src/lib.rs::tuning_marked()` drops every entry whose `split` is not
`tuning`, or whose `expect` is not a rect, **before a file is named or decoded**,
and asserts that 21 remain. No binary in it can reach a held-out entry.

| Command | What it produced |
|---|---|
| `cargo run --release --bin measure` | the MC-048 crop, page column and viewport per entry; row profiles in `out/profile/`; the previews the ruling page shows |
| `cargo run --release --bin assemble` | `proposals.json`: the measurement merged with the hand-read proposals in `proposed.json` |
| `cargo run --release --bin rows -- <i> <y0> <y1>` | one entry's row-by-row dump (used on `Screenshot (67).png` rows 282..316) |
| `cargo run --release --bin score` | sections 1, 3, 4 and 5: the ceiling, the identity rule against the bar, both controls, per reader; output in `out/score.txt` |

The rulings are read back from the Artifact's database, not typed in. Round 2
is `rulings-round2/rulings2/eNN.json` and round 1 is `rulings-round1/`. To
refresh them, list the `rulings2` collection of the Artifact above with its
database tool.

## 8. Scope

In: the 21 marked tuning entries, the reader's own full-row UI, and MC-048's
crop as committed. Out: held-out (not opened), per-site matching (MC-041,
parked), learned detection, colour, re-marking, the panel boundary and page
gutter, the column axis (MC-049), and any code in `crates/`.
