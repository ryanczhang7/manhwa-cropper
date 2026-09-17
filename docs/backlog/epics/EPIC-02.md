---
id: EPIC-02
title: One screenshot is cropped to its artwork, or honestly refused
status: done
stories: [MC-003, MC-004, MC-005, MC-006, MC-007]
---

## Goal

Given the pixels of one screenshot, the core decides where the artwork is:
solid gutters are trimmed, browser and reader chrome outside the panel's
strong edges is removed, the rectangle is pushed outward a few pixels so art
is never cut, and when the detector cannot be confident it says so with a
reason instead of guessing. All of this on pixels in memory, with no file, no
window, and synthetic fixtures whose true art rectangle is known exactly.

## Why now

This is the product. Everything else is plumbing around
`cropper_core::decide`. Doing it first, headless and against generated
fixtures, means the algorithm's correctness is established before the user's
real corpus arrives (EPIC-05), and the corpus then only has to tune constants.

## Done when

`decide(&luma, &Tuning::default())` returns the art rectangle (plus margin)
for every synthetic recipe the generator can produce - solid borders of any
colour and width per side, chrome bands above/below/beside - and returns a
`FlagReason` for a uniform image, an all-art image, a mostly-blank image and
an ambiguous one. The never-clip property holds under `proptest`.

## Stories

1. MC-003 - Uniform solid borders are trimmed on all four sides
2. MC-004 - Row and column edge profiles locate strong lines
3. MC-005 - Chrome strips outside the strong edges are peeled away
4. MC-006 - Crop rectangle expands outward and never clips art
5. MC-007 - Uncertain images are flagged with a reason

## Deliberately not in this epic

Reading or writing any image file, colour handling beyond luminance, tuning
the constants against real screenshots (EPIC-05), panel splitting, and
anything that runs on a thread other than the caller's.
