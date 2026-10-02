# The held-out scores

`EPIC-07`'s bar, reported on held-out sets. Each set is scored once. The
newest score comes first. MC-051's and MC-063's scores are of the **same
detector**: nothing changed under `crates/core`, `crates/engine/src`,
`crates/app/src` or `Cargo.lock` between `d2876f5` and `1438b2b`. **MC-071's
is not.** Its detector is `1438b2b`'s plus the five fixes MC-065 to MC-070.
All three are on different screenshots, and they sit side by side. None
replaces another.

| | set | clips | meets the bar | verdict |
|---|---|---|---|---|
| [MC-071](../backlog/stories/MC-071.md), 2026-10-02, `4539e12` | the second fresh 15 (MC-068) | **2 of 15** | **11 of 15**, against 14 | **not met: a clip** |
| [MC-063](../backlog/stories/MC-063.md), 2026-09-30, `1438b2b` | the fresh 25 (MC-062) | **3 of 25** | **21 of 25**, against 23 | **not met: a clip** |
| [MC-051](../backlog/stories/MC-051.md), 2026-09-29, `d2876f5` | the first 19 marked, spent | 0 of 19 | 16 of 19, against 18 | not met |

**The MC-063 score speaks for crops on known readers only.** The fresh set
holds no unseen reader and no screenshot that should be left alone, so it
says nothing about either. MC-051's unseen-reader result (`manhwaclan` 2 of 2
met, `xbato` 0 of 1) is still the only unseen-reader evidence on record, and
`xbato` is no longer unseen.

**The MC-071 score speaks for crops on four known readers, mid-chapter, dark
pages.** The set holds no unseen reader, no screenshot that should be left
alone, no light page and no `w-network` page, so it says nothing about any of
those.

## MC-071, 2026-10-02: the second fresh held-out set

[MC-071](../backlog/stories/MC-071.md) carries the working, with every command
and its output. This section carries the result and what it spends.

### The verdict

**A clip: 2 of 15. The furniture half is not met either: 11 of 15 meet the
bar, against 14 required.** This is MC-071 AC-4 (c). Zero clips is absolute,
so the clips decide the verdict whatever the furniture count. The user was
asked before anything was acted on, as Open question 4 required, and answered
"Record as not met".

| | result | bar |
|---|---|---|
| clips | **2 of 15** (row 0, column 2) | 0 (absolute) |
| meets the bar (contains the mark, and no furniture) | **11 of 15** | at least 14 of 15 |
| reader furniture absent | 15 of 15 | — |
| browser and OS furniture absent (the scrollbar counts as browser) | 12 of 15 | — |
| marked entries answered `Flagged` | 0 of 15 | — |
| viewport located / declined | 12 / 3 | — |

Per reader (entries, zero-clip, furniture-free, meets, viewport located):
`demonicrevolution` 1 1 0 0 0; `rolia-scans` 4 4 4 4 4; `toongod`
7 5 5 4 5; `xbato` 3 3 3 3 3.

### The run

- **Scored commit:** `4539e12`. Its `crates/`, `Cargo.toml` and `Cargo.lock`
  are identical to `main` after MC-070 (`b9cf008`).
- **Harness:** a standalone crate outside the repository (session
  `d6b1fa50…` scratchpad, `mc071-harness/src/main.rs`, sha256
  `96b15608…5413`, unchanged before and after the run). It calls
  `process_file` at `Tuning::default()` (margin 0, the app's own setting),
  release build, and asks MC-048's viewport stage directly.
- **Before the run:**
  - The same harness, built against `1438b2b`, reproduced MC-063's aggregate
    exactly on its 25 spent entries. Without the scrollbar row it reads 22 of
    25, so that check can fail.
  - The controls fire on 15 of 15: a whole-image crop shows furniture, and a
    mark shrunk by 10 rows clips.
  - No suite under `crates/` crops a held-out entry.
- **Once:** run at 2026-10-02T03:35:22Z. A second invocation is refused. One
  earlier invocation stopped before its first crop, while creating the output
  file, and MC-071 `## Notes` shows where.
- **Output:** `mc071-out/heldout-run.txt`, sha256 `44eee68a…20e6`, quoted
  verbatim in MC-071 `## Notes`.

### The oracle

There is no site header on any of the 15, so `T` is the browser-bar end, the
mark's top. Eleven marks end on their group's usual bottom row, so `B` is the
mark's bottom. For the four that stop short (`n02`, `n06`, `n10`, `n15`), the
user ruled the scrollbar, site-footer and taskbar starts blind on *Four Fresh
Screen Bars*, a page that drew no crop and no mark. No site footer was ruled
on any of them, and there is a scrollbar on three. Checked against the marks,
there are 0 conflicts. The oracle was frozen in `4539e12` before the run.

### The four failing entries

Per-file for failures only (Open question 3), read after the totals were
written:

- **`n02`** (`2025-03-07 00_05_58.png`, `toongod`). **A clip.** The crop is
  columns 703..1101 against the mark's 643..1176, which cuts 60 columns on the
  left and 75 on the right. It also runs the full screen height (rows
  0..1440): browser chrome, scrollbar and taskbar.
- **`n05`** (`2025-11-01 12_34_31.png`, `toongod`). **A clip of one column.**
  The crop starts at column 1007 against the mark's 1006. Its rows are exact.
- **`n06`** (`2025-03-13 12_01_01.png`, `toongod`). No clip, and its columns
  equal the mark's. The crop runs the full screen height, so it keeps browser
  chrome, scrollbar and taskbar.
- **`n13`** (`Screenshot (2507).png`, `demonicrevolution`). No clip. The crop
  starts at row 40 against `T` 133, so it keeps 93 rows of browser chrome.

The viewport stage declined on 3 of 15, and three of the four failures keep
browser chrome. Whether those are the same three is not established here. The
per-file lines do not carry the viewport, and diagnosing that is the follow-up
stories' job.

### What it forces

These are named, not filed:
- a chore, MC-064-shaped, that moves `n02`, `n05`, `n06` and `n13` to
  `tuning`;
- a fix for `n02`'s narrow, full-height crop;
- a fix for `n05`'s one-column cut on the left;
- a fix for the crops that keep the browser bar (`n06`, `n13`, and `n02`'s
  rows).

### What has been spent

- **The second fresh set has been scored.** The 15 gave their one score at
  `4539e12`.
- **A later run is a re-score.** Any later run over these 15 is a
  **re-score on spent held-out**. It is labelled that way wherever it is
  reported, and is **never** `EPIC-07`'s held-out score.
- **Read entries are tuning.** `n02`, `n05`, `n06` and `n13` were read per
  file, so they are `tuning` for any later rule. Moving them in the manifest
  is the follow-up chore.
- **The other 11 stay `held-out` in the manifest**, but they are spent.
- **A later held-out score needs another fresh draw** by MC-068's procedure,
  with a new seed, picked by the Lead PO, at most 20.

## MC-063, 2026-09-30: the fresh held-out set

[MC-063](../backlog/stories/MC-063.md) carries the working. This section
carries the result and what it spends.

### The verdict

**A clip: 3 of 25. The furniture half is not met either: 21 of 25 meet the
bar, against 23 required.** This is MC-063 AC-4 (c). Zero clips is absolute,
so the clips decide the verdict whatever the furniture count.

| | result | bar |
|---|---|---|
| clips | **3 of 25** | 0 (absolute) |
| meets the bar (contains the mark, and no furniture) | **21 of 25** | at least 23 of 25 |
| reader furniture absent | 25 of 25 | — |
| browser and OS furniture absent (the scrollbar counts as browser) | 23 of 25 | — |
| marked entries answered `Flagged` | 0 of 25 | — |
| viewport located / declined | 24 / 1 | — |

- **Scored commit:** `1438b2b`. Its crates are identical to `main` at
  `0addc64`, and `git diff d2876f5 1438b2b -- crates/core crates/engine/src
  crates/app/src Cargo.lock` is empty.
- **Date:** 2026-09-30, 22:36:14 to 22:36:18 UTC.
- **The bar:** 23 of 25 is the brief's 9 in 10 of 25 (22.5), rounded up. The
  same arithmetic gave MC-051 its 18 of 19. The user confirmed 23 of 25
  before the run.
- **What the result says:**
  - **All 3 clips are on the column axis.** No crop clips a row on any of the
    25.
  - **On 23 of 25 the rows are furniture-free.** The two that are not follow
    their column failures (`f13` and `f18`, below).
  - **"Reader furniture absent" is not a finding.** No site header or footer
    was ruled on any fresh entry, because the screenshots are mid-chapter, so
    this set did not test it.

### The run

- **Command:** `./target/release/run.exe`, run once, in the harness
  directory
  `C:\Users\ryanc\AppData\Local\Temp\claude\C--Users-ryanc-Projects-manhwa-cropper\c5832c1c-167d-43a7-bf8f-6d2bb3c7c672\scratchpad\mc063-harness`.
  - The harness is outside the repository. It depends on `crates/core` and
    `crates/engine` by path.
  - It uses the repository's `Cargo.lock` and release profile.
  - A second invocation exits 2 with `REFUSED: out/heldout-run.txt exists`.
- **Harness source hash:**
  `270a85cf445e8451535da1226a3ae0e8f63cf6db832b4e5b6cb26ce8388bfe87`.
  - It is the sha256 of `cat Cargo.toml Cargo.lock src/lib.rs src/bin/*.rs`.
  - It was recorded and committed (`753e9f1`) before the run.
  - It was re-checked before and after the run.
- **Held-out filter:** `src/lib.rs::held_out()` drops every manifest entry
  whose `split` is not `held-out`, on the raw JSON, before any file is named
  or decoded.
  - It asserts 25 marked, 0 flag, and the per-reader counts.
  - It asserts that each mark equals MC-062's frozen mark.
- **Order of operations:**
  - **AC-2b** ran first, through a separate function, `mc051_spent_ruled()`.
    It reproduced MC-051's aggregate exactly on its 19 spent entries. That is
    a reproduction on spent entries, not a score.
  - **AC-2c's controls** fired on 25 of 25: a whole-image "crop" shows
    furniture, and a mark shrunk by 10 rows clips.
  - **The run** wrote its output file before its first crop.
- **The crop** is `process_file` at `Tuning::default()`:
  - for `Cropped`, the rect;
  - for `Flagged`, the whole image.
- **The viewport** is read from MC-048's stage directly, composed as
  `corpus_viewport_stage.rs` composes it.

The aggregate block, verbatim:

```
== AGGREGATE ==
clips: 3 of 25
meets the bar: 21 of 25 (bar: 0 clips and at least 23 of 25) -> NOT MET
  reader furniture absent: 25 of 25
  browser and OS furniture absent: 23 of 25
viewport: marked located 24 / declined 1
marked entries answered Flagged: 0 of 25
per reader (entries, zero-clip, furniture-free, meets, viewport located):
  demonicrevolution   2  2  2  2  2
  rolia-scans         4  3  4  3  4
  toongod            14 12 12 11 13
  w-network           2  2  2  2  2
  xbato               3  3  3  3  3

This score speaks for crops on known readers only. The fresh set holds no unseen reader and no screenshot that should be left alone, so it says nothing about either.
```

There is no flag line and no unseen-reader line. The fresh set holds no flag
entry and no unseen reader (the user's rulings of 2026-09-30, in MC-062).

### The oracle

- **The marks** are the user's 25, frozen in MC-062. Every mark runs the
  visible height, from the browser bar's end (row 115, 133 or 167) to the
  taskbar or just above it.
- **The furniture rows** were frozen in MC-063 `## Notes` before any crop.
  - **17 marks end on their group's usual bottom row.** For these, `T` is the
    mark's top and `B` is the mark's bottom.
  - **8 marks end short.** For these, the user ruled the bottom bars on the
    *Fresh Screen Bars* page,
    https://claude.ai/artifact/T3VoZKiExdC47F77ViAeBC. The page drew only the
    user's own lines.
- **Four of those 8 end on a browser horizontal scrollbar**, not a site
  footer. The user ruled that the scrollbar is browser furniture: *"No, leave
  it out."*
- **No site header or footer** was ruled on any of the 25.
- **Furniture-free** means `crop.y >= T` and `crop.y + crop.h <= B`, exact to
  the row.

### The four failing entries

Open question 2 permits a per-file look at the failures only. These four are
now `tuning` for any later rule. The 21 passing entries were not read per
file.

| id | file | reader | cause | crop rows | crop cols | T / B | mark rows | mark cols |
|---|---|---|---|---|---|---|---|---|
| `f09` | `2025-12-08 17_22_50.png` | `toongod` | **clip** on the right edge: 2 columns of art cut | 167..1400 | 1006..1537 | 167 / 1400 | 167..1400 | 1006..1539 |
| `f13` | `2025-03-16 22_47_44.png` | `toongod` | browser scrollbar kept (the crop's bottom is past 1392); no clip | 115..1400 | 635..2557 | 115 / 1392 | 115..1392 | 635..1168 |
| `f18` | `2025-03-06 12_48_06.png` | `toongod` | **clip**: the crop shares no column with the mark; browser chrome and taskbar kept | 0..1440 | 1828..2545 | 115 / 1399 | 115..1399 | 651..1168 |
| `f20` | `2025-08-07 01_13_55.png` | `rolia-scans` | **clip** on the right edge: 6 columns of art cut | 115..1400 | 1022..1516 | 115 / 1400 | 115..1400 | 1022..1522 |

What they say, without anyone having looked at the images:
- **`f09` and `f20` are small right-edge cuts** with exact rows. `f20` is one
  of the four the user ticked "diagonal" while marking. The user ruled that
  the diagonal lies between panels and moves no edge.
- **`f18` is a crop of the wrong region.** It is `Cropped`, not `Flagged`.
  The crop runs the full height on the right of the screen, and the mark is
  on the left half. MC-052's two split-screen screenshots had their marks
  placed the same way, so a second window is one possible explanation. It is
  not a finding.
- **`f13`'s crop is too wide**, and it keeps the scrollbar. Its mark is also
  on the left half.
- **Diagnosing them is [MC-065](../backlog/stories/MC-065.md)'s job**, once
  [MC-064](../backlog/stories/MC-064.md) has moved the four to `tuning`.

### What has been spent

- **The fresh set has been scored.** The 25 gave their one score at
  `1438b2b`.
- **A later run is a re-score.** Any later run over these 25 is a
  **re-score on spent held-out**. It is labelled that way wherever it is
  reported, and is **never** `EPIC-07`'s held-out score.
- **Read entries are tuning.** `f09`, `f13`, `f18` and `f20` were read per
  file, so they are `tuning` for any later rule. Moving them in the manifest
  is [MC-064](../backlog/stories/MC-064.md).
- **The other 21 stay `held-out` in the manifest**, but they are spent.
- **A later held-out score needs a fresh draw with a new seed**, by MC-062's
  procedure.

## MC-051, 2026-09-29: the first held-out set (spent)

The sections below are MC-051's record, unchanged.

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
