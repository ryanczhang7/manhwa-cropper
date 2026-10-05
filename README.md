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

**It removes the browser's tab and bookmarks bars, the scrollbar and the
Windows taskbar, and trims the side gutters down to the page.** The top and
bottom were the hard half. v1 left the browser bars and the taskbar in. v2
(EPIC-07) added a stage that finds the browser's viewport beside the page
and drops everything above and below it. When it cannot find the viewport,
it leaves the rows alone rather than guess.

What it guarantees is that it **never cuts into the artwork**. Clipping is
the one defect the whole design is arranged against. Where the detector
cannot be sure, it keeps too much rather than too little, or copies the file
unchanged and flags it.

**How it measures up.** The yardstick is a corpus of 140 real screenshots
(`fixtures/corpus/`), each hand-marked with the box the crop should give, or
flagged as one to leave alone:

- **Tuning, 130.** These are the screenshots the detector is developed
  against: zero clips on every one, and 125 of the 127 that are checked on
  the side edges are within the bar.
- **Held-out, 10.** These are kept aside and scored once each. Every
  held-out set so far has been scored after the fixes before it, and each
  found something new: 3 clips of 25, then 2 of 15, then 1 of 10.
  Every one of those is now fixed, and those screenshots are in tuning. The
  current held-out set (Eleceed, one reader) is not scored yet. That is
  MC-085. `docs/wiki/held-out-score.md` has every score.

**Two browser windows side by side** (a reader on the left, a video on the
right) are handled: the crop takes the reader's page and nothing of the
second window (MC-066, MC-082).

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
  core/     the detector: trim, content box, flat regions, page column,
            browser viewport, margin, decision
  engine/   file I/O, codecs, batch, CLI argument parsing, run summary
  app/      the egui window and the exe
fixtures/
  corpus/   140 real screenshots + a manifest: 126 marked with the right crop,
            14 that should be flagged rather than cropped; 130 tuning, 10 held-out
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

v1 is complete. v2 is EPIC-07, "the crop reaches the artwork on the row
axis", and it is in progress. The browser bars and taskbar are now removed,
every crop on the 130 tuning screenshots is clip-free, and every story
filed from the user's reports is done. The goal stays open until a
held-out score meets the bar: zero clips, and at least 9 in 10 of the
marked screenshots cropped to their mark with no furniture kept. The next
score is MC-085, filed and not yet run.

Still deferred: per-site matching, learning-based detection, and colour and
chroma. See [`docs/wiki/v2-candidates.md`](docs/wiki/v2-candidates.md),
which exists so that nobody re-runs the investigations that are already
answered.
