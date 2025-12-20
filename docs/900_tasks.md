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

## T10-B: コードレビュー指摘対応（追加タスク）
**優先度**: P0 | **見積**: 2日 | **依存**: T10
**ステータス**: ✅ **完了**

### 背景
T10完了後のコードレビューで指摘された未完成箇所の対応

### チェックリスト

#### 1. WebSocket配信経路の接続（高優先）
- [x] ポーリング開始/停止UI（CommentControlPanel.tsx）を実装
- [x] フロントからstart_polling/stop_pollingを呼び出す
- [x] ポーリング状態の表示（実行中/停止中）

#### 2. ポーリング状態/クォータの可視化（高優先）
- [x] フロントでpolling-eventを購読
- [x] クォータ残量の推定表示UI
- [x] エラー通知の可視化

#### 3. セットリストオーバーレイの初期同期（高優先）
- [x] WebSocket接続時に初期データを送信（broadcast_setlist_updateコマンド追加）
- [x] currentIndex == -1時のオーバーレイ表示を修正
- [x] HTTP API経由での初期データ取得（`/api/setlist/{id}`エンドポイント追加）
- [x] オーバーレイでURLパラメータからsetlist_idを取得し自動フェッチ

#### 4. APIキー保存の再利用（高優先）
- [x] メイン画面起動時に保存済みAPIキーを読み込む
- [x] 自動入力でそのままポーリング開始可能に

#### 5. メッセージ種別の対応（中優先）
- [x] superStickerの詳細（ステッカーID取得）
- [x] membershipのレベル取得
- [x] membershipGiftのギフト数取得
- [x] オーバーレイ側の表示実装

#### 6. ページネーション状態の永続化（中優先）
- [x] nextPageTokenをDBに保存
- [x] クォータ使用量の永続化
- [x] アプリ再起動時の復元処理（「続きから開始」ボタン）
- [x] StateUpdateイベントにnext_page_tokenとpolling_interval_millisを追加
- [x] 停止時に最新のポーリング状態を取得して保存（10回に1回のstateUpdate問題を解消）

#### 7. セットリストオーバーレイ初期表示改善（追加修正）
- [x] `/api/setlist/latest`エンドポイント追加（setlist_id未指定時に最新セットリストを返す）
- [x] オーバーレイでsetlist_id未指定時も自動で最新セットリストを取得

### 成果物
- `src/components/CommentControlPanel.tsx` - コメント制御パネル（ポーリング制御、状態表示、クォータ可視化）
- `src-tauri/src/commands/youtube.rs` - save_polling_state/load_polling_stateコマンド追加
- `src-tauri/src/youtube/poller.rs` - start_with_stateメソッド追加
- `src-tauri/src/youtube/state.rs` - with_saved_stateコンストラクタ追加
- `src-tauri/overlays/comment.html` - 全メッセージ種別のスタイル・表示対応
- `src-tauri/overlays/setlist.html` - currentIndex === -1時の表示修正、HTTP API初期フェッチ追加
- `src-tauri/src/commands/setlist.rs` - broadcast_setlist_updateコマンド追加
- `src-tauri/src/server/http.rs` - セットリスト取得API（`/api/setlist/{id}`）追加

---

## T10-C: 追加レビュー指摘対応
**優先度**: P0 | **見積**: 1日 | **依存**: T10-B
**ステータス**: ✅ **完了**

### 背景
T10-Bマージ後のレビューで指摘された追加修正項目

### チェックリスト

#### 1. pollingIntervalMillis順守の修正（高優先）
- [x] poller.rsでレスポンス受信後の新しい間隔でsleepするよう修正
- [x] 状態更新後に最新のpolling_intervalを取得して使用

#### 2. ウィザード入力値の引き継ぎ・保存（高優先）
- [x] ウィザードで入力したvideoId/liveChatIdをメイン画面に引き継ぎ
- [x] 設定をDBまたはsettingsに永続化（save_wizard_settings/load_wizard_settingsコマンド）
- [x] ApiKeySetupで保存済み設定を自動読み込み

#### 3. WebSocket接続時のセットリスト初期送信（高優先）
- [x] websocket.rsで接続完了時に最新セットリストを送信
- [x] DBアクセスをピア登録前に実行（タイミング改善）
- 注: HTTP取得失敗時はHTTP APIで取得可能（既存実装）

#### 4. polling_interval_millisの永続化（中優先）
- [x] save_polling_stateでpolling_interval_millisを保存
- [x] load_polling_stateでpolling_interval_millisを復元
- [x] state.rsのwith_saved_stateでpolling_interval_millisを受け取る
- [x] 後方互換性コメント追加

#### 5. 追加改善（レビュー推奨）
- [x] Wizard.tsx: 設定保存失敗時に2秒間警告を表示してから完了
- [x] websocket.rs: 初期送信ログをdebugレベルに変更
- [x] state.rs: with_saved_stateのユニットテスト追加（3ケース）
- [x] websocket.rs: 空行重複修正
- [x] PollingStateData: polling_interval_millisの後方互換性コメント追加

### 設計判断
- **WebSocket setlist_id競合**: 現時点ではsetlist_id指定オーバーレイは未実装のため、最新セットリスト固定で問題なし。将来実装時はSetlistUpdatePayloadにsetlist_idを含めてクライアントでフィルタする設計を検討。

### 成果物
- `src-tauri/src/youtube/poller.rs` - pollingIntervalMillis順守修正
- `src-tauri/src/youtube/state.rs` - polling_interval_millis復元対応、テスト追加
- `src-tauri/src/commands/youtube.rs` - 永続化項目追加、wizard_settingsコマンド追加
- `src-tauri/src/server/websocket.rs` - 接続時初期データ送信、タイミング改善
- `src/components/wizard/Wizard.tsx` - 入力値保存、警告表示改善
- `src/components/ApiKeySetup.tsx` - wizard設定の自動読み込み

---

## T10: 初回設定ウィザード + テストモード
**優先度**: P1 | **見積**: 5日 | **依存**: T02, T05
**ステータス**: ✅ **完了**

### チェックリスト
- [x] ウィザードステップUI
- [x] Step 1: APIキー入力・検証
- [x] Step 2: 動画ID入力（またはURL）
- [x] Step 3: テンプレート選択
- [x] Step 4: OBS設定ガイド
- [x] テストモード（ダミーコメント生成）
- [x] ライブプレビュー（OBSブラウザソースURL表示＋コピー機能）

### テスト項目
- [x] 初回起動でウィザード表示（コード実装完了、手動テスト未実施）
- [x] 全ステップ完了でメイン画面へ（コード実装完了、手動テスト未実施）
- [ ] OBSで透過背景動作（手動テスト必要）
- [ ] 連続コメントでパフォーマンス維持（手動テスト必要）
- [ ] 異常な長文コメントのハンドリング（手動テスト必要）

### 成果物
- `src/components/wizard/Wizard.tsx` - ウィザードコンテナ
- `src/components/wizard/WizardStep1.tsx` - APIキー入力・検証
- `src/components/wizard/WizardStep2.tsx` - 動画ID入力
- `src/components/wizard/WizardStep3.tsx` - テンプレート選択
- `src/components/wizard/WizardStep4.tsx` - OBS設定ガイド
- `src/components/wizard/WizardNavigation.tsx` - ナビゲーション
- `src/components/TestModeButton.tsx` - テストモード
- `src-tauri/src/commands/youtube.rs` - `send_test_comment`コマンド
- `src/App.tsx` - ウィザードモード統合

---

## T11: 認証情報保護
**優先度**: P1 | **見積**: 2日 | **依存**: T02
**ステータス**: ✅ **完了**

### チェックリスト
- [x] keyring クレート導入
- [x] Windows: Credential Manager 連携
- [x] macOS: Keychain 連携
- [x] APIキー保存/取得/削除
- [x] ログマスキング実装

### テスト項目
- [x] APIキーが平文で保存されない
- [x] ログにAPIキーが出力されない

### 成果物
- `src-tauri/src/keyring.rs` - セキュアストレージ抽象化レイヤー
- `src-tauri/src/commands/keyring.rs` - APIキー保存/取得/削除コマンド
- `src-tauri/src/util.rs` - ログマスキングユーティリティ
- `src-tauri/src/youtube/client.rs` - YouTubeClientのDebugトレイトでAPIキーマスキング

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
| T10 | ✅ 完了 | 2025-12-20（Phase 1-4すべて完了） |
| T10-B | ✅ 完了 | 2025-12-20（レビュー指摘対応完了） |
| T10-C | ✅ 完了 | 2025-12-20（追加レビュー指摘対応） |
| T11 | ✅ 完了 | 2025-12-20 |
| T12 | ⬜ 未着手 | - |
| T13 | ⬜ 未着手 | - |

**ステータス凡例**: ⬜ 未着手 / 🔄 進行中 / ✅ 完了 / ⏸️ 保留
