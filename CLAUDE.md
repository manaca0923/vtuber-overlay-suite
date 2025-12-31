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
npm run tauri:dev

# ビルド
npm run tauri:build

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
| `900_tasks.md` | タスク分解・チェックリスト（未完了のみ） |
| `901_tasks_archived.md` | 完了済みタスクアーカイブ |

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

### ファイル構成

| ファイル | 内容 |
|----------|------|
| `docs/900_tasks.md` | **未完了タスクのみ**（軽量化のため） |
| `docs/901_tasks_archived.md` | 完了済みタスクの履歴 |

**マーカー**:
- `- [ ]` 未完了
- `- [x]` 完了

### タスクファイルのメンテナンス

ファイルが大きくなりすぎた場合は、分割スクリプトを実行:
```bash
python3 scripts/split_tasks.py
```
- 完了済みセクションを `901_tasks_archived.md` に移動
- 未完了タスクのみ `900_tasks.md` に残す

### レビュー指摘事項の後回し対応

PRレビューで指摘された改善提案を後回しにする場合は、**必ず** `docs/900_tasks.md` の「本番リリース前チェックリスト」に追加すること。

追加時の記載ルール:
- 項目名の後にPR番号を付与（例: `(PR#19)`）
- 具体的な対応内容を箇条書きで記載
- 該当するカテゴリ（セキュリティ/コード品質/機能改善/パフォーマンス/テスト/ドキュメント）に分類

### レビュー指摘事項のノウハウ蓄積

PRレビューで受けた指摘は `issues/` ディレクトリにノウハウとして蓄積し、同じ指摘を繰り返さないようにする。

**対応手順**:
1. 指摘内容を確認し、修正が必要であれば修正
2. 修正時は同じ観点で他に問題がないかを網羅的に確認
3. 修正が他機能に影響しないかを確認し最適な方法で修正
4. `issues/` ディレクトリに指摘内容と解決方法をドキュメント化
5. 次回以降の対応でよい指摘は `docs/900_tasks.md` に追記
6. 以前と同様の指摘を受ける箇所がないかを確認してからコミット&プッシュ

**issuesディレクトリ構成**:
```
issues/
├── 001_component-system-review.md  # PR #52 コンポーネントシステム
└── ...
```

### ノウハウ参照ガイド（実装前に必ず確認）

実装を開始する前に、関連するissuesを参照して同じミスを繰り返さないようにすること。

| 実装カテゴリ | 参照すべきissues |
|-------------|-----------------|
| **JavaScript関数の入力検証** | issues/013（防御的プログラミング） |
| **オブジェクト型ガード** | issues/013（配列排除、型チェック） |
| **定数・マジックナンバー** | issues/020（定数化、TypeScript/Rust同期） |
| **Rust serde命名** | issues/021（snake_case/camelCase問題） |
| **WebSocket通信** | issues/010（bfcache対応、タイマークリア） |
| **オーバーレイセキュリティ** | issues/002（URLバリデーション、深層防御） |
| **アニメーション** | issues/022（二重実行防止パターン） |
| **設定マイグレーション** | issues/025（旧形式→新形式変換） |
| **フォーム要素** | issues/013（アクセシビリティ: id, htmlFor, aria-label） |

**必須確認タイミング**:
- 新しいWebSocketメッセージ追加時 → issues/021（フィールド命名規則）
- 外部データ受信時 → issues/002（セキュリティ）、issues/013（型ガード）
- 設定項目追加時 → issues/016（オプショナルフィールド）、issues/025（マイグレーション）
- オーバーレイコンポーネント作成時 → issues/010（bfcache）、issues/022（アニメーション）

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
