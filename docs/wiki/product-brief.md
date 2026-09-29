# Product brief: Manhwa Cropper

## Summary

A small, fast, fully offline Windows desktop app that takes screenshots of
manhwa / manga panels captured from a desktop browser or reader and
automatically crops them down to the artwork, removing browser chrome, side
gutters and solid borders. Input arrives by drag-and-drop or Explorer
"Send to"; every file is processed without a preview and written, with its
original name and format, to an output folder the user chooses. When the
detector is not confident it copies the file unchanged and flags it rather
than risk clipping art. It is a single-user tool for the author; the whole
point is that the "open an editor and hand-crop" step disappears from their
routine.

Interviewed: 2026-09-10. Every statement below is the user's answer unless
marked *(PO recommendation, agreed)*.

## 1. Problem

Reading manhwa / manga in a desktop browser and screenshotting a panel
leaves an image with browser tabs, URL bar, dark or white side gutters and
sometimes bars above/below the art. Today the user opens an image editor and
hand-crops each one. That step is slow, repetitive and the thing they want
gone.

What they do instead today: manual cropping, one image at a time, in a
general-purpose editor.

## 2. Users

Exactly one user: the author, on their own Windows PC, for personal use.
Comfortable with computers; needs no onboarding, tooltips or hand-holding.
Not intended to be shared with friends or a community in v1.

## 3. The first five minutes

First run:

1. Launch the app. A single small window appears: a drop zone, an
   output-folder picker, a progress indicator and a result count.
2. Pick an output folder (remembered for next time - *PO recommendation,
   agreed in principle; see open questions*).
3. Drag one or more screenshots onto the window. They are cropped
   immediately with no preview and written to the output folder.
4. The window shows how many were cropped and how many were flagged as
   uncertain (copied unchanged).

Tenth run: select files in Explorer, right-click, "Send to", Manhwa
Cropper. Same result, no interaction needed beyond that.

What brings them back: it is faster than opening an editor, and the result is
right often enough that they stop checking.

## 4. Core features (v1)

Load-bearing, in priority order:

1. **Automatic crop of one image.** Find the artwork's bounding rectangle and
   cut everything outside it. Two mechanisms, both required:
   - trim uniform solid-colour borders (white, black, grey gutters) on all
     four sides;
   - detect the art's edges so that non-uniform regions outside the panel
     (browser tab strip, URL bar, reader UI) are also removed. The bounding
     box is the largest content region delimited by strong horizontal and
     vertical edges.
2. **Never clip artwork.** The crop is biased outward: leaving a few pixels
   of border is acceptable; cutting into the panel is a defect.

   **Amended 2026-09-23, by the user's decision: no border of page
   background is left on any side.** The user's words:

   > The cropping from the side isnt tight enough and I still see the page on
   > the right and left side.

   The "few pixels of border" was a fixed 3 px outward margin
   (`architecture.md` decision 5). On the marked corpus it added nothing but
   page background, on 42 of 42 sides, and that was the band the user saw.
   The user chose a margin of **0 on all four sides**
   ([MC-049](../backlog/stories/MC-049.md), Open question 1). **Never clip is
   untouched and still absolute.** It now rests on the page-column locator
   stopping exactly at the art, with no slack behind it, and the corpus
   zero-clip tests are what hold it there. Seven hand marks that included
   flat page columns were corrected as the user ruled (`corpus.md`,
   "Corrected marks").
3. **Uncertain: copy unchanged and flag.** If no confident crop is found
   (e.g. the whole screenshot is art, or the page is mostly white), the
   original is written to the output folder untouched and listed in the
   run summary. Nothing is silently lost or damaged.
4. **Batch input** by drag-and-drop of one or many files onto the window, and
   by Windows Explorer "Send to" (the app accepts file paths as command-line
   arguments).
5. **Output folder** chosen by the user; every result goes there with the
   original filename and original format.
6. **Formats**: PNG, JPEG and WebP in. PNG stays PNG at original pixel
   resolution with no resampling. JPEG and WebP are re-saved at maximum
   quality (see open question on lossless JPEG cropping).
7. **Run summary**: count cropped, count flagged, and which files were
   flagged.

Pushed back on and dropped from v1: preview/adjust UI (user chose fully
automatic), review list of low-confidence thumbnails (the flag in the summary
is enough), clipboard and hotkey input, watch folder.

## 5. Explicit non-goals (v1)

- Any image editing beyond cropping: no rotate, resize, filters, text
  removal, upscaling.
- Splitting one image into several panels. One input file yields one output
  file.
- Watch-folder, clipboard capture, global hotkeys.
- Preview, manual adjustment, or an approval step.
- Mac, Linux, phone, web. Windows only.
- Multi-user features, sharing, accounts, telemetry, updates over the network.

**Amended 2026-09-21, by the user's decision: the detector may use knowledge
of a specific reader's furniture, but only on top of a rule that works
without it.** Matching a known site's navigation bar, header or footer — which
is pixel-identical across every screenshot from that site — is permitted as an
input to the crop decision. It is **an optimisation, never the mechanism**: the
user's words are *"it should work on any reader"*, so the detector must reach
the bar on a reader it has never seen, and per-site knowledge may only improve
a site it recognises on top of that. A rule that is *only* a per-site matcher
does not satisfy this brief, however well it scores on the sites in the corpus.

This has a consequence worth stating in the same breath, because it is the cost
of the decision: **the general rule does not exist.** Seven investigations have
looked for one and all seven came back reasoned negatives (below), so at the
time of writing nothing meets the requirement this amendment sets, and
[`EPIC-07`](../backlog/epics/EPIC-07.md) story 3 is parked rather than built —
see [MC-041](../backlog/stories/MC-041.md).

The brief was **silent** on this rather than against it, and the amendment is
recorded here because the silence was being read as a prohibition.
[`EPIC-07`](../backlog/epics/EPIC-07.md) says "v1's brief excludes per-reader
special cases" and MC-038's `## Out of scope` repeats it; what section 4 and
this section actually do is describe a **generic** detector — "the largest
content region delimited by strong horizontal and vertical edges" — and never
contemplate per-site knowledge either way. This makes the answer explicit
instead of inferred.

Why it changed: seven investigations have now looked for a general pixel-level
rule that places the **row** edges, and all seven came back reasoned negatives
— MC-026, MC-028, MC-031, MC-032, MC-034, MC-035 and
[MC-038](../wiki/region-row-search.md), the last of which also closed the "2D
structure" opening `EPIC-07` was built on. `architecture.md` decision 14 records
what that costs the product today: the row axis places **0 of 21** and overshoots
every edge by 97 to 310 px, and on a typical screenshot that band *is* the
browser and OS furniture this tool exists to remove.

**What does not change**, and a site matcher that breaks any of these is not
shippable:

- **Never clip** (section 4.2, `architecture.md` decision 13). Unchanged and
  still absolute.
- **An unrecognised reader is not a reason to flag.** Section 4.3's
  flag-and-copy is for a screenshot the detector cannot crop confidently — an
  all-art page, a mostly-white one — and **not** for a site the matcher has
  never seen. An unseen reader gets the general rule's answer, exactly as it
  would if no matcher existed; the tool must get no worse on it than it is now,
  and must not start flagging files it crops today.
- **Offline** (section 6). Furniture is recognised from the pixels in front of
  it; nothing is fetched, and no site is contacted.
- **No settings pane, no tuning file** (`architecture.md` decision 11). The user
  does not maintain a list of sites.
- One input file still yields one output file, in its original format.

**What this does not open — and the decision it now forces.** This amendment
permits **per-site furniture matching as an optimisation** and nothing else.
Colour and chroma stay ruled out (MC-025 `## Context`). Re-marking the corpus
stays declined (the 2026-09-17 decision; `EPIC-07` "deliberately not in this
epic").

Learned or model-based detection is **not opened here, and is now the only
untried signal class left for the general rule.** `EPIC-07` defers it "until
stories 2 and 3 have reported": story 2 reported a reasoned negative
([MC-038](region-row-search.md)), and story 3 is answered not by measurement but
by this decision — per-site matching cannot be the mechanism, so it cannot be
the general rule either. Both of the epic's cheaper ideas are therefore spent.
Opening learned detection is a separate decision, with its own cost (it needs
the grown corpus most of all, and the brief's **offline** constraint means any
model ships inside the exe and runs locally), and it has not been taken.

One caution for whoever measures it, from `docs/wiki/corpus.md`: the corpus
spans seven readers with counts as low as one (`demonicrevolution`), so a
matcher evaluated on `toongod` (10 entries) and on that one entry is not being
evaluated on the same thing twice.

**Amended again 2026-09-21, and this one changes the target: the default crop
removes *furniture*, not *everything that is not artwork*.** The user's words:

> Ignore the gutters between panels, ignore art that overhangs into the
> gutters, ignore sound effects / non-typical speech bubbles that overhang onto
> the gutter or art. Just crop out the sides (which I think we already did),
> and crop out the browser artifacts and taskbar artifacts and any reader
> artifacts. A user can choose to crop more, but that should be the default.

So the default output is **the reader's page content area**: the side gutters
gone (already done — 20 of 21 on the column axis), the browser tab strip,
bookmarks bar and URL bar gone, the Windows taskbar gone, and the reader site's
own header, navigation and footer gone. What is **deliberately left in**:

- **page gutter above and below the artwork** — the crop does not look for a
  panel boundary, and no longer tries to;
- **gutters between panels**, for the same reason;
- **art that overhangs into a gutter**, a sound effect, or an atypical speech
  bubble crossing into the gutter or onto a neighbour. None of these is a
  defect any more. They were the single cause of every failure below, and they
  are now out of scope rather than unsolved.

**Never clip** (section 4.2) is untouched and still absolute. Section 5's
"no preview, manual adjustment, or approval step" is also untouched: *"a user
can choose to crop more"* means they are content to hand-crop the occasional
image, not that the app grows an adjustment UI.

**Why the target moved.** Seven investigations looked for the artwork's own
edge on the row axis — MC-026, MC-028, MC-031, MC-032, MC-034, MC-035,
[MC-038](region-row-search.md) — and all seven came back reasoned negatives.
Every one of them failed on the same thing: telling page gutter from panel
gutter when a bubble crosses it. Dropping that requirement does not work around
the problem; it removes it.

**This reverses one half of the 2026-09-17 decision, and the reversal is
specific.** MC-031's Option B — *"no output contains browser or OS chrome"* —
was refused that day because **the site's own navigation survived into the
crop**. The target above is Option B **with the reader's furniture removed
too**, which is precisely the objection that sank it. `architecture.md`
decision 14 carries the same amendment.

**What this does to the measurement, decided here so no story has to guess.**
The corpus marks **stay tight and stay as they are** — nobody re-marks 59
screenshots. Their role changes: they are the **never-clip oracle**, not the
row-axis accuracy target. A crop is right when it (a) contains the marked
rectangle, exactly as today, and (b) contains no browser, OS or reader
furniture. That is a containment-and-absence predicate rather than an 11 px
window, it needs a furniture oracle rather than a re-marking, and MC-031
section 9 already built two-thirds of one. If a direct accuracy number is
wanted later, re-marking to the page content area is the way to get it, and it
is a separate decision with the user's time in it.

## 6. Constraints

- **Platform**: Windows 11 desktop (user's machine: Windows 11 Home).
- **Offline**: no network access at all. Nothing leaves the machine; no
  telemetry.
- **Performance**: a batch of 100 typical screenshots (1080p to 4K) completes
  in under about 10 seconds on an ordinary PC.
- **Lossless**: never degrade the image. PNG in means identical pixels out
  inside the crop, same resolution. No lossy re-encode of PNG.
- **Packaging**: installer vs single portable exe was offered; the user did
  not require a portable exe, so either is acceptable. *(PO recommendation:
  aim for a single small exe anyway; it costs little with a native stack.)*
- **Stack**: user deferred to the planner. *(PO recommendation, agreed: a
  compiled native stack, Rust or C#/.NET, for a small, fast-starting,
  offline Windows exe with a minimal window. `/plan-product` decides.)*
- **Budget / deadline**: none stated; personal project.
- **Data**: only the user's own screenshots, on their own disk.

## 7. Success

Observable, on the user's real screenshots:

- **At least 9 of 10 need no manual fix.** At most one in ten comes out with
  art clipped or junk left in.
- **Zero clipped panels** across the calibration set. Leftover border is a
  minor miss; cut-into art is a failure.
- **The user stops opening an image editor** for cropping.
- The performance target from section 6 holds: 100 images in under about
  10 seconds.

To make the first two measurable, planning must define a test corpus of
real screenshots with hand-marked expected crop rectangles (see open
questions).

## 8. Look and feel

- One small window: drop zone, output-folder picker, progress bar, and a
  "N cropped, M flagged" result line with the flagged filenames.
- Native Windows look; follows the system light/dark mode.
- No settings pane, no thumbnails, no theme customisation.
- Tone: utilitarian, quiet, invisible when it works. The best UI outcome is
  the user never reading anything on it.

## Open questions

1. **Calibration corpus.** The success criteria need a set of the user's
   real screenshots (varied readers, light and dark browser themes, white
   and black gutters, at least one "all art" and one "mostly white" case)
   with expected crop rectangles. The user must supply these before the
   crop algorithm's stories can have tests that mean anything.
2. **Lossless JPEG.** Cropping a JPEG at arbitrary pixel offsets forces a
   re-encode, which is not strictly lossless. Options: accept max-quality
   re-encode (what the user chose), snap the crop outward to 8/16 px MCU
   boundaries and do a true lossless crop, or output PNG for JPEG inputs.
   Decide at planning; default is the user's choice (re-encode at max
   quality).
3. **Outward bias margin.** How many pixels of slack to leave around the
   detected box to honour "never clip". Proposed: a small fixed margin
   (2 to 4 px) that the corpus can tune. *Settled at 3 px, then changed to 0
   on 2026-09-23 by the user's decision; see section 4, item 2.*
4. **Output folder default.** Proposed: remember the last chosen folder;
   if none has ever been chosen and files arrive via "Send to", fall back to
   a `cropped` subfolder next to the first input file. Needs user agreement.
5. **Name collisions** in the output folder (same filename from two source
   folders). Proposed: overwrite is never silent; append a numeric suffix.
6. **"Send to" installation.** Whether the app creates its own Send-to
   shortcut on first run or the user creates it by hand.

## Next step

Run `/plan-product`. It will choose the stack, write
`docs/wiki/stack.md` and `docs/wiki/architecture.md`, have the Lead Designer
record the window design under `docs/wiki/design/`, and decompose this brief
into epics and stories under `docs/backlog/`, starting with a bootstrap story.
