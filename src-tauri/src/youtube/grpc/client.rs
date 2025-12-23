//! YouTube Live Chat gRPC Client
//!
//! Provides a streaming client for YouTube Live Chat using gRPC.

use super::proto::{
    v3_data_live_chat_message_service_client::V3DataLiveChatMessageServiceClient,
    LiveChatMessageListRequest, LiveChatMessageListResponse,
};
use crate::youtube::backoff::ExponentialBackoff;
use crate::youtube::errors::YouTubeError;
use crate::youtube::types::{ChatMessage, MessageType};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::time::Duration;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tonic::{Request, Status, Streaming};

/// YouTube gRPC endpoint
const YOUTUBE_GRPC_ENDPOINT: &str = "https://youtube.googleapis.com";

/// Default profile image size
const DEFAULT_PROFILE_IMAGE_SIZE: u32 = 64;

/// Default max results per response
const DEFAULT_MAX_RESULTS: u32 = 500;

/// Maximum number of seen message IDs to keep for deduplication
const MAX_SEEN_IDS: usize = 10000;

/// gRPC client for YouTube Live Chat
pub struct GrpcChatClient {
    /// gRPC client instance
    client: V3DataLiveChatMessageServiceClient<Channel>,
    /// API key for authentication
    api_key: String,
    /// Live chat ID
    live_chat_id: String,
    /// Next page token for pagination
    next_page_token: Option<String>,
    /// Backoff strategy for reconnection
    backoff: ExponentialBackoff,
    /// Seen message IDs for deduplication
    seen_ids: std::collections::HashSet<String>,
    /// Order of seen message IDs for FIFO eviction
    seen_order: VecDeque<String>,
}

impl GrpcChatClient {
    /// Create a new gRPC client and connect to YouTube
    pub async fn connect(api_key: String, live_chat_id: String) -> Result<Self, YouTubeError> {
        log::info!("Connecting to YouTube gRPC endpoint: {}", YOUTUBE_GRPC_ENDPOINT);

        // Configure TLS
        let tls_config = ClientTlsConfig::new().with_native_roots();

        // Create endpoint with TLS
        let endpoint = Endpoint::from_static(YOUTUBE_GRPC_ENDPOINT)
            .tls_config(tls_config)
            .map_err(|e| YouTubeError::NetworkError(format!("TLS config error: {}", e)))?
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        // Connect
        let channel = endpoint
            .connect()
            .await
            .map_err(|e| YouTubeError::NetworkError(format!("Connection failed: {}", e)))?;

        let client = V3DataLiveChatMessageServiceClient::new(channel);

        log::info!("Connected to YouTube gRPC endpoint");

        Ok(Self {
            client,
            api_key,
            live_chat_id,
            next_page_token: None,
            backoff: ExponentialBackoff::with_jitter(),
            seen_ids: std::collections::HashSet::new(),
            seen_order: VecDeque::new(),
        })
    }

    /// Start streaming chat messages
    pub async fn stream(
        &mut self,
    ) -> Result<Streaming<LiveChatMessageListResponse>, YouTubeError> {
        let request = self.create_request()?;

        log::info!(
            "Starting gRPC stream for live_chat_id: {}",
            self.live_chat_id
        );

        let response = self
            .client
            .stream_list(request)
            .await
            .map_err(|e| {
                log::error!("gRPC StreamList call failed: code={}, message={}", e.code(), e.message());
                self.handle_grpc_error(e)
            })?;

        // Reset backoff on successful connection
        self.backoff.reset();

        log::info!("gRPC stream started successfully");

        Ok(response.into_inner())
    }

    /// Create a gRPC request with authentication
    fn create_request(&self) -> Result<Request<LiveChatMessageListRequest>, YouTubeError> {
        let request_body = LiveChatMessageListRequest {
            live_chat_id: Some(self.live_chat_id.clone()),
            part: vec![
                "id".to_string(),
                "snippet".to_string(),
                "authorDetails".to_string(),
            ],
            hl: Some("ja".to_string()),
            profile_image_size: Some(DEFAULT_PROFILE_IMAGE_SIZE),
            max_results: Some(DEFAULT_MAX_RESULTS),
            page_token: self.next_page_token.clone(),
        };

        log::debug!(
            "Creating gRPC request: live_chat_id={}, page_token={:?}",
            self.live_chat_id,
            self.next_page_token
        );

        let mut request = Request::new(request_body);

        // Add API key authentication
        let api_key_value: tonic::metadata::AsciiMetadataValue = self
            .api_key
            .parse()
            .map_err(|e| {
                log::error!("Failed to parse API key as metadata value: {:?}", e);
                YouTubeError::InvalidApiKey
            })?;

        request.metadata_mut().insert("x-goog-api-key", api_key_value);
        log::debug!("API key metadata set successfully");

        Ok(request)
    }

    /// Handle gRPC errors and convert to YouTubeError
    fn handle_grpc_error(&self, status: Status) -> YouTubeError {
        match status.code() {
            tonic::Code::Unauthenticated => {
                log::error!("gRPC authentication failed: {}", status.message());
                YouTubeError::InvalidApiKey
            }
            tonic::Code::NotFound => {
                log::error!("Live chat not found: {}", status.message());
                YouTubeError::LiveChatNotFound
            }
            tonic::Code::ResourceExhausted => {
                log::warn!("Rate limit exceeded: {}", status.message());
                YouTubeError::RateLimitExceeded
            }
            tonic::Code::PermissionDenied => {
                // Could be quota exceeded or chat disabled
                let message = status.message().to_lowercase();
                if message.contains("quota") {
                    log::error!("Quota exceeded: {}", status.message());
                    YouTubeError::QuotaExceeded
                } else if message.contains("disabled") {
                    log::error!("Live chat disabled: {}", status.message());
                    YouTubeError::LiveChatDisabled
                } else {
                    log::error!("Permission denied: {}", status.message());
                    YouTubeError::ApiError(status.message().to_string())
                }
            }
            tonic::Code::Unavailable => {
                log::warn!("Service unavailable: {}", status.message());
                YouTubeError::NetworkError(format!("Service unavailable: {}", status.message()))
            }
            _ => {
                log::error!("gRPC error: {} - {}", status.code(), status.message());
                YouTubeError::ApiError(format!("{}: {}", status.code(), status.message()))
            }
        }
    }

    /// Parse a gRPC response into ChatMessage list
    pub fn parse_response(&mut self, response: LiveChatMessageListResponse) -> Vec<ChatMessage> {
        // Update next page token
        if let Some(ref token) = response.next_page_token {
            if !token.is_empty() {
                self.next_page_token = Some(token.clone());
            }
        }

        let mut messages = Vec::new();

        for item in response.items {
            // Get message ID (skip if not present)
            let msg_id = match &item.id {
                Some(id) => id.clone(),
                None => continue,
            };

            // Skip already seen messages
            if self.seen_ids.contains(&msg_id) {
                continue;
            }

            // Add to seen_ids and maintain order for FIFO eviction
            if self.seen_ids.insert(msg_id.clone()) {
                self.seen_order.push_back(msg_id.clone());
            }

            if let Some(msg) = self.convert_to_chat_message(&item, &msg_id) {
                messages.push(msg);
            }
        }

        // FIFO eviction: remove oldest IDs when limit exceeded
        // This prevents memory leak while avoiding duplicate messages after eviction
        while self.seen_ids.len() > MAX_SEEN_IDS {
            if let Some(oldest_id) = self.seen_order.pop_front() {
                self.seen_ids.remove(&oldest_id);
            } else {
                break;
            }
        }

        messages
    }

    /// Convert a gRPC message to ChatMessage
    fn convert_to_chat_message(
        &self,
        item: &super::proto::LiveChatMessage,
        msg_id: &str,
    ) -> Option<ChatMessage> {
        let snippet = item.snippet.as_ref()?;
        let author = item.author_details.as_ref()?;

        // Parse published_at
        let published_at = snippet
            .published_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Determine message type
        let message_type = self.parse_message_type(snippet);

        Some(ChatMessage {
            id: msg_id.to_string(),
            message: snippet.display_message.clone().unwrap_or_default(),
            author_name: author.display_name.clone().unwrap_or_default(),
            author_channel_id: author.channel_id.clone().unwrap_or_default(),
            author_image_url: author.profile_image_url.clone().unwrap_or_default(),
            published_at,
            is_owner: author.is_chat_owner.unwrap_or(false),
            is_moderator: author.is_chat_moderator.unwrap_or(false),
            is_member: author.is_chat_sponsor.unwrap_or(false),
            is_verified: author.is_verified.unwrap_or(false),
            message_type,
            message_runs: None, // gRPC doesn't provide runs
        })
    }

    /// Parse message type from snippet
    fn parse_message_type(
        &self,
        snippet: &super::proto::LiveChatMessageSnippet,
    ) -> MessageType {
        use super::proto::live_chat_message_snippet::type_wrapper::Type;

        // Convert i32 to Type enum
        let msg_type = snippet
            .r#type
            .and_then(|v| Type::try_from(v).ok());

        match msg_type {
            Some(Type::SuperChatEvent) => {
                if let Some(details) = &snippet.super_chat_details {
                    MessageType::SuperChat {
                        amount: details.amount_display_string.clone().unwrap_or_default(),
                        currency: details.currency.clone().unwrap_or_default(),
                    }
                } else {
                    MessageType::Text
                }
            }
            Some(Type::SuperStickerEvent) => {
                if let Some(details) = &snippet.super_sticker_details {
                    // Get sticker ID from metadata
                    let sticker_id = details
                        .super_sticker_metadata
                        .as_ref()
                        .and_then(|m| m.sticker_id.clone())
                        .unwrap_or_default();
                    MessageType::SuperSticker { sticker_id }
                } else {
                    MessageType::Text
                }
            }
            Some(Type::NewSponsorEvent) => {
                if let Some(details) = &snippet.new_sponsor_details {
                    MessageType::Membership {
                        level: details.member_level_name.clone().unwrap_or_default(),
                    }
                } else {
                    MessageType::Membership {
                        level: "New Member".to_string(),
                    }
                }
            }
            Some(Type::MemberMilestoneChatEvent) => {
                if let Some(details) = &snippet.member_milestone_chat_details {
                    MessageType::Membership {
                        level: format!(
                            "{} ({}ヶ月)",
                            details.member_level_name.as_deref().unwrap_or("Member"),
                            details.member_month.unwrap_or(0)
                        ),
                    }
                } else {
                    MessageType::Membership {
                        level: "Member Milestone".to_string(),
                    }
                }
            }
            Some(Type::MembershipGiftingEvent) => {
                if let Some(details) = &snippet.membership_gifting_details {
                    MessageType::MembershipGift {
                        count: details.gift_memberships_count.unwrap_or(1) as u32,
                    }
                } else {
                    MessageType::MembershipGift { count: 1 }
                }
            }
            _ => MessageType::Text,
        }
    }

    /// Get backoff delay for reconnection
    pub fn get_backoff_delay(&mut self) -> Duration {
        self.backoff.next_delay()
    }

    /// Reset backoff
    pub fn reset_backoff(&mut self) {
        self.backoff.reset();
    }

    /// Get the next page token
    pub fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    /// Set the next page token (for resuming)
    pub fn set_next_page_token(&mut self, token: Option<String>) {
        self.next_page_token = token;
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[ignore = "TODO: Phase 6でgRPCレスポンスのモックを使ったテストを実装"]
    fn test_parse_message_type() {
        // This test validates the message type parsing logic
        // In real usage, this would be tested with actual gRPC responses
    }
}
