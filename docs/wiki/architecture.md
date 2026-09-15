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
4. **Outward margin** (`margin`): expand by `Tuning.margin_px` on every side,
   clamped to the image. "Never clip" is the property that the final rect
   contains the art rect on every fixture the generator can produce.
5. **Decision** (`decide`): `CropDecision::Crop(rect)` or
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
