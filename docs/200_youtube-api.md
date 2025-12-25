# YouTube API 仕様書

## 概要

本アプリは**gRPCストリーミング（公式API）**を優先使用し、**InnerTube API（非公式）**はバックアップとして利用可能。

| API | 用途 | 認証 | クォータ制限 | 推奨 |
|-----|------|------|-------------|------|
| gRPC Streaming (公式) | **メイン機能** | APIキー | あり（低消費） | ✅ |
| InnerTube API (非公式) | バックアップ・認証不要時 | 不要 | なし | - |
| YouTube Data API v3 ポーリング | 互換モード | APIキー(BYOK) | あり（高消費） | - |

> **推奨理由**: gRPCは公式APIであり安定性が高く、ストリーミング方式のためクォータ消費も低い。InnerTubeは非公式APIのため仕様変更リスクがあり、バックアップとして位置付ける。

---

## InnerTube API（バックアップ）

### 特徴
- **認証不要**: APIキーなしで動作
- **クォータ制限なし**: 長時間配信でも枯渇しない
- **カスタム絵文字対応**: メンバースタンプの画像URLを取得可能
- **非公式**: 仕様変更リスクあり

### 実装
- `src-tauri/src/youtube/innertube/` モジュール
- コマンド: `start_polling_innertube`, `stop_polling_innertube`

---

### エンドポイント

```
POST https://www.youtube.com/youtubei/v1/live_chat/get_live_chat
```

### リクエスト構造

```json
{
  "context": {
    "client": {
      "clientName": "WEB",
      "clientVersion": "2.20250101.00.00"
    }
  },
  "continuation": "0ofMyAO..."
}
```

### CLIENT_VERSION管理

> ⚠️ **重要リスク**: `clientVersion` はハードコーディングされており、YouTubeの仕様変更で動作停止の可能性があります。

| 項目 | 値 |
|------|-----|
| 現在の値 | `src-tauri/src/youtube/innertube/client.rs` に定義 |
| 形式 | `2.YYYYMMDD.XX.XX`（日付ベース） |
| 更新頻度 | YouTube側で不定期に変更 |

**対策**:
- 初回アクセス時にHTML/JSから動的取得を試行
- 失敗時はハードコーディング値にフォールバック
- 将来的にリモート設定サーバーからの取得も検討

### レスポンス構造

```json
{
  "continuationContents": {
    "liveChatContinuation": {
      "continuations": [
        {
          "invalidationContinuationData": {
            "continuation": "...",
            "timeoutMs": 5000
          }
        }
      ],
      "actions": [
        {
          "addChatItemAction": {
            "item": {
              "liveChatTextMessageRenderer": {
                "id": "...",
                "message": { "runs": [...] },
                "authorName": { "simpleText": "..." },
                "authorPhoto": { "thumbnails": [...] },
                "authorBadges": [...]
              }
            }
          }
        }
      ]
    }
  }
}
```

### メッセージタイプ対応

| レンダラー | 種別 | 対応状況 |
|-----------|------|----------|
| `liveChatTextMessageRenderer` | 通常コメント | ✅ |
| `liveChatPaidMessageRenderer` | スーパーチャット | ✅ |
| `liveChatPaidStickerRenderer` | スーパーステッカー | ✅ |
| `liveChatMembershipItemRenderer` | メンバーシップ登録 | ✅ |
| `liveChatSponsorshipsGiftPurchaseAnnouncementRenderer` | メンバーシップギフト | ✅ |
| `liveChatViewerEngagementMessageRenderer` | システムメッセージ | ⏭️ スキップ |

### カスタム絵文字処理

メッセージ内のカスタム絵文字は `message.runs` 配列で取得:

```json
{
  "runs": [
    { "text": "こんにちは " },
    {
      "emoji": {
        "emojiId": "UC.../...",
        "shortcuts": [":member_emoji:"],
        "image": {
          "thumbnails": [
            { "url": "https://yt3.ggpht.com/...", "width": 24, "height": 24 }
          ]
        }
      }
    }
  ]
}
```

**セキュリティ**: 絵文字画像URLはYouTubeドメインのみ許可（`isValidEmojiUrl`関数で検証）

### ポーリング間隔

| ソース | 間隔 |
|--------|------|
| `timeoutMs`（レスポンス） | 通常 5000ms |
| 最小間隔（強制） | 3000ms |
| エラー時 | 指数バックオフ（5s→10s→20s→...→60s） |

```rust
// 実装例
let interval = response.timeout_ms.unwrap_or(5000).max(3000);
tokio::time::sleep(Duration::from_millis(interval as u64)).await;
```

### エラーハンドリング

| 状況 | HTTP Status | 対処 |
|------|-------------|------|
| 正常 | 200 | 次のcontinuationで続行 |
| レート制限 | 429 | 指数バックオフ |
| 配信終了 | 200 + 空actions | 終了状態として通知 |
| 無効なcontinuation | 400 | 配信ページから再取得 |
| サーバーエラー | 5xx | リトライ（最大5回） |
| 仕様変更 | パース失敗 | エラー通知、手動対応が必要 |

### 既知の制約

| 制約 | 詳細 | 影響 |
|------|------|------|
| 非公式API | YouTube側の仕様変更で動作停止の可能性 | 定期的な動作確認が必要 |
| CLIENT_VERSION | ハードコーディング値が古くなる可能性 | 動的取得で対策済み |
| 地域制限 | 一部地域でブロックされる可能性 | VPN使用時に問題報告あり |
| 限定公開配信 | 視聴権限がないと取得不可 | ユーザーがブラウザでログイン必要 |
| メンバー限定配信 | 認証なしでは取得不可 | 将来的にOAuth対応を検討 |

### 初回continuation取得

配信開始時、continuation tokenは配信ページのHTMLから抽出:

```rust
// 配信ページからcontinuationを抽出
pub async fn get_initial_continuation(video_id: &str) -> Result<String> {
    let url = format!("https://www.youtube.com/live_chat?v={}", video_id);
    let html = client.get(&url).send().await?.text().await?;

    // ytInitialDataからcontinuationを抽出
    let re = Regex::new(r#""continuation":"([^"]+)""#)?;
    // ...
}
```

### デバッグ・トラブルシューティング

| 問題 | 確認方法 | 解決策 |
|------|----------|--------|
| コメント取得できない | ブラウザでチャット表示確認 | 配信URLが正しいか確認 |
| 途中で止まる | ログでHTTPステータス確認 | CLIENT_VERSION更新を試行 |
| 絵文字が表示されない | コンソールでURL確認 | ドメイン許可リスト確認 |
| スパチャ色がおかしい | 金額パース確認 | 通貨形式の対応追加 |

---

## YouTube Data API v3（ポーリングモード）

> **Note**: このAPIはgRPCストリーミングが利用できない環境向けの互換モードです。クォータ消費が高いため、可能な限りgRPCモードを使用してください。

### BYOK（Bring Your Own Key）

ユーザーが自身のGoogle Cloud ProjectでAPIキーを発行して使用。

**ユーザー向けセットアップ手順**:
1. [Google Cloud Console](https://console.cloud.google.com/) にアクセス
2. プロジェクトを作成
3. YouTube Data API v3 を有効化
4. APIキーを作成（制限推奨: YouTube Data API v3のみ）
5. ツールにAPIキーを入力

---

## API フロー

### 1. 配信ID → Live Chat ID 取得

```
GET https://www.googleapis.com/youtube/v3/videos
  ?part=liveStreamingDetails
  &id={VIDEO_ID}
  &key={API_KEY}
```

**レスポンス例**:
```json
{
  "items": [{
    "liveStreamingDetails": {
      "activeLiveChatId": "Cg0KC2FiY2RlZmdoaWpr"
    }
  }]
}
```

**クォータコスト**: 1 unit

---

### 2. コメント取得（ポーリング）

```
GET https://www.googleapis.com/youtube/v3/liveChat/messages
  ?liveChatId={LIVE_CHAT_ID}
  &part=snippet,authorDetails
  &key={API_KEY}
  &pageToken={NEXT_PAGE_TOKEN}  // 2回目以降
```

**レスポンス例**:
```json
{
  "pollingIntervalMillis": 5000,
  "nextPageToken": "QURTSl...",
  "items": [
    {
      "id": "LCC.CjoKGkNO...",
      "snippet": {
        "type": "textMessageEvent",
        "liveChatId": "Cg0KC...",
        "authorChannelId": "UC...",
        "publishedAt": "2025-01-01T12:00:00.000Z",
        "hasDisplayContent": true,
        "displayMessage": "こんにちは！",
        "textMessageDetails": {
          "messageText": "こんにちは！"
        }
      },
      "authorDetails": {
        "channelId": "UC...",
        "channelUrl": "https://www.youtube.com/channel/UC...",
        "displayName": "視聴者名",
        "profileImageUrl": "https://yt3.ggpht.com/...",
        "isVerified": false,
        "isChatOwner": false,
        "isChatSponsor": false,
        "isChatModerator": false
      }
    }
  ]
}
```

**クォータコスト**: 推定 5 units（要実測）

---

## クォータ管理

### デフォルト制限

| 項目 | 値 |
|------|-----|
| 日次クォータ | 10,000 units |
| リセット時刻 | 太平洋時間 0:00（日本時間 17:00） |

### 消費見積もり

| 操作 | コスト | 頻度例 | 1時間消費 |
|------|--------|--------|-----------|
| videos.list | 1 unit | 1回/配信 | 1 unit |
| liveChat/messages.list | ~5 units | 720回/時(5秒間隔) | ~3,600 units |

**結論**: 5秒間隔で約2.78時間で枯渇 → 公式API使用時はBYOK必須（InnerTube APIではクォータ制限なし）

### 推奨ポーリング戦略

```rust
// pollingIntervalMillis を厳守
let interval = response.polling_interval_millis.max(5000); // 最低5秒
tokio::time::sleep(Duration::from_millis(interval)).await;
```

---

## エラーハンドリング

### エラーコード対応表

| エラー | 原因 | 対処 |
|--------|------|------|
| 403 quotaExceeded | クォータ枯渇 | ユーザーに通知、翌日まで待機 |
| 403 rateLimitExceeded | レート制限 | 指数バックオフ（1s→2s→4s→8s→16s） |
| 403 liveChatDisabled | チャット無効 | ユーザーに通知 |
| 404 liveChatNotFound | 配信終了/無効 | 停止、ユーザーに通知 |
| 401 invalidKey | APIキー無効 | 再入力を促す |
| 400 invalidPageToken | トークン無効 | トークンリセット、最初から |

### 指数バックオフ実装

```rust
pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    current_attempt: u32,
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            current_attempt: 0,
        }
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay = self.base_delay * 2u32.pow(self.current_attempt);
        self.current_attempt += 1;
        delay.min(self.max_delay)
    }

    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }
}
```

---

## コメント型定義

### Rust

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_channel_id: String,
    pub author_image_url: String,
    pub published_at: DateTime<Utc>,
    pub is_owner: bool,
    pub is_moderator: bool,
    pub is_member: bool,  // isChatSponsor
    pub is_verified: bool,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    SuperChat { amount: String, currency: String },
    SuperSticker { sticker_id: String },
    Membership { level: String },
    MembershipGift { count: u32 },
}
```

### TypeScript（Frontend/Overlay）

```typescript
interface ChatMessage {
  id: string;
  message: string;
  authorName: string;
  authorChannelId: string;
  authorImageUrl: string;
  publishedAt: string;
  isOwner: boolean;
  isModerator: boolean;
  isMember: boolean;
  isVerified: boolean;
  messageType: MessageType;
}

type MessageType =
  | { type: 'text' }
  | { type: 'superChat'; amount: string; currency: string }
  | { type: 'superSticker'; stickerId: string }
  | { type: 'membership'; level: string }
  | { type: 'membershipGift'; count: number };
```

---

## コマンド解析（!req等）

### 仕様

| 項目 | 仕様 |
|------|------|
| プレフィックス | `!`（設定可能） |
| レート制限 | 同一ユーザー30秒に1回 |
| 権限制御 | 全員 / メンバー限定 / モデレーター限定 |

### 実装例

```rust
pub struct CommandParser {
    prefix: String,
    rate_limiter: HashMap<String, Instant>,
    cooldown: Duration,
}

impl CommandParser {
    pub fn parse(&mut self, msg: &ChatMessage) -> Option<Command> {
        if !msg.message.starts_with(&self.prefix) {
            return None;
        }

        // レート制限チェック
        if let Some(last) = self.rate_limiter.get(&msg.author_channel_id) {
            if last.elapsed() < self.cooldown {
                return None;
            }
        }

        let parts: Vec<&str> = msg.message[self.prefix.len()..].split_whitespace().collect();
        let cmd = parts.first()?;
        let args = parts[1..].to_vec();

        self.rate_limiter.insert(msg.author_channel_id.clone(), Instant::now());

        Some(Command {
            name: cmd.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            author: msg.clone(),
        })
    }
}
```

---

## 統合ポーラー（Unified Poller）

### 概要

3つのAPIモードを統一的に管理する統合ポーラーを実装。ユーザーはUIから自由にモードを切り替え可能。

### APIモード

| モード | 説明 | 認証 | クォータ | 推奨 |
|--------|------|------|----------|------|
| `grpc` | 公式API gRPCストリーミング | APIキー | あり（低消費） | ✅ |
| `innertube` | 非公式API（バックアップ） | 不要 | なし | - |
| `official` | 公式API ポーリング（互換） | APIキー | あり（高消費） | - |

**モード選択ガイド**:
- **gRPC（推奨）**: 公式APIで安定性が高く、ストリーミング方式によりクォータ消費が低い
- **InnerTube**: APIキーなしで動作するため認証設定不要だが、非公式のため仕様変更リスクあり
- **Official**: gRPCが利用できない環境向けの互換モード。クォータ消費が高い

### Tauriコマンド

```typescript
// 統合ポーリング開始
await invoke('start_unified_polling', {
  videoId: 'VIDEO_ID',
  mode: 'innertube' | 'grpc' | 'official',
  useBundledKey: true,  // 同梱キーを使用
  userApiKey: null,     // またはBYOKキー
});

// 統合ポーリング停止
await invoke('stop_unified_polling');

// 実行中確認
const running = await invoke<boolean>('is_unified_polling_running');

// 現在のモード取得
const mode = await invoke<ApiMode | null>('get_unified_polling_mode');
```

### ステータスイベント

各モードは専用のステータスイベントを発行し、フロントエンドで接続状態を監視できる。

```typescript
// InnerTubeモード
listen<InnerTubeStatusEvent>('innertube-status', (event) => {
  const { connected, error, stopped } = event.payload;
});

// gRPCモード
listen<GrpcStatusEvent>('grpc-status', (event) => {
  const { connected, liveChatId, error } = event.payload;
});

// Officialモード（ポーリング）
listen<OfficialStatusEvent>('official-status', (event) => {
  const { connected, error, stopped, quotaExceeded, streamEnded, retrying } = event.payload;
});
```

### フロントエンドでのエラーハンドリング

ステータスイベントを使用して、モードに応じた適切なエラー表示を行う。

| イベント | フィールド | 対処方法 |
|----------|------------|----------|
| `connected: true` | - | 正常接続を表示 |
| `error` + `retrying: true` | error | リトライ中を表示（警告レベル） |
| `error` + `retrying: false` | error | エラー表示、手動対処を促す |
| `quotaExceeded` | - | クォータ超過を通知、翌日まで待機 |
| `streamEnded` | - | 配信終了を通知 |
| `stopped` | reason | 停止理由を表示 |

```typescript
// 実装例（CommentControlPanel.tsx）
if (connected) {
  setConnectionStatus('connected');
  setError(null);
} else if (quotaExceeded) {
  setConnectionStatus('error');
  setError('クォータ超過 - 翌日まで待機してください');
} else if (streamEnded) {
  setConnectionStatus('disconnected');
} else if (statusError) {
  setConnectionStatus(retrying ? 'connected' : 'error');
  if (!retrying) setError(statusError);
}
```

### 実装

- `src-tauri/src/youtube/unified_poller.rs` - 統合ポーラー
- `src-tauri/src/commands/youtube.rs` - Tauriコマンド
- `src/types/api.ts` - TypeScript型定義

---

## gRPCストリーミング（公式API）

### 概要

YouTube Data API v3の`liveChatMessages.streamList`をgRPCで実装。ポーリングよりも低遅延でクォータ消費も少ない。

### 技術スタック

| 項目 | 値 |
|------|-----|
| ライブラリ | tonic 0.12 + prost 0.13 |
| 接続先 | `youtube.googleapis.com:443` |
| プロトコル | gRPC server-streaming |
| 認証 | `x-goog-api-key` メタデータ |

### proto定義

```protobuf
// src-tauri/proto/youtube_live_chat.proto
service LiveChatMessageService {
  rpc StreamList(LiveChatMessageListRequest) returns (stream LiveChatMessageListResponse);
}
```

### 実装

- `src-tauri/src/youtube/grpc/mod.rs` - モジュール定義
- `src-tauri/src/youtube/grpc/client.rs` - gRPCクライアント
- `src-tauri/src/youtube/grpc/poller.rs` - ストリーミングポーラー

### バックオフ戦略

```rust
// 接続失敗時: ExponentialBackoff（ジッタ付き）
// ストリーム切断時: クライアント内部バックオフ
```

| 状況 | バックオフ |
|------|-----------|
| 接続成功 | リセット |
| 接続失敗 | 1s→2s→4s→...→60s（+ジッタ） |
| ストリームエラー | クライアント内部で管理 |
| 認証エラー | セカンダリキーにフォールバック |

### 認証エラー対応

APIキーが無効な場合、セカンダリキーへ自動フォールバック:

```rust
if status.code() == tonic::Code::Unauthenticated {
    retry_with_secondary = true;
}
```

---

## APIキー管理

### 概要

アプリ同梱キー（Primary/Secondary）とユーザーキー（BYOK）を統合管理。

### 構造

```rust
pub struct ApiKeyManager {
    primary_key: Option<String>,    // ビルド時同梱（環境変数）
    secondary_key: Option<String>,  // フォールバック用
    user_key: Option<String>,       // BYOK
    using_secondary: AtomicBool,    // フォールバック中フラグ
}
```

### 優先順位

1. **BYOK優先モード** (`useBundledKey: false`): ユーザーキー → なければエラー
2. **同梱キー優先モード** (`useBundledKey: true`): Primary → Secondary → ユーザーキー

### Tauriコマンド

```typescript
// ステータス確認
const status = await invoke<ApiKeyStatus>('get_api_key_status');
// { hasBundledKey, hasUserKey, usingSecondary, summary }

// 同梱キー有無
const hasBundled = await invoke<boolean>('has_bundled_api_key');

// BYOKキー設定
await invoke('set_byok_key', { apiKey: 'YOUR_KEY' });

// アクティブキー取得
const key = await invoke<string | null>('get_active_api_key', { preferBundled: true });
```

### 実装

- `src-tauri/src/youtube/api_key_manager.rs`

---

## 参考リンク

- [YouTube Data API v3 リファレンス](https://developers.google.com/youtube/v3/docs)
- [streamList リファレンス](https://developers.google.com/youtube/v3/live/docs/liveChatMessages/streamList)
- [Streaming Live Chat ガイド](https://developers.google.com/youtube/v3/live/streaming-live-chat)
