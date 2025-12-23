//! YouTube Live Chat gRPC Client Module
//!
//! This module provides a gRPC client for the YouTube Live Chat StreamList API.
//! It uses server-streaming to receive real-time chat messages.

pub mod client;
pub mod poller;

// Re-export the generated protobuf types
pub mod proto {
    tonic::include_proto!("youtube.api.v3");
}

pub use client::GrpcChatClient;
pub use poller::GrpcPoller;
