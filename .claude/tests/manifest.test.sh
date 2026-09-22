#!/usr/bin/env bash
# Properties of THIS repository's .claude/harness/project.conf.
#
# gates.test.sh is the opposite kind of suite: generic fixture tests of the
# gates.sh machinery, where a floor is written into a throwaway conf and the
# machinery's reaction is asserted. The generic floor behaviour lives there and
# is not repeated here.
#
# What is NOT covered by any fixture is whether the real manifest this project
# ships actually carries the protections it needs. A floor that exists in a
# fixture defends nothing; `integration` had an evidence line and no floor for
# the whole of EPIC-07, so the gate that scores the corpus would have reported
# PASS on a suite that shrank from fourteen tests to one.
#
# profiles.test.sh is the precedent: the same rules `gates.sh --audit` applies
# to a real manifest, applied through $REPO_ROOT to real files rather than
# fixtures.
#
# Deliberately NOT asserted: the VALUE of any floor. That number is measured by
# a full gate run (MC-040 AC-2), and a test that pins it would either re-derive
# a measurement it cannot take or freeze a number the next measurement moves.
# Only its shape is asserted here, plus the one bound the story's own control
# text gives: a floor of 1 passes on any non-empty suite and defends nothing.

. "$(dirname "${BASH_SOURCE[0]}")/_lib.sh"

CONF="$REPO_ROOT/.claude/harness/project.conf"

# --- reading the manifest ----------------------------------------------------

# conf_line <kind> <gate id>   The first `<kind> | <gate id> | ...` line, whole,
# or nothing when there is none.
conf_line() {
  awk -F'|' -v k="$1" -v g="$2" '
    function t(s) { gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
    t($1) == k && t($2) == g { print; exit }
  ' "$CONF"
}

# conf_value <kind> <gate id>   Field 3 of that line, trimmed.
conf_value() {
  awk -F'|' -v k="$1" -v g="$2" '
    function t(s) { gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
    t($1) == k && t($2) == g { print t($3); exit }
  ' "$CONF"
}

# conf_rest <kind> <gate id>   Everything from field 3 on, rejoined: an evidence
# regex may use `|` as alternation.
conf_rest() {
  awk -F'|' -v k="$1" -v g="$2" '
    function t(s) { gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
    function rest(n,   i, o) { o = $n; for (i = n + 1; i <= NF; i++) o = o "|" $i; return t(o) }
    t($1) == k && t($2) == g { print rest(3); exit }
  ' "$CONF"
}

# comment_above <gate id>   The contiguous run of `#` lines immediately above
# the gate's floor line.
#
# It reports a SENTENCE, not an empty string, when there is no floor line to sit
# under - because the assertion this feeds exists to catch a floor that arrives
# with no explanation, and "find the floor, read the comment above it" passes
# vacuously while there is no floor to find. A missing floor must fail the
# comment assertions too, and loudly.
comment_above() {
  awk -F'|' -v g="$1" '
    function t(s) { gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
    { line[NR] = $0 }
    t($1) == "floor" && t($2) == g && !at { at = NR }
    END {
      if (!at) {
        print "(there is no `floor | " g " | <n>` line in project.conf, so there is no comment above one)"
        exit
      }
      for (i = at - 1; i >= 1; i--) {
        if (line[i] !~ /^[[:space:]]*#/) break
        out = line[i] "\n" out
      }
      if (out == "") print "(the `floor | " g "` line has no comment above it at all)"
      else printf "%s", out
    }
  ' "$CONF"
}

# audit_block <gate id>   The `--audit` report for one gate: its own line plus
# the indented lines under it.
audit_block() {
  printf '%s\n' "$AUDIT_OUT" | awk -v g="$1" '
    /^[^[:space:]]/ { inb = ($2 == g) }
    inb { print }
  '
}

# --- the checks, run against two gates ---------------------------------------

# check_floor <gate id>   Every floor rule this story pins, applied identically
# to `integration` (the gate MC-040 adds a floor to) and to `unit` (the floor
# that already exists, and the comment AC-4 names as the template). `unit` is a
# live control: if these checks were vacuous they would pass on a gate with no
# floor, and if they were wrong they would fail on the floor the repository has
# been shipping since MC-005.
check_floor() { # <gate id>
  local g="$1" line value comment

  line="$(conf_line floor "$g")"
  value="$(conf_value floor "$g")"
  comment="$(comment_above "$g")"

  describe "$g: the manifest declares a floor, and says what the floor defends"

  if printf '%s\n' "$line" | grep -qE "^[[:space:]]*floor[[:space:]]*\|[[:space:]]*$g[[:space:]]*\|[[:space:]]*[0-9]+[[:space:]]*$"; then
    _ok "$g has a \`floor | $g | <n>\` line"
  else
    _bad "$g has a \`floor | $g | <n>\` line" "an evidence line proves the gate did SOMETHING; only a floor catches
a gate that quietly started doing much less, and this one has none.
found: ${line:-<no floor line for $g in project.conf>}"
  fi

  if printf '%s' "$value" | grep -qE '^[0-9]+$' && [ "$value" -gt 0 ]; then
    _ok "$g's floor is a positive integer"
  else
    _bad "$g's floor is a positive integer" "gates.sh compares the observed count against this value numerically,
so it has to be digits, and a floor of 0 is satisfied by a suite that ran nothing.
found: ${value:-<none>}"
  fi

  if printf '%s' "$value" | grep -qE '^[0-9]+$' && [ "$value" -gt 1 ]; then
    _ok "$g's floor is more than 1"
  else
    _bad "$g's floor is more than 1" "\"a floor set to 1 passes on any non-empty suite and defends nothing\"
(MC-040 AC-2's own control). The floor's actual value is measured by a full
gate run, not asserted here; only that it is not the degenerate one.
found: ${value:-<none>}"
  fi

  comment_check() { # <label> <regex> <why it matters>
    if printf '%s' "$comment" | grep -qiE "$2"; then
      _ok "$1"
    else
      _bad "$1" "$3
comment above \`floor | $g\`:
$comment"
    fi
  }

  comment_check "$g's floor comment names the test binary the number tracks" \
    '[A-Za-z0-9_]+\.rs' \
    "the number is read out of the FIRST matching \`test result: ok. N\` line, which
belongs to one binary and not to the workspace. A comment that does not name
that binary leaves the next agent unable to tell a real drop from a reordering."

  comment_check "$g's floor comment says what would move the number" \
    'mov(e|es|ed|ing)' \
    "a floor that tracks one binary moves when a different binary starts sorting
first. The comment has to say so, or the move looks like a regression."

  comment_check "$g's floor comment forbids lowering it to make the gate pass" \
    'lower' \
    "lowering a floor is the gate equivalent of deleting a test case. project.conf
states the rule; the comment above the floor is where the next agent will read it."
}

# --- integration's evidence line: the thing a floor is measured out of --------
# All four of these hold today. They are admissibility assertions for AC-1 - a
# floor is refused by `gates.sh --audit` unless the gate has an evidence line
# that is not declared no-count, and it measures a truncated count unless that
# regex covers the whole number.

describe "integration is configured as an optional gate with a countable evidence line"

gate_line="$(conf_line gate integration)"
assert_contains "project.conf configures an \`integration\` gate" "integration" "${gate_line:-<no gate line>}"

sev="$(conf_value gate integration)"
assert_eq "integration stays optional; this story puts a floor under it, it does not promote it" \
  "optional" "$sev"

ev="$(conf_rest evidence integration)"
if [ -n "$ev" ] && [ "$ev" != "-" ]; then
  _ok "integration has an evidence line for a floor to be measured out of"
else
  _bad "integration has an evidence line for a floor to be measured out of" \
    "gates.sh --audit refuses a floor on a gate with no evidence regex.
found: ${ev:-<none>}"
fi

nc="$(conf_line no-count integration)"
if [ -z "$nc" ]; then
  _ok "integration is not declared no-count, so there is a count to compare against"
else
  _bad "integration is not declared no-count, so there is a count to compare against" \
    "a no-count declaration says the evidence regex measures nothing; a floor on it
would compare against a stopwatch, and gates.sh refuses it at audit and run time.
found: $nc"
fi

# What the floor will actually be measured out of. Mirrors scripts/gates.sh
# work_count(): the first run of digits at or after the start of the first
# evidence match, where the match is `(<regex>).*`.
measure() { # <log line> <evidence regex>
  printf '%s\n' "$1" | grep -oE -m1 -- "($2).*" 2>/dev/null | head -1 \
    | grep -oE '[0-9]+' 2>/dev/null | head -1
}

OK14='test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
OK0='test result: ok. 0 passed; 0 failed; 9 ignored; 0 measured; 0 filtered out'

assert_eq "integration's evidence regex reads the whole count out of a 14-test result line" \
  "14" "$(measure "$OK14" "$ev")"

# The property the floor on THIS gate leans on hardest, and the reason the
# ordering question under `-- --ignored` differs from the unit gate's: a test
# binary with no ignored tests prints `test result: ok. 0`, and the leading
# `[1-9]` must keep that line from supplying the count. If it ever matched, the
# floor would be compared against the first binary cargo happened to run.
assert_eq "integration's evidence regex ignores a \`test result: ok. 0\` line entirely" \
  "" "$(measure "$OK0" "$ev")"

# Negative control for the assertion above: the same pipeline, the same line,
# a regex whose leading class admits 0 - it measures 0. Without this, "reads
# nothing" would also be the answer if `measure` were simply broken.
assert_eq "a regex admitting a leading 0 does read 0 off that line, so the check above discriminates" \
  "0" "$(measure "$OK0" 'test result: ok\. [0-9]')"

# AC-1's remaining clause, "the regex covers the whole number", checked on the
# regex rather than on a measurement - deliberately, and this is worth writing
# down. gates.sh's work_count comment says a regex stopping mid-number "measures
# a truncated count"; measured, it does not, because the trailing `.*` extends
# the match to the end of the line and the digit run is read from there:
#   narrow `test result: ok\. [1-9]` on the 14-test line above also reads 14.
# So the clause is a manifest rule with no observable consequence here, and the
# honest way to pin it is as a rule.
# A digit class followed by a repetition: `[0-9]*` in `test result: ok\. [1-9][0-9]*`.
covers_whole_number() { printf '%s' "$1" | grep -qE '\[[^]]*\][*+]'; }

if covers_whole_number "$ev"; then
  _ok "integration's evidence regex covers the whole number, not just its first digit"
else
  _bad "integration's evidence regex covers the whole number, not just its first digit" \
    "project.conf: a floor needs an evidence regex that covers the whole number.
found: $ev"
fi

# Control for the predicate itself: the truncating regex must not satisfy it.
if covers_whole_number 'test result: ok\. [1-9]'; then
  _bad "the whole-number check rejects a regex that stops at the first digit" \
    "covers_whole_number accepted \`test result: ok\\. [1-9]\`, so it accepts anything"
else
  _ok "the whole-number check rejects a regex that stops at the first digit"
fi

# --- the floors ---------------------------------------------------------------

check_floor unit          # the control: the floor and comment AC-4 uses as its template
check_floor integration   # the floor MC-040 adds

# --- the audit accepts the result ---------------------------------------------

describe "gates.sh --audit accepts the manifest, floor included"

AUDIT_OUT="$(cd "$REPO_ROOT" && bash scripts/gates.sh --audit 2>&1)"
audit_rc=$?

assert_eq "gates.sh --audit exits 0 on the real manifest" "0" "$audit_rc"
assert_contains "gates.sh --audit reports the manifest audit passed" \
  "Manifest audit passed." "$AUDIT_OUT"

# The audit exiting 0 would also be satisfied by a floor line the audit never
# saw - a misspelt gate id fails outright, but a floor written under the wrong
# heading, or never written at all, is invisible to an exit code. Assert the
# audit reports the floor against the gate it was meant for.
assert_contains "the audit reports a floor under unit" "floor:" "$(audit_block unit)"
assert_contains "the audit reports a floor under integration" "floor:" "$(audit_block integration)"

summary "manifest"
