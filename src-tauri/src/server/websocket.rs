use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use super::types::WsMessage;

type Tx = mpsc::UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<usize, Tx>>>;

/// WebSocket接続管理状態
pub struct WebSocketState {
    peers: PeerMap,
    next_peer_id: AtomicUsize,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            next_peer_id: AtomicUsize::new(0),
        }
    }

    /// 新しいピアIDを取得
    pub fn next_id(&self) -> usize {
        self.next_peer_id.fetch_add(1, Ordering::SeqCst)
    }

    /// ピアを追加
    pub async fn add_peer(&self, peer_id: usize, tx: Tx) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id, tx);
        log::info!("WebSocket peer {} connected. Total peers: {}", peer_id, peers.len());
    }

    /// ピアを削除
    pub async fn remove_peer(&self, peer_id: usize) {
        let mut peers = self.peers.write().await;
        peers.remove(&peer_id);
        log::info!("WebSocket peer {} disconnected. Total peers: {}", peer_id, peers.len());
    }

    /// 全ピアにメッセージをブロードキャスト
    pub async fn broadcast(&self, message: WsMessage) {
        let json = match serde_json::to_string(&message) {
            Ok(j) => j,
            Err(e) => {
                log::error!("Failed to serialize WebSocket message: {}", e);
                return;
            }
        };

        let peers = self.peers.read().await;
        let msg = Message::Text(json);

        for (peer_id, tx) in peers.iter() {
            if let Err(e) = tx.send(msg.clone()) {
                log::warn!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }

        log::debug!("Broadcasted message to {} peers: {:?}", peers.len(), message);
    }
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocketサーバーを起動
///
/// # 引数
/// - `state`: 共有状態
pub async fn start_websocket_server(state: Arc<RwLock<WebSocketState>>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:19801";
    let listener = TcpListener::bind(addr).await?;
    log::info!("WebSocket server listening on ws://{}/ws", addr);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        log::info!("New WebSocket connection from: {}", peer_addr);
        let state_clone = Arc::clone(&state);
        tokio::spawn(handle_connection(state_clone, stream));
    }

    Ok(())
}

/// WebSocket接続を処理
async fn handle_connection(state: Arc<RwLock<WebSocketState>>, stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    log::info!("WebSocket handshake completed");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // ピアIDを取得して登録
    let peer_id = state.read().await.next_id();
    state.read().await.add_peer(peer_id, tx).await;

    // 送信タスク: チャネルからメッセージを受信してWebSocketに送信
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 受信タスク: WebSocketからメッセージを受信（現在は特に処理なし）
    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(msg) => {
                    if msg.is_close() {
                        log::info!("Peer {} sent close frame", peer_id);
                        break;
                    }
                    // 今後、クライアントからのメッセージを処理する場合はここに追加
                    log::debug!("Received message from peer {}: {:?}", peer_id, msg);
                }
                Err(e) => {
                    log::warn!("WebSocket error from peer {}: {}", peer_id, e);
                    break;
                }
            }
        }
    });

    // どちらかのタスクが終了するまで待機
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // 接続終了時にピアを削除
    {
        let state_lock = state.read().await;
        state_lock.remove_peer(peer_id).await;
    }

    log::info!("WebSocket connection closed for peer {}", peer_id);
}
