---
id: EPIC-05
title: Proven on the user's real screenshots, at speed
status: todo
stories: [MC-017, MC-018, MC-025, MC-019]
---

## Goal

The success criteria in the brief become measurements: 100 screenshots in
under 10 seconds, zero clipped panels across a corpus of the user's own
screenshots, and at least nine in ten needing no manual fix. The corpus is
supplied by the user with hand-marked expected rectangles; the detector's
constants are tuned against it, in the open, under `## Amendments` where a
settled number changes.

## Why now

Synthetic fixtures prove the algorithm does what it is designed to do; only
real screenshots prove the design is right. The corpus cannot be invented by
an agent, so it is a story of its own that the user completes, and the
accuracy story is blocked on it deliberately.

## Done when

The `integration` gate runs the timing test and the corpus test in release
mode and both pass; the corpus manifest names at least twenty screenshots
covering the cases in the brief's open question 1; the accuracy test fails
if any corpus image is clipped by a single pixel.

## Stories

1. MC-017 - One hundred screenshots crop in under ten seconds
2. MC-018 - Calibration corpus of real screenshots with expected crops (chore; the user supplies the files)
3. MC-019 - The detector meets the accuracy bar on the corpus

## Deliberately not in this epic

Benchmarks on hardware other than the user's machine, per-reader special
cases, any learning or model-based detection, and growing the corpus beyond
what is needed to make the two success numbers measurable.
