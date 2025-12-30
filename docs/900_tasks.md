# タスク分解・進捗管理

> 完了済みタスクは `docs/901_tasks_archived.md` を参照

## T02: YouTube API 実装（BYOK + list）
**優先度**: P0 | **見積**: 7日 | **依存**: T01

### チェックリスト
- [ ] クォータ消費量のロギング（T03で詳細実装予定）

### テスト項目
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
### 追加修正（2025-12-18）
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
### 追加実装（2025-12-18）
### テスト項目
- [ ] OBSで透過背景動作
- [ ] 連続コメントでパフォーマンス維持
- [ ] 異常な長文コメントのハンドリング

---

## T07: セットリスト表示オーバーレイ
**優先度**: P0 | **見積**: 4日 | **依存**: T06
**ステータス**: ✅ **完了**（T04で実装済み）

### チェックリスト
### テスト項目
- [ ] OBSで透過背景動作（手動テスト必要）
- [ ] 曲切替の即時反映（手動テスト必要）
### 成果物
- `src-tauri/overlays/setlist.html` - セットリスト表示オーバーレイ（T04で実装済み）

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
**ステータス**: ✅ **完了**

### チェックリスト
### テスト項目
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

## T12: インストーラー + 更新通知
**優先度**: P1 | **見積**: 4日 | **依存**: All
**ステータス**: ✅ **完了**（2025-12-23）

### チェックリスト
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

### 将来的改善（PR#42, PR#92で実装）
- [x] **リトライ回数制限**: 連続失敗時のバックオフ・リトライ制限
  - 実装済み: 最大3回リトライ、exponential backoff（1秒→30秒）
- [x] **ダウンロードキャンセル機能**: ダウンロード中の中断機能
  - 実装済み: キャンセルボタン追加、AbortControllerパターン
- [x] **「このバージョンをスキップ」機能**: dismiss状態の永続化
  - 実装済み: localStorageにスキップバージョンを保存
### テスト項目
- [ ] 新バージョン検出→通知（手動テスト必要）
- [ ] 更新ダウンロード・インストール（手動テスト必要）

---

## T13: PoC - InnerTube API（HTTP）
**優先度**: P1 | **見積**: 3日 | **依存**: T02
**ステータス**: ✅ **完了**（2025-12-21）

### チェックリスト
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
- [x] **本番UI結線**: 設定画面からApiMode切り替え
  - 実装済み: `CommentControlPanel.tsx:503-584` に取得モード選択UI実装済み
  - InnerTube / 公式API(gRPC) / 公式API(ポーリング) の3モード切り替え可能
- [x] **自動テスト追加**: 絵文字キャッシュ・ポーラー切替テスト
  - 絵文字キャッシュテスト: `parser.rs`に7つのテスト実装済み（サイズ制限、LRU、並行アクセス等）
  - ポーラー切替テスト: 統合テストが必要なため手動テスト項目として管理

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

## 本番リリース前チェックリスト

本番リリース前に必ず対応が必要な項目。PRレビューで指摘された技術的負債を含む。

> **更新履歴**: 2025-12-21 実装済み項目を削除（Default trait, メッセージ詳細情報, キーボード操作対応など）

### セキュリティ（必須）

### コード品質

- [x] **Rust側WidgetVisibilitySettings型の重複削減** (PR#93, PR#94で実装)
  - 実装済み: `types.rs`に共通型`WidgetVisibilitySettings`を定義
  - `overlay.rs`と`http.rs`から重複定義を削除し、共通型をインポート
  - `broadcast_settings_update`での手動マッピングを直接渡しに簡略化

- [ ] **他の設定型も同様に統合を検討** (PR#94レビューで提案)
  - `WeatherSettings` / `WeatherSettingsPayload` の統一
  - `CommentSettings` / `CommentSettingsPayload` の統一
  - `SetlistSettings` / `SetlistSettingsPayload` の統一
  - 優先度: 低（現状でも動作に問題なし、段階的対応で可）

### テスト（推奨）

- [x] **Weather APIテストのヘルパー関数抽出** (PR#84, PR#88で実装)
  - 実装済み: `setup_test_client()`および`mock_geocoding_success()`ヘルパー関数を追加
  - 対象ファイル: `src-tauri/src/weather/mod.rs`

- [x] **ネットワークタイムアウト機能** (PR#84, PR#90で実装)
  - 実装済み: HTTPクライアントに10秒タイムアウトを設定、`YouTubeError::Timeout`バリアント追加
  - 注: mockitoではタイムアウト動作の完全シミュレーションが困難なため、動作テストは除外
  - 対象ファイル: `src-tauri/src/youtube/client.rs`, `src-tauri/src/youtube/errors.rs`

- [x] **HTTPモックのクエリパラメータ検証強化** (PR#84, PR#89で実装)
  - 実装済み: 成功テストで`Matcher::AllOf`を使用してAPIキー、ID、partパラメータを検証
  - 対象ファイル: `src-tauri/src/youtube/client.rs`, `src-tauri/src/weather/mod.rs`

- [x] **HTTPモックテストのエラーメッセージ検証強化** (PR#84, PR#90で実装)
  - 実装済み: `assert_eq!`を使用して正確なエラーメッセージフォーマットを検証
  - 対象ファイル: `src-tauri/src/youtube/client.rs`, `src-tauri/src/commands/template.rs`

- [x] **YouTubeテストヘルパー関数の導入** (PR#85, PR#88で実装)
  - 実装済み: `setup_test_client()`ヘルパー関数を追加
  - 対象ファイル: `src-tauri/src/youtube/client.rs`

- [x] **5xxエラーのログレベル検討** (PR#86, PR#88で実装)
  - 実装済み: 5xxエラーの`log::error!`を`log::warn!`に変更（一時的な障害）
  - 対象ファイル: `src-tauri/src/youtube/client.rs`

- [x] **ApiErrorのリトライロジック確認** (PR#86, PR#88で確認)
  - 確認済み: `poller.rs`のcatch-allハンドラでexponential backoffによるリトライが実装済み
  - 対象ファイル: `src-tauri/src/youtube/poller.rs`

- [x] **WeatherClientのタイムアウトエラー変換** (PR#90で実装済み)
  - 実装済み: `is_timeout()`でタイムアウトを検出し`WeatherError::Timeout`に変換
  - geocode_city()とfetch_weather_for_city()の両方で適用済み
  - 対象ファイル: `src-tauri/src/weather/mod.rs`

- [x] **HTTPタイムアウト定数の統一** (PR#91で実装)
  - 実装済み: `src-tauri/src/config.rs`に共通モジュールを作成
  - `HTTP_TIMEOUT_SECS`定数と`http_timeout()`関数を提供
  - YouTubeClient、WeatherClientの両方から参照
  - 対象ファイル: `src-tauri/src/config.rs`, `src-tauri/src/youtube/client.rs`, `src-tauri/src/weather/mod.rs`

- [x] **タイムアウトエラーのログレベル検討** (PR#91で検討完了)
  - 結論: `log::warn!`を維持
  - 理由: タイムアウトはネットワーク問題の兆候であり、頻発自体が異常状態を示すため警告レベルが適切

### セキュリティ（将来課題）

### 機能改善（中優先度）

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

- [ ] **オーバーレイのドラッグ自由配置機能**
  - プリセットレイアウトではなく、プレビュー画面上でドラッグ&ドロップで配置
  - 技術検討: react-rnd または react-draggable の使用
  - 座標・サイズの設定保存、combined.html での動的配置対応
  - 優先度: 低（現在はプリセットレイアウトで対応）

- [ ] **スーパーチャット通貨換算の改善** (PR#33)
  - 現在: ハードコードされた為替レート（USD:150, EUR:160, GBP:190）
  - 将来: 為替レートAPIからの取得、または設定で変更可能に
  - 優先度: 低

- [ ] **レイアウト比率の合計チェックUI** (PR#51)
  - 現在: レイアウト比率の合計が1.0から離れている場合はwarningログのみ
  - 対応: 将来的にユーザーへのフィードバックとして合計値を表示することを検討
  - 優先度: 低

### パフォーマンス

- [ ] **オーバーレイパフォーマンステスト** (PR#19)
  - MAX_COMMENTSを10→30に増加済み
  - OBSブラウザソースでの長時間使用テスト
  - 低スペックマシンでの動作確認

### テスト

- [ ] **手動テスト項目の実施**
  - 各タスクの「テスト項目」で「手動テスト必要」とされている項目
  - 2時間連続取得テスト（T03）
  - OBSでの透過背景動作（T05, T07）
  - 複数ブラウザソース同時接続（T04）

- [ ] **自動テストの追加** (PR#6)
  - WebSocket接続テスト（手動テスト推奨：サーバー起動が必要）
  - メッセージ送受信テスト（手動テスト推奨：サーバー起動が必要）
  - 複数クライアント同時接続テスト（手動テスト推奨：サーバー起動が必要）
  - 注: WebSocketサーバーのモック化が複雑なため、統合テストとして実環境でのテストを推奨

- [x] **save_comments_to_dbの戻り値構造化** (PR#56, PR#88で実装)
  - 実装済み: `SaveCommentsResult { saved, failed, skipped }`構造体を返すように変更
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [x] **SaveCommentsResultのログ出力強化** (PR#88レビュー, PR#89で実装)
  - 実装済み: `save_comments_to_db()`の呼び出し元5箇所でfailed/skippedのwarnログを出力
  - 対象ファイル: `src-tauri/src/youtube/unified_poller.rs`, `src-tauri/src/commands/youtube.rs`, `src-tauri/src/youtube/grpc/poller.rs`

- [x] **save_comments_to_dbの総予算設定可能化** (PR#56, PR#92で実装)
  - 実装済み: `save_comments_to_db_with_timeout(pool, messages, timeout)`関数を追加
  - テスト用に予算注入可能
  - 予算超過時のskippedカウント検証テストを追加
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [ ] **`broadcast_weather_update(force_refresh=true)` のテスト** (PR#57)
  - キャッシュクリア→新規取得→ブロードキャストの動作を検証
  - テスト対象:
    - `force_refresh=true`でキャッシュがクリアされること
    - 新しいデータがWebSocketでブロードキャストされること
  - 対象ファイル: `src-tauri/src/commands/weather.rs`
  - 優先度: 中（サーバー/状態のモック化が必要）

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

- [x] **TransactionResult::DeadlineExceeded専用バリアント検討** (PR#81レビュー, PR#88で実装)
  - 実装済み: `TransactionResult::DeadlineExceeded`バリアントを追加
  - デッドライン超過時は`Busy`ではなく`DeadlineExceeded`を返すように変更
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

- [x] **DeadlineExceededメトリクス計測** (PR#88レビュー, PR#92で実装)
  - 実装済み: `DEADLINE_EXCEEDED_COUNT` AtomicU64カウンター追加
  - `get_deadline_exceeded_count()` で発生回数取得可能
  - DeadlineExceeded発生時にログにカウントを出力
  - 対象ファイル: `src-tauri/src/youtube/db.rs`

### ドキュメント

### InnerTube API関連（PR#24）

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
#### Phase 2: gRPC基盤の構築（2日）
#### Phase 3: gRPCストリーミング実装（2日）
#### Phase 4: 統合ポーラー作成（1日）
#### Phase 5: フロントエンド統合（1日）
#### Phase 6: テスト・ドキュメント（1日）
- [ ] 各モードの接続テスト（手動テスト）
- [ ] モード切り替えテスト（手動テスト）
- [ ] 再接続テスト（ネットワーク断）（手動テスト）
### PRレビュー対応（2025-12-25）

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

- [x] **接続状態管理の最適化** (PR#40) - **対応不要と判断**
  - 分析結果: `isPolling`と`connectionStatus`は異なる概念を表すため、統合は不適切
    - `isPolling`: ポーリングタスクの実行状態
    - `connectionStatus`: 接続の状態（切断/接続/エラー）
  - 例: ポーリング中でも一時的に切断（リトライ中）の状態がありえる
  - 現在の設計が正しいため変更不要

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

## T24: パフォーマンス最適化
**優先度**: P2 | **見積**: 3日 | **依存**: T23
**ステータス**: ✅ **完了**（2025-12-26）

### 概要
100-200msバッチ更新、クランプ規約強制、縮退処理。

### チェックリスト
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
