use reqwest::Client;

use super::{errors::YouTubeError, types::*};

const API_BASE: &str = "https://www.googleapis.com/youtube/v3";

pub struct YouTubeClient {
    client: Client,
    api_key: String,
}

impl YouTubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// APIキーを検証（videos.listでクォータ1消費）
    pub async fn validate_api_key(&self) -> Result<bool, YouTubeError> {
        log::info!("Validating API key (quota cost: 1 unit)");

        let url = format!("{}/videos", API_BASE);

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
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                log::warn!("API key validation failed: invalid or forbidden");
                Err(YouTubeError::InvalidApiKey)
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

        let url = format!("{}/videos", API_BASE);

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
            reqwest::StatusCode::BAD_REQUEST
            | reqwest::StatusCode::UNAUTHORIZED
            | reqwest::StatusCode::FORBIDDEN => {
                log::warn!(
                    "API key invalid or insufficient permissions: status {}",
                    response.status()
                );
                return Err(YouTubeError::InvalidApiKey);
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

        let url = format!("{}/liveChat/messages", API_BASE);

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
                let data: LiveChatMessagesResponse = response.json().await?;
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
}
