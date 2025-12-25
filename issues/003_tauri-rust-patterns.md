# Tauri/Rust統合パターンのノウハウ

## 概要
PR #19, #38, #39, #40 で受けたTauri/Rust統合関連の指摘と解決方法

---

## 1. Tauri 2.0のinvokeパラメータ命名規則

### 指摘内容 (PR#19)
Tauri 2.0では`#[tauri::command]`マクロがRust側のスネークケースパラメータを自動的にcamelCaseに変換する。

### 解決方法
```typescript
// フロントエンド側（TypeScript）
await invoke('start_polling', {
  apiKey: apiKey,           // camelCase
  liveChatId: liveChatId,   // camelCase
  nextPageToken: token,     // camelCase
});

// Rust側
#[tauri::command]
pub async fn start_polling(
    api_key: String,         // snake_case（自動変換される）
    live_chat_id: String,
    next_page_token: Option<String>,
) -> Result<(), String> { ... }
```

### 今後の対策
- Tauri invokeパラメータはフロントエンドでcamelCase
- Rust側はsnake_case（自動変換に任せる）
- より明示的にするなら`#[serde(rename = "apiKey")]`を使用

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

## チェックリスト（Tauri/Rust実装時）

- [ ] invokeパラメータの命名規則は正しいか（camelCase/snake_case）
- [ ] RwLockのpoisonedエラーをハンドリングしているか
- [ ] 機密情報はkeyringで保存しているか
- [ ] ログにAPIキーが出力されないか
- [ ] グローバル状態は適切に管理されているか
- [ ] 正規表現はシングルトン化されているか
