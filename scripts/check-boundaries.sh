#!/usr/bin/env bash
# Defence in depth for CI (and for humans working outside Claude Code).
#
# The phase-guard hook enforces the RED/GREEN lock at write time inside a
# session. This checks the same invariants after the fact, on a diff, where the
# hook was never in the loop - and the invariants the hook cannot see at all,
# because they are about a story file's content over time rather than about
# one write: frozen acceptance criteria, a tool-written gate record that matches
# the code being merged, a filled-in handoff, a scaffold inventory.
#
#   bash scripts/check-boundaries.sh [base-ref]      (default: origin/main)
#
# In GitHub Actions on a pull_request the checkout is a detached merge commit,
# so `git rev-parse --abbrev-ref HEAD` is "HEAD" and names no story. The story
# is identified from GITHUB_HEAD_REF there, and the gate tree hash is recomputed
# at PR_HEAD_SHA (set by the workflow) rather than at the merge commit, whose
# tree includes whatever moved on the base branch since.

set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BASE="${1:-origin/main}"
export CLAUDE_PROJECT_DIR="$ROOT"
. "$ROOT/.claude/hooks/lib.sh"

fail=0
note() { printf '  %s\n' "$1"; }
problem() { printf 'FAIL  %s\n' "$1"; fail=1; }
ok() { printf 'ok    %s\n' "$1"; }

# section <file|-> <heading-prefix>   Body of "## <heading-prefix>..." up to the
# next "## ". "-" reads stdin, so `git show ref:path | section - ...` works.
section() {
  awk -v h="## $2" 'index($0, h) == 1 { on=1; next } on && /^## / { exit } on { print }' "$1"
}

# strip_comments   stdin with every HTML comment removed. The story template is
# mostly comments, so "did anybody write anything here" and "is there evidence
# in here" are both questions about what is left after they go.
strip_comments() {
  awk '{ s = s $0 "\n" }
       END {
         while ((i = index(s, "<!--")) > 0) {
           r = substr(s, i); j = index(r, "-->")
           if (j == 0) { s = substr(s, 1, i - 1); break }
           s = substr(s, 1, i - 1) substr(r, j + 3)
         }
         printf "%s", s
       }'
}

# has_content   True if stdin holds anything besides whitespace and HTML
# comments - i.e. somebody wrote something beyond the template.
has_content() { strip_comments | grep -q '[^[:space:]]'; }

# has_pasted_output   True if stdin contains a fenced or indented block, which
# is what a pasted command output looks like in a story file. It cannot tell
# real output from prose in a fence, and does not try: it separates "here is
# what happened" from "trust me, it happened", which is the distinction the
# non-negotiables are actually about.
has_pasted_output() { strip_comments | grep -qE '^[[:space:]]*(```|~~~)|^    [^[:space:]]'; }

# story_field <file|-> <key>   A frontmatter value.
story_field() { frontmatter_value "$1" "$2"; }   # lib.sh

# --- 1. Story files are well formed ----------------------------------------
story_fail_start=$fail
for f in docs/backlog/stories/*.md; do
  [ -e "$f" ] || continue
  base=$(basename "$f" .md)
  head -1 "$f" | grep -q '^---' || { problem "$f: missing YAML frontmatter"; continue; }
  for key in id title type status phase; do
    grep -qE "^$key:" "$f" || problem "$f: frontmatter missing '$key'"
  done
  fid=$(story_field "$f" id)
  [ "$fid" = "$base" ] || problem "$f: frontmatter id '$fid' does not match filename '$base'"
  ph=$(story_field "$f" phase)
  grep -qE "^[[:space:]]*$ph[[:space:]]*\|" .claude/harness/phases.conf \
    || problem "$f: unknown phase '$ph'"
  grep -q '^## Acceptance criteria' "$f" || problem "$f: no '## Acceptance criteria' section"
done
[ "$fail" -eq "$story_fail_start" ] && ok "story files validated"

# --- 2. Runtime state must never be committed -------------------------------
if git ls-files --error-unmatch .claude/state/current-story.env >/dev/null 2>&1; then
  problem ".claude/state/current-story.env is tracked; it is machine-local state"
fi
ok "harness state not tracked"

# --- 3. The diff, and the story it belongs to --------------------------------
if ! git rev-parse --verify "$BASE" >/dev/null 2>&1; then
  note "base ref '$BASE' not available; skipped diff checks"
  exit $fail
fi

changed=$(git diff --name-only "$BASE"...HEAD 2>/dev/null || true)
classified=$(printf '%s\n' "$changed" | classify_stdin)
src_files=$(printf '%s\n' "$classified" | awk -F'\t' '$1 == "source" { print $2 }')
src=$(printf '%s\n' "$src_files" | grep -c '[^[:space:]]')
tst=$(printf '%s\n' "$classified" | awk -F'\t' '$1 == "test"' | grep -c '[^[:space:]]')

br="${GITHUB_HEAD_REF:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null)}"
sid=$(printf '%s' "$br" | sed -nE 's|^story/([A-Za-z0-9]+-[0-9]+).*|\1|p')
sfile="docs/backlog/stories/$sid.md"
story_type=""; ph=""
if [ -n "$sid" ] && [ -f "$sfile" ]; then
  story_type=$(story_field "$sfile" type)
  ph=$(story_field "$sfile" phase)
fi

# 3a. Production code arrives with tests - or, under SCAFFOLD, with an inventory
if [ "$src" -gt 0 ] && [ "$tst" -eq 0 ]; then
  case "$story_type" in
    bootstrap|chore|spike)
      note "$src source file(s) without tests - allowed for a '$story_type' story, so the inventory must account for them"
      inv="$(section "$sfile" "Scaffold inventory")"
      if ! printf '%s\n' "$inv" | has_content; then
        problem "story $sid: source changed without tests, and ## Scaffold inventory is empty. Name every production file written and the test that covers it."
      else
        missing=""
        while IFS= read -r p; do
          [ -z "$p" ] && continue
          printf '%s\n' "$inv" | grep -qF -- "$p" || missing="$missing $p"
        done <<< "$src_files"
        if [ -n "$missing" ]; then
          problem "story $sid: changed source file(s) not named in ## Scaffold inventory:$missing"
        else
          ok "every changed source file is named in ## Scaffold inventory"
        fi
      fi ;;
    *) problem "$src source file(s) changed with no test changes. Production code ships with the test that demanded it." ;;
  esac
else
  ok "source changes accompanied by test changes ($src source, $tst test)"
fi

[ -n "$sid" ] || exit $fail
if [ ! -f "$sfile" ]; then
  problem "branch '$br' names story $sid but $sfile does not exist"
  exit $fail
fi

# 3b. A PR is opened from REVIEW or DONE
case "$ph" in
  REVIEW|DONE) ok "story $sid is in $ph" ;;
  *) problem "story $sid is in phase '$ph'; a PR should be opened from REVIEW or DONE" ;;
esac

# 3c. The branch is the story's branch
fb=$(story_field "$sfile" branch)
if [ -n "$fb" ] && [ "$fb" != "$br" ]; then
  problem "story $sid: frontmatter says branch '$fb' but this is '$br'"
else
  ok "branch matches the story's frontmatter"
fi

# 3d. Acceptance criteria are frozen: any difference from the base branch needs
#     an ## Amendments entry, whatever phase the edit was made in. CI cannot
#     see when in the branch's history an edit happened, only that it did.
if git cat-file -e "$BASE:$sfile" 2>/dev/null; then
  before=$(git show "$BASE:$sfile" | section - "Acceptance criteria" | sed 's/[[:space:]]*$//')
  after=$(section "$sfile" "Acceptance criteria" | sed 's/[[:space:]]*$//')
  if [ "$before" = "$after" ]; then
    ok "acceptance criteria unchanged since $BASE"
  elif section "$sfile" "Amendments" | has_content; then
    ok "acceptance criteria changed, with an ## Amendments entry"
  else
    problem "story $sid: ## Acceptance criteria differ from $BASE with no ## Amendments entry. Criteria are frozen once a story leaves PLANNED; record which AC changed, what it said, what it says now, who approved it and why."
  fi
else
  note "story file is new in this PR; nothing to freeze the criteria against"
fi

# 3e. The gate record was written by gates.sh and matches the code being merged
case "$ph:$story_type" in
  REVIEW:spike|DONE:spike) note "spike story; gate record not required" ;;
  REVIEW:*|DONE:*)
    gr="$(section "$sfile" "Gate results")"
    if ! printf '%s\n' "$gr" | grep -qF -- "$GATE_MARKER"; then
      problem "story $sid: ## Gate results was not written by scripts/gates.sh. Run 'bash scripts/gates.sh' - it records its own result; a pasted summary is not evidence."
    else
      res=$(printf '%s\n' "$gr" | sed -nE 's/^[[:space:]]*result:[[:space:]]*//p' | head -1)
      case "$res" in
        pass*) ok "recorded gate result: $res" ;;
        *) problem "story $sid: recorded gate result is '$res'" ;;
      esac
      rec=$(printf '%s\n' "$gr" | sed -nE 's/^[[:space:]]*tree:[[:space:]]*([0-9a-f]+).*/\1/p' | head -1)
      if [ -n "${PR_HEAD_SHA:-}" ]; then
        now=$(gate_tree_hash_of "$PR_HEAD_SHA"); where="commit ${PR_HEAD_SHA:0:7}"
      else
        now=$(gate_tree_hash); where="the working tree"
      fi
      if [ -n "$rec" ] && [ "$rec" = "$now" ]; then
        ok "gate record matches $where (tree $rec)"
      else
        problem "story $sid: gates were recorded against tree '${rec:-none}' but $where is '$now'. Source, test or config changed after the last full gate run; run 'bash scripts/gates.sh' again and commit the result."
      fi

      # A gate the story escalated must appear in the record as having passed.
      # gates.sh enforces this while it runs; this catches the record written
      # before the escalation was added, where the gate is optional again by
      # the time anyone looks.
      for g in $(frontmatter_list "$sfile" required_gates); do
        if printf '%s\n' "$gr" | grep -qE "^[[:space:]]*PASS[[:space:]]+$g( |\(|$)"; then
          ok "story-required gate '$g' passed in the recorded run"
        else
          problem "story $sid: frontmatter requires gate '$g', but the recorded run has no PASS for it. Run 'bash scripts/gates.sh' again."
        fi
      done
    fi ;;
esac

# 3f. The handoff was actually written
case "$story_type" in
  feature|fix)
    if section "$sfile" "Handoff" | has_content; then
      ok "## Handoff is filled in"
    else
      problem "story $sid: ## Handoff is empty. It is the only channel to the next agent; RED is not finished without it."
    fi ;;
esac

# 3g. A corrected test was observed to fail too
#
# The first non-negotiable is a property of an ASSERTION, not of a phase: a
# test that has never been seen to fail is not a test. An ordinary RED
# discharges it as a side effect, because the implementation does not exist
# yet. A test corrected on a RETURN cannot: the implementation is right there,
# so the corrected assertion passes on its first execution and passes forever,
# and it could assert nothing at all without anything noticing.
#
# That is why a return to RED has to leave evidence behind - the red from a
# deliberate, reverted mutation, or a before/after measurement where the defect
# was cost. Prose describing either is not either. The same rule already
# applies to ## Gate probes, and for the same reason.
for sec in Regressions "Gate probes"; do
  body="$(section "$sfile" "$sec")"
  printf '%s\n' "$body" | has_content || continue
  if printf '%s\n' "$body" | has_pasted_output; then
    ok "## $sec carries pasted output"
  else
    problem "story $sid: ## $sec describes something without showing it. Paste the output - the failure a reverted mutation produced, or the before/after measurement taken under the gate command. A test corrected while the implementation exists has never been observed to fail, and a description of red is not red."
  fi
done

exit $fail
