# The calibration corpus

MC-018 built it; MC-033 wrote this page. **This is the corpus's one written
source of truth** — the rule the rectangles were drawn to, what the tags mean,
and where the recorded numbers are less precise than they look.

The corpus is 28 real screenshots in `fixtures/corpus/` with one
`fixtures/corpus/manifest.json` entry each: a hand-marked rectangle, or
`"flag"` for a screenshot that should be left alone. It is the oracle for every
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

The first nine are MC-018's AC-3 list and the union of every entry's tags must
still cover all nine — `the_tags_across_the_corpus_cover_every_required_case`
checks that, and it is a different question from this one.

## What the tests can and cannot say

`crates/engine/tests/corpus_manifest.rs` runs in the **required `unit`** gate
and checks the corpus is *well formed*: every file present and decodable, every
rectangle inside its image, the nine required tags covered, every tag in the
vocabulary above, the `diagonal-gutter` list exact, the whole set under 60 MB.

It cannot check that a rectangle is correct. When an accuracy run reports a
clip or a miss, the mark is as likely to be the thing that is wrong as the
detector — and only the person who drew it can rule on that.
