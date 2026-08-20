#!/usr/bin/env bash
# Implement all triaged issue briefs, one fresh opencode session per issue.
#
# WHY: each issue runs in its own `opencode run` subprocess = clean context.
# With a local LLM (100K context limit) you cannot process 24 issues in one
# session — context blows up and quality degrades. Fresh session per issue
# is the "clear context" step, done automatically.
#
# For each issue: fresh opencode run -> agent implements (AGENTS.md + brief)
# -> runs cargo test -> commits. Then next issue. Skips issues marked
# "Status: done" in the brief.
#
# Usage:
#   ./scripts/implement-all.sh [start]        # start at issue number N (1-based)
#   RESUME=1 ./scripts/implement-all.sh       # resume from first undone issue
#
# Env overrides:
#   ISSUES_DIR     briefs dir           (default: .scratch/deepdelve/issues)
#   MODEL          opencode model       (default: qwen-local/Qwen3.8-27B-IQ3_XXS-mtp)
#   VARIANT        reasoning variant    (default: Medium)
set -euo pipefail
cd "$(dirname "$0")/.."   # project root

ISSUES_DIR="${ISSUES_DIR:-.scratch/deepdelve/issues}"
MODEL="${MODEL:-qwen-local/Qwen3.8-27B-IQ3_XXS}"
VARIANT="${VARIANT:-Medium}"
OPENCODE="${OPENCODE:-$HOME/.opencode/bin/opencode}"
LOG=".scratch/implement-all.log"
mkdir -p "$(dirname "$LOG")"

mapfile -t ISSUES < <(ls "$ISSUES_DIR"/[0-9]*.md 2>/dev/null | sort -V)
if [[ ${#ISSUES[@]} -eq 0 ]]; then echo "no issue files in $ISSUES_DIR"; exit 1; fi

START="${1:-1}"
if [[ "${RESUME:-0}" == "1" ]]; then START=1; fi

for ISSUE in "${ISSUES[@]}"; do
    NUM=$(basename "$ISSUE" | cut -d- -f1)
    (( NUM < START )) && continue

    if grep -q "^Status: done" "$ISSUE"; then
        echo "[$NUM] SKIP (Status: done) — $ISSUE"
        continue
    fi

    echo "[$NUM] ========== IMPLEMENTING $(basename "$ISSUE") =========="
    echo "[$NUM] START $(date +%H:%M:%S)" >> "$LOG"
    HEAD_BEFORE=$(git rev-parse HEAD)

    if ! "$OPENCODE" run \
        --dangerously-skip-permissions \
        -m "$MODEL" \
        --variant "$VARIANT" \
        "Implement the issue described in $(realpath "$ISSUE"). Follow AGENTS.md. Run cargo test before committing. Commit the work with a message referencing this issue. If the fix needs discussion or is ambiguous, say so and stop without committing." \
        2>&1 | tee -a "$LOG"; then
        echo "[$NUM] FAILED — opencode run errored" | tee -a "$LOG"
        continue
    fi

    HEAD_AFTER=$(git rev-parse HEAD)
    if [[ "$HEAD_BEFORE" != "$HEAD_AFTER" ]]; then
        echo "[$NUM] COMMITTED: $(git log -1 --oneline)" | tee -a "$LOG"
    else
        echo "[$NUM] WARNING: no new commit (agent may have skipped or work needs review)" | tee -a "$LOG"
    fi
    echo "[$NUM] END $(date +%H:%M:%S)" >> "$LOG"
    echo ""
done

echo "=== done. Log: $LOG ==="
