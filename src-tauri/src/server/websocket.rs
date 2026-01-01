use futures_util::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use super::types::{SetlistUpdatePayload, SongItem, SongStatus, WsMessage};
use crate::youtube::types::ChatMessage;

type Tx = mpsc::UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<usize, Tx>>>;

/// コメントキャッシュの最大数
const MAX_COMMENT_CACHE: usize = 50;

/// WebSocket接続管理状態
pub struct WebSocketState {
    peers: PeerMap,
    next_peer_id: AtomicUsize,
    /// コメントキャッシュ（新規接続時に送信）
    comment_cache: Arc<RwLock<VecDeque<ChatMessage>>>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            next_peer_id: AtomicUsize::new(0),
            comment_cache: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_COMMENT_CACHE))),
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

    /// キャッシュされたコメントを取得
    pub async fn get_cached_comments(&self) -> Vec<ChatMessage> {
        let cache = self.comment_cache.read().await;
        cache.iter().cloned().collect()
    }

    /// コメントをキャッシュに追加
    pub async fn add_to_cache(&self, comment: ChatMessage) {
        let mut cache = self.comment_cache.write().await;
        if cache.len() >= MAX_COMMENT_CACHE {
            cache.pop_front();
        }
        cache.push_back(comment);
    }

    /// 複数コメントをキャッシュに追加
    ///
    /// Note: 現在は未使用だが、バッチインポート機能で使用予定
    #[allow(dead_code)]
    pub async fn add_comments_to_cache(&self, comments: Vec<ChatMessage>) {
        let mut cache = self.comment_cache.write().await;
        for comment in comments {
            if cache.len() >= MAX_COMMENT_CACHE {
                cache.pop_front();
            }
            cache.push_back(comment);
        }
    }

    /// 全ピアにメッセージをブロードキャスト
    pub async fn broadcast(&self, message: WsMessage) {
        // コメントの場合はキャッシュに追加
        if let WsMessage::CommentAdd { ref payload, .. } = message {
            self.add_to_cache(payload.clone()).await;
        }

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

    /// ピアのクローンを取得（同期版・ガード保持時間を最小化するため）
    ///
    /// ## 使用例
    /// ```rust
    /// let peers = {
    ///     let ws_state = server.read().await;
    ///     ws_state.clone_peers()
    /// }; // ここでガード解放
    /// WebSocketState::send_to_peers(&peers, &message); // ガード解放後に送信
    /// ```
    pub fn clone_peers(&self) -> Vec<(usize, Tx)> {
        // 注意: この関数は同期的にピアをクローンする
        // 外側でRwLockガードを取得してからこの関数を呼ぶこと
        // peersフィールドへの直接アクセスが必要なため、try_read()を使用
        if let Ok(peers) = self.peers.try_read() {
            peers.iter().map(|(id, tx)| (*id, tx.clone())).collect()
        } else {
            // ロック取得に失敗した場合は空のリストを返す
            log::warn!("Failed to acquire peers lock for cloning");
            Vec::new()
        }
    }

    /// メッセージを直接送信（ガード不要版）
    ///
    /// ## 設計根拠
    /// `broadcast`メソッドは内部でRwLockガードを取得するため、
    /// 外側でガードを保持したまま呼ぶと二重ロックになる。
    /// このメソッドは事前に取得したピアリストに対して直接送信する。
    pub fn send_to_peers(peers: &[(usize, Tx)], message: &WsMessage) {
        let json = match serde_json::to_string(message) {
            Ok(j) => j,
            Err(e) => {
                log::error!("Failed to serialize WebSocket message: {}", e);
                return;
            }
        };

        let msg = Message::Text(json);
        for (peer_id, tx) in peers.iter() {
            if let Err(e) = tx.send(msg.clone()) {
                log::warn!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }

        log::debug!("Sent message to {} peers: {:?}", peers.len(), message);
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
/// - `db`: データベース接続プール
pub async fn start_websocket_server(
    state: Arc<RwLock<WebSocketState>>,
    db: SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:19801";
    let listener = TcpListener::bind(addr).await?;
    log::info!("WebSocket server listening on ws://{}/ws", addr);

    let db = Arc::new(db);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        log::info!("New WebSocket connection from: {}", peer_addr);
        let state_clone = Arc::clone(&state);
        let db_clone = Arc::clone(&db);
        tokio::spawn(handle_connection(state_clone, stream, db_clone));
    }

    Ok(())
}

/// WebSocket接続を処理
async fn handle_connection(
    state: Arc<RwLock<WebSocketState>>,
    stream: TcpStream,
    db: Arc<SqlitePool>,
) {
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

    // 先にセットリストとキャッシュされたコメントを取得
    let initial_setlist = fetch_latest_setlist_message(&db).await;
    let cached_comments = {
        let state_guard = state.read().await;
        state_guard.get_cached_comments().await
    };

    // ピアIDを取得して登録（1回のロック取得で処理）
    let peer_id = {
        let state_guard = state.read().await;
        let id = state_guard.next_id();
        state_guard.add_peer(id, tx.clone()).await;
        id
    };

    // 接続時に最新セットリストを送信
    if let Some(msg) = initial_setlist {
        if let Ok(json) = serde_json::to_string(&msg) {
            if tx.send(Message::Text(json)).is_err() {
                log::warn!("Failed to send initial setlist to peer {}", peer_id);
            } else {
                log::debug!("Sent initial setlist to peer {}", peer_id);
            }
        }
    }

    // 接続時にキャッシュされたコメントを送信
    // Note: キャッシュコメントは即時表示（instant: true）で送信し、
    // 接続直後のキャッチアップを素早く行う
    if !cached_comments.is_empty() {
        log::info!("Sending {} cached comments to peer {}", cached_comments.len(), peer_id);
        for comment in cached_comments {
            let msg = WsMessage::CommentAdd { payload: comment, instant: true, buffer_interval_ms: None };
            if let Ok(json) = serde_json::to_string(&msg) {
                if tx.send(Message::Text(json)).is_err() {
                    log::warn!("Failed to send cached comment to peer {}", peer_id);
                    break;
                }
            }
        }
    }

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

/// 最新セットリストを取得してWsMessageを生成
async fn fetch_latest_setlist_message(pool: &SqlitePool) -> Option<WsMessage> {
    // 最新のセットリストIDを取得
    let setlist_result = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM setlists ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await;

    let setlist_id = match setlist_result {
        Ok(Some(row)) => row.0,
        Ok(None) => {
            log::debug!("No setlists found for initial WebSocket message");
            return None;
        }
        Err(e) => {
            log::error!("Failed to fetch setlist for initial message: {}", e);
            return None;
        }
    };

    // 楽曲リスト取得
    let songs_result = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>)>(
        r#"
        SELECT
            ss.id, s.title, s.artist, ss.started_at, ss.ended_at
        FROM setlist_songs ss
        JOIN songs s ON ss.song_id = s.id
        WHERE ss.setlist_id = ?
        ORDER BY ss.position
        "#,
    )
    .bind(&setlist_id)
    .fetch_all(pool)
    .await;

    let rows = match songs_result {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Failed to fetch songs for initial message: {}", e);
            return None;
        }
    };

    // 現在の曲のインデックスを計算（started_atがあり、ended_atがないものが現在の曲）
    let current_index = rows
        .iter()
        .position(|row| row.3.is_some() && row.4.is_none())
        .map(|i| i as i32)
        .unwrap_or(-1);

    // 曲リストを構築
    let songs: Vec<SongItem> = rows
        .into_iter()
        .enumerate()
        .map(|(idx, row)| {
            let status = if current_index == -1 {
                SongStatus::Pending
            } else if (idx as i32) < current_index {
                SongStatus::Done
            } else if (idx as i32) == current_index {
                SongStatus::Current
            } else {
                SongStatus::Pending
            };

            SongItem {
                id: row.0,
                title: row.1,
                artist: row.2.unwrap_or_default(),
                status,
            }
        })
        .collect();

    let payload = SetlistUpdatePayload {
        setlist_id,
        current_index,
        songs,
    };

    log::debug!("Generated initial setlist message with {} songs", payload.songs.len());
    Some(WsMessage::SetlistUpdate { payload })
}
