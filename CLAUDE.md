# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 言語設定

**ユーザへの返答は常に日本語で行うこと**（PR説明文、コミットメッセージも含む）

## プロジェクト概要

VTuber配信支援ツール - 「わんコメ＋セトリスタ」の上位互換を目指すオールインワンデスクトップアプリケーション

## 技術スタック

- **Desktop**: Tauri 2.0 (Rust)
- **Frontend**: React + TypeScript + Vite
- **Styling**: Tailwind CSS
- **Database**: SQLite
- **Realtime**: WebSocket (localhost:19801)
- **HTTP Server**: Axum (localhost:19800)

## 開発コマンド（実装後）

```bash
# 開発サーバー起動
npm run tauri dev

# ビルド
npm run tauri build

# フロントエンドのみ
npm run dev

# 型チェック
npm run typecheck

# リント
npm run lint

# テスト
npm run test
```

## ドキュメント参照

`docs/` フォルダに設計ドキュメントがある。実装時は必ず参照すること。

| ファイル | 内容 |
|----------|------|
| `001_requirements.md` | 要件サマリー・受け入れ基準 |
| `100_architecture.md` | 技術アーキテクチャ・ディレクトリ構成 |
| `110_development-environment.md` | 開発環境セットアップ・要件 |
| `200_youtube-api.md` | YouTube API仕様・クォータ管理 |
| `300_overlay-specs.md` | オーバーレイ・WebSocket仕様 |
| `400_data-models.md` | SQLiteスキーマ・型定義 |
| `900_tasks.md` | タスク分解・チェックリスト |

docs/ファイルは番号プレフィックスで分類:
- 0xx: 概要・要件
- 1xx: アーキテクチャ
- 2xx: 外部API
- 3xx: UI/オーバーレイ
- 4xx: データモデル
- 9xx: タスク管理

## ブランチ・PR運用

タスク実行時は必ずブランチを作成し、完了後にPRを作成する。

**ブランチ命名規則** (Conventional Branch):
```
feature/[FeatureName]-[実装した機能名]
例: feature/youtube-api-polling
例: feature/setlist-drag-drop
```

**PR作成後の必須手順**:
1. PRを作成したら、必ず `@codex review` とコメントを送信すること
2. レビュー指摘事項を修正した後も、再度 `@codex review` とコメントを送信すること

**PRレビューコメント確認の必須手順**:
GitHubのPRには2種類のコメントがあり、**両方を必ず確認すること**:
- **Issue Comments（会話コメント）**: `gh pr view <PR番号> --json comments`
- **Review Comments（レビューコメント）**: `gh api repos/{owner}/{repo}/pulls/<PR番号>/comments` ⚠️ 重要

両方を確認するコマンド例:
```bash
# Issue Comments
gh pr view 2 --json comments --jq '.comments[-3:] | .[] | "[\(.author.login)] \(.body)"'

# Review Comments（ファイルの特定行へのコメント）
gh api repos/manaca0923/vtuber-overlay-suite/pulls/2/comments | jq -r '.[] | "[\(.path):\(.line)] \(.body)"'
```

## タスク管理

`docs/900_tasks.md` のチェックリストで進捗管理:
- `- [ ]` 未完了
- `- [x]` 完了

タスク完了時は必ずチェックを更新すること。

## 開発環境

### 現在の環境
- **主要開発環境**: macOS (Apple Silicon M3)
- **対象プラットフォーム**: Windows, macOS（将来的にLinuxも検討）

### Windows対応方針
- macOS環境で開発を行う
- **Windows環境でのビルド可能性を常に担保**するため、GitHub ActionsのWindowsランナーでビルド＆成果物生成を自動実行（`.github/workflows/build-windows.yml`）
- Windows環境での本格的な開発は必要になったタイミングで開始

詳細は `docs/110_development-environment.md` を参照。

## アーキテクチャポイント

### 通信フロー
```
YouTube API → Rust Poller → WebSocket Server → OBS Browser Source
                  ↓
               SQLite (ログ保存)
```

### ポート設計
- HTTP: `localhost:19800` - オーバーレイ配信
- WebSocket: `localhost:19801` - リアルタイム更新

### セキュリティ
- APIキーはOSセキュアストレージに保存（keyringクレート）
- ログ出力時はAPIキーをマスキング
