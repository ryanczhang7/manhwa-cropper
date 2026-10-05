---
id: EPIC-07
title: The crop reaches the artwork on the row axis
status: planned
stories: [MC-036, MC-037, MC-038, MC-039, MC-040, MC-041, MC-042, MC-043, MC-044, MC-045, MC-046, MC-047, MC-048, MC-050, MC-051, MC-052, MC-054, MC-056, MC-059, MC-060, MC-061, MC-062, MC-063, MC-064, MC-065, MC-066, MC-067, MC-068, MC-069, MC-070, MC-071, MC-072, MC-073, MC-074, MC-075, MC-076, MC-077, MC-078, MC-079, MC-080, MC-081, MC-082, MC-083, MC-084, MC-085]
---

## Goal

**Amended 2026-09-21 by the user's decision — the last clause is struck.** The
goal was "the artwork and nothing else", including "no wide band of page gutter
above or below the panel". That last part is given up: the crop no longer looks
for a panel boundary. Read the goal below with it removed, and
`product-brief.md` section 5's third amendment of 2026-09-21 for the user's own
words and the measurement consequence.

A cropped screenshot contains the artwork and nothing else: no browser tab bar,
no bookmarks bar, no site navigation menu, no Windows taskbar, and ~~no wide band
of page gutter above or below the panel~~. Today the horizontal crop achieves
this and the vertical crop does not — the app keeps 97 to 310 px beyond the
artwork on the row axis, and on a typical screenshot that band *is* the browser
and OS furniture the product exists to remove. The user still never sees a
clipped panel: zero clips is not traded away for any of this.

What the amendment leaves is the part that was always the point and was never
the hard part: **the furniture goes, the page gutter stays.** Gutters between
panels, art overhanging a gutter, sound effects and atypical bubbles crossing
onto a neighbour are all explicitly out of scope — they were the single cause of
all seven reasoned negatives, and they are now ignored rather than unsolved.

## Why now

**v1 shipped the vertical half of its headline promise unmet, and the numbers
hid it.** `architecture.md` decision 14 recorded the row axis as "loose but
safe" on the strength of an overshoot measured in pixels. Looking at the actual
output on 2026-09-17 showed what those pixels are: tab bars, bookmarks, site
navigation and the taskbar. The decision's amendment records the correction.

**The weaker criterion was offered and declined.** MC-031 section 9 Option B
proposed asserting "no output contains browser or OS chrome" instead of an
accuracy bar, backed by a chrome oracle that never clips and lies outside the
mark on 19 of 19 entries where it speaks. Shown the crop it produces for
`Screenshot (67).png`, the user's answer was **no — the site's own navigation
is also unacceptable**. So the target is the artwork itself, the corpus marks
stay **tight**, and this epic inherits the bar six investigations failed.

**There is a structural reason they failed, and it was not in any of their
conclusions.** MC-025, MC-028, MC-031, MC-032, MC-034 and MC-035 each reduced
the image to a **1D row statistic** — `row_spread`, `row_cover`, row flatness —
and so does the shipped detector: `crates/core/src/` has `trim`, `content`,
`flat`, `edges` and `margin`, and no connected-component, contour or region
analysis anywhere. A speech bubble is a *small 2D object*; projected onto a
row it contaminates the whole row, so a gutter containing one arc reads as
content. Two of the four documented failure causes — "bubble / overhang" and
"diagonal gutter" — are artefacts of that projection rather than properties of
the image. Each spike concluded "flatness does not work"; the sharper statement
is that **no row statistic can work when the thing that distinguishes gutter
from art is spatial extent**. That is the opening this epic is built on, and
it costs nothing to test: it needs no learning, no colour and no per-site
knowledge.

**And every number so far is optimistic by an unknown amount.** 21 marked
entries, with every rule tuned *and* scored on all 21 and no held-out set. Any
v2 claim made the same way would repeat the overfit, which is why the corpus
story comes first and blocks the rest.

## Done when

**Amended 2026-09-21 with the goal.** The clause "no more than a thin margin of
page gutter" is struck, and the measurement changes with it: the bar is no
longer an 11 px window against the marks but **containment plus absence of
furniture** — a crop contains the marked rectangle and contains no browser, OS
or reader furniture. The marks stay tight and become the never-clip oracle;
nobody re-marks. `product-brief.md` section 5's third amendment of 2026-09-21
states the predicate and why re-marking was not chosen.

The user can drop a folder of reader screenshots on the window and the outputs
are ones they would post or keep without opening an editor: the page content
area, with no browser, OS or reader furniture, and page gutter above and below
the artwork accepted. That is measured on a corpus **grown well past v1's 21
entries and split into a tuning set and a held-out set**, with the bar reported
on the held-out set — the number v1 never had. Zero clips remains absolute, on
both sets. Where the detector cannot crop safely it still flags and copies the
file unchanged rather than guessing — but an **unrecognised reader is not a
reason to flag** (the brief's second amendment of the same day).

**The held-out score, 2026-09-29 ([MC-051](../stories/MC-051.md),
[`held-out-score.md`](../../wiki/held-out-score.md)): not met.** At scored
commit `d2876f5`, **16 of 19** marked held-out entries meet the bar, against 18
required, with **0 clips on 19 of 19**. The three misses:
- `kunmanga`'s same-tone site header, left in;
- a one-row site header on one `rolia-scans` entry;
- an unseen-reader (`xbato`) page the detector flagged rather than cropped.

The held-out set is now spent. The next held-out score needs new screenshots.

**The second held-out score, 2026-09-30 ([MC-063](../stories/MC-063.md),
[`held-out-score.md`](../../wiki/held-out-score.md)): not met, with a
clip.** It ran on MC-062's 25 fresh screenshots at scored commit `1438b2b`,
whose detector is the same as at `d2876f5`:
- **3 clips of 25**, all on the column axis, so the absolute half fails;
- **21 of 25** meet the bar, against 23.

It sits beside MC-051's 16 of 19 and does not replace it. **It speaks for
crops on known readers only.** The fresh set holds no unseen reader and no
screenshot that should be left alone, so it says nothing about either.

**The goal stays open.** The user's words were that it *"should work on any
reader"*. The fresh set is spent too, and the next held-out score needs a
fresh draw with a new seed.

**The third held-out score, 2026-10-02 ([MC-071](../stories/MC-071.md),
[`held-out-score.md`](../../wiki/held-out-score.md)): not met, with a
clip.** It ran on MC-068's 15 fresh screenshots at scored commit `4539e12`,
whose detector has MC-065 to MC-070's five fixes:
- **2 clips of 15**, both on the column axis, so the absolute half fails;
- **11 of 15** meet the bar, against 14.

It sits beside MC-051's and MC-063's and replaces neither. **It speaks for
crops on four known readers, mid-chapter, dark pages.** The set holds no
unseen reader, no screenshot that should be left alone, no light page and no
`w-network` page, so it says nothing about any of those. The four failures
(`n02`, `n05`, `n06`, `n13`) go to `tuning` in a follow-up chore, and each
fix is its own story. The goal stays open. The 15 are spent, and the next
held-out score needs another fresh draw.

**The fourth held-out score, 2026-10-04 ([MC-078](../stories/MC-078.md),
[`held-out-score.md`](../../wiki/held-out-score.md)): not met, with a
clip.** It ran on MC-077's 10 fresh screenshots at scored commit `c108bd1`,
whose detector has MC-074 to MC-076's three fixes:
- **1 clip of 10**, one column on the left of `n05`, so the absolute half
  fails;
- **9 of 10** meet the bar, against 9: no browser, OS or reader furniture
  kept on any of the 10, and the viewport found on all 10.

It sits beside the earlier three and replaces none. **It speaks for crops on
four known readers, mid-chapter, dark pages.** The set holds no unseen
reader, no screenshot that should be left alone, no light page and no
`demonicrevolution` page, so it says nothing about any of those; with 10
entries each miss is 10 points. Had it passed, it would have been recorded
as "meets the bar on sites it knows, at this version" with the goal kept
open (the user, 2026-10-03). **The goal stays open.** The failure (`n05`)
goes to `tuning` in a follow-up chore, and its fix is its own story. The 10
are spent, and the next held-out score needs another fresh draw.

## Stories

To be written; the order below is the dependency order and the first is the
only one that can start.

1. **Grow the corpus, and split it.** More screenshots, hand-marked to the
   existing tight convention in `docs/wiki/corpus.md`, plus a tuning / held-out
   split recorded in the manifest. The user supplies and marks the files; an
   agent cannot. Everything else in this epic depends on it, and the split is
   the half that is easy to skip and expensive to add afterwards.

   **Written as two stories**, because the two halves block on different people
   and joining them holds the schema hostage to a data-collection session:

   - **[MC-036](../stories/MC-036.md)** — the manifest gains a validated `split`
     field, applied to the 28 entries that exist. Agent-completable today. The
     load-bearing part is that **all 28 are `"tuning"`**: they are contaminated,
     six investigations fitted thresholds against them, so none of them can be
     held out. The held-out set is legitimately empty when it is done.
   - **[MC-037](../stories/MC-037.md)** — the user adds and marks new
     screenshots, and the held-out set becomes real, with floors under both
     sets. `depends_on: [MC-036]`. Its counts and its split ratio are proposals
     awaiting the user's confirmation, and it carries two questions only the
     user can answer: stratified or site-disjoint, and whether every entry
     gains a `site:` tag.
2. **Spike: does 2D structure locate the panel edge where 1D projections
   cannot?** Connected components or panel-rectangle detection, scored against
   the same corpus and the same MC-019 predicate, with the bubble entries
   (`2708`, `2630`, `1661`, `13_33_41`, `23_30_20`, `13_45_59`, `70`) as the
   cases that decide it. Output is a document, as MC-028's, MC-031's, MC-034's
   and MC-035's were.

   **Written as [MC-038](../stories/MC-038.md)**, `depends_on: [MC-037]`. Two
   things settled when it was written, because both are easy to get wrong later:
   it measures over the **28 `tuning`** entries only — a spike is tuning work by
   definition, and the held-out set is not scored, not once — and it carries
   `required_gates: [integration]`, because the `Split::Tuning` filter every
   number in its document is denominated by is exercised by no other gate.
3. **Spike or feature, depending on 2: known site furniture.** A site's
   navigation bar is pixel-identical across every screenshot from that site,
   and matching known furniture is far more reliable than inferring it. v1's
   brief excludes per-reader special cases; adopting this is a product decision
   and needs an amendment there, not an agent's judgement.

   **Written as [MC-041](../stories/MC-041.md) and PARKED the same day,
   2026-09-21, by the user's decision — do not start it.** The user settled the
   premise the story rested on: *"It should work on any reader. Do not flag
   other readers."* The brief amendment was rewritten to match, and per-site
   knowledge is now permitted **as an optimisation on top of a rule that works
   without it, never as the mechanism**. A story measuring whether a furniture
   matcher reaches the bar is measuring the wrong thing: it can only answer for
   the sites in the corpus, and the requirement is explicitly the sites that
   are not. MC-041's `## Parked` section carries the reasoning and what would
   unpark it — a general row-axis rule that meets the bar on an unseen reader,
   and nothing else.

   **This spends the epic's second cheap idea**, and the consequence is in the
   "deliberately not in this epic" list below: learned or model-based detection
   was held "until stories 2 and 3 have reported", story 2 reported a reasoned
   negative and story 3 is answered by product decision rather than by
   measurement. Both conditions are discharged. Opening it is a separate
   decision and has not been taken.

   The four things settled when MC-041 was written, which still hold for
   whoever unparks it:

   - **The amendment has been made.** The user amended
     [`product-brief.md`](../../wiki/product-brief.md) §5 on **2026-09-21** to
     permit per-site furniture matching, and listed five things it does not
     change. That block is the authority for the story and also its ceiling.
     It corrects the sentence above in one respect, and the correction is worth
     reading: the brief was **silent** on per-reader special cases rather than
     against them, and the silence was being read as a prohibition. The
     sentence is left as written because the amendment quotes it.
   - **`spike`, not `feature`**, because [MC-038](../stories/MC-038.md) §9b
     recommends running it and the only bar a furniture rule can currently
     reach is **MC-031 Option B**, which the user declined on 2026-09-17. A
     feature story would have to assert a bar that does not exist yet, which is
     what MC-032's `## Closed` records going wrong. The split line for the
     feature that might follow — recognising furniture, then acting on it — is
     named in MC-041 rather than left to be invented later.
   - **It measures over the 28 `tuning` entries / 21 marked**, like every prior
     number in this epic, and carries `required_gates: [integration]` on
     MC-038's reasoning. It does **not** depend on MC-040.
   - **The tuning set carries no `site:` tag** — all 31 are on held-out
     entries — so the story discovers furniture groups from the pixels, which
     is what the product must do at run time anyway, and may not attribute the
     28 to readers.
4. **The row accuracy story**, once a signal exists that earns one. Its bar and
   its evidence are written when it is unparked, not now — MC-032's `## Closed`
   is the record of what happens when a story's criteria are written before a
   rule exists.

   **Rewritten 2026-09-21 by the amendment to this epic's goal.** It is no
   longer waiting on a signal, because it is no longer the *artwork* it has to
   reach. The story is now: **the crop removes browser, OS and reader
   furniture, and nothing below it.** Three things make it writable today where
   the old version was not:

   - **The bar exists and needs no new measurement to state.** A crop is right
     when it contains the marked rectangle (the marks stay tight and become the
     never-clip oracle) and contains no furniture. Containment plus absence,
     not an 11 px window.
   - **Two-thirds of the oracle is already built.** MC-031 section 9's chrome
     oracle locates the browser viewport and the taskbar, **never clips, and
     lies outside the mark on 19 of 19 entries where it speaks**
     (`chrome-row-search.md` §4). What it does not locate is the reader's own
     header, navigation and footer — which is the remaining work, and the
     reason its Option B was refused in the first place.
   - **It must generalise.** The user's decision that *"it should work on any
     reader"* stands: the furniture rule is a general one, and per-site
     matching may only improve a site it recognises on top of it — see
     [MC-041](../stories/MC-041.md), parked, and the brief's second amendment
     of 2026-09-21.

   Not yet written as a story. It is the next one to plan, and it supersedes
   the "row accuracy" framing above rather than sitting beside it.

   **Written 2026-09-23 as two stories, staged, by the user's decision.** The
   user's words that day: *"I consider this a failure currently. Not good
   enough for v1. [...] the app completely failed to crop out the browser
   artifacts up top."* Asked how to proceed on the top edge, the user chose
   **staged**:

   - **[MC-048](../stories/MC-048.md)** — *fix*: the crop removes the browser
     chrome and the taskbar. MC-031's chrome oracle, shipped as an **internal
     stage** of the crop — which the "deliberately not" list below permits —
     with containment held absolute and the columns pinned unchanged. It is
     progress toward this story's bar, **not** the bar: the reader's own
     header, navigation and footer survive it, and the story says so in its
     own `## Context` together with everything v1 still lacks after it.
   - **[MC-050](../stories/MC-050.md)** — *spike*, `depends_on: [MC-048]`: is
     the reader's own furniture locatable by a rule that works on any reader?
     A spike rather than a feature because no furniture oracle exists yet and
     a feature would have to assert a bar no rule has been shown to meet
     (MC-032's `## Closed`). It is not an eighth run of the panel-boundary
     search: the boundary it looks for touches the viewport edge, as chrome
     does. Per-site matching remains an optimisation only; MC-041 stays parked.

   If MC-050 finds a rule, the feature it sketches is the story that meets
   this item's bar and earns the epic's first held-out score.

   **MC-050 found none was needed on tuning** (0 clips and no reader furniture
   on 19 of 21 with MC-048's crop alone, `reader-furniture-search.md` §1), so
   the held-out score is filed directly, 2026-09-24, as
   **[MC-051](../stories/MC-051.md)**. It is a *spike*: the user rules held-out
   furniture blind, and then the crop is run once over the 24 marked and 7 flag
   held-out entries against 0 clips and at least 22 of 24. It is the story that
   selects `Split::HeldOut`, and its open questions decide whether it waits
   for [MC-049](../stories/MC-049.md) (recommended) so that the one score
   judges the crop v1 ships.

   **2026-09-24: MC-048 shipped with held-out row clips, because its held-out
   run was skipped.** MC-048's Open question 2 had planned one aggregate,
   counts-only clip check at the end of GATES. It was reversed in RED
   (MC-048 `## Amendments`, "Open question 2 — the held-out run is not
   made"), so nothing ran the frozen stage over held-out before merge. The
   first run after merge, on `26eddcb`, read 7 clips where `0f94c75` read 4.
   The three new ones are all on the row axis (`bottom:103 bottom:31 top:3`,
   plus a `bottom:2` that the user ruled a mark error). Two more marked
   entries were also newly flagged `LowContent`.
   - Two are split-screen screenshots. There the stage reads a second
     browser window's chrome as the reader's, and clamps the crop's rows into
     the art.
   - **[MC-052](../stories/MC-052.md)** — *fix*, the user's top priority of
     2026-09-24: the viewport stage reads only the reader's own window. It
     moves the two to `tuning`, corrects the third mark, and requires zero
     row clips on held-out in an aggregate run that **gates REVIEW**, so the
     skip is not repeated. The `LowContent` pair is its Open question 1.
   - It runs before `EPIC-08`'s MC-053, which depends on it.
   - **[MC-054](../stories/MC-054.md)** — *fix*, filed 2026-09-25. A random
     property test found a generated page on which MC-048's viewport stage
     cuts 20 rows of art. It is about 1 in 12,500 cases, and MC-052 did not
     cover it. The user ruled on 2026-09-25 that it is a separate fix, run
     after MC-053.
   - The lesson for any later story here: a stage that changes where crops
     land runs the held-out clip check before merge. Skipping it is an
     explicit, recorded decision with a cost, not a default.

   **MC-051's verdict, 2026-09-29: item 4 is not earned on held-out.** The
   one held-out run, at `d2876f5` after MC-049, scored **16 of 19 against
   18**, with **0 clips on 19 of 19**
   ([`held-out-score.md`](../../wiki/held-out-score.md)). The absolute half
   holds. The furniture half misses by two:
   - `kunmanga`'s same-tone header, the failure MC-050 predicted;
   - a one-row `rolia-scans` header;
   - the flagged `xbato` page.

   Even without the one-row call it would be 17. The next story is a
   same-tone-header furniture rule (MC-050 §6), fitted on tuning plus the
   three now-read entries. It must still work on any reader. Why the `xbato`
   page flags `Ambiguous` is a separate question. Held-out is spent: the next
   `EPIC-07` held-out score needs new screenshots, in a story shaped like
   MC-037.

   **MC-063's verdict, 2026-09-30: item 4 is still not earned, and the fresh
   set has clipped.** [MC-062](../stories/MC-062.md) drew and the user marked
   25 fresh screenshots. [MC-063](../stories/MC-063.md) ran the same
   detector (`1438b2b`, with crates identical to `d2876f5`) over them once.
   The result: **3 clips of 25, and 21 of 25 meeting the bar against 23**
   ([`held-out-score.md`](../../wiki/held-out-score.md)). The absolute half
   fails for the first time on held-out.
   - **All 3 clips are on the column axis.**
     - Two are right-edge cuts, of 2 and 6 columns.
     - One is a crop of a different region of the screen, the full height on
       the right, where the mark is on the left.
   - **No row clips on any of the 25.**
   - **The fourth failure** is a crop too wide, which keeps the browser's
     horizontal scrollbar. The user ruled the scrollbar browser furniture.

   **It speaks for crops on known readers only.** The set holds no unseen
   reader and no screenshot that should be left alone, so it says nothing
   about either. MC-051's 16 of 19 stands beside it. The goal stays open.

   The user's ruling on the clips: *"Write up and file"*. Two stories follow:
   - **[MC-064](../stories/MC-064.md)** — *chore*: move the four entries read
     per file to `tuning`.
   - **[MC-065](../stories/MC-065.md)** — *fix*, `depends_on: [MC-064]`:
     reproduce and remove the column-axis clips on those entries.

   **Split on 2026-09-30, after all four were viewed** at the user's request,
   and each mark ruled to stand. There are three causes, and the user chose
   two stories:
   - **[MC-065](../stories/MC-065.md)** — *fix*: the two right-edge cuts.
     - `f20`: a flat white page edge, trimmed as background.
     - `f09`: a near-black art edge, trimmed.

     In both, the site's background is a column of one exact value, so luma
     separates the page from the site, and colour stays out of scope.
     **Narrowed 2026-10-01** (MC-065 `## Amendments`): by luma, `f09`'s
     fringe is the same as `Screenshot (3538).png`'s, which the user wants
     left out. So MC-065 ships `f20` alone, and `f09` moves to
     **[MC-067](../stories/MC-067.md)**, *fix*, `depends_on: [MC-065]`,
     which carries the colour question.
   - **[MC-066](../stories/MC-066.md)** — *fix*, `depends_on: [MC-064]`: the
     two second-window screenshots.
     - `f18`: the app cropped a YouTube window beside the reader.
     - `f13`: it joined the page to that window and kept the scrollbar.

     By the user's rulings, the crop must stay inside the reader's window,
     and flagging `f18` is only an approved stopgap.

   **Where the fix belongs.** The column axis is `EPIC-08`'s bar. Zero clips
   is this epic's absolute, and these clips broke this epic's held-out score,
   so the fix is filed here. The fresh set is spent. The next held-out score
   needs a fresh draw with a new seed.

   All four were fixed (MC-065 to MC-067, done 2026-10-01; MC-067 by the
   user's ruling that `Screenshot (3538).png` grows by its fringe).

   **A second fresh held-out set: [MC-068](../stories/MC-068.md)** — *chore*,
   `depends_on: [MC-067]`. The user asked for one on 2026-10-01, picked by
   the Lead PO and not by them, at most 20. The Lead PO drew 20 blind with a
   new seed by MC-062's method; the user then cut it to 15 and raised
   `MAX_BYTES` to 120 MiB. The 21 spent `held-out` entries move to `tuning`.
   The user marks the 15 before it starts; the scored run on them is a later
   MC-063-shaped spike.

   **Real-use `Ambiguous` answers: [MC-069](../stories/MC-069.md)** — *fix*,
   `depends_on: [MC-068]`. 14 screenshots the app left uncropped when the
   user ran it (Screenshots 14 to 58, `toongod` and `demonicrevolution`, and
   `Screenshot (2705).png`, `toongod`) go into `tuning`, marked by the user, and must
   crop. Kept out of the held-out set because they were chosen by failing.

   **`Screenshot (2705).png` is cropped: [MC-070](../stories/MC-070.md)** —
   *fix*, `depends_on: [MC-069]`. Split from MC-069 on the user's answer of
   2026-10-01: its close call is on the page's own bottom rows, inside the
   crop, and its crop ends one column short of the art.

   **The third held-out score: [MC-071](../stories/MC-071.md)** — *spike*,
   `depends_on: [MC-070]`. The user asked for it on 2026-10-01, once MC-069
   and MC-070 were in. It runs the crop once over MC-068's 15 fresh
   screenshots, an MC-063-shaped run. Unlike MC-063's, the detector has
   changed since the last score (MC-065 to MC-070), so it is the first test
   of whether those fixes hold on screenshots no rule was fitted on. The bar
   is 0 clips and at least 14 of 15, confirmed by the user. It
   speaks for four known readers, mid-chapter, dark pages only. No entry is
   read per file before the totals are written, and any failure becomes its
   own story.

   **Result (2026-10-02): not met, with a clip.** 2 clips of 15 (`n02`
   cut on both sides, `n05` by one column), and 11 of 15 meet the bar against
   14. Speaks for four known readers, mid-chapter, dark pages only.

   **Filed after MC-071 (2026-10-02).** Five stories follow. The causes are
   candidates: no agent has measured the four failing screenshots, and the
   Lead PO measures each before its RED. Two of the fixes may merge once
   measured.
   - **[MC-072](../stories/MC-072.md)** — *chore*, `depends_on: [MC-071]`:
     move `n02`, `n05`, `n06` and `n13` to `tuning` (MC-064-shaped), with
     their wrong crops listed as known problems, exact in both directions.
     Held-out drops to 11 entries across 3 readers (`n13` was its only
     `demonicrevolution` entry), so two floors move on the user's ruling:
     `MIN_HELD_OUT_MARKED` 15 to 11 and `MIN_HELD_OUT_SITES` 4 to 3. Its RED
     records which of the four the viewport stage declines on.
   - **[MC-073](../stories/MC-073.md)** — *fix*, `depends_on: [MC-072]`:
     `n05`'s one-column cut on the left. Whether column 1006 is art is
     measured first; if it reads like `Screenshot (2705)`'s column 1472, the
     user decides between a re-mark and a fix, as in MC-070.
   - **[MC-074](../stories/MC-074.md)** — *fix*, `depends_on: [MC-072]`:
     `n02`'s page column, cut 60 columns on the left and 75 on the right.
     If its full-height rows turn out to be the same fault, it takes them
     from MC-075.
   - **[MC-075](../stories/MC-075.md)** — *fix*, `depends_on: [MC-072,
     MC-074]`: the full-height crops that keep the browser bar, scrollbar
     and taskbar (`n06`, and `n02`'s rows). It adds a corpus check against
     MC-071's frozen top and bottom rows, because no tuning suite judges a
     crop whose viewport the stage declined.
   - **[MC-076](../stories/MC-076.md)** — *fix*, `depends_on: [MC-072]`:
     `n13`'s crop starts at row 40, keeping 93 rows of browser chrome. Its
     symptom differs from MC-075's (the bottom is exact), so it is separate
     unless the measurement shows one cause.

   None of these is a held-out score. Any later run over the 15 is a
   re-score on spent held-out; the next held-out score needs a fresh draw.

   **A third fresh held-out set: [MC-077](../stories/MC-077.md)** —
   *chore*, `depends_on: [MC-076]`. The user asked on 2026-10-03 for 10,
   "and pick some from manhwa_panels". The Lead PO drew 10 blind with
   MC-068's scripts and a new seed (mahwa panels 3, Eleceed 3, Hero Killer
   2, Unholy Blood 2); the user marked them; MC-071's 11 spent entries
   moved to `tuning`; the ceiling rose to 140 MiB and the floors to 10
   marked and 4 readers on the user's answers.

   **The fourth held-out score: [MC-078](../stories/MC-078.md)** — *spike*,
   `depends_on: [MC-077]`. One run over MC-077's 10, MC-071-shaped, after
   reproducing MC-071 exactly on its spent 15. Bar 0 clips and at least 9
   of 10, confirmed by the user; two bottoms ruled blind.

   **Result (2026-10-04): not met, with a clip.** 1 clip of 10 (`n05`,
   `2025-10-26 12_13_16.png`, one column on the left), and 9 of 10 meet the
   bar against 9, with no furniture kept on any of the 10. Speaks for four
   known readers, mid-chapter, dark pages only. Named, not filed: a chore
   moving `n05` to `tuning`, and a fix story for its one-column cut that
   measures the column first (MC-073's precedent).

   **Filed after MC-078 (2026-10-04).**
   - **[MC-079](../stories/MC-079.md)** — *chore*, `depends_on: [MC-078]`:
     move `n05` (`2025-10-26 12_13_16.png`) to `tuning` (MC-072-shaped), a
     known clip exact in both directions. Held-out drops to 9 across 4
     readers; `MIN_HELD_OUT_MARKED` 10 to 9, `MIN_HELD_OUT_SITES` stays 4,
     on the user's answer "9 boxes, 4 sites".
   - **[MC-080](../stories/MC-080.md)** — *fix*, `depends_on: [MC-079]`:
     `n05`'s one-column cut on the left. Column 973 is measured first; if it
     reads like MC-073's seam, the user decides between a re-mark and a fix.

   Neither is a held-out score. Any later run over MC-077's 10 is a
   re-score on spent held-out; the next held-out score needs a fresh draw.

   **Filed from the user's Eleceed run (2026-10-04).** The user ran the app
   on their Eleceed folder, where many screenshots show the reader on the
   left and YouTube on the right, and sent 11 crops: 10 took the YouTube
   window, one kept the bookmarks bar and taskbar. Rulings: **"Fix story +
   blind draw"** and **"Raise cap to 160 MiB"**; seven distinct screenshots
   (one of five byte-identical repeats) go to `tuning`.
   - **[MC-081](../stories/MC-081.md)** — *chore*, `depends_on: [MC-080]`:
     the seven join `tuning` with the user's marks (pre-marked by the Lead PO
     from the left window's pixels, adjusted by the user), the cap rises to
     160 MiB, and today's failures are pinned as known exceptions, exact in
     both directions.
   - **[MC-082](../stories/MC-082.md)** — *fix*, `depends_on: [MC-081]`:
     the six that took the right-hand window. MC-066's shape, back; the
     cause is measured before RED and the criteria re-cut.
   - **[MC-083](../stories/MC-083.md)** — *fix*, `depends_on: [MC-081]`:
     `2025-03-07 01_02_31.png`, cropped full height. Separate by symptom;
     merges with MC-082 if the measurement shows one cause.
   - **[MC-084](../stories/MC-084.md)** — *chore*, `depends_on: [MC-082,
     MC-083]`: a fourth fresh held-out set, 10 drawn blind from the Eleceed
     folder during planning (seed and script in the story). How the boxes
     are pre-filled for marking is the user's open question. The score
     follows as a spike, after the fixes.

## Deliberately not in this epic

- **Re-marking the corpus.** The marks stay tight, by the user's decision of
  2026-09-17. MC-031 section 9 Option C — re-marking rows to the page's
  vertical extent, under which a rule provably exists — was considered and not
  taken. Reopening it is a product decision and an MC-019 amendment.
- **Shipping chrome removal as its own criterion.** MC-031's Option B,
  declined above. It may still return as an *internal stage* of a rule that
  goes further; it may not return as the product's bar.
  [MC-048](../stories/MC-048.md) (2026-09-23) is that internal stage; its
  criteria are framed as progress toward story 4 and say what is still missing.
- **Learned or model-based detection**, until stories 2 and 3 have reported.
  It is no longer excluded — EPIC-05 excluded it and `architecture.md`
  decision 14 defers it to v2 — but it is the most expensive option, it needs
  the grown corpus most of all, and two cheaper ideas are untested.

  **Both conditions are now discharged, 2026-09-21.** Story 2 reported a
  reasoned negative ([MC-038](../../wiki/region-row-search.md)); story 3 is
  answered by the user's decision that the detector must work on any reader,
  which makes per-site matching an optimisation rather than a mechanism and
  parks [MC-041](../stories/MC-041.md). Neither cheap idea is untested any
  more, and **no general row-axis rule exists** — so this is the only untried
  signal class left. It is still deferred: discharging the conditions makes it
  *available* to open, not opened. Opening it is a product decision with its
  own cost, and the brief's **offline** constraint is a real constraint on it —
  any model ships inside the exe and runs locally, with no network at all.
- **Colour and chroma**, ruled out in MC-025 `## Context` and not revisited
  here.
- **The column axis**, settled at 20 of 21 and not reopened.

  **Reopened 2026-09-23 by the user — in [`EPIC-08`](EPIC-08.md), not here.**
  *"The cropping from the side isnt tight enough and I still see the page on
  the right and left side."* It stays out of this epic because this epic's
  scope sentence below is about the top and bottom edges only, and the column
  axis has a different bar (the marks remain its accuracy target). EPIC-08
  records why.
- **Any change to the flag-and-copy behaviour**, the output naming rules, the
  window, or the formats. This epic changes where the crop rectangle's top and
  bottom edges land, and nothing else.
- **Re-running v1's six investigations.** `docs/wiki/v2-candidates.md` indexes
  them and says what each answered.
