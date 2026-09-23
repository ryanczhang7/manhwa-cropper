#!/usr/bin/env bash
# Run the project's quality gates as declared in .claude/harness/project.conf.
#
#   bash scripts/gates.sh                  run every gate; record the result in the active story
#   bash scripts/gates.sh --story WORLD-3  ... and record it in that story instead
#   bash scripts/gates.sh --list           show what is configured
#   bash scripts/gates.sh --gate unit      run one gate  (not recorded: a partial run is not evidence)
#   bash scripts/gates.sh --required       required gates only  (not recorded)
#   bash scripts/gates.sh --fast           every gate not marked `slow`  (not recorded)
#   bash scripts/gates.sh --audit          check the manifest itself, run nothing
#   bash scripts/gates.sh --no-wait        refuse rather than queue behind another run
#
# Only one run at a time, per checkout. Everything that RUNS a gate takes an
# exclusive lock first and waits for one already in progress; --list and
# --audit execute nothing and take nothing. Exit 3 means the lock was not
# obtained, which is neither a pass nor a failure of any gate.
#
# The gate NAMES are stable across every project ("the coverage gate"); the
# COMMANDS behind them are per-stack. That indirection is what lets the same
# agents drive a Python service, a TypeScript app or a Godot game.
#
# Exit 0 is not proof that a gate did any work: a test runner that discovers no
# tests, or a linter pointed at an empty directory, exits 0 with nothing to say.
# `evidence` lines assert that work was OBSERVED, not that it succeeded.
# `floor` lines go further and assert HOW MUCH: the number the evidence regex
# matched must not fall below a recorded minimum, so a suite that quietly
# shrinks from 47 tests to 3 fails instead of passing faster. `no-count` lines
# say the opposite about a gate - that its evidence regex proves liveness but
# measures nothing, so the digits sitting under it are not a count of work and
# a floor on them would be nonsense. Such a gate reports `observed -`, and a
# floor declared on it is refused. `waiver` lines name an optional gate that is
# known to fail, and why, so that WARN in the summary always means something
# changed. All four are described in the quality-gates skill.
#
# `slow` lines name the gates a --fast run leaves out. --fast exists so that RED
# and GREEN can ask the gates whether the tests are even ADMISSIBLE - lint, types,
# and the instrumented test command they will actually be judged by - without
# paying for a release bundle on every loop. A gate is fast unless something says
# otherwise, so the subset is right by default and wrong only where someone said
# so out loud. A --fast run is never recorded: it is not a full run.
#
# A full run writes its own summary into the story's ## Gate results, stamped
# with the commit and a hash of the code it ran against. Nobody pastes it.

set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONF="$ROOT/.claude/harness/project.conf"
LOGDIR="$ROOT/.claude/state/gate-logs"
STAMP="$ROOT/.claude/state/last-gate-run"

export CLAUDE_PROJECT_DIR="$ROOT"
. "$ROOT/.claude/hooks/lib.sh"

[ -f "$CONF" ] || { printf 'error: missing %s\n' "$CONF" >&2; exit 1; }
mkdir -p "$LOGDIR"

BOOTSTRAPPED="$(grep -E '^BOOTSTRAPPED=' "$CONF" | head -1 | cut -d= -f2- | tr -d '[:space:]')"
[ -z "$BOOTSTRAPPED" ] && BOOTSTRAPPED=no

ONLY=""; REQUIRED_ONLY=0; LIST=0; AUDIT=0; STORY=""; FAST=0; NOWAIT=0
ARGV_DESC="gates.sh${*:+ $*}"
while [ $# -gt 0 ]; do
  case "$1" in
    --list) LIST=1 ;;
    --gate) shift; ONLY="${1:-}" ;;
    --required) REQUIRED_ONLY=1 ;;
    --fast) FAST=1 ;;
    --audit) AUDIT=1 ;;
    --no-wait) NOWAIT=1 ;;
    --story) shift; STORY="${1:-}" ;;
    -h|--help) sed -n '2,43p' "$0"; exit 0 ;;
    *) printf 'unknown option: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

# When this run started, not when it finishes. The record carries it, and a run
# that started earlier will not overwrite a record a later one already wrote -
# see record_in_story.
RUN_STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

trim() { printf '%s' "$1" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'; }

TAB=$(printf '\t')
ESC=$(printf '\033')

# --- one run at a time --------------------------------------------------------
# Two gate runs on one checkout are not independent. They share the build
# directory - `target/`, `node_modules/.vite`, `__pycache__` - and under
# coverage instrumentation they fight over the same profraw counters. Then they
# both rewrite the story's ## Gate results and the last writer wins, which is
# not the same as the right one winning. During MC-026 an orchestrator and a
# subagent it had dispatched ran at once: the story ended up recording
# `result: fail` against tree 2b4248021a96, then `result: pass` against the
# same tree, with three runs' durations interleaved, while the coverage log on
# disk held a complete table at 99.71% against a floor of 95. Nothing had
# failed. Only check-boundaries.sh comparing the recorded tree hash stood
# between that record and a PR, and that is the last line, not the first.
#
# So every invocation that RUNS something holds an exclusive lock for the whole
# run, recording included. That covers --gate and --fast as much as a full run:
# a single gate shares `target/` with everything else, and a partial run that
# corrupts a full one's counters is the same bug wearing a smaller hat.
# --list and --audit execute no gate command and take no lock.
#
# mkdir, not flock: flock is missing on macOS and on Git Bash, and the harness
# is bash/awk/coreutils by rule (see .claude/harness/rules.md, "Portability").
# `mkdir` fails on a path that exists and creates one that does not, in a single
# atomic step, on every filesystem that matters here.
LOCKDIR="$ROOT/.claude/state/gate-run.lock"
LOCK_HELD=0
LOCK_HOST="$(hostname 2>/dev/null || printf '%s' "${HOSTNAME:-unknown}")"
LOCK_WAIT="${GATES_LOCK_WAIT:-1800}"
LOCK_POLL=2

release_lock() {
  [ "$LOCK_HELD" = 1 ] || return 0
  LOCK_HELD=0
  rm -rf "$LOCKDIR"
}

lock_field() { # <owner text> <field>
  printf '%s\n' "$1" | sed -n "s/^$2:[[:space:]]*//p" | head -1
}

# acquire_lock   Take the lock, or wait for the run that holds it.
#
# A lock whose owner process is gone, on this host, is broken and retaken: the
# way one gets left behind is a hard kill, and a harness that wedges until
# somebody reads a message about `rm -rf` teaches people to `rm -rf` first and
# read later. A lock owned by a live process - or by another host, whose
# process table this one cannot see - is waited for and never broken. It is
# taken over by renaming first, so that two waiters deciding the same lock is
# stale cannot both remove it and have the loser delete the winner's new lock.
acquire_lock() {
  local waited=0 owner pid host announced=0 stolen=0 seen_empty=0
  while :; do
    if mkdir "$LOCKDIR" 2>/dev/null; then
      LOCK_HELD=1
      printf 'pid:     %s\nhost:    %s\nstarted: %s\ncommand: %s\n' \
        "$$" "$LOCK_HOST" "$RUN_STARTED_AT" "$ARGV_DESC" > "$LOCKDIR/owner"
      trap 'release_lock' EXIT
      trap 'release_lock; exit 130' INT
      trap 'release_lock; exit 143' TERM
      [ "$announced" = 1 ] && printf 'The other gate run finished; starting.\n' >&2
      return 0
    fi

    owner="$(cat "$LOCKDIR/owner" 2>/dev/null)"
    pid="$(lock_field "$owner" pid)"
    host="$(lock_field "$owner" host)"

    # A lock with no owner file names nobody to wait for. The only way to make
    # one is to die in the microseconds between `mkdir` and the write that
    # follows it - so it is always debris, but "always" is worth one poll of
    # evidence before acting on it. Without this, that microsecond crash costs
    # the next run the whole GATES_LOCK_WAIT for no reason.
    if [ -z "$owner" ]; then
      if [ "$seen_empty" = 1 ]; then
        printf 'note: the gate run lock names no owner, so nothing holds it. Taking it over.\n' >&2
        if mv "$LOCKDIR" "$LOCKDIR.stale.$$" 2>/dev/null; then rm -rf "$LOCKDIR.stale.$$"; fi
        seen_empty=0
        continue
      fi
      seen_empty=1
      sleep "$LOCK_POLL"
      waited=$((waited + LOCK_POLL))
      continue
    fi
    seen_empty=0

    if [ "$stolen" = 0 ] && [ -n "$pid" ] && [ "$host" = "$LOCK_HOST" ] \
       && ! kill -0 "$pid" 2>/dev/null; then
      stolen=1
      printf 'note: the gate run lock was left behind by pid %s, which is gone. Taking it over.\n' "$pid" >&2
      if mv "$LOCKDIR" "$LOCKDIR.stale.$$" 2>/dev/null; then rm -rf "$LOCKDIR.stale.$$"; fi
      continue
    fi

    if [ "$announced" = 0 ]; then
      announced=1
      printf '\nAnother gate run is already in progress on this checkout:\n' >&2
      printf '%s\n' "$owner" | sed 's/^/    /' >&2
      printf 'They would share the build directory and both rewrite the story record.\n' >&2
      if [ "$NOWAIT" = 1 ]; then
        printf 'Refusing to start a second one (--no-wait). Re-run when it finishes.\n' >&2
        exit 3
      fi
      printf 'Waiting for it to finish (up to %ss; set GATES_LOCK_WAIT to change).\n' "$LOCK_WAIT" >&2
    fi

    if [ "$waited" -ge "$LOCK_WAIT" ]; then
      printf 'error: gave up waiting for the gate run lock after %ss.\n' "$LOCK_WAIT" >&2
      printf 'Nothing ran, so this is neither a pass nor a failure. If that run is really\n' >&2
      printf 'gone, remove .claude/state/gate-run.lock and try again.\n' >&2
      exit 3
    fi
    sleep "$LOCK_POLL"
    waited=$((waited + LOCK_POLL))
    if [ $((waited % 60)) -eq 0 ]; then
      printf '  ... still waiting (%ss)\n' "$waited" >&2
    fi
  done
}

# --- evidence, floor, waiver and slow tables --------------------------------
# Read up front, so that --gate <id> still finds its own lines. Stored as
# "<id><TAB><value>" lines; no associative arrays, for bash 3.2.
EVIDENCE=""; WAIVERS=""; FLOORS=""; SLOWS=""; CIFACTORS=""; NOCOUNTS=""; GATE_IDS=""
while IFS= read -r line; do
  line="${line%%$'\r'}"
  case "$(trim "$line")" in ''|'#'*) continue ;; esac
  case "$line" in *'|'*) ;; *) continue ;; esac
  kind=$(trim "$(printf '%s' "$line" | cut -d'|' -f1)")
  tid=$(trim "$(printf '%s' "$line" | cut -d'|' -f2)")
  # Every gate id, so that --audit can tell a `slow` line naming a real gate
  # from one naming a typo. That distinction matters more here than for the
  # other tables: a misspelt `evidence` id makes its gate report "no evidence
  # line", and a misspelt `floor` id fails the audit outright, but a misspelt
  # `slow` id is silent - the gate it meant to exclude simply stays in --fast.
  [ "$kind" = "gate" ] && { GATE_IDS="$GATE_IDS $tid"; continue; }
  case "$kind" in evidence|waiver|floor|slow|ci-factor|no-count) ;; *) continue ;; esac
  # -f3- so that a regex containing `|` (alternation) survives the split.
  tval=$(trim "$(printf '%s' "$line" | cut -d'|' -f3-)")
  [ -n "$tid" ] || continue
  case "$kind" in
    evidence) EVIDENCE="$EVIDENCE$tid$TAB$tval
" ;;
    waiver)   WAIVERS="$WAIVERS$tid$TAB$tval
" ;;
    floor)    FLOORS="$FLOORS$tid$TAB$tval
" ;;
    slow)     SLOWS="$SLOWS$tid$TAB$tval
" ;;
    ci-factor) CIFACTORS="$CIFACTORS$tid$TAB$tval
" ;;
    no-count) NOCOUNTS="$NOCOUNTS$tid$TAB$tval
" ;;
  esac
done < "$CONF"

# A typo in --gate ran nothing and reported "All required gates passed (0
# ran)", exit 0. Nothing to run is not a pass.
if [ -n "$ONLY" ]; then
  case " $GATE_IDS " in
    *" $ONLY "*) ;;
    *) printf "error: no gate named '%s' in project.conf; see --list\n" "$ONLY" >&2; exit 2 ;;
  esac
fi

# table_lookup <table> <id>   Echoes the value. Exact string comparison, never
# a regex match: a gate id containing `.` or `*` must not silently adopt a
# different gate's line. Returns 1 when the id has no line.
table_lookup() {
  local eid ere
  while IFS="$TAB" read -r eid ere; do
    if [ "$eid" = "$2" ]; then printf '%s' "$ere"; return 0; fi
  done <<< "$1"
  return 1
}

# A gate's log as the regex should see it: no ANSI colour, no CR.
clean_log() {
  sed -e "s/${ESC}\[[0-9;]*[a-zA-Z]//g" -e 's/\r$//' "$1"
}

# work_count <log> <evidence regex>   How much work the gate was observed doing:
# the first run of digits at or after the start of the first evidence match.
#
# The evidence regex is the measurement, rather than a second regex, because
# there should be one description per gate of what "having done something"
# looks like. The match is `($2).*`, so the counted span ALWAYS runs to the end
# of the line: the whole number is inside it however early the regex stopped,
# and a regex that stops mid-number - the `[1-9]` in `test result: ok\. [1-9]`
# - reads the same count as one that spells the number out. The width of the
# evidence regex does not change the number read. (Nor is the `.*` idle: for
# `TOTAL` and `Finished .* profile` the match ends BEFORE the digits, and
# without it there would be nothing to count at all.)
#
# What the evidence regex DOES decide is which LINE supplies the count.
# `grep -oE -m1` takes the first line that matches, and a line it does not
# match is skipped entirely, so the leading digit class selects the line:
# `test result: ok\. [1-9][0-9]*` walks past every `test result: ok. 0` line
# and counts the first target that ran something, where `[0-9][0-9]*` would
# match that first `ok. 0` line and report 0. That is the property
# `floor | integration | 9` in project.conf rests on - see the note above it.
# Parenthesised, so that an evidence regex using top-level alternation
# (`a|b`) does not bind the trailing `.*` to its last branch alone.
#
# The sharper consequence is that this function ALWAYS returns whatever digits
# it finds, and cannot tell a count from a stopwatch. `Finished .* profile`
# proves cargo ran and measures nothing: the digits after it are the elapsed
# time, so a warm cache "observes 0" and a cold one "observes 2". That is not
# detectable from the regex either - `TOTAL` has no digit class and is followed
# by a real region count - so it is DECLARED, with a `no-count` line, and the
# callers below consult that instead of guessing. Nothing here changes: this
# function stays a dumb reader of digits, and its callers decide whether to
# believe them.
work_count() {
  clean_log "$1" \
    | grep -oE -m1 -- "($2).*" 2>/dev/null | head -1 \
    | grep -oE '[0-9]+' 2>/dev/null | head -1
}

# --- recording ----------------------------------------------------------------
# Replace the body of the story's "## Gate results" section with a block this
# script wrote: the marker check-boundaries.sh looks for, the UTC time, the
# commit, the tree hash of the code the gates saw, and the summary. If the
# section is missing (an older story file) it is appended.
#
# The lock above already keeps two gates.sh runs from interleaving here. This
# is the second half of the same guarantee, for the cases the lock cannot see:
# a lock broken as stale while its owner was merely unresponsive, a run started
# before the lock existed, a --story pointed at a file another checkout is also
# recording into. A record whose `run:` is LATER than this run's start is newer
# evidence than anything this run holds, so this run does not touch it.
# Returns 1 without writing when it declines; the caller says so out loud.
RECORD_SKIPPED=""
record_in_story() { # <story-file> <result-text> <summary-lines>
  local f="$1" res="$2" body="$3" commit dirty tree block prev pn cn
  prev="$(awk '
    /^## Gate results/ { inblock = 1; next }
    inblock && /^## / { exit }
    inblock && $1 == "run:" { print $2; exit }
  ' "$f")"
  # Compared as digits, not as strings: both are the same fixed UTC format, so
  # YYYYMMDDHHMMSS orders them exactly, and no locale's collation gets a vote.
  # A `run:` of some other shape compares as nothing and is overwritten, which
  # is right - it was not written by this script.
  pn="$(printf '%s' "$prev" | tr -cd '0-9')"
  cn="$(printf '%s' "$RUN_STARTED_AT" | tr -cd '0-9')"
  if [ -n "$pn" ] && [ "${#pn}" = "${#cn}" ] && [ "$pn" -gt "$cn" ]; then
    RECORD_SKIPPED="$prev"
    return 1
  fi
  commit="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || printf 'no commit')"
  dirty=""
  [ -z "$(git -C "$ROOT" status --porcelain -- . ':!docs' 2>/dev/null)" ] || dirty=" (working tree had uncommitted changes)"
  tree="$(gate_tree_hash)"
  block="$(printf '%s\n' \
    "$GATE_MARKER" \
    "" \
    "    run:    $RUN_STARTED_AT" \
    "    commit: $commit$dirty" \
    "    tree:   $tree" \
    "    result: $res" \
    "" \
    "$(printf '%s\n' "$body" | sed -e '/^[[:space:]]*$/d' -e 's/^/    /')")"
  grep -q '^## Gate results' "$f" || printf '\n## Gate results\n' >> "$f"
  # ENVIRON rather than -v: the block contains regexes with backslashes.
  BLK="$block" awk '
    /^## Gate results/ { print; print ""; print ENVIRON["BLK"]; print ""; skip=1; next }
    skip && /^## / { skip=0 }
    !skip { print }
  ' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
}

# --- gates this story requires of itself ------------------------------------
# `integration` is optional for the repo because it needs a browser, and a busy
# laptop should not block unrelated stories on it. But a story whose central
# claims are only ever checked there - "the canvas draws a non-blank first
# frame" - can pass every required gate while its evidence went unrun. So a
# story may escalate a gate for itself, in its frontmatter:
#
#     required_gates: [integration]
#
# Optional for the repo, binding for the story that depends on it.
if [ -z "$STORY" ]; then load_state; STORY="$STORY_ID"; fi
STORY_FILE="$ROOT/docs/backlog/stories/$STORY.md"
STORY_REQUIRES=""
if [ -n "$STORY" ] && [ -f "$STORY_FILE" ]; then
  STORY_REQUIRES=" $(frontmatter_list "$STORY_FILE" required_gates) "
fi

# Nothing below this line runs a gate command until the lock is held. --list
# and --audit walk the same loop but only read project.conf, so they are exempt
# - an audit should never queue behind a twenty-minute coverage run.
if [ "$LIST" = 0 ] && [ "$AUDIT" = 0 ]; then
  acquire_lock
fi

fails=0; warns=0; known=0; unconfigured=0; ran=0; noevidence=0; skipped=""
results=""

while IFS= read -r line; do
  line="${line%%$'\r'}"
  case "$(trim "$line")" in ''|'#'*) continue ;; esac
  case "$line" in *'|'*) ;; *) continue ;; esac

  kind=$(trim "$(printf '%s' "$line" | cut -d'|' -f1)")
  [ "$kind" = "gate" ] || continue
  id=$(trim   "$(printf '%s' "$line" | cut -d'|' -f2)")
  req=$(trim  "$(printf '%s' "$line" | cut -d'|' -f3)")
  cwd=$(trim  "$(printf '%s' "$line" | cut -d'|' -f4)")
  cmd=$(trim  "$(printf '%s' "$line" | cut -d'|' -f5-)")
  [ -z "$cwd" ] && cwd="."

  # An escalation makes the gate required for everything below, and says so
  # wherever the gate is reported, so nobody has to wonder why `integration`
  # blocked this story and not the last one.
  escalated=""
  case "$STORY_REQUIRES" in
    *" $id "*)
      [ "$req" = "required" ] || escalated=" (required by story $STORY)"
      req=required ;;
  esac

  [ -n "$ONLY" ] && [ "$ONLY" != "$id" ] && continue
  [ "$REQUIRED_ONLY" = 1 ] && [ "$req" != "required" ] && continue

  # --fast leaves out the gates a `slow` line names. It is a deliberate subset,
  # not a cheaper full run: it is never recorded, and a gate the story escalated
  # is skipped here like any other. The full run before REVIEW is what judges
  # the story; --fast only answers whether the tests are admissible to it.
  if [ "$FAST" = 1 ] && [ "$LIST" = 0 ] && [ "$AUDIT" = 0 ] \
     && table_lookup "$SLOWS" "$id" >/dev/null; then
    skipped="$skipped $id"; continue
  fi

  exp=$(table_lookup "$EVIDENCE" "$id") || exp="<none>"
  waiver=$(table_lookup "$WAIVERS" "$id") || waiver=""
  slowwhy=$(table_lookup "$SLOWS" "$id"); is_slow=$?
  floor=$(table_lookup "$FLOORS" "$id") || floor=""
  cifactor=$(table_lookup "$CIFACTORS" "$id") || cifactor=""
  nocountwhy=$(table_lookup "$NOCOUNTS" "$id"); is_nocount=$?
  logrel=".claude/state/gate-logs/$id.log"

  if [ "$LIST" = 1 ]; then
    printf '%-12s %-9s %-6s %s\n' "$id" "$req" "$cwd" "${cmd:-<unconfigured>}"
    printf '%-12s %-9s %-6s evidence: %s\n' "" "" "" "$exp"
    [ -n "$floor" ]  && printf '%-12s %-9s %-6s floor:    %s\n' "" "" "" "$floor"
    [ "$is_nocount" = 0 ] && printf '%-12s %-9s %-6s no-count: %s (no floor possible)\n' "" "" "" "${nocountwhy:-no reason given}"
    [ -n "$cifactor" ] && printf '%-12s %-9s %-6s ci-factor: %s\n' "" "" "" "$cifactor"
    [ -n "$waiver" ] && printf '%-12s %-9s %-6s waiver:   %s\n' "" "" "" "$waiver"
    [ "$is_slow" = 0 ] && printf '%-12s %-9s %-6s slow:     %s (left out of --fast)\n' "" "" "" "${slowwhy:-no reason given}"
    [ -n "$escalated" ] && printf '%-12s %-9s %-6s optional for the repo,%s\n' "" "" "" "$escalated"
    continue
  fi

  # A `slow` line with no reason is how a gate quietly leaves the fast subset
  # and nobody remembers why. The reason is the whole value of the line.
  if [ "$is_slow" = 0 ] && [ -z "$slowwhy" ]; then
    if [ "$AUDIT" = 1 ]; then
      printf 'FAIL %-12s marked slow with no reason; say what makes it too slow for --fast\n' "$id"
    else
      results="$results\nFAIL         $id (marked slow with no reason in project.conf)"
    fi
    fails=$((fails+1)); continue
  fi

  # A `no-count` line with no reason is the same failure as a `slow` one: the
  # next person needs to know WHY the regex measures nothing, or they will read
  # the line as a shrug and delete it.
  if [ "$is_nocount" = 0 ] && [ -z "$nocountwhy" ]; then
    if [ "$AUDIT" = 1 ]; then
      printf 'FAIL %-12s marked no-count with no reason; say what its evidence regex measures instead\n' "$id"
    else
      results="$results\nFAIL         $id (marked no-count with no reason in project.conf)"
    fi
    fails=$((fails+1)); continue
  fi

  # A floor is measured out of the evidence match, so it needs one, it has to
  # be a number, and the match has to contain a count rather than a stopwatch.
  # All three are checked before anything runs: a floor that cannot be
  # evaluated would otherwise sit in project.conf looking like protection.
  floor_broken=""
  if [ -n "$floor" ]; then
    case "$floor" in
      ''|*[!0-9]*) floor_broken="floor '$floor' is not a number" ;;
    esac
    if [ -z "$floor_broken" ] && { [ "$exp" = "<none>" ] || [ "$exp" = "-" ]; }; then
      floor_broken="floor needs an evidence regex to measure, and $id has none"
    fi
    # The dangerous case, and the quiet one: /$exp/ matches, a number is found,
    # the floor compares cleanly against it, and the number means nothing. A
    # floor on `Finished .* profile` passes or fails on how warm the build
    # cache is. Refusing it is the whole point of the no-count declaration.
    if [ -z "$floor_broken" ] && [ "$is_nocount" = 0 ]; then
      floor_broken="floor cannot be measured: $id is declared no-count ($nocountwhy)"
    fi
  fi
  if [ -n "$floor_broken" ]; then
    if [ "$AUDIT" = 1 ]; then
      printf 'FAIL %-12s %s\n' "$id" "$floor_broken"
    else
      results="$results\nFAIL         $id ($floor_broken)"
    fi
    fails=$((fails+1)); continue
  fi

  # A ci-factor is a measurement, so it carries the number AND where the number
  # came from. Without the second half nobody can tell a figure read off a CI
  # log from one somebody assumed, and an assumed factor is worse than none: it
  # is acted on. The whole-gate ratio is the wrong number here and reads
  # plausible - 14x for a coverage gate whose per-test compute factor is 3.4x -
  # so a story can spend a day optimising a test that was already fine.
  cifactor_broken=""
  if [ -n "$cifactor" ]; then
    cif_n=$(trim "$(printf '%s' "$cifactor" | cut -d'|' -f1)")
    # cut prints the whole field when the delimiter is absent, which would
    # read a missing source as a source repeating the number.
    case "$cifactor" in
      *'|'*) cif_src=$(trim "$(printf '%s' "$cifactor" | cut -d'|' -f2-)") ;;
      *)     cif_src="" ;;
    esac
    case "$cif_n" in
      ''|*[!0-9.]*|*.*.*|.|.*|*.) cifactor_broken="ci-factor '$cif_n' is not a number" ;;
    esac
    [ -z "$cifactor_broken" ] && [ -z "$cif_src" ] \
      && cifactor_broken="ci-factor has no source; say which CI run it was measured from"
  fi
  if [ -n "$cifactor_broken" ]; then
    if [ "$AUDIT" = 1 ]; then
      printf 'FAIL %-12s %s\n' "$id" "$cifactor_broken"
    else
      results="$results\nFAIL         $id ($cifactor_broken)"
    fi
    fails=$((fails+1)); continue
  fi

  # A waiver on a required gate is a bypass, not a waiver - including when the
  # story is what made it required.
  if [ -n "$waiver" ] && [ "$req" = "required" ]; then
    if [ "$AUDIT" = 1 ]; then
      printf 'FAIL %-12s has a waiver but is required%s; waivers are for optional gates only\n' "$id" "$escalated"
    else
      results="$results\nFAIL         $id (has a waiver but is required$escalated; waivers are for optional gates only)"
    fi
    fails=$((fails+1)); continue
  fi

  if [ "$AUDIT" = 1 ]; then
    if [ -z "$cmd" ]; then
      if [ "$req" = "required" ] && [ "$BOOTSTRAPPED" = "yes" ]; then
        printf 'FAIL %-12s no command\n' "$id"; fails=$((fails+1))
      else
        printf 'ok   %-12s (unconfigured)\n' "$id"
      fi
      continue
    fi
    if [ ! -d "$ROOT/$cwd" ]; then
      printf 'FAIL %-12s cwd does not exist: %s\n' "$id" "$cwd"; fails=$((fails+1)); continue
    fi
    if [ "$exp" = "<none>" ]; then
      printf 'WARN %-12s no evidence line; a vacuous pass would go unnoticed\n' "$id"
      noevidence=$((noevidence+1)); continue
    fi
    if [ "$exp" = "-" ]; then
      printf 'ok   %-12s (liveness declared unassertable)\n' "$id"
    else
      printf 'ok   %-12s evidence: %s\n' "$id" "$exp"
    fi
    [ -n "$floor" ]  && printf '     %-12s floor:  %s\n' "" "$floor"
    [ "$is_nocount" = 0 ] && printf '     %-12s no-count: %s (reports `observed -`; no floor possible)\n' "" "$nocountwhy"
    [ -n "$cifactor" ] && printf '     %-12s ci-factor: %s\n' "" "$cifactor"
    [ "$is_slow" = 0 ] && printf '     %-12s slow:   %s\n' "" "$slowwhy"
    [ -n "$waiver" ] && printf '     %-12s waiver: %s\n' "" "$waiver"
    continue
  fi

  if [ -z "$cmd" ]; then
    if [ "$req" = "required" ] && [ "$BOOTSTRAPPED" = "yes" ]; then
      results="$results\nFAIL         $id (required gate has no command in project.conf)"
      fails=$((fails+1))
    else
      results="$results\nUNCONFIGURED $id"
      unconfigured=$((unconfigured+1))
    fi
    continue
  fi

  printf '\n=== gate: %s (%s%s) ===\n%s\n' "$id" "$req" "$escalated" "$cmd"
  log="$LOGDIR/$id.log"
  start=$(date +%s)
  ( cd "$ROOT/$cwd" && eval "$cmd" ) 2>&1 | tee "$log"
  rc=${PIPESTATUS[0]}
  dur=$(( $(date +%s) - start ))
  ran=$((ran+1))

  # Three outcomes. Liveness is only ever consulted for a gate that already
  # exited 0: success is the exit code's job, and this asks the separate
  # question of whether the command had anything to do. A gate that fails keeps
  # failing for its own reason, with its own message.
  outcome=pass; why=""; observed=""
  if [ "$rc" -ne 0 ]; then
    outcome=fail
  elif [ "$exp" != "<none>" ] && [ "$exp" != "-" ] && ! clean_log "$log" | grep -Eq -- "$exp"; then
    outcome=noevidence
    why="ran but produced no evidence of work: expected /$exp/"
  elif [ "$exp" != "<none>" ] && [ "$exp" != "-" ] && [ "$is_nocount" = 0 ]; then
    # Live, but unmeasurable by declaration. Print the dash rather than the
    # digits work_count would have found: a number nobody may act on is worse
    # than no number, because it reads exactly like one somebody may.
    observed="-"
  elif [ "$exp" != "<none>" ] && [ "$exp" != "-" ]; then
    # The gate did work. How much, and is that less than it used to be? A suite
    # that shrinks from 47 tests to 3 exits 0 and matches its evidence regex
    # just as happily as one that grew.
    observed="$(work_count "$log" "$exp")"
    if [ -n "$floor" ]; then
      if [ -z "$observed" ]; then
        outcome=noevidence
        why="floor of $floor declared, but no number was found in the evidence match, so the work could not be measured"
      elif [ "$observed" -lt "$floor" ]; then
        outcome=noevidence
        why="did $observed units of work, below the floor of $floor in project.conf"
      fi
    fi
  fi

  case "$outcome" in
    pass)
      seen=""
      [ -n "$observed" ] && seen=", observed $observed"
      [ -n "$observed" ] && [ -n "$floor" ] && seen=", observed $observed, floor $floor"
      if [ -n "$waiver" ]; then
        results="$results\nPASS         $id (${dur}s$seen) -- waiver no longer needed, remove it: $waiver"
      elif [ "$exp" = "<none>" ] && [ "$BOOTSTRAPPED" = "yes" ] && [ "$req" = "required" ]; then
        results="$results\nPASS         $id (${dur}s) -- no evidence line: a vacuous pass would go unnoticed"
        noevidence=$((noevidence+1))
      else
        results="$results\nPASS         $id (${dur}s$seen)"
      fi ;;
    noevidence)
      if [ "$req" = "required" ]; then
        results="$results\nFAIL         $id$escalated (${dur}s, $why) -> $logrel"; fails=$((fails+1))
      elif [ -n "$waiver" ]; then
        results="$results\nKNOWN        $id (${dur}s, $why; $waiver)"; known=$((known+1))
      else
        results="$results\nWARN         $id (${dur}s, $why, optional)"; warns=$((warns+1))
      fi ;;
    fail)
      if [ "$req" = "required" ]; then
        results="$results\nFAIL         $id$escalated (${dur}s, exit $rc) -> $logrel"; fails=$((fails+1))
      elif [ -n "$waiver" ]; then
        results="$results\nKNOWN        $id (${dur}s, exit $rc; $waiver) -> $logrel"; known=$((known+1))
      else
        results="$results\nWARN         $id (${dur}s, exit $rc, optional) -> $logrel"; warns=$((warns+1))
      fi ;;
  esac
done < "$CONF"

[ "$LIST" = 1 ] && exit 0

if [ "$AUDIT" = 1 ]; then
  # A `slow` line naming a gate that does not exist excludes nothing, silently.
  while IFS="$TAB" read -r sid _; do
    [ -n "$sid" ] || continue
    case " $GATE_IDS " in
      *" $sid "*) ;;
      *) printf 'FAIL %-12s a `slow` line names no configured gate\n' "$sid"; fails=$((fails+1)) ;;
    esac
  done <<< "$SLOWS"
  # Same for a ci-factor: a measurement filed against a gate that does not
  # exist is a number nobody will ever find when they need it.
  while IFS="$TAB" read -r cid _; do
    [ -n "$cid" ] || continue
    case " $GATE_IDS " in
      *" $cid "*) ;;
      *) printf 'FAIL %-12s a `ci-factor` line names no configured gate\n' "$cid"; fails=$((fails+1)) ;;
    esac
  done <<< "$CIFACTORS"
  # Same for a no-count: it is the thing standing between a real gate and a
  # floor on a stopwatch, and a misspelt id leaves that gate undefended while
  # looking, in project.conf, exactly as though it were covered.
  while IFS="$TAB" read -r nid _; do
    [ -n "$nid" ] || continue
    case " $GATE_IDS " in
      *" $nid "*) ;;
      *) printf 'FAIL %-12s a `no-count` line names no configured gate\n' "$nid"; fails=$((fails+1)) ;;
    esac
  done <<< "$NOCOUNTS"
  if [ "$noevidence" -gt 0 ]; then
    printf '\n%d required gate(s) have no evidence line. Add one per gate:\n' "$noevidence"
    printf '  evidence | <id> | <regex proving the tool did work>\n'
    printf 'Use `evidence | <id> | -` only where no such output exists, and say why in the story.\n'
  fi
  if [ "$fails" -gt 0 ]; then
    printf '\n%d manifest problem(s).\n' "$fails"; exit 1
  fi
  printf '\nManifest audit passed.\n'
  exit 0
fi

printf '\n--- gate summary ---%b\n' "$results"

if [ "$BOOTSTRAPPED" != "yes" ]; then
  printf '\nNote: project.conf is not bootstrapped yet (BOOTSTRAPPED=no), so unconfigured\n'
  printf 'required gates are warnings. The bootstrap story must fill them in and flip the flag.\n'
fi

if [ "$noevidence" -gt 0 ]; then
  printf '\nWarning: %d required gate(s) ran without an evidence line, so a command that\n' "$noevidence"
  printf 'did no work at all would still have been recorded as PASS. Add to project.conf:\n'
  printf '  evidence | <id> | <regex proving the tool did work>\n'
fi

if [ "$warns" -gt 0 ]; then
  printf '\n%d optional gate(s) WARNed. A WARN means something changed since the last run:\n' "$warns"
  printf 'read it. A failure that is known and permanent belongs in a waiver, so that the\n'
  printf 'next WARN is not buried next to it:\n'
  printf '  waiver | <id> | <why this optional gate is expected to fail, and where that is recorded>\n'
fi

if [ "$fails" -gt 0 ]; then
  result="fail ($fails required gate(s) failed)"
  printf 'RESULT=fail\nWHEN=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$STAMP"
else
  result="pass ($ran ran, $unconfigured unconfigured, $known known)"
  printf 'RESULT=pass\nWHEN=%s\nRAN=%d\nUNCONFIGURED=%d\nNOEVIDENCE=%d\nKNOWN=%d\n' \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$ran" "$unconfigured" "$noevidence" "$known" > "$STAMP"
fi

if [ "$FAST" = 1 ]; then
  if [ -n "$skipped" ]; then
    printf '\n--fast skipped:%s\n' "$skipped"
  else
    printf '\n--fast skipped nothing: no gate carries a `slow` line, so this was a full run\n'
    printf 'in everything but the record. Mark the expensive gates:\n'
    printf '  slow | <id> | <why it is too slow to run every loop>\n'
  fi
  printf 'This is a subset, not a verdict. The full run before REVIEW is what judges the story.\n'
fi

# --- record -----------------------------------------------------------------
# Only a full run is evidence. `--gate unit` passing says nothing about lint.
if [ -n "$ONLY" ] || [ "$REQUIRED_ONLY" = 1 ] || [ "$FAST" = 1 ]; then
  printf '\n(not recorded in the story: a partial run is not evidence of anything)\n'
else
  if [ -z "$STORY" ]; then
    printf '\n(not recorded: no active story; use --story <id> to record it in one)\n'
  elif [ ! -f "$STORY_FILE" ]; then
    printf '\n(not recorded: no story file at docs/backlog/stories/%s.md)\n' "$STORY"
  else
    if record_in_story "$STORY_FILE" "$result" "$(printf '%b' "$results")"; then
      printf '\nrecorded in docs/backlog/stories/%s.md (## Gate results)\n' "$STORY"
    else
      printf '\n(not recorded: docs/backlog/stories/%s.md already holds a record from a gate\n' "$STORY"
      printf 'run that started at %s. This one started at %s, so its\n' "$RECORD_SKIPPED" "$RUN_STARTED_AT"
      printf 'evidence is the older of the two and the record already there stands.)\n'
    fi
  fi
fi

if [ "$fails" -gt 0 ]; then
  printf '\n%d required gate(s) failed.\n' "$fails"
  exit 1
fi
printf '\nAll required gates passed (%d ran, %d unconfigured, %d known).\n' "$ran" "$unconfigured" "$known"
if [ "$FAST" = 0 ] && [ -z "$ONLY" ] && [ "$REQUIRED_ONLY" = 0 ]; then
  printf 'CI runs one more script that this does not: bash scripts/check-boundaries.sh\n'
  printf 'It is not a gate because it judges the COMMIT rather than the code - the phase in\n'
  printf 'the committed frontmatter, the criteria against the base branch, and whether this\n'
  printf 'very record still matches the tree. Run it after committing, before the PR.\n'
fi
