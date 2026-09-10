# The sections of a story file

Written by different agents at different times. Each has one owner.

## Frontmatter - Lead PO

`id`, `title`, `slug`, `epic`, `type`, `status`, `phase`, `branch`,
`depends_on`, `required_gates`. Only `scripts/phase.sh` should change `phase`,
`status` and `branch` once the story is in flight.

`required_gates` names gates that are optional for the repo but binding for
this story - use it when an acceptance criterion is only ever verified by one
of them. `gates.sh` enforces it while the story is active and
`check-boundaries.sh` refuses a PR whose recorded run has no PASS for it.

## Context - Lead PO

Why this story exists and what constrains it. Link the epic and the wiki pages
that bound the solution. Two or three sentences; the Test Developer reads this
first and needs orientation, not history.

## Acceptance criteria - Lead PO

Numbered, observable, testable. See the main skill file.

## Amendments - Lead PO, with the user

Acceptance criteria are frozen once the story leaves PLANNED. If one is wrong or
unsatisfiable, the story stops, the product owner decides, and the change is
recorded here: which AC, what it said, what it says now, who approved it and
why. `check-boundaries.sh` fails a PR whose criteria differ from the base branch
without an entry. Omit when unused.

## Model guidance - Lead PO

Optional, and written *before* the phase it applies to. Use it when one phase of
a story is worth running on a model other than the default: which phase, which
model, why that phase specifically, what the orchestrator stays on, and how to
brief it differently - a model chosen for judgement is given the criteria and
the constraints, not a pre-decided test design.

End it with a success condition that could come out either way, and record the
**verdict** against that condition when the phase ends, with evidence. The
verdict is the part that gets skipped, and a model choice with no verdict is
folklore: nobody can argue with it and nobody can undo it.

Carry the **oracle partition** here too - which criteria are settled and to be
read out, which are oracle-free and want an invented metric with a negative
control, which are mechanical and want exact pinning (main skill file, "Brief
RED by oracle"). It is the part of the brief the recorded verdict points at:
the same partitioned brief on the default model out-performed a stronger model
without it, so the section earns its place even when the model never changes.

## Out of scope - Lead PO

The explicit non-goals. This is how the Feature Developer knows where to stop,
and it is often the most valuable section in the file.

## Design notes - Lead Designer

For user-facing stories: components, states, tokens, breakpoints, accessibility
requirements. Concrete enough to implement without a second conversation. Omit
entirely for headless work rather than writing "n/a".

## Test plan - Test Developer

Which tests, at which level, and which acceptance criterion each covers. Written
during RED, before or as the tests are written.

## Handoff: RED -> GREEN - Test Developer

The only channel to the Feature Developer. See the `tdd-cycle` skill,
`reference/handoff.md`.

## Regressions - Test Developer, with the Lead PO

Required for any story that returned to RED after GREEN or GATES; omitted
otherwise. One block per return: which test, what was wrong with it, how that
was found, what it asserts now, and what earns the correction.

That last part is the whole section. On a return, "watched it fail" cannot apply
- the implementation exists and is often correct, so the corrected assertion is
green the moment it is written, and green on every run after, whether or not it
asserts anything. It earns its place the same way a green-on-arrival test does:
a **probe** (mutate the specific behaviour it pins, paste the red, confirm the
revert), or - where the defect was cost rather than correctness, a test too slow
for its timeout - a **before and after measurement taken under the gate
command**, which is the instrumented one, not the plain test command.

Paste the output. `check-boundaries.sh` refuses a PR whose `## Regressions` or
`## Gate probes` describes a failure without showing one, because a description
of red is the one thing an agent that skipped the probe would also write.

Record too whether GREEN was a no-op, with the output proving the source was
untouched and still passes. A no-op GREEN is a legitimate outcome that the
orchestrator verifies rather than delegates; dispatching an implementer with
nothing to do invites them to find some.

## Gate results - scripts/gates.sh, nobody else

Written by the script itself on every full run: a marker line, the UTC time,
the commit, a hash of the source/test/config content the gates ran against, and
the summary. Not pasted, not edited. `check-boundaries.sh` refuses a PR whose
section lacks the marker or whose recorded hash does not match the code being
merged - so a story that changes code after its last gate run has to run the
gates again, which is the point.

## Gate probes - Feature Developer, or whoever scaffolds

Required for any story that adds or changes a gate, its command, or its
`evidence` line; omitted entirely otherwise. For each such gate: what was broken
to make it fail, the failure output, and confirmation the probe was reverted.
The output, not an account of it - `check-boundaries.sh` checks for a pasted
block here for the same reason it does in `## Regressions`.

This is RED applied to the gates. Without it a story can add a gate that has
never been seen to do anything, and every story afterwards inherits it as proof.
See the `quality-gates` skill.

## Scaffold inventory - whoever scaffolds

Required for a `bootstrap` or `chore` story that writes production code under
SCAFFOLD, and for a `spike` that commits its throwaway code; omitted otherwise. One line per production file written, and for
anything with behaviour rather than configuration, the test that covers it.
SCAFFOLD is the one phase where nothing forces a test to exist first, and it is
the phase every later story's correctness rests on; this is where the scaffolder
shows the tests were written anyway. `check-boundaries.sh` refuses the PR if a
changed source file is not named here. Whether the named test is adequate is a
reviewer's judgement, not the script's.

## Notes - anyone

Decisions taken mid-story, surprises, things deliberately deferred, the result
of an orchestrator's own mutation run. A return to RED is recorded in
`## Regressions`, not here.
