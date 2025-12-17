# GitHub Actions ワークフロー

このリポジトリでは、Claude Code公式のGitHub Actionsを使用しています。

## 利用可能なワークフロー

### 1. `claude.yml` - コメントトリガー
PRやIssueのコメントで `@claude` とメンションすると、Claudeが自動的にタスクを実行します。

**使用方法:**
```
@claude このPRをレビューしてください
```

### 2. `claude-code-review.yml` - 自動レビュー
PRが作成または更新されると、自動的にClaudeがコードレビューを実行します。

**トリガー:**
- PR作成時（`opened`）
- PR更新時（`synchronize`）

## セットアップ

これらのワークフローは `CLAUDE_CODE_OAUTH_TOKEN` シークレットを使用します。

リポジトリに既に設定されているため、追加のセットアップは不要です。

## カスタマイズ

ワークフローの動作をカスタマイズする場合は、各ワークフローファイルの `claude_args` パラメータを編集してください。

詳細は公式ドキュメントを参照：
- https://github.com/anthropics/claude-code-action
- https://code.claude.com/docs/en/github-actions
