---
id: EPIC-05
title: Proven on the user's real screenshots, at speed
status: todo
stories: [MC-017, MC-018, MC-025, MC-026, MC-027, MC-028, MC-031, MC-019, MC-032, MC-033, MC-034]
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

**Amended 2026-09-17: the nine-in-ten measurement is split by axis, and this
epic can close with only half of it met.** MC-019 carries the **column** axis,
which measures 20 of 21 inside the 11 px window, plus zero clips and
`failed() == 0`. MC-032 carries the **row** axis, which measures 0 of 21, and it
is **parked**: MC-028 and MC-031 closed both pixel-level directions on it
(`docs/wiki/panel-gutter-search.md`, `docs/wiki/chrome-row-search.md`), and the
corpus itself offers only 5 of 21 entries whose marked panel is bounded by a
flat gutter on both sides. Unparking MC-032 needs a signal class outside v1 or a
product decision about what the marks mean - not a tuning change. The brief's
"nine in ten needing no manual fix" is therefore **not** met on the row axis at
the close of this epic, and MC-032 is where that debt is recorded rather than
forgotten. The decision and the orchestrator's own reproduction of the evidence
are in MC-019 `## Amendments`.

## Stories

1. MC-017 - One hundred screenshots crop in under ten seconds
2. MC-018 - Calibration corpus of real screenshots with expected crops (chore; the user supplies the files)
3. MC-025 - Panel edges are found by flatness where the gradient goes blind
4. MC-026 - The decision gates admit a real reader page (tuning; unblocks measurement)
5. MC-027 - The page column is located by its flat page margins (the column axis, 19 of 21)
6. MC-028 - Is a panel gutter locatable from pixels (spike; output is a document)
7. MC-031 - Are the page row edges locatable from full-width chrome (spike; output is a document)
8. MC-019 - The detector meets the accuracy bar on the corpus column axis
9. MC-032 - The detector meets the accuracy bar on the corpus row axis (parked; both pixel-level directions closed by MC-028 and MC-031)
10. MC-033 - The corpus records its marking rule, its diagonal gutters and their tolerance (chore; SCAFFOLD, no production source)
11. MC-034 - Are the page row edges locatable from the panel's own art edge (spike; output is a document, and a reasoned negative is expected)

The order matters and was paid for. MC-026 must come first: until
`min_content_fraction` and `ambiguity_band` admit a real reader page, a perfect
locator crops 2 of 21 and no locator story can be measured at all - which is
what MC-025 discovered the expensive way, by shipping a correct locator that
moved the corpus by zero entries. MC-027 then lands the half of the problem
that is solved. MC-028 asks, before anyone builds anything, whether the other
half is solvable; **a reasoned "no" is a valid outcome** and sends MC-019's
own numbers - the manifest, the 11 px tolerance, or the 90% - back to the user
under `## Amendments`. MC-019 depends on all three and cannot start before
MC-028 has answered.

The two blank regions in a reader screenshot are different things and now have
different names: **page margin** (left and right of the page column, MC-027)
and **panel gutter** (between panels, MC-028). `docs/wiki/architecture.md`,
"Two kinds of blank space", defines both.

## Deliberately not in this epic

Benchmarks on hardware other than the user's machine, per-reader special
cases, any learning or model-based detection, and growing the corpus beyond
what is needed to make the two success numbers measurable.
