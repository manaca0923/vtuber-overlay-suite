# 完了済みタスクアーカイブ

> このファイルは `docs/900_tasks.md` から移動した完了済みタスクの履歴です。

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
