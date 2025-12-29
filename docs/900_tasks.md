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

#### 追加実装（PR#33 2025-12-21）
- [x] 統合オーバーレイ（/overlay/combined）- コメント+セットリストを1つのURLで表示
- [x] レイアウトプリセット機能（streaming, talk, music, gaming, custom）
- [x] レイアウトプリセット選択UI（ビジュアルプレビュー付き）
- [x] OBS画面サイズ（1920x1080）対応プレビュー
- [x] ResizeObserverによるレスポンシブスケーリング

### テスト項目
- [x] 設定変更が即時反映（WebSocket broadcast）
- [x] 設定がアプリ再起動後も保持（DB保存）
- [x] 統合オーバーレイでコメント・セットリスト同時表示
- [x] レイアウトプリセット切替で配置変更

### 成果物
- `src/types/overlaySettings.ts` - 型定義・テーマプリセット・レイアウトプリセット
- `src-tauri/src/commands/overlay.rs` - 保存/読み込み/ブロードキャストコマンド
- `src/components/settings/` - 設定UIコンポーネント群
- `src/components/settings/LayoutPresetSelector.tsx` - レイアウトプリセット選択UI
- `src/components/settings/OverlayPreview.tsx` - OBS画面サイズ対応プレビュー
- `src-tauri/overlays/comment.html` - 設定対応・XSS対策
- `src-tauri/overlays/setlist.html` - 設定対応・マーキー機能
- `src-tauri/overlays/combined.html` - 統合オーバーレイ
- `src-tauri/src/server/http.rs` - HTTP API（/api/overlay/settings, /overlay/combined）
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
**ステータス**: ✅ **完了**（2025-12-23）

### チェックリスト
- [x] Tauri Updater 設定（tauri.conf.json, Cargo.toml）
- [x] 更新サーバー設定（GitHub Releases）
- [x] 起動時更新チェック（UpdateChecker.tsx）
- [x] 更新通知ダイアログ（ダウンロード・再起動UI）
- [x] Windows インストーラー（.msi, .exe）- GitHub Actionsで生成
- [x] macOS インストーラー（.dmg）- GitHub Actionsで生成
- [x] 自動更新フロー実装

### 成果物
- `src-tauri/tauri.conf.json` - Updater設定追加
- `src-tauri/Cargo.toml` - tauri-plugin-updater, tauri-plugin-process追加
- `src-tauri/src/lib.rs` - プラグイン初期化
- `src-tauri/capabilities/default.json` - updater, processパーミッション
- `src/components/UpdateChecker.tsx` - 更新チェック・通知UI
- `.github/workflows/release.yml` - リリースワークフロー
- `docs/500_release.md` - リリース手順書

### 初回リリース前の手動作業
- [ ] 署名キー生成（`npx tauri signer generate`）
- [ ] tauri.conf.jsonにpubkey設定
- [ ] GitHub Secretsに秘密鍵登録

### 将来的改善（PR#42）
- [ ] **リトライ回数制限**: 連続失敗時のバックオフ・リトライ制限
- [ ] **ダウンロードキャンセル機能**: ダウンロード中の中断機能
- [ ] **「このバージョンをスキップ」機能**: dismiss状態の永続化
- [x] **pubkey空チェックCI**: ワークフロー側でpubkey設定を検証するステップ (PR#71)
- [x] **リリースノート自動生成**: CHANGELOG.mdまたはコミット履歴からの自動生成 (PR#72)
- [x] **リリースノートBREAKING CHANGE対応** (PR#72レビュー, PR#75): `feat!:`や`fix!:`などの破壊的変更表記への対応
- [x] **リリースノートperf/style分類** (PR#72レビュー, PR#75): `perf:`をパフォーマンス改善セクション、`style:`をスキップ対象に追加
- [x] **リリースノートperf!/revert対応** (PR#75レビュー, PR#76): `perf!:`の破壊的変更対応、`revert:`を🔄リバートセクションに分類

### テスト項目
- [ ] 新バージョン検出→通知（手動テスト必要）
- [ ] 更新ダウンロード・インストール（手動テスト必要）

---

## T13: PoC - InnerTube API（HTTP）
**優先度**: P1 | **見積**: 3日 | **依存**: T02
**ステータス**: ✅ **完了**（2025-12-21）

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
**ステータス**: ✅ **完了**（2025-12-21）

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
| T12 | ✅ 完了 | 2025-12-23 |
| T13 | ✅ 完了 | 2025-12-21 |
| T14 | ✅ 完了 | 2025-12-21 |
| T15 | ✅ 完了 | 2025-12-21 |
| T16 | ✅ 完了 | 2025-12-23 |
| T20 | ✅ 完了 | 2025-12-25 |
| T21 | ✅ 完了 | 2025-12-25 |
| T22 | ✅ 完了 | 2025-12-25 |
| T23 | ✅ 完了 | 2025-12-25 |
| T24 | ✅ 完了 | 2025-12-26 |
| T25 | ✅ 完了 | 2025-12-27 |

**ステータス凡例**: ⬜ 未着手 / 🔄 進行中 / ✅ 完了 / ⏸️ 保留

---

## 本番リリース前チェックリスト

本番リリース前に必ず対応が必要な項目。PRレビューで指摘された技術的負債を含む。

> **更新履歴**: 2025-12-21 実装済み項目を削除（Default trait, メッセージ詳細情報, キーボード操作対応など）

### セキュリティ（必須）

- [x] **APIキーのセキュアストレージ移行** (PR#19, PR#26で対応済み)
  - ~~現在: SQLiteに平文保存~~
  - 対応済み: `src-tauri/src/keyring.rs`のkeyringクレート実装に移行完了
  - macOS=Keychain, Windows=Credential Manager, Linux=Secret Service API
  - 既存DBからの自動移行機能付き（get_api_key時に自動でkeyringに移行）
  - tokio::spawn_blockingでブロッキング呼び出しをラップ

- [x] **空文字列APIキーのバリデーション** (PR#15, PR#19で対応済み)
  - ~~`save_api_key`で空文字列のAPIキーを保存できてしまう~~
  - 対応済み: `src-tauri/src/commands/keyring.rs:27-30`でバリデーション追加

### コード品質

- [x] **console.logの整理** (PR#19, PR#22で対応済み)
  - ~~対象: `App.tsx`, `ApiKeySetup.tsx`, `WizardStep2.tsx`~~
  - 対応済み: デバッグ用console.logを削除（console.errorは維持）

- [x] **未使用コードの整理** (PR#19, PR#22, PR#25, PR#26で対応済み)
  - ~~Rustコンパイラ警告の解消（`cargo check`で確認）~~
  - 対応済み: 将来使用予定のコードに`#[allow(dead_code)]`を付与
  - `src-tauri/src/keyring.rs`: セキュアストレージ移行完了（DB実装からkeyring実装に移行済み）

- [x] **オーバーレイ設定のposition型をenum化** (PR#23, PR#27で対応済み)
  - ~~現在: `position: String`~~
  - 対応済み: Rust側でenum型を定義し、不正な値を型レベルで防止
  - `CommentPosition`: TopLeft, TopRight, BottomLeft, BottomRight
  - `SetlistPosition`: Top, Bottom, Left, Right
  - TypeScript側の型定義と一致（serde rename_allで変換）

- [x] **オーバーレイ共通ロジックの抽出（第1段階）** (PR#49, PR#61で対応)
  - ~~`combined.html`と`combined-v2.html`でJavaScriptロジックが重複（約300行）~~
  - 対応済み: `shared/overlay-core.js`に共通ロジックを抽出
  - 抽出済み: updateSetlistDisplay, fetchLatestSetlist, 定数類

- [x] **オーバーレイ共通ロジックの抽出（第2段階）** (PR#61レビュー) ✅ 対応済み（2025-12-27）
  - ~~現在: WebSocketManager, SettingsFetcher等のクラスは定義済みだが未使用~~
  - 対応済み: combined.htmlとcombined-v2.htmlでWebSocketManager, SettingsFetcherを使用
  - updateSetlistDisplayにDOM要素のnullチェック追加
  - fetchLatestSetlistにタイムアウト処理追加
  - 効果: 約110行のコード削減（267削除, 157追加）

- [x] **combined.htmlへのbfcacheハンドリング追加** (PR#62, PR#63) ✅ 対応済み（2025-12-27）
  - ~~現在: combined-v2.htmlにはbfcache対応（pagehide/pageshow）があるが、combined.htmlにはない~~
  - 対応済み: combined.htmlにpagehide/pageshowハンドラを追加
  - pagehide: event.persisted=falseの場合のみwsManager.cleanup()
  - pageshow: event.persisted=trueの場合にsettingsFetcher.reset()とwsManager.reinitialize()

- [x] **SettingsFetcherのhasFetched()リセット機能** (PR#62) ✅ 対応済み（2025-12-27）
  - ~~現在: bfcache復元時にfetchSucceededがリセットされない~~
  - 対応済み: reset()メソッドを追加し、combined-v2.htmlのbfcache復元時に呼び出し

- [x] **layout-v2.cssのセットリストスタイル重複** (PR#49) ✅ 対応済み（2025-12-28）
  - ~~`layout-v2.css:112-166`と`combined.html`のスタイル定義（`.setlist-item`等）が重複~~
  - 対応済み: `shared/setlist-common.css`に共通スタイルを抽出し、両ファイルから参照

### テスト（推奨）

- [x] **overlay-core.jsのユニットテスト** (PR#62 Codexレビュー) ✅ 対応済み（2025-12-28）
  - 対応済み: `src/utils/overlay-core.test.ts`に26個のテストを追加
  - テスト内容: WebSocketManager.reinitialize()、SettingsFetcher.reset()、updateSetlistDisplay()、validateTimeout()など

- [x] **JSON Schema の `$id` URL更新** (PR#51) ✅ 対応済み（2025-12-26）
  - ~~現在: `https://example.local/...`~~
  - 対応済み: 相対パス（`./template-mvp-1.0.json`）に変更
  - 対象ファイル: `src-tauri/schemas/template-mvp-1.0.json`

- [x] **コンポーネントタイプの同期維持** (PR#51) ✅ 対応済み（2025-12-28）
  - ~~現在: RustとTypeScriptで`ComponentType`の列挙型が手動で同期~~
  - 対応済み: `scripts/validate-component-types.ts`を追加し、JSON Schema/TypeScript/Rust間の同期を検証
  - `npm run validate:types`で実行可能

- [x] **CommentControlPanel設定読み込みの統合** (PR#60) ✅ 対応済み（PR#65）
  - ~~現在: `loadApiMode`と`loadUseBundledKey`が別々のuseEffectで実装~~
  - 対応済み: 2つのuseEffectを単一の`loadInitialSettings`関数に統合
  - 対象ファイル: `src/components/CommentControlPanel.tsx`

- [x] **layout_type の検証がない** (PR#51) ✅ 対応済み（PR#65）
  - ~~現在: `layout_type`は"threeColumn"固定だが、バリデーションで検証されていない~~
  - 対応済み: `LayoutType` enumを追加し、型レベルで検証（デシリアライズ時に不正値はエラー）
  - 対象ファイル: `src-tauri/src/server/template_types.rs`

- [x] **コンポーネントIDの一意性チェック** (PR#51) ✅ 既に実装済み
  - ~~現在: slot重複チェックはあるが、コンポーネントIDの重複チェックがない~~
  - 対応済み: `has_id_duplicates()`が`template_types.rs`に実装済み、`template.rs`で使用
  - テストも追加済み

- [x] **slotIdの型安全性** (PR#50) ✅ 対応済み（2025-12-28）
  - ~~現在: `toSlotId`/`cssIdToSlotId`の正規表現の意図が曖昧~~
  - 対応済み: カラム名（left/center/right）を明示的にチェックする方式に変更
  - 対象ファイル: `src-tauri/overlays/shared/slots.js`, `src/types/slot.ts`
  - `src/types/slot.test.ts`にテストを追加

- [x] **SLOT定義の単一ソース化** (PR#50) ✅ 対応済み（PR#65）
  - ~~現在: slot定義が3箇所（Rust, TypeScript, JavaScript）に分散~~
  - 対応済み: `validate-slot-types.ts`スクリプトを追加し、4箇所（JSON Schema, TypeScript, Rust, JavaScript）の同期を検証
  - 対象ファイル: `scripts/validate-slot-types.ts`, `package.json`（`validate:slots`スクリプト追加）

- [x] **validate-component-types.tsのraw string内`"`対応** (PR#64) ✅ スキップ（PR#65）
  - 現在: `r##"foo"bar"##` のように`"`を含むraw stringで途中切れする可能性
  - 対応: 現時点ではserde(rename = r#"..."#)形式は未使用のため対応不要（コメントに制限事項を明記済み）
  - 対象ファイル: `scripts/validate-component-types.ts`

- [x] **overlay-core.test.tsのprocess.cwd()依存** (PR#64) ✅ 対応済み（PR#65）
  - ~~現在: テスト実行ディレクトリがリポジトリ直下でない場合に読み込み失敗~~
  - 対応済み: `import.meta.url`からの相対解決を優先し、`process.cwd()`をフォールバックに
  - 対象ファイル: `src/utils/overlay-core.test.ts`

### セキュリティ（将来課題）

- [x] **テンプレートstyleフィールドのXSS考慮** (PR#51, 調査完了)
  - 現在: `style`フィールドが`serde_json::Value`で定義されており任意のJSONを受け入れる
  - 調査結果（2024-12時点）:
    - `innerHTML`は要素クリア専用（`innerHTML = ''`）で使用
    - ユーザー入力は`textContent`で設定（XSS安全）
    - `sanitizeFontFamily`で危険文字を除去（CSSインジェクション対策済み）
    - 数値は`isValidNumber`で検証
  - 結論: 現状XSS脆弱性なし。将来的にスキーマ検証を検討可能だが優先度低
  - 対象ファイル: `src-tauri/src/server/template_types.rs`

### 機能改善（中優先度）

- [x] **スーパーチャット金額別色分け** (PR#7, PR#28で対応済み)
  - ~~現在: 一律赤色~~
  - 対応済み: YouTube本家に準拠した7段階色分け
  - 青(¥100-199) → 水色(¥200-499) → 緑(¥500-999) → 黄(¥1,000-1,999) → オレンジ(¥2,000-4,999) → マゼンタ(¥5,000-9,999) → 赤(¥10,000+)
  - 主要通貨対応（USD/EUR/GBP→JPY換算）

- [x] **保存状態の有効期限** (PR#17, PR#29で対応済み)
  - ~~古い状態で再開すると取得済みコメントが重複する可能性~~
  - 対応済み: 24時間の有効期限を設定し、期限切れ状態は自動削除
  - `load_polling_state`で`saved_at`をチェックし、期限切れならNoneを返す

- [x] **コメント削除のフェードアウトアニメーション** (PR#23, PR#25で対応済み)
  - ~~comment.htmlにCSSアニメーション（.removing）があるが未使用~~
  - 対応済み: removeCommentWithAnimation()で.removingクラスを使用
  - comment:removeイベントにも対応

- [x] **コメントログのDB保存** (PR#23, PR#32で対応済み)
  - ~~スキーマ（`001_initial.sql`の`comment_logs`テーブル）は存在~~
  - 対応済み: ポーリング取得時に`save_comments_to_db`でDBに保存
  - 公式API/InnerTube API両方に対応、重複はINSERT OR IGNOREで無視

- [x] **バックオフ最大試行回数の見直し** (PR#23, PR#30で対応済み)
  - ~~現在: 最大10回で停止（`backoff.rs`）~~
  - 対応済み: MAX_ATTEMPTSをu32::MAXに変更し事実上無制限に
  - ユーザーは手動でポーリング停止可能

- [x] **テストモードでの各種メッセージタイプ送信** (PR#23, PR#25で対応済み)
  - ~~現在: Textメッセージのみ送信~~
  - 対応済み: superChat, superSticker, membership, membershipGiftに対応
  - TestModeButtonにメッセージタイプ選択UIを追加

- [x] **ApiKeySetup更新後のCommentControlPanel即時反映** (PR#23, PR#31で対応済み)
  - ~~ApiKeySetup.tsxでAPIキー/LiveChatIdを更新しても上部のCommentControlPanelに即時反映されない~~
  - 対応済み: ApiKeySetupにonSettingsChangeコールバックを追加
  - App.tsx側でコールバックを受け取り、状態を即時更新

- [ ] **オーバーレイのカスタムCSS機能**
  - 設定画面にカスタムCSSテキストエリアを追加
  - APIで保存し、オーバーレイ読み込み時に適用
  - または`customCssUrl`パラメータで外部CSSファイルを読み込む方式
  - ユーザーがコメント欄の表示を自由にカスタマイズ可能に

- [ ] **3カラムレイアウト比率のCSS実装** (PR#48)
  - 現在: `--left-col: 22%`, `--center-col: 56%`, `--right-col: 22%` で定義
  - 問題: gutter（24px）を含めると実際のコンテンツエリアが100%を超える可能性
  - 対応: 実装時に `calc()` または `fr` 単位の使用を検討
  - 対象ファイル: `src-tauri/overlays/shared/layout-v2.css`（実装時に作成）

- [x] **QueueListのmaxItems範囲の統一** (PR#48) ✅ 対応済み（2025-12-26）
  - ~~現在: TypeScript型定義で `maxItems: 6-20`、slot配置表で `QueueList: 3-10` と記述不整合~~
  - 対応済み: クランプ範囲を3〜20に統一（SetList推奨:14、QueueList推奨:6）
  - 修正ファイル: `template_types.rs`, `template.ts`, `template-mvp-1.0.json`, `docs/400_data-models.md`

- [ ] **オーバーレイのドラッグ自由配置機能**
  - プリセットレイアウトではなく、プレビュー画面上でドラッグ&ドロップで配置
  - 技術検討: react-rnd または react-draggable の使用
  - 座標・サイズの設定保存、combined.html での動的配置対応
  - 優先度: 低（現在はプリセットレイアウトで対応）

- [x] **prompt()の代替UI実装** (PR#35, PR#43で対応済み)
  - ~~現在: `App.tsx`でwizardSettings未設定時に`prompt('Video ID:')`を使用~~
  - 対応済み: `VideoIdModal`コンポーネントでモーダルダイアログに置き換え
  - `alert()`もステータスメッセージUIに置き換え
  - ポーリング状態表示・ボタン無効化も追加

- [x] **統合オーバーレイのコード共通化** (PR#33, PR#36で対応済み)
  - ~~現在: `combined.html`と`comment.html`でコード重複~~
  - 対応済み: 共通モジュールを作成し抽出
    - `src-tauri/overlays/shared/overlay-common.css` - 共通CSSスタイル
    - `src-tauri/overlays/shared/comment-renderer.js` - コメントレンダリング関連JavaScript
  - `comment.html`と`combined.html`を共通モジュール使用に修正

- [ ] **スーパーチャット通貨換算の改善** (PR#33)
  - 現在: ハードコードされた為替レート（USD:150, EUR:160, GBP:190）
  - 将来: 為替レートAPIからの取得、または設定で変更可能に
  - 優先度: 低

- [ ] **レイアウト比率の合計チェックUI** (PR#51)
  - 現在: レイアウト比率の合計が1.0から離れている場合はwarningログのみ
  - 対応: 将来的にユーザーへのフィードバックとして合計値を表示することを検討
  - 優先度: 低

- [x] **useBundledKey状態の永続化** (PR#40) ✅ 対応済み（2025-12-27）
  - ~~現在: `useBundledKey`はローカルステートのみで管理、アプリ再起動時にリセット~~
  - 対応済み: wizard_settings（DBのsettingsテーブル）に保存、起動時に読み込み
  - 対象ファイル: `src/components/CommentControlPanel.tsx`, `src-tauri/src/commands/youtube.rs`

- [x] **コメントキューの即時/バッファモード分離** (PR#59) ✅ 対応済み（2025-12-27）
  - ~~現在: `displayQueue`と`isProcessingQueue`が即時モードとバッファモードで共有~~
  - 対応済み: `instantQueue`/`isProcessingInstant`と`bufferQueue`/`isProcessingBuffer`を分離
  - 対象ファイル: `src-tauri/overlays/shared/comment-renderer.js`

### パフォーマンス

- [x] **コメントDB保存のバッチ処理最適化** (PR#53) ✅ 対応済み（2025-12-26）
  - ~~現在: コメントを1件ずつINSERT~~
  - 対応済み: トランザクション内で複数INSERTを実行しI/O効率を向上
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [ ] **オーバーレイパフォーマンステスト** (PR#19)
  - MAX_COMMENTSを10→30に増加済み
  - OBSブラウザソースでの長時間使用テスト
  - 低スペックマシンでの動作確認

- [x] **絵文字キャッシュのサイズ制限** (PR#24) ✅ 対応済み（2025-12-26）
  - ~~現在: `EMOJI_CACHE`（RwLock<HashMap>）はサイズ制限なし~~
  - 対応済み: LRUキャッシュ（lruクレート）に置き換え、上限2000エントリに設定
  - 対象ファイル: `src-tauri/src/youtube/innertube/parser.rs`

- [x] **DensityManagerの定期クリーンアップ** (PR#54, PR#56で対応済み)
  - ~~現在: `recordUpdate()`呼び出し時のみ古いエントリを除去~~
  - ~~問題: 更新が止まった場合に履歴が残り続ける可能性~~
  - 対応済み: 定期クリーンアップタイマー（5秒間隔）と`destroy()`メソッドを実装
  - 対象ファイル: `src-tauri/overlays/shared/density-manager.js`

- [x] **DensityManager閾値の設定可能化** (PR#54, PR#68で対応済み)
  - ~~現在: `highDensityThreshold: 5`（2秒間に5回）がハードコード~~
  - ~~問題: 低スペック環境では閾値を下げたい場合がある~~
  - 対応済み: 設定画面「詳細」タブでスライダー調整可能（1-20）
  - 対象ファイル: `src/components/settings/PerformanceSettingsPanel.tsx`

- [x] **DensityManager destroy()呼び出しの追加** (PR#56)
  - ~~現在: `destroy()`メソッドは実装済みだが、呼び出し箇所がない~~
  - 対応済み: `combined-v2.html`で`pagehide`/`beforeunload`イベントで`cleanup()`関数を呼び出し
  - `UpdateBatcher`にも`destroy()`メソッドを追加

- [x] **keyringブロッキング呼び出し対応** (PR#15, PR#26で対応済み)
  - ~~keyring操作はOS APIへのブロッキング呼び出しの可能性~~
  - 対応済み: 全てのkeyring操作を`tokio::task::spawn_blocking`でラップ

- [x] **天気API keyringアクセスのspawn_blocking対応** (PR#57) ✅ 対応済み（2025-12-27）
  - ~~現在: `ensure_api_key_synced()`がasyncコンテキストで直接keyringを呼び出し~~
  - ~~問題: OS keyringへのアクセスはブロッキング呼び出しであり、UIスレッドをブロックする可能性~~
  - 対応済み: 全てのkeyring操作を`tokio::task::spawn_blocking`でラップ
    - `ensure_api_key_synced`: `get_weather_api_key`をspawn_blocking
    - `set_weather_api_key`: `save_weather_api_key`をspawn_blocking
    - `lib.rs`起動時: keyring読み取りをspawn_blocking
  - 対象ファイル: `src-tauri/src/commands/weather.rs`, `src-tauri/src/lib.rs`

- [x] **SQLITE_BUSYリトライ/backoff** (PR#55, PR#56で対応済み)
  - ~~現在: `busy_timeout`設定済み、フォールバック処理（個別INSERT）で対応~~
  - ~~問題: 高負荷時にトランザクション開始/コミットが失敗する可能性~~
  - 対応済み: リトライロジック（最大3回）とexponential backoff（100ms→200ms→400ms）を実装
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 並行書き込みテスト追加（ファイルベースDB使用）

- [x] **絵文字キャッシュのMutex contention最適化** (PR#55, PR#56で対応済み)
  - ~~現在: `Mutex<LruCache>`で毎回ロックを取得してget()を呼び出し~~
  - ~~問題: 高スループット時にロック競合が発生しパース遅延が悪化する可能性~~
  - 対応済み: ショートカットの重複排除をロック取得前に行い、ユニークなショートカットのみget()
  - 対象ファイル: `src-tauri/src/youtube/innertube/parser.rs`

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

- [x] **TypeScript側のテンプレートバリデーションテスト** (PR#51) ✅ 対応済み（2025-12-26）
  - ~~Rust側にはテストがあるが、TypeScript側の`hasSlotDuplicates`と`hasIdDuplicates`関数のテストがない~~
  - 対応済み: Vitestでユニットテストを追加（14テストケース）
  - 対象ファイル: `src/types/template.test.ts`

- [x] **UpdateBatcher/DensityManagerのユニットテスト** (PR#54) ✅ 対応済み（2025-12-28）
  - パフォーマンス最適化モジュールのテストカバレッジ追加
  - 対応済み: Vitestでユニットテストを追加
    - UpdateBatcher: 17テスト（queue上書き、forceFlush、clear、setBatchIntervalクランプ、destroy）
    - DensityManager: 25テスト（recordUpdate、閾値判定、forceDegraded/forceNormal、setThresholdクランプ、destroy）
  - 対象ファイル: `src/utils/update-batcher.test.ts`, `src/utils/density-manager.test.ts`

- [x] **天気API（Open-Meteo）Geocodingエラーケースのテスト** (PR#58, PR#73で部分対応, PR#84で完了)
  - [x] `geocode_city`が`results: None`または空配列のとき`WeatherError::CityNotFound`になるケースのテスト
  - [x] Geocoding/Weather APIのHTTP非200時の`ApiError`生成経路のテスト（mockitoでHTTPモック実装）
  - [x] Geocoding APIがタイムアウトした場合に`WeatherError::Timeout`になるテスト（構造上タイムアウトテストは除外、Timeout型テストのみ）
  - [x] CityNotFound判定ロジックテストをHTTPモック導入時に統合テストに置き換え（PR#84でmockitoベース統合テスト追加）
  - 追加テスト: WeatherErrorフォーマット、GeocodingResponseパース（5件）、CityNotFound判定ロジック（3件）、HTTPモックテスト（7件）
  - 対象ファイル: `src-tauri/src/weather/mod.rs`, `src-tauri/src/weather/types.rs`

- [x] **コメント即時/バッファモード混在テスト** (PR#59 → PR#68で対応済み)
  - ~~`instant=true`と`instant=false`が同時に到着するケースで、公式API由来コメントが5秒均等表示のまま維持されることの確認~~
  - ~~`instant=true`連打時のスロットリング（`INSTANT_DISPLAY_INTERVAL`）が効いて重複が抑止されることの確認~~
  - 対応済み: CommentQueueManagerのユニットテスト17件追加
  - 対象ファイル: `src/utils/comment-queue-manager.test.ts`

- [x] **天気ウィジェット設定フォールバックテスト** (PR#59) ✅ 対応済み（2025-12-28）
  - `settings.weather.position`が未知値/欠落のとき、安全にデフォルトへフォールバックすることの確認
  - 対応済み: Vitestでユニットテストを追加（21テスト）
    - デフォルト値フォールバック（style未指定時）
    - style.temp=0がデフォルト値にならないこと
    - update()でのnull/undefined処理
    - getIconForCode()のマッピングとフォールバック
  - 対象ファイル: `src/utils/weather-widget.test.ts`

- [x] **SQLITE_BUSY並行書き込みテスト** (PR#55, PR#56で対応済み)
  - ~~2接続で同時書き込みし、busy_timeoutが正しく動作するかを検証~~
  - 対応済み: `test_concurrent_writes_with_retry`を追加
    - ファイルベースDB使用（in-memoryでは各接続が独立DBを持つため）
    - 2タスクから同時に30件ずつ書き込み
    - リトライ/フォールバックで60件全て保存されることを検証
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [ ] **save_comments_to_dbの総予算によるデータスキップ検討** (PR#56)
  - 現在: 2秒の総予算を超えると残りのメッセージをスキップ
  - 設計判断: SQLITE_BUSYなしでもスキップが発生する可能性がある（データ損失リスク）
  - 対応案:
    - 予算をチャンクごとにスコープするか、チャンク数でスケールする
    - スキップ数をログ/メトリクスで報告し、上流でリキュー可能にする
    - 現状はリアルタイム性を優先し、2秒以内に完了することを保証
    - **戻り値を構造化**: `{ saved: usize, failed: usize, skipped: usize }`を返し、呼び出し元に通知
    - **予算を設定可能に**: メッセージ数/チャンク数に比例させる、または設定ファイルで変更可能に
    - **テスト用に予算を注入可能に**: `test_concurrent_writes_with_retry`が2秒固定予算でフレーキーになる可能性あり（遅いディスク/CI環境）
    - **テスト追加**: 予算超過時のskippedカウントを検証するテスト（構造化戻り値実装後）
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 中（本番運用後にフィードバックを収集）

- [x] **busy_timeoutのタイムアウト保証に関するドキュメント明確化** (PR#56, PR#77)
  - 対応: ドキュメントコメントで「ロック待ちのみ制限」を明記
  - `RETRY_TOTAL_TIMEOUT_MS`定数と`set_busy_timeout`関数に制限範囲の説明を追加
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [x] **絵文字キャッシュのストレステスト/ベンチマーク** (PR#55, PR#78)
  - 並行アクセステスト: 8スレッド×100回=800呼び出し、86,636 calls/sec
  - レイテンシテスト: 50絵文字×100回、平均173μs/call
  - 対象ファイル: `src-tauri/src/youtube/innertube/parser.rs`

- [x] **pool.acquire()タイムアウト時のスキップログテスト** (PR#56, PR#79)
  - `test_pool_acquire_timeout_returns_busy`: 単一接続保持状態でacquireタイムアウト→Busy返却を検証
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [x] **天気API keyringフォールバックのテスト** (PR#57) → **不要化** (PR#58)
  - Open-Meteo移行によりAPIキー不要化、テスト対象が削除されたため不要

- [x] **YouTube `get_live_stream_stats` のHTTPステータスマッピングテスト** (PR#57 → PR#84で完了)
  - 404/400/5xxエラー時のYouTubeErrorマッピングを検証
  - テスト対象:
    - 403 quota exceeded → QuotaExceeded
    - 403 rate limit → RateLimitExceeded
    - 401 → InvalidApiKey
    - 404 → VideoNotFound
    - 400 → VideoNotFound
    - 5xx (500/502/503) → ApiError（一時的障害メッセージ）
    - その他ステータス (418など) → ApiError（予期しないエラー）
  - 対応済み: mockitoでHTTPモック実装（13テスト追加）
  - 対象ファイル: `src-tauri/src/youtube/client.rs`

- [ ] **`broadcast_weather_update(force_refresh=true)` のテスト** (PR#57)
  - キャッシュクリア→新規取得→ブロードキャストの動作を検証
  - テスト対象:
    - `force_refresh=true`でキャッシュがクリアされること
    - 新しいデータがWebSocketでブロードキャストされること
  - 対象ファイル: `src-tauri/src/commands/weather.rs`
  - 優先度: 中（サーバー/状態のモック化が必要）

- [x] **bfcache/リロード時のWebSocket再接続テスト** (PR#56) ✅ 対応済み（2025-12-28）
  - 対応済み: Vitestでユニットテストを追加（8テスト）
    - pagehideイベント（persisted=true/false）のハンドリング
    - pageshowイベント（persisted=true/false）のハンドリング
    - WebSocketManager.reinitialize()でのbfcache復元時再接続
    - SettingsFetcher.reset()後のfetchAndApply()実行可能性
    - cleanup()後のconnect()で新しい接続が作成されること
  - 対象ファイル: `src/utils/overlay-core.test.ts`

- [x] **テストヘルパー関数の共通化** (PR#66 → PR#67で実装)
  - `resolveScriptPath()`が4つのテストファイルで重複
  - 対応: `src/utils/test-helpers.ts`に共通モジュールとして抽出
  - 対象ファイル: `update-batcher.test.ts`, `density-manager.test.ts`, `weather-widget.test.ts`, `overlay-core.test.ts`

- [x] **UpdateBatcherのflush()後broadcast呼び出し確認テスト** (PR#66 → PR#67で実装)
  - `forceFlush()`後に`ComponentRegistry.broadcast()`が正しく呼び出されることを確認するテスト追加
  - 対象ファイル: `src/utils/update-batcher.test.ts`

- [x] **loadScriptContentのエラーハンドリング改善** (PR#67 → PR#69で対応済み)
  - ~~`fs.readFileSync`が失敗した場合のエラーメッセージに具体的なパスを含める~~
  - 対応済み: try-catchでラップし、ファイルパスと元のエラーメッセージを含む詳細なエラーを投げるように改善

- [x] **get_busy_timeoutをResult型に変更してBUSYエラー対応** (PR#56, PR#81で対応)
  - 現在: `get_busy_timeout()`失敗時は`None`を返し、リトライを停止
  - 問題: 一時的なBUSYエラーの場合、リトライすべきでは
  - 対応案:
    - `Result<u64, sqlx::Error>`に変更
    - BUSYエラーなら`TransactionResult::Busy`を返してリトライ
    - 非BUSYエラーならconn.detach()してOtherError
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低（PRAGMA busy_timeoutは通常BUSYにならない）
  - ✅ `GetBusyTimeoutResult`列挙型を導入、BUSYエラーとその他のエラーを区別

- [x] **busy_timeout復元のBUSYリトライ回数増加検討** (PR#56, PR#81で対応)
  - 現在: 20msバックオフ後1回リトライ、失敗時はdetach
  - 問題: 高負荷時にBUSYが連続すると接続をchurnしてプール容量が減少
  - 対応案:
    - 2-3回のリトライループに変更（例: 20ms, 40ms, 80ms）
    - リトライ上限に達した場合のみdetach
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低（本番運用でBUSY頻発時に検討）
  - ✅ 最大3回リトライ（exponential backoff: 20ms, 40ms, 80ms）に変更

- [x] **beforeunloadイベントのbfcache影響検討** (PR#56)
  - ~~現在: `beforeunload`を登録しているとブラウザがbfcacheを無効にする可能性がある~~
  - ~~問題: 通常ブラウザでの戻る/進む時にキャッシュから復元されない~~
  - 対応済み: `beforeunload`でのcleanup()呼び出しを削除し、`pagehide`のみを使用
    - `pagehide`で`event.persisted`チェック、falseの場合のみcleanup実行
    - OBSブラウザソース等のbfcache非対応環境でもpagehide(persisted=false)で確実にクリーンアップ
  - 対象ファイル: `src-tauri/overlays/combined-v2.html`

- [x] **トランザクション内でのデッドライン強制** (PR#56, PR#81で対応)
  - 現在: 2秒の総予算は`save_chunk_with_retry`ループで管理されているが、トランザクション内の遅いI/Oは制限されない
  - 問題: 遅いディスクI/O（SQLITE_BUSYなし）で2秒を超える可能性がある
  - 対応案:
    - `save_chunk_with_transaction_on_conn`にdeadlineパラメータを追加
    - 各INSERT前にデッドライン超過チェック
    - 超過時はrollbackして早期終了
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低（遅いディスクI/Oは稀なケース）
  - ✅ deadlineパラメータ追加、各INSERT前とコミット前にチェック

- [x] **遅いINSERTシミュレーションテスト** (PR#56, PR#81で対応)
  - 遅いINSERTをシミュレートし、デッドライン超過時に早期rollback + 部分コミットなしを検証
  - テスト対象: `save_chunk_with_transaction_on_conn`のデッドライン強制
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低
  - ✅ `test_deadline_enforcement_in_transaction`テスト追加

- [x] **テスト用DBファイルの分離改善** (PR#56, PR#81で対応)
  - 現在: PIDベースの命名（`test_{}.db`）で分離、テスト終了時に削除
  - 問題: テスト中断時に古いファイルが残り、再実行時にデータが残留する可能性
  - 対応案:
    - `tempfile`クレートを使用して自動削除されるテンポラリファイルを使用
    - または、テスト開始前にファイルが存在すれば削除するヘルパー関数を追加
    - ユニークなパス生成にUUIDを併用（PID + UUID）
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低（現状でも`CREATE TABLE IF NOT EXISTS`で問題回避、テスト終了時に削除処理あり）
  - ✅ `tempfile`クレートを導入、新規テストで使用

- [ ] **busy_timeout=0のfail-fast動作保持検討** (PR#56)
  - 現在: `original_timeout == 0`は「制限なし」として`u64::MAX`扱い → 500msにクランプ
  - 問題: SQLiteの`busy_timeout=0`は「即座にBUSYエラーを返す（fail-fast）」という意味
    - オペレーターが意図的に0を設定した場合、500ms待機するように動作が変わる
  - 対応案:
    - Option A: 0は0として扱い、busy_timeout=0で即座にBUSYを返す
    - Option B: 設定可能な`min_busy_timeout_ms`を追加し、0を上書きするかを制御
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低（プール設定でbusy_timeout=0を使うケースは稀）
  - 備考: 現在の実装は「リトライを優先する」設計判断に基づく

- [x] **busy_timeout=0のfail-fast動作テスト追加** (PR#56, PR#80で対応)
  - `busy_timeout=0`設定時にリトライパスが即座にBUSYを返す（または現在の動作を明示的に検証）
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低
  - ✅ `test_busy_timeout_zero_fail_fast`テスト追加済み

- [x] **既存テストのtempfileパターン統一** (PR#81レビュー, PR#82で対応)
  - 現在: 新規テストはtempfile使用、既存テストは手動削除パターン
  - 対応案: 既存テストもtempfileパターンに統一し、テスト中断時のファイル残存を完全に防止
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低
  - ✅ 9個のテストをtempfileパターンに統一

- [ ] **TransactionResult::DeadlineExceeded専用バリアント検討** (PR#81レビュー)
  - 現在: デッドライン超過時は`TransactionResult::Busy`を返す
  - 問題: デッドライン超過は「リトライしても同じ結果になる可能性が高い」ケース
  - 対応案: 専用の`TransactionResult::DeadlineExceeded`を追加して明示的に区別
  - 対象ファイル: `src-tauri/src/youtube/db.rs`
  - 優先度: 低（現状でも上位で予算チェックされるため実害なし）

### ドキュメント

- [x] **README更新** (2025-12-22対応済み)
  - ~~インストール手順~~
  - ~~使用方法~~
  - ~~システム要件~~
  - 特徴、OBS設定、URLパラメータ、トラブルシューティング追加

- [x] **InnerTube API ドキュメント追記** (PR#24) ✅ 完了
  - InnerTube APIの制約（rate limit、仕様変更リスク）について
  - docs/200_youtube-api.mdへ詳細追記済み

### InnerTube API関連（PR#24）

- [x] **test_innertube_connectionの本番無効化** ✅ 対応済み（2025-12-21）
  - `lib.rs`で`#[cfg(debug_assertions)]`による条件付きコンパイル分岐を実装
  - デバッグビルドのみtest_innertube_connectionコマンドを登録

- [x] **クライアントバージョンの自動更新機構** ✅ 対応済み（2025-12-22）
  - ライブチャットページHTMLから`clientVersion`を動的抽出
  - 取得失敗時はフォールバック値（2.20251201.01.00）を使用
  - テストケース4件追加

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

## T15: 本番リリース準備対応
**優先度**: P0 | **見積**: 0.5日 | **依存**: T14
**ステータス**: ✅ **完了**（2025-12-21）

### 背景
コードレビューで指摘された本番リリース前の必須対応項目

### チェックリスト

#### Critical/High（対応済み）
- [x] **overlaysフォルダのバンドル設定追加**
  - `tauri.conf.json`に`bundle.resources: ["overlays"]`を追加
  - 本番ビルドで`/overlay/*`が404/500にならないよう修正

- [x] **CSPにframe-srcとimg-srcを追加**
  - iframeプレビュー（OverlayPreview.tsx）がブロックされる問題を修正
  - `frame-src 'self' http://localhost:19800`
  - `img-src 'self' data: http://localhost:19800 https://*.ggpht.com https://*.googleusercontent.com https://*.ytimg.com`

- [x] **maxCount設定を完全削除**
  - 画面高さベースの自動調整に統一
  - フロントエンド: CommentSettingsPanel.tsx、overlaySettings.ts、OverlayPreview.tsx
  - バックエンド: overlay.rs、types.rs、http.rsからmax_count削除

#### Medium（対応済み）
- [x] **InnerTubeを本番機能として正式採用**
  - `docs/001_requirements.md`を更新
  - 認証不要のInnerTube APIをメインで使用
  - 公式API（YouTube Data API v3）はデバッグモードで利用可能

- [x] **公式APIをデバッグモードのみに制限**
  - `App.tsx`で`import.meta.env.DEV`による条件分岐
  - CommentControlPanel、ApiKeySetupはデバッグ時のみ表示
  - InnerTubeボタンを「コメント取得開始/停止」としてメイン機能化

- [x] **二重ポーリング対策**
  - `commands/youtube.rs`のstart_pollingでstop()後に200ms待機を追加
  - ロック解放→待機→ロック再取得で安全に切り替え

- [x] **WebSocket仕様書を実装に合わせて更新**
  - `docs/300_overlay-specs.md`: subscribe送信例を削除（未実装機能）
  - `docs/300_overlay-specs.md`: setlist:updateにsetlistIdフィールド追加

- [x] **YouTube API仕様書をInnerTube優先方針に更新**
  - `docs/200_youtube-api.md`: InnerTube APIをメイン、公式APIをデバッグ用と明記
  - `docs/200_youtube-api.md`: BYOK必須表記を「公式API使用時のみ」に修正

- [x] **関連ドキュメントをInnerTube優先方針に整合**
  - `docs/001_requirements.md`: 受け入れ基準をInnerTubeメインに更新
  - `docs/100_architecture.md`: 技術スタック表とシステム構成図をInnerTubeメインに更新
  - `docs/100_architecture.md`: 通信フロー図を `[YouTube InnerTube] ──(HTTP)──►` に更新（2025-12-22追記）

### 成果物
- `src-tauri/tauri.conf.json` - bundle.resources追加、CSP更新
- `src/types/overlaySettings.ts` - maxCount削除
- `src/components/settings/CommentSettingsPanel.tsx` - maxCount UI削除
- `src/components/settings/OverlayPreview.tsx` - maxCount URLパラメータ削除
- `src/App.tsx` - デバッグモード条件分岐、InnerTubeボタン名称変更、エラーハンドリング改善
- `src-tauri/src/commands/youtube.rs` - 二重ポーリング対策、定数化
- `src-tauri/src/commands/overlay.rs` - max_count削除
- `src-tauri/src/server/types.rs` - CommentSettingsPayloadからmax_count削除
- `src-tauri/src/server/http.rs` - CommentSettingsApiからmax_count削除
- `docs/001_requirements.md` - InnerTube正式採用記載、受け入れ基準更新
- `docs/100_architecture.md` - 技術スタック表・システム構成図をInnerTubeメインに更新
- `docs/200_youtube-api.md` - InnerTube優先方針追記、BYOK必須表記修正
- `docs/300_overlay-specs.md` - subscribe削除、setlistId追加
- `docs/400_data-models.md` - maxCount削除

---

## T16: YouTube公式API + streamList gRPC 実装
**優先度**: P0 | **見積**: 8日 | **依存**: T14
**ステータス**: ✅ **完了**（2025-12-23）

### 背景
正式運用に向けて、InnerTube API（非公式）に加えて公式YouTube Data API + streamList gRPCを実装し、ユーザーが切り替え可能にする。
マネタイズ対応のため、公式APIの利用が必要。

### 方針
- InnerTube / 公式API(ポーリング) / 公式API(gRPC) の3モード切り替え
- アプリ同梱キー + BYOK の両方サポート
- gRPC優先実装（クォータ消費が少ない）
- Remote Configは後回し（Phase 2以降で対応）

### チェックリスト

#### Phase 1: APIキー管理の拡張（1日）
- [x] `api_key_manager.rs` 作成（同梱キー+BYOK管理）
- [x] グローバルシングルトンApiKeyManager実装
- [x] Tauriコマンド追加（get_api_key_status, has_bundled_api_key, set_byok_key等）
- [x] lib.rsにコマンド登録
- [x] ビルド確認

#### Phase 2: gRPC基盤の構築（2日）
- [x] Cargo.tomlにtonic/prost依存追加
- [x] build.rsにproto生成処理追加
- [x] `proto/youtube_live_chat.proto` 作成
- [x] gRPCモジュール基盤作成（`youtube/grpc/mod.rs`, `client.rs`）

#### Phase 3: gRPCストリーミング実装（2日）
- [x] GrpcStreamClient実装 ✅ 2025-12-22（PR#39）
- [x] API keyメタデータ付与
- [x] server-streaming受信ループ
- [x] 指数バックオフ + ジッタによる再接続
- [x] messageIdベースの重複排除

#### Phase 4: 統合ポーラー作成（1日）
- [x] UnifiedPoller実装（3モード統一管理） ✅ 2025-12-22（PR#39）
- [x] ApiModeに`Grpc`を追加
- [x] フォールバック機構（gRPC→ポーリング）
- [x] 新規コマンド実装（start_unified_polling, stop_unified_polling）

#### Phase 5: フロントエンド統合（1日）
- [x] CommentControlPanel.tsx改修（モード切替UI） ✅ 2025-12-23（PR#40）
- [x] TypeScript型定義追加（src/types/api.ts）
- [x] InnerTube/gRPCステータスイベントリスナー
- [x] 統合ポーラーコマンド登録

#### Phase 6: テスト・ドキュメント（1日）
- [ ] 各モードの接続テスト（手動テスト）
- [ ] モード切り替えテスト（手動テスト）
- [ ] 再接続テスト（ネットワーク断）（手動テスト）
- [x] `docs/200_youtube-api.md` 更新 ✅ 2025-12-23

### PRレビュー対応（2025-12-25）

- [x] **統合ポーラーにWS/DB連携を追加** (PR#52レビュー)
  - 統合ポーラー（InnerTube/gRPC両方）からのコメントがWebSocketブロードキャストとSQLite保存されるよう修正
  - `unified_poller.rs`に`server_state`と`db_pool`パラメータを追加
  - `grpc/poller.rs`にDB保存処理を追加

- [x] **「続きから開始」ボタンの非表示化** (PR#52レビュー)
  - 統合ポーラーが保存状態復元をサポートするまでUIを非表示
  - `CommentControlPanel.tsx`でボタンセクションをコメントアウト

- [x] **楽曲編集でフィールドをクリア可能に** (PR#52レビュー)
  - `setlist.rs`のCOALESCEを削除し、オプショナルフィールドのクリアを許可
  - `SongForm.tsx`で空文字列を`null`として送信
  - `song.ts`の型定義を`null`許容に更新

- [x] **ドキュメント整合性修正** (PR#52レビュー)
  - `docs/400_data-models.md`: MessageType表記をcamelCaseに修正（superChat, superSticker）
  - `docs/400_data-models.md`: Song.tags型を`string | null`に修正
  - `docs/200_youtube-api.md`: gRPC優先・InnerTubeバックアップ方針に更新

- [x] **console.log整理（comment.html）** (PR#52レビュー)
  - 本番オーバーレイ用にデバッグログを削除
  - エラーハンドリングはコメントで説明

### 技術的負債（次イテレーションで対応予定）

以下はPR#40レビューで指摘された改善項目。機能実装完了後に対応予定。

- [ ] **グローバル状態の統合** (PR#40)
  - 現在: `UNIFIED_POLLER`, `INNERTUBE_*`, `AppState.poller` が並行して存在
  - 対応: 統合ポーラーに一元化してコード保守性を向上
  - @see `src-tauri/src/commands/youtube.rs:1051-1055`

- [ ] **公式APIモードの保存状態再開機能** (PR#40)
  - 現在: 統合ポーラー使用により一時的に無効化（`_useSavedState`パラメータ）
  - 対応: 統合ポーラーに保存状態復元機能を追加
  - @see `src/components/CommentControlPanel.tsx:285-286`

- [ ] **接続状態管理の最適化** (PR#40)
  - 現在: `isPolling`と`connectionStatus`を別々に管理
  - 提案: `connectionStatus`を拡張（`'disconnected' | 'connecting' | 'connected' | 'error'`）し、`isPolling`を派生状態化

### 技術スタック
- gRPC: tonic 0.12 + prost 0.13
- 接続先: `youtube.googleapis.com:443`
- 認証: `x-goog-api-key` メタデータ

### 成果物
- `src-tauri/src/youtube/api_key_manager.rs` - APIキー管理
- `src-tauri/src/youtube/grpc/` - gRPCクライアント
- `src-tauri/src/youtube/unified_poller.rs` - 統合ポーラー
- `src/components/CommentControlPanel.tsx` - モード切替UI

---

## Phase 2: 3カラムレイアウト実装（将来計画）

> **ステータス**: 設計完了、実装予定
>
> 3カラム・テンプレ要件仕様書 v1.1 に基づく段階的実装計画。

---

## T20: 3カラムレイアウト基盤
**優先度**: P1 | **見積**: 5日 | **依存**: T08
**ステータス**: ✅ **完了**（2025-12-25）

### 概要
既存オーバーレイシステムに3カラム固定レイアウト（22%/56%/22%）を追加。v1との後方互換性を維持。

### チェックリスト
- [x] CSS変数の拡張（overlay-common.css）
- [x] 3カラムHTML構造作成（combined-v2.html）
- [x] CSS Grid実装（layout-v2.css）
- [x] HTTPエンドポイント追加（/overlay/combined-v2）
- [x] v1/v2切替UI追加（LayoutPresetSelector.tsx）

### 成果物
- `src-tauri/overlays/combined-v2.html` - 3カラム統合オーバーレイ（11個のslot構造）
- `src-tauri/overlays/shared/layout-v2.css` - CSS Grid 3カラムレイアウト
- `src-tauri/overlays/shared/overlay-common.css` - v2用CSS変数追加
- `src-tauri/src/server/http.rs` - `/overlay/combined-v2`エンドポイント追加
- `src-tauri/src/server/types.rs` - `LayoutPreset::ThreeColumn`追加
- `src/types/overlaySettings.ts` - `three-column`プリセット追加
- `src/components/settings/LayoutPresetSelector.tsx` - 3カラムプレビュー追加
- `src/components/settings/OverlayPreview.tsx` - v2プレビューURL対応

---

## T21: slot・Design Token整備
**優先度**: P1 | **見積**: 3日 | **依存**: T20
**ステータス**: ✅ **完了**

### 概要
11個のslot配置システムとCSS変数によるDesign Token。

### チェックリスト
- [x] slot管理JavaScript作成（slots.js）
- [x] Design Token CSS作成（design-tokens.css）
- [x] TypeScript slot型定義（slot.ts）
- [x] Rust SlotId列挙型追加（server/types.rs）

### slot定義（11個）
| slot | 役割 |
|------|------|
| left.top | 時刻 |
| left.topBelow | 天気 |
| left.middle | コメント |
| left.lower | スパチャ |
| left.bottom | ロゴ |
| center.full | 主役 |
| right.top | ラベル |
| right.upper | セトリ |
| right.lowerLeft | KPI |
| right.lowerRight | 短冊 |
| right.bottom | 告知 |

### 成果物
- `src-tauri/overlays/shared/slots.js` - slot管理モジュール（SlotManager API）
- `src-tauri/overlays/shared/design-tokens.css` - Design Token CSS変数
- `src/types/slot.ts` - TypeScript slot型定義
- `src-tauri/src/server/types.rs` - SlotId列挙型追加

---

## T22: 型定義・JSON Schema
**優先度**: P1 | **見積**: 3日 | **依存**: T21
**ステータス**: ✅ **完了**（2025-12-25）

### 概要
テンプレート設定の型定義とJSON Schema検証。

### チェックリスト
- [x] JSON Schema作成（src-tauri/schemas/template-mvp-1.0.json）
- [x] TypeScript型定義（src/types/template.ts）
- [x] Rust型定義（src-tauri/src/server/template_types.rs）
- [x] テンプレート検証コマンド（commands/template.rs）
- [x] クランプ関数実装

### クランプ規約
| パラメータ | 範囲 | デフォルト |
|-----------|------|----------|
| offsetX/Y | -40〜+40 | 0 |
| maxLines | 4〜14 | 10 |
| maxItems | 3〜20 | 14 (QueueList推奨:6) |
| cycleSec | 10〜120 | 30 |
| showSec | 3〜15 | 6 |
| leftPct | 0.18〜0.28 | 0.22 |
| centerPct | 0.44〜0.64 | 0.56 |
| rightPct | 0.18〜0.28 | 0.22 |
| gutterPx | 0〜64 | 24 |
| safeArea | 0.0〜0.10 | 0.04 |

### 成果物
- `src-tauri/schemas/template-mvp-1.0.json` - テンプレートJSON Schema
- `src/types/template.ts` - TypeScript型定義・クランプ関数（themeクランプ含む）
- `src-tauri/src/server/template_types.rs` - Rust型定義・クランプ関数（layout_type検証含む）
- `src-tauri/src/commands/template.rs` - テンプレート検証コマンド（ID重複チェック含む）

### PRレビュー対応（2025-12-25）
- [x] TypeScript側themeクランプ実装（panel/shadow/outline）
- [x] Rust側layout_type検証（"threeColumn"に強制）
- [x] コンポーネントID一意性チェック（TypeScript/Rust両方）
- [x] clampOffsetX/clampOffsetYに整数丸め処理追加
- [x] DEFAULT_TEMPLATEの空components配列についてコメント追加

### PR#53 追加レビュー対応（2025-12-25）
- [x] 中: comment_logs保存形式の統一（db.rsを旧形式に統一）
  - message_type=短い文字列、message_data=詳細JSON、published_at=RFC3339
  - youtube.rsの重複関数を削除し、db.rsの共通関数を使用
- [x] 中: published_at保存形式の統一（RFC3339）
- [x] 中: 統合ポーラー開始時に旧ポーラーを停止（二重ポーリング防止）
- [x] 低: _savedStateのlint警告対応（[, setSavedState]に変更）
- [x] 低: MessageType文字列表記をcamelCaseに統一（docs/400_data-models.md）
- [x] 低: InnerTubeドキュメント整合性修正（非対象→バックアップとして実装済み）

### PR#53 追加レビュー対応②（2025-12-26）
- [x] 中: start_polling_innertubeに統合ポーラー停止処理を追加
  - 旧経路（InnerTube単体）起動時にも統合ポーラーを停止するよう修正
  - 相互排他の双方向化完了
- [x] 低: gRPC優先/InnerTubeバックアップのドキュメント統一
  - 001_requirements.md: gRPC Streaming優先、InnerTube=バックアップに更新
  - 100_architecture.md: 技術スタック表とシステム構成図を更新

### PR#53 追加レビュー対応③（2025-12-26）
- [x] 中: start_polling（旧Official/REST）にも統合ポーラー停止処理を追加
  - 3経路すべて（統合/InnerTube/Official）で相互排他が完成
- [x] 低: 001_requirements.md制約・前提セクションをgRPC優先に統一
  - line 75「InnerTube優先」→「gRPC優先」に修正
- [x] 低: 100_architecture.mdシステム構成図をgRPC優先に統一
  - line 49「YouTube InnerTube」→「YouTube API(gRPC)」に修正

---

## T23: 新コンポーネント追加
**優先度**: P2 | **見積**: 10日 | **依存**: T22
**ステータス**: ✅ **完了**（2025-12-25）

### 概要
8個の新規コンポーネントとコンポーネント管理システムを追加。

### チェックリスト

#### Phase 1: 基盤構築（完了）
- [x] ComponentRegistry（共有モジュール）
- [x] BaseComponent（基底クラス）
- [x] components.css（コンポーネント固有スタイル）

#### Phase 2: 静的コンポーネント（完了）
- [x] ClockWidget - 時刻/日付表示
- [x] WeatherWidget - 天気情報（スタブ）
- [x] BrandBlock - ロゴ
- [x] MainAvatarStage - 中央ステージ
- [x] ChannelBadge - チャンネルバッジ

#### Phase 3: 動的コンポーネント（完了）
- [x] KPIBlock - KPI数値（スロットリング対応）
- [x] PromoPanel - 告知（cycle対応）
- [x] QueueList - 待機キュー（maxItems対応）

#### Phase 4: 統合（完了）
- [x] combined-v2.html統合（スクリプト読み込み、初期化、WSハンドラ）
- [x] Rust側WSメッセージ追加（KpiUpdatePayload, QueueUpdatePayload, PromoUpdatePayload）
- [x] ビルド確認

### 成果物
- `src-tauri/overlays/shared/component-registry.js` - コンポーネント管理
- `src-tauri/overlays/components/base-component.js` - 基底クラス
- `src-tauri/overlays/components/clock-widget.js` - 時計
- `src-tauri/overlays/components/weather-widget.js` - 天気（スタブ）
- `src-tauri/overlays/components/brand-block.js` - ロゴ
- `src-tauri/overlays/components/main-avatar-stage.js` - 中央ステージ
- `src-tauri/overlays/components/channel-badge.js` - バッジ
- `src-tauri/overlays/components/kpi-block.js` - KPI
- `src-tauri/overlays/components/promo-panel.js` - 告知
- `src-tauri/overlays/components/queue-list.js` - 待機キュー
- `src-tauri/overlays/styles/components.css` - コンポーネントスタイル
- `src-tauri/overlays/combined-v2.html` - 統合（修正）
- `src-tauri/src/server/types.rs` - WSメッセージ追加

---

## T24: パフォーマンス最適化
**優先度**: P2 | **見積**: 3日 | **依存**: T23
**ステータス**: ✅ **完了**（2025-12-26）

### 概要
100-200msバッチ更新、クランプ規約強制、縮退処理。

### チェックリスト
- [x] バッチ更新実装（requestAnimationFrame）
- [x] クランプ規約の強制実装
- [x] 右下過密時の縮退処理
- [ ] パフォーマンステスト（手動テスト必要）

### 成果物

#### 新規作成（3ファイル）
- `src-tauri/overlays/shared/clamp-constants.js` - クランプ定数（ソース・オブ・トゥルース）
- `src-tauri/overlays/shared/update-batcher.js` - 100-200msバッチ更新システム
- `src-tauri/overlays/shared/density-manager.js` - 右下過密検出・縮退処理

#### 変更（5ファイル）
- `src-tauri/overlays/combined-v2.html` - スクリプト読み込み、WSハンドラ修正
- `src-tauri/overlays/components/base-component.js` - `clampByKey()`メソッド追加
- `src-tauri/overlays/components/kpi-block.js` - 共有定数使用、density対応
- `src-tauri/overlays/components/queue-list.js` - 共有定数使用、density対応
- `src-tauri/overlays/components/promo-panel.js` - 共有定数使用、density対応

### 実装詳細

#### バッチ更新システム（UpdateBatcher）
- WebSocket更新を150msでバッチ処理
- requestAnimationFrameで1フレームにまとめて適用
- 同一コンポーネントタイプへの複数更新は最新のみ保持

#### クランプ規約統一（clamp-constants.js）
- CLAMP_RANGES定数でオーバーレイ側の値制限を一元管理
- TypeScript（template.ts）およびRust（template_types.rs）と同期
- clampByKey()メソッドで共有定数を参照

#### 右下過密検出（DensityManager）
- right.lowerLeft（KPI）、right.lowerRight（Queue）、right.bottom（Promo）を監視
- 2秒間に5回以上の更新で高負荷判定
- density:high/density:normalイベントで各コンポーネントに縮退処理を通知
- 縮退時: updateThrottle 2秒→4秒、maxItems 6→4、showSec 6秒→10秒

---

## T25: 外部API連携
**優先度**: P2 | **見積**: 5日 | **依存**: T23
**ステータス**: ✅ **完了** (PR #57, PR #58)

### 概要
天気API、YouTube Analytics連携。

### チェックリスト
- [x] 天気APIクライアント実装（weather/mod.rs）
- [x] 天気情報キャッシュ（15分TTL）
- [x] YouTube視聴者数/高評価数取得（get_live_stream_stats）
- [x] KPIBlock用データ取得（broadcast_kpi_update）
- [x] WeatherSettings UI
- [x] Open-Meteoへの移行（APIキー不要化）(PR #58)

### 天気API
- Open-Meteo API を採用（APIキー不要、登録不要）
- Geocoding APIで都市名→緯度経度変換
- WMOコードから天気絵文字/日本語説明を生成

---


