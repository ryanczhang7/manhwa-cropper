#!/usr/bin/env bash
# Tests for scripts/gates.sh - the liveness machinery, not any real toolchain.
#
# Every gate command here is a `printf`, so the suite runs in a second and
# tests exactly one thing: whether gates.sh can tell a gate that did work from
# one that only exited 0.

. "$(dirname "${BASH_SOURCE[0]}")/_lib.sh"

FIX="$(make_project_fixture)"
trap 'rm -rf "$FIX"' EXIT

gates() { ( cd "$FIX" && bash scripts/gates.sh "$@" 2>&1 ); }

# ---------------------------------------------------------------------------
describe "evidence: a gate that exits 0 having done nothing"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
out="$(gates)"
assert_contains "a live gate passes" "PASS         unit" "$out"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'No test files found, exiting with code 0\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
out="$(gates)"
assert_contains "a vacuous gate fails" "no evidence of work" "$out"
assert_contains "and the run fails"    "1 required gate(s) failed" "$out"

# ---------------------------------------------------------------------------
describe "--gate names a gate that exists"

# `--gate untt` ran nothing and printed "All required gates passed (0 ran)",
# exit 0. A typo is not a pass.
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
out="$(gates --gate untt)"; rc=$?
assert_contains "a typo is refused" "no gate named 'untt'" "$out"
assert_eq "and exits non-zero" "2" "$rc"

# ---------------------------------------------------------------------------
describe "floor: a gate that started doing much less"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
floor    | unit | 40
EOF
out="$(gates)"
assert_contains "above the floor passes"   "PASS         unit" "$out"
assert_contains "and reports the count"    "observed 47, floor 40" "$out"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  3 passed (3)\n'
evidence | unit | Tests +[1-9][0-9]* passed
floor    | unit | 40
EOF
out="$(gates)"
assert_contains "below the floor fails"     "below the floor of 40" "$out"
assert_contains "naming what it observed"   "did 3 units of work" "$out"

# An optional gate below its floor warns rather than blocking, like any other
# optional failure.
write_conf "$FIX" <<'EOF'
gate     | integration | optional | . | printf 'Tests  1 passed (1)\n'
evidence | integration | Tests +[1-9][0-9]* passed
floor    | integration | 10
EOF
out="$(gates)"
assert_contains "an optional gate below its floor warns" "WARN         integration" "$out"

describe "floor: a floor that cannot be evaluated is a manifest error"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
floor    | unit | 40
EOF
out="$(gates --audit)"
assert_contains "a floor without an evidence line" "floor needs an evidence regex" "$out"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
floor    | unit | lots
EOF
out="$(gates --audit)"
assert_contains "a floor that is not a number" "is not a number" "$out"

describe "floor: an evidence regex that uses alternation"

# `a|b` at the top level would otherwise bind the trailing `.*` to the last
# branch alone, so a count that follows the matched text is invisible.
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests passed: 47\n'
evidence | unit | Tests passed:|Examples passed:
floor    | unit | 40
EOF
out="$(gates)"
assert_contains "counts what follows the branch that matched" "observed 47" "$out"
assert_contains "so the gate passes" "PASS         unit" "$out"

describe "the count is reported even with no floor"

write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'Checked 132 files in 400ms\n'
evidence | lint | Checked [1-9][0-9]* files
EOF
out="$(gates)"
assert_contains "observed count in the summary" "observed 132" "$out"

# ---------------------------------------------------------------------------
describe "no-count: an evidence regex that proves liveness and measures nothing"

# The field report: cargo's `Finished \`dev\` profile ... in 0.29s` proves the
# tool ran and offers no count, so work_count reports the ELAPSED TIME. It read
# 0, then 2, then 6 across runs of the same unchanged workspace. Harmless while
# decorative; a coin-toss gate the moment somebody puts a floor under it.
write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s\n'
evidence | lint | Finished .* profile
EOF
out="$(gates)"
assert_contains "undeclared, the stopwatch is reported as a count" "observed 0" "$out"

write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s\n'
evidence | lint | Finished .* profile
no-count | lint | the digits after it are elapsed seconds, not a count
EOF
out="$(gates)"
assert_contains "declared, the number is withheld" "observed -" "$out"
assert_contains "and liveness still holds"         "PASS         lint" "$out"

# The same gate with nothing to say still fails: `no-count` withholds the
# measurement, it does not excuse the gate from proving it ran.
write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'nothing to do\n'
evidence | lint | Finished .* profile
no-count | lint | the digits after it are elapsed seconds, not a count
EOF
out="$(gates)"
assert_contains "no-count does not weaken the evidence assertion" "no evidence of work" "$out"

describe "no-count: a floor on it is refused"

write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.29s\n'
evidence | lint | Finished .* profile
no-count | lint | the digits after it are elapsed seconds, not a count
floor    | lint | 5
EOF
out="$(gates --audit)"
assert_contains "the audit refuses it" "declared no-count" "$out"
assert_contains "and the audit fails"  "1 manifest problem" "$out"

# Refused at run time too, not only under --audit: a floor that would otherwise
# have PASSED here purely because the fixture's stopwatch read 5.29s.
out="$(gates)"
assert_contains "the run refuses it"  "FAIL         lint" "$out"
assert_contains "rather than passing on the clock" "declared no-count" "$out"

describe "no-count: a line that defends nothing is a manifest error"

write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'Finished `dev` profile in 0.29s\n'
evidence | lint | Finished .* profile
no-count | lint |
EOF
out="$(gates --audit)"
assert_contains "no-count without a reason fails the audit" "marked no-count with no reason" "$out"

write_conf "$FIX" <<'EOF'
gate     | lint | required | . | printf 'Finished `dev` profile in 0.29s\n'
evidence | lint | Finished .* profile
no-count | lnit | a typo, so lint is left open to a floor on its stopwatch
floor    | lint | 5
EOF
out="$(gates --audit)"
assert_contains "no-count naming no gate fails the audit" "names no configured gate" "$out"

# A count-free regex is not detectable from the regex, which is why it is
# declared. `TOTAL` has no digit class either, and the number after it is a
# real one a floor may use.
write_conf "$FIX" <<'EOF'
gate     | coverage | required | . | printf 'TOTAL  1629  16  99.02%%\n'
evidence | coverage | TOTAL
floor    | coverage | 1000
EOF
out="$(gates)"
assert_contains "a digit-class-free regex may still measure" "observed 1629, floor 1000" "$out"

# ---------------------------------------------------------------------------
describe "required_gates: a story can escalate an optional gate for itself"

write_conf "$FIX" <<'EOF'
gate     | unit        | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit        | Tests +[1-9][0-9]* passed
gate     | integration | optional | . | printf 'boom\n'; exit 1
evidence | integration | [1-9][0-9]* passed
EOF

story "$FIX" T-1 GATES </dev/null
set_phase "$FIX" GATES
out="$(gates)"
assert_contains "optional by default: it warns" "WARN         integration" "$out"
assert_contains "and the run still passes"      "All required gates passed" "$out"

story "$FIX" T-1 GATES <<'EOF'
required_gates: [integration]
EOF
out="$(gates)"
assert_contains "escalated: it fails"      "FAIL         integration" "$out"
assert_contains "naming the story"         "required by story T-1" "$out"
assert_contains "and the run fails"        "1 required gate(s) failed" "$out"

# A waiver cannot silence a gate the story requires - that is a bypass, the
# same one waivers are already refused for on repo-required gates.
write_conf "$FIX" <<'EOF'
gate     | integration | optional | . | printf 'boom\n'; exit 1
evidence | integration | [1-9][0-9]* passed
waiver   | integration | the service is not up in CI
EOF
out="$(gates)"
assert_contains "a waiver on a story-required gate is refused" "waiver" "$out"
assert_contains "and it fails"                                 "1 required gate(s) failed" "$out"

set_phase "$FIX" ""


# ---------------------------------------------------------------------------
describe "--fast: the subset that judges whether tests are admissible"

# The field report this came from: RED and GREEN only ever ran the plain test
# command, so a suite that passed both, and passed sixteen local gates, still
# failed a REQUIRED gate in CI - the same tests under coverage instrumentation,
# where one property test crossed the 5s timeout. --fast is the primitive that
# lets RED and GREEN ask the gates the question, without paying for a bundle.
write_conf "$FIX" <<'EOF'
gate     | lint     | required | . | printf 'Checked 12 files\n'
gate     | unit     | required | . | printf 'Tests  47 passed (47)\n'
gate     | coverage | required | . | printf 'Tests  47 passed (47)\n'
gate     | build    | required | . | printf 'Bundled 3 targets\n'
evidence | lint     | Checked [1-9][0-9]* files
evidence | unit     | Tests +[1-9][0-9]* passed
evidence | coverage | Tests +[1-9][0-9]* passed
evidence | build    | Bundled [1-9][0-9]* targets
slow     | build    | a Tauri release bundle; RED has no use for it
EOF
out="$(gates --fast)"
assert_contains "a fast gate runs"          "PASS         lint" "$out"
assert_contains "the instrumented one runs" "PASS         coverage" "$out"
case "$out" in
  *"PASS         build"*) _bad "a slow gate is skipped" "build ran anyway: $out" ;;
  *) _ok "a slow gate is skipped" ;;
esac
assert_contains "and is named"        "--fast skipped: build" "$out"
assert_contains "with the caveat"     "This is a subset, not a verdict" "$out"

# A subset is not evidence, for the same reason --gate and --required are not.
story "$FIX" T-1 GATES <<'EOF'
EOF
set_phase "$FIX" GATES
out="$(gates --fast)"
assert_contains "a fast run is never recorded" "not recorded in the story" "$out"
out="$(gates)"
assert_contains "a full run still is"          "recorded in docs/backlog/stories/T-1.md" "$out"
assert_contains "and points at CI's other script" "check-boundaries.sh" "$out"
set_phase "$FIX" ""

# With nothing marked slow, --fast is a full run in everything but the record,
# and says so rather than letting anyone believe they bought speed.
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
out="$(gates --fast)"
assert_contains "no slow lines is reported" "--fast skipped nothing" "$out"

# ---------------------------------------------------------------------------
describe "slow: a line that excludes nothing is a manifest error"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
slow     | unit |
EOF
out="$(gates --audit)"
assert_contains "slow without a reason fails the audit" "marked slow with no reason" "$out"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
slow     | unti | a typo, so `build` never leaves the fast subset
EOF
out="$(gates --audit)"
assert_contains "slow naming no gate fails the audit" "names no configured gate" "$out"

# ---------------------------------------------------------------------------
describe "ci-factor: a measurement, or nothing"

# The number is how much slower ONE TEST is on CI under this gate. It is worth
# recording because the obvious way to derive it - the gate's own wall time,
# most of which is fixed overhead - overestimates it several-fold and sends
# somebody optimising a test that was already fast enough.
write_conf "$FIX" <<'EOF'
gate      | coverage | required | . | printf 'Tests  47 passed (47)\n'
evidence  | coverage | Tests +[1-9][0-9]* passed
ci-factor | coverage | 3.4 | actions run 412, AC-4 file 1,262 ms instrumented
EOF
out="$(gates --audit)"
assert_contains "a measured factor passes the audit" "ci-factor: 3.4" "$out"

write_conf "$FIX" <<'EOF'
gate      | coverage | required | . | printf 'Tests  47 passed (47)\n'
evidence  | coverage | Tests +[1-9][0-9]* passed
ci-factor | coverage | about 14x | eyeballed it
EOF
out="$(gates --audit)"
assert_contains "a factor that is not a number fails" "is not a number" "$out"

write_conf "$FIX" <<'EOF'
gate      | coverage | required | . | printf 'Tests  47 passed (47)\n'
evidence  | coverage | Tests +[1-9][0-9]* passed
ci-factor | coverage | 3.4
EOF
out="$(gates --audit)"
assert_contains "a factor with no source fails" "has no source" "$out"

write_conf "$FIX" <<'EOF'
gate      | coverage | required | . | printf 'Tests  47 passed (47)\n'
evidence  | coverage | Tests +[1-9][0-9]* passed
ci-factor | covrage  | 3.4 | actions run 412
EOF
out="$(gates --audit)"
assert_contains "a factor naming no gate fails the audit" "names no configured gate" "$out"

summary "gates"
