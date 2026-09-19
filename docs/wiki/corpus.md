# The calibration corpus

MC-018 built it; MC-033 wrote this page. **This is the corpus's one written
source of truth** — the rule the rectangles were drawn to, what the tags mean,
and where the recorded numbers are less precise than they look.

The corpus is 59 real screenshots in `fixtures/corpus/` with one
`fixtures/corpus/manifest.json` entry each: a hand-marked rectangle, or
`"flag"` for a screenshot that should be left alone. 28 are `tuning` and 31 are
`held-out` — see "The tuning / held-out split" below, which is the section to
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

Added by MC-037. Each held-out entry carries **exactly one**, naming the reader
it was captured from. **Pre-EPIC-07 entries carry none and are not required
to** — see "Why the old twenty-eight have no `site:` tag" below.

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
screenshot was marked. Every reader in the held-out set except three also
appears in the tuning set, so the corpus answers *"does this rule generalise to
an unseen page?"*; three readers appear **only** in held-out, so it also
answers, at lower resolution, *"does it generalise to an unseen reader?"* The
gap between those two numbers is itself a finding, and getting both out of one
corpus is why the split was drawn this way.

The **31 new entries are held out in their entirety** and the tuning set was
deliberately **not** grown. The proposal was to give tuning 8 more marked
entries; the user declined, on the grounds that held-out is what new files are
for. So the marked ratio is 21 tuning : 24 held-out, and that is a decision
rather than an accident — do not "rebalance" it.

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
| `toongod` | |
| `rolia-scans` | |
| `w-network` | |
| `xbato` | |

The user's ruling of 2026-09-18, given as a **set** rather than per file. The
28 pre-EPIC-07 entries carry no `site:` tag and are not required to: attributing
a reader to a screenshot captured a year earlier is exactly the situation that
manufactures guesses, and this page already rules that a guessed tag is worse
than an absent one. Naming the handful of readers in use is a different and far
safer act of recall than attributing 28 files one by one.

A held-out reader counts as **unseen** when its slug is outside that set. Three
are: `manhwaclan`, `kunmanga` and `demonicrevolution`.

**Read this with the confidence it was given.** The user's words were that these
three are *likely* not in the old set. It is a recall claim, not a derivation,
and nothing in this repository can check it — the same footing as every
rectangle in the corpus. What would falsify it is someone going through the 28
and finding one of those three; if that ever happens, the unseen count drops and
the entries stay where they are. Contamination runs one way, and so does this.

**The unseen-reader number rests on five entries** — `manhwaclan` 2,
`kunmanga` 2, `demonicrevolution` 1 — across three readers. That satisfies the
floor and it is *thin*: one entry decides twenty percent of it, and a
single-entry reader measures almost nothing on its own. Treat a v2 unseen-reader
result as a direction, not a percentage, and say so wherever it is reported.
The unseen-**page** number, resting on all 31 held-out entries, is the one with
resolution.

## What the tests can and cannot say

`crates/engine/tests/corpus_manifest.rs` runs in the **required `unit`** gate
and checks the corpus is *well formed*: every file present and decodable, every
rectangle inside its image, the nine required tags covered, every tag in the
vocabulary above, the `diagonal-gutter` list exact, the whole set under 60 MB.

It cannot check that a rectangle is correct. When an accuracy run reports a
clip or a miss, the mark is as likely to be the thing that is wrong as the
detector — and only the person who drew it can rule on that.
