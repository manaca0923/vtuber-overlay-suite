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

// NOTE: GrpcChatClientはpoller内部でのみ使用されるが、
// 将来的に外部からの直接利用を可能にするためre-exportを維持
#[allow(unused_imports)]
pub use client::GrpcChatClient;
pub use poller::GrpcPoller;
