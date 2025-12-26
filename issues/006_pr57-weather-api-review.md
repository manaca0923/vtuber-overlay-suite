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

## 今後の注意点

1. **キャッシュ実装時**: TTL値は一箇所で管理し、すべての判定で同じ値を使用する
2. **キャッシュのキー管理**: キャッシュにはデータを識別するキー情報も保存し、取得時に検証する
3. **React状態管理**: UI状態とデータ状態は分離し、キャンセル時はバックエンド状態を再取得
4. **API連携時**: HTTPステータスコードごとに適切なエラー型を定義し、ログを出力する
5. **HTTPタイムアウト**: 外部APIへのリクエストには必ずタイムアウトを設定（10秒推奨）
6. **テスト**: 境界条件（TTL期限切れ、空データ、エッジケース、競合状態）を網羅する
