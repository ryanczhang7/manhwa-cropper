# Architecture

Manhwa Cropper is one exe with a pure crop engine at its centre and two thin
entry points around it: a headless CLI mode (what Explorer "Send to" and the
tests use) and a single egui window. Everything that decides *where to crop*
runs on pixels in memory and never touches a file or a widget; everything that
touches files is in one crate that never touches a widget; the window is a
view over a state machine that is itself testable without egui.

## Components

```
   Explorer "Send to" / shell         drag-and-drop onto the window
            |                                   |
            v                                   v
   +-------------------+   args   +-----------------------------+
   | main.rs (entry)   |--------->| app: view-model (lib.rs)    |  AppState + Event, pure
   |  --no-gui? ------+          |  gui.rs = egui paint over it |  (gui.rs excluded from coverage)
   +-------------------+          +-----------------------------+
            |                                   |
            v          run(files, out, progress) v
   +---------------------------------------------------------------+
   | cropper-engine  (files in, files out; no egui)                 |
   |  args.rs       CLI parsing -> Invocation                       |
   |  codec.rs      decode PNG/JPEG/WebP -> Luma + keep source fmt  |
   |                encode: PNG lossless, JPEG q=100, WebP lossless |
   |  naming.rs     output path, collision suffix                   |
   |  batch.rs      rayon over inputs, progress channel, RunSummary |
   |  settings.rs   %APPDATA% settings.json                         |
   +---------------------------------------------------------------+
            |  Luma plane + Tuning                 ^ CropDecision
            v                                      |
   +---------------------------------------------------------------+
   | cropper-core  (pixels in, decision out; no I/O, no egui)       |
   |  luma.rs      Luma { width, height, data: Vec<u8> }, Rect      |
   |  trim.rs      uniform-border trim                              |
   |  edges.rs     row/column edge profiles, strong lines           |
   |  content.rs   largest edge-bounded content box                 |
   |  flat.rs      page-margin locator by per-line flatness (MC-025)|
   |  margin.rs    outward expansion, clamp                         |
   |  decide.rs    pipeline + flag reasons -> CropDecision          |
   +---------------------------------------------------------------+
```

### cropper-core (`crates/core`)

Pure functions over a `Luma` plane (8-bit luminance; the engine converts RGB
to luma with the BT.601 weights so JPEG chroma noise does not reach the
detector).

#### Two kinds of blank space, and the words for them

"Gutter" was used here for both of the blank regions in a reader screenshot,
which are different things with different detectors and different owners. They
now have separate names and this file uses them consistently.

- **Page margin** — the flat area to the **left and right of the page column**:
  browser background beside the strip, running the full height of the reader
  view. This is what stage 1's trim and MC-025's flatness locator address, and
  it is the sense every earlier occurrence of "gutter" in this file carried.
- **Panel gutter** — the white or black space **between panels**, running
  horizontally across the strip. Speech bubbles routinely sit on top of one; it
  is still a panel gutter and it is still not wanted in the crop. The user's
  hand-marked rects in the corpus (`fixtures/corpus/manifest.json`) bound a
  panel **between panel gutters**; everything outside them vertically is panel
  gutter, not artwork.

**The panel gutter is a v1 detection target that no current stage addresses.**
Stages 1 to 4 below locate the page margin and the chrome; nothing locates a
panel gutter, which is why the corpus scores what it does. MC-028 is the spike
that decides whether a rule exists; its evidence is in MC-025 and MC-026.

Responsibilities, in pipeline order:

1. **Uniform-border trim** (`trim`): from each edge inward, drop rows/columns
   whose pixels — measured *within the current rect* — span at most
   `Tuning.uniform_tolerance` from darkest to lightest (`max - min <=
   tolerance`). Passes repeat until no side changes, which is what makes
   full-width top/bottom bands and full-height left/right **page margins**
   both work without the detector knowing which it has. Yields the inner rect, or
   `None` when the whole image is one colour. (MC-003 settled this rule with
   the user. The earlier wording here — within tolerance of that edge's
   border colour, the median of the outermost row/column — lost because it
   judges a textured art row of `{c-10, c, c+10}` uniform and trims it.)
2. **Edge profiles** (`edges`): for each row, the mean absolute luma
   difference to the next row; likewise for columns. A **strong line** is a
   row/column whose profile value is at or above `Tuning.edge_threshold`,
   after merging runs of adjacent strong lines into one (taking the run's
   outer bound on the side nearest the image edge).
3. **Content box** (`content`): chrome is peeled from the outside in. On
   each side of the current rect, the *outermost strip* is the band between
   that edge and the nearest strong line parallel to it. The strip is
   **chrome-like** when all three hold: its extent is at most
   `Tuning.chrome_max_extent` of the image's dimension on that axis; its
   **flat fraction** - the share of its pixels within `uniform_tolerance` of
   the strip's median luma - is at least `Tuning.chrome_flat_fraction`; and
   the region left after removing it still has luma standard deviation at
   or above `Tuning.min_content_stddev`. A chrome-like strip is removed and
   the loop repeats on every side until no strip qualifies. A strip whose
   flat fraction falls within `Tuning.ambiguity_band` below the threshold
   is left in place and marks the result *ambiguous*. The box is what
   remains: everything that is not chrome, so a multi-panel page keeps all
   its panels instead of being cut to the largest one. Then a second uniform
   trim inside it, because removing chrome often exposes a **page margin**
   that was not an edge of the image a moment ago.
4. **Browser viewport** (`viewport`, MC-048), after MC-025's flatness locator
   and MC-027's page column: on the **row axis only**, clamp the rect's rows
   to the browser viewport — the rows between the browser chrome and the
   Windows taskbar. Per row, the share of the pixels **outside the page
   column, over the full image width**, within `uniform_tolerance` of the
   page background tone (the modal per-row margin median in 8-level bins); a
   row at 0.90 or above is page-like, and the viewport runs from the first
   run of at least 16 page-like rows to the end of the last. This is MC-031's
   chrome oracle as `chrome-row-search.md` §3b and §4 settled it, ported
   unchanged: 0.90 and 16 are module constants, not `Tuning` fields. When no
   such run exists the stage declines and moves nothing; there is no
   narrower fallback. The columns never move here.
   **Since MC-052 (2026-09-24) the stage makes a second reading, over the
   reader's own window, that can only widen the first.** The full-width
   reading above is kept exactly, and it alone decides whether the stage
   speaks. When it finds a viewport that overlaps the page column's rows, the
   stage finds the reader's window. From each side of the page column it reads
   outward over columns that are page background (a majority of their rows
   inside that viewport within `uniform_tolerance` of the tone), and stops at
   the first column that is not: the reader window's edge, such as its
   scrollbar. It then scores the rows again over those columns alone, with
   the same 0.90 and 16-row run. The viewport becomes the union of the two
   readings. Pixels past the window's edge, such as a second browser window on
   a split-screen screenshot, cannot pull the reader's rows out. Where MC-048
   declined, this still declines, and where it found rows over the page, it
   keeps all of them. The four settled values (0.90, 16 rows, the tone
   estimate, `uniform_tolerance`) are unchanged.
5. **Outward margin** (`margin`): expand by `Tuning.margin_px` on every side,
   clamped to the image — and, on the row axis, to the viewport stage 4
   located, so the margin never puts a row of chrome or taskbar back
   (MC-048 AC-7). "Never clip" is the property that the final rect contains
   the art rect on every fixture the generator can produce.
6. **Decision** (`decide`): `CropDecision::Crop(rect)` or
   `CropDecision::Flag(reason)`; see the data model for when each reason
   fires. The core never returns a rect it is not confident in; "not
   confident" is a discrete reason, not a score.

`Tuning` is a plain struct with `Default` holding the settled constants:

| Constant | Default | Tuned by |
|---|---|---|
| `uniform_tolerance` | 10 (per-channel, 0..255) | corpus story MC-019 |
| `edge_threshold` | 24 (mean abs luma diff) | MC-019 |
| `min_content_stddev` | 12 | MC-019 |
| `chrome_flat_fraction` | 0.85 | MC-019 |
| `chrome_max_extent` | 0.30 of the image's height (horizontal strips) or width (vertical strips) | MC-019 |
| `ambiguity_band` | 0.0025 (a strip with flat fraction in [0.8475, 0.85) is ambiguous) | corpus story MC-026 |
| `min_content_fraction` | 0.05 of the image area | corpus story MC-026 |
| `min_content_side` | 64 px | fixed |
| `margin_px` | 3 | MC-019 |
| `min_line_spread` | 8.0 (mean abs deviation of a line about its own mean) | MC-025 |
| `central_band_fraction` | 0.6 (share of a rect's rows the column locator measures over, centred) | MC-027 |

These are **settled** for RED (read them out; do not calibrate) and the corpus
stories may change them with the corpus as the evidence. MC-019 owns that
reservation, with one carve-out the user made after MC-026's measurements:
`min_content_fraction` and `ambiguity_band` are re-settled by **MC-026**,
because at their current values the corpus cannot judge any locator at all.

`central_band_fraction` is new in MC-027 rather than re-settled, and it is the
one constant in the table whose window is stated in two places: its floor and
its ceiling are fixtures in `crates/core/tests/flatness.rs`, so the required
`unit` gate fires when a later story leaves the window `(0.5, 2/3]`, while the
*corpus* floor is 0.58 and a fixture will not say so. The derivation, the
1 %-resolution sweep behind it and that warning are in MC-027's `## Test plan`
and `## Notes`; read them before moving it.

### cropper-engine (`crates/engine`)

Everything that reads or writes a file, and nothing that draws:

- **`codec`**: decode a path into pixels plus its `SourceFormat` (from the
  decoded container, not the extension); crop to a rect without resampling;
  encode back in the same format. PNG is encoded losslessly at the source
  bit depth and colour type (RGBA stays RGBA, greyscale stays greyscale), so
  pixels inside the crop are byte-identical. JPEG is encoded at quality 100,
  4:4:4. WebP is encoded lossless. Anything else is `Unsupported`.
- **`naming`**: `<out>/<stem>.<ext>`; on collision `<stem> (2).<ext>`,
  `(3)`, ... never overwrite, never silently.
- **`batch`**: `run(inputs, out_dir, tuning, progress) -> RunSummary`, files
  processed in parallel on the rayon pool, results in input order, one
  file's failure never aborting the others. A flagged file is *copied* to the
  output byte-for-byte (`fs::copy`), not re-encoded.
- **`settings`**: `Settings { output_dir }` in
  `<config_dir>/settings.json` where `config_dir` is
  `directories::ProjectDirs::from("", "", "manhwa-cropper").config_dir()`
  (`%APPDATA%\manhwa-cropper\config` on Windows), overridable with the
  `MANHWA_CROPPER_CONFIG_DIR` environment variable so tests never touch the
  real one. A missing or corrupt file is the defaults; the app never fails
  to start over its settings.
- **`args`**: `Invocation { inputs, out_dir, no_gui, summary_path }` from
  `argv`; unknown flags are an error, everything else is an input path.

### manhwa-cropper (`crates/app`)

- **`main.rs`**: parse args. If `--no-gui`: run the batch headlessly, write
  the JSON summary to `--summary <path>` if given, exit 0 when every input
  was cropped or flagged, 1 if any input could not be written at all, 2 on
  bad arguments. Otherwise open the window; if there were input paths, the
  window starts processing them at once (this is the Send-to path).
- **`lib.rs`**: the view-model. `AppState` (Idle, Processing{done,total},
  Done{summary}, Error{message}) and `Event` (FilesDropped, FolderChosen,
  Progress, Finished, Failed). `AppState::handle(&mut self, Event)` is pure
  and fully tested. It owns the remembered output folder and the rule that a
  drop with no output folder is refused with a message rather than guessed.
- **`gui.rs`**: eframe `App` that paints `AppState` and turns egui input
  (dropped files, folder-button click via `rfd`, progress channel) into
  `Event`s. Thin enough to be excluded from line coverage; its states are
  checked headlessly with `egui_kittest` against the design's component
  spec (`docs/wiki/design/components.md`).

Threading: the GUI thread never decodes. `batch::run` is spawned on a worker
thread with an `mpsc::Sender<Progress>`; the paint loop drains the receiver
and calls `ctx.request_repaint()` while processing.

## Data model (outline)

```rust
// cropper-core
pub struct Luma { pub width: u32, pub height: u32, pub data: Vec<u8> }
pub struct Rect { pub x: u32, pub y: u32, pub w: u32, pub h: u32 }   // pixel coords, w,h >= 1
pub struct Tuning { /* table above */ }
pub enum FlagReason {
    Uniform,          // the whole image is one colour; nothing to crop
    NoBorderFound,    // no border and no interior strong line: the screenshot is all art
    LowContent,       // content box < min_content_fraction of the image, or a side < min_content_side
    Ambiguous,        // an edge strip was nearly chrome-like (flat fraction inside ambiguity_band)
}
pub enum CropDecision { Crop(Rect), Flag(FlagReason) }
pub fn decide(img: &Luma, t: &Tuning) -> CropDecision;

// cropper-engine
pub enum SourceFormat { Png, Jpeg, WebP }
pub enum Outcome {
    Cropped { rect: Rect, output: PathBuf },
    Flagged { reason: Flag, output: PathBuf },   // copied unchanged
    Failed  { error: String },                   // nothing written
}
pub enum Flag { Detector(FlagReason), Unsupported, DecodeFailed(String) }
pub struct FileResult { pub input: PathBuf, pub outcome: Outcome }
pub struct RunSummary { pub results: Vec<FileResult> }  // cropped(), flagged(), failed() derived
pub struct Settings { pub output_dir: Option<PathBuf> }
pub struct Invocation { pub inputs: Vec<PathBuf>, pub out_dir: Option<PathBuf>, pub no_gui: bool, pub summary_path: Option<PathBuf> }
```

The summary line the window shows and the JSON the CLI writes are both
derived from `RunSummary`; there is one summary type, not two.

## Where state lives

| State | Where | Lifetime |
|---|---|---|
| Output folder | `settings.json` in the config dir above | forever, until changed in the window |
| Tuning constants | compiled into `cropper-core` (`Tuning::default()`) | per release; no user-facing settings pane in v1 |
| A run's progress and summary | `AppState`, in memory | until the next drop |
| Everything else | nowhere | no database, no cache, no log file, no network |

## Deployment shape

One file: `target/release/manhwa-cropper.exe`, built by `cargo build --release
--workspace`, copied wherever the user likes. No installer in v1. To install
"Send to": the user presses Win+R, runs `shell:sendto`, and creates a shortcut
to the exe there (a one-time, documented step; see decisions). The exe opens
its window, processes the files it was handed, and stays open showing the
summary. Nothing writes outside the chosen output folder and the settings
file.

## Test support

Synthetic fixtures are generated in tests, not stored: a generator under
`crates/core/tests/common/` builds a `Luma` from a recipe (art rect, per-side
border colour and width, optional chrome bands with texture) so every
detector test knows its expected rect exactly, and `proptest` draws recipes at
random for the never-clip property. Engine tests write real PNG/JPEG/WebP
files into `tempfile` directories from the same recipes. The real corpus
(`fixtures/corpus/`, story MC-018) is the only stored fixture set, and it is
the oracle for accuracy, not for unit behaviour.

## Decisions

Each records the alternatives and why they lost. Re-open one only by editing
this file and the stories that depend on it.

1. **Rust, with egui for the window.** Alternatives: C#/.NET WPF or WinUI 3
   (true native widgets, but no verified harness profile, `dotnet` absent on
   the machine, tens of MB self-contained, coverage thresholds need MSBuild
   plumbing); Rust + raw Win32 via `windows-rs` (native look, but drag-drop,
   a folder dialog and a progress bar are hundreds of lines of unsafe code
   that no test can drive). Egui is not native widgets; "native look" in the
   brief is honoured as: native title bar and decorations, the real Windows
   folder dialog through `rfd`, system light/dark followed, and a layout the
   designer keeps to Windows 11 proportions (`docs/wiki/design/`). The
   deciding reason is the harness: `cargo test`, `clippy` and `llvm-cov` are
   a verified profile on Windows and `egui_kittest` tests the window headlessly.
2. **Three crates, not one.** Alternative: one binary crate with modules.
   Lost because the phase lock classifies `src/**` as source, so the only way
   RED can write story-level tests is `tests/`, and `tests/` of a single
   binary crate can only reach a library target; the split also stops egui
   types leaking into the detector.
3. **Detector returns a reason, not a confidence score.** Alternative: a
   float in [0,1] with a threshold. Lost because every threshold on a score is
   a number nobody can defend and the user has no way to act on it; a
   discrete reason is testable and appears in the summary as a word.
4. **Lossy inputs are re-encoded at maximum quality.** The user chose this.
   Alternatives: snap the crop outward to the JPEG MCU grid and cut
   losslessly (correct, but needs a JPEG-domain cropper, none in pure Rust);
   output PNG for JPEG input (changes the format, which AC 5 in the brief
   forbids). JPEG at quality 100 with 4:4:4 subsampling; WebP lossless,
   which is the maximum WebP quality and keeps `image`'s pure-Rust encoder.
5. **Outward margin is a fixed 3 px.** Alternative: proportional to image
   size. Lost because the corpus will decide, and a constant is the easiest
   thing for it to change. A leftover 3 px border is the brief's accepted
   cost.

   **Challenged 2026-09-23 by the user; the value is not yet changed.** *"The
   cropping from the side isnt tight enough and I still see the page on the
   right and left side."* A probe that day found the column locator already on
   the page-background / page boundary on every marked tuning entry, so the
   band the user sees is this margin. [MC-049](../backlog/stories/MC-049.md)
   (`EPIC-08`) proposes `margin_px` = 0 and asks the user first; this decision
   is rewritten when that story lands, not before.
6. **Output folder is remembered; Send-to with no folder ever chosen falls
   back to a `cropped` folder next to the first input.** Alternative: refuse
   and open the window on the picker. Lost because the tenth-run flow in the
   brief is "no interaction needed"; the fallback is created if absent, and
   the window still shows where the files went.
7. **Name collisions get ` (2)`, ` (3)` suffixes.** Explorer's own convention;
   never overwrite. Alternative: overwrite with a warning. Lost: "nothing is
   silently lost or damaged".
8. **Flagged files are copied byte-for-byte, never re-encoded.** A JPEG that
   is flagged is not degraded on its way through.
9. **Send-to shortcut is created by the user.** Alternative: the app writes
   into `shell:sendto` on first run. Lost for v1: it is a one-time
   `shell:sendto` step for a single user, and writing into the profile on
   first launch is the kind of side effect this tool otherwise never has.
   Revisit if a second user ever appears.
10. **Headless mode reports through files and exit codes, not stdout.** The
    exe is built with the Windows GUI subsystem so Send-to never flashes a
    console; stdout may be unattached. `--summary <path>` writes the JSON
    summary; tests and scripts read that. The three exit codes are the
    contract.
11. **No settings pane, no tuning file.** `Tuning` is compiled in. The brief
    says no settings pane; a hidden config file would be a settings pane the
    user cannot see. Corpus tuning happens in code, under a story.
12. **Parallel batch, results in input order.** Rayon `par_iter` over the
    inputs; progress is a count, not a name, so the progress bar never has
    to explain which file is where.
13. **Chrome is peeled from the edges; the box is everything that is not
    chrome.** The brief's wording is "the largest content region delimited by
    strong horizontal and vertical edges". Taken literally, a screenshot of a
    page with three panels separated by strong panel borders would be cut to
    its largest panel - a clip, which the brief ranks above every other
    defect. So the detector removes only strips that touch an edge, are short
    relative to the image and are mostly one flat colour (a tab strip, a URL
    bar, a reader toolbar, a sidebar), and keeps everything else. For the
    common case of chrome plus one panel the two definitions give the same
    rectangle. Alternative kept in reserve: the literal largest-region rule
    with an ambiguity flag when a second large region exists; it loses because
    it clips small panels silently and the flag threshold is a number nobody
    could defend. The user may overturn this by editing this decision and
    MC-005.
14. **v1 crops the column axis accurately and leaves the row axis loose but
    safe.** The user's decision of 2026-09-17, taken after every measurement
    below had been run. On the **column** axis the detector places the page's
    left and right edges inside MC-019's 11 px window on **20 of 21** corpus
    entries. On the **row** axis it places **0 of 21** and overshoots every top
    and bottom edge outward by **97 to 310 px**, *never clipping* (MC-031's
    baseline, reproduced by MC-034's `base` binary). That overshoot is accepted
    as v1's behaviour: a leftover border is the brief's accepted cost
    (decision 5), and a clip is the defect decision 13 ranks above every other.

    Two alternatives, both lost to measurement rather than to argument:

    - **Tune a row rule to the 90% bar.** The stories that measured this axis
      are listed in MC-032 `## Context`, with MC-034 and MC-035 after them.
      Across two rule families and 780 parameterisations the best single rule
      places **8 of 21 and clips 9**, and **no member of either family is
      clip-free at all** — 0 of 780. That holds after admitting the
      gutter-crossing band `corpus.md` records, and again at twice its width
      (`docs/wiki/gutter-band-rescore.md`).
    - **Split the bar by edge**, asserting accuracy on the top edge only, where
      MC-034 reports 20 of 21 *reachable*. Lost because reachability is a
      per-file cherry-pick across the whole sweep that no single rule can make.
      Measured over the same 780 parameterisations: the best single rule places
      **14 of 21** top edges *and clips*; **0 of 780** members avoid clipping a
      top edge; and forgiving `Screenshot (103).jpg`, whose top all 780 clip,
      leaves **9 members placing at most 7 of 20**. There is no top-edge bar
      that is both met and worth having. The numbers are in MC-032
      `## Closed`; they are the only place they are written down.

    Reopening this needs a **signal class v1 excludes**, not a tuning change:
    EPIC-05 rules out learning-based detection by name, and MC-025
    `## Context` ruled out colour and chroma. The user has deferred both to
    **v2**, together with growing the corpus. MC-032 is closed rather than
    parked, and carries the evidence.

    **Amended 2026-09-17, the same day, after the decision's output was looked
    at rather than read about.** Two things this decision said were wrong in a
    way the numbers concealed, and both change what v2 is for.

    - **"Loose but safe" understated it.** The 97–310 px overshoot is not
      whitespace. On a typical entry it is the browser tab bar, the bookmarks
      bar, the site's own navigation menu and the **Windows taskbar**, all
      surviving into the output. The brief's headline promise is "removing
      browser chrome"; vertically it is not delivered. The *safe* half stands
      — zero clips, every entry — and that is why this is a defect to fix in
      v2 rather than a reason to ship something riskier now.
    - **MC-031's Option B was put to the user and rejected.** Its section 9
      offered a weaker but *achievable* criterion — "no output contains
      browser or OS chrome" instead of "within 11 px" — backed by a chrome
      oracle that never clips and lies outside the mark on 19 of 19 entries
      where it speaks. Shown the crop it produces for `Screenshot (67).png`,
      the user's answer was **no: the site's own navigation is also
      unacceptable.** So chrome removal is not a shippable criterion on its
      own, and the target stays the artwork itself. The corpus marks stay
      **tight**, by the same decision.

    This decision therefore stands as the record of what v1 ships and why, and
    **not** as a closure of the row axis. **EPIC-07 reopens it**, under this
    paragraph rather than around it, with the tight marks kept and the corpus
    grown. The structural finding that motivates it — every rule ever tried,
    and the shipped detector, reduce the image to 1D row statistics, which is
    why a single speech bubble defeats them — is in that epic's `## Why now`.

    **Amended 2026-09-21 by the user's decision, and this one moves the
    target.** The row axis is no longer aimed at the artwork's own rectangle.
    The default crop removes **furniture** — browser chrome, the Windows
    taskbar, and the reader site's own header, navigation and footer — and
    stops there. Page gutter above and below the artwork stays in the output,
    as do gutters between panels, overhanging art, sound effects and atypical
    bubbles that cross a gutter. `product-brief.md` section 5 carries the
    user's words and the full statement; this is the decision record.

    Three consequences, because each one reverses something written above.

    - **Option B is adopted after all, in the form that was missing.** The
      paragraph above records it refused on 2026-09-17 because "the site's own
      navigation is also unacceptable". The new target removes the site's
      navigation, so the objection is answered rather than overruled. What was
      refused was chrome removal *alone*; what is adopted is chrome, OS and
      reader furniture together.
    - **"The target stays the artwork itself" no longer holds**, and neither
      does the sentence that every rule ever tried failed on 1D row statistics
      being the thing to fix. It is still true, and it is now *irrelevant*: the
      panel boundary those rules were hunting is no longer being located.
      [MC-038](region-row-search.md) is the last of the seven and the record of
      why hunting it was abandoned.
    - **The corpus marks stay tight, and change job.** They remain exactly as
      marked — nobody re-marks — but they are now the **never-clip oracle**
      rather than the row-axis accuracy target. Accuracy becomes containment
      plus absence of furniture; `product-brief.md`'s amendment states the
      predicate and why re-marking was not chosen.

    Zero clips is untouched by all of this and remains absolute.

    **2026-09-24, MC-048: browser chrome and the taskbar are removed, as an
    internal stage.** The `viewport` stage (pipeline step 4 above) is MC-031's
    chrome oracle shipped inside the crop, which is the only form `EPIC-07`
    permits it in. It clamps the crop's rows to the browser viewport, and the
    outward margin is clamped to that viewport as well, so the margin never
    puts a chrome or taskbar row back. On the 19 marked `tuning` entries where
    `chrome-row-search.md` §4 locates a viewport, the stage reproduces §4 to
    the row on 19 of 19, every crop row lies inside it, the columns do not
    move, and there are still zero clips on all 21 entries.

    **2026-09-24, MC-052: the stage reads only the reader's own window.** As
    MC-048 shipped it, the stage scored each row over every pixel outside the
    page column across the full image width. On split-screen screenshots, a
    second browser window beside the reader has its own chrome and bottom edge,
    so the stage returned that window's viewport and cut the reader's art
    (`2025-03-06 01_22_45.png` by 3 rows at the top and 103 at the bottom,
    `2025-03-07 00_58_06.png` by 31 at the bottom). The first held-out run
    after MC-048 merged found it. The stage now also reads the rows over the
    reader's own window, bounded at its edge on each side, and takes the
    union with MC-048's reading (pipeline step 4). A first version that
    *replaced* the full-width reading was caught at GATES by the
    `detect.rs` property test, which found generated scenes it clipped.
    MC-048's reading did not clip them (MC-052 `## Notes`, "GATES: the
    proptest clip"). The settled
    constants did not move, §4 still reads 19 of 19, the 21 original tuning
    crops did not move by a pixel, and the two files, now `tuning`, crop to the
    reader's viewport (rows 115..1374 and 115..1399) with zero clips.

    - **Named limitation: the two `2025-08-05` WebPs.** At the full margin
      width the oracle finds no page-like run there and declines, so their
      crops are exactly what they were before and still keep the browser
      chrome. No narrower reading was adopted: the one measured stops inside
      the chrome at the settled threshold (MC-048 `## Amendments`). Any
      screenshot whose page background beside the column is not flat enough
      behaves the same way.
    - **The `EPIC-07` bar is not yet met.** The bar is containment of the mark
      plus absence of *all* furniture. This stage delivers the browser and OS
      part only. The reader site's own header, navigation and footer are
      still in the output ([MC-050](../backlog/stories/MC-050.md)), and so is
      the side strip of page background beside the column
      ([MC-049](../backlog/stories/MC-049.md), `EPIC-08`). No held-out score
      of the full predicate exists yet, and none can until MC-050 lands.
