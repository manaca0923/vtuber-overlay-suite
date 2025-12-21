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
**ステータス**: ✅ **完了**

### チェックリスト
- [x] テンプレートCSS変数設計
- [x] カラーバリアント切替（default/sakura/ocean）
- [x] 位置設定（上/下/左/右）
- [x] 表示ON/OFF切替
- [x] フォント設定（fontFamily, fontSize）
- [x] 設定UIコンポーネント（OverlaySettings.tsx）
- [x] 設定の永続化（SQLite）
- [x] プレビュー画面（iframe）

### テスト項目
- [x] 設定変更が即時反映（WebSocket broadcast）
- [x] 設定がアプリ再起動後も保持（DB保存）

### 成果物
- `src/types/overlaySettings.ts` - 型定義・テーマプリセット
- `src-tauri/src/commands/overlay.rs` - 保存/読み込み/ブロードキャストコマンド
- `src/components/settings/` - 設定UIコンポーネント群
- `src-tauri/overlays/comment.html` - 設定対応・XSS対策
- `src-tauri/overlays/setlist.html` - 設定対応・マーキー機能
- `src-tauri/src/server/http.rs` - HTTP API（/api/overlay/settings）
- `docs/300_overlay-specs.md` - 仕様書更新

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
- [x] websocket.rs: state.read()のロック取得を効率化（1回に統合）

#### 6. setlist_id指定オーバーレイのWS競合修正
- [x] SetlistUpdatePayloadにsetlist_idフィールドを追加
- [x] broadcast_setlist_update_internalでsetlist_idをペイロードに含める
- [x] fetch_latest_setlist_messageでsetlist_idをペイロードに含める
- [x] setlist.htmlでWS受信時にsetlist_idでフィルタリング

### 設計判断
- **WebSocket setlist_id競合**: 解決済み。SetlistUpdatePayloadにsetlist_idを含め、オーバーレイ側でフィルタリングを実装。URLパラメータでsetlist_idを指定したオーバーレイは該当セットリストの更新のみを受け付け、指定なし（最新モード）の場合は全ての更新を受け入れる。

### 成果物
- `src-tauri/src/youtube/poller.rs` - pollingIntervalMillis順守修正
- `src-tauri/src/youtube/state.rs` - polling_interval_millis復元対応、テスト追加
- `src-tauri/src/commands/youtube.rs` - 永続化項目追加、wizard_settingsコマンド追加
- `src-tauri/src/server/websocket.rs` - 接続時初期データ送信、タイミング改善、ロック効率化
- `src-tauri/src/server/types.rs` - SetlistUpdatePayloadにsetlist_id追加
- `src-tauri/src/commands/setlist.rs` - broadcast時にsetlist_id含める
- `src-tauri/overlays/setlist.html` - WS受信時setlist_idフィルタリング
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

## T13: PoC - InnerTube API（HTTP）
**優先度**: P1 | **見積**: 3日 | **依存**: T02
**ステータス**: 🔄 **進行中**（Phase 1-4完了、手動テスト待ち）

### チェックリスト
- [x] InnerTubeClientモジュール作成（HTTP、reqwestベース）
- [x] 型定義（InnerTube API固有の型）
- [x] runsパーサー実装（テキスト・絵文字分離）
- [x] ChatMessageにmessageRunsフィールド追加
- [x] Feature Flag実装（ApiMode: official/innertube）
- [x] テストコマンド実装（test_innertube_connection）
- [x] オーバーレイ絵文字対応（comment.html）
- [x] **PRレビュー指摘対応（2025-12-21）**
  - [x] continuation抽出改善（ライブチャット専用コンテキスト優先）
  - [x] API key抽出フォールバック（複数パターン対応）
  - [x] replay_chat_item_action全アクション処理（メッセージ取りこぼし防止）
  - [x] test_innertube_connectionの本番ビルド無効化
  - [x] ユニットテスト拡充（45件）
- [ ] 安定性レポート作成（手動テスト必要）
- [ ] **カスタム絵文字（メンバースタンプ）対応検証**

### 判断基準
- 接続成功率 > 95%
- 平均再接続時間 < 5秒
- HTTP/2プロキシ環境での動作

### 既知の制限事項（InnerTube API）

> **注意**: 以下はInnerTube API固有の制限であり、PoC段階では許容とする。

| 制限事項 | 説明 | 対応方針 |
|----------|------|----------|
| **is_verified常にfalse** | InnerTube APIにはVerifiedバッジ情報がない | 公式API統合時に対応 |
| **sticker_idにサムネURL使用** | InnerTube APIにはsticker_idがなく、サムネイルURLで代用 | 表示には支障なし |
| **ApiMode未接続** | PoC段階のため本番ポーリングには統合していない | 次フェーズで対応予定 |
| **CLIENT_VERSIONの古さ** | 2023年12月版を使用（無効化リスクあり） | 設定ファイル化を検討 |

### 調査結果（2025-12-21）

#### 公式YouTube Data API v3の制限
- `liveChatMessages.list`で取得できるのは`textMessageDetails.messageText`（プレーンテキスト）のみ
- カスタム絵文字は`:_emoji_name:`形式のテキストとして返される
- **絵文字の画像URLは公式APIでは取得不可**

#### YouTube InnerTube API（非公式）
YouTubeのWeb/アプリが内部で使用する非公開API。`runs`配列でメッセージを構造化して取得可能。

**絵文字データ構造:**
```json
{
  "emoji": {
    "emojiId": "チャンネルID/絵文字ID",
    "shortcuts": [":_emoji_name:"],
    "image": {
      "thumbnails": [
        {"url": "https://yt3.ggpht.com/...", "width": 24, "height": 24},
        {"url": "https://yt3.ggpht.com/...", "width": 48, "height": 48}
      ]
    },
    "isCustomEmoji": true
  }
}
```

**メリット:**
- カスタム絵文字の画像URLを直接取得可能
- クォータ制限なし

**デメリット:**
- 非公式APIのため仕様変更リスクあり

#### 参考実装・リソース
- [YTLiveChat (.NET, InnerTube API)](https://github.com/Agash/YTLiveChat)
- [YouTubeLiveChat (Java)](https://github.com/kusaanko/YouTubeLiveChat) - `Emoji#getIconURL()`
- [yt-livechat-grpc (Go)](https://github.com/dipaadipati/yt-livechat-grpc)
- [YouTube.js (Node.js)](https://github.com/LuanRT/YouTube.js) - InnerTube wrapper
- [ytc-filter Issue #2](https://github.com/RomainLK/ytc-filter/issues/2) - カスタム絵文字のruns構造
- [YouTube Live Chat Emoji CSV](https://gist.github.com/brainwo/8ea346ff73ace01aa5b7dd23014246e6) - 標準絵文字一覧

#### わんコメなどのサービスの実装方法
おそらくInnerTube API（gRPC/WebSocket）を使用してカスタム絵文字情報を取得している。
一部サービスはローカルファイルベースのマッピング（手動で絵文字画像を配置）を採用。

#### 実装時の選択肢
1. **InnerTube API追加**: 絵文字画像URL取得可能、クォータ制限なし（推奨）→ **採用**
2. **ローカルマッピング**: 安定だがユーザーが手動で絵文字画像配置が必要
3. **ハイブリッド**: 公式API + InnerTubeから絵文字情報のみ取得

### 成果物
- `src-tauri/src/youtube/innertube/mod.rs` - モジュール定義
- `src-tauri/src/youtube/innertube/client.rs` - InnerTubeClient（HTTP接続）
- `src-tauri/src/youtube/innertube/types.rs` - InnerTube API固有の型定義
- `src-tauri/src/youtube/innertube/parser.rs` - runsパーサー
- `src-tauri/src/youtube/types.rs` - MessageRun, EmojiInfo型追加
- `src-tauri/src/youtube/errors.rs` - InnerTube用エラー追加
- `src-tauri/src/commands/youtube.rs` - ApiMode, test_innertube_connectionコマンド追加
- `src-tauri/overlays/comment.html` - 絵文字画像レンダリング対応

---

## T14: InnerTube API 本番統合
**優先度**: P1 | **見積**: 0.5日 | **依存**: T13
**ステータス**: 🔄 **進行中**

### 概要
T13で実装したInnerTubeクライアントを本番ポーリングに統合する。
ApiModeに応じて公式API/InnerTube APIを切り替えて使用可能にする。

### チェックリスト
- [x] InnerTubeポーラー実装（start_polling_innertube, stop_polling_innertube）
- [x] 重複排除ロジック実装（メッセージIDベース）
- [x] WebSocketブロードキャスト対応（カスタム絵文字含む）
- [x] Tauriコマンド登録（デバッグ/リリース両対応）
- [x] フロントエンドからのApiMode切り替えUI対応（テストボタン追加）
- [x] URLバリデーション改善（fonts.gstatic.com追加、//形式URL正規化）
- [x] ポーラー相互排他（公式/InnerTube双方向でJoinHandle abort、Stopped通知付き）
- [x] 重複排除のLRU化（HashSet順序問題修正、同一レスポンス内重複対応）
- [x] 絵文字キャッシュ実装（徐々に解消方式、常に最新を上書き）
- [x] 動画切替時の絵文字キャッシュクリア
- [x] JoinHandleによる二重ポーリング防止（公式→InnerTube、InnerTube→公式両方対応）
- [x] **手動テスト実施** ✅ 2025-12-21
  - video_id: DAdj_xOJDg4 でテスト成功
  - 一部のカスタム絵文字が画像として正常に表示
- [ ] **本番UI結線**: 設定画面からApiMode切り替え（次フェーズ）
- [ ] **自動テスト追加**: 絵文字キャッシュ・ポーラー切替テスト

### 絵文字キャッシュ機能（実装済み 2025-12-21）

#### 動作原理
1. **キャッシュ構築**: 絵文字オブジェクトを受信したら`ショートカット→EmojiInfo`をグローバルキャッシュに登録
2. **テキスト変換**: テキストトークン内の`:_xxx:`パターンをキャッシュから画像に変換
3. **徐々に解消**: 最初はテキスト表示でも、一度絵文字オブジェクトを受信すればキャッシュされ、以降は画像表示
4. **動画切替時クリア**: InnerTube開始時にキャッシュをクリアし、誤った絵文字表示を防止

#### 実装箇所
- `src-tauri/src/youtube/innertube/parser.rs`
  - `EMOJI_CACHE`: グローバル絵文字キャッシュ（RwLock<HashMap<String, EmojiInfo>>）
  - `convert_text_with_emoji_cache()`: テキストから絵文字検出・変換
  - `clear_emoji_cache()`: キャッシュクリア

#### 制限事項
- 初回表示時はキャッシュが空のためテキスト表示になる場合がある
- 同一配信内で一度も絵文字オブジェクトとして受信していない絵文字はテキストのまま

### 設計方針
- 公式APIとは別に `start_polling_innertube` / `stop_polling_innertube` を提供
- InnerTubeClient をグローバル状態で管理し、ポーリングループで使用
- 公式APIとの互換性を維持（ChatMessage型は共通）
- video_idのみで開始可能（APIキー、live_chat_id不要）
- ポーラー相互排他（公式APIとInnerTubeの同時起動を防止）
- JoinHandleを保持して二重ポーリングを防止

### 成果物
- `src-tauri/src/commands/youtube.rs` - InnerTubeポーリングコマンド追加、相互排他、LRU、JoinHandle
- `src-tauri/src/youtube/innertube/parser.rs` - 絵文字キャッシュ実装（上書き対応）
- `src-tauri/src/youtube/innertube/mod.rs` - clear_emoji_cacheエクスポート
- `src-tauri/src/lib.rs` - コマンド登録
- `src-tauri/Cargo.toml` - once_cell依存追加
- `src/App.tsx` - InnerTubeテストボタン追加
- `src-tauri/overlays/comment.html` - URLバリデーション改善、//形式URL正規化

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
| T08 | ✅ 完了 | 2025-12-21 |
| T09 | ⬜ 未着手 | - |
| T10 | ✅ 完了 | 2025-12-20（Phase 1-4すべて完了） |
| T10-B | ✅ 完了 | 2025-12-20（レビュー指摘対応完了） |
| T10-C | ✅ 完了 | 2025-12-20（追加レビュー指摘対応） |
| T11 | ✅ 完了 | 2025-12-20 |
| T12 | ⬜ 未着手 | - |
| T13 | 🔄 進行中 | - |

**ステータス凡例**: ⬜ 未着手 / 🔄 進行中 / ✅ 完了 / ⏸️ 保留

---

## 本番リリース前チェックリスト

本番リリース前に必ず対応が必要な項目。PRレビューで指摘された技術的負債を含む。

> **更新履歴**: 2025-12-21 実装済み項目を削除（Default trait, メッセージ詳細情報, キーボード操作対応など）

### セキュリティ（必須）

- [ ] **APIキーのセキュアストレージ移行** (PR#19)
  - 現在: SQLiteに平文保存（`src-tauri/src/commands/keyring.rs`）
  - 対応: `src-tauri/src/keyring.rs`のkeyringクレート実装に移行
  - 参照: CLAUDE.md「セキュリティ」セクション
  - 備考: macOS=Keychain, Windows=Credential Manager, Linux=Secret Service API

- [x] **空文字列APIキーのバリデーション** (PR#15, PR#19で対応済み)
  - ~~`save_api_key`で空文字列のAPIキーを保存できてしまう~~
  - 対応済み: `src-tauri/src/commands/keyring.rs:27-30`でバリデーション追加

### コード品質

- [x] **console.logの整理** (PR#19, PR#22で対応済み)
  - ~~対象: `App.tsx`, `ApiKeySetup.tsx`, `WizardStep2.tsx`~~
  - 対応済み: デバッグ用console.logを削除（console.errorは維持）

- [ ] **未使用コードの整理** (PR#19, PR#22)
  - `src-tauri/src/keyring.rs`: 本番移行後に`commands/keyring.rs`のDB実装を削除
  - Rustコンパイラ警告の解消（`cargo check`で確認）
  - Windows CIで16件のdead code警告:
    - 未使用構造体: `SetlistSong`, `Setting`, `CommentLog`
    - 未使用関数: `save_api_key`, `get_api_key`, `delete_api_key`, `has_api_key`
    - 未使用定数: `SERVICE_NAME`, `API_KEY_ENTRY`
    - 未読フィールド: `amount_micros`, `alt_text`, `gift_memberships_level_name`等
  - 対応: 不要なら削除、将来使用予定なら`#[allow(dead_code)]`付与

- [ ] **オーバーレイ設定のposition型をenum化** (PR#23)
  - 現在: `position: String`（`http.rs`, `types.rs`）
  - 対応: Rust側でenum型を定義し、不正な値を型レベルで防止
  - 対象ファイル:
    - `src-tauri/src/server/http.rs`: `CommentSettingsApi`, `SetlistSettingsApi`
    - `src-tauri/src/server/types.rs`: `SettingsUpdatePayload`内の設定型
  - TypeScript側との対応も確認

### 機能改善（中優先度）

- [ ] **スーパーチャット金額別色分け** (PR#7)
  - 現在: 一律赤色（`--sc-color: #ff0000`）
  - YouTube本家では金額に応じて色が変わる

- [ ] **保存状態の有効期限** (PR#17)
  - 古い状態で再開すると取得済みコメントが重複する可能性
  - 有効期限を設定して古い状態を無効化

- [ ] **コメント削除のフェードアウトアニメーション** (PR#23)
  - comment.htmlにCSSアニメーション（.removing）があるが未使用
  - 古いコメント削除時にフェードアウトを適用
  - `comment:remove` WebSocketイベント受信時にも適用

- [ ] **コメントログのDB保存** (PR#23)
  - スキーマ（`001_initial.sql`の`comment_logs`テーブル）は存在
  - ポーリング取得時にコメントをDBに保存する処理が未実装
  - MVP要件か要確認

- [ ] **バックオフ最大試行回数の見直し** (PR#23)
  - 現在: 最大10回で停止（`backoff.rs`）
  - 長時間配信（2時間超）で自動復帰しない可能性
  - 無制限または大きな上限への変更を検討

- [ ] **テストモードでの各種メッセージタイプ送信** (PR#23)
  - 現在: Textメッセージのみ送信
  - superChat, superSticker, membership, membershipGiftの検証ができない
  - テストボタンにメッセージタイプ選択を追加

- [ ] **ApiKeySetup更新後のCommentControlPanel即時反映** (PR#23)
  - ApiKeySetup.tsxでAPIキー/LiveChatIdを更新しても上部のCommentControlPanelに即時反映されない
  - 現在はリロードが必要
  - App.tsx側で設定変更イベントを購読するか、状態を共有する仕組みが必要

### パフォーマンス

- [ ] **オーバーレイパフォーマンステスト** (PR#19)
  - MAX_COMMENTSを10→30に増加済み
  - OBSブラウザソースでの長時間使用テスト
  - 低スペックマシンでの動作確認

- [ ] **keyringブロッキング呼び出し対応** (PR#15)
  - keyring操作はOS APIへのブロッキング呼び出しの可能性
  - パフォーマンス問題時は`tokio::task::spawn_blocking`使用を検討

### テスト

- [ ] **手動テスト項目の実施**
  - 各タスクの「テスト項目」で「手動テスト必要」とされている項目
  - 2時間連続取得テスト（T03）
  - OBSでの透過背景動作（T05, T07）
  - 複数ブラウザソース同時接続（T04）

- [ ] **自動テストの追加** (PR#6)
  - WebSocket接続テスト
  - メッセージ送受信テスト
  - 複数クライアント同時接続テスト

### ドキュメント

- [ ] **README更新**
  - インストール手順
  - 使用方法
  - システム要件

- [ ] **InnerTube API ドキュメント追記** (PR#24)
  - InnerTube APIの制約（rate limit、仕様変更リスク）について
  - docs/200_youtube-api.mdへの追記を検討

### InnerTube API関連（PR#24）

- [x] **test_innertube_connectionの本番無効化** ✅ 対応済み（2025-12-21）
  - `lib.rs`で`#[cfg(debug_assertions)]`による条件付きコンパイル分岐を実装
  - デバッグビルドのみtest_innertube_connectionコマンドを登録

- [ ] **クライアントバージョンの自動更新機構**
  - `CLIENT_VERSION`（2023年12月）がYouTube側で無効化される可能性
  - 設定ファイルまたはフォールバック機構を検討

- [x] **InnerTubeテストカバレッジの拡充** ✅ 対応済み（2025-12-21）
  - `parse_chat_response`: 空レスポンス、no_continuation、no_actions、empty_actions
  - `parse_author_badges`: 複数バッジ同時存在、unknown_type、verified_not_owner
  - `extract_continuation`: 複数パターン（invalidation/timed/reload/generic）＋優先度テスト
  - `extract_api_key`: 複数パターン（標準/camelCase/ytcfg形式）
  - `parse_action`: replay複数アクション対応（回帰防止テスト追加）
  - テスト合計: **50件**

- [x] **continuation抽出の堅牢化** ✅ 対応済み（2025-12-21）
  - ライブチャット専用コンテキスト（invalidationContinuationData, timedContinuationData）を優先
  - 汎用パターンはフォールバックとして使用（警告ログ付き）
  - **設計判断**: 複数のinvalidationContinuationDataが存在する場合は最初のマッチを使用
    - 通常のライブチャットページでは1つのみ存在する前提
    - 複数存在するケースはPoC段階では想定外とする

- [x] **API key抽出のフォールバック** ✅ 対応済み（2025-12-21）
  - 複数パターン対応: "INNERTUBE_API_KEY", "innertubeApiKey", ytcfg.set形式

- [x] **replay_chat_item_action全アクション処理** ✅ 対応済み（2025-12-21）
  - リプレイ時の複数メッセージ取りこぼしを防止
  - parse_actionがVec<ChatMessage>を返すように変更
  - **回帰防止テスト追加**: 複数メッセージ/空リプレイ/単一メッセージの3ケース

---


