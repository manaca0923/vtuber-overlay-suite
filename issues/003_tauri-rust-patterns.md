# Tauri/Rust統合パターンのノウハウ

## 概要
PR #19, #38, #39, #40 で受けたTauri/Rust統合関連の指摘と解決方法

---

## 1. Tauri 2.0のinvokeパラメータ命名規則

### ⚠️ 重要: 命名規則の訂正 (PR#58で判明)

**以前の記載は誤りでした。** 実際にはTauri 2.0でも**フロントエンド側でsnake_caseを使用する必要があります**。

### 正しい解決方法
```typescript
// フロントエンド側（TypeScript）
await invoke('start_polling', {
  api_key: apiKey,           // ✅ snake_case（Rustと一致させる）
  live_chat_id: liveChatId,  // ✅ snake_case
  next_page_token: token,    // ✅ snake_case
});

// Rust側
#[tauri::command]
pub async fn start_polling(
    api_key: String,         // snake_case
    live_chat_id: String,
    next_page_token: Option<String>,
) -> Result<(), String> { ... }
```

### 今後の対策
- Tauri invokeパラメータは**フロントエンドでもsnake_case**
- Rust側のパラメータ名と完全に一致させる
- 詳細は `issues/007_tauri-invoke-snake-case.md` を参照

---

## 2. RwLockのポイズンエラーハンドリング

### 指摘内容 (PR#39)
`RwLock::read()`や`write()`が`Err`を返すのはpoisoned状態のときのみ。詳細なエラー情報をログに出力すべき。

### 解決方法
```rust
match manager.write() {
    Ok(mut guard) => {
        // 正常処理
        guard.switch_to_secondary();
    }
    Err(poison_error) => {
        // パニック原因をログ出力
        log::error!(
            "API key manager write lock is poisoned: {}",
            poison_error
        );
        return Err(YouTubeError::ApiError(
            "API key manager lock poisoned".to_string(),
        ));
    }
}
```

### 今後の対策
- poisonedエラーはパニックの結果なので、詳細をログ出力
- 可能なら回復処理を試みる
- 共通のヘルパー関数やマクロに抽出を検討

---

## 3. APIキーのセキュア保存

### 指摘内容 (PR#19)
APIキーがSQLiteに平文で保存されるのはセキュリティリスク。

### 解決方法
```rust
// keyringクレートを使用したOS APIへの保存
use keyring::Entry;

pub async fn save_api_key(api_key: String) -> Result<(), String> {
    if api_key.trim().is_empty() {
        return Err("APIキーが空です".to_string());
    }

    // ブロッキング呼び出しをラップ
    tokio::task::spawn_blocking(move || {
        let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)?;
        entry.set_password(&api_key)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
```

### 今後の対策
- 機密情報はkeyringクレートでOS APIに保存
- macOS=Keychain, Windows=Credential Manager
- ブロッキング呼び出しは`tokio::task::spawn_blocking`でラップ

---

## 4. ログにおけるAPIキーのマスキング

### 指摘内容 (PR#19)
APIキーがログに出力されないよう注意が必要。

### 解決方法
```rust
impl std::fmt::Debug for YouTubeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YouTubeClient")
            .field("api_key", &"[REDACTED]")  // マスキング
            .finish()
    }
}

// ログ出力時のマスキング関数
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        return "***".to_string();
    }
    format!("{}...{}", &key[..4], &key[key.len()-4..])
}
```

### 今後の対策
- Debugトレイトでは機密情報をマスキング
- ログ出力時は専用のマスキング関数を使用
- URLにAPIキーが含まれる場合も注意

---

## 5. グローバル状態の管理

### 指摘内容 (PR#40)
複数のグローバル状態（`OnceLock`、`Mutex`など）が並行して存在すると混乱の原因になる。

### 解決方法
```rust
// 推奨: 単一のAppStateに統合
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub ws_clients: Arc<Mutex<HashMap<u32, WsClient>>>,
    pub poller: Arc<Mutex<Option<Poller>>>,
}

// Tauriのmanaged stateとして使用
fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .run(tauri::generate_context!())
        .expect("error while running");
}
```

### 今後の対策
- グローバル状態は`AppState`に統合
- Tauriの`manage()`で状態を管理
- 新しい状態は既存のパターンに従う

---

## 6. 正規表現のシングルトン化

### 指摘内容 (PR#24)
正規表現を毎回コンパイルするとパフォーマンスに影響。

### 解決方法
```rust
use std::sync::OnceLock;
use regex::Regex;

static CONTINUATION_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_continuation_regex() -> &'static Regex {
    CONTINUATION_REGEX.get_or_init(|| {
        Regex::new(r#""continuation"\s*:\s*"([^"]+)""#)
            .expect("Invalid regex pattern")
    })
}

// 使用時
let re = get_continuation_regex();
if let Some(caps) = re.captures(html) { ... }
```

### 今後の対策
- 正規表現は`OnceLock`でシングルトン化
- コンパイルエラーは`expect`で処理（ビルド時に検出可能）
- 複雑なパターンはテストでカバー

---

## 7. リトライループのドキュメント明確化

### 指摘内容 (PR#81)
`for attempt in 0..=RESTORE_BUSY_TIMEOUT_MAX_RETRIES` で `RESTORE_BUSY_TIMEOUT_MAX_RETRIES = 3` の場合、ドキュメントに「最大3回リトライ」と記載すると誤解を招く。実際は `0..=3` で4回ループ（0, 1, 2, 3）するため、「初回試行 + 3回リトライ = 4回試行」となる。

### 解決方法
```rust
/// ## リトライ戦略
/// - 初回試行 + 最大3回リトライ = 最大4回試行
/// - リトライ間隔: 20ms → 40ms → 80ms（exponential backoff）
const RESTORE_BUSY_TIMEOUT_MAX_RETRIES: u32 = 3;

async fn restore_busy_timeout_with_retry(
    conn: &mut SqliteConnection,
    original_timeout: u64,
) -> bool {
    for attempt in 0..=RESTORE_BUSY_TIMEOUT_MAX_RETRIES {
        // 0..=3 は 4回ループ: 初回(0) + リトライ(1,2,3)
        match set_busy_timeout(conn, original_timeout).await {
            Ok(()) => return true,
            Err(e) => {
                if attempt >= RESTORE_BUSY_TIMEOUT_MAX_RETRIES {
                    return false; // 最後のリトライも失敗
                }
                // リトライを試みる
            }
        }
    }
    false
}
```

### 今後の対策
- ループ回数を表す定数名は `MAX_RETRIES`（リトライ回数）と `MAX_ATTEMPTS`（試行回数）を使い分ける
- ドキュメントには「初回試行 + 最大N回リトライ = 最大N+1回試行」と明記
- `0..=N` は N+1 回ループする点をコメントで補足

---

## 8. RwLockガードをawait境界をまたいで保持しない

### 指摘内容 (PR#115)
`server.read().await`で取得したガードを保持したまま`.broadcast(...).await`を呼ぶと、デッドロックや性能劣化のリスクがある。

### 問題のあるコード
```rust
// ❌ 危険: ガードを保持したまま.await
let ws_state = state.server.read().await;  // ガード取得
ws_state
    .broadcast(WsMessage::QueueUpdate { payload })
    .await;  // この.awaitの間ガードを保持し続ける
```

### 解決方法: Fire-and-forgetパターン
```rust
use std::sync::Arc;

// ✅ 正しい: Arc::cloneしてtokio::spawnで分離
let server = Arc::clone(&state.server);
tokio::spawn(async move {
    let ws_state = server.read().await;
    ws_state
        .broadcast(WsMessage::QueueUpdate { payload })
        .await;
    log::debug!("Queue update broadcasted");
});

Ok(())  // 呼び出し元は即座にreturn
```

### 設計ノート
- **Fire-and-forget**: ブロードキャストは`tokio::spawn`でバックグラウンド実行
- **呼び出し元はブロードキャスト完了を待たない**: 即座に`Ok(())`を返す
- **ブロードキャスト失敗はログ出力のみ**: 呼び出し元のコマンド成功には影響しない
- **RwLockガードの分離**: `tokio::spawn`で新しいタスクにすることで、ガードがawait境界をまたがない

### 既存の正しい実装例
`src-tauri/src/commands/setlist.rs:830-839` で同様のパターンを使用。

### 今後の対策
- ブロードキャスト処理は`Arc::clone` + `tokio::spawn`パターンを使用
- RwLockガードを取得した後に`.await`を呼ぶ場合は要注意
- 既存コード（youtube.rs, weather.rs, overlay.rs等）にも同様の問題あり → 技術的負債として管理

---

## チェックリスト（Tauri/Rust実装時）

- [ ] invokeパラメータの命名規則は正しいか（フロントエンドもsnake_case）
- [ ] RwLockのpoisonedエラーをハンドリングしているか
- [ ] 機密情報はkeyringで保存しているか
- [ ] ログにAPIキーが出力されないか
- [ ] グローバル状態は適切に管理されているか
- [ ] 正規表現はシングルトン化されているか
- [ ] リトライループのドキュメントは明確か（試行回数 vs リトライ回数）
- [ ] RwLockガードをawait境界をまたいで保持していないか（tokio::spawnで分離）
