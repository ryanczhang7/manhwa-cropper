# Harness runtime state

Machine-local, gitignored. Written by the scripts and read by the hooks.

| File | Written by | Read by |
|---|---|---|
| `current-story.env` | `scripts/phase.sh` | the phase guard, the status line |
| `last-gate-run` | `scripts/gates.sh` | the stop hook |
| `gate-logs/*.log` | `scripts/gates.sh` | you, when a gate fails |

Do not edit these by hand and do not commit them. `.claude/settings.json` denies
direct writes here for exactly that reason: `bash scripts/phase.sh set` is the
supported path, because it keeps this file and the story frontmatter in
agreement.

No `current-story.env` means no active story, which means the phase lock is off.
