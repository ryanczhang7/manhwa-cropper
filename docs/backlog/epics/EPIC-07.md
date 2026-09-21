---
id: EPIC-07
title: The crop reaches the artwork on the row axis
status: planned
stories: [MC-036, MC-037, MC-038, MC-039, MC-040, MC-041, MC-042]
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

## Deliberately not in this epic

- **Re-marking the corpus.** The marks stay tight, by the user's decision of
  2026-09-17. MC-031 section 9 Option C — re-marking rows to the page's
  vertical extent, under which a rule provably exists — was considered and not
  taken. Reopening it is a product decision and an MC-019 amendment.
- **Shipping chrome removal as its own criterion.** MC-031's Option B,
  declined above. It may still return as an *internal stage* of a rule that
  goes further; it may not return as the product's bar.
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
- **Any change to the flag-and-copy behaviour**, the output naming rules, the
  window, or the formats. This epic changes where the crop rectangle's top and
  bottom edges land, and nothing else.
- **Re-running v1's six investigations.** `docs/wiki/v2-candidates.md` indexes
  them and says what each answered.
