# Fire-and-forget ブロードキャストパターン

## 概要

RwLockガードをawait境界をまたいで保持しないため、WebSocketブロードキャストを`tokio::spawn`でバックグラウンド実行するパターン。

## 問題

### 解決した問題（RwLockガードのawait跨ぎ）

```rust
// NG: RwLockガードがawait境界を跨いでいる
async fn broadcast_update(server: &ServerState) {
    let ws_state = server.read().await;  // ガード取得
    let peers = ws_state.get_peers();
    for (id, tx) in peers {
        tx.send(message).await;  // ここでawait! デッドロックリスク
    }
}  // ガード解放
```

### 解決策（Fire-and-forget）

```rust
// OK: ガードのスコープを分離してtokio::spawnで非同期実行
async fn broadcast_update(server: &ServerState) {
    let server = Arc::clone(server);
    let message = WsMessage::Update { payload };
    tokio::spawn(async move {
        let peers_arc = {
            let ws_state = server.read().await;
            ws_state.get_peers_arc()  // Arc<RwLock<...>> を取得
        };  // ws_stateガード解放
        let peers_guard = peers_arc.read().await;
        let peers: Vec<_> = peers_guard
            .iter()
            .map(|(id, tx)| (*id, tx.clone()))
            .collect();
        drop(peers_guard);  // 明示的にガード解放
        WebSocketState::send_to_peers(&peers, &message);
    });
}
```

## 新たに発生する問題

### 1. レース条件

`tokio::spawn`で起動したタスクは呼び出し元と非同期で実行される。

```rust
// 呼び出し側
broadcast_update(&server).await;  // Ok(()) が返る
// しかし、この時点でブロードキャストは完了していない可能性がある

// すぐに状態取得すると古いデータが返る可能性
let state = get_current_state().await;  // レース条件!
```

**影響**:
- UI操作直後の状態取得で不整合
- 連続操作で更新が失われる可能性

### 2. タスク増大

連続呼び出し時に無制限にタスクが積み上がる。

```rust
// 高頻度呼び出し
for _ in 0..100 {
    broadcast_update(&server).await;  // 100個のタスクがspawnされる
}
```

**影響**:
- メモリ使用量増加
- タスクスケジューラの負荷増加
- 古いデータの送信が遅れる

## 対応策

### レース条件の対策

#### A) 用途別に使い分け

```rust
// 即時反映が必要な場合（ユーザー操作）
async fn broadcast_update_sync(server: &ServerState, message: &WsMessage) {
    // awaitで完了を待つ
    let peers_arc = {
        let ws_state = server.read().await;
        ws_state.get_peers_arc()
    };
    let peers_guard = peers_arc.read().await;
    let peers: Vec<_> = peers_guard.iter().map(|(id, tx)| (*id, tx.clone())).collect();
    drop(peers_guard);
    WebSocketState::send_to_peers(&peers, message);  // 同期的に完了
}

// 定期通知はfire-and-forget
fn broadcast_update_async(server: &ServerState, message: WsMessage) {
    let server = Arc::clone(server);
    tokio::spawn(async move { /* ... */ });
}
```

#### B) 戻り値で明示

```rust
enum BroadcastResult {
    Completed,    // 送信完了
    Pending,      // 非同期実行中
}
```

### タスク増大の対策

#### A) 送信中スキップ

```rust
static BROADCASTING: AtomicBool = AtomicBool::new(false);

fn broadcast_update_throttled(server: &ServerState, message: WsMessage) {
    // 前回送信中ならスキップ
    if BROADCASTING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        log::debug!("Broadcast skipped (previous in progress)");
        return;
    }

    let server = Arc::clone(server);
    tokio::spawn(async move {
        // ... ブロードキャスト処理 ...
        BROADCASTING.store(false, Ordering::SeqCst);
    });
}
```

#### B) 最新のみ保持（キュー化）

```rust
// 送信キューで最新の1つだけ保持
let (tx, rx) = tokio::sync::watch::channel(None);

// 送信側: 常に最新を設定
tx.send(Some(message)).ok();

// 受信側: 最新を取り出して送信
loop {
    let message = rx.borrow().clone();
    if let Some(msg) = message {
        send_to_peers(&msg);
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

## 適用ファイル（PR#118）

| ファイル | 関数 |
|----------|------|
| `commands/overlay.rs` | `broadcast_settings_update` |
| `commands/youtube.rs` | `broadcast_kpi_update`, `fetch_and_broadcast_viewer_count`, `fetch_viewer_count_innertube` |
| `commands/weather.rs` | `broadcast_weather_update`, `broadcast_weather`, `set_weather_city_and_broadcast`, `broadcast_weather_multi` |
| `weather/auto_updater.rs` | `fetch_and_broadcast_single`, `fetch_and_broadcast_multi` |

## 判断基準

| ユースケース | 推奨パターン |
|-------------|-------------|
| ユーザー操作の即時反映 | 同期送信（await） |
| 定期自動更新 | Fire-and-forget |
| 設定変更ブロードキャスト | Fire-and-forget（単一ユーザー前提） |
| 高頻度イベント | スロットリング付きfire-and-forget |

## 関連

- `issues/003_tauri-rust-patterns.md#8`: RwLockガードのawait境界
- `docs/900_tasks.md`: Fire-and-forgetブロードキャストのレース条件対策、tokio::spawnタスク増大の抑制
