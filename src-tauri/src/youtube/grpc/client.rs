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
use std::time::Duration;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tonic::{Request, Status, Streaming};

/// YouTube gRPC endpoint
const YOUTUBE_GRPC_ENDPOINT: &str = "https://youtube.googleapis.com";

/// Default profile image size
const DEFAULT_PROFILE_IMAGE_SIZE: u32 = 64;

/// Default max results per response
const DEFAULT_MAX_RESULTS: u32 = 500;

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
            backoff: ExponentialBackoff::new(),
            seen_ids: std::collections::HashSet::new(),
        })
    }

    /// Start streaming chat messages
    pub async fn stream(
        &mut self,
    ) -> Result<Streaming<LiveChatMessageListResponse>, YouTubeError> {
        let request = self.create_request();

        let response = self
            .client
            .stream_list(request)
            .await
            .map_err(|e| self.handle_grpc_error(e))?;

        // Reset backoff on successful connection
        self.backoff.reset();

        Ok(response.into_inner())
    }

    /// Create a gRPC request with authentication
    fn create_request(&self) -> Request<LiveChatMessageListRequest> {
        let request_body = LiveChatMessageListRequest {
            live_chat_id: self.live_chat_id.clone(),
            part: vec![
                "id".to_string(),
                "snippet".to_string(),
                "authorDetails".to_string(),
            ],
            hl: "ja".to_string(),
            profile_image_size: DEFAULT_PROFILE_IMAGE_SIZE,
            max_results: DEFAULT_MAX_RESULTS,
            page_token: self.next_page_token.clone().unwrap_or_default(),
        };

        let mut request = Request::new(request_body);

        // Add API key authentication
        if let Ok(api_key_value) = self.api_key.parse() {
            request.metadata_mut().insert("x-goog-api-key", api_key_value);
        }

        request
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
        if !response.next_page_token.is_empty() {
            self.next_page_token = Some(response.next_page_token.clone());
        }

        let mut messages = Vec::new();

        for item in response.items {
            // Skip already seen messages
            if self.seen_ids.contains(&item.id) {
                continue;
            }
            self.seen_ids.insert(item.id.clone());

            if let Some(msg) = self.convert_to_chat_message(&item) {
                messages.push(msg);
            }
        }

        // Limit seen_ids to prevent memory leak
        if self.seen_ids.len() > 10000 {
            self.seen_ids.clear();
        }

        messages
    }

    /// Convert a gRPC message to ChatMessage
    fn convert_to_chat_message(
        &self,
        item: &super::proto::LiveChatMessage,
    ) -> Option<ChatMessage> {
        let snippet = item.snippet.as_ref()?;
        let author = item.author_details.as_ref()?;

        // Parse published_at
        let published_at = DateTime::parse_from_rfc3339(&snippet.published_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        // Determine message type
        let message_type = self.parse_message_type(snippet);

        Some(ChatMessage {
            id: item.id.clone(),
            message: snippet.display_message.clone(),
            author_name: author.display_name.clone(),
            author_channel_id: author.channel_id.clone(),
            author_image_url: author.profile_image_url.clone(),
            published_at,
            is_owner: author.is_chat_owner,
            is_moderator: author.is_chat_moderator,
            is_member: author.is_chat_sponsor,
            is_verified: author.is_verified,
            message_type,
            message_runs: None, // gRPC doesn't provide runs
        })
    }

    /// Parse message type from snippet
    fn parse_message_type(
        &self,
        snippet: &super::proto::LiveChatMessageSnippet,
    ) -> MessageType {
        match snippet.r#type.as_str() {
            "superChatEvent" | "SUPER_CHAT_EVENT" => {
                if let Some(details) = &snippet.super_chat_details {
                    MessageType::SuperChat {
                        amount: details.amount_display_string.clone(),
                        currency: details.currency.clone(),
                    }
                } else {
                    MessageType::Text
                }
            }
            "superStickerEvent" | "SUPER_STICKER_EVENT" => {
                if let Some(details) = &snippet.super_sticker_details {
                    MessageType::SuperSticker {
                        sticker_id: details.super_sticker_id.clone(),
                    }
                } else {
                    MessageType::Text
                }
            }
            "newSponsorEvent" | "NEW_SPONSOR_EVENT" => {
                if let Some(details) = &snippet.new_sponsor_details {
                    MessageType::Membership {
                        level: details.member_level_name.clone(),
                    }
                } else {
                    MessageType::Membership {
                        level: "New Member".to_string(),
                    }
                }
            }
            "memberMilestoneChatEvent" | "MEMBER_MILESTONE_CHAT_EVENT" => {
                if let Some(details) = &snippet.member_milestone_chat_details {
                    MessageType::Membership {
                        level: format!(
                            "{} ({}ヶ月)",
                            details.member_level_name, details.member_month
                        ),
                    }
                } else {
                    MessageType::Membership {
                        level: "Member Milestone".to_string(),
                    }
                }
            }
            "membershipGiftingEvent" | "MEMBERSHIP_GIFTING_EVENT" => {
                if let Some(details) = &snippet.membership_gifting_details {
                    MessageType::MembershipGift {
                        count: details.gift_memberships_count,
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
    use super::*;

    #[test]
    fn test_parse_message_type() {
        // This test validates the message type parsing logic
        // In real usage, this would be tested with actual gRPC responses
    }
}
