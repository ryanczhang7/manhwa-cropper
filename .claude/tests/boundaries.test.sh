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

# story_on_branch   Writes docs/backlog/stories/T-1.md on a fresh story branch
# cut from main, with the body on stdin appended after the frontmatter.
story_on_branch() {
  git -C "$FIX" checkout -q main 2>/dev/null
  git -C "$FIX" branch -D story/T-1-fixture >/dev/null 2>&1
  git -C "$FIX" checkout -q -b story/T-1-fixture 2>/dev/null
  mkdir -p "$FIX/docs/backlog/stories"
  {
    printf -- '---\nid: T-1\ntitle: Fixture story\nslug: fixture\ntype: feature\nstatus: todo\nphase: REVIEW\nbranch: story/T-1-fixture\n---\n\n'
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

summary "boundaries"
