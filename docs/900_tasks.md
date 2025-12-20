# タスク分解・進捗管理

## Phase 1: MVP（目標: 8週間）

---

## T01: Tauri 2.0 プロジェクト初期化
**優先度**: P0 | **見積**: 2日 | **依存**: なし

### チェックリスト
- [x] Tauri CLI インストール (`cargo install tauri-cli`)
- [x] プロジェクト作成 (`cargo tauri init`)
- [x] React + TypeScript + Vite セットアップ
- [x] Tailwind CSS 導入
- [x] 基本ウィンドウ設定（サイズ、タイトル）
- [x] ホットリロード動作確認
- [x] ビルド確認（dev/release）

### 成果物
- `src-tauri/` ディレクトリ
- `src/` ディレクトリ（React）
- `package.json`, `Cargo.toml`

---

## T02: YouTube API 実装（BYOK + list）
**優先度**: P0 | **見積**: 7日 | **依存**: T01

### チェックリスト
- [x] APIキー入力UI（React）
- [x] APIキー検証エンドポイント（Rust）
- [x] 動画ID入力 → activeLiveChatId 取得
- [x] `liveChatMessages.list` 実装
- [x] レスポンスパース → ChatMessage型
- [x] コメント型のフロントエンド連携（Tauri Command）
- [ ] クォータ消費量のロギング（T03で詳細実装予定）

### テスト項目
- [x] 有効なAPIキーで認証成功
- [x] 無効なAPIキーでエラー表示
- [ ] ライブ配信中のコメント取得（手動テスト必要）
- [ ] 配信終了時のハンドリング（T03で実装）

### 成果物
- `src-tauri/src/youtube/` モジュール（client, types, errors）
- `src-tauri/src/commands/youtube.rs` Tauriコマンド
- `src/components/ApiKeySetup.tsx` テスト用UI

---

## T03: ポーリング制御・エラーハンドリング
**優先度**: P0 | **見積**: 3日 | **依存**: T02

### チェックリスト
- [x] `pollingIntervalMillis` 順守ロジック
- [x] `nextPageToken` の永続化・復元
- [x] 指数バックオフ実装
- [x] rateLimitExceeded 対応
- [x] quotaExceeded 対応（ユーザー通知）
- [x] liveChatNotFound 対応（配信終了）
- [x] 自動再接続ロジック
- [x] クォータ残量の推定表示

### 成果物
- `src-tauri/src/youtube/backoff.rs` - 指数バックオフロジック
- `src-tauri/src/youtube/state.rs` - ポーリング状態管理
- `src-tauri/src/youtube/poller.rs` - ポーリングマネージャー
- `src-tauri/src/commands/youtube.rs` - ポーリング制御コマンド追加

### テスト項目
- [ ] 2時間連続取得で停止しない（手動テスト必要）
- [ ] ネットワーク切断→自動復帰（手動テスト必要）
- [ ] レート制限時のバックオフ動作（手動テスト必要）

---

## T04: WebSocket/HTTP サーバー
**優先度**: P0 | **見積**: 3日 | **依存**: T01

### チェックリスト
- [x] HTTP サーバー起動（localhost:19800）
- [x] WebSocket サーバー起動（localhost:19801）
- [x] `/overlay/comment` エンドポイント
- [x] `/overlay/setlist` エンドポイント
- [x] `/api/health` ヘルスチェック
- [x] WebSocket 接続管理（複数クライアント対応）
- [x] コメント配信（WebSocket broadcast）
- [x] セットリスト更新配信（ダミー実装、T06で実データ対応）

### 追加修正（2025-12-18）
- [x] messageType形式の不整合を修正（オーバーレイ側でタグ付きenumに対応）
- [x] YouTube APIのsnippet.typeをパースしてMessageTypeを正しく設定
- [x] スーパーチャットの金額表示実装
- [x] セットリスト更新配信のダミーコマンド実装（`broadcast_setlist_update`）

### テスト項目
- [ ] ブラウザからオーバーレイ表示（手動テスト必要）
- [ ] OBS ブラウザソースで表示（手動テスト必要）
- [ ] 複数ブラウザソース同時接続（手動テスト必要）

### 成果物
- `src-tauri/src/server/` モジュール（http, websocket, types）
- `src-tauri/overlays/comment.html` - コメント表示オーバーレイ
- `src-tauri/overlays/setlist.html` - セットリスト表示オーバーレイ
- `src-tauri/src/lib.rs` - サーバー自動起動統合

---

## T05: コメント表示オーバーレイ
**優先度**: P0 | **見積**: 5日 | **依存**: T04
**ステータス**: ✅ **実質完了**（T04で大部分を実装済み）

### チェックリスト
- [x] 基本HTML構造（T04で実装済み）
- [x] WebSocket接続・再接続（T04で実装済み、指数バックオフ対応）
- [x] コメント受信→DOM追加（T04で実装済み）
- [x] 古いコメント自動削除（表示件数制限）（T04で実装済み、MAX_COMMENTS=10）
- [x] フェードイン/アウトアニメーション（T04で実装済み）
- [x] ユーザーアイコン表示（T04で実装済み）
- [x] バッジ表示（モデ/メンバー/オーナー）（T04で実装済み）
- [x] スーパーチャット専用スタイル（2025-12-18修正完了）
- [x] スクロール制御（新着で自動スクロール）（T04で実装済み）

### 追加実装（2025-12-18）
- [x] スーパーチャットの金額表示機能
- [x] MessageType のタグ付きenumサポート
- [x] YouTube APIの全メッセージタイプ対応（superChat, superSticker, membership, membershipGift）

### テスト項目
- [ ] OBSで透過背景動作
- [ ] 連続コメントでパフォーマンス維持
- [ ] 異常な長文コメントのハンドリング

---

## T06: セットリストUI・データ管理
**優先度**: P0 | **見積**: 6日 | **依存**: T01
**ステータス**: ✅ **完了**（Phase 1-3すべて完了）

### チェックリスト

#### Phase 1: データベース基盤（完了）
- [x] SQLiteスキーマ設計・マイグレーション
- [x] 楽曲CRUD（Tauri Command）
- [x] セットリストCRUD（Tauri Command）

#### Phase 2: 基本UI実装（完了）
- [x] TypeScript型定義（song.ts, setlist.ts, commands.ts）
- [x] React: 楽曲一覧表示（SongList.tsx）
- [x] React: 楽曲作成・編集フォーム（SongForm.tsx）
- [x] React: セットリスト一覧表示（SetlistList.tsx）
- [x] React: セットリスト編集画面（SetlistEditor.tsx）
- [x] App.tsxにタブUI統合

#### Phase 3: 高度な機能（完了）
- [x] @dnd-kit導入
- [x] 曲切替コマンド実装（set_current_song, next_song, previous_song）
- [x] 曲切替UI実装（前へ/次へボタン、現在曲ハイライト）
- [x] 曲切替時タイムスタンプ記録（started_at, ended_at）
- [x] ドラッグ&ドロップ曲順変更（@dnd-kit）
- [x] WebSocket統合（セットリスト更新配信）
- [x] YouTube概要欄用タイムスタンプ出力UI

### テスト項目
- [x] 楽曲追加・編集・削除
- [x] セットリスト追加・編集・削除
- [x] 楽曲をセットリストに追加・削除
- [x] 曲切替（次へ/前へ/指定位置）
- [x] 曲順ドラッグで変更
- [x] タイムスタンプのコピー

### 成果物（Phase 1-3完了）
- `src-tauri/migrations/001_initial.sql` - データベーススキーマ
- `src-tauri/src/db/mod.rs`, `src-tauri/src/db/models.rs` - データベースモジュール
- `src-tauri/src/commands/setlist.rs` - 楽曲・セットリストCRUDコマンド（12個）
- `.sqlx/` - sqlxオフラインモードメタデータ
- `src/types/song.ts`, `src/types/setlist.ts`, `src/types/commands.ts` - 型定義
- `src/components/SongList.tsx`, `src/components/SongForm.tsx` - 楽曲管理UI
- `src/components/SetlistList.tsx`, `src/components/SetlistEditor.tsx` - セットリスト管理UI
- `src/App.tsx` - タブUI統合

---

## T07: セットリスト表示オーバーレイ
**優先度**: P0 | **見積**: 4日 | **依存**: T06
**ステータス**: ✅ **完了**（T04で実装済み）

### チェックリスト
- [x] 基本HTML構造
- [x] WebSocket接続・再接続
- [x] 現在の曲ハイライト
- [x] 曲切替アニメーション
- [x] 前後曲の表示
- [x] 進行インジケーター

### テスト項目
- [ ] OBSで透過背景動作（手動テスト必要）
- [ ] 曲切替の即時反映（手動テスト必要）
- [x] 長い曲名の省略表示

### 成果物
- `src-tauri/overlays/setlist.html` - セットリスト表示オーバーレイ（T04で実装済み）

---

## T08: 神テンプレ実装 + 簡易設定
**優先度**: P0 | **見積**: 5日 | **依存**: T05, T07

### チェックリスト
- [ ] テンプレートCSS変数設計
- [ ] カラーバリアント切替
- [ ] 位置設定（上/下/左/右）
- [ ] 表示ON/OFF切替
- [ ] フォント設定
- [ ] 設定UIコンポーネント
- [ ] 設定の永続化（SQLite）
- [ ] プレビュー画面

### テスト項目
- [ ] 設定変更が即時反映
- [ ] 設定がアプリ再起動後も保持

---

## T09: 神テンプレ1セット制作
**優先度**: P0 | **見積**: 8日（並行可） | **依存**: T08

### チェックリスト
- [ ] デザインコンセプト策定
- [ ] 背景デザイン
- [ ] コメント枠デザイン
- [ ] セットリスト枠デザイン
- [ ] 小物アセット
- [ ] カラーバリアント（2-3種）
- [ ] CSS実装
- [ ] VTuberレビュー（5名中4名合格）

---

## T10: 初回設定ウィザード + テストモード
**優先度**: P1 | **見積**: 5日 | **依存**: T02, T05

### チェックリスト
- [ ] ウィザードステップUI
- [ ] Step 1: APIキー入力・検証
- [ ] Step 2: 動画ID入力（またはURL）
- [ ] Step 3: テンプレート選択
- [ ] Step 4: OBS設定ガイド
- [ ] テストモード（ダミーコメント生成）
- [ ] ライブプレビュー

### テスト項目
- [ ] 初回起動でウィザード表示
- [ ] 全ステップ完了でメイン画面へ

---

## T11: 認証情報保護
**優先度**: P1 | **見積**: 2日 | **依存**: T02

### チェックリスト
- [ ] keyring クレート導入
- [ ] Windows: Credential Manager 連携
- [ ] macOS: Keychain 連携
- [ ] APIキー保存/取得/削除
- [ ] ログマスキング実装

### テスト項目
- [ ] APIキーが平文で保存されない
- [ ] ログにAPIキーが出力されない

---

## T12: インストーラー + 更新通知
**優先度**: P1 | **見積**: 4日 | **依存**: All

### チェックリスト
- [ ] Tauri Updater 設定
- [ ] 更新サーバー設定（GitHub Releases等）
- [ ] 起動時更新チェック
- [ ] 更新通知ダイアログ
- [ ] Windows インストーラー（.msi）
- [ ] macOS インストーラー（.dmg）
- [ ] 自動更新または手動ダウンロード

### テスト項目
- [ ] 新バージョン検出→通知
- [ ] 更新ダウンロード・インストール

---

## T13: PoC - streamList（gRPC）
**優先度**: P1 | **見積**: 3日 | **依存**: T02

### チェックリスト
- [ ] tonic クレート導入
- [ ] proto ファイル取得・生成
- [ ] gRPC 接続確立
- [ ] ストリーミング受信
- [ ] 切断検知・再接続
- [ ] Feature Flag 実装
- [ ] 安定性レポート作成

### 判断基準
- 接続成功率 > 95%
- 平均再接続時間 < 5秒
- HTTP/2プロキシ環境での動作

---

## 進捗サマリー

| タスク | ステータス | 完了日 |
|--------|------------|--------|
| T01 | ✅ 完了 | 2025-12-17 |
| T02 | ✅ 完了 | 2025-12-18 |
| T03 | ✅ 完了 | 2025-12-18 |
| T04 | ✅ 完了 | 2025-12-18 |
| T05 | ✅ 完了 | 2025-12-18（T04で実装済み、追加修正完了） |
| T06 | ✅ 完了 | 2025-12-19（Phase 1-3すべて完了） |
| T07 | ✅ 完了 | 2025-12-18（T04で実装済み） |
| T08 | ⬜ 未着手 | - |
| T09 | ⬜ 未着手 | - |
| T10 | ⬜ 未着手 | - |
| T11 | ⬜ 未着手 | - |
| T12 | ⬜ 未着手 | - |
| T13 | ⬜ 未着手 | - |

**ステータス凡例**: ⬜ 未着手 / 🔄 進行中 / ✅ 完了 / ⏸️ 保留
