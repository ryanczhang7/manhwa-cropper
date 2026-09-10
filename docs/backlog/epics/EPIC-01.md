---
id: EPIC-01
title: The toolchain stands and one file goes in and comes out
status: todo
stories: [MC-001, MC-002]
---

## Goal

The repository is a real Rust workspace whose gates all run and have each been
seen to fail, and the exe can take one PNG path on the command line and write
that file, unchanged, into an output folder. Nothing is cropped yet; the whole
path from Explorer to disk exists end to end.

## Why now

Every later story depends on `project.conf` being true (`stack.md` is
researched, not verified) and on the crate layout that keeps the detector
headless. The walking skeleton exercises decode-free file I/O, argument
parsing and the exit-code contract before any algorithm exists, so the
algorithm stories never have to discover an I/O problem.

## Done when

`bash scripts/gates.sh` passes with `BOOTSTRAPPED=yes`, and
`manhwa-cropper.exe --no-gui --out D:\out C:\shots\a.png` produces
`D:\out\a.png` byte-identical to the input and exits 0.

## Stories

1. MC-001 - Rust workspace with every gate green on a walking skeleton (bootstrap)
2. MC-002 - CLI copies one PNG unchanged into the output folder

## Deliberately not in this epic

Any cropping, any format other than passing bytes through, the window, the
settings file, name collisions, and installing the Send-to shortcut.
