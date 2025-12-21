//! InnerTube API クライアント実装

use regex::Regex;
use reqwest::Client;
use serde_json::json;

use super::types::InnerTubeChatResponse;
use crate::youtube::errors::YouTubeError;

const INNERTUBE_API_URL: &str = "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// InnerTube APIクライアント
pub struct InnerTubeClient {
    client: Client,
    video_id: String,
    continuation: Option<String>,
    timeout_ms: u64,
    api_key: Option<String>,
}

impl InnerTubeClient {
    /// 新しいクライアントを作成
    pub fn new(video_id: String) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            video_id,
            continuation: None,
            timeout_ms: 5000,
            api_key: None,
        }
    }

    /// 初期化: ライブチャットページからcontinuationトークンを取得
    pub async fn initialize(&mut self) -> Result<(), YouTubeError> {
        let url = format!(
            "https://www.youtube.com/live_chat?is_popout=1&v={}",
            self.video_id
        );

        log::info!("Fetching live chat page: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| YouTubeError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(YouTubeError::ApiError(format!(
                "Failed to fetch live chat page: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| YouTubeError::NetworkError(e.to_string()))?;

        // ytInitialDataからcontinuationを抽出
        self.continuation = Self::extract_continuation(&body)?;
        // INNERTUBE_API_KEYを抽出
        self.api_key = Self::extract_api_key(&body);

        if self.continuation.is_some() {
            log::info!("InnerTube client initialized successfully");
            Ok(())
        } else {
            Err(YouTubeError::InnerTubeNotInitialized)
        }
    }

    /// continuationトークンを抽出
    fn extract_continuation(html: &str) -> Result<Option<String>, YouTubeError> {
        // ytInitialData内のcontinuationを正規表現で抽出
        // パターン: "continuation":"..." または "reloadContinuationData":{"continuation":"..."}

        // 方法1: invalidationContinuationData
        let re = Regex::new(r#""continuation"\s*:\s*"([^"]+)""#)
            .map_err(|e| YouTubeError::ParseError(e.to_string()))?;

        if let Some(caps) = re.captures(html) {
            if let Some(cont) = caps.get(1) {
                let continuation = cont.as_str().to_string();
                // 無効な短いトークンを除外
                if continuation.len() > 50 {
                    log::debug!("Found continuation token (length: {})", continuation.len());
                    return Ok(Some(continuation));
                }
            }
        }

        // 方法2: reloadContinuationData
        let re2 = Regex::new(r#""reloadContinuationData"\s*:\s*\{\s*"continuation"\s*:\s*"([^"]+)""#)
            .map_err(|e| YouTubeError::ParseError(e.to_string()))?;

        if let Some(caps) = re2.captures(html) {
            if let Some(cont) = caps.get(1) {
                let continuation = cont.as_str().to_string();
                if continuation.len() > 50 {
                    log::debug!(
                        "Found continuation token from reloadContinuationData (length: {})",
                        continuation.len()
                    );
                    return Ok(Some(continuation));
                }
            }
        }

        log::warn!("No valid continuation token found in page");
        Ok(None)
    }

    /// INNERTUBE_API_KEYを抽出
    fn extract_api_key(html: &str) -> Option<String> {
        let re = Regex::new(r#""INNERTUBE_API_KEY"\s*:\s*"([^"]+)""#).ok()?;
        re.captures(html)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// チャットメッセージを取得
    pub async fn get_chat_messages(&mut self) -> Result<InnerTubeChatResponse, YouTubeError> {
        let continuation = self
            .continuation
            .as_ref()
            .ok_or(YouTubeError::InnerTubeNotInitialized)?;

        let request_body = self.build_request_body(continuation);

        // URLにAPI keyを追加（あれば）
        let url = if let Some(api_key) = &self.api_key {
            format!("{}?key={}", INNERTUBE_API_URL, api_key)
        } else {
            INNERTUBE_API_URL.to_string()
        };

        log::debug!("Fetching chat messages from InnerTube API");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Origin", "https://www.youtube.com")
            .header("Referer", format!("https://www.youtube.com/watch?v={}", self.video_id))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| YouTubeError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("InnerTube API error: {} - {}", status, body);
            return Err(YouTubeError::ApiError(format!(
                "InnerTube API error: {}",
                status
            )));
        }

        let data: InnerTubeChatResponse = response
            .json()
            .await
            .map_err(|e| YouTubeError::ParseError(format!("InnerTube parse error: {}", e)))?;

        // 次回用のcontinuationを更新
        if let Some((next_continuation, timeout_ms)) = data.get_next_continuation() {
            self.continuation = Some(next_continuation);
            self.timeout_ms = timeout_ms;
            log::debug!("Updated continuation, next timeout: {}ms", timeout_ms);
        } else {
            log::warn!("No next continuation found in response");
        }

        Ok(data)
    }

    /// リクエストボディを構築
    fn build_request_body(&self, continuation: &str) -> serde_json::Value {
        json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20231219.01.00",
                    "hl": "ja",
                    "gl": "JP",
                    "timeZone": "Asia/Tokyo"
                }
            },
            "continuation": continuation
        })
    }

    /// 次回ポーリングまでの待機時間（ミリ秒）
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// 初期化済みかどうか
    pub fn is_initialized(&self) -> bool {
        self.continuation.is_some()
    }

    /// continuationをリセット（再初期化が必要）
    pub fn reset(&mut self) {
        self.continuation = None;
        self.timeout_ms = 5000;
    }
}

impl std::fmt::Debug for InnerTubeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerTubeClient")
            .field("video_id", &self.video_id)
            .field("initialized", &self.is_initialized())
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_continuation() {
        let html = r#"{"continuation":"Cop4BxoYQ2dncl9jb21tb25fY2hhdF9tZXNzYWdlcw=="}"#;
        let result = InnerTubeClient::extract_continuation(html).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_api_key() {
        let html = r#""INNERTUBE_API_KEY":"AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8""#;
        let result = InnerTubeClient::extract_api_key(html);
        assert_eq!(
            result,
            Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8".to_string())
        );
    }
}
