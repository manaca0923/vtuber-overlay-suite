#!/usr/bin/env bash
set -euo pipefail

BASE_BRANCH="${1:-main}"
mkdir -p .codex .codex/history

# 常に「最新だけ」にする（混入ゼロ）
: > .codex/review.md

ts="$(date +%Y%m%d-%H%M%S)"
sha="$(git rev-parse --short HEAD 2>/dev/null || echo 'nosha')"
snap=".codex/history/review-${ts}-${sha}.md"
status_file=".codex/review.status"

# JSON出力用関数（Claude Codeのhooksはこの形式でユーザーに表示される）
output_json() {
  local message="$1"
  local context="${2:-}"
  if [ -n "$context" ]; then
    jq -n --arg msg "$message" --arg ctx "$context" '{
      "systemMessage": $msg,
      "additionalContext": $ctx
    }'
  else
    jq -n --arg msg "$message" '{
      "systemMessage": $msg
    }'
  fi
}

# 差分がない場合
if git diff --quiet "${BASE_BRANCH}...HEAD"; then
  printf "No diffs to review against %s.\n" "$BASE_BRANCH" > .codex/review.md
  echo "NODIFF" > "$status_file"
  cp .codex/review.md "$snap"
  echo "$snap" > .codex/review.latest_snapshot
  output_json "📋 Codex Review: No diffs against ${BASE_BRANCH}" "Snapshot: $snap"
  exit 0
fi

DIFF="$(git diff --patch --minimal "${BASE_BRANCH}...HEAD")"

PROMPT=$'You are a senior code reviewer.\n'
PROMPT+=$'Review the diff against the base branch.\n'
PROMPT+=$'Focus on: correctness, edge cases, security, performance regressions, and test coverage.\n'
PROMPT+=$'Output sections:\n'
PROMPT+=$'1) Summary (3 bullets)\n'
PROMPT+=$'2) High-risk issues (file paths + why)\n'
PROMPT+=$'3) Concrete fixes (code-level guidance)\n'
PROMPT+=$'4) Missing/insufficient tests (what to add)\n'
PROMPT+=$'Avoid style nits unless they impact maintainability.\n'

# Codex 実行（失敗したら失敗メッセージで上書き）
if ! {
  {
    printf "%s\n\n---\n\n# Diff\n\n%s\n" "$PROMPT" "$DIFF"
  } | codex exec - --output-last-message ".codex/review.md"
}; then
  printf "Codex review failed. Please rerun.\n" > .codex/review.md
  echo "FAIL" > "$status_file"
  cp .codex/review.md "$snap"
  echo "$snap" > .codex/review.latest_snapshot
  output_json "❌ Codex Review: Failed" "Please check .codex/review.md for details"
  exit 1
fi

echo "OK" > "$status_file"
cp .codex/review.md "$snap"
echo "$snap" > .codex/review.latest_snapshot

# レビュー結果の全文を表示
review_content=$(cat .codex/review.md)
output_json "✅ Codex Review: Complete (Saved: $snap)" "$review_content"
