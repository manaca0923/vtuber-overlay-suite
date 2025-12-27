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

PROMPT=$(cat <<'EOF'
あなたはシニアのコードレビュー担当です。
以下の差分（Diff）をレビューしてください。

観点：正しさ、境界条件、セキュリティ、性能劣化、テスト網羅性。

出力は必ず日本語で記述し、英語での見出し（Summary 等）は使用しないでください。
ただし、ファイルパス・シンボル名・コード片は原文のまま記載してください。

出力フォーマット（Markdown）：

## 要約
- （3点、箇条書き）

## 高リスクの指摘
- 各指摘に重要度ラベルを付与: [Critical] / [High] / [Medium]
- フォーマット: [重要度] ファイルパス + 根拠 + 影響
- 該当なしの場合は「該当なし」と記載

## 具体的な修正案
- （必要な場合のみ）どのファイルのどこをどう直すか具体的に

## 不足しているテスト
- 追加すべきテストケース（具体例）

## プロジェクト固有の注意点
以下のルールに違反していないか確認してください：
- Tauriコマンド引数はsnake_case必須（TypeScriptのinvoke呼び出し確認）
- keyringアクセスはspawn_blockingでラップ
- RwLockガードをawait境界をまたいで保持しない
- HTTPクライアントにはタイムアウト設定必須
- APIキーはkeyringに永続化（メモリのみ不可）

スタイル指摘は保守性に影響する場合のみ。
上記の見出し・順序を変更せず、必ずこの形式で出力してください。
EOF
)

# Codex 実行（失敗したら失敗メッセージで上書き）
if ! {
  {
    printf "%s\n\n---\n\n# Diff\n\n%s\n" "$PROMPT" "$DIFF"
  } | codex exec - --output-last-message ".codex/review.md" 1>/dev/null
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
