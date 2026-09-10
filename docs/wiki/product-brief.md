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
   (2 to 4 px) that the corpus can tune.
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
