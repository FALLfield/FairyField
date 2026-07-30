#!/usr/bin/env bash
# Launch OMX for FairyField from Ghostty without fallback shell injection.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"

usage() {
    cat <<'EOF'
Usage:
  scripts/omx-ghostty.sh [start] [OMX leader options...]
  scripts/omx-ghostty.sh doctor
  scripts/omx-ghostty.sh help

Start an OMX-managed Codex leader in tmux for Ghostty.

Safety contract:
  - The fallback notification watcher is disabled. It can otherwise mistake a
    fish prompt for a Codex prompt and type team-status text into the shell.
  - Team workers get a three-minute readiness/evidence window for Codex startup.
  - Do not run `omx team ...` from a bare Ghostty shell. Ask the live Codex
    leader to use `$team`, so team messages target an active Codex pane.

After OMX starts, attach from Ghostty with the session name it reports:
  tmux attach -t <session-name>
EOF
}

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Error: required command not found: %s\n' "$1" >&2
        exit 127
    fi
}

require_command omx
require_command tmux

command="${1:-start}"
case "$command" in
    help|-h|--help)
        usage
        exit 0
        ;;
    doctor)
        cd "$PROJECT_ROOT"
        exec omx doctor --team
        ;;
    team)
        printf '%s\n' 'Refusing to start an OMX team from a bare Ghostty shell.' >&2
        printf '%s\n' 'Start the leader with this script, then ask the live Codex leader to use `$team`.' >&2
        exit 64
        ;;
    start)
        shift
        ;;
esac

for argument in "$@"; do
    if [[ "$argument" == "--direct" ]]; then
        printf '%s\n' 'Error: --direct bypasses the Ghostty/tmux safety contract.' >&2
        exit 64
    fi
done

cd "$PROJECT_ROOT"

# The fallback watcher is the path that injected status text into a fish prompt.
# Native notifications remain available when a real Codex leader is active.
export OMX_NOTIFY_FALLBACK=0
export OMX_TEAM_READY_TIMEOUT_MS="${OMX_TEAM_READY_TIMEOUT_MS:-180000}"
export OMX_TEAM_STARTUP_EVIDENCE_TIMEOUT_MS="${OMX_TEAM_STARTUP_EVIDENCE_TIMEOUT_MS:-180000}"

# Workers run in isolated worktrees but still need to update the team mailbox,
# task state, and locks. Preserve an explicit user override unchanged.
if [[ -z "${OMX_TEAM_WORKER_LAUNCH_ARGS:-}" ]]; then
    export OMX_TEAM_WORKER_LAUNCH_ARGS="--add-dir \"$PROJECT_ROOT/.omx/state\""
fi

exec omx --tmux "$@"
