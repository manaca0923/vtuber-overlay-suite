mod http;
pub mod types;
mod websocket;

pub use http::start_http_server;
pub use types::ServerState;
pub use websocket::start_websocket_server;

use std::sync::Arc;
use tokio::sync::RwLock;

/// サーバー用の共有状態を作成
pub fn create_server_state() -> ServerState {
    Arc::new(RwLock::new(websocket::WebSocketState::new()))
}
