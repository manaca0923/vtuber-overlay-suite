use thiserror::Error;

#[derive(Error, Debug)]
pub enum YouTubeError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API key is invalid or missing")]
    InvalidApiKey,

    #[error("Video not found or not a live stream")]
    VideoNotFound,

    #[error("Live chat not found or disabled")]
    LiveChatNotFound,

    #[error("Live chat is disabled for this video")]
    LiveChatDisabled,

    #[error("Invalid page token - resetting pagination")]
    InvalidPageToken,

    #[error("Quota exceeded - please try again tomorrow")]
    QuotaExceeded,

    #[error("Rate limit exceeded - retrying with backoff")]
    RateLimitExceeded,

    #[error("Poller is already running")]
    PollerAlreadyRunning,

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Request timeout: API応答がありません")]
    Timeout,

    #[error("InnerTube API not initialized - call initialize() first")]
    InnerTubeNotInitialized,

    #[error("InnerTube continuation expired - reinitializing required")]
    InnerTubeContinuationExpired,
}

impl From<YouTubeError> for String {
    fn from(err: YouTubeError) -> String {
        err.to_string()
    }
}
