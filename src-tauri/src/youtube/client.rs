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
        let url = format!(
            "{}/videos?part=id&id=dQw4w9WgXcQ&key={}",
            API_BASE, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(true),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                Err(YouTubeError::InvalidApiKey)
            }
            _ => Ok(false),
        }
    }

    /// 動画IDからactiveLiveChatIdを取得
    pub async fn get_live_chat_id(&self, video_id: &str) -> Result<String, YouTubeError> {
        let url = format!(
            "{}/videos?part=liveStreamingDetails&id={}&key={}",
            API_BASE, video_id, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(YouTubeError::VideoNotFound);
        }

        let data: VideoResponse = response.json().await?;

        data.items
            .first()
            .and_then(|item| item.live_streaming_details.as_ref())
            .and_then(|details| details.active_live_chat_id.clone())
            .ok_or(YouTubeError::LiveChatNotFound)
    }

    /// ライブチャットメッセージ取得
    pub async fn get_live_chat_messages(
        &self,
        live_chat_id: &str,
        page_token: Option<&str>,
    ) -> Result<LiveChatMessagesResponse, YouTubeError> {
        let mut url = format!(
            "{}/liveChat/messages?liveChatId={}&part=snippet,authorDetails&key={}",
            API_BASE, live_chat_id, self.api_key
        );

        if let Some(token) = page_token {
            url.push_str(&format!("&pageToken={}", token));
        }

        let response = self.client.get(&url).send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let data = response.json().await?;
                Ok(data)
            }
            reqwest::StatusCode::FORBIDDEN => {
                let error_text = response.text().await?;
                if error_text.contains("quotaExceeded") {
                    Err(YouTubeError::QuotaExceeded)
                } else if error_text.contains("rateLimitExceeded") {
                    Err(YouTubeError::RateLimitExceeded)
                } else {
                    Err(YouTubeError::InvalidApiKey)
                }
            }
            reqwest::StatusCode::NOT_FOUND => Err(YouTubeError::LiveChatNotFound),
            _ => Err(YouTubeError::ParseError(format!(
                "Unexpected status: {}",
                response.status()
            ))),
        }
    }
}
