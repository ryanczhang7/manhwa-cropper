---
name: story-authoring
description: How to write epics and stories for this harness - sizing a story to one RED to GREEN cycle, phrasing acceptance criteria as testable behaviour, and choosing story types. Use when planning a product, adding a story to the backlog, or judging whether a story is ready to start.
---

# Writing epics and stories

## Create stories with the script

    bash scripts/new-story.sh WORLD-014 "Regions render with distinct biome colours" EPIC-03 feature

It writes the canonical file with the frontmatter the harness depends on. Do not
hand-roll story files; `scripts/check-boundaries.sh` validates the frontmatter
in CI and the hooks read it.

## Sizing

A story is one RED to GREEN cycle: one behaviour, testable in isolation,
typically a handful of files. Signs it is too big:

- the acceptance criteria describe two features joined by "and"
- you cannot name the tests before writing them
- it touches the data model, the API and the UI at once
- you would want to commit halfway through

Split by behaviour, not by layer. "Backend for regions" and "frontend for
regions" is a split that produces two stories neither of which can be
demonstrated. "Regions persist across reload" and "Regions render with distinct
colours" is a split that produces two demonstrable behaviours.

The rule above is framed around RED→GREEN. The bootstrap story runs under
SCAFFOLD instead, where nothing forces a test to exist, so over-sizing it is
both easier and more expensive: keep it to the toolchain and the gates, and
put every project-specific piece in a story of its own afterwards. See
`reference/bootstrap-story.md`, Sizing.

Order matters as well as size. `depends_on` in the frontmatter is enforced:
`phase.sh set` refuses to start a story while a dependency is not DONE. Use it
whenever a spike decides something a later story builds on.

## When the evidence lives in an optional gate

Before leaving PLANNED, read the acceptance criteria against the gate list. If
an AC can only be verified by a gate `project.conf` marks `optional` - a
browser-driven `integration` suite, most often - then `bash scripts/gates.sh`
can come back green with that criterion unverified, and nothing notices. Say so
in the frontmatter:

    required_gates: [integration]

That gate is then binding for this story and optional for every other. See the
`quality-gates` skill.

Do this for every story, not only the obviously browser-bound ones: **name,
before RED, the required gate that would fail if the story's artifact broke.**
If the answer is "none" or "an optional one", this is the moment to fix it. The
trap is not one wrong decision but three right ones adding up: a test project
that needs a real browser (correct - jsdom has no WebGL, and testing a renderer
there is theatre), an `integration` gate marked `optional` because it needs
that browser (correct), a coverage `include` that skips the same directory
(correct). Together they put all twenty-five tests of a renderer where nothing
could block on them, and `All required gates passed` was printed over a story
whose artifact no required gate had exercised. `optional` means "this
gate reports information"; it must never be the only thing testing a shipped
feature. `gates.sh` cannot warn you, because nothing in `project.conf` says
which paths a gate exercises - so the story says it.

## Brief RED by oracle

Not every criterion is the same kind of claim, and a Test Developer told "you
have no oracle, design the metric" across *all* of them will re-derive values a
design decision already fixed - the audit problem arriving from the other
direction. Before RED, sort the criteria and say which is which:

| Kind | Instruction to RED |
|---|---|
| **Settled** - a design doc, an audit's `## Decided`, a validated constant stands behind it | *Read the numbers out. Do not derive, tune or "calibrate" them.* |
| **Oracle-free** - perceptual, statistical, "reads as continents" | *Invent the metric, and demand a negative control that fires hard.* |
| **Mechanical** - an API shape, an event, instrumentation | *Precision beats invention; pin it exactly, leave nothing open-ended.* |

This partition is the part of a RED brief that has been measured to matter.
Two stories ran RED with it, one on a stronger model and one on the default,
with the default-model claim written down first so it could come out either
way. The default-model run produced the sharper controls - 69x separation
against 10x - and a second control for a criterion whose first metric could be
defeated by per-triangle flat shading. One run each, so the honest reading is
"the brief did the work, not the model"; and the brief is the cheap variable.
Put the partition in the story, under `## Model guidance` or beside the
criteria, so that the orchestrator's dispatch prompt carries it.

## When the story depends on an audit or a spike

A story that follows an earlier audit says so, and says it precisely. "Read the
audit before starting and follow its recommendation; do not re-litigate it" is
the right instruction about the audit's `## Decided` section and the wrong one
about its `## Evidence`. Spell out both halves:

- the **decision** is settled - name it, and say the story implements it;
- the **evidence** is not - name any number the story is about to depend on
  (a threshold, a tolerance, a claim that two things agree) and say that it is
  to be verified, not assumed.

This is not pedantry. An audit here recommended an approach and supported it
with a claim of bit-identical output measured on three lucky seeds; the
underlying assumption fails for 58% of inputs. The recommendation was fine. A
story that had trusted the number would have shipped the bug.

The same goes for spike code. If a story is expected to draw on a throwaway
spike, say what it may take - the algorithm, the shape - and what it must
re-derive, in one sentence rather than two paragraphs apart. Spike code is
unreviewed by definition; "throwaway" and "reuse its algorithm" cancel out if
the story does not resolve them.

## Acceptance criteria

Each one is a behaviour observable from outside the code, phrased so that a test
either passes or fails against it. Number them AC-1, AC-2 - the Test Developer
cites them and the PR body maps tests to them.

Good:

- **AC-1** - Given a world with three regions, when the map is rendered, then
  each region is filled with the colour registered for its biome.
- **AC-2** - Given a region whose biome is unknown, when the map is rendered,
  then it is filled with the neutral placeholder colour and a warning names the
  region.

Not criteria:

- "The map looks good." Not observable.
- "Refactor the renderer." No behaviour; that is a chore.
- "Use a quadtree." An implementation choice - if it matters it belongs in the
  architecture doc, and if it does not, let the Feature Developer choose.

Performance and accessibility criteria are welcome, with numbers: "renders a
10,000-tile world in under 100ms on the reference machine", "every control is
reachable by keyboard in visual order".

### A criterion that names a measurement names a control too

When an AC names a **statistic, a metric or a threshold**, it must arrive with a
**negative control**: a deliberately broken input the metric is required to
reject, and roughly what it should score. If you cannot name one, the criterion
is not ready to leave PLANNED.

Two different questions hide here, and only the first is usually asked:

- *Is 25% the right threshold?* — a control on the **number**.
- *Is variance the right quantity?* — a control on the **choice of metric**,
  one level up. This is the one that had a wrong answer.

An AC read as: "height **variance** in a polar band and an equatorial band of
equal area are within 25% of each other — no smearing at the poles". Impeccably
testable, reviewed, approved, and blind. Variance is a **one-point** statistic,
and the marginal distribution of a stationary noise field does not change when
the domain is stretched; "smearing" is a **two-point** property, about how fast
the field varies with distance. Measured on 400,000 points, the AC's own metric
separated a correct field from the exact defect it names by **9%**, against a
25% threshold — which side of the line you land on is decided by sampling noise.
The two-point statistic that replaced it separated them by **10x**.

Every agent downstream would have implemented that faithfully, and the test
would have passed on broken code.

So write the control into the story next to the criterion:

> **AC-5** — a polar band and an equal-area equatorial band have mean squared
> gradients within 25% of each other.
> *Control:* the same field sampled in the lon/lat plane instead of on the
> sphere — the defect this AC exists to catch — must fail this check by a wide
> margin (measured ~10x).

**A criterion that is precise, measurable and blind is more dangerous than a
vague one.** A vague criterion gets challenged in review; a blind one survives
review and produces a green tick over a broken implementation.

If the Test Developer reports that a criterion's metric cannot detect what the
criterion is about, that is an AC change: it stops, the product owner decides,
and the change is recorded under `## Amendments` — after the orchestrator has
reproduced the finding independently, on its own inputs.

## Types

| Type | When | Phase path |
|---|---|---|
| `feature` | new behaviour | PLANNED, RED, GREEN, GATES, REVIEW, DONE |
| `fix` | a bug; criteria reproduce it | same - the failing test becomes the regression test |
| `chore` | tooling or migrations with no behaviour change | may use SCAFFOLD |
| `bootstrap` | turns the repo into the chosen stack | SCAFFOLD, GATES, REVIEW, DONE |
| `spike` | a timeboxed investigation; output is a document | PLANNED, REVIEW, DONE |

A bug is never fixed without a test that reproduces it first. That test is the
whole value of the fix.

## More

- `reference/sections.md` - what belongs in each section of a story file
- `reference/bootstrap-story.md` - how to write the first story of a project
- `reference/epics.md` - epic format and ordering
