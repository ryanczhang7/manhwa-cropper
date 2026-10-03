# The calibration corpus

MC-018 built it; MC-033 wrote this page. **This is the corpus's one written
source of truth** — the rule the rectangles were drawn to, what the tags mean,
and where the recorded numbers are less precise than they look.

The corpus is 113 real screenshots in `fixtures/corpus/` with one
`fixtures/corpus/manifest.json` entry each: a hand-marked rectangle, or
`"flag"` for a screenshot that should be left alone. 102 are `tuning` and 11 are
`held-out` (MC-069 added 14 to `tuning`; see "Screenshots the app called
`Ambiguous`: MC-069" below; MC-072 moved four more). Since MC-072 (2026-10-02), `held-out` means only the 11
of the 15 fresh screenshots drawn blind on 2026-10-01 that were never read per file. They are spent. Every earlier held-out entry is spent and
`tuning`. The split was 28 : 31 at MC-037, and each move since is recorded
under "Moves out of held-out" below. See "The
tuning / held-out split" below, which is the section to
read before using any of them for anything. It is the oracle for every
accuracy claim this project makes — MC-019's column-axis bar, MC-026's decision
gates, MC-027's page column. Nothing in the repository can check that a
rectangle is the *right* rectangle. Only the person who drew it can, and this
page is where their rulings are written down so that the next agent reads them
instead of re-deriving them.

## The marking rule

> The expected rectangle is the tightest axis-aligned rectangle containing all
> of **one page's own artwork**. **A panel's own overhang extends its
> rectangle; a neighbouring panel's overhang, or a caption box sitting in the
> gutter, does not.**

Confirmed by the user on 2026-09-17, against the marks they drew.

**This supersedes MC-018's phrasing**, which reads "the tightest axis-aligned
rectangle containing all the artwork, including any non-flat overhang into the
gutter". MC-018 is DONE and is the historical record of how the corpus was
built; it is left alone, and it is not the rule. The old wording is too broad
in one specific way — it says "any non-flat overhang" without asking *whose* —
and read literally it makes two correct marks look like mistakes:

- **`Screenshot (2744).jpg`, bottom at row 1298.** A caption box ("IF DELPHINE
  HAD BEEN ON THE 40TH FLOOR…") straddles the mark. The mark sits on the art
  edge and the caption box is **outside** the rectangle. Ruling: the panel's
  art edge is the boundary. The mark stands.
- **`Screenshot (2630).jpg`, top at row 224.** The speech bubble from the panel
  *above* hangs down across the mark, and is **outside** the rectangle. Ruling:
  a neighbour's overhang belongs to the neighbour. The mark stands.

Both were put to the user on 2026-09-17 with the images open; the full survey
of all 21 marked entries is in **MC-032's `## Notes`**.

The practical consequence for a detector is that the boundary it has to find is
sometimes the **panel's own edge**, not an empty gutter. MC-028 and MC-031 both
searched for the gutter and tested it by flatness, which an overhanging bubble
defeats by construction while the panel edge stays plainly visible to a reader.

## The diagonal-gutter tolerance

Two entries have a gutter that runs **diagonally** rather than along a row, so
there is no single row that is the boundary.

> Where the gutter is diagonal, a mark on the **higher edge, the lower edge, or
> the middle** of the diagonal is equally acceptable. It is not a hard line.

The user's rule, stated for the first time on 2026-09-17.

`manifest.json` stores one exact rectangle per entry and has nowhere to put a
range, so **the recorded rectangle for a `diagonal-gutter` entry reads as
precise when it is not**. Anything scoring against those two entries should
read the number as one acceptable answer out of a band, and a story that wants
to act on that is free to; MC-019's criteria are frozen and DONE, and
`Screenshot (93).jpg` stays the accepted column miss it is.

This is also the explanation MC-019's RED could not give. `Screenshot (93).jpg`
is MC-019's **sole column miss**, at +20 px on the right against an 11 px
window, and its RED recorded only that the miss is "structural, not a threshold
away". The diagonal is why: when the gutter runs diagonally the page's left and
right edges are **row-dependent**, so no single pair of columns is correct for
the whole page.

### These two were found by a person, and the next one will be too

`Screenshot (93).jpg` (top edge) and `2025-10-20 15_37_25.png` (bottom edge)
were confirmed by eye with the user on 2026-09-17. **Diagonal entries cannot
currently be found automatically.** MC-032 tried — per column, the row of
strongest vertical gradient near the mark, then the slope and fit of those rows
against x — and it finds `2025-10-20 15_37_25`'s *top* edge clearly (−50.3 rows
per 100 columns, r² 0.47) and nothing else. `Screenshot (93).jpg`, identified
instantly by eye, scores 14.9 at r² 0.12; the user's diagonal on `15_37_25` is
its *bottom* edge, which the same instrument calls flat. Art texture dominates
the gradient. Do not re-run it, and do not add a third tag on an agent's own
judgement: a third entry is a person's call.

## The gutter-crossing tolerance

MC-033 settled whose *overhang* counts. It was silent on a different question
MC-034 ran into: may a mark sit **below a gutter**, so that the rectangle spans
one? Three entries turn on it — in each, an outward scan stops at the top of a
gutter that is plainly there, and the recorded mark is some way below it.

> **Both are acceptable.** Where a gutter separates the page's content from what
> follows and the recorded mark sits below that gutter, a rect ending at the
> **top of the gutter** and one ending at the **recorded mark** are equally
> right.

The user's ruling of 2026-09-17, given on these three with the images open:

| Entry | top of the gutter | recorded mark | the band |
|---|---|---|---|
| `Screenshot (2708).jpg` | 1305 | 1322 | 17 px |
| `Screenshot (3538).png` | 1314 | 1330 | 16 px |
| `Screenshot (2698).jpg` | 1080 | 1085 | 5 px |

This is the same *kind* of thing as the diagonal-gutter tolerance above: the
manifest stores one exact rectangle and has nowhere to put a range, so the
recorded number reads as precise when a band was intended.

**It is a ruling on three entries, not yet a rule about the corpus.** The
general form — that any mark with a gutter above it carries a band back to that
gutter — is *plausible and unconfirmed*; only these three were put to the user.
Treat a fourth the way `diagonal-gutter` is treated: a person's call, not an
agent's. The three are deliberately **not tagged** in `manifest.json`, because a
tag needs a vocabulary entry and a test, and that is a story of its own rather
than something to slip in beside a ruling.

**What it changes for anything scoring the row axis.** MC-034's bottom-edge
numbers (`panel-edge-search.md`) are a **lower bound**: entries it scored as
misses or clips because the rule stopped at the gutter were, on these three,
producing an acceptable answer. Its document is left as the record of what was
measured; the correction lives here, and MC-032 carries what it implies.

## Corrected marks

A mark is corrected only when a person has looked at the image and ruled, and
every correction is recorded here with the old and the new rectangle.
`MARKED_TUNING_RECTS` in `crates/engine/tests/corpus_manifest.rs` pins the
current value of every marked `tuning` rectangle, so a correction is always a
visible edit in two places, never a quiet one.

**The rule every correction below applies** is the marking rule above: the
rectangle holds the page's own artwork and nothing else. A column that is
**flat page background** over the mark's rows is outside it. (MC-049's
definition of flat: at least 0.95 of the column's pixels over the mark's rows
within `uniform_tolerance`, 10, of the column's median luma.)

### `2026-01-05 13_45_59.png` — MC-027, 2026-09-17 (the precedent)

`{ x: 1040, y: 204, w: 471, h: 1167 }` → `w: 466`. Columns 1506–1510 were
exactly 255 on all 1167 marked rows. The user saw the image and the
measurement and corrected the manifest. MC-027's `## Amendments` has the
column table.

### Seven marks with flat page columns on one side — MC-049, 2026-09-23

With `Tuning::margin_px` going to 0 (MC-049), nothing is left to pad over a
mark that holds page background. Each of these marks had 1–3 columns of flat
page on one side, just outside where MC-027's column locator puts the page
edge. The user approved correcting them before RED, on the `13_45_59`
precedent. RED measured every column before editing (share within 10 of the
column's median over the mark's rows; the column just inside each new edge is
art, share 0.02–0.73).

| Entry | Side | Flat columns removed (share / median luma) | Old rect | New rect |
|---|---|---|---|---|
| `2025-10-14 23_30_20.png` | right | 1513, 1514, 1515 (1.00 / 255) | `1040,171 476x1208` | `1040,171 473x1208` |
| `Screenshot (2630).jpg` | right | 1591 (1.00 / 1), 1592 (1.00 / 11) | `958,224 635x889` | `958,224 633x889` |
| `Screenshot (67).png` | right | 1535 (1.00 / 255) | `1012,290 524x1098` | `1012,290 523x1098` |
| `Screenshot (70).jpg` | right | 1535 (1.00 / 251) | `1016,298 520x1012` | `1016,298 519x1012` |
| `Screenshot (3187).png` | right | 1560 (1.00 / 26) | `987,142 574x1237` | `987,142 573x1237` |
| `2026-01-05 13_33_41.png` | left | 1073 (1.00 / 11) | `1073,233 397x1060` | `1074,233 396x1060` |
| `Screenshot (2708).jpg` | left | 1073 (1.00 / 11) | `1073,220 393x1102` | `1074,220 392x1102` |

**`Screenshot (2630).jpg` got its own look.** Column 1591 is nearly black
(mean luma 3.1, min 2, max 141) against page background at 11–13. A
one-pixel dark line could be the panel's own border, and a panel border is
art. The user was shown an 8x zoom of columns 1560–1607 and the full-height
edge, and ruled 1591 **page background**. Both 1591 and 1592 leave the mark.
Had the ruling gone the other way, the mark would have stood and MC-049 would
have stopped: a zero margin would then clip real art.

Nothing else moved: same `y` and `h`, same tags, no image touched, no
`held-out` entry edited. After the correction, MC-027's column locator lands
**on** the mark's edge on these seven and on `13_45_59`'s right and
`Screenshot (3538).png`'s left. It lands outside the mark everywhere else.

### `2025-10-14 23_29_06.png`, widened to the art: the user, 2026-09-29

This correction goes the other way: the mark was **too tight**. MC-049's
Open question 3 asked whether the columns the crop keeps outside the mark are
page. On this entry the crop (margin 0) was columns 1003–1542 and the mark
1008–1535, so 5 columns on the left and 7 on the right sat outside the mark.
The user was shown the full screenshot with the crop and mark drawn, and
10x zooms of 40 columns around each side. They ruled: *"yes they're art,
widen the mark"*.

The measurement agrees. Luma over every second marked row, share within 10
of the column's median:

| Columns | What they are | Median | Share |
|---|---|---|---|
| 998–1002 | page, left | 255 | 1.00 |
| 1003–1007 | added to the mark | 77–78 | 0.30–0.32 |
| 1008–1011 | old mark edge, for comparison | 78–79 | 0.28–0.30 |
| 1532–1535 | old mark edge, for comparison | 156–157 | 0.30–0.34 |
| 1536–1542 | added to the mark | 156–157 | 0.27–0.31 |
| 1543–1546 | page, right | 255 | 1.00 |

`1008,188 528x1138` → `1003,188 540x1138`. The side edges are now exactly
the art's first and last columns, which is also where the crop lands. `y`
and `h` are unchanged.

### `2025-07-17 14_20_23.png`, left edge widened to the art: the user, 2026-09-29

Found by MC-056 when the entry moved to `tuning`. The edge check read the
mark's first column, 967, as page background (share 0.999), which is the
shape of MC-049's seven mark errors. The close-ups showed the opposite:
- The page is dark grey (luma 23) up to column 961.
- The picture's own black background starts at **962**. Columns 962–972
  are flat black (median 0), and the mark began 5 columns inside it.

The user was shown the context strip and 12x zooms at the mark's top, middle
and bottom, and ruled: *"widen to 962"*.

`967,171 609x1211` → `962,171 614x1211`. Only the left edge moves.

Because the art's own edge is flat black, the edge check still reads column
962 as background (share 1.0). The edge is therefore listed in that check as
one the user ruled art (`RULED_ART_EDGES` in `corpus_page_column.rs`), in
both directions.

### `Screenshot (67).png`, top edge moved down to the art: the user, 2026-09-30

Found by MC-059 (`site-header-search.md` §5), filed as MC-060. The mark's top
three rows, 290..292, are not art. Rows 290 and 291 are page white, and row
292 is the thin grey line that closes `kunmanga`'s site header. The picture
starts on row 293. Those rows are byte-identical over the full width to the
same rows of `Screenshot (73).png`, where the user ruled the site header to
end at 293. Measured over the mark's columns, 1012..1535 (share of pixels
within 10, `uniform_tolerance`, of the row's median):

| Row | Max departure from 255 | Median | Share within 10 of median |
|---|---|---|---|
| 288 | 0 | 255 | 1.00 |
| 289 | 0 | 255 | 1.00 |
| 290 | 0 | 255 | 1.00 |
| 291 | 0 | 255 | 1.00 |
| 292 | 20 | 235 | 1.00 |
| 293 | 192 | 224 | 0.59 |
| 294 | 246 | 240 | 0.19 |
| 295 | 247 | 238 | 0.18 |

The user was shown 8x zooms of rows 285..296 at the art's left edge (columns
960..1119) and its middle (1200..1359), and ruled: *"No art there, move the
box"*.

`1012,290 523x1098` → `1012,293 523x1095`. Only the top edge moves; the
bottom stays at 1388.

### `2025-08-05 00_11_13.webp`, top edge moved down one row: the user, 2026-10-02

Found by MC-048, whose original AC-3 could not be met because this mark
started at row 114, one row above the chrome end that two independent
readings place at 115 (MC-028 section 5a; `chrome-row-search.md` section
5e). MC-076's Lead PO zoomed rows 100..130 at columns 930..1130 (x8): row 114
is the dark one-pixel line closing the browser toolbar, one tone across the
page and the margin, and the art starts on row 115. The user was told this
in words, not shown the zoom, as part of a question about the WebPs' crops,
and answered *"Fix all three"*, which ruled row 114 browser bar (MC-076
`## Notes`).

`958,114 631x1216` → `958,115 631x1215`. Only the top edge moves; the
bottom stays at 1330.

## Tag vocabulary

Every tag on every manifest entry must appear in this table — it is parsed from
this page by `every_tag_in_the_manifest_is_in_the_documented_vocabulary` in
`crates/engine/tests/corpus_manifest.rs`, which runs in the required `unit`
gate. The table is the source and the test is the reader, deliberately: a
second copy of this list in Rust is exactly the drift MC-033 exists to remove.

A tag that is documented here and carried by nothing is fine — see
`overhang-text`. A tag in the manifest that is *not* here fails the test, which
is what catches a typo like `diagonal-gutters`: today such a tag belongs to no
category, nothing reads it, and the entry it was meant to classify is silently
untagged.

<!-- tag-vocabulary -->
| Tag | Meaning |
|---|---|
| `light-theme` | the reader page is rendered light |
| `dark-theme` | the reader page is rendered dark |
| `white-gutter` | the space between panels is light |
| `black-gutter` | the space between panels is dark |
| `all-art` | the screenshot is all artwork; `expect` must be `"flag"` |
| `mostly-white` | too blank to call; `expect` must be `"flag"` |
| `png` | the file is a PNG |
| `jpeg` | the file is a JPEG |
| `webp` | the file is a WebP |
| `diagonal-gutter` | a page edge runs diagonally, so the recorded rectangle is one answer out of a band (see above). Applied only by a person |
| `overhang-text` | a speech bubble or caption crosses a page edge. **Defined and deliberately unapplied**: the survey that would settle which entries deserve it has not been done, and a guessed tag is worse than an absent one |
| `site:toongod` | reader page captured from ToonGod |
| `site:rolia-scans` | reader page captured from Rolia Scans |
| `site:xbato` | reader page captured from xBato |
| `site:manhwaclan` | reader page captured from ManhwaClan |
| `site:kunmanga` | reader page captured from KunManga |
| `site:demonicrevolution` | reader page captured from Demonic Revolution |
| `site:w-network` | reader pages served on rotating `wNN.<series>.com` subdomains — one reader template across many per-series domains. See "Why this is one slug and not two" below |

The first nine are MC-018's AC-3 list and the union of every entry's tags must
still cover all nine — `the_tags_across_the_corpus_cover_every_required_case`
checks that, and it is a different question from this one.

### The `site:` tags

Added by MC-037 for the held-out set and completed by MC-042 for the tuning
set. **Every one of the 59 entries carries exactly one**, naming the reader it
was captured from: the 31 held-out entries since 2026-09-18, and the 28 tuning
entries since the user labelled them all on 2026-09-21. What that labelling
measured, and what it overturned, is under "`PRE_EPIC_07_SITES` — the readers
the original 28 came from" below.

They exist for one reason. A reader's furniture — navigation bar, header,
page margins — is pixel-identical across every screenshot from that reader, so
a rule tuned on one site can score well by learning *that site's* geometry
rather than learning what a page is. More screenshots from the same site will
not expose it; a screenshot from a reader the rule has never seen will. The tag
is what lets a test ask that question.

#### Why `w-network` is one slug and not two

Two held-out entries were first tagged `site:w16` and `site:w18`, read off the
subdomain. They are the same reader: the domain rotates its number, and the
same reader template is served across per-series domains — the user was on
`w18.pickmeupgacha.com` when this was settled on 2026-09-18.

Splitting one reader across two slugs is not a cosmetic error. It inflates the
distinct-site count, and it can make a reader look **unseen** while its twin
sits in the tuning set — an optimistic answer to the exact question the
held-out set exists to answer honestly. The slug names the *template*, because
the template is what the furniture belongs to. Five entries carry it.

A sixth entry from a `wNN` domain is a person's call, on the same footing as
`diagonal-gutter`: whether it is the same reader is a judgement about what the
page looks like, and nothing in this repository can make it.

## The tuning / held-out split

Every manifest entry carries a `split`, and the two permitted values are the
two rows of the table below — parsed from this page by
`every_split_in_the_manifest_is_in_the_documented_vocabulary` in
`crates/engine/tests/corpus_manifest.rs`, the same way the tag vocabulary above
is. The table is the source; the test is the reader.

<!-- split-vocabulary -->
| Split | Meaning |
|---|---|
| `tuning` | the entry may be looked at, measured against, and fitted to, as often as anyone likes. Every threshold in the detector was chosen against this set |
| `held-out` | the entry is scored **once**, at the end, and never tuned against. Nobody reads its per-file result before the rule that produced it is frozen |

### Why the split exists

Every rule v1 ever tried — MC-025, MC-028, MC-031, MC-032, MC-034, MC-035 —
was tuned *and* scored on the same marked entries. There was no held-out set,
so every v1 accuracy number is optimistic by an unknown amount. The size of
that optimism cannot be recovered after the fact; the only repair is to score
the next rule against screenshots that no tuning has seen.

### The rule, stated so it cannot be read two ways

A held-out entry is scored once and never tuned against. In practice that
means: no threshold is chosen, adjusted or rejected on the strength of a
held-out result; no per-file table of held-out results is read while a rule is
still being changed; and an entry that has been used to make any such decision
is no longer held out, whatever its `split` says. Moving an entry from
`held-out` to `tuning` is permitted and irreversible — contamination only runs
one way, and pretending otherwise is the failure this whole section exists to
prevent.

### All twenty-eight original entries are `tuning`, permanently

The corpus as it stood at commit `a453aaa` — the twenty-eight entries that
existed before EPIC-07 — is `tuning` in its entirety and cannot become held
out. Six investigations fitted thresholds against those entries, and their
per-file tables have been read by the people and agents planning v2. They are
contaminated. Assigning some of them to a held-out set would produce a number
that *looks* rigorous and is not.

That list is pinned by name in `PRE_EPIC_07_ENTRIES` in
`crates/engine/tests/corpus_manifest.rs`, read out of `a453aaa` rather than
re-derived, and
`every_pre_epic_07_entry_is_in_the_tuning_split` fails if any of them is ever
flipped.

The held-out set was therefore **empty** as of MC-036, and that was correct
rather than a defect: there was nothing legitimate to put in it yet. MC-037
filled it.

### How the split was assigned — MC-037, 2026-09-18

**Stratified, plus whole readers held out.** The user's ruling, made before any
screenshot was marked. Most readers in the held-out set also appear in the
tuning set, so the corpus answers *"does this rule generalise to an unseen
page?"*; a few appear **only** in held-out, so it also answers, at lower
resolution, *"does it generalise to an unseen reader?"* The ruling was recorded
as "all but three"; the labelling of 2026-09-21 measured it as **two** —
`xbato` and `manhwaclan` — for the reasons under `PRE_EPIC_07_SITES` below. The
gap between those two numbers is itself a finding, and getting both out of one
corpus is why the split was drawn this way.

The **31 new entries are held out in their entirety** and the tuning set was
deliberately **not** grown. The proposal was to give tuning 8 more marked
entries; the user declined, on the grounds that held-out is what new files are
for. So the marked ratio is 21 tuning : 24 held-out, and that is a decision
rather than an accident — do not "rebalance" it. (It is **26 : 19** since
MC-052's move of two reproducing entries and MC-053's move of three, under
"Moves out of held-out" below. Each was a ruling on named files, not a
rebalance.)

**The held-out set is scored once, at the end of a v2 attempt, and is never
tuned against.** Not once, not "just to see". The rule above under "The rule,
stated so it cannot be read two ways" governs, and it governs these entries
from the moment they landed.

#### Every accuracy suite runs over `tuning` only, and that is enforced in code

`crates/engine/tests/corpus.rs` and `crates/engine/tests/corpus_accuracy.rs`
both filter to `Split::Tuning` before measuring anything —
`corpus.rs::tuning_only()` and the filter inside `corpus_accuracy.rs::run_corpus()`.
Neither calls `corpus::load()` directly any more, and a new accuracy suite must
not either.

This does two jobs, and the second is easy to miss. It stops the `integration`
gate spending the held-out set on a measurement nobody asked for. It also keeps
**MC-019's, MC-026's and MC-032's recorded numbers meaning what they meant**:
those were measured against the twenty-eight pre-EPIC-07 entries, and without
the filter their denominator would silently have become fifty-nine, so every v1
claim would quietly describe a corpus it was never written about.

Whichever EPIC-07 story earns the first held-out score selects `Split::HeldOut`
deliberately, once, and says so in its own file.

##### The incident that produced this rule, recorded because it will recur

On 2026-09-18, the moment MC-037's screenshots landed, the `integration` gate
ran the v1 detector across all fifty-nine entries — the suites predated the
split and knew nothing about it — and four tests that had passed under MC-036
failed. `integration` is optional, so `All required gates passed` printed over
it.

**Ruling: the held-out set is still held out.** The rule above is that
contamination is *using* a held-out result to choose, adjust or reject a
threshold, or reading a per-file table while a rule is still being changed.
Neither happened: v1 is frozen and closed, no v2 rule exists yet to be
influenced, and the per-file tables in the gate log were deliberately not
opened. What was observed is one aggregate fact — that v1 does worse on the new
screenshots than on the old — which is not a per-file result and not a decision.
The user's ruling of 2026-09-18.

The lesson worth keeping is the shape of it: **the split was enforced in one
test file and assumed everywhere else.** MC-036 added `split` and validated it
in `corpus_manifest.rs`; nothing made the suites that actually *score* the
corpus respect it, because held-out was empty and the omission could not bite.
It bit the first day it could.

#### `PRE_EPIC_07_SITES` — the readers the original 28 came from

Parsed from the table below by `documented_pre_epic_07_readers()` in
`crates/engine/tests/corpus_manifest.rs`, the same way the tag and split
vocabularies above are. The table is the source; the test is the reader. A
second copy of this list in Rust is exactly the drift MC-033 exists to remove.

<!-- pre-epic-07-readers -->
| Reader | Note |
|---|---|
| `toongod` | 15 entries, 11 of them marked |
| `rolia-scans` | 2 entries, both marked |
| `w-network` | 5 entries, 2 of them marked |
| `kunmanga` | 2 entries, both marked — counted *unseen* until 2026-09-21 |
| `demonicrevolution` | 4 entries, all marked — counted *unseen* until 2026-09-21 |

**Measured, not recalled, since 2026-09-21.** The user labelled all 28 tuning
entries by reader with the Corpus Reader Labeller, and MC-042 wrote those labels
into the manifest, so this table is now a count of `site:` tags rather than a
list someone remembered. Each of the 28 carries exactly one tag, and
`the_reader_attribution_matches_the_counts_the_user_recorded` pins the per-file
mapping and the per-reader totals.

**The 2026-09-18 recall was falsified in both directions, which is exactly what
this page said would retire it.** That ruling named four readers — `toongod`,
`rolia-scans`, `w-network` and `xbato` — as a set rather than per file, and was
explicit that finding one of the supposedly unseen readers among the 28 would
overturn it. Two were found, and a fourth turned out never to have been there:

- **`kunmanga` (2 entries) and `demonicrevolution` (4) are in the tuning set**,
  though both were counted as unseen readers.
- **`xbato` has zero tuning entries**, though the recalled set named it. The
  user re-checked on 2026-09-21 and confirmed: *"xbato wasn't in the old 28."*

So the row for `xbato` is gone from the table above and the rows for `kunmanga`
and `demonicrevolution` are new. **No entry moved between splits**, and none
will: this page's own rule is that when the recall is falsified "the unseen
count drops and the entries stay where they are". Contamination runs one way,
and finding that `kunmanga` was in tuning all along does not make its held-out
entries fresh.

A held-out reader counts as **unseen** when its slug is outside the table above
**and no `tuning` entry carries it**. The second half is the user's ruling of
2026-09-24 (MC-053 Open question 2). It was added when MC-053 moved two of
`xbato`'s three entries into `tuning` and designed its fix on them. `xbato`
stayed outside the pre-EPIC-07 table, but a reader the rule has been tuned on
is not unseen. **One is unseen now: `manhwaclan`, on 2 entries.**
`MIN_UNSEEN_SITES` is **1** since that ruling, and the test reads both halves
of the definition.

**The unseen-reader claim is now one reader on two entries.** Treat any v2
unseen-reader result as an anecdote, not a measurement. New screenshots from
readers outside the corpus are the only repair.

**Since MC-062 (2026-09-30) there is no unseen reader at all.** `manhwaclan`'s
two entries moved to `tuning` with the rest of the spent set. All five readers
in the fresh held-out set also have `tuning` entries, and `MIN_UNSEEN_SITES`
is **0** by the user's ruling (see "The fresh held-out set" below).

The rest of this section is the record as it stood before MC-053.

**The unseen-reader number rested on five entries** — `xbato` 3 and
`manhwaclan` 2 — across **two** readers. Five entries either way, by
coincidence; a different five, and one reader fewer than before 2026-09-21.
`MIN_UNSEEN_SITES` in `crates/engine/tests/corpus_manifest.rs` is **2**, so the
count now sits **exactly at the floor with no margin**: attributing a sixth
reader into the tuning set, or retagging either of these two, drops it to 1 and
turns the required `unit` gate red. That is deliberate, and it is recorded here
so the next person to edit this table learns it from the page rather than from
the failure.

It is also still *thin*: one entry decides twenty percent of it, and a
two-entry reader measures almost nothing on its own. Treat a v2 unseen-reader
result as a direction, not a percentage, and say so wherever it is reported.
The unseen-**page** number, resting on all 26 held-out entries (31 before
MC-052 and MC-053), is the one with resolution.

### Moves out of held-out, and held-out mark corrections

Each is a ruling on named files, recorded here because the rule above makes
every move irreversible.

#### MC-052, 2026-09-24: two split-screen entries move to `tuning`

`2025-03-06 01_22_45.png` and `2025-03-07 00_58_06.png` moved from `held-out`
to `tuning`. They are the two files on which MC-048's viewport stage cut the
art, because each is a split-screen screenshot with a second browser window
beside the reader, and the stage read that window's chrome as the reader's.
MC-052 fixes that bug and is designed while looking at them, so they are
contaminated from that moment and cannot stay held out. The user's ruling of
2026-09-24. They were found by one aggregate held-out run after MC-048 merged;
the per-file look that followed was approved by the user and was confined to
the row-clipped entries.

- **New counts:** tuning 30 : held-out 29; marked 23 : 22.
- **Both are `toongod`**, a reader already in the tuning set, so the
  unseen-reader count (`xbato`, `manhwaclan`) does not change, and neither does
  the `PRE_EPIC_07_SITES` table above, which describes the original 28.
- Their marks are unchanged: `648,118 520x1244` and `671,362 474x928`.

#### MC-052, 2026-09-24: `2025-09-29 14_33_15.png`'s bottom edge

```
"expect": { "x": 984, "y": 171, "w": 576, "h": 1231 }   ->   "h": 1229
```

That moves the mark's last row from 1401 to 1399 and changes nothing else: same
`x`, `y`, `w`, tags and split. The entry **stays `held-out`**. The user ruled on
a labelled render of the rows around the edge: rows 1400–1401 are taskbar, and
the art ends at row 1399. The evidence is the whole-screen mean brightness per
row, about 57 over rows 1396–1400 and about 62.4 from row 1401 on: the taskbar's
tone begins at the old mark's last rows, not below them. The mark included two
rows of taskbar against the marking rule, so the detector cutting them was
right and the mark was wrong. The first aggregate held-out run after MC-048
reported the edge as a `bottom:2` clip, which is how it came up; no threshold
was chosen against it.

#### MC-053's rulings, landed with MC-055 (2026-09-28): the held-out hold is lifted

The four subsections below are the user's rulings made during MC-053. MC-053
became a spike on 2026-09-27, so its data edits did not merge; they landed
together with the fix, [MC-055](../backlog/stories/MC-055.md), on
2026-09-28. Until then no held-out score was taken, because the three
entries below were contaminated while still marked `held-out`. **That hold
is lifted now.** MC-055's GATES took the one aggregate held-out run its
AC-5 names (counts only, both margins): 19 entries, 18 cropped, 1 not
cropped (`Ambiguous`), 0 clips, which equals the baseline measured before
the fix. Nothing was chosen against it.

#### MC-053, 2026-09-24: three dark-art entries move to `tuning`

`2025-07-17 14_41_58.png`, `2025-07-17 14_55_10.png` (both `xbato`) and
`Screenshot (73).png` (`kunmanga`) moved from `held-out` to `tuning`. On each,
the page column locator trims the outer columns of dark, low-texture art as if
they were page background, and the crop cuts into the art. MC-055 fixes that,
and it and MC-053 were designed while looking at them, so they cannot stay
held out. The
user's ruling of 2026-09-24. The bug was found by MC-049's aggregate held-out
zero-clip check, and the user and orchestrator then looked at the clipped
edges.

- **New counts:** tuning 33 : held-out 26; marked 26 : 19.
- **The unseen-reader claim changes.** `xbato` now has two tuning entries, so
  under the ruling above it is no longer unseen. `manhwaclan` is the only
  unseen reader left.

#### MC-053, 2026-09-25: the three's marks widened to their art

The three are `tuning`, so looking at them costs nothing. Before any test
pinned them, the orchestrator rendered every side edge of the three at 8×
across, over the mark's rows. Each render ticked the mark's edge and the first
page-background column, by MC-049's predicate (at least 0.95 of the column's
pixels over the mark's rows within `uniform_tolerance` of its median). On all
six edges the art runs past the mark to the page background. **The user
ruled, on the renders: widen all six.** This is MC-027's precedent in the
other direction: the mark left out art, against the marking rule.

```
2025-07-17 14_41_58.png  931,273 681x1096  ->  928,273 690x1096   (left 928-930, right 1612-1617 added)
2025-07-17 14_55_10.png  931,171 668x1224  ->  928,171 690x1224   (left 928-930, right 1599-1617 added)
Screenshot (73).png      1007,293 530x1086 ->  1003,293 540x1086  (left 1003-1006, right 1537-1542 added)
```

Every added column scores 0.14–0.92 by the predicate, which is art. The column
beyond each new edge scores 1.00, which is page background. Rows are
unchanged.

#### MC-053, 2026-09-24: two white-page held-out marks corrected

```
Screenshot (56).png  "expect": { "x": 1040, "y": 171, "w": 480, "h": 1218 }  ->  "w": 472
Screenshot (59).png  "expect": { "x": 1043, "y": 224, "w": 464, "h": 1161 }  ->  "w": 463
```

Both move the right edge in and change nothing else. `(56)` drops columns
1512–1519, and `(59)` drops column 1506. Both entries **stay `held-out`**.
MC-049's held-out zero-clip check reported the two edges as right-side clips
on white pages. The user ruled on 2026-09-24 that both are mark errors: the
dropped columns are page, not art. No threshold was chosen against them.

#### `MIN_HELD_OUT_MARKED` lowered to 19 — MC-053, 2026-09-24

The two moves leave **19** marked held-out entries, under MC-037's floor of
20, and the test then named `the_held_out_set_carries_at_least_twenty_marked_entries`
(now `the_held_out_set_carries_at_least_nineteen_marked_entries`) would fail
the required `unit` gate. **The user's ruling (MC-053 Open question 6): the
floor drops to 19.** This is the same kind of ruling as `MIN_UNSEEN_SITES`.
At 19 entries one miss is 5.3 %, against 5 % at 20. The held-out set has
shrunk by five marked entries since MC-037, all moved for cause, and **new
screenshots are how it grows back**. Held-out is what new files are for.

#### MC-056, 2026-09-29: the three MC-051 read per file move to `tuning`

[MC-051](../backlog/stories/MC-051.md) scored held-out once and, under its
Open question 2, read only its three failing entries per file:
- `2025-07-17 14_20_23.png` (`xbato`): the detector flags it `Ambiguous`;
- `2025-08-03 11_27_49.png` (`rolia-scans`): a one-row site header;
- `Screenshot (68).png` (`kunmanga`): a same-tone site header.

The next furniture rule is to be fitted on them, so they cannot stay
`held-out`. [MC-056](../backlog/stories/MC-056.md) moved them. It also
widened `14_20_23`'s mark (see "Corrected marks").

**New counts:**
- **tuning 36 : held-out 23**;
- **marked 29 : 16**;
- flags 7 : 7.

**`xbato` and `kunmanga` now have no held-out entries**, so held-out spans 5
readers: `toongod`, `w-network`, `demonicrevolution`, `rolia-scans` and
`manhwaclan`. `manhwaclan` is still the one unseen reader, so
`MIN_UNSEEN_SITES` (1) and `MIN_HELD_OUT_SITES` (4) hold unchanged.

**`2025-07-17 14_20_23.png` is the one marked `tuning` entry the detector
does not crop.** By the user's ruling (MC-056 Open question 2), it is the
only name in a known-exceptions list in each of the eight corpus tests that
require every marked `tuning` entry to be cropped or unambiguous. Each list
fails in both directions: the story that makes it crop must empty them.

#### `MIN_HELD_OUT_MARKED` lowered to 16 — MC-056, 2026-09-29

The move leaves **16** marked held-out entries, under the floor of 19.
**The user's ruling (MC-056 Open question 1): the floor drops to 16**, as it
dropped to 19 before. The test is now
`the_held_out_set_carries_at_least_sixteen_marked_entries`. At 16 entries, one
miss is 6.3 %.

The set is already spent (below), so the floor now guards only its shape.
New screenshots are how it grows back.

### The held-out set is spent: MC-051, 2026-09-29

[MC-051](../backlog/stories/MC-051.md) selected `Split::HeldOut` deliberately,
once, at scored commit `d2876f5`, 2026-09-29 18:44 UTC, and says so in its own
file. It ran over **19 marked and 7 flag** entries against furniture rows the
user ruled blind. The result is in
[`held-out-score.md`](held-out-score.md):
- **0 clips on 19 of 19.**
- **16 of 19 meet `EPIC-07`'s bar**, against 18 required, so the bar is not
  met.
- Of the flag entries, 5 were `Flagged` and 2 `Cropped`.

**Read per file:** only the three failing entries, `2025-07-17 14_20_23.png`,
`2025-08-03 11_27_49.png` and `Screenshot (68).png`. They are `tuning` for
any later rule. The follow-up chore that moves them is
[MC-056](../backlog/stories/MC-056.md), done 2026-09-29 (above).

**Spent:** every one of the 26 has now given its one score. Any later run over
them is a **re-score on spent held-out**, labelled that way, and never
`EPIC-07`'s held-out score. A fresh held-out score needs new screenshots.

### The fresh held-out set: MC-062, 2026-09-30

[MC-062](../backlog/stories/MC-062.md) drew 25 new screenshots at random from
the user's own folders. The user marked them. The 23 spent entries moved to
`tuning`. `held-out` now means **only** these 25, and nothing has been run
on them.

**The fresh set contains no unseen reader and no flag entry.** A score on it
speaks for crops on known readers only. It says nothing about an unseen
reader, or about leaving a screenshot alone. Every report of that score must
say so.

#### The draw

The user asked for it: *"search in the folders I have specified before for
screenshots not already cropped, that havent been used before."* It was done
by names, sizes and file headers only. No drawn file was decoded or seen by an
agent before the user marked it. The scripts are outside the repository and
their hashes are in the story.

1. **Folders:** the eight the corpus came from, and nothing else.
   - `Downloads\mahwa panels`;
   - in `OneDrive\Pictures\Screenshots 1`: Demonic Evolution, Eleceed, Hero
     Killer, Jungle Juice, Pick Me Up Infinite Gacha, Unholy Blood and
     Suicidal Battle God.
2. **Pool:** every image in those folders that is not already in the corpus,
   matched by name without extension. That is 2,840 files.
3. **Whole-screen only.** "Not already cropped" means exactly 2560x1440 or
   1920x1080, read from the header. Four folders have none: Demonic Evolution,
   Jungle Juice, Pick Me Up Infinite Gacha and Suicidal Battle God hold only
   trimmed or stitched images. Whole-screen counts elsewhere: mahwa panels
   541, Eleceed 355, Hero Killer 472, Unholy Blood 111.
4. **Order:** within each folder, by
   `sha256("MC-062|2026-09-30|<folder>|<file name>")`, ascending.
5. **No near-twins.** A file is skipped when it was captured within 30
   minutes of any corpus screenshot or any file already drawn. For
   `Screenshot (N)` names, it is skipped when N is within 20 of any such
   number.
6. **Quotas:** mahwa panels 9, Eleceed 9, Hero Killer 4, Unholy Blood 3.
   That is an equal share, capped by what the twin rule leaves, with the rest
   split evenly.

**Two corrections, both made before any image was seen.** The seed never
changed.
- The first whole-screen test was a ratio, and it let in two wide stitched
  images. It became the exact sizes above.
- The first twin rule compared files within a folder only, and by day. 12 of
  its 25 were from the same day as a corpus screenshot. It became the
  30-minute rule, applied against the corpus too.

**The 25** (the manifest holds their marks and tags; each copy's SHA-256 was
checked against the story's table):

| Folder | Files |
|---|---|
| mahwa panels | `Screenshot (2461).png`, `Screenshot (2669).png`, `2025-10-23 11_31_40.png`, `2025-12-09 00_00_17.png`, `Screenshot (2486).png`, `Screenshot (2368).png`, `2025-11-12 17_43_44.png`, `Screenshot (9).png`, `2025-12-08 17_22_50.png` |
| Eleceed | `2025-03-04 14_28_54.png`, `2025-03-06 02_01_06.png`, `2025-03-24 22_44_31.png`, `2025-03-16 22_47_44.png`, `2025-03-07 16_07_21.png`, `2025-03-25 22_06_29.png`, `2025-03-18 12_37_27.png`, `2025-03-23 23_56_16.png`, `2025-03-06 12_48_06.png` |
| Hero Killer | `2025-08-07 11_20_12.png`, `2025-08-07 01_13_55.png`, `2025-08-07 14_33_43.png`, `2025-08-04 17_10_16.png` |
| Unholy Blood | `2025-07-21 17_47_22.png`, `2025-07-21 08_26_37.png`, `2025-07-17 23_45_48.png` |

**The marks.** The user marked all 25 on the *Fresh Test Marks* page
(collection `marks-mc062`) on 2026-09-30.
- All 25 are art boxes, and all are dark pages.
- Every box runs the visible height, from the end of the browser bar to the
  taskbar. They are mid-chapter screenshots, and none shows a site header.
- Gutter tags follow the user's answer: light gives `white-gutter`, dark gives
  `black-gutter`, and "can't tell" gives neither.
- **No `diagonal-gutter`.** The user ruled "No label" for the four they
  ticked: the diagonal gap lies inside each box and moves no edge. Their note:
  *"the diagonal only applies to the gap between panels"*.

#### The 23 spent entries move to `tuning`

The user's answer: *"move them to practice"*. All 23 `held-out` entries have
given their one score (MC-051), so all 23 are `tuning` now. That includes
`manhwaclan`'s two and all 7 of the old held-out flags.

**New counts:**
- **tuning 59 : held-out 25**;
- **marked 45 : 25**;
- **flags 14 : 0**.

**Readers:**
- **Held-out** spans 5 readers: `toongod` 14, `rolia-scans` 4, `xbato` 3,
  `demonicrevolution` 2 and `w-network` 2.
- **Marked tuning:** `toongod` 18 of 45 (40 %), `rolia-scans` 9,
  `demonicrevolution` 5, `w-network` 4, `kunmanga` 4, `xbato` 3 and
  `manhwaclan` 2.

**What the 23 broke in the tuning suites, and the user's rulings.** 16 of the
23 are marked. All 16 are cropped with 0 clips at both margins, none is
ambiguous, and the viewport stage locates all 16. 5 of the 7 flags are
flagged. Three things did not hold, and the user ruled on each:
1. **Two flag entries are cropped:** `2025-03-03 11_00_13.png` (568x1270) and
   `2025-05-12 20_48_42.png` (1892x4684). They are MC-051's "2 Cropped", and
   both are trimmed panel images, not screenshots. **Ruling: known misses.**
   They are named beside `2025-02-27 22_46_15.png` in `corpus.rs`'s
   `KNOWN_CROPPED_FLAGS`, and fixed later.
2. **The ambiguity band's measured ceiling drops.** `Screenshot (56).png` and
   `Screenshot (59).png` turn ambiguous at 0.005, though not at the default
   0.0025. `(56)` is now the binding entry, at 0.00265363. So 0.0025 sits
   1.06x below the ceiling, where it was 1.30x. **Ruling: record it.**
   `AMBIGUOUS_PAST_THE_CLIFF` pins all four names, and `ambiguity_band` is
   unchanged. The margin is thin now, and any rule that moves ambiguity will
   meet it first.
3. **The 90 % column-axis bar read 53 of 59 (89.8 %).** **Ruling: named
   known misses are left out of the bar.** `corpus_accuracy.rs`'s
   `KNOWN_MISSES` names `2025-02-27 22_46_15.png`, `2025-07-17 14_20_23.png`
   and the two above, and the bar reads **53 of 55 (96.4 %)**. The list is
   exact in both directions. `Screenshot (1720).png`, 3 px loose on the
   right with no clip, still counts as a miss.

#### The floors, re-set by the user's rulings of 2026-09-30

- **`MIN_HELD_OUT_MARKED` = 20** (it was 16). There are 25 present, and the
  floor goes up for the first time.
- **`MIN_HELD_OUT_SITES` = 4.** Unchanged, with 5 present.
- **`MIN_HELD_OUT_FLAGS` = 0** (it was 4). The ruling: "Drop that
  requirement". A blind draw of real use found no screenshot that should be
  left alone. `tuning`'s 14 flags carry that job. The test pins the count
  **equal** to 0, because a floor of 0 cannot fail. A flag entry that
  arrives in held-out is a change a later story must rule on.
- **`MIN_UNSEEN_SITES` = 0** (it was 1). The ruling: "Drop it; say so in the
  result". The eight folders hold only these readers. It is pinned equal to
  0 in the same way.

The one scored run on the fresh set is the next story, an MC-051-shaped
spike. It selects `Split::HeldOut` once, and says so in its own file.

#### The fresh set is spent: MC-063, 2026-09-30

[MC-063](../backlog/stories/MC-063.md) selected `Split::HeldOut`
deliberately, once, at scored commit `1438b2b`, 2026-09-30 22:36 UTC, and
says so in its own file. Its crates are identical to `d2876f5`, so the
detector is the one MC-051 scored. It ran over **25 marked and 0 flag**
entries. `T` and `B` came from the marks, plus the user's blind ruling of the
8 marks that end short. The user ruled that the browser's horizontal
scrollbar is browser furniture. The result is in
[`held-out-score.md`](held-out-score.md):
- **3 clips of 25**, all on the column axis. The absolute half fails.
- **21 of 25 meet `EPIC-07`'s bar**, against 23 required.
- Reader furniture is absent on 25 of 25, and browser and OS furniture on 23
  of 25. The viewport was located on 24 and declined on 1. None was
  `Flagged`.

**This score speaks for crops on known readers only.** The set holds no
unseen reader and no screenshot that should be left alone, so it says
nothing about either.

**Read per file:** only the four failing entries.
- `2025-12-08 17_22_50.png` (`f09`): clip.
- `2025-03-16 22_47_44.png` (`f13`): scrollbar kept.
- `2025-03-06 12_48_06.png` (`f18`): clip.
- `2025-08-07 01_13_55.png` (`f20`): clip.

These four are `tuning` for any later rule. The chore that moves them is
[MC-064](../backlog/stories/MC-064.md), and the fix for the clips is
[MC-065](../backlog/stories/MC-065.md).

**Spent:** all 25 have given their one score. Any later run over them is a
**re-score on spent held-out**, labelled that way, and never `EPIC-07`'s
held-out score. That includes the 21 that stay `held-out` in the manifest. A
later held-out score needs a fresh draw with a new seed, by the procedure
above.

##### MC-064, 2026-09-30: the four MC-063 read per file move to `tuning`

[MC-064](../backlog/stories/MC-064.md) changed four `"split"` values from
`held-out` to `tuning` and nothing else. No `expect`, tag or mark moved:
`2025-12-08 17_22_50.png` (`f09`, `toongod`), `2025-03-16 22_47_44.png`
(`f13`, `toongod`), `2025-03-06 12_48_06.png` (`f18`, `toongod`) and
`2025-08-07 01_13_55.png` (`f20`, `rolia-scans`). The user ruled that the
other 21 stay `held-out`, spent but unread.

**New counts:**
- **tuning 63 : held-out 21**;
- **marked 49 : 21**;
- flags 14 : 0.

Held-out still spans 5 readers (`toongod` 14 becomes 11, `rolia-scans` 4
becomes 3), so `MIN_HELD_OUT_MARKED` (20) and `MIN_HELD_OUT_SITES` (4) hold
unchanged.

**The exception lists.** The user ruled *"List them as known"*. Each list
pins the crop the app makes today, measured on `43e8e61` (crates identical to
`d2876f5`), and fails in both directions: if any other entry fails the same
way, or if a listed entry's crop moves at all. So the fix
([MC-065](../backlog/stories/MC-065.md)) has to empty them.

| List | Where | Entries |
|---|---|---|
| `KNOWN_CLIPS` | `corpus.rs` (both zero-clip tests), `corpus_accuracy.rs`, `corpus_sides.rs` | `f18` `1828,0 717x1440`, `f20` `1022,115 494x1285`, `f09` `1006,167 531x1233` (margin 0) |
| `KNOWN_CLIPS` | `corpus_viewport.rs` (both margins) | `f18`, `f20`. `f09` fails nothing there: at margin 3 its crop contains the mark, and at margin 0 that test checks rows only |
| `KNOWN_MISSES` (now 8) | `corpus_accuracy.rs`, the 90 % bar | all four, each held to its crop; the bar reads 53 of 55 |
| `KNOWN_BACKGROUND_SIDES` | `corpus_sides.rs`, `corpus_page_column.rs` | `f13` right: its crop keeps the browser scrollbar (`635,115 1922x1285`) |
| `MC064_BACKGROUND_EDGES` | `corpus_page_column.rs` | `f20` right: the mark's last column (1521) reads as page background (share 0.988). The user left the mark unchanged; MC-065 decides whether the mark or the app is wrong |
| `STAGE_DECLINED` | `corpus_viewport.rs`, `corpus_viewport_stage.rs` | `f18`: the viewport stage declines (`locate` returns `None`) |

`MAIN_CROPS` (`corpus_tuning_crops_unmoved.rs`) gains the four at both
margins, measured.

### The second fresh held-out set: MC-068, 2026-10-01

[MC-068](../backlog/stories/MC-068.md) drew 15 new screenshots at random,
after MC-065 to MC-067 fixed the four MC-063 read per file. The user asked
for the Lead PO to pick, not them, at most 20; the draw took 20 and the user
then cut it to 15. **Held-out now means only these 15.**

#### The draw

MC-062's procedure, with a new seed, read from names, sizes and file headers
only; no drawn image was viewed or measured before the user marked it. The
scripts are in MC-068 `## Notes` (`dims.sh` `4a3c01721bb46c65`, `draw.awk`
`f6bf09cb8472efe3`).
1. **Pool:** MC-062's eight folders, minus every corpus file by name without
   extension, and minus the 14 screenshots of
   [MC-069](../backlog/stories/MC-069.md) (the app's real-use `Ambiguous`
   answers, which go to `tuning` there). 1,454 whole-screen files
   (2560x1440), all in mahwa panels, Eleceed, Hero Killer and Unholy Blood;
   the other four folders still hold none.
2. **Order:** within each folder by
   `sha256("MC-068|2026-10-01|<folder>|<file name>")`, ascending.
3. **No near-twins:** MC-062's rule (30 minutes by name time; 20 by
   `Screenshot (N)` number), against the corpus, MC-069's 14 and each file
   already drawn. 41 skipped.
4. **Quotas:** round-robin over the four folders until 15: mahwa panels 4,
   Eleceed 4, Hero Killer 4, Unholy Blood 3.

**One correction before any image was seen:** the first run drew 20 against
the corpus alone. Re-run with MC-069's 14 counted as corpus and `TARGET=15`,
it is the first run's first 15 except that `Screenshot (32).png` (a near-twin
of MC-069's screenshots) is skipped and `Screenshot (2507).png` takes its
place. The 15 files, byte counts and SHA-256 prefixes are MC-068's
`## Context` table; each copy in `fixtures/corpus/` was checked against it.

**The marks**, frozen by the user on 2026-10-01 from *New Test Marks*
(`marks-mc068`): all 15 are art boxes, 0 flags, all dark pages. Readers:
`toongod` 7, `rolia-scans` 4, `xbato` 3, `demonicrevolution` 1. Every box
spans the visible height, browser bar to taskbar: mid-chapter screenshots.

#### The 21 spent entries move to `tuning`

The 21 that stayed `held-out` after MC-064, every one already scored once by
MC-063, became `tuning`. No `expect` or tag changed. Nothing they do in the
tuning suites is an exception: no clip at either margin, all 21 cropped, none
ambiguous at the default band, every mark edge read as art, no page-background
side, and the viewport located on all 21. Read-out pins (`MAIN_CROPS`,
`STAGE_MEASURED`, the page-column and viewport counts) record what the app
does on them. One measured list grew: `Screenshot (9).png` goes ambiguous at
the 0.005 control band (threshold 0.00474861), so `AMBIGUOUS_PAST_THE_CLIFF`
is 5 names; the binding entry (`Screenshot (56).png`, 0.00265363) and
`ambiguity_band` are unchanged.

**New counts:** 99 screenshots; **tuning 84 : held-out 15**; marked 70 : 15;
flags 14 : 0. The size ceiling is **120 MiB** (the user's answer of
2026-10-01); the corpus is 107.2 MiB.

#### The floors, by the user's rulings of 2026-10-01

`MIN_HELD_OUT_MARKED` = **15** (was 20; matches the set, so losing any of the
15 is caught). `MIN_HELD_OUT_SITES` = 4 (4 present), `MIN_HELD_OUT_FLAGS` = 0,
`MIN_UNSEEN_SITES` = 0, unchanged.

**What this set can speak for:** crops on four known readers, mid-chapter,
dark pages. It holds **no unseen reader, no screenshot that should be left
alone, no light page, and no `w-network` page**, so a score on it says
nothing about any of those. The scored run is a later MC-063-shaped spike;
until then the 15 must not be run as `held-out` by anything else.

### Screenshots the app called `Ambiguous`: MC-069, 2026-10-01

[MC-069](../backlog/stories/MC-069.md) adds the 14 screenshots the user's own
run of the app left uncropped as `Ambiguous` (`Screenshot (N).png`, N = 14,
19, 20, 23, 42, 48 to 53, 57, 58 and 2705), all to **`tuning`**: they were
chosen by the app failing on them, so they can never be held out. Copied
byte-for-byte under their original names (SHA-256 checked against MC-069's
`## Notes`), marked by the user on *Ambiguous Shots Marks* (`marks-mc069`)
and frozen the same day, with one amendment: `Screenshot (42).png` starts at
row 167, not 166 (row 166 is the browser bar). All 14 are art boxes:
`toongod` 6 (dark pages), `demonicrevolution` 8 (light pages).

On 13 of them the close call is the browser scrollbar, outside the crop;
`(2705)`'s is the page's own bottom rows, inside it, and it stays a known
`Ambiguous` exception until MC-070. `2025-07-17 14_20_23.png` (MC-056's
known exception) has the same scrollbar close call and crops after MC-069.

**New counts:** 113 screenshots; **tuning 98 : held-out 15**; marked 84 : 15;
flags 14 : 0. The corpus is 125,214,059 bytes (119.4 MiB), under the 120 MiB
ceiling, which MC-069 does not raise.

**MC-070 (2026-10-01).** `Screenshot (2705).png`'s mark is amended on the user's
answer ("Mark ends at 1471") from `1073,133 400x1259` to `1073,133 399x1259`:
column 1472 is a dim edge column of the same kind as edge columns left outside
the marks of six other entries (MC-070 `## Amendments`). Its close call on the
page's own bottom rows is fixed, so it crops and there is no known `Ambiguous`
exception left.

**Spent by MC-071 (2026-10-02).** The 15 gave their one score at scored commit
`4539e12`: **2 clips of 15, and 11 of 15 meet the bar against 14. Not met.**
Read per file, after the totals were written: `n02`
(`2025-03-07 00_05_58.png`), `n05` (`2025-11-01 12_34_31.png`), `n06`
(`2025-03-13 12_01_01.png`) and `n13` (`Screenshot (2507).png`). They are
`tuning` for any later rule, and moving them in the manifest is a follow-up
chore. The other 11 stay `held-out` but are spent. Any later run over the 15
is a re-score. The score speaks for crops on four known readers, mid-chapter,
dark pages. The set holds no unseen reader, no screenshot that should be left
alone, no light page and no `w-network` page, so it says nothing about any of
those. See `held-out-score.md`.

**Moved by MC-072 (2026-10-02).** The four read per file move to `tuning`:
`2025-03-07 00_05_58.png` (`n02`), `2025-11-01 12_34_31.png` (`n05`),
`2025-03-13 12_01_01.png` (`n06`) and `Screenshot (2507).png` (`n13`). Only
their `split` changes; marks, tags and bytes are unchanged and no file is
added. **New counts:** tuning 102 (88 marked + 14 flag), held-out 11 (11 marked
+ 0 flag) across 3 readers (`toongod` 4, `rolia-scans` 4, `xbato` 3).
- **Floors:** `MIN_HELD_OUT_MARKED` 15 -> **11** and `MIN_HELD_OUT_SITES` 4
  -> **3**, on the user's answer "all recommended" to the plain-language
  questions in MC-072: the set is spent, and the next fresh draw sets them
  again. `n13` was held-out's only `demonicrevolution` entry.
- **Known exceptions, exact in both directions,** until their fixes land:
  `n02` and `n05` clip (`KNOWN_CLIPS` in `corpus.rs`, `corpus_accuracy.rs`
  and `corpus_sides.rs`; `n02` alone in `corpus_viewport.rs`, since `n05`'s
  margin-3 crop contains its mark). Their marks' edge columns that read as
  page background (`n02` left and right, `n05` left) are in
  `MC072_BACKGROUND_EDGES`; that is not a ruling that they are art. (`n05`
  is off all of these since MC-073, below.)
- **The viewport stage declines on `n02`, `n06` and `n13`** (`STAGE_DECLINED`)
  and locates 167..1400 on `n05`. Those three are MC-071's three declines, so
  all 11 still held-out had their viewport located. A declined crop's rows are
  judged by no tuning suite; the three are held by `MAIN_CROPS`.
- Fixes: MC-073 (`n05`), MC-074 (`n02`'s columns), MC-075 (full-height crops),
  MC-076 (`n13`).

**Re-marked by MC-073 (2026-10-02): `n05` is off every list.** `n05`
(`2025-11-01 12_34_31.png`)'s mark moved from `1006,167 532x1233` to
**`1007,167 531x1233`** by the user's ruling *"Box starts at 1007"*. The
crop was never moved and no code changed: at margin 0 it already equals the
new mark. What prompted the question was a measurement of column 1006: over
the central band, median 10, range 2..26, share 0.999 within 10 of its own
median, beside a site of exactly 11 and art of about 71. That is a thin dark
line beside bright art. It reads like column 1006 on `Screenshot (3605).png`,
where the user's box starts at 1007, and on `2025-10-20 15_37_25.png`, where it
starts at 1008. `f09` (`2025-12-08 17_22_50.png`) keeps its 1006 only because
its 1007 is dark art too. A rule that kept `n05`'s column would move (3605)
and `10_20` outward by that column, which MC-070's candidate B did. `n05`
left `KNOWN_CLIPS` in `corpus.rs`, `corpus_accuracy.rs` and `corpus_sides.rs`,
`MC064_CROPS` in `corpus_accuracy.rs` and `corpus_sides.rs`, and
`MC072_BACKGROUND_EDGES` in `corpus_page_column.rs`; its new first column,
1007, reads as art (share 0.376). The known exceptions above are now `n02`'s
alone. MC-071's held-out score is spent and is not re-computed under the
amended mark.

## What the tests can and cannot say

`crates/engine/tests/corpus_manifest.rs` runs in the **required `unit`** gate
and checks the corpus is *well formed*: every file present and decodable, every
rectangle inside its image, the nine required tags covered, every tag in the
vocabulary above, the `diagonal-gutter` list exact, the whole set under 120 MiB
(60 MiB until MC-062, 100 MiB until MC-068, each raised at the user's request; the corpus
is 107.2 MiB after MC-068 and 119.4 MiB after MC-069).

It cannot check that a rectangle is correct. When an accuracy run reports a
clip or a miss, the mark is as likely to be the thing that is wrong as the
detector — and only the person who drew it can rule on that.

### Every accuracy number in EPIC-07 before MC-056 is majority one reader

**Since MC-056 (2026-09-29), `toongod` is 13 of the 29 marked tuning entries,
45 %, and no longer a majority.** MC-056 moved in one `xbato`, one `kunmanga`
and one `rolia-scans` entry. The other 16 are `demonicrevolution` 4,
`kunmanga` 4, `xbato` 3, `rolia-scans` 3 and `w-network` 2. The history
below is kept as it was written, and the numbers it lists were each measured
while `toongod` was a majority.

**`toongod` was 13 of the 26 marked tuning entries**, exactly half, after
MC-053 moved in two `xbato` and one `kunmanga` entry on 2026-09-24. It was
13 of 23 (57 %) after MC-052, and 11 of 21 (52 %) before it. The other 13 are
now `demonicrevolution` 4, `kunmanga` 3, `w-network` 2, `rolia-scans` 2 and
`xbato` 2. Every accuracy
suite runs over `tuning` only, and every score in EPIC-07 is measured over its
marked entries, so MC-019's 20 of 21, MC-026's 8 of 21, MC-028's 4, MC-031's 5,
MC-034's 8, MC-035's re-score and MC-038's 8 are each a majority-`toongod`
number. None of those documents says so, because until the labels landed on
2026-09-21 nobody could count it. On the 21, the remaining ten were
`demonicrevolution` 4, `w-network` 2, `rolia-scans` 2 and `kunmanga` 2.

This is counted from the manifest, not re-measured, and it **reweights nothing**.
Whether 52 % one reader qualifies any of those numbers is a question for
whoever next measures accuracy; MC-042 was a tagging story and did not reopen
them. What the count does establish is that "does this generalise across
readers?" cannot be answered on the tuning set at all — which is what the
held-out `site:` tags and the unseen-reader floor above exist for.

### The difficulty is not spread evenly across readers — a pointer, not a finding

MC-038's per-file ceiling (`region-row-search.md` §8) re-grouped by the new
labels, which is that table's own numbers regrouped and not a new measurement:

| Reader | Marked tuning entries where both edges are reachable |
|---|---|
| `rolia-scans` | 2 of 2 |
| `kunmanga` | 2 of 2 |
| `demonicrevolution` | 2 of 4 |
| `toongod` | 4 of 11 |
| `w-network` | 0 of 2 |

The rows sum to §8's own **10 of 21**, which is the arithmetic check that this
is a regrouping and nothing more.

**Read this as a pointer, not a finding.** The counts are tiny: two entries
decide two of those five rows outright, and one flipped entry moves
`demonicrevolution` from 2 of 4 to 1 of 4 or 3 of 4. It is not evidence that
`rolia-scans` is easy or that `w-network` is hard — it is a suggestion about
where to look first if a v2 rule stalls, and a reason to check a per-reader
breakdown before concluding that an aggregate ceiling is a property of the
method rather than of one site's furniture.
