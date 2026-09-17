# Does the gutter-crossing tolerance retire any row-axis clips?

MC-035. A **re-score, not a search**: MC-034's harness, MC-034's two families,
MC-034's 780 parameterisations, with one thing changed — the scoring predicate
is widened on the three entries the user's ruling of 2026-09-17 gave a band. No
new rule, no new signal, no new threshold. Nothing in `crates/` changes and no
production or test code is written.

**The answer in one line.** The clip numbers moved — **387 of 8 659 clipped
scorings retired**, and the spread family's best-ranked member changed identity
and gained two entries at the same clip count — but **the clip-free best stayed
at 0 of 21 in both families**, and it stayed there because *no parameterisation
of either family is clip-free at all*, at the ruled band width or at twice it.
The row axis is now closed on measurement rather than on argument.

---

## 1. The band, frozen, and the predicate (AC-1)

### The three entries, read out

Sourced by name to `docs/wiki/corpus.md`, "The gutter-crossing tolerance", the
user's ruling of 2026-09-17. These numbers are **read out and not re-derived
from pixels**; the recorded mark is the manifest's own `y + h`, checked against
`fixtures/corpus/manifest.json` and not re-measured.

| Entry | top of the gutter | recorded mark | the band |
|---|---|---|---|
| `Screenshot (2708).jpg` | 1305 | 1322 | 17 px |
| `Screenshot (3538).png` | 1314 | 1330 | 16 px |
| `Screenshot (2698).jpg` | 1080 | 1085 | 5 px |

All three are **bottom** edges. No top mark carries a band, so nothing in this
document moves a top-edge number, and the widened scoring leaves `dtop`
untouched by construction.

**No fourth entry carries a band here.** `corpus.md` is explicit that the
general form — any mark with a gutter above it carries a band back to that
gutter — is *plausible and unconfirmed*, and that only these three were put to
the user. A fourth is a person's call, exactly as a third `diagonal-gutter` tag
is. The one fourth entry that appears below is a **throwaway probe** in control
(a), run in its own process and never in a result.

### The predicate is a disjunction of two exact MC-019 scorings

MC-019 scores a bottom edge `b` against an expected bottom `e` as `b >= e` (no
clip) **and** `b <= e + 11` (inside the `margin_px + 8` window; `margin_px` is 3
on this tree). The band gives two expected bottoms, so the widened predicate is

> a **hit** if it is a hit against **either** expected bottom;
> a **clip** only if it clips against **both**.

It is **not** `e_gutter <= b <= e_mark + 11`. Written out:

| Entry | acceptable bottoms under the disjunction | rows the naive widening would *also* admit |
|---|---|---|
| `Screenshot (2708).jpg` | `[1305,1316] ∪ [1322,1333]` | 1317–1321 |
| `Screenshot (3538).png` | `[1314,1325] ∪ [1330,1341]` | 1326–1329 |
| `Screenshot (2698).jpg` | `[1080,1096]` — contiguous; the 5 px band sits inside the 11 px window | — |

A rect ending at row 1318 on `2708` is 13 px below the gutter top and 4 px above
the mark: a hit against **neither** answer the user ruled acceptable. Admitting
it would be the oracle being edited rather than read.

### Control on the predicate, shown rather than asserted

`3538` makes this checkable on real data. Across the 780 parameterisations, **90
scorings produce a bottom edge in 1326–1329** — precisely the gap the naive
single window would swallow — and the disjunction scores **every one of them a
miss**:

```
$ awk -F'|' '$3 ~ /3538/ {split($8,q,","); b=q[2]+q[4];
             if (b>=1326 && b<=1329) print b, $4, $5}' inert-frozen.txt | sort | uniq -c
     18 1326  row=.   clip=.
     34 1327  row=.   clip=.
     12 1328  row=.   clip=.
     26 1329  row=.   clip=.
```

Under the naive widening all 90 would have counted as hits. On `2708` the check
is **vacuous and is reported as such**: no parameterisation happens to land in
1317–1321, so nothing is admitted there and nothing could have been. The
accepted bottoms observed on `2708` fall exactly inside the two windows above —
1305…1314 and 1327…1333, never between.

---

## 2. The un-widened harness reproduces MC-034 exactly (AC-2)

The widening is a **separate binary** reading a mode argument, and MC-034's
`sweep`, `sweep2` and `detail` are byte-for-byte untouched, so both runs come
out of the same tree. Two independent reproductions were required before any
"after" number was looked at.

**MC-034's own binaries, re-run:**

```
$ cargo run --bin sweep
family best rows           : 6 of 21          (art_th 6.0 gap 12 backoff 0, 4 clips)
family best CLIP-FREE rows : 0 of 21

$ cargo run --bin sweep2
best rows         : 8 of 21                   (dev 20 min_cover 0.10 gap 4 backoff 0, 9 clips)
best CLIP-FREE    : 0 of 21
```

**The new binary with the widening disabled:**

```
$ cargo run --bin bsweep -- off
entries this mode can move: 0 []
spread:   best-ranked = 6 of 21, 4 clips, 2 flag crops  [panel art_th 6.0 gap 12 backoff 0]
spread:   best CLIP-FREE = 0 of 21
coverage: best-ranked = 8 of 21, 9 clips, 3 flag crops  [cover dev 20 min_cover 0.10 gap 4 backoff 0]
coverage: best CLIP-FREE = 0 of 21
```

Against `panel-edge-search.md` sections 1, 3b and 3c: coverage best **8 of 21
with 9 clips** ✓, coverage best clip-free **0 of 21** ✓, spread best **6 of 21
with 4 clips** ✓. The full ranking tables match `sweep` and `sweep2` line for
line, including the numbers this story does not report. No discrepancy arose, so
nothing had to be resolved before the "after" numbers were believed.

### One clarification the reproduction forced, about MC-034's own number

`sweep` and `sweep2` compute "best CLIP-FREE" as `filter(clips == 0).max(rows)`
with `unwrap_or(0)`. Counting members directly:

```
off:  panel 240 members, clip-free members 0 · cover 540 members, clip-free members 0
```

So MC-034's **0 of 21 does not mean "a clip-free rule exists and scores zero
rows"**. It means **no clip-free member exists at all** — 0 of 780. That is a
stronger statement than the number reads as, and it is what section 5's verdict
rests on.

---

## 3. Before and after, side by side (AC-3)

MC-019's scoring exactly as MC-034 used it: the 11 px window, the clip count,
computed over the **final, margin-expanded, clamped** rect.

### 3a. The headline numbers

| Measure | before (exact mark) | after (frozen band) | Δ |
|---|---|---|---|
| spread best-ranked member | `panel art_th 6.0 gap 12 backoff 0` | `panel art_th 8.0 gap 8 backoff 2` | **changes identity** |
| — its rows / clips | 6 of 21, **4 clips** | 8 of 21, **4 clips** | rows +2, clips 0 |
| spread best clip-free | **0 of 21** (no clip-free member exists) | **0 of 21** (still none) | **0** |
| spread lowest clips anywhere | **2**, at 0 of 21 (`art_th 4.0 gap 8 backoff 8`) | **1**, same member | **−1** |
| coverage best-ranked member | `cover dev 20 min_cover 0.10 gap 4 backoff 0` | **unchanged** | — |
| — its rows / clips | 8 of 21, **9 clips** | 8 of 21, **9 clips** | **0** |
| coverage best clip-free | **0 of 21** (none) | **0 of 21** (none) | **0** |
| coverage lowest clips anywhere | **1**, at 1 of 21 (`dev 20 min_cover 0.02 gap 8 backoff 8`) | **1**, unchanged | **0** |
| headline rows, best of both families | 8 of 21 | 8 of 21 | **0** |

The headline rows number is reported because the same sweep produces it, and is
**not** the question; `## Context` bounds its movement at +2 by derivation, and
the measured movement of the best of both families is **0**. The spread family
alone moves +2, which is that bound being spent inside the weaker family.

Where the rank changed, both members are named in full:
**before** `panel art_th 6.0 gap 12 backoff 0`; **after** `panel art_th 8.0 gap
8 backoff 2`. The new leader scored **6 of 21 with 6 clips** under the exact
mark — the band is what makes it 8 of 21 with 4.

### 3b. Every scoring, not only the best-ranked ones

780 parameterisations × 21 marked entries = **16 380 scorings** per run.

| | clipped scorings | hits |
|---|---|---|
| before (exact mark) | **8 659** | 1 616 |
| after (frozen band) | **8 272** | 1 893 |
| Δ | **−387** | **+277** |

By family: spread **1 660 → 1 421** (−239), coverage **6 999 → 6 851** (−148).
Every one of the 387 is a `CLIP → no clip` transition; none goes the other way,
which is the monotonicity a disjunction has to have. Of the 387, 277 also become
hits and 110 become non-clipping misses — a rect that stops below the gutter top
no longer cuts into it, but is still too loose for the 11 px window.

### 3c. Per-entry, for each of the three band entries

`pBot` is the produced bottom row; `dBot` is against the recorded mark.

**On the spread family's new leader, `panel art_th 8.0 gap 8 backoff 2`** — the
member the band promotes:

| Entry | pBot | dTop | dBot | before | after |
|---|---|---|---|---|---|
| `Screenshot (2698).jpg` | 1087 | 7 | 2 | hit | hit — *already acceptable under the exact mark* |
| `Screenshot (2708).jpg` | 1310 | 7 | −12 | **CLIP** | **hit** — 1310 ∈ `[1305,1316]` |
| `Screenshot (3538).png` | 1319 | 0 | −11 | **CLIP** | **hit** — 1319 ∈ `[1314,1325]` |

**On the coverage family's leader, `cover dev 20 min_cover 0.10 gap 4 backoff 0`** —
the member that carries the 8-of-21 headline:

| Entry | pBot | dTop | dBot | before | after |
|---|---|---|---|---|---|
| `Screenshot (2698).jpg` | 1085 | 4 | 0 | hit | hit — *unchanged* |
| `Screenshot (2708).jpg` | 1327 | 5 | 5 | hit | hit — *unchanged* |
| `Screenshot (3538).png` | 1247 | −2 | −83 | **CLIP** | **CLIP** — *unchanged* |

**On the spread family's old leader, `panel art_th 6.0 gap 12 backoff 0`:** all
three unchanged — `2698` and `2708` were already hits, and `3538` stops at 1319,
inside the band's lower window, but `dTop = −1` clips the **top**, and the band
widens only the bottom. 6 of 21 with 4 clips, before and after.

### 3d. This corrects this story's own `## Context`, and it is why the headline cannot move

MC-035 `## Context` states that `panel-edge-search.md` section 3c "names all
three entries as failures of the best coverage member in the 'stops above the
mark' shape". Measured against the committed implementation, that is wrong in
two ways:

- on the best coverage member, **`2698` and `2708` are already hits with no
  clip** (`dBot` 0 and 5) — there is nothing there for a band to retire;
- **`3538` stops at 1247**, not the 1314 section 3c quotes — 67 px above the
  gutter top and far outside the band — *and* clips the top at `dTop = −2`.

Section 3c's "stops 1305 / 1314 / 1080" describes the family's characteristic
failure *shape*, across its members, not the best member's behaviour. So the
`## Context` prediction that "up to three clips can be retired from a given
parameterisation's count" is, for the headline member, **zero**, and the
coverage family's headline could not have moved. The clips the band does retire
are elsewhere in the grid — 387 of them.

---

## 4. The controls (AC-4)

### (a) Inertness on the 18 entries that carry no band

Every per-entry score — hit, miss, clip and the produced rect — dumped for all
780 parameterisations, before and after, and diffed:

```
$ cargo run --bin binert -- off    > inert-off.txt      # 16 380 lines
$ cargo run --bin binert -- frozen > inert-frozen.txt   # 16 380 lines
$ diff <(grep -v -e 2698 -e 2708 -e 3538 inert-off.txt) \
       <(grep -v -e 2698 -e 2708 -e 3538 inert-frozen.txt)      # no output
$ grep -vc -e 2698 -e 2708 -e 3538 inert-off.txt
14040                                                    # = 780 × 18
```

**The diff is empty** on all 14 040 unbanded scorings. Every one of the 774
differing lines (387 changed records) belongs to one of the three banded
entries.

**Shown firing.** This control passes by construction if the widening was never
wired to the entry list at all, so a band was deliberately invented on a fourth
entry, `2025-08-05 00_11_13.webp` (mark bottom 1330), with a throwaway
alternative bottom of 1320:

```
$ cargo run --bin binert -- "probe4:2025-08-05 00_11_13.webp:1320" > inert-probe4b.txt
# frozen -> probe4, changed records:
     17  2025-08-05 00_11_13.webp :: row=. | clip=CLIP -> row=ok | clip=.
     14  2025-08-05 00_11_13.webp :: row=. | clip=CLIP -> row=.  | clip=.
# lines differing on the other 20 entries: 0
```

31 scorings move on the invented entry and **nothing moves anywhere else**. The
probe is a separate process; no file carries it, and it appears in no result.

**The first probe chosen did not fire, and that is worth recording.**
`2026-01-05 13_49_39.png` with an alternative bottom of 1350 changed **nothing**:
that entry clips its **top** in every parameterisation where its bottom would
have mattered, and a bottom-only widening cannot retire a clip that a top edge
is also causing. The same mechanism is what keeps `3538` clipped on the coverage
leader in 3c. A probe that does not fire is a bad probe, not a passing control;
it was replaced rather than reported.

### (b) Band-width sensitivity

**The probe `## Context` specified is a no-op by geometry, not by measurement,
and this has to be said before its result is read.** AC-4(b) proposes taking
each gutter's *bottom* from `panel-edge-search.md` section 7 — `2708` 1352,
`3538` 1404, `2698` 1254. Every one of those lies **below** the recorded mark
(1352 > 1322, 1404 > 1330, 1254 > 1085). Widening in that direction admits rects
that reach *further past* the mark, while the defect the band addresses is rules
stopping *above* it. It therefore cannot retire a "stops above the mark" clip at
any width. Measured, it does exactly that:

```
$ cargo run --bin bsweep -- wide      # gutter BOTTOM in place of gutter top
   every headline number identical to `off`;  8 659 clipped scorings (= off)
$ cargo run --bin bsweep -- super     # mark ∪ gutter top ∪ gutter bottom
   every headline number identical to `frozen`;  8 272 clipped scorings (= frozen)
```

`super` is a strict superset of the frozen band and retires **exactly zero**
extra scorings, which confirms the gutter bottoms contribute nothing rather than
merely failing to change a headline.

**So the control was re-pointed in the direction the ruling actually points**,
which is the one thing in AC-4(b) that is oracle-free and had to be invented:
the band **doubled upward**, `2 × gutter_top − mark` — 1288 on `2708`, 1298 on
`3538`, 1075 on `2698` — a strict superset of the frozen band, plainly wider
than the ruling and plainly not it.

```
$ cargo run --bin bsweep -- double
spread:   best-ranked = 8 of 21, 4 clips  [panel art_th 8.0 gap 8 backoff 2]
spread:   best CLIP-FREE = 0 of 21 ;  lowest clips anywhere = 1
coverage: best-ranked = 8 of 21, 9 clips  [cover dev 20 min_cover 0.10 gap 4 backoff 0]
coverage: best CLIP-FREE = 0 of 21 ;  lowest clips anywhere = 1
$ cargo run --bin binert -- double
   8 212 clipped scorings, 1 922 hits     (frozen: 8 272 / 1 893)
```

**The control fires: the doubled band retires 60 more clipped scorings than the
frozen one** (49 on `2708`, 11 on `3538`), and 29 more hits. So the
frozen band is **not decorative** — the measurement is demonstrably sensitive to
band width, and the 387 scorings it retires are a property of the ruled width
rather than of an arbitrarily generous window.

And the finding that matters more: **a band twice the ruled width moves no
headline number at all.** Same leaders, same 8 of 21, same clip-free 0 of 21,
same lowest clip counts, and still **0 clip-free members of 780**. Growing the
band does not improve the numbers this story exists to move, which is MC-032's
caution — "a band that grows until the numbers improve is not a tolerance, it is
the oracle being edited to pass a test" — answered with a measurement instead of
a promise.

### (c) MC-034's own three controls are cited, not re-run

From `panel-edge-search.md` section 4. A predicate that widens three **marked
bottoms** cannot touch any of them:

- **(a) a neighbour's overhang** — a *synthetic page* with no manifest entry, so
  no mark and no band;
- **(b) the seven `"expect": "flag"` entries** — they carry no marked rectangle
  at all, so there is no bottom to widen;
- **(c) a flat full-width row inside the panel** — again a synthetic page with
  no manifest entry.

**The exception applies to the spread family only.** Control (b)'s "the best
coverage rule crops 3 of 7" is a property of that member, and the coverage
leader is **unchanged**, so 3 of 7 stands unre-read. The **spread** leader
changed, so it was re-read: `panel art_th 8.0 gap 8 backoff 2` crops **2 of 7**
— identical to the 2 of 7 MC-034 reports for its own best spread rule, and still
a regression against the **1 of 7** no-regression baseline. Control (b) fires on
the new leader exactly as it did on the old one.

---

## 5. The verdict against MC-032's parking (AC-5)

### The threshold, stated before the numbers

MC-019's bar is **20 of 21 with zero clips**. MC-034's per-file ceiling is **10
of 21**, and unparking MC-032 needs the **ceiling** to clear the bar — not the
clip count, and not the headline. `## Context` derives from
`panel-edge-search.md` section 2c that the ceiling **cannot move**: all three
banded entries are already `both: yes` there, widening a predicate can only turn
a `NO` into a `yes`, and none of the eleven entries nothing places carries a
band. So, stated plainly and in advance:

> **This story cannot unpark MC-032 and is not trying to.**

### The verdict: the clip numbers moved, and the clip-free best did not

Of MC-035's two exactly permitted verdicts, this is the first — **the clip
numbers moved** — with the movement confined below the headline:

- **387 of 8 659 clipped scorings retired** (−4.5 %), 239 in the spread family
  and 148 in the coverage family;
- the **spread family's best-ranked member changed identity**, gaining two
  entries at the same clip count — 6 of 21 with 4 clips becomes 8 of 21 with 4;
- the **lowest clip count anywhere in the spread family fell from 2 to 1**;
- the **coverage family's leader did not move at all**, for the reason in 3d;
- the **best clip-free rule stayed at 0 of 21 in both families** — and *0 of 780
  members is clip-free*, before, after, and under a band twice as wide.

**What that implies for a future clip-free row rule.** MC-005 decision 13 ranks a
clip above every other defect and `corpus.md` calls it the defect the whole
design is arranged against, so a clip-free rule is the only kind this product
could ship. The band was the last cheap thing that might have produced one, and
it does not: after it, the best any of 780 members manages is a single clip, on
a member that places **0 of 21** rows. The two numbers are not close to being
reachable together — the members with few clips are the ones that over-reach so
far that they hit nothing, and the members that hit anything clip nine times.
That is a shape, not a tuning gap, and no band width tried changes it.

### What changes elsewhere

- **MC-032 `## Parked`, item 1** — "How much of the deficit that recovers is
  **unmeasured**" is no longer true: it is measured here, it recovers nothing at
  the ceiling and nothing at the headline, and the parking stands with its reason
  unchanged.
- **EPIC-05's closing amendment** — "this epic closes with one measurement
  outstanding rather than none" can be read as settled: the outstanding
  re-score was run, and the epic's row-axis debt is now closed on measurement
  rather than on argument.

Both are one-line pointers to this document; neither story is unparked and no
acceptance criteria are written here, which remains the user's call.

---

## 6. Reproducing every number (AC-6)

The harness is MC-034's, **outside the repository**, exactly as MC-025's,
MC-026's, MC-028's, MC-031's and MC-034's were:

```
C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\
  d2462a4d-1798-4ec1-a131-529a1297daa3\scratchpad\mc034
```

MC-034's nine binaries are untouched. This story adds one module, `src/band.rs`
— the band table, read out of `corpus.md`, and the disjunction — and three
binaries beside them. `ceiling` was **not** run: `## Context` derives that the
per-file ceiling and the per-edge reachability cannot move, and no such number
appears above.

| Command | What it produced |
|---|---|
| `cargo run --bin sweep` / `--bin sweep2` | section 2: MC-034's own numbers, re-run unchanged |
| `cargo run --bin bsweep -- off` | section 2: the same numbers through the widened harness with the widening disabled |
| `cargo run --bin bsweep -- frozen` | section 3a: every "after" headline number, both families |
| `cargo run --bin bsweep -- wide \| super \| double` | section 4(b): the three over-wide bands |
| `cargo run --bin bdetail -- <mode> panel 8.0 8 2` | section 3c, spread leader, before and after |
| `cargo run --bin bdetail -- <mode> panel 6.0 12 0` | section 3c, the old spread leader |
| `cargo run --bin bdetail -- <mode> cover 20 0.10 4 0` | section 3c, coverage leader, before and after |
| `cargo run --bin binert -- <mode>` | sections 1, 3b, 4(a), 4(b): all 16 380 scorings, one line each |
| `cargo run --bin binert -- "probe4:<file>:<bottom>"` | section 4(a), shown firing |

`<mode>` is `off \| frozen \| wide \| super \| double \| probe4:<file>:<bottom>`.
Numbers read out of `corpus.md` or `panel-edge-search.md` cite the section they
come from and are **not** re-measured; `panel-edge-search.md` is not edited, as
MC-033 left MC-018 alone.

**This document writes no production or test code.** `Tuning::default()`, the
corpus and `fixtures/corpus/manifest.json` are untouched, nothing is committed
under `crates/`, and the three entries are deliberately **not** tagged in the
manifest — that needs a vocabulary entry and a test, and `corpus.md` says it is a
story of its own.
