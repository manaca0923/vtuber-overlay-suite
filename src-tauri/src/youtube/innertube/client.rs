//! InnerTube API クライアント実装

use regex::Regex;
use reqwest::Client;
use serde_json::json;
use std::sync::OnceLock;

use super::types::{ContinuationType, InnerTubeChatResponse, InnerTubePlayerResponse, VideoDetails};
use crate::youtube::errors::YouTubeError;

const INNERTUBE_API_URL: &str = "https://www.youtube.com/youtubei/v1/live_chat/get_live_chat";
const INNERTUBE_PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// InnerTubeクライアントバージョン（フォールバック用）
/// YouTube側で定期的に更新されるため、動的取得を優先する
/// このハードコーディング値は動的取得に失敗した場合のフォールバック
const FALLBACK_CLIENT_VERSION: &str = "2.20251201.01.00";

/// continuationトークン抽出用の最小長さ
/// 有効なcontinuationトークンは通常100文字以上のBase64エンコード文字列
/// 短いトークン（ページネーション以外の用途）を除外するための閾値
const MIN_CONTINUATION_LENGTH: usize = 50;

// 正規表現のシングルトン（OnceLockで初回のみコンパイル）
// invalidationContinuationData (ライブチャット用の優先度高いトークン)
static INVALIDATION_CONTINUATION_RE: OnceLock<Regex> = OnceLock::new();
// timedContinuationData (ライブチャット用のタイムアウト付きトークン)
static TIMED_CONTINUATION_RE: OnceLock<Regex> = OnceLock::new();
// reloadContinuationData (リロード用トークン)
static RELOAD_CONTINUATION_RE: OnceLock<Regex> = OnceLock::new();
// 汎用continuation (フォールバック用)
static GENERIC_CONTINUATION_RE: OnceLock<Regex> = OnceLock::new();
// API key用パターン (複数形式対応)
static API_KEY_RE_1: OnceLock<Regex> = OnceLock::new();
static API_KEY_RE_2: OnceLock<Regex> = OnceLock::new();
static API_KEY_RE_3: OnceLock<Regex> = OnceLock::new();
// CLIENT_VERSION用パターン
static CLIENT_VERSION_RE: OnceLock<Regex> = OnceLock::new();

/// invalidationContinuationData内のcontinuationを抽出（最優先）
/// ライブチャット専用のトークンを確実に取得
fn get_invalidation_continuation_regex() -> &'static Regex {
    INVALIDATION_CONTINUATION_RE.get_or_init(|| {
        Regex::new(r#""invalidationContinuationData"\s*:\s*\{[^}]*"continuation"\s*:\s*"([^"]+)""#)
            .expect("Failed to compile invalidationContinuationData regex")
    })
}

/// timedContinuationData内のcontinuationを抽出（優先度2）
fn get_timed_continuation_regex() -> &'static Regex {
    TIMED_CONTINUATION_RE.get_or_init(|| {
        Regex::new(r#""timedContinuationData"\s*:\s*\{[^}]*"continuation"\s*:\s*"([^"]+)""#)
            .expect("Failed to compile timedContinuationData regex")
    })
}

/// reloadContinuationData内のcontinuationを抽出（優先度3）
fn get_reload_continuation_regex() -> &'static Regex {
    RELOAD_CONTINUATION_RE.get_or_init(|| {
        Regex::new(r#""reloadContinuationData"\s*:\s*\{[^}]*"continuation"\s*:\s*"([^"]+)""#)
            .expect("Failed to compile reloadContinuationData regex")
    })
}

/// 汎用のcontinuation抽出（フォールバック、優先度最低）
fn get_generic_continuation_regex() -> &'static Regex {
    GENERIC_CONTINUATION_RE.get_or_init(|| {
        Regex::new(r#""continuation"\s*:\s*"([^"]+)""#)
            .expect("Failed to compile generic continuation regex")
    })
}

/// API key抽出パターン1: "INNERTUBE_API_KEY": "..."
fn get_api_key_regex_1() -> &'static Regex {
    API_KEY_RE_1.get_or_init(|| {
        Regex::new(r#""INNERTUBE_API_KEY"\s*:\s*"([^"]+)""#)
            .expect("Failed to compile API key regex 1")
    })
}

/// API key抽出パターン2: "innertubeApiKey": "..." (camelCase形式)
fn get_api_key_regex_2() -> &'static Regex {
    API_KEY_RE_2.get_or_init(|| {
        Regex::new(r#""innertubeApiKey"\s*:\s*"([^"]+)""#)
            .expect("Failed to compile API key regex 2")
    })
}

/// API key抽出パターン3: ytcfg.set({...INNERTUBE_API_KEY: "..."}) 形式
fn get_api_key_regex_3() -> &'static Regex {
    API_KEY_RE_3.get_or_init(|| {
        Regex::new(r#"INNERTUBE_API_KEY\s*[":]\s*"([^"]+)""#)
            .expect("Failed to compile API key regex 3")
    })
}

/// CLIENT_VERSION抽出パターン: "clientVersion":"X.YYYYMMDD.XX.XX"
/// メジャーバージョン変更にも対応できるよう、先頭を\d+にしている
fn get_client_version_regex() -> &'static Regex {
    CLIENT_VERSION_RE.get_or_init(|| {
        Regex::new(r#""clientVersion"\s*:\s*"(\d+\.\d{8}\.\d{2}\.\d{2})""#)
            .expect("Failed to compile client version regex")
    })
}

/// InnerTube APIクライアント
pub struct InnerTubeClient {
    client: Client,
    video_id: String,
    continuation: Option<String>,
    timeout_ms: u64,
    api_key: Option<String>,
    /// 動的に取得したクライアントバージョン（取得失敗時はFALLBACK_CLIENT_VERSIONを使用）
    client_version: String,
    /// 現在のContinuation種別（ポーリング間隔制御に使用）
    continuation_type: ContinuationType,
}

impl InnerTubeClient {
    /// 新しいクライアントを作成
    ///
    /// # Errors
    /// HTTPクライアントのビルドに失敗した場合にエラーを返す
    pub fn new(video_id: String) -> Result<Self, YouTubeError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| YouTubeError::NetworkError(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            video_id,
            continuation: None,
            timeout_ms: 5000,
            api_key: None,
            client_version: FALLBACK_CLIENT_VERSION.to_string(),
            continuation_type: ContinuationType::Invalidation, // 初期値（最も一般的）
        })
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
        self.continuation = Self::extract_continuation(&body);
        // INNERTUBE_API_KEYを抽出
        self.api_key = Self::extract_api_key(&body);
        // CLIENT_VERSIONを抽出（動的取得）
        if let Some(version) = Self::extract_client_version(&body) {
            log::info!("Dynamically extracted client version: {}", version);
            self.client_version = version;
        } else {
            log::warn!(
                "Failed to extract client version, using fallback: {}",
                FALLBACK_CLIENT_VERSION
            );
        }

        if self.continuation.is_some() {
            log::info!("InnerTube client initialized successfully");
            Ok(())
        } else {
            Err(YouTubeError::InnerTubeNotInitialized)
        }
    }

    /// continuationトークンを抽出（ライブチャット専用コンテキストを優先）
    ///
    /// 優先順位:
    /// 1. invalidationContinuationData - ライブチャット用のリアルタイム更新トークン
    /// 2. timedContinuationData - タイムアウト付きのライブチャットトークン
    /// 3. reloadContinuationData - リロード用トークン
    /// 4. 汎用continuation - フォールバック（他のcontinuationと混同するリスクあり）
    fn extract_continuation(html: &str) -> Option<String> {
        // 優先度1: invalidationContinuationData（ライブチャット専用、最も確実）
        let re1 = get_invalidation_continuation_regex();
        if let Some(caps) = re1.captures(html) {
            if let Some(cont) = caps.get(1) {
                let continuation = cont.as_str().to_string();
                if continuation.len() > MIN_CONTINUATION_LENGTH {
                    log::info!(
                        "Found invalidationContinuationData token (length: {}, priority: 1)",
                        continuation.len()
                    );
                    return Some(continuation);
                }
            }
        }

        // 優先度2: timedContinuationData（タイムアウト付きライブチャットトークン）
        let re2 = get_timed_continuation_regex();
        if let Some(caps) = re2.captures(html) {
            if let Some(cont) = caps.get(1) {
                let continuation = cont.as_str().to_string();
                if continuation.len() > MIN_CONTINUATION_LENGTH {
                    log::info!(
                        "Found timedContinuationData token (length: {}, priority: 2)",
                        continuation.len()
                    );
                    return Some(continuation);
                }
            }
        }

        // 優先度3: reloadContinuationData（リロード用）
        let re3 = get_reload_continuation_regex();
        if let Some(caps) = re3.captures(html) {
            if let Some(cont) = caps.get(1) {
                let continuation = cont.as_str().to_string();
                if continuation.len() > MIN_CONTINUATION_LENGTH {
                    log::info!(
                        "Found reloadContinuationData token (length: {}, priority: 3)",
                        continuation.len()
                    );
                    return Some(continuation);
                }
            }
        }

        // 優先度4: 汎用パターン（フォールバック、他のcontinuationと混同リスクあり）
        // 警告: このパターンは他の用途のcontinuationにマッチする可能性がある
        let re4 = get_generic_continuation_regex();
        if let Some(caps) = re4.captures(html) {
            if let Some(cont) = caps.get(1) {
                let continuation = cont.as_str().to_string();
                if continuation.len() > MIN_CONTINUATION_LENGTH {
                    log::warn!(
                        "Using generic continuation token (length: {}, priority: 4) - may not be live chat specific",
                        continuation.len()
                    );
                    return Some(continuation);
                }
            }
        }

        log::warn!("No valid continuation token found in page");
        None
    }

    /// INNERTUBE_API_KEYを抽出（複数パターンでフォールバック）
    ///
    /// 対応パターン:
    /// 1. "INNERTUBE_API_KEY": "..." (標準JSON形式)
    /// 2. "innertubeApiKey": "..." (camelCase形式)
    /// 3. INNERTUBE_API_KEY: "..." (ytcfg.set形式、クォートなしキー)
    fn extract_api_key(html: &str) -> Option<String> {
        // パターン1: "INNERTUBE_API_KEY": "..."
        let re1 = get_api_key_regex_1();
        if let Some(caps) = re1.captures(html) {
            if let Some(key) = caps.get(1) {
                let api_key = key.as_str().to_string();
                log::debug!("Found INNERTUBE_API_KEY (pattern 1)");
                return Some(api_key);
            }
        }

        // パターン2: "innertubeApiKey": "..."
        let re2 = get_api_key_regex_2();
        if let Some(caps) = re2.captures(html) {
            if let Some(key) = caps.get(1) {
                let api_key = key.as_str().to_string();
                log::debug!("Found innertubeApiKey (pattern 2)");
                return Some(api_key);
            }
        }

        // パターン3: INNERTUBE_API_KEY: "..." (ytcfg.set形式)
        let re3 = get_api_key_regex_3();
        if let Some(caps) = re3.captures(html) {
            if let Some(key) = caps.get(1) {
                let api_key = key.as_str().to_string();
                log::debug!("Found INNERTUBE_API_KEY (pattern 3, ytcfg format)");
                return Some(api_key);
            }
        }

        log::warn!("No INNERTUBE_API_KEY found - API calls may fail");
        None
    }

    /// CLIENT_VERSIONを抽出（動的取得）
    ///
    /// YouTubeのHTMLから現在のクライアントバージョンを抽出
    /// 形式: "clientVersion":"2.YYYYMMDD.XX.XX"
    fn extract_client_version(html: &str) -> Option<String> {
        let re = get_client_version_regex();
        if let Some(caps) = re.captures(html) {
            if let Some(version) = caps.get(1) {
                let version_str = version.as_str().to_string();
                log::debug!("Found clientVersion: {}", version_str);
                return Some(version_str);
            }
        }
        None
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
        if let Some((next_continuation, timeout_ms, cont_type)) = data.get_next_continuation() {
            self.continuation = Some(next_continuation);
            self.timeout_ms = timeout_ms;
            self.continuation_type = cont_type;
            log::debug!(
                "Updated continuation, next timeout: {}ms, type: {:?}",
                timeout_ms,
                cont_type
            );
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
                    "clientVersion": &self.client_version,
                    "hl": "ja",
                    "gl": "JP",
                    "timeZone": "Asia/Tokyo"
                }
            },
            "continuation": continuation
        })
    }

    /// 動画情報を取得（視聴者数を含む）
    ///
    /// InnerTube `/player` エンドポイントを使用して動画の詳細情報を取得。
    /// APIキー不要で視聴回数などの統計情報にアクセス可能。
    ///
    /// # Returns
    /// - `VideoDetails` - 動画の詳細情報（視聴回数、タイトル等）
    pub async fn get_video_details(&self) -> Result<VideoDetails, YouTubeError> {
        let request_body = self.build_player_request_body();

        // URLにAPI keyを追加（あれば）
        let url = if let Some(api_key) = &self.api_key {
            format!("{}?key={}", INNERTUBE_PLAYER_URL, api_key)
        } else {
            INNERTUBE_PLAYER_URL.to_string()
        };

        // 定期呼び出し（30秒ごと）のためtraceレベル
        log::trace!("Fetching video details from InnerTube Player API");

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
            log::error!("InnerTube Player API error: {} - {}", status, body);
            return Err(YouTubeError::ApiError(format!(
                "InnerTube Player API error: {}",
                status
            )));
        }

        let data: InnerTubePlayerResponse = response
            .json()
            .await
            .map_err(|e| YouTubeError::ParseError(format!("InnerTube Player parse error: {}", e)))?;

        data.video_details.ok_or_else(|| {
            YouTubeError::ParseError("video_details not found in response".to_string())
        })
    }

    /// Player APIリクエストボディを構築
    fn build_player_request_body(&self) -> serde_json::Value {
        json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": &self.client_version,
                    "hl": "ja",
                    "gl": "JP",
                    "timeZone": "Asia/Tokyo"
                }
            },
            "videoId": &self.video_id
        })
    }

    /// 次回ポーリングまでの待機時間（ミリ秒）
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// 現在のContinuation種別を取得
    ///
    /// ポーリング間隔の制御に使用:
    /// - `Invalidation`: 推奨間隔（短縮可能、1〜5秒にクランプ）
    /// - `Timed`: 明示的な待機時間（APIの値を厳守）
    /// - `Reload`: 初期化用（1秒固定）
    pub fn get_continuation_type(&self) -> ContinuationType {
        self.continuation_type
    }

    /// 初期化済みかどうか
    pub fn is_initialized(&self) -> bool {
        self.continuation.is_some()
    }

    /// continuationをリセット（再初期化が必要）
    pub fn reset(&mut self) {
        self.continuation = None;
        self.timeout_ms = 5000;
        self.continuation_type = ContinuationType::Invalidation;
    }
}

impl std::fmt::Debug for InnerTubeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerTubeClient")
            .field("video_id", &self.video_id)
            .field("initialized", &self.is_initialized())
            .field("timeout_ms", &self.timeout_ms)
            .field("continuation_type", &self.continuation_type)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_continuation() {
        // MIN_CONTINUATION_LENGTHより長いトークンが必要
        let long_token = "Cop4BxoYQ2dncl9jb21tb25fY2hhdF9tZXNzYWdlcw==".repeat(3);
        let html = format!(r#"{{"continuation":"{}"}}"#, long_token);
        let result = InnerTubeClient::extract_continuation(&html);
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_continuation_short_token_ignored() {
        // 短いトークンは無視される
        let html = r#"{"continuation":"shorttoken"}"#;
        let result = InnerTubeClient::extract_continuation(html);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_continuation_invalidation_priority() {
        // invalidationContinuationDataが最優先
        let long_token = "Cop4BxoYQ2dncl9jb21tb25fY2hhdF9tZXNzYWdlcw==".repeat(3);
        let html = format!(
            r#"{{"invalidationContinuationData":{{"continuation":"{}"}}}}"#,
            long_token
        );
        let result = InnerTubeClient::extract_continuation(&html);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), long_token);
    }

    #[test]
    fn test_extract_continuation_timed_priority() {
        // timedContinuationDataも抽出可能
        let long_token = "TimedContinuationToken123456789012345678901234567890".repeat(2);
        let html = format!(
            r#"{{"timedContinuationData":{{"timeoutMs":5000,"continuation":"{}"}}}}"#,
            long_token
        );
        let result = InnerTubeClient::extract_continuation(&html);
        assert!(result.is_some());
    }

    #[test]
    fn test_extract_continuation_reload() {
        // reloadContinuationDataからの抽出
        let long_token = "ReloadContinuationToken123456789012345678901234567890".repeat(2);
        let html = format!(
            r#"{{"reloadContinuationData":{{"continuation":"{}"}}}}"#,
            long_token
        );
        let result = InnerTubeClient::extract_continuation(&html);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), long_token);
    }

    #[test]
    fn test_extract_continuation_priority_order() {
        // 複数パターンがある場合、invalidationが優先される
        let invalidation_token = "InvalidationToken12345678901234567890123456789012345".repeat(2);
        let reload_token = "ReloadToken12345678901234567890123456789012345678901234".repeat(2);
        let html = format!(
            r#"{{"reloadContinuationData":{{"continuation":"{}"}},"invalidationContinuationData":{{"continuation":"{}"}}}}"#,
            reload_token, invalidation_token
        );
        let result = InnerTubeClient::extract_continuation(&html);
        assert!(result.is_some());
        // invalidationが優先されるべき
        assert_eq!(result.unwrap(), invalidation_token);
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

    #[test]
    fn test_extract_api_key_camel_case() {
        // camelCase形式もサポート
        let html = r#""innertubeApiKey":"AIzaSyBBCamelCase123456789""#;
        let result = InnerTubeClient::extract_api_key(html);
        assert_eq!(
            result,
            Some("AIzaSyBBCamelCase123456789".to_string())
        );
    }

    #[test]
    fn test_extract_api_key_ytcfg_format() {
        // ytcfg.set形式もサポート
        let html = r#"ytcfg.set({INNERTUBE_API_KEY: "AIzaSyYtcfgFormat"})"#;
        let result = InnerTubeClient::extract_api_key(html);
        assert_eq!(
            result,
            Some("AIzaSyYtcfgFormat".to_string())
        );
    }

    #[test]
    fn test_extract_client_version() {
        // 標準的なclientVersionの抽出
        let html = r#"{"clientVersion":"2.20251215.01.00"}"#;
        let result = InnerTubeClient::extract_client_version(html);
        assert_eq!(result, Some("2.20251215.01.00".to_string()));
    }

    #[test]
    fn test_extract_client_version_with_spaces() {
        // スペースを含む形式
        let html = r#"{"clientVersion" : "2.20251220.02.01"}"#;
        let result = InnerTubeClient::extract_client_version(html);
        assert_eq!(result, Some("2.20251220.02.01".to_string()));
    }

    #[test]
    fn test_extract_client_version_invalid_format() {
        // 無効な形式（バージョン形式に合わない）
        let html = r#"{"clientVersion":"invalid_version"}"#;
        let result = InnerTubeClient::extract_client_version(html);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_client_version_not_found() {
        // clientVersionが存在しない
        let html = r#"{"someOtherKey":"value"}"#;
        let result = InnerTubeClient::extract_client_version(html);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_client_version_multiple_occurrences() {
        // 複数のclientVersionがある場合、最初のものを取得
        let html = r#"{"clientVersion":"2.20251201.01.00","other":{"clientVersion":"2.20250101.00.00"}}"#;
        let result = InnerTubeClient::extract_client_version(html);
        assert_eq!(result, Some("2.20251201.01.00".to_string()));
    }

    #[test]
    fn test_extract_client_version_future_major_version() {
        // 将来のメジャーバージョン変更にも対応
        let html = r#"{"clientVersion":"3.20260101.00.00"}"#;
        let result = InnerTubeClient::extract_client_version(html);
        assert_eq!(result, Some("3.20260101.00.00".to_string()));
    }
}


