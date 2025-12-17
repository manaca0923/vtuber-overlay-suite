# GitHub Actions セットアップ

## Claude Code Review

このワークフローは、PRコメントで `@claude review` または `@codex review` とメンションすると、Claude Opus 4.5を使用した自動コードレビューを実行します。

### セットアップ手順

1. **Anthropic API Keyの取得**
   - https://console.anthropic.com/ にアクセス
   - APIキーを生成

2. **GitHub Secretsに登録**
   ```
   リポジトリ設定 > Secrets and variables > Actions > New repository secret
   Name: ANTHROPIC_API_KEY
   Secret: <your-api-key>
   ```

3. **使用方法**
   PRコメントに以下を投稿：
   ```
   @claude review
   ```
   または
   ```
   @codex review
   ```

### 使用モデル

- **Claude Opus 4.5** (`claude-opus-4-5-20251101`)
- 最高品質のコードレビューを提供
- より高度な推論能力で複雑な問題を検出

### カスタマイズ

`claude-review.yml` の `claude_args` を編集することで、以下をカスタマイズ可能：
- モデルの変更（Opus ↔ Sonnet ↔ Haiku）
- max-turns（会話の最大ターン数）
- system-prompt（レビュー方針）
