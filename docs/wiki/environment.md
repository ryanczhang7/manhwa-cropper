# Environment

How to get a fresh machine from "cannot run the gates" to "runs the gates".
Written for Windows 11 first, because that is the only platform this
product targets (`product-brief.md`, section 6). Last verified on this
machine on 2026-09-10; every "Verify" line below was actually run that day.

The stack is Rust (`stack.md`, profile `rust-cargo`). Everything the gates
need comes from `rustup` plus two `cargo install`s. There is no Node, no
Python, no container.

## Required

| Tool | Version expected | Used by |
|---|---|---|
| `rustup` | any current (1.29 seen) | Installs and pins the toolchain; reads `rust-toolchain.toml` |
| `rustc` / `cargo` | `1.98.1` (pinned by `rust-toolchain.toml` once bootstrap writes it) | Every gate and task: `typecheck`, `unit`, `build`, `dev`, `install` |
| MSVC build tools (linker) | Visual Studio 2022 Build Tools, "Desktop development with C++" | Linking every binary and test. `rustup` will offer to install it |
| `clippy` (rustup component) | ships with the toolchain | `lint` gate |
| `rustfmt` (rustup component) | ships with the toolchain | `format` gate (optional but cheap) |
| `llvm-tools` (rustup component) | ships with the toolchain | `coverage` gate, via `cargo-llvm-cov` |
| `cargo-llvm-cov` | `0.9.1` seen working; any `0.6+` | `coverage` gate |

## Install (Windows)

Run in PowerShell or a terminal, not inside this repo's Git Bash session
(rustup edits PATH and the shell must be reopened afterwards).

```bash
winget install --id Rustlang.Rustup
```

If `rustup` reports that the MSVC prerequisites are missing, let it install
them, or do it first:

```bash
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Then, in a new shell:

```bash
rustup component add clippy rustfmt llvm-tools
```

```bash
cargo install cargo-llvm-cov
```

`cargo install` builds from source; expect a few minutes the first time.
There is no project-local alternative for `cargo-llvm-cov` (cargo has no
per-project tool directory), and it is a developer tool, not a dependency,
so a global install is the right shape.

macOS / Linux, for completeness (nothing here is tested):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

then the same `rustup component add` and `cargo install` lines.

## Verify

Each line, and the shape of a good answer:

```bash
rustup show
```
Lists a `stable-x86_64-pc-windows-msvc` (or `1.98.1-...`) toolchain as
active with target `x86_64-pc-windows-msvc`.

```bash
cargo --version
```
`cargo 1.98.1 (...)`.

```bash
cargo clippy --version
```
`clippy 0.1.98 (...)`. Anything but "no such command" is fine.

```bash
cargo fmt --version
```
`rustfmt 1.x-stable (...)`.

```bash
cargo llvm-cov --version
```
`cargo-llvm-cov 0.9.1` or newer. If it prints a version but the gate later
fails with "llvm-tools not found", run `rustup component add llvm-tools`.

```bash
rustup component list --installed
```
Must include `clippy-`, `rustfmt-` and `llvm-tools-` entries for the
active toolchain.

The whole chain at once: from the repo root, once bootstrap (MC-001) exists,

```bash
bash scripts/doctor.sh
```

```bash
bash scripts/gates.sh --list
```

Before MC-001 there is nothing in `project.conf`, so both print "nothing
configured"; that is expected, not a failure.

## Optional

- **`cargo-mutants`** for `/audit-mutations`. **Installed** here as of
  2026-09-13 (v27.1.0), for the `crates/app` audit recorded in
  `docs/wiki/audits/app-window-2026-09-13.md`.

  ```bash
  cargo install cargo-mutants --locked
  ```

  Verify with `cargo mutants --version`.

  **It is not a gate.** `project.conf` carried a `gate | mutation` line until
  2026-09-13, waived on the grounds that the tool was not installed. Installing
  it made that waiver false, and a waived gate still runs - so the next full
  `gates.sh` would have executed `cargo mutants --workspace`: **466 mutants,
  1-2 hours**, on every story's GATES phase. The gate line was removed instead
  (MC-017 `## Notes`, decided by the product owner). Mutation testing is a
  deliberate audit invoked by `/audit-mutations`, not a per-story gate, which
  is how it was actually used to produce EPIC-06.

  Scope a run rather than taking the workspace default:

  ```bash
  cargo mutants --file 'crates/app/src/*.rs' --output .claude/state/mutants
  ```

  151 mutants over `crates/app` took 20 minutes with a warm cache.

## Notes

- **The toolchain pin.** `stack.md` pins `channel = "1.98.1"` in
  `rust-toolchain.toml`. This machine's default toolchain is `stable`, which
  is 1.98.1 today, but rustup treats `1.98.1` as a distinct toolchain and
  will download it (and needs `clippy`, `rustfmt`, `llvm-tools` for it) the
  first time cargo runs in the repo. Bootstrap should list those three in the
  `components = [...]` key of `rust-toolchain.toml` so rustup installs them
  automatically instead of the lint and coverage gates failing with "no such
  command" on a fresh clone.
- **`link.exe` in Git Bash is not the linker.** `command -v link.exe` in
  this repo's bash resolves to `/usr/bin/link.exe`, the coreutils `link`.
  Cargo does not use PATH to find the MSVC linker; it locates Visual Studio
  through the registry and `vswhere`, so this is harmless. Do not "fix" it by
  putting the MSVC bin directory ahead of `/usr/bin`; that shadows coreutils.
- **llvm-cov prints backslash paths on Windows.** A probe run reported
  `C:\...\src\lib.rs`, not `C:/.../src/lib.rs`. The `--ignore-filename-regex`
  in `stack.md` written with forward slashes will therefore *not* exclude
  `crates/app/src/main.rs` and `gui.rs` here. Bootstrap must use the
  separator-agnostic form the stack file already offers,
  `crates[/\\]app[/\\]src[/\\](main|gui)\.rs`, and prove it by watching the
  coverage number move when the pattern is removed.
- **`cargo fmt --check` fails when it should.** Verified on purpose: an
  unformatted file makes it exit 1 with a coloured diff. That is what makes
  the `format` gate a gate.
- **No Python, no Node.** Nothing in this project needs either. A bare
  `python` on this machine hits the Microsoft Store alias (`rules.md`);
  harness scripts stay in bash, awk and coreutils.
- **`cargo install` output lands in `%USERPROFILE%\.cargo\bin`**, which the
  rustup installer adds to the user PATH. If a new shell cannot find
  `cargo-llvm-cov` after install, that directory is missing from PATH.
- **Build times.** A release build of a workspace pulling `image`, `egui`
  and `rayon` takes minutes cold and seconds warm. `target/` is gitignored
  and classified `ignored` by the phase lock, so it is always writable.
  Measured at MC-001: `cargo test --workspace` cold 3 to 4 minutes (eframe
  and its Windows backends), warm seconds; `cargo build --release` 2 m 07 s
  cold, 71 s warm because fat LTO relinks the exe; `cargo llvm-cov` 5 s warm.
- **Windows Smart App Control must be off.** With it on (Windows 11 Home
  ships it in evaluation or on), Code Integrity blocks unsigned executables
  by per-file cloud reputation, which includes every build script, test
  binary and exe cargo produces: `An Application Control policy has blocked
  this file. (os error 4551)`. The block is per file and unpredictable, so
  builds fail at random dependencies. Check with
  `(Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy').VerifiedAndReputablePolicyState`
  in PowerShell: `0` is off, `1` on, `2` evaluation. Turn it off under
  Settings, Privacy & security, Windows Security, App & browser control,
  Smart App Control settings. Microsoft makes this one-way: it can only be
  re-enabled by a clean install. Found and fixed during MC-001.
- **CI runs the gates on `windows-latest`**, with bash as the step shell,
  for the reasons in `.github/workflows/gates.yml`. The `boundaries` job
  needs only bash and git and stays on ubuntu.
