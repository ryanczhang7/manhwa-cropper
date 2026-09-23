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
describe "the evidence regex's width does not change the count it reads"

# The field report (MC-043): the comment over work_count claimed that a regex
# which stops mid-number - the `[1-9]` in `test result: ok\. [1-9]` - "measures
# a truncated count", and pointed the next agent at widening it. That is false
# and cannot be otherwise: the match is `($2).*`, and the trailing `.*` always
# runs to the end of the line, so the whole number is inside the counted span
# whatever the regex stopped at. A regex cannot truncate a count it does not
# have to contain. Narrow and wide read the SAME number. Nothing executable
# tested the claim, which is how it survived for months; these two runs are
# that test.
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'test result: ok. 836 passed; 0 failed; 0 ignored\n'
evidence | unit | test result: ok\. [1-9]
EOF
narrow="$(gates)"
assert_contains "a regex stopping at the first digit still reads the whole number" \
  "observed 836" "$narrow"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'test result: ok. 836 passed; 0 failed; 0 ignored\n'
evidence | unit | test result: ok\. [1-9][0-9]*
EOF
wide="$(gates)"
assert_contains "and so does one that covers the whole number" \
  "observed 836" "$wide"

# Asserted as the equality too, not left implicit in two literals: the claim is
# that the WIDTH makes no difference, and that is the sentence a reader has to
# be able to break.
observed_of() { printf '%s\n' "$1" | sed -n 's/.*, observed \([0-9][0-9]*\).*/\1/p' | head -1; }
assert_eq "so the narrow and the wide regex read the same count" \
  "$(observed_of "$wide")" "$(observed_of "$narrow")"

# ---------------------------------------------------------------------------
describe "the evidence regex's leading digit class decides which line is counted"

# The consequence that IS real, and the one `floor | integration | 9` in
# project.conf rests on. A line the evidence regex does not match is skipped
# entirely - `grep -oE -m1` takes the FIRST line that matches - so the count
# comes from a later line, or from an earlier one that a wider leading class
# happens to admit. Under `cargo test --workspace --release -- --ignored`
# twelve `test result: ok. 0` lines go by unmatched before the first match,
# which is why that floor tracks `crates/engine tests/corpus.rs` rather than
# whichever binary cargo ran first.
#
# Three result lines, not two: cargo keeps printing them after the one that
# matched, and the third makes "the FIRST match wins" falsifiable as well as
# "the `ok. 0` line is skipped".
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'test result: ok. 0 passed; 0 failed; 9 ignored\ntest result: ok. 14 passed; 0 failed; 0 ignored\ntest result: ok. 7 passed; 0 failed; 0 ignored\n'
evidence | unit | test result: ok\. [1-9][0-9]*
EOF
out="$(gates)"
assert_contains "a leading [1-9] skips the \`ok. 0\` line and counts the next match" \
  "observed 14" "$out"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'test result: ok. 0 passed; 0 failed; 9 ignored\ntest result: ok. 14 passed; 0 failed; 0 ignored\ntest result: ok. 7 passed; 0 failed; 0 ignored\n'
evidence | unit | test result: ok\. [0-9][0-9]*
EOF
out="$(gates)"
assert_contains "a leading [0-9] matches that same line, and the count is 0" \
  "observed 0" "$out"

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

# ---------------------------------------------------------------------------
describe "one run at a time: two gate runs on one checkout"

# The field report: an orchestrator and a subagent it had dispatched both ran
# gates.sh on the same checkout. They shared `target/`, fought over
# cargo-llvm-cov's profraw counters, and both rewrote the story's ## Gate
# results - so the SAME tree hash was recorded `result: fail` and then
# `result: pass`, with three runs' durations interleaved, while the coverage
# log on disk held a complete table at 99.71% against a floor of 95. Nothing
# had failed. check-boundaries.sh caught it on the tree hash, which is the last
# line of defence being asked to do the first line's job.

LOCK="$FIX/.claude/state/gate-run.lock"
rm -rf "$LOCK"

# fake_lock <pid> <host>   A lock as gates.sh would have left one.
fake_lock() {
  rm -rf "$LOCK"; mkdir -p "$LOCK"
  printf 'pid:     %s\nhost:    %s\nstarted: 2020-01-01T00:00:00Z\ncommand: gates.sh\n' \
    "$1" "$2" > "$LOCK/owner"
}

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF

# A run that finishes normally must not leave the next one queueing forever.
out="$(gates)"
assert_contains "an ordinary run still passes" "All required gates passed" "$out"
if [ -d "$LOCK" ]; then _bad "the lock is released on exit" "$LOCK still exists"
else _ok "the lock is released on exit"; fi

# --- held by a live process -------------------------------------------------
# The owner has to be alive for every assertion below it, and a background
# `sleep N` is a wall clock: under load the seven gates.sh invocations that
# follow outlast it, the owner dies mid-block, acquire_lock() correctly takes a
# stale lock over, and five assertions that exist to pin "a live owner is waited
# for" observe the opposite (MC-044). This runner's own pid is alive for the
# whole block by construction - it IS the block - so the fixture is
# deterministic with no change to scripts/gates.sh: the take-over branch turns
# on `host = $LOCK_HOST` AND `! kill -0 "$pid"`, and `kill -0 $$` from the
# gates.sh child always succeeds. Nothing is started, so nothing needs killing
# afterwards.
LIVE=$$
fake_lock "$LIVE" "$(hostname 2>/dev/null || printf '%s' "${HOSTNAME:-unknown}")"

out="$(gates --no-wait)"; rc=$?
assert_contains "a second run says what is holding it" "Another gate run is already in progress" "$out"
assert_contains "and names the owner"                  "pid:     $LIVE" "$out"
assert_contains "and refuses rather than proceeding"   "Refusing to start a second one" "$out"
assert_eq "with an exit code that is not a gate verdict" "3" "$rc"
case "$out" in
  *"PASS         unit"*) _bad "and runs nothing" "the gate ran anyway: $out" ;;
  *) _ok "and runs nothing" ;;
esac

# --gate shares `target/` with everything else, so it queues like any other run.
out="$(gates --gate unit --no-wait)"; rc=$?
assert_eq "--gate is locked too"   "3" "$rc"
out="$(gates --fast --no-wait)"; rc=$?
assert_eq "--fast is locked too"   "3" "$rc"
out="$(gates --required --no-wait)"; rc=$?
assert_eq "--required is locked too" "3" "$rc"

# Reading project.conf is not running a gate. An audit must not queue behind a
# twenty-minute coverage run to tell you a regex is misspelt.
out="$(gates --audit)"; rc=$?
assert_eq "--audit takes no lock"  "0" "$rc"
out="$(gates --list)"; rc=$?
assert_eq "--list takes no lock"   "0" "$rc"

# Waiting is the default, and it gives up with a message rather than hanging.
out="$(GATES_LOCK_WAIT=2 gates)"; rc=$?
assert_contains "waiting is the default"     "Waiting for it to finish" "$out"
assert_contains "and it gives up out loud"   "gave up waiting for the gate run lock" "$out"
assert_contains "saying nothing ran"         "neither a pass nor a failure" "$out"
assert_eq "and exits 3"                      "3" "$rc"

# The claim the four assertions above only imply: it waited because the owner is
# ALIVE, not because it happened to give up on something it had already broken.
# Stated directly, as the foreign-host case below states it, so that a fixture
# whose owner dies mid-block is caught here rather than read as a pass.
case "$out" in
  *"Taking it over"*) _bad "a live owner's lock is never taken over" "broke it: $out" ;;
  *) _ok "a live owner's lock is never taken over" ;;
esac

# No teardown: the live owner above is this script, which the next fake_lock
# simply stops naming. (It was `kill "$LIVE"` when the owner was a `sleep`.)

# --- left behind by a dead process ------------------------------------------
# A hard kill leaves a lock with nobody behind it. Wedging until somebody reads
# a message about `rm -rf` teaches people to `rm -rf` first and read later.
sleep 0 & DEAD=$!; wait "$DEAD" 2>/dev/null
fake_lock "$DEAD" "$(hostname 2>/dev/null || printf '%s' "${HOSTNAME:-unknown}")"
out="$(GATES_LOCK_WAIT=2 gates)"; rc=$?
assert_contains "a lock with no live owner is taken over" "which is gone. Taking it over" "$out"
assert_contains "and the run proceeds"                    "All required gates passed" "$out"
assert_eq "successfully"                                  "0" "$rc"

# A lock with no owner file names nobody to wait for. It is only makeable by
# dying in the microseconds between `mkdir` and the write after it, so it is
# debris - but it used to cost the next run the whole wait for nothing.
rm -rf "$LOCK"; mkdir -p "$LOCK"
out="$(GATES_LOCK_WAIT=120 gates)"; rc=$?
assert_contains "a lock naming no owner is taken over" "names no owner" "$out"
assert_eq "and the run proceeds"                       "0" "$rc"

# A lock held from another machine is waited for, never broken: this host
# cannot see that process table, so "no such pid" means nothing about it.
fake_lock 999991 some-other-host
out="$(GATES_LOCK_WAIT=2 gates)"; rc=$?
case "$out" in
  *"Taking it over"*) _bad "another host's lock is never broken" "broke it: $out" ;;
  *) _ok "another host's lock is never broken" ;;
esac
assert_eq "and the run gives up instead" "3" "$rc"
rm -rf "$LOCK"

# --- two real runs, racing --------------------------------------------------
# Not a fabricated lock: two actual gates.sh processes, the way MC-026 had them.
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'; sleep 5
evidence | unit | Tests +[1-9][0-9]* passed
EOF
story "$FIX" T-1 GATES </dev/null
set_phase "$FIX" GATES

( cd "$FIX" && bash scripts/gates.sh >"$FIX/a.out" 2>&1 ) & RACER=$!
sleep 2
out="$(gates --no-wait)"; rc=$?
assert_eq "the second of two real runs is refused" "3" "$rc"
assert_contains "naming the first"                 "Another gate run is already in progress" "$out"
wait "$RACER"
assert_contains "and the first one records its result" \
  "recorded in docs/backlog/stories/T-1.md" "$(cat "$FIX/a.out")"

# One run, one record. The MC-026 story file carried three runs' worth.
runs="$(grep -c '^ *run: ' "$FIX/docs/backlog/stories/T-1.md")"
assert_eq "the story holds exactly one gate record" "1" "$runs"

# ---------------------------------------------------------------------------
describe "the record: an older run does not overwrite a newer one"

# Belt to the lock's braces, for what the lock cannot see: a lock broken as
# stale while its owner was merely unresponsive, a run that started before the
# lock existed, a --story pointed at a file another checkout is recording into.
# In MC-026 the losing process wrote last, so the story ended up claiming a
# failure that the winning run had already disproved.
write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
story "$FIX" T-1 GATES </dev/null
MARKER='<!-- gates.sh: written by bash scripts/gates.sh; do not edit or paste by hand -->'
{
  sed -e '/^## Gate results/q' "$FIX/docs/backlog/stories/T-1.md"
  printf '\n%s\n\n' "$MARKER"
  printf '    run:    2099-01-01T00:00:00Z\n    commit: deadbee\n    tree:   abc123\n'
  printf '    result: pass (1 ran, 0 unconfigured, 0 known)\n    PASS         unit (0s)\n\n'
  printf '## Notes\n'
} > "$FIX/t1.new" && mv "$FIX/t1.new" "$FIX/docs/backlog/stories/T-1.md"

out="$(gates)"; rc=$?
assert_contains "the older run declines to record"  "not recorded" "$out"
assert_contains "saying whose record stands"        "2099-01-01T00:00:00Z" "$out"
assert_eq "and the gates' own verdict is unchanged" "0" "$rc"
assert_contains "the newer record is left alone" "2099-01-01T00:00:00Z" \
  "$(cat "$FIX/docs/backlog/stories/T-1.md")"
runs="$(grep -c '^ *run: ' "$FIX/docs/backlog/stories/T-1.md")"
assert_eq "and was not appended to"  "1" "$runs"

# An equally-old or older record is this run's to replace: same-second re-runs
# are ordinary, and refusing them would make the record unwritable.
story "$FIX" T-1 GATES </dev/null
{
  sed -e '/^## Gate results/q' "$FIX/docs/backlog/stories/T-1.md"
  printf '\n%s\n\n' "$MARKER"
  printf '    run:    2020-01-01T00:00:00Z\n    commit: deadbee\n    tree:   abc123\n'
  printf '    result: fail (1 required gate(s) failed)\n\n## Notes\n'
} > "$FIX/t1.new" && mv "$FIX/t1.new" "$FIX/docs/backlog/stories/T-1.md"
out="$(gates)"
assert_contains "an older record is replaced" "recorded in docs/backlog/stories/T-1.md" "$out"
case "$(cat "$FIX/docs/backlog/stories/T-1.md")" in
  *2020-01-01T00:00:00Z*) _bad "and the stale one is gone" "2020 record survived" ;;
  *) _ok "and the stale one is gone" ;;
esac
set_phase "$FIX" ""

# ---------------------------------------------------------------------------
describe "a gate that reads its stdin cannot delete the gates after it"

# MC-046, the user's report. The gate loop read project.conf on its own stdin
# and ran each gate inside it without redirecting stdin, so a gate that ran
# `cat` swallowed the rest of the manifest. The loop saw EOF and stopped: every
# later gate vanished from the run, the summary and the count, and a required
# gate that exits 1 was certified "All required gates passed", exit 0.
#
# gates_isolated runs gates.sh with stdin at end-of-input and descriptors 3-9
# closed, so that a gate which drains a descriptor can only ever reach what
# gates.sh itself gave it - never this test process's terminal or pipes, which
# would block the suite instead of failing it. AC-3 is the test about what the
# caller's stdin can reach; these are about the manifest.
gates_isolated() {
  ( cd "$FIX" && bash scripts/gates.sh "$@" 2>&1 ) </dev/null 3<&- 4<&- 5<&- 6<&- 7<&- 8<&- 9<&-
}

# assert_gate_failed <what> <id> <output>   The summary line for a required gate
# that exited 1, by id: `FAIL         <id> (<n>s, exit 1) -> <its log>`. The
# duration is the one field not pinned.
assert_gate_failed() {
  case "$3" in
    *"FAIL         $2 ("*"s, exit 1) -> .claude/state/gate-logs/$2.log"*) _ok "$1" ;;
    *) _bad "$1" "expected a summary line: FAIL         $2 (<n>s, exit 1) -> .claude/state/gate-logs/$2.log
actual:   $3" ;;
  esac
}

# assert_lacks <what> <needle> <haystack>
assert_lacks() {
  case "$3" in
    *"$2"*) _bad "$1" "expected NOT to contain: $2
actual:                   $3" ;;
    *) _ok "$1" ;;
  esac
}

# AC-1(a): the reported bug. Today: `later` is absent, "(1 ran, ...)", exit 0.
write_conf "$FIX" <<'EOF'
gate     | eats  | required | . | cat >/dev/null; printf 'Tests 1 passed\n'
gate     | later | required | . | printf 'Tests 2 passed\n'
evidence | eats  | Tests [1-9][0-9]* passed
evidence | later | Tests [1-9][0-9]* passed
EOF
out="$(gates_isolated)"; rc=$?
assert_contains "the gate after a stdin-reading gate still runs" "=== gate: later (required) ===" "$out"
assert_contains "the stdin-reading gate passes"          "PASS         eats (" "$out"
assert_contains "and so does the gate after it"          "PASS         later (" "$out"
assert_contains "and both are counted as having run" \
  "All required gates passed (2 ran, 0 unconfigured, 0 known)." "$out"
assert_eq "a run of two passing gates exits 0" "0" "$rc"

# AC-1(b): worse than reported. Today: exit 0, "All required gates passed".
AC1B_CONF="gate     | eats  | required | . | cat >/dev/null; printf 'Tests 1 passed\n'
gate     | later | required | . | exit 1
evidence | eats  | Tests [1-9][0-9]* passed"
printf '%s\n' "$AC1B_CONF" | write_conf "$FIX"
out="$(gates_isolated)"; rc=$?
assert_gate_failed "a failing required gate after a stdin-reading gate is reported FAIL" later "$out"
assert_contains "and the run says a required gate failed" "1 required gate(s) failed." "$out"
assert_lacks "and does not certify the run" "All required gates passed" "$out"
assert_eq "and exits 1" "1" "$rc"

# ---------------------------------------------------------------------------
describe "a gate cannot reach the manifest through any inherited descriptor"

# AC-2. The obvious fix - read the manifest on fd 3 instead of fd 0 - passes
# AC-1 and relocates the defect: the eval'd command inherits fd 3 and a gate
# that reads it drains the manifest exactly as before. This gate reads 0 and
# every descriptor 3 to 9 an ordinary command can reach by accident.
write_conf "$FIX" <<'EOF'
gate     | eats  | required | . | for fd in 0 3 4 5 6 7 8 9; do cat <&$fd >/dev/null; done 2>/dev/null; printf 'Tests 1 passed\n'
gate     | later | required | . | exit 1
evidence | eats  | Tests [1-9][0-9]* passed
EOF
out="$(gates_isolated)"; rc=$?
assert_gate_failed "a gate after one that drains fds 0 and 3-9 still runs and is reported FAIL" later "$out"
assert_contains "and the run says a required gate failed" "1 required gate(s) failed." "$out"
assert_lacks "and does not certify the run" "All required gates passed" "$out"
assert_eq "and exits 1" "1" "$rc"

# ---------------------------------------------------------------------------
describe "a gate reads end-of-input, not the manifest and not the caller's stdin"

# AC-3. Today the gate reads the rest of the manifest. Under the fd-3-only fix
# it reads whatever gates.sh was given - here a pipe, in a terminal the TTY,
# where it would block forever. The pipe is finite, so this test cannot block
# whichever of those it meets; it fails instead.
write_conf "$FIX" <<'EOF'
gate  | count | required | . | printf 'stdin-bytes=%s\n' "$(wc -c | tr -d ' ')"
gate  | after | required | . | printf 'Tests 2 passed\n'
EOF
rm -f "$FIX/.claude/state/gate-logs/count.log"
out="$(printf 'CALLER-DATA\n' | ( cd "$FIX" && bash scripts/gates.sh 2>&1 ))"
assert_contains "a run given data on stdin still finishes" "--- gate summary ---" "$out"
seen="$(grep '^stdin-bytes=' "$FIX/.claude/state/gate-logs/count.log" 2>/dev/null)"
assert_eq "a gate reads zero bytes of stdin, whatever gates.sh was given" "stdin-bytes=0" "$seen"

# ---------------------------------------------------------------------------
describe "--fast and --required give the same guarantee as a full run"

# AC-4. The subsets run the same loop. (--gate <id> is deliberately absent: a
# filtered-out gate hits `continue` before anything runs, so it has no defect.)
printf '%s\n%s\n%s\n' "$AC1B_CONF" \
  "gate     | slow_one | required | . | printf 'Tests 3 passed\n'" \
  "slow     | slow_one | stands in for a release build; RED has no use for it" \
  | write_conf "$FIX"
out="$(gates_isolated --fast)"; rc=$?
assert_gate_failed "--fast: a failing gate after a stdin-reading gate is reported FAIL" later "$out"
assert_contains "--fast: the slow gate after them is still seen, and skipped" "--fast skipped: slow_one" "$out"
assert_contains "--fast: the run says a required gate failed" "1 required gate(s) failed." "$out"
assert_eq "--fast: and exits 1" "1" "$rc"

printf '%s\n' "$AC1B_CONF" | write_conf "$FIX"
out="$(gates_isolated --required)"; rc=$?
assert_gate_failed "--required: a failing gate after a stdin-reading gate is reported FAIL" later "$out"
assert_contains "--required: the run says a required gate failed" "1 required gate(s) failed." "$out"
assert_eq "--required: and exits 1" "1" "$rc"

summary "gates"
