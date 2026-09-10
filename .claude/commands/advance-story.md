---
description: Move one story forward by exactly one phase
argument-hint: <story-id>
allowed-tools: Bash(bash scripts/phase.sh:*), Bash(bash scripts/gates.sh:*), Bash(bash scripts/check-boundaries.sh:*), Bash(bash scripts/task.sh:*), Bash(git:*), Read, Grep, Glob, Edit, Write, Task
---

Story: $1

Current harness state:
!`bash scripts/phase.sh show`

Act as the **lead-po** orchestrator and advance this story by **one phase only**,
then stop and report. The user chose this command over `/complete-story`
because they want to inspect the result before the next phase runs.

Read `docs/backlog/stories/$1.md` first. Then, based on its current phase:

**PLANNED → RED.** Confirm the acceptance criteria are testable; fix them with
the user if they are not — this is the last phase in which they may change
without an `## Amendments` entry. Then name the gate that would fail if this
story's artifact broke, and check it is `required`. Read the criteria against
`bash scripts/gates.sh --list`: if the only gate that exercises what the story
builds is `optional` — a browser-driven `integration` suite, most often — put it
in the story's `required_gates` now. That is a PO decision made here, not a
discovery for GATES. Three individually sound exclusions (a test project that
needs a real browser, an `optional` integration gate because it needs one, a
coverage `include` that skips the same directory) once combined so that every
test of a renderer ran where nothing could block on it, and `All required gates
passed` was printed over a story whose artifact no required gate had touched.
If no gate at all can verify the artifact, the story is not ready.

Create and switch to the story's branch
(`story/<id>-<slug>`) if it does not exist. Set the phase; `phase.sh` refuses
if a `depends_on` story is not DONE or the checkout is on another branch, and
either refusal is a reason to stop and tell the user, not to reach for
`--force`. Then dispatch the
**test-developer** subagent with the story path, the criteria restated in full
and **partitioned by oracle** — which carry a settled number to read out, which
are oracle-free and need an invented metric with a negative control, which are
mechanical and want exact pinning (see `story-authoring`, "Brief RED by
oracle") — the relevant constraints from `docs/wiki/`, and the exact test
command from `.claude/harness/project.conf`. When it returns, verify: read the test files it
wrote and run the tests yourself. Confirm they fail, and fail for the right
reason. If they pass, or fail on an unrelated error, send it back.

Then run `bash scripts/gates.sh --fast` and read it before leaving RED. This is
not a pass/fail check — the test gates are *supposed* to be red here. It asks a
different question: are these tests **admissible** to the gates that will judge
them? Expect lint and typecheck to pass, and the test gates to fail with the
assertion the story is about. A test gate failing for any other reason — a
timeout, a coverage threshold, a config error, a lint rule the test file trips —
means the tests are not admissible yet and RED is not finished. Note especially
that the coverage gate runs the same tests *instrumented*, which is slower than
the plain test command and slower again on CI hardware; a test that only just
fits its timeout here does not fit there.

**RED → GREEN.** Check the `## Handoff: RED -> GREEN` section is filled in; if
it is not, the RED phase is not finished. Where the story has negative controls
— the deliberately broken inputs a threshold has to reject — the handoff must
carry their **expected values**, not just the fact that they exist: in RED the
suite failed at import, so no assertion in the file ran and every control is an
unverified claim. Set the phase, then dispatch the **feature-developer** subagent
with the story path, the handoff, and the test command, and tell it to confirm
each recorded control value against the shipped module. When it returns, run the
tests yourself, then `bash scripts/gates.sh --fast` — the same admissibility
question, now expecting green.

**GREEN → GATES.** Set the phase and run `bash scripts/gates.sh`. It writes
its own summary into the story's `## Gate results`; never paste or edit one.
On failure, dispatch the **feature-developer** to fix it, unless the failure
means a test is wrong — in which case return the story to RED (see below). A
`WARN` on an optional gate is read, not skipped; a known permanent failure gets
a `waiver` line with its reason.

**GATES → REVIEW.** Only when every required gate passes. If this story added or
changed a gate, `## Gate probes` must record it having been observed to fail —
a gate nobody has seen fail is not evidence of anything, and refusing to move on
without it is the point.

Then, **in this order**:

1. `bash scripts/phase.sh set $1 REVIEW`
2. Commit, with a message that names the story and what it does.
3. `bash scripts/check-boundaries.sh` — the second script CI runs, and the one
   `gates.sh` cannot stand in for. Fix anything it reports before pushing.
4. Push the branch and open a PR whose body links the story file and lists the
   acceptance criteria with the test that covers each.

The phase is set **before** the commit, and the order is not cosmetic:
`check-boundaries.sh` reads the phase out of the *committed* story frontmatter,
so a commit made while the story still says `phase: GATES` is a commit CI
rejects. Committing first happens to survive when the PR is opened before that
job runs, which makes it fail intermittently rather than every time — the worse
of the two. Do not reorder these to be helpful.

**REVIEW → DONE.** Only once the PR is merged. Set the phase to DONE, clear the
lock with `bash scripts/phase.sh clear`, and report what the next story is.

**Returning to RED from GREEN or GATES.** A test that is wrong sends the story
back to RED; it is never edited into passing. On arrival RED means something
narrower than it did the first time, because the implementation already exists
and may well be correct:

- Set the phase back to RED and say in `## Regressions` what is wrong with the
  test, how it was found, and what it should assert instead.
- The **test-developer**'s remit is the defective test and nothing else. Source
  is frozen again by the lock, which is correct — do not treat that as a signal
  to change phase.
- "Watch it fail" cannot apply, because the code whose absence would make it
  fail is no longer absent — so it is replaced, not waived. The corrected
  assertion passes on its first execution and every one after, whether or not it
  asserts anything, and one of these two is required before the phase ends:
  **probe** it by mutating the *specific* production behaviour the test claims
  to pin, watching that one assertion go red, and reverting (this is
  `## Gate probes` applied to a test); or, where the defect was about *cost*
  rather than correctness — a test too slow for its timeout under
  instrumentation — record the before and after measurement under the gate
  command, not the plain test command. The **output** goes in `## Regressions`,
  not a description of it: `check-boundaries.sh` refuses a PR whose
  `## Regressions` or `## Gate probes` shows none.
- GREEN may then be a genuine no-op: the source is untouched and already passes.
  Verify that yourself by running the suite and the fast gates. Do **not**
  dispatch the feature-developer with nothing to do — an agent given no work
  will find some.

Rules for you as orchestrator:

- Change phase only with `bash scripts/phase.sh set $1 <PHASE>`.
- Verify every subagent claim against the filesystem and a real command run. A
  report of success is a claim.
- **A claim that the contract itself is wrong — an acceptance criterion, a
  threshold, a frozen test — you reproduce independently before accepting it:
  different inputs, and without reusing the subagent's own code.** Running its
  probe again is not verification. Record the reproduction in the story next to
  the `## Amendments` or `## Regressions` entry it justifies. This is the exact
  shape of claim an agent makes when it wants to stop failing — *the
  specification is wrong, not my work* — and it is also the shape of the two
  most valuable escalations this harness has seen. Independent reproduction is
  the only thing that separates them, and it is cheap: a fresh probe on
  different inputs, or reading the fixture and enumerating the cases the file
  asserts to show no rule satisfies all of them.
- **When the claim is "this suite discriminates", the check is a mutation you
  run.** A handoff's mutation table — *changing X fails 9 tests, changing Y
  fails 1* — could not be verified in RED, where the suite did not load, and
  is easy to write. Against the committed implementation, pick a mutation the
  table predicts a count for, preferring one whose predicted catch is a
  **single** assertion (a lone assertion is where a vacuous test hides), run
  the suite, compare the count, restore the file byte-for-byte and confirm the
  suite is green again. Two mutations, one run each, is enough; matching
  counts turn the table from a claim into evidence. Record it in `## Notes`.
  The failure screenshots a runner writes under an ignored directory while you
  do this are not a code change, and the Stop hook knows it.
- Keep the story file current as you go — it is the only thing the next agent
  will see.
- Stop and ask the user on any product ambiguity. Do not invent scope. A
  subagent that escalates a scope question instead of resolving it quietly has
  done the right thing; answer it rather than sending it back.

Finish by reporting: the phase you moved from and to, what changed, the real
command output that justifies it, and the exact command to run next.
