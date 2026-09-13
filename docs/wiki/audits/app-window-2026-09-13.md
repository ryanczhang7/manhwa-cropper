# Mutation audit: `crates/app`, the window

Commissioned by `docs/backlog/epics/EPIC-04.md`, "## Follow-up, agreed
2026-09-13", which asked for the window's **look-only rules** to be examined
first once MC-016 closed the epic.

## Scope

Audited: every mutant `cargo-mutants` generates for the four files of
`crates/app/src/` — `lib.rs` (the view-model, MC-014), `gui.rs` (the painter,
MC-015), `shell.rs` (the wiring, MC-016) and `main.rs` (the process entry).

At commit **`f2440c921128618402e879576a2feeb25333790b`**, working tree clean,
branch `story/MC-016-the-window-wires-folder-picker-drops-and`.

Deliberately left out:

- **`crates/core` and `crates/engine` were not audited at all.** This pass was
  scoped to the window by the commissioning note. Nothing in this document says
  anything about the detector, the batch runner, the settings file or the
  argument parser, and no reader should infer that their tests are in any
  particular state.
- Mutants of test files, of `build.rs` and of dependencies: `cargo-mutants`
  does not generate them and none were asked for.
- The `mutation` gate as configured (`cargo mutants --workspace`) was **not**
  run. It covers all three crates and was out of scope; the scoped invocation
  below is what ran.

## Decided

These are settled. A story citing this audit implements them and does not
reopen them. Each names the measurement it rests on.

1. **The view-model needs nothing.** `lib.rs` killed every viable mutant
   (evidence E-2). No story should be filed against `crates/app/src/lib.rs`
   on the strength of this audit, and `tests/model.rs` is the pattern the rest
   of the crate should be judged against.

2. **The look-only survivors in `gui.rs` can only be killed by image snapshot
   tests, not by AccessKit assertions.** This confirms EPIC-04's premise rather
   than merely restating it: the AccessKit tree carries text and roles, so a
   fill, a stroke, a text colour or a vertical offset changes nothing any
   existing query can see (evidence E-3, E-4). An AccessKit assertion cannot be
   written that kills these, and a story that proposes one is wrong before it
   starts. The renderer feature this needs is real and present in the pinned
   dependency (evidence E-7), but has never been built here.

3. **The progress bar's fill fraction does not belong in the painter.**
   `gui.rs:516` and `gui.rs:519` compute `done / total` inside `progress_bar`,
   where the only way to observe it is a rendered image. Every other number the
   window prints already lives in `lib.rs` precisely so it can be compared as a
   value (`lib.rs` module docs, "Why every string lives here"). The fraction is
   the one that escaped. Move it, and it is killable by a unit test with no
   renderer at all. This is a decision about *where the boundary sits*, and it
   follows from the same reasoning MC-014 already applied — not from the
   mutation counts.

4. **Three survivor groups are accepted gaps, not defects, and no story is
   filed for them.** `RfdPicker::pick` (`gui.rs:687`, 2 mutants) opens a modal
   OS dialog; `main::open_window` (`main.rs:92`, 1 mutant) blocks on a real
   native window. Neither body can execute under any headless test, which
   `gui.rs`'s own module comment already states and which is why both live in
   coverage-excluded files. Recording them as survivors is correct; filing work
   against them is not.

5. **`CropperApp::ui` (`gui.rs:723`, 1 mutant) stays unpinned, knowingly.**
   Replacing its body with `()` makes the window paint nothing at all, and no
   test catches it — the most severe single survivor in the run by what a user
   would see. It is four lines of delegation to `Shell::frame`, which is
   thoroughly tested (evidence E-5), and reaching it requires an
   `eframe::CreationContext`, i.e. a real window. The cost of closing it is out
   of proportion to a four-line delegation. Stated here so it is a decision
   rather than an oversight.

6. **`main.rs:93 delete field viewport` is a real, cheaply-closable gap that
   this audit does not file.** With the field deleted the exe opens an
   untitled, default-sized window: MC-015's AC-8 pins `gui::viewport()` as a
   value but nothing pins that `open_window` passes it to eframe. It is
   killable headlessly by extracting the `NativeOptions` construction into a
   pure function. It is named here rather than filed because it is one line in
   a file with no other work pending; a Lead PO may promote it.

## Evidence

Not settled. A story depending on any number below verifies it first.

### E-1 — the run itself

**Claim:** `cargo-mutants` generated 151 mutants for the four files of
`crates/app/src/`, of which 76 survived, 61 were caught, 14 did not compile and
0 timed out.

**Inputs:** the four files `crates/app/src/{lib,gui,shell,main}.rs` at commit
`f2440c921128618402e879576a2feeb25333790b`. No other file in the workspace was
mutated. Rust 1.98.1 (`x86_64-pc-windows-msvc`), cargo-mutants 27.1.0, egui /
eframe / egui_kittest 0.36.

**Tool and command**, run from the repository root:

```
cargo mutants --file 'crates/app/src/*.rs' --output .claude/state/mutants
```

Final line of its output:

```
151 mutants tested in 20m: 76 missed, 61 caught, 14 unviable
```

Full lists are in `.claude/state/mutants/mutants.out/{missed,caught,unviable,timeout}.txt`
— machine-local, gitignored, and deleted by any `cargo clean`-adjacent tidy-up.
Re-run the command above to regenerate.

**What would make it unrepresentative:** the counts are a property of
cargo-mutants 27.1.0's mutation operators, not of the code. A different version
generates a different set, so "76 of 151" is not comparable across upgrades —
compare the *named* survivors instead. The run used the default timeout
multiplier; 0 timeouts means nothing was near it on this machine, and slower
hardware could convert a caught mutant into a timeout.

### E-2 — the view-model is fully pinned; the painter is not

**Claim:** of the mutants that compiled, `lib.rs` killed 31 of 31,
`shell.rs` 11 of 13, `main.rs` 3 of 5 and `gui.rs` **16 of 88**.

**Inputs:** the same single run as E-1. Derived by:

```
for f in lib gui shell main; do
  echo "$f caught=$(grep -c "src/$f.rs" .claude/state/mutants/mutants.out/caught.txt)" \
       "missed=$(grep -c "src/$f.rs" .claude/state/mutants/mutants.out/missed.txt)" \
       "unviable=$(grep -c "src/$f.rs" .claude/state/mutants/mutants.out/unviable.txt)"
done
```

which printed:

```
lib.rs: caught=31 missed=0 unviable=3
gui.rs: caught=16 missed=72 unviable=8
shell.rs: caught=11 missed=2 unviable=3
main.rs: caught=3 missed=2 unviable=0
```

**What would make it unrepresentative:** the per-file rate is heavily skewed by
`rounded_rect_outline`, which alone contributes 30 of `gui.rs`'s 72 survivors
(E-4) for one visual feature. A story reading "18% of `gui.rs`" as "82% of the
painter is untested behaviour" has over-read it: the ratio counts mutation
sites, not rules in `components.md`. `lib.rs`'s 100% is the more robust number,
because its 31 mutants are spread across 11 distinct functions.

### E-3 — the look-only survivors that change what a user sees

**Claim:** these survivors each alter a colour, a border or a position that
`docs/wiki/design/components.md` specifies, and no test notices. Quoted
verbatim from `.claude/state/mutants/mutants.out/missed.txt`:

```
crates/app/src/gui.rs:269:5: replace widgets -> Widgets with Default::default()
crates/app/src/gui.rs:428:16: delete ! in folder_button
crates/app/src/gui.rs:376:46: replace / with % in drop_zone
crates/app/src/gui.rs:376:46: replace / with * in drop_zone
crates/app/src/gui.rs:377:44: replace - with + in drop_zone
crates/app/src/gui.rs:377:44: replace - with / in drop_zone
crates/app/src/gui.rs:377:59: replace / with % in drop_zone
crates/app/src/gui.rs:377:59: replace / with * in drop_zone
crates/app/src/gui.rs:481:59: replace + with - in status_slot
crates/app/src/gui.rs:481:59: replace + with * in status_slot
crates/app/src/gui.rs:481:45: replace + with - in status_slot
crates/app/src/gui.rs:481:45: replace + with * in status_slot
crates/app/src/gui.rs:482:55: replace / with % in status_slot
crates/app/src/gui.rs:482:55: replace / with * in status_slot
crates/app/src/gui.rs:482:44: replace - with + in status_slot
crates/app/src/gui.rs:482:44: replace - with / in status_slot
crates/app/src/gui.rs:505:43: replace / with % in centred_line
crates/app/src/gui.rs:505:43: replace / with * in centred_line
crates/app/src/gui.rs:505:32: replace - with + in centred_line
crates/app/src/gui.rs:505:32: replace - with / in centred_line
crates/app/src/gui.rs:567:43: replace - with + in flagged_row
crates/app/src/gui.rs:567:43: replace - with / in flagged_row
crates/app/src/gui.rs:582:5: replace format -> TextFormat with Default::default()
crates/app/src/gui.rs:583:9: delete field font_id from struct TextFormat expression in format
crates/app/src/gui.rs:584:9: delete field color from struct TextFormat expression in format
```

The two worth singling out, because they are the ones a user would notice
first:

- `gui.rs:428 delete !` turns `if !enabled {` into `if enabled {`. The mutant's
  own diff, from `.claude/state/mutants/mutants.out/diff/crates__app__src__gui.rs_line_428_col_16.diff`:

  ```
  -            if !enabled {
  +            if  /* ~ changed by cargo-mutants ~ */enabled {
  ```

  The block it guards sets `override_text_color = Some(palette.text_disabled)`
  among other things, so with it inverted the **enabled** "Choose folder…"
  button paints its caption in disabled grey and the **disabled** one loses the
  three tokens `components.md` gives it. All 15 `tests/gui.rs` tests and all 11
  `tests/shell.rs` tests stay green, because `add_enabled(enabled, ..)` — the
  part AccessKit can see — is untouched.

- `gui.rs:269 replace widgets -> Widgets with Default::default()` discards the
  whole widget palette for both themes: every button fill, boundary and text
  colour reverts to egui's own. `the_window_follows_the_system_theme` survives
  it because that test reads `visuals.panel_fill` and `visuals.window_fill`,
  which `visuals()` sets directly and which the mutant does not touch.

**Tool and command:** as E-1. The `if !enabled` reading was confirmed against
the stored diff, not inferred.

**What would make it unrepresentative:** each of these is *survival*, which is
established. Whether each is *worth killing* is a design judgement this audit
does not make — the six `drop_zone` and `status_slot` arithmetic survivors
shift things by a few points, whereas `gui.rs:269` and `gui.rs:428` change
colours wholesale. A story that treats them as one priority will over-invest.

### E-4 — the dashed drop-zone border is 30 survivors for one visual feature

**Claim:** `rounded_rect_outline` (`gui.rs:638`–`662`) contributes 30 surviving
mutants, including `replace rounded_rect_outline -> Vec<Pos2> with vec![]`,
which removes the drop zone's border entirely.

**Inputs:** as E-1. Counted with
`grep -c 'rounded_rect_outline' .claude/state/mutants/mutants.out/missed.txt`
→ `30`. First and last of them, verbatim:

```
crates/app/src/gui.rs:638:5: replace rounded_rect_outline -> Vec<Pos2> with vec![]
crates/app/src/gui.rs:662:32: replace + with - in rounded_rect_outline
```

**What would make it unrepresentative:** at least one of the 30 is arguably
**equivalent at the only call site that exists**. `gui.rs:642:68 replace / with
*` widens `radius.clamp(0.0, min(w,h) / 2.0)` to `min(w,h) * 2.0`; the drop
zone is 120 px tall and the radius asked for is 7.5, so the clamp is inactive
either way and the painted output is byte-identical. A snapshot test cannot
kill that one, and a story that treats "30 survivors" as "30 tests to write"
has mis-scoped itself. This audit did **not** work through the other 29 for
equivalence — that inspection is left to the story.

### E-5 — the wiring is well pinned except for one conversion

**Claim:** `shell.rs` has exactly two survivors, both of `fn count`:

```
crates/app/src/shell.rs:269:5: replace count -> u32 with 0
crates/app/src/shell.rs:269:5: replace count -> u32 with 1
```

`count` is the `usize` → `u32` conversion used **only** by `ThreadRunner`, and
`tests/shell.rs`'s one real-runner test,
`a_real_run_of_two_screenshots_writes_both_crops_and_reports_them`, polls until
the model leaves `Processing` and then asserts the final summary. It never
asserts on a `Progress` event, so nothing observes a converted count. With
`count -> 0` a real run of any size reports `0 of 0` under a bar that never
fills, all the way to the summary.

Every other `shell.rs` mutant was caught, including
`shell.rs:213:13 delete ! in read_files` (the hover flag) and
`shell.rs:170/181/188 replace {drain,dispatch,perform} with ()`.

**Inputs and command:** as E-1; the survivor list filtered with
`grep 'shell.rs' .claude/state/mutants/mutants.out/missed.txt`. The test's
contents read from `crates/app/tests/shell.rs:1035`–`1088` at the same commit.

**What would make it unrepresentative:** the two `count` mutants are only
reachable through `ThreadRunner`, so a story that tests `count` as a pure
function kills them without pinning the wiring that made them matter. The
useful test is one that observes a `Progress` from a real run — which needs
enough input files that the run cannot finish inside one poll interval, and is
therefore timing-sensitive in a way this audit did not measure.

### E-6 — the truncation tests are theatre

**Claim:** eleven survivors in `text_width`, `truncate_left` and
`truncate_right` — including replacing each function's whole body with a
constant — because the only assertion over them checks the accessible name,
which `gui.rs` re-states independently of what the truncation returned.

```
crates/app/src/gui.rs:593:5: replace text_width -> f32 with 0.0
crates/app/src/gui.rs:593:5: replace text_width -> f32 with 1.0
crates/app/src/gui.rs:593:5: replace text_width -> f32 with -1.0
crates/app/src/gui.rs:602:5: replace truncate_left -> String with String::new()
crates/app/src/gui.rs:602:5: replace truncate_left -> String with "xyzzy".into()
crates/app/src/gui.rs:603:36: replace <= with > in truncate_left
crates/app/src/gui.rs:608:46: replace <= with > in truncate_left
crates/app/src/gui.rs:618:5: replace truncate_right -> String with String::new()
crates/app/src/gui.rs:618:5: replace truncate_right -> String with "xyzzy".into()
crates/app/src/gui.rs:619:36: replace <= with > in truncate_right
crates/app/src/gui.rs:625:46: replace <= with > in truncate_right
```

The mechanism: `path_label` paints the truncated string and then calls
`response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, .., full))`,
overwriting the node's value with the **full** path.
`a_long_output_path_keeps_its_whole_path_as_the_accessible_name`
(`crates/app/tests/gui.rs:356`) asserts exactly one node is named with the full
path. That assertion is true whatever `truncate_left` returns — including the
empty string. `flagged_row` does the same for rows, which is why
`truncate_right` survives identically.

The test is honest about what it *says* it checks (the accessible name, which
matters for a screen reader). The point is that it is the only test near the
truncation, so the truncation itself is pinned by nothing.

**Inputs and command:** as E-1, plus reading `crates/app/src/gui.rs:464`–`472`
and `crates/app/tests/gui.rs:356`–`367` at the same commit.

**What would make it unrepresentative:** this cluster is asserted here to be
killable *without* a renderer, because the truncated string is a `String` a
test could compare directly. That has **not been demonstrated** — no such test
was written. The functions currently take `&Ui` and are private, so making them
observable is a real change of shape and might be judged not worth it, in which
case this cluster falls back into the snapshot story. Verify before sizing.

### E-7 — the renderer feature the snapshot story needs exists at the pinned version

**Claim:** `egui_kittest` 0.36.2, the exact version this workspace resolves,
declares both a `snapshot` and a `wgpu` feature.

**Inputs:** `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui_kittest-0.36.2/Cargo.toml`,
`[features]` section, lines 56–76, read directly:

```
snapshot = ["dep:dify", "dep:image", "dep:open", "dep:tempfile", "image/png"]
wgpu = ["dep:egui-wgpu", "dep:pollster", "dep:image", "dep:wgpu", "eframe?/wgpu"]
```

`Cargo.toml` line 36 pins `egui_kittest = "0.36"`; `crates/app/Cargo.toml`
line 34 takes it from the workspace with no features enabled.

**What would make it unrepresentative:** the features being *declared* is not
the same as them *working on this machine or on CI*. No snapshot test was
built, compiled or run during this audit. Whether a wgpu adapter is available
under `cargo test` on the reference Windows machine, whether a software adapter
is needed, and how much build time the feature adds are all unmeasured. A story
that budgets on "the feature exists" is budgeting on E-7 alone, which is thin.

## What would have to be true for this to be wrong

- `cargo-mutants` 27.1.0 applied every mutant it listed and reported truthfully;
  no mutant was silently skipped after the baseline build succeeded.
- The suite is deterministic. A flaky test could be recorded as "caught" for a
  mutant it did not actually detect, inflating the caught count.
- The suite was run with the same features and profile for every mutant as for
  the baseline, so a "caught" result means the mutation and not a build
  difference.
- AccessKit genuinely carries no colour or geometry at egui 0.36. Every "only a
  snapshot can kill this" conclusion rests on that; it is consistent with all 72
  `gui.rs` survivors but was not proved from the AccessKit schema.
- `crates/app/tests/window.rs`, `cli.rs` and `headless.rs` add no coverage of
  the painter beyond what is credited here. They were listed but not read in
  full.
- The single `rounded_rect_outline` equivalence argued in E-4 is the only one;
  the remaining 29 were assumed non-equivalent without checking.
- 20 minutes of wall clock is the run's true cost on this machine and not an
  artefact of a warm build cache. The cache was warm.

## What was not checked

- **`crates/core` and `crates/engine` — not audited.** Two of the three crates
  in the workspace, including every line of the detector and the batch runner,
  were outside this pass entirely. Their mutation state is unknown.
- **The configured `mutation` gate was not exercised.** `project.conf` line 161
  runs `cargo mutants --workspace`; this audit ran a `--file`-scoped invocation
  instead. Whether the workspace-wide gate completes in reasonable time, or at
  all, is unmeasured — and it is still marked `optional`.
- **No snapshot test was written, built or run.** E-7 reads a manifest. The
  whole of Decided-2 rests on the *absence* of colour in AccessKit plus the
  *presence* of a feature flag; neither end was demonstrated by running code.
- **Equivalence was checked for exactly one mutant** (`gui.rs:642:68`). The
  other 75 survivors are reported as survivors, which they are, without a
  claim that each is a genuine defect.
- **`tests/window.rs` (22 lines), `tests/cli.rs` (317) and `tests/headless.rs`
  (636) were skimmed by test name only.** If any of them pins something
  credited above as unpinned, this audit is wrong about that item — though the
  mutation run is the stronger evidence, since those suites ran against every
  mutant too.
- **One platform, one toolchain, one run.** Windows 11, Rust 1.98.1 MSVC, a
  single execution with no repetition. No Linux, no CI hardware, no second run
  to check stability of the caught/missed split.
- **Timing sensitivity of the `count` fix (E-5) was not measured.** How many
  input files a real run needs before a `Progress` is reliably observable is
  unknown, and it is the crux of MC-024's feasibility.
- **The `slow` classification was not re-examined.** The mutation gate is
  marked `slow`; 20 minutes for one crate is consistent with that, but the
  workspace figure that justifies it was not measured.

## Spike code

None. This audit produced no code. `.claude/state/mutants/` holds
cargo-mutants' own output — machine-local, gitignored, and not committed. It is
a record of the run, not code for a later story to draw on; regenerate it with
the command in E-1 rather than trusting a stale copy.

## Stories filed

- **MC-021** — The window's painted look is pinned by image snapshots. From
  E-3 and E-4: ~55 look-only survivors in `gui.rs` that only a rendered image
  can kill (Decided-2).
- **MC-022** — The path label and the flagged row show the text they truncated.
  From E-6: eleven survivors behind an assertion that reads the accessible
  name, which is re-stated independently of the truncation.
- **MC-023** — The progress bar's fill fraction is the view-model's number.
  From E-3 (`gui.rs:516`, `gui.rs:519`) and Decided-3.
- **MC-024** — A real run reports its progress counts. From E-5: both `count`
  survivors in `shell.rs`.

Not filed, and why: `gui.rs:687` and `main.rs:92` (Decided-4, unreachable by
construction), `gui.rs:723` (Decided-5, accepted), `main.rs:93` (Decided-6,
cheap but unowned — a Lead PO may promote it).

All four live in **EPIC-06**, "The window's look and its numbers are pinned by
tests", created by the Lead PO on 2026-09-13 after this audit reported. They
were filed against EPIC-04 — the epic that commissioned the audit — but
EPIC-04's goal was delivered and closed by MC-016, so a new epic carries them
rather than reopening a finished one. EPIC-06 also carries Decided-6 forward
under "Carried over, unfiled".
