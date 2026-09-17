# Manhwa Cropper

A small, fast, fully offline Windows app that crops manhwa and manga
screenshots down to the artwork — removing browser chrome, side gutters and
solid borders — without a preview or an approval step.

Drop files on the window, or right-click them in Explorer and use **Send to**.
Every file is written to your output folder with its original name and format.
When the detector is not confident, it **copies the file unchanged and flags
it** rather than risk cutting into the art.

Single user, single machine, no network, no telemetry, no settings pane.

## Running it

The build is one file — `target/release/manhwa-cropper.exe` — with no installer
and nothing to configure. Copy it wherever you like.

```bash
cargo build --release --workspace
```

**The window.** Launch the exe, or:

```bash
bash scripts/task.sh dev
```

Pick an output folder once; it is remembered in `settings.json` in your config
directory. Then drag screenshots onto the drop zone. The window shows progress
and a count of how many were cropped and how many were flagged.

**Send to.** Press <kbd>Win</kbd>+<kbd>R</kbd>, run `shell:sendto`, and put a
shortcut to the exe in the folder that opens. After that: select files in
Explorer, right-click, Send to, Manhwa Cropper. This is a one-time manual step
by design — the app never writes into your profile on first launch.

**Headless**, for scripts:

```bash
manhwa-cropper.exe --no-gui --out <folder> --summary <file.json> <images...>
```

`--out` and `--summary` are optional; a later `--out` wins; everything not
starting with `--` is an input, in order. The exe is built with the Windows GUI
subsystem so Send to never flashes a console, which means **stdout may not be
attached** — the summary file and the exit code are the contract, not printed
output.

| Exit | Means |
|---|---|
| `0` | every input was cropped or deliberately flagged |
| `1` | at least one file could not be written |
| `2` | bad arguments |

Output names never overwrite: a collision gets ` (2)`, ` (3)`, Explorer-style,
and names that differ only by letter case are treated as colliding.

## What the crop looks like today

**The left and right edges are accurate. The top and bottom are barely cropped
at all — read this before you judge the output.**

Measured against a 28-screenshot calibration corpus (`fixtures/corpus/`) — 21
with hand-marked rectangles, 7 expected to be flagged rather than cropped:

| Axis | Result |
|---|---|
| columns (left, right) | **20 of 21** inside an 11 px window |
| rows (top, bottom) | **0 of 21** inside that window; overshoots by **97–310 px**, *never clipping* |

Horizontally the crop is tight: side gutters and the browser's left and right
furniture are gone. **Vertically it is not.** That 97–310 px is not whitespace
— on a typical screenshot it is the browser's tab bar, the bookmarks bar, the
site's own navigation menu and the **Windows taskbar**, all of which survive
into the output. The app does not currently deliver the vertical half of
"removes browser chrome".

What it *does* guarantee is that it never cuts into artwork: zero clips across
every corpus entry. That is the deliberate trade — clipping is the one defect
the whole design is arranged against, so where the detector cannot be sure it
keeps too much rather than too little. Six investigations failed to find a row
rule that is both accurate and clip-free; the reasoning is in
[`docs/wiki/architecture.md`](docs/wiki/architecture.md) decision 14, and the
measurements are indexed from
[`docs/wiki/v2-candidates.md`](docs/wiki/v2-candidates.md).

**Fixing this is v2's whole purpose** — see
[`docs/backlog/epics/EPIC-07.md`](docs/backlog/epics/EPIC-07.md).

Formats: PNG is cropped losslessly at original resolution; JPEG and WebP are
re-encoded at maximum quality (JPEG q100 4:4:4, WebP lossless). A flagged file
is copied byte-for-byte and never re-encoded.

## Building and testing

Rust, three crates, egui for the window. You need a Rust toolchain and
`cargo-llvm-cov` for the coverage gate.

```bash
bash scripts/doctor.sh      # is the toolchain installed and complete?
bash scripts/gates.sh       # format, lint, typecheck, tests, coverage, build
cargo test --workspace      # just the tests
```

```
crates/
  core/     the detector: trim, content box, flat regions, margin, decision
  engine/   file I/O, codecs, batch, CLI argument parsing, run summary
  app/      the egui window and the exe
fixtures/
  corpus/   28 real screenshots + a manifest: 21 marked with the right crop,
            7 that should be flagged rather than cropped
docs/wiki/  brief, stack, architecture, the corpus's marking rules, spike records
```

`docs/wiki/corpus.md` is the source of truth for what the corpus marks mean —
read it before changing a mark or adding an entry.

## How this repo is built

This project is developed with Claude Code under a test-first harness that
enforces the discipline in tooling rather than asking for it in a prompt: a
story cannot reach production code without a failing test that demanded it, and
a phase guard refuses writes that violate the current phase.

That machinery is live and is **not** scrap to be cleaned up — v2 work is
planned and runs through it. It never ships in the binary; `cargo build` sees
only `crates/`.

- [`CLAUDE.md`](CLAUDE.md) — the standing rules, and the loop
- [`.claude/harness/rules.md`](.claude/harness/rules.md) — path ownership and the non-negotiables
- [`.claude/harness/project.conf`](.claude/harness/project.conf) — every build, test and lint command for this project
- `docs/backlog/` — the epics and stories, including the closed ones and why

The harness itself came from `agentic-dev-harness` (a private repository); its
own documentation lives there rather than being duplicated here.

## Status

v1 is complete. Every story in the backlog is done, except MC-032 — the row-axis
accuracy bar — which is **closed by decision** rather than outstanding. Deferred
to v2: a larger corpus and a signal class v1 excludes (learning-based detection;
colour and chroma). See
[`docs/wiki/v2-candidates.md`](docs/wiki/v2-candidates.md), which exists so that
nobody re-runs the six investigations that are already answered.
