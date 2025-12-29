use reqwest::Client;
use std::fmt;

use super::errors::YouTubeError;
use super::types::{LiveChatMessagesResponse, LiveStreamStats, VideoResponse};

const API_BASE: &str = "https://www.googleapis.com/youtube/v3";

#[derive(Clone)]
pub struct YouTubeClient {
    client: Client,
    api_key: String,
    /// テスト用: APIのベースURL（デフォルトはAPI_BASE）
    #[cfg(test)]
    base_url: String,
}

impl fmt::Debug for YouTubeClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("YouTubeClient")
            .field("client", &"<Client>")
            .field("api_key", &crate::util::mask_api_key(&self.api_key))
            .finish()
    }
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

    /// テスト用: カスタムベースURLでクライアントを作成
    #[cfg(test)]
    pub fn new_with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// APIのベースURLを取得（テスト時はbase_url、本番時はAPI_BASE）
    #[inline]
    fn get_base_url(&self) -> &str {
        #[cfg(test)]
        {
            &self.base_url
        }
        #[cfg(not(test))]
        {
            API_BASE
        }
    }

    /// APIキーを検証（videos.listでクォータ1消費）
    pub async fn validate_api_key(&self) -> Result<bool, YouTubeError> {
        log::info!("Validating API key (quota cost: 1 unit)");

        let url = format!("{}/videos", self.get_base_url());

        let response = self
            .client
            .get(&url)
            .query(&[("part", "id"), ("id", "dQw4w9WgXcQ"), ("key", &self.api_key)])
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                log::info!("API key validation successful");
                Ok(true)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                log::warn!("API key validation failed: unauthorized");
                Err(YouTubeError::InvalidApiKey)
            }
            reqwest::StatusCode::FORBIDDEN => {
                let error_text = response.text().await?;
                log::warn!("API key validation forbidden: {}", error_text);

                if error_text.contains("quotaExceeded") {
                    Err(YouTubeError::QuotaExceeded)
                } else if error_text.contains("rateLimitExceeded") {
                    Err(YouTubeError::RateLimitExceeded)
                } else {
                    Err(YouTubeError::InvalidApiKey)
                }
            }
            status => {
                log::warn!("API key validation returned unexpected status: {}", status);
                Ok(false)
            }
        }
    }

    /// 動画IDからactiveLiveChatIdを取得
    pub async fn get_live_chat_id(&self, video_id: &str) -> Result<String, YouTubeError> {
        log::info!(
            "Fetching live chat ID for video: {} (quota cost: 1 unit)",
            video_id
        );

        let url = format!("{}/videos", self.get_base_url());

        let response = self
            .client
            .get(&url)
            .query(&[
                ("part", "liveStreamingDetails"),
                ("id", video_id),
                ("key", &self.api_key),
            ])
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::BAD_REQUEST => {
                let error_text = response.text().await?;
                log::warn!("Bad request for video {}: {}", video_id, error_text);

                if error_text.contains("keyInvalid") {
                    return Err(YouTubeError::InvalidApiKey);
                } else {
                    // 動画IDの問題など
                    return Err(YouTubeError::VideoNotFound);
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                log::warn!("Unauthorized - API key invalid");
                return Err(YouTubeError::InvalidApiKey);
            }
            reqwest::StatusCode::FORBIDDEN => {
                let error_text = response.text().await?;
                log::warn!("Forbidden for video {}: {}", video_id, error_text);

                if error_text.contains("quotaExceeded") {
                    return Err(YouTubeError::QuotaExceeded);
                } else if error_text.contains("rateLimitExceeded") {
                    return Err(YouTubeError::RateLimitExceeded);
                } else if error_text.contains("keyInvalid") {
                    return Err(YouTubeError::InvalidApiKey);
                } else {
                    // その他の権限不足など
                    return Err(YouTubeError::VideoNotFound);
                }
            }
            reqwest::StatusCode::NOT_FOUND => {
                log::warn!("Video not found: {}", video_id);
                return Err(YouTubeError::VideoNotFound);
            }
            status => {
                log::warn!("Failed to fetch live chat ID: status {}", status);
                return Err(YouTubeError::VideoNotFound);
            }
        }

        let data: VideoResponse = response.json().await?;

        let chat_id = data
            .items
            .first()
            .and_then(|item| item.live_streaming_details.as_ref())
            .and_then(|details| details.active_live_chat_id.clone())
            .ok_or(YouTubeError::LiveChatNotFound)?;

        log::info!("Live chat ID retrieved: {}", chat_id);
        Ok(chat_id)
    }

    /// ライブチャットメッセージ取得
    pub async fn get_live_chat_messages(
        &self,
        live_chat_id: &str,
        page_token: Option<&str>,
    ) -> Result<LiveChatMessagesResponse, YouTubeError> {
        log::info!(
            "Fetching live chat messages for chat ID: {} (quota cost: ~5 units)",
            live_chat_id
        );

        let url = format!("{}/liveChat/messages", self.get_base_url());

        let mut query_params = vec![
            ("liveChatId", live_chat_id),
            ("part", "snippet,authorDetails"),
            ("key", &self.api_key),
        ];

        // pageTokenがある場合は追加
        let page_token_string;
        if let Some(token) = page_token {
            page_token_string = token.to_string();
            query_params.push(("pageToken", &page_token_string));
            log::debug!("Using page token: {}", token);
        }

        let response = self.client.get(&url).query(&query_params).send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                // JSONパースエラーの詳細を取得するため、まずテキストとして取得
                let body = response.text().await?;
                let data: LiveChatMessagesResponse = serde_json::from_str(&body)
                    .map_err(|e| {
                        log::error!("Failed to parse chat messages response: {}", e);
                        log::debug!("Response body: {}", &body[..std::cmp::min(500, body.len())]);
                        YouTubeError::ParseError(format!("JSON parse error: {}", e))
                    })?;
                log::info!(
                    "Successfully fetched {} messages (polling interval: {}ms)",
                    data.items.len(),
                    data.polling_interval_millis
                );
                Ok(data)
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let error_text = response.text().await?;
                log::error!("YouTube API bad request: {}", error_text);

                if error_text.contains("keyInvalid") {
                    log::error!("API key is invalid");
                    Err(YouTubeError::InvalidApiKey)
                } else if error_text.contains("invalidPageToken") {
                    log::warn!("Invalid page token - will reset pagination");
                    Err(YouTubeError::InvalidPageToken)
                } else {
                    log::error!("Bad request - invalid parameters");
                    Err(YouTubeError::ParseError(format!(
                        "Bad request: {}",
                        error_text
                    )))
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                log::error!("Unauthorized - API key invalid");
                Err(YouTubeError::InvalidApiKey)
            }
            reqwest::StatusCode::FORBIDDEN => {
                let error_text = response.text().await?;
                log::error!("YouTube API forbidden error: {}", error_text);

                if error_text.contains("quotaExceeded") {
                    log::error!("Quota exceeded - daily limit reached");
                    Err(YouTubeError::QuotaExceeded)
                } else if error_text.contains("rateLimitExceeded") {
                    log::warn!("Rate limit exceeded - will retry with backoff");
                    Err(YouTubeError::RateLimitExceeded)
                } else if error_text.contains("liveChatDisabled") {
                    log::warn!("Live chat is disabled for this video");
                    Err(YouTubeError::LiveChatDisabled)
                } else {
                    log::error!("API key invalid or insufficient permissions");
                    Err(YouTubeError::InvalidApiKey)
                }
            }
            reqwest::StatusCode::NOT_FOUND => {
                log::warn!("Live chat not found - stream may have ended");
                Err(YouTubeError::LiveChatNotFound)
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                log::error!(
                    "Unexpected API response - status: {}, body: {}",
                    status,
                    error_text
                );
                Err(YouTubeError::ParseError(format!(
                    "Unexpected status: {} - {}",
                    status, error_text
                )))
            }
        }
    }

    /// ライブ配信の視聴者数・統計情報を取得（クォータ消費: 約3 units）
    pub async fn get_live_stream_stats(
        &self,
        video_id: &str,
    ) -> Result<LiveStreamStats, YouTubeError> {
        log::debug!(
            "Fetching live stream stats for video: {} (quota cost: ~3 units)",
            video_id
        );

        let url = format!("{}/videos", self.get_base_url());

        let response = self
            .client
            .get(&url)
            .query(&[
                ("part", "liveStreamingDetails,statistics"),
                ("id", video_id),
                ("key", &self.api_key),
            ])
            .send()
            .await?;

        let status = response.status();
        match status {
            reqwest::StatusCode::OK => {}
            reqwest::StatusCode::FORBIDDEN => {
                let error_text = response.text().await?;
                log::warn!("YouTube API 403 Forbidden: {}", error_text);
                if error_text.contains("quotaExceeded") {
                    return Err(YouTubeError::QuotaExceeded);
                } else if error_text.contains("rateLimitExceeded") {
                    return Err(YouTubeError::RateLimitExceeded);
                }
                return Err(YouTubeError::InvalidApiKey);
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                log::warn!("YouTube API 401 Unauthorized");
                return Err(YouTubeError::InvalidApiKey);
            }
            reqwest::StatusCode::NOT_FOUND => {
                log::warn!("YouTube API 404 Not Found for video: {}", video_id);
                return Err(YouTubeError::VideoNotFound);
            }
            reqwest::StatusCode::BAD_REQUEST => {
                let error_text = response.text().await?;
                log::warn!("YouTube API 400 Bad Request: {}", error_text);
                // 不正な動画IDの可能性
                return Err(YouTubeError::VideoNotFound);
            }
            _ if status.is_server_error() => {
                // 5xx サーバーエラー: 一時的な障害の可能性
                let error_text = response.text().await.unwrap_or_default();
                log::error!("YouTube API server error ({}): {}", status, error_text);
                return Err(YouTubeError::ApiError(format!(
                    "サーバーエラー ({}): 一時的な障害の可能性があります",
                    status
                )));
            }
            _ => {
                // その他の予期しないステータス
                let error_text = response.text().await.unwrap_or_default();
                log::error!("Unexpected YouTube API status ({}): {}", status, error_text);
                return Err(YouTubeError::ApiError(format!(
                    "予期しないエラー ({})",
                    status
                )));
            }
        }

        let data: VideoResponse = response.json().await?;

        let item = data.items.first().ok_or(YouTubeError::VideoNotFound)?;

        let concurrent_viewers = item
            .live_streaming_details
            .as_ref()
            .and_then(|d| d.concurrent_viewers.as_ref())
            .and_then(|v| v.parse::<i64>().ok());

        let like_count = item
            .statistics
            .as_ref()
            .and_then(|s| s.like_count.as_ref())
            .and_then(|v| v.parse::<i64>().ok());

        let view_count = item
            .statistics
            .as_ref()
            .and_then(|s| s.view_count.as_ref())
            .and_then(|v| v.parse::<i64>().ok());

        Ok(LiveStreamStats {
            concurrent_viewers,
            like_count,
            view_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    // =============================================================================
    // get_live_stream_stats HTTPステータスマッピングテスト
    // =============================================================================

    #[tokio::test]
    async fn test_get_live_stream_stats_success() {
        let mut server = Server::new_async().await;

        // 正常なレスポンス
        let response_body = serde_json::json!({
            "items": [{
                "statistics": {
                    "viewCount": "1000",
                    "likeCount": "100"
                },
                "liveStreamingDetails": {
                    "concurrentViewers": "50"
                }
            }]
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("id".into(), "test_video".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.view_count, Some(1000));
        assert_eq!(stats.like_count, Some(100));
        assert_eq!(stats.concurrent_viewers, Some(50));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_empty_items() {
        let mut server = Server::new_async().await;

        // items配列が空
        let response_body = serde_json::json!({
            "items": []
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("nonexistent").await;
        assert!(matches!(result, Err(YouTubeError::VideoNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_403_quota_exceeded() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "quotaExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(matches!(result, Err(YouTubeError::QuotaExceeded)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_403_rate_limit() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "rateLimitExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(matches!(result, Err(YouTubeError::RateLimitExceeded)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_403_other() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"message": "Forbidden"}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_401_unauthorized() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_404_not_found() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(404)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(matches!(result, Err(YouTubeError::VideoNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_400_bad_request() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body("Invalid video ID")
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("invalid!id").await;
        assert!(matches!(result, Err(YouTubeError::VideoNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_500_server_error() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        match result {
            Err(YouTubeError::ApiError(msg)) => {
                assert!(msg.contains("サーバーエラー"));
            }
            _ => panic!("Expected ApiError, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_502_bad_gateway() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(502)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        match result {
            Err(YouTubeError::ApiError(msg)) => {
                assert!(msg.contains("サーバーエラー"));
                assert!(msg.contains("502"));
            }
            _ => panic!("Expected ApiError, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_503_service_unavailable() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        match result {
            Err(YouTubeError::ApiError(msg)) => {
                assert!(msg.contains("サーバーエラー"));
            }
            _ => panic!("Expected ApiError, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_unexpected_status() {
        let mut server = Server::new_async().await;

        // 418 I'm a teapot - 予期しないステータス
        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(418)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        match result {
            Err(YouTubeError::ApiError(msg)) => {
                assert!(msg.contains("予期しないエラー"));
                assert!(msg.contains("418"));
            }
            _ => panic!("Expected ApiError, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_get_live_stream_stats_partial_data() {
        let mut server = Server::new_async().await;

        // viewCountのみ存在（likeCount, concurrentViewersなし）
        let response_body = serde_json::json!({
            "items": [{
                "statistics": {
                    "viewCount": "5000"
                }
            }]
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_stream_stats("test_video").await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.view_count, Some(5000));
        assert_eq!(stats.like_count, None);
        assert_eq!(stats.concurrent_viewers, None);
    }

    // =============================================================================
    // validate_api_key HTTPステータスマッピングテスト
    // =============================================================================

    #[tokio::test]
    async fn test_validate_api_key_success() {
        let mut server = Server::new_async().await;

        let response_body = serde_json::json!({
            "items": [{"id": "dQw4w9WgXcQ"}]
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.validate_api_key().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_validate_api_key_401_unauthorized() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "invalid_key".to_string(),
            server.url(),
        );

        let result = client.validate_api_key().await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_validate_api_key_403_quota_exceeded() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "quotaExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.validate_api_key().await;
        assert!(matches!(result, Err(YouTubeError::QuotaExceeded)));
    }

    #[tokio::test]
    async fn test_validate_api_key_403_rate_limit() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "rateLimitExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.validate_api_key().await;
        assert!(matches!(result, Err(YouTubeError::RateLimitExceeded)));
    }

    #[tokio::test]
    async fn test_validate_api_key_403_other() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"message": "Forbidden"}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.validate_api_key().await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_validate_api_key_unexpected_status() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.validate_api_key().await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // 予期しないステータスはfalseを返す
    }

    // =============================================================================
    // get_live_chat_id HTTPステータスマッピングテスト
    // =============================================================================

    #[tokio::test]
    async fn test_get_live_chat_id_success() {
        let mut server = Server::new_async().await;

        let response_body = serde_json::json!({
            "items": [{
                "liveStreamingDetails": {
                    "activeLiveChatId": "test_chat_id_123"
                }
            }]
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_chat_id_123");
    }

    #[tokio::test]
    async fn test_get_live_chat_id_no_chat_id() {
        let mut server = Server::new_async().await;

        // liveStreamingDetailsはあるが、activeLiveChatIdがない
        let response_body = serde_json::json!({
            "items": [{
                "liveStreamingDetails": {}
            }]
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(matches!(result, Err(YouTubeError::LiveChatNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_empty_items() {
        let mut server = Server::new_async().await;

        let response_body = serde_json::json!({
            "items": []
        });

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("nonexistent_video").await;
        assert!(matches!(result, Err(YouTubeError::LiveChatNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_400_key_invalid() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"error": {"errors": [{"reason": "keyInvalid"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "invalid_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_400_other() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"error": {"message": "Invalid video ID"}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("invalid!video!id").await;
        assert!(matches!(result, Err(YouTubeError::VideoNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_401_unauthorized() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "invalid_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_403_quota_exceeded() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "quotaExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(matches!(result, Err(YouTubeError::QuotaExceeded)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_403_rate_limit() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "rateLimitExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(matches!(result, Err(YouTubeError::RateLimitExceeded)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_403_key_invalid() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "keyInvalid"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "invalid_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_404_not_found() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(404)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("nonexistent_video").await;
        assert!(matches!(result, Err(YouTubeError::VideoNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_chat_id_500_server_error() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/videos")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_id("test_video").await;
        // 予期しないステータスはVideoNotFoundになる（現在の実装）
        assert!(matches!(result, Err(YouTubeError::VideoNotFound)));
    }

    // =============================================================================
    // get_live_chat_messages HTTPステータスマッピングテスト
    // =============================================================================

    #[tokio::test]
    async fn test_get_live_chat_messages_success() {
        let mut server = Server::new_async().await;

        let response_body = serde_json::json!({
            "pollingIntervalMillis": 5000,
            "nextPageToken": "next_token_123",
            "items": [{
                "id": "msg_123",
                "snippet": {
                    "type": "textMessageEvent",
                    "displayMessage": "Hello World!",
                    "publishedAt": "2025-01-01T00:00:00Z"
                },
                "authorDetails": {
                    "channelId": "UC123",
                    "displayName": "TestUser",
                    "profileImageUrl": "https://example.com/image.jpg",
                    "isVerified": false,
                    "isChatOwner": false,
                    "isChatSponsor": false,
                    "isChatModerator": false
                }
            }]
        });

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.polling_interval_millis, 5000);
        assert_eq!(response.next_page_token, Some("next_token_123".to_string()));
        assert_eq!(response.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_with_page_token() {
        let mut server = Server::new_async().await;

        let response_body = serde_json::json!({
            "pollingIntervalMillis": 3000,
            "items": []
        });

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client
            .get_live_chat_messages("test_chat_id", Some("page_token"))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_400_key_invalid() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"error": {"errors": [{"reason": "keyInvalid"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "invalid_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_400_invalid_page_token() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"error": {"errors": [{"reason": "invalidPageToken"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client
            .get_live_chat_messages("test_chat_id", Some("invalid_token"))
            .await;
        assert!(matches!(result, Err(YouTubeError::InvalidPageToken)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_400_other() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_body(r#"{"error": {"message": "Bad request"}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::ParseError(_))));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_401_unauthorized() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(401)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "invalid_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_403_quota_exceeded() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "quotaExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::QuotaExceeded)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_403_rate_limit() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "rateLimitExceeded"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::RateLimitExceeded)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_403_live_chat_disabled() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"errors": [{"reason": "liveChatDisabled"}]}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::LiveChatDisabled)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_403_other() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(403)
            .with_body(r#"{"error": {"message": "Forbidden"}}"#)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::InvalidApiKey)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_404_not_found() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(404)
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("nonexistent_chat", None).await;
        assert!(matches!(result, Err(YouTubeError::LiveChatNotFound)));
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_500_server_error() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        match result {
            Err(YouTubeError::ParseError(msg)) => {
                assert!(msg.contains("500"));
            }
            _ => panic!("Expected ParseError, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_get_live_chat_messages_invalid_json() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/liveChat/messages")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("not valid json")
            .create_async()
            .await;

        let client = YouTubeClient::new_with_base_url(
            "test_api_key".to_string(),
            server.url(),
        );

        let result = client.get_live_chat_messages("test_chat_id", None).await;
        assert!(matches!(result, Err(YouTubeError::ParseError(_))));
    }
}
