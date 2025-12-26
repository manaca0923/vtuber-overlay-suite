# PR #57 天気API連携 レビュー指摘事項

## 概要

天気API連携機能（T25）のCodexレビューで指摘された問題と解決方法をまとめる。

## 指摘事項

### 1. キャッシュTTL整合性問題（高リスク）

**問題**:
`cache.rs`で`is_expired()`がハードコードされた`CACHE_TTL_SECS`定数を使用し、`ttl_remaining()`は`ttl_secs`フィールドを使用していた。テストやカスタムTTL設定時に、TTL表示と実際のキャッシュ有効期限が乖離する可能性があった。

**解決策**:
```rust
// Before: 定数を使用
fn is_expired(&self) -> bool {
    self.created_at.elapsed() > Duration::from_secs(CACHE_TTL_SECS)
}

// After: フィールド値を引数で受け取り一貫性を確保
fn is_expired(&self, ttl_secs: u64) -> bool {
    self.created_at.elapsed() > Duration::from_secs(ttl_secs)
}

// 呼び出し側で self.ttl_secs を渡す
Some(e) if !e.is_expired(self.ttl_secs) => { ... }
```

**教訓**:
- キャッシュのTTL値は一箇所で管理し、すべての判定ロジックで同じ値を使用すること
- テスト用にカスタムTTLを設定できる場合、本番コードがその設定を正しく使用しているか確認すること
- TTLの一貫性テストを追加すること（`test_ttl_consistency_between_get_and_ttl_remaining`）

---

### 2. React状態管理の分離

**問題**:
`WeatherSettingsPanel.tsx`で`hasKey`をUI編集モードの状態としても兼用していたため、「変更」ボタンをクリックすると実際にはAPIキーが存在するのに「未設定」のような見た目になる可能性があった。

**解決策**:
```typescript
// Before: hasKeyを編集モード判定にも使用
const [hasKey, setHasKey] = useState(false);
// hasKeyがfalseになると「未設定」表示になってしまう

// After: 編集モード状態を分離
const [hasKey, setHasKey] = useState(false);
const [isEditingKey, setIsEditingKey] = useState(false);

// キャンセル時はバックエンド状態を再取得
const handleCancelEditKey = useCallback(async () => {
  setIsEditingKey(false);
  setApiKeyValue('');
  const hasApiKey = await hasWeatherApiKey();
  setHasKey(hasApiKey);
}, []);
```

**教訓**:
- UI状態（編集中かどうか）とデータ状態（実際に値が存在するか）は分離すること
- キャンセル操作時はバックエンドの実際の状態を再取得して整合性を保つこと
- 状態の意味を明確にする命名をすること（`isEditingKey`など）

---

### 3. HTTPエラーマッピングの精密化

**問題**:
`youtube/client.rs`で404以外の非OKレスポンスがすべて`VideoNotFound`にマッピングされていた。一時的なサーバー障害なのか、無効な動画IDなのか区別できなかった。

**解決策**:
```rust
match status {
    reqwest::StatusCode::OK => {}
    reqwest::StatusCode::FORBIDDEN => { /* quota/rate limit checks */ }
    reqwest::StatusCode::UNAUTHORIZED => Err(YouTubeError::InvalidApiKey),
    reqwest::StatusCode::NOT_FOUND => Err(YouTubeError::VideoNotFound),
    reqwest::StatusCode::BAD_REQUEST => {
        log::warn!("YouTube API 400 Bad Request: {}", error_text);
        Err(YouTubeError::VideoNotFound) // 無効な動画ID
    }
    _ if status.is_server_error() => {
        log::error!("YouTube API server error ({})", status);
        Err(YouTubeError::ApiError("サーバーエラー: 一時的な障害の可能性".into()))
    }
    _ => {
        log::error!("Unexpected YouTube API status ({})", status);
        Err(YouTubeError::ApiError(format!("予期しないエラー ({})", status)))
    }
}
```

**教訓**:
- HTTPステータスコードごとに適切なエラー型にマッピングすること
- 5xxサーバーエラーは一時的障害として扱い、4xxクライアントエラーとは区別すること
- すべてのエラーケースでログを出力し、デバッグを容易にすること
- 「catch-all」のデフォルトケースでも具体的なエラー情報を提供すること

---

### 4. キャッシュの都市キー問題（高リスク・追加指摘）

**問題**:
キャッシュが単一エントリで都市名をキーとしていなかった。`set_city`で都市変更時にキャッシュをクリアしても、並行実行中の`get_weather`が古い都市のデータを返す競合状態が発生する可能性があった。

**解決策**:
```rust
// Before: 都市名なしのキャッシュエントリ
struct CacheEntry {
    data: WeatherData,
    created_at: Instant,
}

// After: 都市名を含むキャッシュエントリ
struct CacheEntry {
    data: WeatherData,
    city: String,  // 追加
    created_at: Instant,
}

// get()で都市名も検証
pub async fn get(&self, city: &str) -> Option<WeatherData> {
    match entry.as_ref() {
        Some(e) if !e.is_expired(self.ttl_secs) && e.matches_city(city) => {
            Some(e.data.clone())
        }
        // ...
    }
}
```

**教訓**:
- キャッシュにはデータだけでなく、そのデータを識別するキー情報も保存すること
- 並行実行時の競合状態を考慮し、取得時にもキーを検証すること
- 都市変更テスト（`test_cache_city_mismatch`, `test_cache_city_change_invalidates_old_city`）を追加

---

### 5. HTTPタイムアウト未設定（高リスク・追加指摘）

**問題**:
`reqwest::Client::new()`はデフォルトでタイムアウトがなく、外部APIがハングするとUIがフリーズする。

**解決策**:
```rust
// Before: タイムアウトなし
let client = Client::new();

// After: 10秒タイムアウト設定
const HTTP_TIMEOUT_SECS: u64 = 10;

let client = Client::builder()
    .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
    .build()
    .unwrap_or_else(|_| Client::new());

// タイムアウトエラーを専用の型で処理
.map_err(|e| {
    if e.is_timeout() {
        WeatherError::Timeout
    } else {
        WeatherError::HttpError(e)
    }
})
```

**教訓**:
- 外部APIへのHTTPリクエストには必ずタイムアウトを設定すること
- タイムアウトエラーは専用のエラー型で処理し、ユーザーに分かりやすいメッセージを表示
- ライブツールでは信頼性が重要なため、ネットワーク障害への耐性を確保

---

## テスト追加

レビュー指摘に基づき、以下のテストを追加した：

### cache.rs
- `test_cache_expiry_with_short_ttl`: 短いTTLでの期限切れ検証
- `test_ttl_consistency_between_get_and_ttl_remaining`: get()とttl_remaining()のTTL整合性
- `test_cache_city_mismatch`: 都市名不一致時のキャッシュミス検証（追加）
- `test_cache_city_change_invalidates_old_city`: 都市変更後のキャッシュ無効化検証（追加）

### types.rs
- `test_from_openweathermap_full_parsing`: OpenWeatherMapレスポンスの完全パース検証
- `test_from_openweathermap_empty_weather`: 空のweather配列処理
- `test_from_openweathermap_negative_temp`: 負の気温の処理

---

### 6. 都市の二重読み取り競合（高リスク・追加指摘）

**問題**:
`get_weather()`で都市を読み取り、`fetch_weather()`で再度都市を読み取っていた。都市変更がこの間に発生すると、フェッチしたデータが古い都市のキャッシュキーで保存され、不整合が発生する。

**解決策**:
```rust
// Before: 都市を複数回読み取り（競合の可能性）
pub async fn get_weather(&self) -> Result<WeatherData, WeatherError> {
    let city = self.city.read().await.clone();
    // ... cache check
    let data = self.fetch_weather().await?;  // fetch_weatherで再度cityを読む
    self.cache.set(data.clone(), city).await;  // 最初のcityでキャッシュ
}

// After: 都市を一度だけ読み取り、同じ値を使用
pub async fn get_weather(&self) -> Result<WeatherData, WeatherError> {
    let city = self.city.read().await.clone();
    // ... cache check
    let data = self.fetch_weather_for_city(&city).await?;  // 同じcityを渡す
    self.cache.set(data.clone(), city).await;  // 同じcityでキャッシュ
}

// 内部関数で都市を引数として受け取る
async fn fetch_weather_for_city(&self, city: &str) -> Result<WeatherData, WeatherError> {
    // cityを直接使用（再読み取りしない）
}
```

**教訓**:
- 並行処理で複数回の状態読み取りがある場合、競合状態を考慮する
- 一度読み取った値を後続の処理で一貫して使用することで競合を防止
- 内部関数に値を渡すパターンで、読み取り回数を最小化する

---

### 7. 天気APIキーのkeyring保存（高リスク・追加指摘）

**問題**:
天気APIキーがメモリにのみ保存され、アプリ再起動で失われていた。CLAUDE.mdのセキュリティガイドライン（「APIキーはOSセキュアストレージに保存」）に違反。

**解決策**:
```rust
// keyring.rsに天気APIキー用の関数を追加
pub fn save_weather_api_key(api_key: &str) -> Result<(), KeyringError>;
pub fn get_weather_api_key() -> Result<String, KeyringError>;
pub fn has_weather_api_key() -> Result<bool, KeyringError>;

// commands/weather.rsでkeyringを使用
pub async fn set_weather_api_key(...) -> Result<(), String> {
    keyring::save_weather_api_key(&api_key).map_err(|e| e.to_string())?;
    state.weather.set_api_key(api_key).await;
    Ok(())
}

// lib.rsで起動時にkeyringからAPIキーを復元
if let Ok(api_key) = keyring::get_weather_api_key() {
    weather_clone.set_api_key(api_key).await;
    log::info!("Weather API key restored from keyring");
}
```

**教訓**:
- 新しい外部API連携を追加する際は、既存のセキュリティパターン（keyring使用）に従う
- APIキーは必ず永続化し、再起動後も利用可能にする
- セキュリティに関するガイドライン（CLAUDE.md等）を常に参照する

---

### 8. タイムアウトフォールバック問題（高リスク・追加指摘）

**問題**:
`Client::builder().build().unwrap_or_else(|_| Client::new())` としていたため、ビルダーが失敗した場合にタイムアウトなしの `Client::new()` にフォールバックしていた。これでは「外部APIがハングするとUIがフリーズする」問題が再発する。

**解決策**:
```rust
// Before: フォールバックでタイムアウトなしクライアントを使用
let client = Client::builder()
    .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
    .build()
    .unwrap_or_else(|_| Client::new());

// After: expect()で確実にタイムアウト付きクライアントを使用
let client = Client::builder()
    .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
    .build()
    .expect("Failed to build HTTP client with timeout - this should never fail");
```

**教訓**:
- エラー回復時にセキュリティ/信頼性要件を緩めるフォールバックは避ける
- HTTPクライアント構築の失敗は起動時のみ発生し得るため、panicで早期発見する方が安全
- タイムアウト設定は「あれば良い」ではなく「必須」として扱う

---

### 9. APIキー状態の非同期問題（高リスク・追加指摘）

**問題**:
`has_weather_api_key`コマンドはkeyringから確認し、`get_weather`はメモリ内のキーを使用していた。起動時のkeyring復元が失敗した場合（キーチェーンロック解除遅延等）、UIは「設定済み」を表示するが、天気取得は`ApiKeyNotConfigured`で失敗する。

**解決策**:
```rust
// Before: keyringの確認のみ
#[tauri::command]
pub async fn has_weather_api_key(state: State<'_, AppState>) -> Result<bool, String> {
    match keyring::get_weather_api_key() {
        Ok(_) => Ok(true),
        Err(keyring::KeyringError::NotFound) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

// After: keyringにあるがメモリにない場合は同期
#[tauri::command]
pub async fn has_weather_api_key(state: State<'_, AppState>) -> Result<bool, String> {
    match keyring::get_weather_api_key() {
        Ok(api_key) => {
            // keyringにキーがある場合、メモリにもセット（まだ無い場合）
            if !state.weather.has_api_key().await {
                state.weather.set_api_key(api_key).await;
                log::info!("Weather API key synced from keyring to memory");
            }
            Ok(true)
        }
        Err(keyring::KeyringError::NotFound) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}
```

**教訓**:
- 永続ストレージとメモリキャッシュの両方を持つ場合、同期ポイントを設ける
- UI表示用の確認関数で同期を行うと、表示タイミングで自動的に状態が整合する
- 起動時の復元失敗を想定し、後からリカバリできる設計にする

---

### 10. TTL残り時間の都市考慮不足（高リスク・追加指摘）

**問題**:
`ttl_remaining()`が都市を考慮せずにキャッシュの残り時間を返していた。都市変更後でもUIに正の値のTTLが表示され、実際にはキャッシュミスになるのにユーザーが有効なキャッシュがあると誤解する。

**解決策**:
```rust
// Before: 都市を無視
pub async fn ttl_remaining(&self) -> u64 {
    match entry.as_ref() {
        Some(e) => { /* TTL計算 */ }
        None => 0,
    }
}

// After: 都市も検証
pub async fn ttl_remaining(&self, city: &str) -> u64 {
    match entry.as_ref() {
        Some(e) if e.matches_city(city) => { /* TTL計算 */ }
        _ => 0,  // 都市不一致または期限切れの場合は0
    }
}

// WeatherClientでも現在の都市を渡す
pub async fn cache_ttl_remaining(&self) -> u64 {
    let city = self.city.read().await.clone();
    self.cache.ttl_remaining(&city).await
}
```

**教訓**:
- キャッシュの`get()`と`ttl_remaining()`は同じ条件で判定すること
- TTL表示とキャッシュヒット条件が乖離するとUXが混乱する
- テスト追加: `test_cache_ttl_remaining_city_mismatch`

---

### 11. ブロードキャスト時のキャッシュデータ問題（高リスク・追加指摘）

**問題**:
`broadcast_weather_update`が常にキャッシュ優先で天気データを取得していた。都市やAPIキー変更直後に呼び出すと、古いキャッシュデータがブロードキャストされる可能性があった。

**解決策**:
```rust
// Before: 常にキャッシュ優先
#[tauri::command]
pub async fn broadcast_weather_update(state: State<'_, AppState>) -> Result<(), String> {
    let weather_data = state.weather.get_weather().await.map_err(|e| e.to_string())?;
    // ...
}

// After: force_refreshオプションを追加
#[tauri::command]
pub async fn broadcast_weather_update(
    state: State<'_, AppState>,
    force_refresh: Option<bool>,
) -> Result<(), String> {
    let weather_data = if force_refresh.unwrap_or(false) {
        state.weather.clear_cache().await;
        state.weather.get_weather().await.map_err(|e| e.to_string())?
    } else {
        state.weather.get_weather().await.map_err(|e| e.to_string())?
    };
    // ...
}
```

**フロントエンド対応**:
```typescript
// TypeScript側も更新
export const broadcastWeatherUpdate = (forceRefresh?: boolean) =>
  invoke<void>('broadcast_weather_update', { forceRefresh });

// UIに「強制配信」ボタンを追加
<button onClick={() => broadcastWeatherUpdate(true)}>強制配信</button>
```

**教訓**:
- ブロードキャスト/通知系コマンドには強制リフレッシュオプションを設ける
- 設定変更直後は古いキャッシュが残っている可能性を考慮する
- UIでユーザーが意図的に最新データを送信できる手段を提供する

---

### 12. 天気取得時のkeyringフォールバック不足（高リスク・追加指摘）

**問題**:
`get_weather`/`fetch_weather`がメモリ内のAPIキーのみを使用していた。起動時のkeyring復元失敗時（キーチェーンロック等）、UIを開くまで天気取得が失敗し続け、オーバーレイが動作しない。

**解決策**:
```rust
// 共通のkeyring同期ヘルパーを追加
async fn ensure_api_key_synced(state: &AppState) {
    if !state.weather.has_api_key().await {
        if let Ok(api_key) = keyring::get_weather_api_key() {
            state.weather.set_api_key(api_key).await;
            log::info!("Weather API key synced from keyring to memory (auto-recovery)");
        }
    }
}

// 各コマンドで同期を呼び出す
#[tauri::command]
pub async fn get_weather(state: State<'_, AppState>) -> Result<WeatherData, String> {
    ensure_api_key_synced(&state).await;  // 追加
    state.weather.get_weather().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_weather(state: State<'_, AppState>) -> Result<WeatherData, String> {
    ensure_api_key_synced(&state).await;  // 追加
    state.weather.clear_cache().await;
    state.weather.get_weather().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn broadcast_weather_update(...) -> Result<(), String> {
    ensure_api_key_synced(&state).await;  // 追加
    // ...
}
```

**教訓**:
- 永続ストレージとメモリの両方を持つ場合、データ取得の各エントリポイントで同期を確認する
- UIを開かなくても呼ばれる可能性のあるコマンド（定期更新、ブロードキャスト等）には必ず同期ロジックを入れる
- 共通の同期ヘルパー関数を作成し、コードの重複を避ける

---

### 13. async await中のRwLockガード保持（Concrete fix）

**問題**:
`fetch_weather_for_city`で`api_key`のread lockを取得した後、`.send().await`のネットワークリクエスト中もロックを保持していた。これにより、遅いリクエスト中に`set_api_key`がブロックされる。

**解決策**:
```rust
// Before: ロックをawait中も保持
async fn fetch_weather_for_city(&self, city: &str) -> Result<WeatherData, WeatherError> {
    let api_key = self.api_key.read().await;
    let api_key = api_key.as_ref().ok_or(WeatherError::ApiKeyNotConfigured)?;
    // ... await中もロックを保持 ...
    let response = self.client.get(...).send().await?;
}

// After: cloneしてロックを即座に解放
async fn fetch_weather_for_city(&self, city: &str) -> Result<WeatherData, WeatherError> {
    let api_key = {
        let guard = self.api_key.read().await;
        guard.as_ref().ok_or(WeatherError::ApiKeyNotConfigured)?.clone()
    };
    // ロックは解放済み、awaitは安全
    let response = self.client.get(...).send().await?;
}
```

**教訓**:
- RwLockのガードをasync await境界をまたいで保持しない
- ネットワークI/O前に必要なデータをcloneしてガードをdrop
- Stringなど小さなデータはcloneのコストより並行性を優先

---

## 今後の注意点

1. **キャッシュ実装時**: TTL値は一箇所で管理し、すべての判定で同じ値を使用する
2. **キャッシュのキー管理**: キャッシュにはデータを識別するキー情報も保存し、取得時に検証する
3. **React状態管理**: UI状態とデータ状態は分離し、キャンセル時はバックエンド状態を再取得
4. **API連携時**: HTTPステータスコードごとに適切なエラー型を定義し、ログを出力する
5. **HTTPタイムアウト**: 外部APIへのリクエストには必ずタイムアウトを設定（10秒推奨）
6. **テスト**: 境界条件（TTL期限切れ、空データ、エッジケース、競合状態）を網羅する
7. **並行処理の状態読み取り**: 複数回の読み取りは競合を招くため、一度だけ読み取って使い回す
8. **APIキーのセキュリティ**: 新規API連携時は必ずkeyringパターンに従い永続化する
9. **フォールバックの安全性**: エラー回復時にセキュリティ/信頼性要件を緩めないこと
10. **永続ストレージとメモリの同期**: 両方を持つ場合、UI確認時などに同期ポイントを設ける
11. **TTLとキャッシュヒット条件の一致**: TTL表示はキャッシュヒット条件と同じ条件で判定すること
12. **ブロードキャスト時の強制リフレッシュ**: 設定変更後に確実に最新データを送信できるオプションを提供
13. **全エントリポイントでの永続ストレージ同期**: UIを開かなくても呼ばれるコマンドにも同期ロジックを入れる
14. **RwLockとasync**: ロックガードをawait境界をまたいで保持しない。データをcloneして即座にdrop
