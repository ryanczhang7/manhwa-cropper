#!/usr/bin/env bash
# Tests for scripts/check-boundaries.sh - the half of CI that judges the
# COMMIT rather than the code.
#
# It reads a story file as it was committed and as it stands on the base
# branch, so every case here is a real two-branch repository: a base commit, a
# story branch, and a diff between them.

. "$(dirname "${BASH_SOURCE[0]}")/_lib.sh"

FIX="$(make_project_fixture)"
trap 'rm -rf "$FIX"' EXIT

git -C "$FIX" -c user.email=t@t -c user.name=t branch -M main >/dev/null 2>&1

# The fixture's own branch decides which story is under test, so the CI
# variables that override that have to be cleared - otherwise this suite reads
# the story id out of the branch of whatever PR is running it, finds no story,
# and exits before reaching a single one of the checks below. It passes locally
# and fails on a runner, which is the failure mode the harness spends the rest
# of its documentation warning about. PR_HEAD_SHA goes for the same reason: it
# would recompute the gate hash at a commit in the real repository.
boundaries() {
  ( cd "$FIX" && GITHUB_HEAD_REF= PR_HEAD_SHA= bash scripts/check-boundaries.sh main 2>&1 )
}
commit_all() { git -C "$FIX" add -A >/dev/null 2>&1
               git -C "$FIX" -c user.email=t@t -c user.name=t commit -qm "${1:-wip}" >/dev/null 2>&1; }

# story_on_branch [type] [extra-frontmatter]   Writes docs/backlog/stories/T-1.md
# on a fresh story branch cut from main, with the body on stdin appended after
# the frontmatter. The type defaults to `feature` and the extra frontmatter to
# nothing, so a caller that wants the ordinary story passes neither; the two
# arguments exist for the spike cases below, which differ from every other
# fixture here only in `type:` and `required_gates:`.
story_on_branch() {
  local type="${1:-feature}" extra="${2:-}"
  git -C "$FIX" checkout -q main 2>/dev/null
  git -C "$FIX" branch -D story/T-1-fixture >/dev/null 2>&1
  git -C "$FIX" checkout -q -b story/T-1-fixture 2>/dev/null
  mkdir -p "$FIX/docs/backlog/stories"
  {
    printf -- '---\nid: T-1\ntitle: Fixture story\nslug: fixture\ntype: %s\nstatus: todo\nphase: REVIEW\nbranch: story/T-1-fixture\n' "$type"
    [ -n "$extra" ] && printf -- '%s\n' "$extra"
    printf -- '---\n\n'
    printf -- '## Acceptance criteria\n\n- **AC-1** - it works.\n\n## Handoff: RED -> GREEN\n\nthe command, the failure, the export shape.\n\n'
    cat
  } > "$FIX/docs/backlog/stories/T-1.md"
  commit_all "story T-1"
}

# ---------------------------------------------------------------------------
describe "a return to RED has to show the red"

# The first non-negotiable is a property of an assertion, not of a phase. On a
# corrective RED the implementation already exists, so the corrected assertion
# passes on its first execution and passes forever unless somebody deliberately
# breaks what it pins. Prose saying that happened is not evidence that it did.
story_on_branch <<'EOF'
## Regressions

The seaFloorM test asserted a RangeError the rule does not require. Corrected
to a floor below sea level, and it passes now.
EOF
out="$(boundaries)"
assert_contains "described but not shown" "## Regressions describes something without showing it" "$out"

story_on_branch <<'EOF'
## Regressions

The seaFloorM test asserted a RangeError the rule does not require. Corrected
to a floor below sea level. Probed by mutating the guard to compare against
zero, which is the bug the test names:

```
 x compares seaFloorM against the document's sea level, not against zero
 Tests  1 failed | 27 passed (28)
```

Reverted; `git diff` clean.
EOF
out="$(boundaries)"
assert_contains "shown in a fence" "ok    ## Regressions carries pasted output" "$out"

story_on_branch <<'EOF'
## Regressions

Corrected the AC-4 helper, which was too slow for the coverage gate. Before and
after, both under `bash scripts/gates.sh --gate coverage`:

    AC-4 property test   4,275 ms  ->  367 ms

Thresholds, seeds and numRuns untouched; the measured statistics are identical.
EOF
out="$(boundaries)"
assert_contains "shown as an indented measurement" "ok    ## Regressions carries pasted output" "$out"

# The section is optional. A story that never returned to RED omits it, and
# the template's own commented-out block is not a claim about anything.
story_on_branch <<'EOF'
## Notes

One clean cycle.
EOF
out="$(boundaries)"
case "$out" in
  *"## Regressions"*) _bad "an absent section is not a failure" "complained anyway: $out" ;;
  *) _ok "an absent section is not a failure" ;;
esac

story_on_branch <<'EOF'
## Regressions

<!-- REQUIRED if this story ever returned to RED after GREEN or GATES; omit
     otherwise. One block per return:
       * which test, what it asserted, and what was wrong with it
       * what earns it, since "watched it fail" cannot apply once the
         implementation exists -->
EOF
out="$(boundaries)"
case "$out" in
  *"## Regressions"*) _bad "an untouched template block is not a claim" "complained anyway: $out" ;;
  *) _ok "an untouched template block is not a claim" ;;
esac

# ---------------------------------------------------------------------------
describe "the story comes from GITHUB_HEAD_REF where CI sets it"

# On a pull_request the checkout is a detached merge commit, so the branch name
# says "HEAD" and names no story; CI passes the real one in GITHUB_HEAD_REF.
# Pinned here because this suite was written without it, inherited the variable
# from the runner, and silently checked nothing at all.
story_on_branch <<'EOF'
## Regressions

Described, not shown.
EOF
head_sha="$(git -C "$FIX" rev-parse HEAD)"
git -C "$FIX" checkout -q --detach "$head_sha" 2>/dev/null
out="$( cd "$FIX" && GITHUB_HEAD_REF=story/T-1-fixture PR_HEAD_SHA= bash scripts/check-boundaries.sh main 2>&1 )"
assert_contains "a detached checkout still finds the story" "story T-1 is in REVIEW" "$out"
git -C "$FIX" checkout -q story/T-1-fixture 2>/dev/null

# ---------------------------------------------------------------------------
describe "a PR is opened from REVIEW, as committed"

# This is why /advance-story sets the phase BEFORE committing. phase.sh set
# rewrites the frontmatter; this script reads the frontmatter back out of the
# commit; so a commit made while the story still said GATES carries GATES to
# CI no matter what the working tree says afterwards. Committing first passed
# whenever the PR happened to be opened before this job ran, which is worse
# than always failing: it taught one project that the order was cosmetic.
git -C "$FIX" checkout -q main 2>/dev/null
git -C "$FIX" branch -D story/T-1-fixture >/dev/null 2>&1
git -C "$FIX" checkout -q -b story/T-1-fixture 2>/dev/null
mkdir -p "$FIX/docs/backlog/stories"
printf -- '---\nid: T-1\ntitle: Fixture story\nslug: fixture\ntype: feature\nstatus: todo\nphase: GATES\nbranch: story/T-1-fixture\n---\n\n## Acceptance criteria\n\n- **AC-1** - it works.\n\n## Handoff: RED -> GREEN\n\nthe command, the failure, the export shape.\n' \
  > "$FIX/docs/backlog/stories/T-1.md"
commit_all "T-1 committed before the phase was set"
out="$(boundaries)"
assert_contains "a commit carrying GATES is refused" "a PR should be opened from REVIEW or DONE" "$out"

# ---------------------------------------------------------------------------
describe "the same rule covers gate probes"

story_on_branch <<'EOF'
## Gate probes

Broke the import boundary and the lint gate failed, as expected. Reverted.
EOF
out="$(boundaries)"
assert_contains "a gate probe described but not shown" "## Gate probes describes something without showing it" "$out"

# ---------------------------------------------------------------------------
describe "acceptance criteria are frozen"

# The anchor case: this is what 3d exists for, and it also proves the suite is
# reading the base branch rather than the working tree.
git -C "$FIX" checkout -q main 2>/dev/null
mkdir -p "$FIX/docs/backlog/stories"
printf -- '---\nid: T-1\ntitle: Fixture story\nslug: fixture\ntype: feature\nstatus: todo\nphase: PLANNED\nbranch: story/T-1-fixture\n---\n\n## Acceptance criteria\n\n- **AC-1** - it works.\n' \
  > "$FIX/docs/backlog/stories/T-1.md"
commit_all "T-1 planned"

git -C "$FIX" branch -D story/T-1-fixture >/dev/null 2>&1
git -C "$FIX" checkout -q -b story/T-1-fixture 2>/dev/null
printf -- '---\nid: T-1\ntitle: Fixture story\nslug: fixture\ntype: feature\nstatus: todo\nphase: REVIEW\nbranch: story/T-1-fixture\n---\n\n## Acceptance criteria\n\n- **AC-1** - it works differently now.\n\n## Handoff: RED -> GREEN\n\nthe command, the failure, the export shape.\n' \
  > "$FIX/docs/backlog/stories/T-1.md"
commit_all "T-1 review"
out="$(boundaries)"
assert_contains "changed criteria with no amendment" "## Acceptance criteria differ from main" "$out"


# ---------------------------------------------------------------------------
describe "a large section is read, not silently treated as empty"

# The bug this pins: `has_content` and `has_pasted_output` were
# `strip_comments | grep -q ...`, and this script runs under `set -o pipefail`.
# `grep -q` leaves at the first match, the awk upstream takes SIGPIPE on its
# next write, and the pipeline reports 141 - which reads as false. So a story
# was refused for having a handoff too BIG to fit one awk output buffer, and
# the message said the handoff was empty. Nothing was printed to stderr,
# because a SIGPIPE death is silent.
#
# The buffer size is the implementation's, so the threshold moves: gawk's is
# large and mawk's is 8 KB, which is why this passed on developer machines and
# failed on Ubuntu runners. The sections below are a megabyte so that the bug
# reproduces under EVERY awk rather than only the runner's - a test that only
# fails on CI is not much of a test.
big_text() { # <lines> <prefix>
  awk -v n="$1" -v p="$2" 'BEGIN { for (i = 0; i < n; i++)
    print p " line " i " with enough text on it to make the section large" }'
}

# `story_on_branch` has already opened `## Handoff: RED -> GREEN`, so the first
# block below lands inside it. A second heading of the same name would give
# `section` a small section to stop at and the test would pass either way.
{
  big_text 16000 "handoff"
  printf -- '\n## Gate probes\n\n'
  big_text 16000 "    probe"
  printf -- '\n'
} > "$FIX/.big-sections"

story_on_branch < "$FIX/.big-sections"
out="$(boundaries)"
assert_contains "a megabyte handoff is filled in" "## Handoff is filled in" "$out"
assert_contains "a megabyte gate probe shows its output" "## Gate probes carries pasted output" "$out"
rm -f "$FIX/.big-sections"

# ---------------------------------------------------------------------------
describe "a spike that escalates a gate has to have run it"

# 3e exempts a spike from the gate record, and that exemption is right: a spike
# normally delivers a document, and demanding a full recorded gate run to merge
# markdown would be ceremony. Its WIDTH was wrong. The `required_gates`
# re-check lived inside the non-spike branch of the same `case`, so a spike was
# exempted from that too - it could declare `required_gates: [integration]`,
# never run the gate, and merge with nothing said but
# `note  spike story; gate record not required`. A spike that has gone to the
# trouble of naming a gate has made a deliberate claim that something here
# needs checking, and that claim is the one case the exemption must not cover.
#
# The records below are written by a real `gates.sh` run inside the fixture,
# never pasted. The `tree:` hash has to be the one check-boundaries.sh
# recomputes from the fixture's working tree, and the only way to be sure of
# that is to let the script that owns the hash produce it. project.conf is
# gated, so it is written BEFORE the run that records against it and not
# touched afterwards - editing it in between would move the hash and the test
# would fail on a stale record instead of on what it is about.
gates_record() { # <label>   a full gates.sh run, recorded into T-1
  local out
  out="$( cd "$FIX" && bash scripts/gates.sh --story T-1 2>&1 )"
  assert_contains "fixture: $1 was recorded by gates.sh" \
    "recorded in docs/backlog/stories/T-1.md" "$out"
}

# AC-1: the bug, with no recorded run at all.
story_on_branch spike 'required_gates: [integration]' <<'EOF'
## Notes

A spike that named a gate in its frontmatter and never ran it.
EOF
out="$(boundaries)"; rc=$?
assert_contains "an escalating spike with no gate record is refused" \
  "story T-1: frontmatter requires gate 'integration'" "$out"
assert_eq "and the run fails rather than noting and moving on" "1" "$rc"

# AC-1, second half: a record that exists and does not cover the escalation.
# `integration` is absent from project.conf, which is exactly the hole this
# check was written for - gates.sh silently skips an escalated gate it has no
# command for, so the story's claim goes unrun with a `result: pass` next to it.
story_on_branch spike 'required_gates: [integration]' </dev/null
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
gates_record "a run that never touched integration"
commit_all "T-1 recorded, integration unrun"
out="$(boundaries)"; rc=$?
assert_contains "a recorded run with no PASS for the escalated gate is refused" \
  "story T-1: frontmatter requires gate 'integration'" "$out"
assert_eq "and that run fails too" "1" "$rc"

# AC-2: the exemption still exists for the case it was written for. Every spike
# before this one - MC-028, MC-031, MC-034, MC-035 - merged with no gate record
# and none of them should have needed one, so a fix that simply deleted the
# exemption would be the wrong fix and this is what says so.
story_on_branch spike 'required_gates: []' <<'EOF'
## Notes

A spike that escalated nothing, which is what a spike usually is.
EOF
out="$(boundaries)"; rc=$?
assert_contains "a spike escalating nothing still needs no gate record" \
  "spike story; gate record not required" "$out"
assert_eq "and merges" "0" "$rc"

# AC-3: the escalation is satisfiable. AC-1 and AC-3 differ only in the
# recorded run, so together they separate "the escalation binds" from "every
# escalating spike is refused" - a fix that rejected all of them would pass
# AC-1 and fail here.
story_on_branch spike 'required_gates: [integration]' </dev/null
write_conf "$FIX" <<'EOF'
gate     | unit        | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit        | Tests +[1-9][0-9]* passed
gate     | integration | optional | . | printf 'Tests  3 passed (3)\n'
evidence | integration | Tests +[1-9][0-9]* passed
EOF
gates_record "a run that did cover integration"
assert_contains "fixture: the record carries a PASS for integration" "PASS         integration" \
  "$(cat "$FIX/docs/backlog/stories/T-1.md")"
commit_all "T-1 recorded, integration run"
# "Against a matching tree" is half of what AC-3 claims, and today's spike
# branch never checks it - so without this the fixture could carry a stale hash
# and nobody would know until a fix started looking, which is the worst moment
# to find out. Asserted here against the hash's own owner, not against the
# script under test.
assert_eq "fixture: the record's tree is the tree being merged" \
  "$( cd "$FIX" && CLAUDE_PROJECT_DIR="$FIX" bash -c '. .claude/hooks/lib.sh; gate_tree_hash' )" \
  "$(sed -nE 's/^[[:space:]]*tree:[[:space:]]*([0-9a-f]+).*/\1/p' "$FIX/docs/backlog/stories/T-1.md" | head -1)"
out="$(boundaries)"; rc=$?
assert_eq "a spike that ran the gate it escalated merges" "0" "$rc"
case "$out" in
  *FAIL*) _bad "and nothing in the run objects" "objected: $out" ;;
  *) _ok "and nothing in the run objects" ;;
esac

summary "boundaries"
