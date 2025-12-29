# PR#84/85 HTTPモックテストパターン

## 概要

PR#84/85でYouTube APIとWeather APIのHTTPモックテストを追加した際の実装パターンとレビュー指摘事項。

- PR#84: `get_live_stream_stats`のテスト追加
- PR#85: `validate_api_key`, `get_live_chat_id`, `get_live_chat_messages`のテスト追加（30テスト追加、合計43テスト）

## 使用ライブラリ

- **mockito 1.6**: Rust用HTTPモックライブラリ
- 選定理由: wiremock 0.6.5はRust互換性問題（let chains）で使用不可

## 実装パターン

### 1. テスト用ベースURL注入

プロダクションコードに影響を与えずにテスト用URLを注入するパターン。

```rust
pub struct YouTubeClient {
    client: Client,
    api_key: String,
    #[cfg(test)]
    base_url: String,
}

impl YouTubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            #[cfg(test)]
            base_url: API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    #[inline]
    fn get_base_url(&self) -> &str {
        #[cfg(test)]
        { &self.base_url }
        #[cfg(not(test))]
        { API_BASE }
    }
}
```

**ポイント**:
- `#[cfg(test)]`でテスト専用フィールドを追加（プロダクションビルドに影響なし）
- `get_base_url()`メソッドで条件分岐を隠蔽
- `new_with_base_url()`コンストラクタはテスト専用

### 2. mockitoの基本的な使い方

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_api_error() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)  // クエリパラメータを無視
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(matches!(result, Err(YouTubeError::ApiError(_))));
    }
}
```

**重要**: mockitoは未マッチのリクエストに501を返すため、`match_query(Matcher::Any)`を使用してクエリパラメータを無視する必要がある。

### 3. 複数APIエンドポイントのテスト（Weather API例）

```rust
#[tokio::test]
async fn test_weather_api_500_error() {
    let mut server = Server::new_async().await;

    // Geocoding APIは成功
    let _geocoding_mock = server
        .mock("GET", "/v1/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results": [{"id": 1, "name": "Tokyo", ...}]}"#)
        .create_async()
        .await;

    // Weather APIは500エラー
    let _weather_mock = server
        .mock("GET", "/v1/forecast")
        .match_query(mockito::Matcher::Any)
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = WeatherClient::new_with_base_urls(
        format!("{}/v1/search", server.url()),
        format!("{}/v1/forecast", server.url()),
    );

    let result = client.fetch_weather().await;
    assert!(matches!(result, Err(WeatherError::ApiError { status: 500, .. })));
}
```

## レビュー指摘事項

### 1. クエリパラメータ検証の強化（優先度: 低）

**問題**: `Matcher::Any`を多用すると、誤ったパラメータでもモックが応答してしまう。

**推奨対応**:
```rust
// より厳密な検証
.match_query(mockito::Matcher::AllOf(vec![
    mockito::Matcher::UrlEncoded("key".into(), "test_api_key".into()),
    mockito::Matcher::UrlEncoded("id".into(), "test_video".into()),
]))
```

**現状**: テストの主目的はHTTPステータスコードのマッピング検証であり、`Matcher::Any`で十分。

### 2. ヘルパー関数の抽出（優先度: 低）

**問題**:
- Weather API: Geocodingレスポンスのモック設定が各テストで重複
- YouTube API: セットアップコード（`Server::new_async().await`, `YouTubeClient::new_with_base_url`）が重複

**推奨対応（Weather API）**:
```rust
async fn mock_geocoding_success(server: &mut Server) -> mockito::Mock {
    server
        .mock("GET", "/v1/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results": [{"id": 1, "name": "Tokyo", ...}]}"#)
        .create_async()
        .await
}
```

**推奨対応（YouTube API）** (PR#85):
```rust
async fn setup_test_client() -> (Server, YouTubeClient) {
    let server = Server::new_async().await;
    let client = YouTubeClient::new_with_base_url(
        "test_api_key".to_string(),
        server.url(),
    );
    (server, client)
}
```

### 3. タイムアウトテスト（優先度: 低）

mockitoの`with_delay`を使用してタイムアウトテストが可能:
```rust
.with_chunked_body(|| std::iter::repeat(vec![0; 1]).take(100))
.with_delay(Duration::from_secs(30))
```

ただし、HTTPクライアントのタイムアウト設定（10秒）とテスト実行時間のトレードオフがある。

### 4. 5xxエラーハンドリングの一貫性 ✅ (PR#86で対応済み)

**問題**: `get_live_chat_id`と`get_live_chat_messages`で5xxエラーが不適切なエラー型にマッピングされていた。

**修正内容** (PR#86):
- `get_live_chat_id`: 5xxエラーと予期しないステータスを`ApiError`にマッピング
- `get_live_chat_messages`: 同様に`ApiError`にマッピング（以前は`ParseError`）
- `get_live_stream_stats`との一貫性を確保

**修正後のパターン**:
```rust
// 5xxサーバーエラー
status if status.is_server_error() => {
    Err(YouTubeError::ApiError(format!(
        "サーバーエラー ({}): 一時的な障害の可能性があります",
        status
    )))
}
// その他の予期しないステータス
status => {
    Err(YouTubeError::ApiError(format!(
        "予期しないエラー ({})",
        status
    )))
}
```

## 注意点

1. **mockitoの501エラー**: 未マッチリクエストに501を返すため、必ず適切なマッチャーを設定
2. **非同期テスト**: `Server::new_async()`と`.create_async()`を使用
3. **モック変数**: `_mock`として保持し、テスト終了までドロップされないようにする

## 対象ファイル

- `src-tauri/src/youtube/client.rs`: YouTubeClientの実装とテスト
- `src-tauri/src/weather/mod.rs`: WeatherClientの実装とテスト
- `src-tauri/Cargo.toml`: mockito dev-dependency追加
