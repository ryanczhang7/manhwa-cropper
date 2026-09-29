# The held-out score (MC-051)

`EPIC-07`'s bar, reported once on the held-out set, the number v1 never had.
This is a spike record: [MC-051](../backlog/stories/MC-051.md) carries the
working, and this page carries the result and what it spends.

## 1. The verdict

**Not met. 16 of 19 marked held-out entries meet the bar, against 18 required,
with 0 clips on 19 of 19.**

| | result | bar |
|---|---|---|
| clips | **0 of 19** | 0 (absolute) |
| meets the bar (contains the mark, and no furniture) | **16 of 19** | at least 18 of 19 |
| reader furniture absent | 17 of 19 | — |
| browser and OS furniture absent | 18 of 19 | — |
| marked entries answered `Flagged` | 1 of 19 | — |

- **Scored commit:** `d2876f5` (`main`, after MC-049).
- **Date:** 2026-09-29, 18:44:16 UTC.
- **Verdict:** MC-051 AC-4 (b). The absolute half holds; the furniture half
  falls 2 short.

The bar is the brief's 9 in 10, rounded up. The story was written against 24
marked entries (22 of 24). MC-052, MC-053 and MC-055 moved five to `tuning`
first, so it reads 18 of 19. The user confirmed 18 of 19 before the run.

**The miss does not rest on the ruling's precision.** One of the three
failures (§5, `h11`) turns on a single row the user ruled as site header. Had
it been ruled "none visible", the score would be 17 of 19, still short of 18.

## 2. The run

- **Command:** `./target/release/run.exe`, run once, in the harness
  directory
  `C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\f368f274-47db-4076-8b44-807cf994b9c3\scratchpad\mc051`.
  It is outside the repository and depends on `crates/core` and
  `crates/engine` by path. A second invocation exits 2 with `REFUSED: …
  out/heldout-run.txt exists`.
- **Harness source hash:**
  `c357de5fbb3303b6cb8ca06cfe3b9ea1f5dde7a2711d7538f52c83351451cb9f`. That is
  the sha256 of `cat Cargo.toml Cargo.lock src/lib.rs src/score.rs
  src/bin/*.rs`, recorded before the run and re-checked immediately before
  it.
- **Held-out filter:** `src/lib.rs::held_out()` drops every manifest entry
  whose `split` is not `held-out` before its file is read. It asserts 19
  marked and 7 flag.
- **Tuning path:** AC-2b uses a separate function, `tuning_ruled()`.
- **The crop** is `process_file` at `Tuning::default()`:
  - for `Cropped`, the rect;
  - for `Flagged`, the whole image.
- **The viewport** is read from the stage directly, composed as
  `corpus_viewport_stage.rs` composes it.

The aggregate block, verbatim:

```
== AGGREGATE ==
clips: 0 of 19
meets the bar: 16 of 19 (bar: 0 clips and at least 18 of 19) -> NOT MET
  reader furniture absent: 17 of 19
  browser and OS furniture absent: 18 of 19
viewport: marked located 19 / declined 0; flag located 1 / declined 6
flags: of 7, Flagged 5, Cropped 2
marked entries answered Flagged: 1 of 19
per reader (entries, zero-clip, furniture-free, meets, viewport located):
  demonicrevolution   1  1  1  1  1
  kunmanga            1  1  0  0  1
  manhwaclan          2  2  2  2  2
  rolia-scans         7  7  6  6  7
  toongod             5  5  5  5  5
  w-network           2  2  2  2  2
  xbato               1  1  0  0  1
```

**Unseen readers, as a direction and not a percentage** (`corpus.md`):
- `manhwaclan` met the bar on both of its entries.
- `xbato`'s one entry was flagged rather than cropped.

**The 7 flag entries** carry no bar (MC-051 `## Context`). The app flagged 5
and cropped 2. The viewport stage located a viewport on 1 of the 7.

## 3. The oracle: held-out furniture, ruled blind

The user ruled four rows per marked entry through the Artifact *Kept-Aside
Screen Bars*, https://claude.ai/artifact/1cTq9VKYMjD3jGbJc3Cvqy:
- browser bar end;
- site header end;
- site footer start;
- taskbar start.

Each could be "none visible". The page drew only the user's own lines. It
showed no crop, viewport, cropper row, proposal or mark. The harness step that
built its data emitted only these keys:

```
file, height, id, previews, previews.bot, previews.bot.file, previews.bot.scale, previews.bot.yOffset, previews.top, previews.top.file, previews.top.scale, previews.top.yOffset, previews.whole, previews.whole.file, previews.whole.scale, previews.whole.yOffset, reader, width
```

It took three passes, all before any crop ran:
- **Pass 1** (collection `rulings`) was clicked on the half-size preview, so
  every row fell on a multiple of 4. That is too coarse for an exact-row
  test.
- **Pass 2** (collection `rulings2`) added a pixel magnifier.
- **Three top lines were then re-checked.** The magnifier had opened on a
  speech bubble there, and it now opens at the image's left edge.

The rulings were frozen at 18:40 UTC. The frozen table is in MC-051
`## Notes`. The shape of the oracle:
- **Browser bar end:** 115, 133, 137 or 167, depending on the reader.
- **Taskbar start:** 1400 on the 2025-dated screenshots and 1392 on the
  `Screenshot (n)` ones.
- **Separate site bars:** only `kunmanga`'s header (to 319), a one-row header
  on one `rolia-scans` entry, and one-row footers on two `toongod` entries.

"Ends" is the first row below the bar; "starts" is the bar's first row.
- **T:** the browser bar end or the site header end, whichever is lower down
  the screen.
- **B:** the site footer start or the taskbar start, whichever is higher up.
- **Furniture-free:** `crop.y >= T` and `crop.y + crop.h <= B`, exact to the
  row.

## 4. Before the run (AC-2)

- **(a) Oracle against mark:** 0 conflicts. No ruled furniture lies inside a
  mark, so the ceiling is 19 of 19.
- **(b) Tuning reproduction:** the same scoring code against MC-050's round-2
  reader rulings on the 21 marked tuning entries it ruled. **0 clips on 21 of
  21, reader-furniture-free on 19 of 21**, which is `reader-furniture-search.md`
  §1 exactly. MC-049 moved the columns and every mark is still contained.
- **(c) Controls on held-out oracle data, no crop run:**
  - A whole-image "crop" reports furniture on 19 of 19. That is exactly the
    19 whose rulings name any.
  - Each mark shrunk by 10 rows at the top and bottom reports a clip on 19 of
    19.

  Both fire.

The full output is in MC-051 `## Notes`.

## 5. The three failing entries

Open question 2 permits a per-file look at failures only. These three are now
`tuning` for any later rule. The 16 passing entries were not read per file.

| id | file | reader | cause | crop rows | T / B | mark rows |
|---|---|---|---|---|---|---|
| `h09` | `2025-07-17 14_20_23.png` | `xbato` (unseen) | flagged `Ambiguous`, so the whole image is scored: browser chrome and taskbar | 0..1440 | 167 / 1400 | 171..1382 |
| `h11` | `2025-08-03 11_27_49.png` | `rolia-scans` | reader header: the crop starts at 115, the browser bar's end, and the user ruled one row of site header (115..116) | 115..1400 | 116 / 1400 | 118..1385 |
| `h21` | `Screenshot (68).png` | `kunmanga` | reader header: the site's same-tone header, 167..319, is left in | 167..1392 | 319 / 1392 | 392..1336 |

What they say:
- **`h21` is the failure MC-050 predicted.** The two tuning misses are both
  `kunmanga`'s same-tone header, and held-out's one `kunmanga` entry misses
  the same way. It is the one cause a furniture rule would have to fix.
- **`h09` was known in aggregate before this story.** MC-055's AC-5 count
  reported 1 of 19 `Ambiguous`. It is a crop the detector declined, not a
  crop that left furniture in.
- **`h11` is a one-row call.** The crop's top edge is exactly the browser
  bar's end, so MC-048's stage did its part. Whether a one-row line under the
  browser bar is the site's own header is the user's ruling, and it stands.
  As §1 says, the verdict does not depend on it.

## 6. What it forces

- **`EPIC-07` item 4 is not earned on held-out.** The epic's held-out score
  is **16 of 19 at `d2876f5`**, short of 18 of 19, with 0 clips.
- **The next story is a furniture rule for same-tone site headers**, as
  MC-050 §6 predicted. It is fitted on tuning plus the three entries above,
  now read. It must still work on any reader (MC-041 stays parked).
- **Why `h09` flags `Ambiguous` is a second, separate question.** It is the
  detector declining a page, not a furniture rule.
- **The held-out score is owed again.** Any later rule's score has to come
  from **new** screenshots, a story shaped like MC-037.

## 7. What has been spent

- **The whole set has now been scored.** The 19 marked and 7 flag held-out
  entries have given their one score, at `d2876f5`.
- **A later run is a re-score.** Any later run over these entries is a
  **re-score on spent held-out**. It must be labelled that way wherever it is
  reported, and is **never** `EPIC-07`'s held-out score.
- **Read entries are tuning.** `h09`, `h11` and `h21`, the three read per
  file, are `tuning` for any later rule.
- **Moving them in the manifest is a follow-up chore.** It is not done here,
  because the spike's phases cannot write `fixtures/`.
- **A fresh held-out score needs new screenshots from the user.**

**Prior contact, for the record.** Before this story, MC-049, MC-052, MC-053
and MC-055 each ran an aggregate zero-clip count over these entries (counts
only, and the user approved each). That weakens the claim that the **clip**
half was unseen, because a clip found by those runs was fixed, and the
clipped entries were moved to `tuning`. It does not touch the **furniture**
half, which nothing had measured before this run.
