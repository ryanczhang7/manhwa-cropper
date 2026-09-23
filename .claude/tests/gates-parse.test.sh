#!/usr/bin/env bash
# Tests for how scripts/gates.sh PARSES project.conf (MC-045).
#
# --list and --audit read project.conf and print it; they run no gate. Before
# MC-045 both parse loops forked a `printf | sed` for every line of the file,
# comments included, and several more `$(trim "$(printf | cut)")` pairs per
# content line, so parsing this repository's 295-line project.conf cost 1-3
# minutes on a machine where a fork costs ~0.15 s.
#
# Wall-clock time on that machine moves by a factor of two between runs, so
# nothing here reads a clock. Two deterministic process counts are used
# instead, from ONE invocation of gates.sh per config per mode:
#
#   execs   - processes gates.sh execs through PATH, counted by a directory of
#             logging shims prepended to PATH. Blind to a call made by absolute
#             path.
#   nested  - lines of a `bash -x` trace at depth >= 2 (prefix `++`): commands
#             run inside a command substitution or subshell. Sees a fork that
#             execs nothing, e.g. `$(trim "$line")` with a pure-bash trim, and
#             an absolute-path call made inside `$(...)`. Blind to a top-level
#             pipeline with no substitution, which the shim sees.
#
# Each oracle covers the other's blind spot, which is why AC-1 and AC-2 demand
# both. The xtrace goes to its own fd (BASH_XTRACEFD) so that gates.sh's stdout
# and stderr stay exactly what a user sees, and can be compared byte for byte.
#
# Slow on purpose-built hardware and slower here: under the pre-MC-045 parser
# every instrumented invocation costs tens of seconds on the Windows dev
# machine. A 400 s timeout on this suite is not a hang.
#
#   bash scripts/selftest.sh gates-parse
#   GATES_PARSE_ONLY=ac3 bash scripts/selftest.sh gates-parse   (one AC: ac1|ac2|ac3)

. "$(dirname "${BASH_SOURCE[0]}")/_lib.sh"

ONLY_AC="${GATES_PARSE_ONLY:-}"
[ -n "$ONLY_AC" ] && printf '  note: GATES_PARSE_ONLY=%s - the other criteria are NOT being checked\n' "$ONLY_AC"
want() { [ -z "$ONLY_AC" ] || [ "$ONLY_AC" = "$1" ]; }

FIX="$(make_project_fixture)"
WORK="$(mktemp -d 2>/dev/null || mktemp -d -t gates-parse)"
trap 'rm -rf "$FIX" "$WORK"' EXIT
T=$(printf '\t')

# --- the exec shim -----------------------------------------------------------
# Real paths are resolved BEFORE anything is prepended to PATH, so a shim execs
# the binary and never itself. `bash` is deliberately not shimmed.
SHIMS="$WORK/shims"
mkdir -p "$SHIMS"
for n in sed cut tr grep head tail date hostname git awk mkdir cat wc sort \
         basename dirname rm mv cp ls find uname mktemp env sleep tee xargs \
         od readlink realpath expr uniq; do
  real="$(command -v "$n" 2>/dev/null)" || continue
  case "$real" in /*) ;; *) continue ;; esac
  printf '#!/bin/sh\nprintf "%%s\\n" %s >> "$GATES_PARSE_SHIM_LOG"\nexec '"'"'%s'"'"' "$@"\n' \
    "$n" "$real" > "$SHIMS/$n"
  chmod +x "$SHIMS/$n"
done

# measure <mode>   Runs `gates.sh <mode>` against the fixture's current
# project.conf, once, under both oracles. Sets:
#   M_EXECS  number of shimmed execs
#   M_NESTED number of trace lines at depth >= 2
#   M_OUT    stdout, M_ERR stderr, M_RC exit code
measure() {
  local log="$WORK/execs.log" trace="$WORK/trace.log"
  : > "$log"; : > "$trace"
  (
    cd "$FIX" || exit 99
    exec 9> "$trace"
    GATES_PARSE_SHIM_LOG="$log" PATH="$SHIMS:$PATH" BASH_XTRACEFD=9 PS4='+ ' \
      bash -x scripts/gates.sh "$1" > "$WORK/out" 2> "$WORK/err"
  )
  M_RC=$?
  M_OUT="$(cat "$WORK/out")"
  M_ERR="$(cat "$WORK/err")"
  M_EXECS=$(wc -l < "$log" | tr -d '[:space:]')
  M_NESTED=$(grep -c '^++' "$trace")
}

# plain <mode>   gates.sh with no instrumentation. Sets P_OUT, P_ERR, P_RC.
plain() {
  ( cd "$FIX" && bash scripts/gates.sh "$1" > "$WORK/out" 2> "$WORK/err" )
  P_RC=$?
  P_OUT="$(cat "$WORK/out")"
  P_ERR="$(cat "$WORK/err")"
}

# ===========================================================================
if want ac1; then
describe "AC-1: comment and blank lines cost nothing to parse"

AC1_BODY='gate     | unit | required | . | printf '"'"'Tests  47 passed (47)\n'"'"'
evidence | unit | Tests +[1-9][0-9]* passed'

# The same two content lines with 100 comment, blank, whitespace-only and
# indented-comment lines around AND between them - interleaved, not only
# appended, so a parser that stops early at the last content line cannot
# pass. 40 before the gate line, 30 between, 30 after.
pad() { # <count> <tag>
  local i=1
  while [ "$i" -le "$1" ]; do
    case $((i % 5)) in
      0) printf '# %s comment %d: gate | fake | required | . | nothing\n' "$2" "$i" ;;
      1) printf '\n' ;;
      2) printf '   # %s indented comment %d\n' "$2" "$i" ;;
      3) printf '%s#%s tab-indented comment %d\n' "$T" "$2" "$i" ;;
      4) printf '  %s  \n' "$T" ;;
    esac
    i=$((i+1))
  done
}
AC1_PADDED="$(pad 40 before)
gate     | unit | required | . | printf 'Tests  47 passed (47)\n'
$(pad 30 between)
evidence | unit | Tests +[1-9][0-9]* passed
$(pad 30 after)"
n_pad=$(printf '%s\n' "$AC1_PADDED" | grep -cv '^[a-z]')
assert_eq "the padded fixture really adds 100 comment/blank lines" "100" "$n_pad"

for mode in --list --audit; do
  printf '%s\n' "$AC1_BODY" | write_conf "$FIX"
  measure "$mode"
  b_execs=$M_EXECS; b_nested=$M_NESTED; b_out=$M_OUT; b_err=$M_ERR; b_rc=$M_RC
  printf '%s\n' "$AC1_PADDED" | write_conf "$FIX"
  measure "$mode"
  printf '    measured %s: execs %s -> %s, nested %s -> %s (unpadded -> padded)\n' \
    "$mode" "$b_execs" "$M_EXECS" "$b_nested" "$M_NESTED"

  assert_eq "gates.sh $mode: the exec shim saw the unpadded run at all" "yes" \
    "$([ "$b_execs" -gt 0 ] && echo yes || echo "no ($b_execs)")"
  assert_eq "gates.sh $mode: the trace was captured for the unpadded run" "yes" \
    "$([ "$b_nested" -gt 0 ] && echo yes || echo "no ($b_nested)")"
  assert_eq "gates.sh $mode execs no extra process for 100 comment/blank lines" "$b_execs" "$M_EXECS"
  assert_eq "gates.sh $mode runs no extra nested command for 100 comment/blank lines" "$b_nested" "$M_NESTED"
  assert_eq "gates.sh $mode prints byte-identical stdout with the padding" "$b_out" "$M_OUT"
  assert_eq "gates.sh $mode prints byte-identical stderr with the padding" "$b_err" "$M_ERR"
  assert_eq "gates.sh $mode exits the same with the padding" "$b_rc" "$M_RC"
  assert_eq "gates.sh $mode passes on the AC-1 fixture (a clean parse, not an early exit)" "0" "$M_RC"
done
fi

# ===========================================================================
if want ac2; then
describe "AC-2: gate lines cost nothing to parse"

# One gate carrying evidence plus every table kind that can sit on a single
# gate without failing the audit: floor (digit-bearing evidence), waiver (so the
# gate is optional), slow and ci-factor, each with its reason or source. A
# `no-count` cannot share a gate with a `floor` (the audit refuses it) and
# cannot name a missing gate (the audit refuses that too), so the ten-gate
# config is where it appears. The ten-gate config is the one-gate config plus
# nine gates, each with evidence and between them every kind again, so the
# tables grow with the gate count as well - table parsing and table_lookup
# are both measured scaling, not just the gate loop.
AC2_ONE='gate      | g01 | optional | . | printf '"'"'Tests  47 passed (47)\n'"'"'
evidence  | g01 | Tests +[1-9][0-9]* passed
floor     | g01 | 40
waiver    | g01 | flaky on the fixture runner, see MC-000
slow      | g01 | builds a release bundle
ci-factor | g01 | 3.4 | measured from CI run 12345'

AC2_TEN="$AC2_ONE"
i=2
while [ "$i" -le 10 ]; do
  id=$(printf 'g%02d' "$i")
  AC2_TEN="$AC2_TEN
gate      | $id | required | . | printf 'Tests  47 passed (47)\n'"
  case $((i % 5)) in
    0) AC2_TEN="$AC2_TEN
evidence  | $id | Finished .* profile
no-count  | $id | the digits after it are elapsed time, not work" ;;
    1) AC2_TEN="$AC2_TEN
evidence  | $id | Tests +[1-9][0-9]* passed
floor     | $id | 10
slow      | $id | instrumented" ;;
    2) AC2_TEN="$AC2_TEN
evidence  | $id | Tests +[1-9][0-9]* passed|Examples +[0-9]+
ci-factor | $id | 2.5 | measured from CI run 777" ;;
    3) AC2_TEN="$AC2_TEN
evidence  | $id | Tests +[1-9][0-9]* passed
floor     | $id | 5" ;;
    4) AC2_TEN="$AC2_TEN
evidence  | $id | -" ;;
  esac
  i=$((i+1))
done
# The one optional gate with a waiver stays g01; required gates may not carry one.
n_gates=$(printf '%s\n' "$AC2_TEN" | grep -c '^gate ')
assert_eq "the ten-gate fixture declares ten gates" "10" "$n_gates"
for k in evidence floor waiver slow ci-factor no-count; do
  assert_contains "the ten-gate fixture has a $k line" "
$k " "
$AC2_TEN"
done

for mode in --list --audit; do
  printf '%s\n' "$AC2_ONE" | write_conf "$FIX"
  measure "$mode"
  o_execs=$M_EXECS; o_nested=$M_NESTED; o_rc=$M_RC
  printf '%s\n' "$AC2_TEN" | write_conf "$FIX"
  measure "$mode"
  printf '    measured %s: execs %s -> %s, nested %s -> %s (1 gate -> 10 gates)\n' \
    "$mode" "$o_execs" "$M_EXECS" "$o_nested" "$M_NESTED"

  # Mechanical guard: a shim that intercepts nothing, or a trace that was never
  # written, gives 0 == 0 and would pass the equality below vacuously.
  assert_eq "gates.sh $mode: the exec shim saw the one-gate run at all" "yes" \
    "$([ "$o_execs" -gt 0 ] && echo yes || echo "no ($o_execs)")"
  assert_eq "gates.sh $mode: the trace was captured for the one-gate run" "yes" \
    "$([ "$o_nested" -gt 0 ] && echo yes || echo "no ($o_nested)")"
  assert_eq "gates.sh $mode passes on the one-gate fixture (a clean parse, not an early continue)" "0" "$o_rc"
  assert_eq "gates.sh $mode passes on the ten-gate fixture (a clean parse, not an early continue)" "0" "$M_RC"
  [ "$mode" = --audit ] && assert_contains "the ten-gate audit reached every gate" "ok   g10" "$M_OUT"
  assert_eq "gates.sh $mode execs no extra process for nine more gates" "$o_execs" "$M_EXECS"
  assert_eq "gates.sh $mode runs no extra nested command for nine more gates" "$o_nested" "$M_NESTED"
done
fi

# ===========================================================================
if want ac3; then
describe "AC-3: the parse means exactly what it meant"

# Every edge the pre-MC-045 trim/cut parse handles, in one config. Written
# with printf so the tabs and CRs are real bytes. `W` is leading/trailing
# whitespace of both kinds; most lines end in CRLF, a few in plain LF.
W=" $T "
{
  printf 'BOOTSTRAPPED=yes\r\n'
  printf '# a comment\r\n'
  printf '   # an indented comment | with | pipes | in it\r\n'
  printf '%s# a tab-indented comment | gate | ghost | required | . | x\r\n' "$T"
  printf '\r\n'
  printf '%s\r\n' "$W"
  printf 'this line has no pipe at all\r\n'
  printf 'frobnicate | unit | an unknown kind is ignored\r\n'
  # Whitespace around every field, a cmd containing `|` (relies on -f5-) and an
  # internal tab and double space that must survive the trim.
  printf '%sgate%s|%sunit%s|%srequired%s|%ssrc%s|%sprintf "a|b"  |%scat%s\r\n' \
    "$W" "$W" "$W" "$W" "$W" "$W" "$W" "$W" "$W" "$T" "$W"
  # An evidence regex with top-level alternation (relies on -f3-).
  printf '%sevidence%s|%sunit%s|%sTests +[1-9][0-9]* passed|Examples +[0-9]+%s\r\n' \
    "$W" "$W" "$W" "$W" "$W" "$W"
  printf 'floor|unit|%s40%s\r\n' "$T" "$T"
  printf 'ci-factor | unit |%s3.4%s|%smeasured from CI run 12345 | job coverage%s\r\n' \
    "$T" "$W" "$W" "$T"
  printf 'slow  |  unit  |  %sruns under instrumentation%s\n' "$T" "$T"
  # An empty cwd field becomes `.`; an optional gate with a waiver.
  printf 'gate | lint | optional |%s| printf "Checked 3 files"\r\n' "$W"
  printf 'evidence | lint | Checked [1-9][0-9]* files\r\n'
  printf 'waiver | lint |  known to fail on Windows  %s\r\n' "$T"
  printf 'no-count | lint | %sthe count is a file total%s\r\n' "$T" "$T"
  # A ci-factor with no `|`-separated source at all, and one whose source is
  # only whitespace: both are "no source".
  printf 'gate | build | optional | . | printf "Finished release profile"\r\n'
  printf 'evidence | build | Finished .* profile\r\n'
  printf 'ci-factor | build | 2.0%s\r\n' "$T"
  printf 'gate | docs | optional | . | printf "built 4 pages"\n'
  printf 'evidence | docs | built [0-9]+ pages\n'
  printf 'ci-factor | docs | 1.5 |%s\r\n' "$W"
  # An empty id in a table line is skipped; an unconfigured optional gate.
  printf 'evidence |%s| orphan regex\r\n' "$W"
  printf 'gate | e2e | optional | . |%s\r\n' "$W"
} > "$FIX/.claude/harness/project.conf"

AC3_LIST_EXPECTED='unit         required  src    printf "a|b"  |<TAB>cat
                              evidence: Tests +[1-9][0-9]* passed|Examples +[0-9]+
                              floor:    40
                              ci-factor: 3.4 <TAB> | <TAB> measured from CI run 12345 | job coverage
                              slow:     runs under instrumentation (left out of --fast)
lint         optional  .      printf "Checked 3 files"
                              evidence: Checked [1-9][0-9]* files
                              no-count: the count is a file total (no floor possible)
                              waiver:   known to fail on Windows
build        optional  .      printf "Finished release profile"
                              evidence: Finished .* profile
                              ci-factor: 2.0
docs         optional  .      printf "built 4 pages"
                              evidence: built [0-9]+ pages
                              ci-factor: 1.5 |
e2e          optional  .      <unconfigured>
                              evidence: <none>'
AC3_AUDIT_EXPECTED='ok   unit         evidence: Tests +[1-9][0-9]* passed|Examples +[0-9]+
                  floor:  40
                  ci-factor: 3.4 <TAB> | <TAB> measured from CI run 12345 | job coverage
                  slow:   runs under instrumentation
ok   lint         evidence: Checked [1-9][0-9]* files
                  no-count: the count is a file total (reports `observed -`; no floor possible)
                  waiver: known to fail on Windows
FAIL build        ci-factor has no source; say which CI run it was measured from
FAIL docs         ci-factor has no source; say which CI run it was measured from
ok   e2e          (unconfigured)

2 manifest problem(s).'

plain --list
assert_eq "--list prints the edge-case config exactly as before MC-045" \
  "${AC3_LIST_EXPECTED//<TAB>/$T}" "$P_OUT"
assert_eq "--list writes nothing to stderr for the edge-case config" "" "$P_ERR"
assert_eq "--list exits 0 for the edge-case config" "0" "$P_RC"

plain --audit
assert_eq "--audit reports the edge-case config exactly as before MC-045" \
  "${AC3_AUDIT_EXPECTED//<TAB>/$T}" "$P_OUT"
assert_eq "--audit writes nothing to stderr for the edge-case config" "" "$P_ERR"
assert_eq "--audit exits 1 for the edge-case config (two sourceless ci-factors)" "1" "$P_RC"
fi

summary gates-parse
