#!/usr/bin/env bash
# Tests for scripts/doctor.sh - specifically the discovery checks, which are
# the answer to a gate whose scope collapsed to nothing without anyone noticing.

. "$(dirname "${BASH_SOURCE[0]}")/_lib.sh"

FIX="$(make_project_fixture)"
trap 'rm -rf "$FIX"' EXIT

doctor() { ( cd "$FIX" && bash scripts/doctor.sh 2>&1 ); }

describe "discovery: the runner is asked what it can see"

write_conf "$FIX" <<'EOF'
gate      | unit     | required | . | printf 'Tests  1 passed (1)\n'
evidence  | unit     | Tests +[1-9][0-9]* passed
discovery | platform | . | printf 'src/platform/gl.test.ts\n' | grep -q "src/platform/"
EOF
out="$(doctor)"
assert_contains "a directory the runner can see" "ok       platform     discovered" "$out"

write_conf "$FIX" <<'EOF'
gate      | unit     | required | . | printf 'Tests  1 passed (1)\n'
evidence  | unit     | Tests +[1-9][0-9]* passed
discovery | platform | . | printf 'src/ui/app.test.ts\n' | grep -q "src/platform/"
EOF
out="$(doctor)"
assert_contains "a directory it cannot" "MISSING  platform" "$out"
assert_contains "says what that costs"  "committed and never run" "$out"

describe "discovery: nothing declared is reported, not skipped silently"

write_conf "$FIX" <<'EOF'
gate     | unit | required | . | printf 'Tests  1 passed (1)\n'
evidence | unit | Tests +[1-9][0-9]* passed
EOF
out="$(doctor)"
assert_contains "the section still appears" "none declared" "$out"

summary "doctor"
