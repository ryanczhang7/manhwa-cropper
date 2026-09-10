# Stack

> **Verified by MC-001 on 2026-09-10.** Every gate command below has been run
> through `scripts/gates.sh`, observed to fail on a deliberate break, and
> observed to pass again (`docs/backlog/stories/MC-001.md`, Gate probes).
> What moved between planning and execution, and was corrected here and in
> `.claude/harness/project.conf`:
> - `eframe`/`egui`/`egui_kittest` were researched at `0.32`; they are at
>   `0.36`, whose `App` trait is `fn ui(&mut self, &mut egui::Ui, &mut Frame)`.
>   `rfd` is at `0.17`, not `0.15`.
> - `cargo-llvm-cov` prints backslash paths on Windows, so the coverage
>   gate's ignore regex is the separator-agnostic form, proven load-bearing.
> - The rustup component is `llvm-tools`; `rust-toolchain.toml` lists the
>   components so a fresh clone installs them on first use.
> - The release exe is 10 MB, not "a few MB" (egui + glow + wgpu backends).
> - `integration` and `mutation` carry waivers until MC-017 adds an
>   `#[ignore]` test and `cargo-mutants` is installed.
> - CI's `gates` job runs on `windows-latest` with a toolchain step.
> - Windows Smart App Control had to be turned off (`environment.md`).

Chosen at planning on 2026-09-10 for the constraints in
`product-brief.md`: Windows 11 only, fully offline, a single small fast exe,
100 images in ~10 s, lossless PNG, and a crop engine that must be unit-testable
without a window. Profile: `rust-cargo`
(`.claude/skills/stack-profiles/reference/rust-cargo.md`).

## Language and toolchain

| Item | Pinned | Why (brief constraint) |
|---|---|---|
| Rust | `1.98.1` via `rust-toolchain.toml` (`channel = "1.98.1"`, `components = ["clippy", "rustfmt", "llvm-tools"]`, edition 2024) | Compiled native exe, no runtime to install, starts instantly, offline by construction; already installed here. |
| cargo | ships with the toolchain; workspace `resolver = "3"` | Build, test, dependency lock; every gate is a cargo subcommand scriptable from bash. |
| `clippy`, `rustfmt` | rustup components, listed in `rust-toolchain.toml` | Linter and formatter gates. |
| `llvm-tools` | rustup component (the `-preview` name is an alias), listed in `rust-toolchain.toml` | Required by `cargo llvm-cov`. |
| `cargo-llvm-cov` | `0.9.1` installed and verified 2026-09-10 (`cargo install cargo-llvm-cov`) | Coverage without nightly; see `environment.md` for the Windows path-separator caveat on `--ignore-filename-regex`. |
| `cargo-mutants` | optional, latest `25.x` | The optional `mutation` gate; not required for v1. |

Why not C#/.NET (WPF/WinUI): the harness has no verified profile for it,
`dotnet` is not installed here, a self-contained WPF publish is tens of MB
against the "single small exe" recommendation, and enforcing a coverage
threshold needs extra MSBuild plumbing. Why not raw Win32 through
`windows-rs`: drag-and-drop, a folder dialog and a progress bar in raw Win32 is
more unsafe code than the whole crop engine, and none of it is testable.
Alternatives are recorded with the decision in `architecture.md`.

## Crates

Lines confirmed with `cargo search <crate> --limit 1` at MC-001 (2026-09-10);
the "latest seen" column is what the registry answered that day. Every pin
lives in the root `[workspace.dependencies]`; member crates say
`workspace = true`. Only the crates MC-001 uses are resolved in `Cargo.lock`
so far (`eframe`, `egui_kittest`); each later story adds its crate to the
member manifest in GREEN, which resolves and locks it then. Every crate below
is pure Rust (no C toolchain beyond the MSVC linker rustup already needs),
which keeps the build reproducible and offline.

| Crate | Pinned line | Latest seen | Used by | Why |
|---|---|---|---|---|
| `image` | `0.25` (features `png`, `jpeg`, `webp`, no defaults) | `0.25.10` | engine | Decodes PNG, JPEG and WebP; encodes PNG losslessly, JPEG with a quality parameter (set to 100) and WebP losslessly. Pure Rust decoders satisfy "offline, no native deps". MC-009 must verify this version encodes WebP (the `image-webp` backend). |
| `rayon` | `1` | `1.12.0` | engine | A batch of 100 files runs across cores; single-threaded 4K PNG decoding alone would threaten the 10 s budget. |
| `serde` + `serde_json` | `1` | `1.0.229` / `1.0.151` | engine | Settings file (remembered output folder) and the machine-readable run summary. |
| `directories` | `6` | `6.0.0` | engine | Locates `%APPDATA%` for the settings file without hand-rolling Windows paths. |
| `eframe` / `egui` | `0.36` | `0.36.2` | app | The one small window: drop zone, folder picker, progress, summary. Follows the system light/dark theme, uses the native title bar, single exe, no WebView. `egui` is reached as `eframe::egui`; no separate dependency. |
| `egui_kittest` | `0.36` (dev) | `0.36.2` | app | Headless egui harness so the window's states are tested without a display. Proven at MC-001: `Harness::new_ui(gui::paint)` + `get_by_label`. |
| `rfd` | `0.17` | `0.17.2` | app | Native Windows folder picker (`IFileDialog`) - the one control that must look like Windows because it *is* Windows. |
| `tempfile` | `3` (dev) | `3.27.0` | all | Temp directories for file I/O tests. |
| `proptest` | `1` (dev) | `1.11.0` | core | The never-clip invariant is a property over generated fixtures, not an example. |

Deliberately not used: `clap` (three flags; a 40-line parser in the engine
crate is testable and saves a dependency), `imageproc`/`opencv` (row and column
gradient profiles are a few dozen lines and avoid a native build), `tokio`
(nothing is async; rayon covers the parallelism), `windows-rs` (no Win32 calls
in v1; the Send-to shortcut is created by the user, see `architecture.md`).

## Layout

    Cargo.toml                 virtual workspace; [workspace.dependencies] carries every pin
    Cargo.lock                 committed (binaries)
    rust-toolchain.toml        channel = "1.98.1"
    crates/core/               cropper-core   : pixels in, rectangle or flag out. No I/O, no egui.
      src/lib.rs
      tests/                   story-level tests; tests/common/ holds the synthetic fixture generator
    crates/engine/             cropper-engine : codecs, naming, batch runner, summary, settings, CLI args
      src/lib.rs
      tests/                   file-level tests against temp dirs; perf test (#[ignore], run by the integration gate)
    crates/app/                manhwa-cropper : the exe. main.rs (entry, arg dispatch) + gui.rs (egui shell) + lib.rs (view-model)
      src/main.rs, src/gui.rs, src/lib.rs
      tests/                   egui_kittest state tests; end-to-end CLI test that spawns the built exe
    fixtures/corpus/           the user's real screenshots + manifest.toml (story MC-018; classified `test`)

Unit tests inside `src/**/*.rs` under `#[cfg(test)]` are classified `source` by
the phase lock and are therefore an implementation detail written in GREEN.
**Story-level tests live in `tests/`**, per the profile's recommendation.

## Gates

Copied from the profile with this project's layout and threshold applied,
and transcribed into `.claude/harness/project.conf` by MC-001 after each one
was run and seen to fail. `project.conf` is the authority; this is the
explanation.

    gate | format      | optional | . | cargo fmt --all --check
    gate | lint        | required | . | cargo clippy --workspace --all-targets -- -D warnings
    gate | typecheck   | required | . | cargo check --workspace --all-targets
    gate | unit        | required | . | cargo test --workspace
    gate | coverage    | required | . | cargo llvm-cov --workspace --fail-under-lines 95 --ignore-filename-regex 'crates[/\\]app[/\\]src[/\\](main|gui)\.rs'
    gate | integration | optional | . | cargo test --workspace --release -- --ignored
    gate | build       | required | . | cargo build --release --workspace
    gate | mutation    | optional | . | cargo mutants --workspace

    task | install | - | . | cargo fetch
    task | dev     | - | . | cargo run -p manhwa-cropper
    task | test    | - | . | cargo test --workspace

- **Test runner**: `cargo test --workspace`. `--workspace` is mandatory: a bare
  `cargo test` at the root of a workspace with a root package tests only that
  package (see the profile). The virtual layout removes the root package, and
  `--workspace` stays anyway.
- **Coverage tool**: `cargo llvm-cov`, threshold in the command. 95 lines
  rather than the profile's 100 because the engine crate has OS-error branches
  (permission denied on write, disk full) that cannot be provoked portably.
  `crates/app/src/main.rs` and `gui.rs` are excluded because a process entry
  point and an immediate-mode paint function are exercised by the end-to-end
  and kittest tests, not measured by line coverage; everything with logic in
  the app crate lives in `crates/app/src/lib.rs` or the engine crate, where it
  is measured. `cargo-llvm-cov` 0.9.1 prints **backslash** paths on Windows
  (`app\src\gui.rs`), so the pattern is the separator-agnostic
  `crates[/\\]app[/\\]src[/\\](main|gui)\.rs`. It is load-bearing: without
  it MC-001's scaffold measured 42.86% lines (`main.rs` at 0%), with it
  100%. Note that the shell wrapper some tools use collapses `\\` to `\`;
  write this line with a tool that does not (MC-001 Notes).
- **Linter**: `cargo clippy --workspace --all-targets -- -D warnings`.
- **Type checker**: the Rust compiler, via `cargo check --workspace --all-targets`.
  There is no separate type checker in this ecosystem; `check` type-checks
  every target including tests without codegen.
- **Formatter**: `cargo fmt --all --check`, optional.
- **Build**: `cargo build --release --workspace` produces
  `target/release/manhwa-cropper.exe`.
- **Integration**: the `#[ignore]`d tests - the 100-image performance test and
  the corpus accuracy test - in release mode. Optional for the repo; stories
  whose criteria live there set `required_gates: [integration]`.

### Evidence of work

    evidence | unit        | test result: ok\. [1-9][0-9]*
    evidence | lint        | Finished .* profile
    evidence | typecheck   | Finished .* profile
    evidence | build       | Finished .* profile
    evidence | integration | test result: ok\. [1-9][0-9]*
    evidence | format      | -
    evidence | coverage    | TOTAL          # verified: the table ends in a TOTAL row
    evidence | mutation    | [1-9][0-9]* mutants tested   # unverified: cargo-mutants not installed; waived

    waiver | mutation    | cargo-mutants is not installed; install it and delete the line
    waiver | integration | no #[ignore] test exists until MC-017; MC-017 deletes the line

The `unit` regex is widened from the profile's `[1-9]` so a `floor` can read the
whole count. `cargo fmt --check` prints nothing when everything is formatted and
a diff otherwise; it is honest about having work and gets `-`. The three
`Finished` regexes are weak by design (warm-cache runs print nothing else);
`--workspace` and the `unit` gate carry the real protection.

### Floor, discovery, slow

    floor     | unit   | 1            # observed 1 at MC-001 (first `test result: ok. N` line is crates/core/tests/scaffold.rs); raise per story
    discovery | core   | . | cargo test --workspace --no-run 2>&1 | grep -q "cropper_core"
    discovery | engine | . | cargo test --workspace --no-run 2>&1 | grep -q "cropper_engine"
    discovery | app    | . | cargo test --workspace --no-run 2>&1 | grep -q "manhwa_cropper"

    slow | build       | release build of the workspace; minutes, and RED has no use for it
    slow | integration | release rebuild plus a 100-image timing run and the corpus; minutes
    slow | mutation    | cargo mutants re-runs the suite once per mutant

`cargo test --no-run` prints an `Executable ... (target\debug\deps\<crate>-<hash>.exe)`
line per test binary on every run, warm or cold, which is why the discovery
greps the crate name in its underscore form rather than a `Compiling` line.
Confirmed at MC-001; `bash scripts/doctor.sh` reports all three discovered.

`coverage` stays in `--fast` on purpose: it is the command that judges the
tests, and it is the slowest fast gate in this stack.

## Prerequisites (for `/setup-environment`)

    winget install --id Rustlang.Rustup          # if rustup is absent; it is present here
    cargo --version                              # in the repo: rustup reads rust-toolchain.toml and installs 1.98.1 + components
    cargo install cargo-llvm-cov
    cargo install cargo-mutants                  # optional

Verify: `cargo --version`, `cargo clippy --version`, `cargo fmt --version`,
`cargo llvm-cov --version`. The MSVC build tools are required by rustup on
Windows; `rustc 1.98.1` being present means they already are. Windows Smart
App Control must be off or cargo's build scripts are blocked at random; see
`environment.md`.

## Release shape

`cargo build --release --workspace` emits one exe. `[profile.release]` sets
`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`,
`strip = true`; measured at MC-001: 10.2 MB with eframe's default `glow` and
`wgpu` backends compiled in, 2 m 07 s cold and 71 s warm to link. Trimming
eframe's features is a later story if size matters. `#![windows_subsystem = "windows"]` on
the binary so no console window opens on launch or on Send-to. Because of
that, the headless CLI mode never relies on stdout for its result: the exit
code and the files it writes are the contract (`architecture.md`, CLI entry).
