#!/usr/bin/env bash
set -euo pipefail
# 無効化フラグがあれば何もしない
if [ -f "$CLAUDE_PROJECT_DIR/.claude/disable_codex_review" ]; then
  exit 0
fi
"$CLAUDE_PROJECT_DIR"/scripts/codex-review.sh main
