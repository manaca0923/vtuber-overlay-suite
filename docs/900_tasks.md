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

- [x] **InnerTubeポーリング間隔計算ロジックの重複削減** (PR#99)
  - 実装済み: `ContinuationType::effective_timeout_ms()`メソッドを追加
  - `youtube.rs`と`unified_poller.rs`の重複ロジックを統一
  - `MAX_POLLING_INTERVAL_MS`定数も`types.rs`に一元化

- [x] **Rust側WidgetVisibilitySettings型の重複削減** (PR#93, PR#94で実装)
  - 実装済み: `types.rs`に共通型`WidgetVisibilitySettings`を定義
  - `overlay.rs`と`http.rs`から重複定義を削除し、共通型をインポート
  - `broadcast_settings_update`での手動マッピングを直接渡しに簡略化

- [x] **他の設定型も同様に統合を検討** (PR#94レビューで提案, PR#95で実装)
  - `WeatherSettings` / `WeatherSettingsPayload` の統一 → 完了
  - `CommentSettings` / `CommentSettingsPayload` の統一 → 完了
  - `SetlistSettings` / `SetlistSettingsPayload` の統一 → 完了
  - `broadcast_settings_update`での手動マッピングを直接渡しに簡略化
  - `http.rs`の`*Api`型を削除し、共通型を使用

- [ ] **http.rs のJSONパース処理の簡略化** (PR#95レビューで提案)
  - 現在: `get_overlay_settings_api`で手動で各フィールドをパース（390-463行目付近）
  - 改善案: `serde_json::from_str::<OverlaySettings>`で直接デシリアライズ
  - 注意: DBスキーマとの整合性、マイグレーション対応を考慮
  - 優先度: 低（現状でも動作に問題なし）

- [ ] **types.rs の分割検討** (PR#95レビューで提案)
  - 設定関連の型が増えてきているため、将来的にファイルが肥大化した際は分割を検討
  - 分割案: `types/settings.rs`, `types/websocket.rs` など
  - 優先度: 低（現時点では問題なし）

- [x] **react-hooks/set-state-in-effect警告の解消** (PR#97レビューで提案, PR#114で実装)
  - 対象ファイル: `UpdateChecker.tsx`, `VideoIdModal.tsx`, `WizardStep2.tsx`
  - 対応:
    - useState遅延初期化パターンでlocalStorage読み込みを最適化
    - 正当なパターン（モーダルリセット、データフェッチング）にはeslint-disableコメントで対応
  - ノウハウ: `issues/029_react-compiler-dependency-inference.md`

- [ ] **eslint-disableコメントの配置スタイル統一** (PR#114レビューで提案)
  - 対象ファイル: `App.tsx`, `VideoIdModal.tsx`, `WizardStep2.tsx`, `CommentControlPanel.tsx`
  - 現状: 行末コメント（`// eslint-disable-line`）と次行コメント（`// eslint-disable-next-line`）が混在
  - 改善案: どちらかに統一（推奨: 次行コメント）
  - 優先度: 低（動作に問題なし、コードスタイルの一貫性のみ）

- [x] **ContinuationType へのDefaultトレイト実装** (PR#99レビューで提案, PR#111で実装)
  - 対象ファイル: `src-tauri/src/youtube/innertube/types.rs`
  - `#[default]`アトリビュートでInvalidationをデフォルト値に設定
  - client.rsの初期化コメントを削除してdefault()を使用

- [x] **WizardSettingsData型の一元化** (PR#110レビューで提案, PR#111で実装)
  - 対象ファイル: `src/types/wizard.ts`に新規作成
  - OverlayPreview.tsxのローカル定義を削除して共通型をインポート

- [ ] **lib.rsのコマンドリスト重複削減** (PR#110レビューで提案)
  - 対象ファイル: `src-tauri/src/lib.rs`
  - 現状: デバッグビルドとリリースビルドで大量のコマンドリストが重複
  - 改善案: マクロやcfg-ifクレートで共通部分を抽出
  - 優先度: 低（Tauriマクロの制約により複雑）

- [ ] **既存コードのRwLockガードawait境界問題の修正** (PR#115レビューで発見)
  - 対象ファイル:
    - `src-tauri/src/commands/youtube.rs` (3箇所: L1286, L1350, L1413)
    - `src-tauri/src/commands/weather.rs` (4箇所: L66, L116, L142, L257)
    - `src-tauri/src/commands/overlay.rs` (1箇所: L209)
    - `src-tauri/src/weather/auto_updater.rs` (2箇所: L140, L201)
  - 問題: `server.read().await`でガードを取得後に`.broadcast(...).await`を呼んでいる
  - 改善案: `Arc::clone` + `tokio::spawn`パターンで分離（queue.rsで実装済み）
  - 参考: `issues/003_tauri-rust-patterns.md#8`
  - 優先度: 中（デッドロックのリスクあり、ただし現状問題は発生していない）

- [x] **CSSキャッシュバスターのバージョン管理** (PR#110レビューで提案, PR#111で実装)
  - 対象ファイル: `src-tauri/overlays/*.html`
  - 全ファイルのキャッシュバスターを`?v=3`に統一
  - combined-v2.htmlにバージョン管理ルールのコメントを追加

- [x] **ポーリング間隔定数の根拠をコメントに追記** (PR#99レビューで提案, PR#111で実装)
  - 対象ファイル: `src-tauri/src/youtube/innertube/types.rs`
  - MAX_POLLING_INTERVAL_MS, MIN_POLLING_INTERVAL_MS に設計根拠を詳細記載

- [x] **SuperchatSettingsのDefault trait実装** (PR#105レビューで提案, PR#111で実装)
  - 対象ファイル: `src-tauri/src/server/types.rs`
  - 実装例:
    ```rust
    impl Default for SuperchatSettings {
        fn default() -> Self {
            Self {
                max_display: 1,
                display_duration_sec: 60,
                queue_enabled: true,
            }
        }
    }
    ```
  - 将来的にデフォルト値が必要になった場合に備えて
  - 優先度: 低（現時点では全フィールドがオプショナル）

- [x] **オーバーレイJSのDEBUG定数統一** (PR#105レビューで提案, PR#113で確認)
  - 対象ファイル: `src-tauri/overlays/shared/*.js`, `src-tauri/overlays/components/*.js`
  - 確認結果: 各ファイルはブラウザのプレーンJSであり、ESモジュールではないためスタンドアロン動作が必要
  - 結論: 現状維持が適切（各ファイルで個別にDEBUG定数を定義）

- [ ] **commentEnabled/setlistEnabledとwidget設定の統合検討** (PR#102レビューで提案)
  - 対象ファイル: `src-tauri/overlays/combined-v2.html`
  - 現状: 2つの異なる仕組みが同じスロット（left.middle, right.upper）を制御
    - `settings.comment.enabled` / `settings.setlist.enabled`: レガシーなコメント/セトリ表示制御
    - `settings.widget.comment` / `settings.widget.setlist`: v2スロットベース可視性制御
  - 問題: 実行順序（comment → widget）によりwidget設定が優先されるが、意図が不明瞭
  - 対応案:
    - A) widget設定に統一し、レガシー設定を非推奨化
    - B) 両方の設定をAND条件で評価（両方trueの場合のみ表示）
    - C) 現状維持（widget設定が優先）をドキュメント化
  - 優先度: 低（現状でも動作に問題なし）

- [x] **queue()呼び出し時のsetBufferInterval()最適化** (PR#100レビューで提案, PR#112で実装)
  - 対象ファイル: `src-tauri/overlays/shared/comment-renderer.js`
  - 実装: `queue()` 側で値が異なる場合のみ `setBufferInterval()` を呼び出し

- [ ] **Invalidation種別のポーリング間隔設計再検討** (PR#100レビューで提案)
  - 対象ファイル: `src-tauri/src/youtube/innertube/types.rs:43`
  - 現在: `Invalidation` は常に1秒固定
  - 検討案: APIの推奨値も考慮して `api_timeout.max(1000).min(5000)` とする
  - サーバー負荷とリアルタイム体験のトレードオフを再検討
  - 優先度: 低（設計判断、次回以降の検討事項）

- [ ] **hasChanges計算のメモ化最適化** (PR#103レビューで提案)
  - 対象ファイル: `src/components/settings/OverlaySettings.tsx`
  - 現在: `JSON.stringify`によるフル比較
  - 問題: 大きなオブジェクトに対するJSON.stringify比較は重くなる可能性
  - 改善案: `lodash.isEqual`や`fast-deep-equal`などの専用ライブラリを使用
  - 優先度: 低（現時点では設定オブジェクトが小さいため問題なし）

- [ ] **combined.htmlへのpostMessageハンドラ追加検討** (PR#103レビューで確認)
  - 対象ファイル: `src-tauri/overlays/combined.html`
  - 現状: v2レイアウト（combined-v2.html）のみPostMessageHandler対応
  - 対応案: v1レイアウトも利用者がいる場合は同様の対応を追加
  - 優先度: 低（v2レイアウトへの移行を推奨）

- [x] **postMessageメッセージ型のドキュメント追記** (PR#103レビューで提案, PR#113で確認)
  - 対象ファイル: `docs/300_overlay-specs.md`
  - 確認結果: `preview:settings:update`メッセージ型は既にドキュメント化済み（lines 250-258）

- [ ] **システムフォント取得のエラーハンドリング強化** (PR#106レビューで提案)
  - 対象ファイル: `src-tauri/src/commands/system.rs`
  - 現在: フォント一覧が空の場合のハンドリングが不明確
  - 改善案: 空リストを許容するか、エラーとして扱うかを明示
  - 優先度: 低（空リストでも動作に問題なし）

- [ ] **フォントプレビューの読み込み待機** (PR#106レビューで提案)
  - 対象ファイル: `src/components/settings/FontSelector.tsx`
  - 現在: Google Fonts選択時、フォント読み込み中でもプレビューが表示される
  - 改善案: `document.fonts.ready`で読み込み完了を待つ or ローディング表示
  - 優先度: 低（UX改善のみ）

- [ ] **Rust側ThemeSettings型のenum化** (PR#106レビューで提案)
  - 対象ファイル: `src-tauri/src/server/types.rs`
  - 現在: `global_theme`と`font_preset`がString型
  - 改善案: enumを使用して型安全性を向上
  - 優先度: 低（後方互換性を考慮すると現状維持でも問題なし）

- [x] **CSS変数のフォールバック値の統一** (PR#106レビューで提案, PR#113で確認)
  - 対象ファイル: `src-tauri/overlays/shared/design-tokens.css`, `src-tauri/overlays/styles/components.css`
  - 確認結果: `--primary-color: #ffffff`が`overlay-common.css`で一元定義済み
  - 各ウィジェットカラーは`var(--widget-*-color, var(--primary-color, #ffffff))`で一貫性あり

- [ ] **キャッシュTTLと自動更新間隔の表示整理** (PR#107レビューで提案)
  - 対象ファイル: `src/components/settings/WeatherSettingsPanel.tsx`
  - 現在: `cache_ttl_remaining`（APIキャッシュ残り時間）を表示しているが、自動更新間隔（15分）は`WeatherAutoUpdater`で管理されており別の値
  - 対応案: 表示テキストにコメントを追加するか、UIで自動更新までの時間も表示する
  - 優先度: 低（動作に問題なし、UX改善のみ）

- [ ] **マルチシティ都市数の上限チェック** (PR#108レビューで提案)
  - 対象ファイル: `src/components/settings/WeatherSettingsPanel.tsx`
  - 現状: カスタム都市追加で無制限に増える可能性
  - 改善案: `MAX_CITIES = 20` 等の上限を設けてエラー表示
  - 優先度: 低（実用上10-20都市で十分）

- [x] **ローテーション間隔の最小値検証** (PR#108レビューで提案, PR#112で実装)
  - 対象ファイル: `src-tauri/src/commands/weather.rs`
  - 実装: `MIN_ROTATION_INTERVAL_SEC`定数を追加し、`.max()`でガード

- [x] **updateMultiの型安全性強化** (PR#108レビューで提案, PR#112で実装)
  - 対象ファイル: `src-tauri/overlays/components/weather-widget.js`
  - 実装: `Array.isArray(data.cities)`チェックを追加

- [ ] **マルチシティ機能のユニットテスト追加** (PR#108レビューで提案, PR#112で追加項目)
  - 対象ファイル: `src-tauri/src/weather/mod.rs`, `src-tauri/src/commands/weather.rs`
  - テストケース:
    - `get_weather_multi` - 正常系（複数都市取得）
    - `get_weather_multi` - 一部都市が失敗した場合
    - `broadcast_weather_multi` - 空の都市リストでエラーが返ること
    - `broadcast_weather_multi` / `set_multi_city_mode` で `rotation_interval_sec = 0` を渡したとき1秒にクランプされること (PR#112)
  - 優先度: 中（モック化が必要）

- [x] **WeatherWidget定数化** (PR#108レビューで提案, PR#112で実装)
  - 対象ファイル: `src-tauri/overlays/components/weather-widget.js`
  - 実装: `DEFAULT_ROTATION_INTERVAL_MS`, `FADE_OUT_DURATION_MS`, `FADE_IN_DURATION_MS`等の定数を追加

- [ ] **set_multi_city_config の非同期設計の改善** (PR#108レビューで提案)
  - 対象ファイル: `src-tauri/src/weather/auto_updater.rs`
  - 問題: 関数は同期的に戻るが、設定の更新は非同期で行われる
  - 改善案A: 関数をasyncにしてawait
  - 改善案B: ドキュメントに「設定は非同期で反映される」旨を記載
  - 優先度: 中（競合状態の可能性があるが、実用上は問題なし）

- [ ] **マルチシティ並列取得のレート制限対応** (PR#108レビューで提案)
  - 対象ファイル: `src-tauri/src/weather/mod.rs`
  - 問題: 10都市同時リクエストでAPI制限に引っかかる可能性
  - 改善案: `futures::stream::buffer_unordered(3)` で同時3リクエストに制限
  - 優先度: 低（Open-Meteo APIは寛容だが、将来的に都市数が増える場合に備えて）

- [x] **CityTuple型エイリアスの追加** (PR#108レビューで提案, PR#112で実装)
  - 対象ファイル: `src/types/weather.ts`, `src/components/settings/WeatherSettingsPanel.tsx`
  - 実装: `CityTuple`型エイリアスを`weather.ts`に定義し、各所でインポートして使用

- [ ] **マルチシティ部分的成功時のUI通知** (PR#108レビューで提案)
  - 対象ファイル: `src/components/settings/WeatherSettingsPanel.tsx`
  - 問題: 一部都市のみ取得成功した場合のフロントエンド通知がない
  - 改善案: 「X/Y 都市の天気を取得しました」等の通知を表示
  - 優先度: 低（ログには記録されているため運用上は問題なし）

- [x] **天気ウィジェットCSSトランジションの確認** (PR#108レビューで提案, PR#113で修正)
  - 対象ファイル: `src-tauri/overlays/styles/components.css`
  - 修正内容: 重複していた`.weather-widget`セレクタを1つに統合し、`transition`プロパティを含めた

- [ ] **InnerTubeクライアントの再作成コスト最適化** (PR#109レビューで提案)
  - 対象ファイル: `src-tauri/src/commands/youtube.rs:1378-1380`
  - 現在: `fetch_viewer_count_innertube`で毎回`InnerTubeClient::new()`を呼び出し
  - 問題: 30秒ごとに新しいクライアントを作成し、コネクションプールの再利用ができない
  - 改善案: 既存のポーリングで使用している`InnerTubeClient`を再利用するか、`AppState`にキャッシュ
  - 優先度: 低（現状でも動作上の問題なし、パフォーマンス最適化として）

- [ ] **InnerTubeモードの制限についてのUIフィードバック** (PR#109レビューで提案)
  - 対象ファイル: `src/components/settings/` (設定画面)
  - 現在: InnerTubeモードでは高評価数が取得できない（`sub: None`）
  - 問題: ユーザーがなぜ高評価数が表示されないのか分からない
  - 改善案: 設定画面やウィジェット表示でInnerTubeモードの制限事項を表示
  - 優先度: 低（UX改善のみ）

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

- [ ] **スーパーチャット通貨換算の改善** (PR#33, PR#104)
  - 現在: ハードコードされた為替レート（USD:150, EUR:160, GBP:190）
  - 将来: 為替レートAPIからの取得、または設定で変更可能に
  - 優先度: 低

- [ ] **スパチャ表示タイマーのキャンセル処理** (PR#104)
  - 現在: `schedule_superchat_removal`のtokio::spawnタスクはキャンセル不可
  - 問題: アプリ終了時やポーリング停止時にタイマーが残り続ける可能性
  - 対応: JoinHandleを保持してキャンセル可能にする
  - 対象ファイル: `src-tauri/src/superchat/mod.rs`
  - 優先度: 低（高額スパチャ複数時以外は問題なし）

- [ ] **スパチャキュー上限の追加** (PR#104)
  - 現在: `superchat-card.js`の`queue`配列に上限がない
  - 問題: 短時間に大量のスパチャが来た場合にメモリを圧迫する可能性
  - 対応: 100件程度の上限を設けて古いものからドロップする
  - 対象ファイル: `src-tauri/overlays/components/superchat-card.js`
  - 優先度: 低（極端なケースのみ）

- [ ] **スパチャコンポーネントのデバッグログ整理** (PR#104)
  - 現在: 開発用の`console.log`がそのまま残っている
  - 対応: 本番リリース前に`LOG_LEVEL`設定導入または削除
  - 対象ファイル: `src-tauri/overlays/components/superchat-card.js`
  - 優先度: 低（デバッグログ共通対応時に検討）

- [ ] **displayDurationSecとTier別表示時間の整合性** (PR#104)
  - 現在: フロントエンドで`displayDurationSec`を設定できるが、バックエンドはTierベースの固定値
  - 説明: 設定UIの`displayDurationSec`はプレビュー用にのみ使用される
  - 対応案A: 設定UIに「※実際の表示時間はTierに応じて自動設定されます」の注記を追加
  - 対応案B: 将来的に設定で上書き可能にする（WebSocket経由で設定を受け取る）
  - 対象ファイル: `src/components/settings/SuperchatSettingsPanel.tsx`
  - 優先度: 低（UIのみの改善）

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

- [ ] **comment-renderer.jsのユニットテスト追加** (PR#100レビューで提案)
  - 対象ファイル: `src-tauri/overlays/shared/comment-renderer.js`
  - テスト対象: `setBufferInterval()` の入力検証・タイマー管理
  - 優先度: 低（将来的なJavaScriptテスト基盤構築時に対応）

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

## ウィジェット実装状況（2025-12-31 調査）

> ウィジェット表示設定（設定画面のトグル）に表示される9つのウィジェットの実装状況。
> 各ウィジェットは3層（バックエンド/WebSocket送信/オーバーレイUI）で構成される。

### 実装状況サマリー

| ウィジェット | スロット | バックエンド | WS送信 | UI表示 | 総合判定 |
|-------------|---------|-------------|--------|--------|----------|
| 時計 | left.top | - | - | ✅ | ✅ **完全動作**（ローカル実行設計） |
| 天気 | left.topBelow | ✅ | ✅ | ✅ | ✅ **完全動作** |
| コメント | left.middle | ✅ | ✅ | ✅ | ✅ **完全動作** |
| スパチャ | left.lower | ✅ | ✅ | ✅ | ✅ **完全動作** |
| **ロゴ** | left.bottom | ❌ | ❌ | ✅ | ⚠️ UI のみ（設定UIなし） |
| セトリ | right.upper | ✅ | ✅ | ✅ | ✅ **完全動作** |
| **KPI** | right.lowerLeft | ✅ | ✅ | ✅ | ✅ **完全動作**（gRPC/公式APIモード時） |
| **短冊** | right.lowerRight | ✅ | ✅ | ✅ | ✅ **完全動作** |
| **告知** | right.bottom | ❌ | ✅型のみ | ✅ | ⚠️ スタブ表示 |

### 詳細説明

#### ✅ 完全動作（6個）

1. **時計** (`ClockWidget`)
   - 設計上バックエンド不要（ブラウザのローカル時刻を使用）
   - 毎秒更新、日本語フォーマット対応
   - ファイル: `src-tauri/overlays/components/clock-widget.js`

2. **天気** (`WeatherWidget`)
   - Open-Meteo API連携（APIキー不要）
   - 都市名 → 緯度経度変換 → 天気取得
   - 15分キャッシュ、WMOコード → 絵文字変換
   - WebSocket: `weather:update` メッセージ
   - ファイル: `src-tauri/src/weather/`, `src-tauri/src/commands/weather.rs`

3. **コメント**
   - gRPC/InnerTube/公式API 3モード対応
   - 即時表示（instant）/バッファ表示 両対応
   - DB保存、履歴キャッシュ（50件）
   - WebSocket: `comment:add` / `comment:remove` メッセージ
   - ファイル: `src-tauri/src/youtube/`, `src-tauri/overlays/shared/comment-renderer.js`

4. **セトリ**
   - DB完備（setlists, setlist_songs テーブル）
   - 曲の再生状態管理、リアルタイム更新
   - WebSocket: `setlist:update` メッセージ
   - ファイル: `src-tauri/src/commands/setlist.rs`

5. **スパチャ** (`SuperchatCard`)
   - スパチャ専用表示ウィジェット（コメント欄とは別に目立たせて表示）
   - 金額帯Tier判定（1-7）、通貨換算（JPY, USD, EUR等対応）
   - Tierに応じた表示時間（10秒〜5分）
   - キュー管理（同時表示1件、待機キュー）
   - WebSocket: `superchat:add` / `superchat:remove` メッセージ
   - ファイル: `src-tauri/src/superchat/mod.rs`, `src-tauri/overlays/components/superchat-card.js`

6. **KPI** (`KPIBlock`)
   - UIコンポーネント完成（数値フォーマット、パルスアニメーション）
   - YouTube Data APIから同時接続者数・高評価数を30秒ごとに取得
   - gRPC/公式APIモードでのみ動作（InnerTubeモードではスキップ）
   - WebSocket: `kpi:update` メッセージ
   - ファイル: `src-tauri/overlays/components/kpi-block.js`

#### ⚠️ 部分実装（3個）

7. **ロゴ** (`BrandBlock`)
   - UIコンポーネントは完成（画像URL/テキスト表示対応）
   - **欠けているもの**: 設定画面からロゴURL/テキストを入力するUI
   - ファイル: `src-tauri/overlays/components/brand-block.js`

8. **短冊** (`QueueList`)
   - UIコンポーネント完成（リスト表示、最大6件、空時非表示）
   - キュー管理バックエンド完成（DB保存、CRUD操作）
   - WebSocket: `queue:update` メッセージでリアルタイム反映
   - 設定UI完成（アイテム追加/削除/クリア、タイトル設定）
   - ファイル: `src-tauri/overlays/components/queue-list.js`, `src-tauri/src/commands/queue.rs`

9. **告知** (`PromoPanel`)
   - UIコンポーネントは完成（サイクル表示、フェードアニメーション）
   - WebSocket型定義あり: `promo:update` (`PromoUpdatePayload`)
   - **欠けているもの**: 告知コンテンツ管理のバックエンド、設定UI
   - 現状: `afterMount`で固定ダミー文言（「チャンネル登録よろしく」等）を表示
   - ファイル: `src-tauri/overlays/components/promo-panel.js`

### 関連ファイル

- オーバーレイHTML: `src-tauri/overlays/combined-v2.html`
- コンポーネント: `src-tauri/overlays/components/*.js`
- WebSocket型定義: `src-tauri/src/server/types.rs`
- 設定画面: `src/components/settings/WidgetSettingsPanel.tsx`

---

## T25: スパチャウィジェット実装
**優先度**: P1 | **見積**: 5日 | **依存**: T16
**ステータス**: ✅ **完了**（2025-12-31）

### 概要
スーパーチャット専用表示ウィジェット（left.lower スロット）を実装する。
コメント欄で流れるスパチャとは別に、目立つ専用領域で一定時間表示する。

### 背景
- 現在スパチャはコメント欄で他のコメントと一緒に流れている
- 配信者がスパチャを見逃しやすい
- 専用ウィジェットで目立たせて表示し、一定時間固定表示 + キュー管理が必要

### チェックリスト

#### Phase 1: バックエンド実装
- [x] スパチャデータの抽出ロジック（コメント取得時にスパチャを別途保持）
- [x] スパチャキュー管理（表示待ちスパチャのキュー）
- [x] WebSocket送信: `superchat:add` / `superchat:remove` メッセージ
- [x] 表示時間管理（金額に応じた表示時間）

#### Phase 2: オーバーレイUI実装
- [x] `superchat-card.js` コンポーネント作成
- [x] combined-v2.html にWebSocketハンドラ追加
- [x] スパチャカラー（金額帯に応じた背景色）対応
- [x] アニメーション（スライドイン/アウト）

#### Phase 3: 設定UI（将来実装）
- [ ] スパチャウィジェットの設定項目追加
  - 最小表示金額フィルター
  - 表示時間設定
  - 読み上げ連携（将来）

### 技術仕様

#### WebSocketメッセージ
```typescript
// スパチャ追加
{
  type: 'superchat:add',
  payload: {
    id: string,
    authorName: string,
    authorImageUrl: string,
    amount: string,        // "¥1,000" 等
    amountMicros: number,  // マイクロ単位の金額
    currency: string,      // "JPY" 等
    message: string,
    tier: number,          // 1-7 (金額帯)
    displayDurationMs: number
  }
}

// スパチャ削除（表示完了）
{
  type: 'superchat:remove',
  payload: { id: string }
}
```

#### 金額帯とカラー（YouTube公式準拠）
| Tier | 金額 (JPY) | 背景色 |
|------|-----------|--------|
| 1 | ¥100-199 | #1565C0 (Blue) |
| 2 | ¥200-499 | #00B8D4 (Cyan) |
| 3 | ¥500-999 | #00BFA5 (Teal) |
| 4 | ¥1,000-1,999 | #FFB300 (Yellow) |
| 5 | ¥2,000-4,999 | #F57C00 (Orange) |
| 6 | ¥5,000-9,999 | #E91E63 (Pink) |
| 7 | ¥10,000+ | #E62117 (Red) |

### 成果物
- `src-tauri/overlays/components/superchat-card.js` - UIコンポーネント（キュー管理、アニメーション）
- `src-tauri/src/server/types.rs` - `SuperchatPayload`, `SuperchatRemovePayload` 型追加
- `src-tauri/src/superchat/mod.rs` - スパチャモジュール（Tier判定、表示時間計算、通貨換算）
- `src-tauri/overlays/combined-v2.html` - WebSocketハンドラ追加
- `src-tauri/src/youtube/unified_poller.rs` - スパチャ検出・ブロードキャスト
- `src-tauri/src/youtube/grpc/poller.rs` - スパチャ検出・ブロードキャスト

---

## T26: KPIデータ連携
**優先度**: P2 | **見積**: 2日 | **依存**: T16
**ステータス**: ✅ **完了**（2025-12-31）

### 概要
KPIウィジェット（right.lowerLeft）に実データを供給する。
YouTube APIから同時接続者数・高評価数を取得してリアルタイム表示。

### 背景
- KPIBlockコンポーネントは完成済み（UI、WebSocket受信、density対応）
- WebSocket型定義 `kpi:update` も存在
- **実装完了**: データ取得・ブロードキャストのバックエンドロジック

### チェックリスト
- [x] YouTube Data API v3 から同時接続者数取得（`videos.list` の `liveStreamingDetails.concurrentViewers`）
- [x] 定期的な取得（30秒間隔）
- [x] `fetch_and_broadcast_viewer_count()` コマンド実装
- [x] InnerTubeモードでの視聴者数表示対応（`fetch_viewer_count_innertube` コマンド）
- [ ] 設定UI: 表示するKPI項目の選択（視聴者数/高評価数/チャット速度等）（将来対応）

### 技術仕様
- 取得間隔: 30秒（クォータ節約とリアルタイム性のバランス）
- 主数値: 同時接続者数（`concurrentViewers`）
- 副数値: 高評価数（`likeCount`）
- **KPI取得は常に同梱APIキーを使用**（コメント取得モードに関係なく）
  - 理由: InnerTube APIの`viewCount`は総視聴回数であり、同時接続者数ではないため不正確
  - クォータ消費: 約3 units/30秒（10時間配信で約3,600 units、許容範囲）

### 設計判断（PR#110）
| 機能 | 取得方法 | 理由 |
|------|----------|------|
| コメント取得 | モードに応じて切替 | InnerTubeモードでクォータ節約 |
| KPI取得 | 常に同梱APIキー | 正確な同時接続者数が必要 |

### 成果物
- `src-tauri/src/commands/youtube.rs` - `fetch_and_broadcast_viewer_count` コマンド（KPI取得）
- `src/components/CommentControlPanel.tsx` - ポーリング中の定期取得ロジック（常に同梱キー使用）
- `src-tauri/overlays/components/kpi-block.js` - モックデータ削除、実データ待機に変更
- `src-tauri/src/youtube/innertube/client.rs` - `get_video_details()` メソッド（デバッグ用）
- `src-tauri/src/youtube/innertube/types.rs` - `VideoDetails`, `InnerTubePlayerResponse` 型追加
- `issues/028_pr109-log-level-trace.md` - 定期実行ログレベル指針のノウハウ

---

## T27: 短冊（キュー）管理機能
**優先度**: P3 | **見積**: 3日 | **依存**: なし
**ステータス**: ✅ **完了**（2026-01-01）

### 概要
短冊ウィジェット（right.lowerRight）のキュー管理機能を実装。
リクエスト曲や待機者リストなどを管理・表示する。

### 背景
- QueueListコンポーネントは完成済み（UI、WebSocket受信）
- WebSocket型定義 `queue:update` も存在
- ~~**欠けているもの**: キュー管理のバックエンド、設定UI~~ → 実装完了

### チェックリスト
- [x] キューデータのDB保存（settingsテーブル）
- [x] キュー操作コマンド: `add_queue_item`, `remove_queue_item`, `clear_queue`
- [x] `broadcast_queue_update()` 関数実装
- [x] 設定UI: キュータイトル、アイテム管理
- [x] OverlaySettingsへのタブ統合

### ユースケース
- リクエスト曲の待ち行列
- 参加者リスト
- 抽選待ちリスト

### 成果物
- `src-tauri/src/commands/queue.rs` - キュー管理コマンド
  - `get_queue_state`, `save_queue_state` - 状態取得・保存
  - `add_queue_item`, `remove_queue_item`, `clear_queue` - アイテム操作
  - `set_queue_title` - タイトル設定
  - `broadcast_queue_update`, `save_and_broadcast_queue` - WebSocketブロードキャスト
- `src/components/settings/QueueSettingsPanel.tsx` - 設定UI（アイテムCRUD）

---

## T28: 告知コンテンツ管理
**優先度**: P3 | **見積**: 2日 | **依存**: なし
**ステータス**: 未着手

### 概要
告知ウィジェット（right.bottom）のコンテンツ管理機能を実装。
ユーザーが告知文を設定し、サイクル表示させる。

### 背景
- PromoPanelコンポーネントは完成済み（UI、WebSocket受信、サイクル表示）
- WebSocket型定義 `promo:update` も存在
- **欠けているもの**: 告知コンテンツの設定UI、保存機能
- 現状: 固定ダミー文言（「チャンネル登録よろしく」等）を表示

### チェックリスト
- [ ] 告知コンテンツのDB保存
- [ ] 告知編集コマンド: `set_promo_items`, `get_promo_items`
- [ ] `broadcast_promo_update()` 関数実装
- [ ] 設定UI: 告知文の追加・編集・削除、サイクル間隔設定

### 成果物
- `src-tauri/src/commands/promo.rs` - 告知管理コマンド
- `src/components/settings/PromoSettingsPanel.tsx` - 設定UI

---

## T29: ロゴ設定UI
**優先度**: P3 | **見積**: 1日 | **依存**: なし
**ステータス**: 未着手

### 概要
ロゴウィジェット（left.bottom）の設定UIを実装。
ユーザーがロゴ画像URL/テキストを設定できるようにする。

### 背景
- BrandBlockコンポーネントは完成済み（UI、画像/テキスト表示）
- **欠けているもの**: 設定画面からロゴを設定するUI
- 現状: データがないため何も表示されない

### チェックリスト
- [ ] ロゴ設定の保存（DB or 設定ファイル）
- [ ] 設定UI: ロゴURL入力、代替テキスト入力、プレビュー表示
- [ ] 設定変更時のWebSocketブロードキャスト

### 成果物
- `src/components/settings/BrandSettingsPanel.tsx` - 設定UI
- 設定保存ロジック追加

---
