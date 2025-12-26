# PR #56 Codexレビュー指摘事項

## 概要
PR #56「高優先度パフォーマンス最適化（3項目）」のCodexレビューで受けた指摘と対応をまとめる。

## 指摘事項と対応

### 1. エラー判定の早期リターン問題（db.rs）

**指摘内容**:
```rust
// Before（問題あり）:
fn is_sqlite_busy_error(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        if let Some(code) = db_err.code() {
            return code.as_ref() == "5";  // ← "5"以外は即return false
        }
        // ↓ コードがない場合のみメッセージ判定に到達
        let msg = db_err.message().to_lowercase();
        return msg.contains("database is locked") || msg.contains("busy");
    }
    false
}
```

- エラーコードが存在し`"5"`でない場合、`return false`で終了
- `SQLITE_LOCKED`(6)、`SQLITE_BUSY_TIMEOUT`などの変種が検出されない
- メッセージによる判定もスキップされる

**対応**:
```rust
// After（修正済み）:
fn is_sqlite_busy_error(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        // エラーコードで判定（優先）
        if let Some(code) = db_err.code() {
            let code = code.as_ref();
            // SQLite拡張エラーコードは (extended_code % 256) で基本コードを取得
            // 5 = SQLITE_BUSY, 6 = SQLITE_LOCKED
            if code == "5" || code == "6" {
                return true;
            }
            // 文字列として"SQLITE_BUSY"/"SQLITE_LOCKED"が返される場合も対応
            let code_upper = code.to_uppercase();
            if code_upper.starts_with("SQLITE_BUSY") || code_upper.starts_with("SQLITE_LOCKED") {
                return true;
            }
        }
        // エラーメッセージでも判定（フォールバック）
        // コードが取得できない場合やコードが未知の場合に対応
        let msg = db_err.message().to_lowercase();
        if msg.contains("database is locked") || msg.contains("busy") || msg.contains("locked") {
            return true;
        }
    }
    false
}
```

**今後の対策**:
- 複数条件で判定する場合、早期リターンがフォールバック判定をスキップしないか確認
- 条件分岐は「true条件を列挙 → 最後にfalse」の形式が望ましい
- 拡張エラーコード（SQLITE_BUSY_TIMEOUTなど）も考慮する

### 2. リトライ回数と試行回数の命名混同

**指摘内容**:
```rust
// Before（誤解を招く）:
/// SQLITE_BUSYエラー時の最大リトライ回数
const MAX_RETRY_ATTEMPTS: u32 = 3;

// ログメッセージ:
log::warn!("SQLITE_BUSY: Max retries ({}) exceeded", MAX_RETRY_ATTEMPTS);
```

- `MAX_RETRY_ATTEMPTS = 3` は「リトライ3回」と読める
- 実際の動作は「初回 + 2リトライ = 3回試行」
- 3回のbackoff delay（100ms/200ms/400ms）を期待すると齟齬が生じる

**対応**:
```rust
// After（明確な命名）:
/// SQLITE_BUSYエラー時の最大試行回数（初回 + リトライ）
/// 3回 = 初回試行 + 2回リトライ（100ms, 200ms後）
const MAX_ATTEMPTS: u32 = 3;

// ドキュメント追加:
/// 例: MAX_ATTEMPTS=3の場合
///   1回目: 即時試行
///   2回目: 100ms後にリトライ
///   3回目: 200ms後にリトライ
///   → 失敗時はフォールバック（個別INSERT）
```

**今後の対策**:
- リトライ回数（retry）と試行回数（attempt）を区別する
  - リトライ回数: 失敗後の再試行回数
  - 試行回数: 初回を含む総試行回数
- 定数名はロジックの意味と一致させる
- コメントで具体例を示す

### 3. タイマーリソースのクリーンアップ（destroy()）

**指摘内容**:
- `DensityManager`にsetIntervalでタイマーを設定
- `destroy()`メソッドを実装したが、呼び出し箇所がない
- オーバーレイがアンロードされた場合にタイマーがリークする可能性

**対応**:
- `docs/900_tasks.md`に将来タスクとして追記
- OBSブラウザソースはアンロードされることがほとんどないため、優先度低

**今後の対策**:
- `setInterval`/`setTimeout`を使用する場合:
  1. 必ず`destroy()`または`cleanup()`メソッドを実装
  2. 呼び出し箇所（`beforeunload`イベントなど）を確保
  3. クラスのdocコメントにライフサイクル管理の説明を追加
- リソース管理のパターン:
  ```javascript
  class Manager {
    constructor() {
      this._timerId = setInterval(...);
    }

    destroy() {
      if (this._timerId) {
        clearInterval(this._timerId);
        this._timerId = null;
      }
    }
  }

  // 使用側
  window.addEventListener('beforeunload', () => manager.destroy());
  ```

### 4. テストカバレッジの追加

**指摘内容**:
- SQLITE_LOCKED（コード6）など拡張コードのテストがない
- エラー判定ロジックの変更時にリグレッションを防げない

**対応**:
```rust
// MockDatabaseErrorを使用したユニットテスト追加

#[test]
fn test_is_sqlite_busy_error_code_5() {
    let err = create_mock_db_error(Some("5"), "database is busy");
    assert!(is_sqlite_busy_error(&err));
}

#[test]
fn test_is_sqlite_busy_error_code_6() {
    let err = create_mock_db_error(Some("6"), "database table is locked");
    assert!(is_sqlite_busy_error(&err));
}

#[test]
fn test_is_sqlite_busy_error_string_code() {
    // SQLITE_BUSY, SQLITE_BUSY_TIMEOUT, SQLITE_LOCKED
    let err = create_mock_db_error(Some("SQLITE_BUSY_TIMEOUT"), "timeout");
    assert!(is_sqlite_busy_error(&err));
}

#[test]
fn test_is_sqlite_busy_error_message_fallback() {
    // コードがない場合のメッセージ判定
    let err = create_mock_db_error(None, "database is locked");
    assert!(is_sqlite_busy_error(&err));
}
```

**今後の対策**:
- エラー判定ロジックにはユニットテストを追加
- モックエラーを使用して各条件分岐をカバー
- 新しいエラーコードを追加する場合はテストも追加

### 5. 数値拡張エラーコードの % 256 判定（追加指摘）

**指摘内容**:
- 数値エラーコード（517, 261等）を文字列比較でしか判定していない
- 拡張エラーコードは `extended_code % 256` で基本コードを取得する必要がある
- 例: 517 (SQLITE_BUSY_SNAPSHOT) は 517 % 256 = 5 (SQLITE_BUSY)

**対応**:
```rust
fn is_sqlite_busy_error(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        if let Some(code) = db_err.code() {
            let code_str = code.as_ref();

            // 数値コードの場合: パースして % 256 で基本コードを取得
            if let Ok(code_num) = code_str.parse::<i32>() {
                let base_code = code_num % 256;
                if base_code == 5 || base_code == 6 {
                    return true;
                }
            }
            // ...文字列判定...
        }
        // ...メッセージフォールバック...
    }
    false
}
```

**テスト追加**:
```rust
#[test]
fn test_is_sqlite_busy_error_extended_codes() {
    // 517 = SQLITE_BUSY_SNAPSHOT (517 % 256 = 5)
    let err = create_mock_db_error(Some("517"), "snapshot busy");
    assert!(is_sqlite_busy_error(&err));

    // 261 = SQLITE_BUSY_RECOVERY (261 % 256 = 5)
    let err = create_mock_db_error(Some("261"), "recovery busy");
    assert!(is_sqlite_busy_error(&err));

    // 262 = SQLITE_LOCKED_SHAREDCACHE (262 % 256 = 6)
    let err = create_mock_db_error(Some("262"), "shared cache locked");
    assert!(is_sqlite_busy_error(&err));
}
```

**今後の対策**:
- SQLite拡張エラーコードは常に `% 256` で基本コードを取得して判定
- 主要な拡張コード:
  - 261 = SQLITE_BUSY_RECOVERY
  - 517 = SQLITE_BUSY_SNAPSHOT
  - 262 = SQLITE_LOCKED_SHAREDCACHE

### 6. SqliteConnectOptions::new().filename()の使用（追加指摘）

**指摘内容**:
- `SqliteConnectOptions::from_str()` はWindows/特殊パスで問題が発生する可能性
- `sqlite:C:\path\to\file.db` のようなパスが正しくパースされない場合がある

**対応**:
```rust
// Before（問題あり）:
let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}?mode=rwc", path))
    .unwrap()
    .busy_timeout(Duration::from_millis(50));

// After（修正済み）:
let connect_options = SqliteConnectOptions::new()
    .filename(path)
    .create_if_missing(true)
    .busy_timeout(Duration::from_millis(50));
```

**今後の対策**:
- SQLite接続オプションは `SqliteConnectOptions::new().filename()` を使用
- `from_str()` はURI形式のパース問題があるため避ける
- `create_if_missing(true)` で `?mode=rwc` と同等の動作

### 7. combined-v2.htmlでdestroy()呼び出し追加（追加指摘 → 対応済み）

**指摘内容**:
- `densityManager.destroy()` が呼び出されていない
- オーバーレイのリロード時にタイマーがリークし、多重dispatchの可能性

**対応**:
```javascript
// combined-v2.htmlに追加
function cleanup() {
  if (densityManager) {
    densityManager.destroy();
  }
  if (updateBatcher) {
    updateBatcher.destroy();
  }
  if (ws) {
    ws.close();
  }
}

window.addEventListener('pagehide', cleanup);
window.addEventListener('beforeunload', cleanup);
```

**UpdateBatcherにも`destroy()`追加**:
```javascript
destroy() {
  this.clear();
}
```

**今後の対策**:
- setInterval/setTimeoutを使用するクラスには必ず `destroy()` メソッドを実装
- 使用側で `pagehide`/`beforeunload` イベントで `destroy()` を呼び出す
- `pagehide` が推奨（bfcache対応）、`beforeunload` はフォールバック

### 8. WebSocket再接続がcleanup時に発生する問題（追加指摘）

**指摘内容**:
- `cleanup()`で`ws.close()`を呼び出すが、`ws.onclose`が常に再接続をスケジュール
- `pagehide`/`beforeunload`時に新しいソケットが作成される可能性
- 両イベントで二重に再接続がスケジュールされる可能性

**対応**:
```javascript
let reconnectTimerId = null;
let isShuttingDown = false;

ws.onclose = () => {
  // シャットダウン中は再接続しない
  if (isShuttingDown) {
    console.log('WebSocket closed (shutdown)');
    return;
  }
  console.log('WebSocket closed, reconnecting...');
  reconnectTimerId = setTimeout(() => {
    reconnectTimerId = null;
    reconnectDelay = Math.min(reconnectDelay * 2, 30000);
    connectWebSocket();
  }, reconnectDelay);
};

function cleanup() {
  // シャットダウンフラグを立てて再接続を防止
  isShuttingDown = true;

  // 再接続タイマーをクリア
  if (reconnectTimerId) {
    clearTimeout(reconnectTimerId);
    reconnectTimerId = null;
  }

  // WebSocketを閉じる（oncloseハンドラを無効化してから）
  if (ws) {
    ws.onclose = null;
    ws.close();
  }
  // ...その他のクリーンアップ
}
```

**今後の対策**:
- WebSocket再接続ロジックにはシャットダウンフラグを設ける
- `onclose`ハンドラを無効化してからclose()を呼び出す
- 再接続タイマーIDを保持し、cleanup時にclearTimeout()

### 9. リトライ処理の総タイムアウト追加（追加指摘）

**指摘内容**:
- `busy_timeout`(5秒) × リトライ回数(3回) = 最大15秒のブロック
- チャット取り込みが長時間停滞する可能性

**対応**:
```rust
/// リトライ処理の総タイムアウト（ミリ秒）
const RETRY_TOTAL_TIMEOUT_MS: u64 = 2000;

async fn save_chunk_with_retry(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    let start_time = Instant::now();
    let timeout = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);
    // ...

    loop {
        match save_chunk_with_transaction(pool, messages).await {
            // ...
            TransactionResult::Busy => {
                // 総タイムアウトチェック
                let elapsed = start_time.elapsed();
                if elapsed >= timeout {
                    log::warn!(
                        "SQLITE_BUSY: Total timeout ({}ms) exceeded after {} attempts, giving up",
                        RETRY_TOTAL_TIMEOUT_MS,
                        attempt
                    );
                    return false;
                }
                // ...
            }
        }
    }
}
```

**今後の対策**:
- リトライ処理には総タイムアウトを設ける
- `busy_timeout`とリトライの組み合わせで最悪ケースを計算
- チャットのようなリアルタイム処理では短いタイムアウト（2秒程度）を使用

### 10. tokio::time::timeoutで各試行をラップ（追加指摘）

**指摘内容**:
- `RETRY_TOTAL_TIMEOUT_MS`が実際には強制されていない
- 各`save_chunk_with_transaction`がプールの`busy_timeout`(5秒)でブロックする
- 最悪ケース: 5秒×3回 = 15秒のブロック

**対応**:
```rust
use tokio::time::timeout;

async fn save_chunk_with_retry(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    let start_time = Instant::now();
    let total_timeout = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);

    loop {
        // 残り時間を計算
        let elapsed = start_time.elapsed();
        if elapsed >= total_timeout {
            return false;
        }
        let remaining = total_timeout - elapsed;

        // 各試行をtokio::time::timeoutでラップ
        let result = timeout(remaining, save_chunk_with_transaction(pool, messages)).await;

        match result {
            Ok(TransactionResult::Success) => return true,
            Ok(TransactionResult::OtherError) => return false,
            Ok(TransactionResult::Busy) | Err(_) => {
                // Busyまたはタイムアウト → リトライ可能
                // ...
            }
        }
    }
}
```

**今後の対策**:
- リトライ処理には`tokio::time::timeout(remaining, ...)`で各試行をラップ
- タイムアウトはBusyと同様に扱い、リトライ可能とする
- 残り時間を毎回計算して、総タイムアウトを超えないようにする

### 11. bfcache対応（pageshow）（追加指摘）

**指摘内容**:
- `pagehide`でcleanupするとbfcacheから復元時に切断されたまま
- 通常ブラウザで戻る/進むした場合にオーバーレイが動作しない

**対応**:
```javascript
window.addEventListener('pagehide', (event) => {
  // bfcacheに保存される場合はcleanupを呼ばない
  if (!event.persisted) {
    cleanup();
  }
});

window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    // bfcacheから復元された場合
    console.log('Page restored from bfcache, reconnecting...');
    isShuttingDown = false;
    reconnectDelay = 1000;
    if (!ws || ws.readyState === WebSocket.CLOSED) {
      connectWebSocket();
    }
  }
});
```

**今後の対策**:
- `pagehide`では`event.persisted`をチェックしてbfcache保存時はcleanupしない
- `pageshow`で`event.persisted`がtrueならbfcache復元なので再接続
- OBSブラウザソースではbfcacheは使われないが、通常ブラウザでのテストに対応

### 12. メッセージフォールバックの絞り込み（追加指摘）

**指摘内容**:
- `is_sqlite_busy_error`のメッセージフォールバックが広すぎる
- "locked"や"busy"を含む非一時的エラーを誤検出する可能性
- 例: "FOREIGN KEY constraint failed: parent table is locked"

**対応**:
```rust
fn is_sqlite_busy_error(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        if let Some(code) = db_err.code() {
            // ... コード判定 ...

            // エラーコードがあるが上記に該当しない場合 → 非一時的エラー
            // メッセージフォールバックをスキップして誤検出を防ぐ
            return false;
        }

        // エラーコードがNoneの場合のみメッセージで判定
        // SQLite固有のフレーズに絞り込み
        let msg = db_err.message().to_lowercase();
        if msg.contains("database is locked") || msg.contains("database is busy") {
            return true;
        }
    }
    false
}
```

**テスト追加**:
```rust
#[test]
fn test_is_sqlite_busy_error_code_overrides_message() {
    // 非busy/lockedコードがあれば、メッセージに"locked"があってもfalse
    let err = create_mock_db_error(Some("1"), "database is locked");
    assert!(!is_sqlite_busy_error(&err));
}
```

**今後の対策**:
- メッセージフォールバックは`code()`がNoneの場合のみ使用
- フレーズはSQLite固有のものに絞り込む（"database is locked"、"database is busy"）
- 単に"busy"や"locked"を含むだけでは検出しない

### 13. tokio::time::timeoutはブロッキングSQLiteをキャンセルしない（高リスク指摘）

**指摘内容**:
- `tokio::time::timeout`は基盤となるブロッキングSQLite操作をキャンセルしない
- タイムアウトした後もトランザクションが実行中/ロック保持を続ける可能性
- リトライが並行して動作し、複数の書き込みが重複する危険性
- プール枯渇や「2秒総タイムアウト」が実際には強制されない

**対応**:
```rust
/// 1回の試行あたりの最大busy_timeout（ミリ秒）
const MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS: u64 = 500;

/// コネクションにPRAGMA busy_timeoutを設定
async fn set_busy_timeout(conn: &mut SqliteConnection, timeout_ms: u64) -> Result<(), sqlx::Error> {
    sqlx::query(&format!("PRAGMA busy_timeout = {}", timeout_ms))
        .execute(&mut *conn)
        .await?;
    Ok(())
}

async fn save_chunk_with_retry(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    let start_time = Instant::now();
    let total_timeout = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);

    loop {
        let elapsed = start_time.elapsed();
        if elapsed >= total_timeout {
            return false;
        }
        let remaining = total_timeout - elapsed;

        // 残り時間に応じてbusy_timeoutを計算（最大500ms、残り時間を超えない）
        let busy_timeout_ms = remaining
            .as_millis()
            .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS as u128) as u64;

        // コネクション取得 → busy_timeout設定 → トランザクション実行
        let result = save_chunk_with_transaction_and_timeout(pool, messages, busy_timeout_ms).await;
        // ...
    }
}
```

**今後の対策**:
- `tokio::time::timeout`はブロッキング操作をキャンセルしないことを認識
- SQLiteのブロック時間を制限するには`PRAGMA busy_timeout`を使用
- 各試行前にコネクションを取得し、busy_timeoutを残り時間に応じて動的設定
- プールの`busy_timeout`設定とは別に、リトライパスでは短いbusy_timeoutを使用

### 14. bfcache復元時の二重WebSocket接続問題（高リスク指摘）

**指摘内容**:
- `pageshow`での再接続前に、`ws.onclose`がスケジュールした再接続タイマーをクリアしていない
- bfcacheから復元時に複数のWebSocketが開かれる可能性
- メッセージの重複処理が発生する危険性

**対応**:
```javascript
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    console.log('Page restored from bfcache, reconnecting...');
    isShuttingDown = false;
    reconnectDelay = 1000;

    // ペンディング中の再接続タイマーをクリア（二重接続防止）
    // ws.oncloseがタイマーをスケジュール済みの場合があるため、
    // connectWebSocket()を呼ぶ前にクリアする
    if (reconnectTimerId) {
      clearTimeout(reconnectTimerId);
      reconnectTimerId = null;
    }

    // 二重接続を防止: CONNECTING/OPEN状態では再接続しない
    if (!ws || (ws.readyState !== WebSocket.CONNECTING && ws.readyState !== WebSocket.OPEN)) {
      connectWebSocket();
    }
  }
});
```

**今後の対策**:
- bfcache復元時には必ずペンディング中の再接続タイマーをクリア
- `connectWebSocket()`前に`ws.readyState`がCONNECTINGまたはOPENでないことを確認
- WebSocket接続状態の管理には`reconnectTimerId`と`isShuttingDown`の両方を使用

### 15. PRAGMA busy_timeoutがプール接続に漏れる問題（高リスク指摘）

**指摘内容**:
- リトライ処理で短い`busy_timeout`を設定後、復元せずに接続をプールに戻す
- プール内の接続が短いタイムアウト（<=500ms）を保持し続ける
- 通常の操作でもSQLITE_BUSYが発生しやすくなり、フォールバック率が増加

**対応**:
```rust
/// プールのデフォルトbusy_timeout（ミリ秒）
const DEFAULT_POOL_BUSY_TIMEOUT_MS: u64 = 5000;

async fn save_chunk_with_transaction_and_timeout(
    pool: &SqlitePool,
    messages: &[ChatMessage],
    busy_timeout_ms: u64,
) -> TransactionResult {
    // コネクション取得
    let mut conn = match pool.acquire().await { ... };

    // 短いbusy_timeoutを設定
    if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await { ... }

    // トランザクション実行
    let result = save_chunk_with_transaction_on_conn(&mut conn, messages).await;

    // ★ busy_timeoutをデフォルトに復元（プール内接続への影響を防ぐ）
    if let Err(e) = set_busy_timeout(&mut conn, DEFAULT_POOL_BUSY_TIMEOUT_MS).await {
        log::warn!("Failed to restore busy_timeout: {:?}", e);
    }

    result
}
```

**テスト追加**:
```rust
#[tokio::test]
async fn test_single_connection_pool_timeout_restoration() {
    // 単一接続プールでbusy_timeout復元を厳密にテスト
    let pool = SqlitePoolOptions::new()
        .max_connections(1) // 同じコネクションを再利用
        .connect_with(connect_options)
        .await
        .unwrap();

    save_chunk_with_retry(&pool, &messages).await;

    // 同じコネクションを再取得
    let mut conn = pool.acquire().await.unwrap();
    let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&mut *conn)
        .await
        .unwrap();

    // DEFAULT_POOL_BUSY_TIMEOUT_MS（5000ms）に復元されていること
    assert_eq!(timeout.0, 5000);
}
```

**今後の対策**:
- プール接続でPRAGMA設定を変更する場合、処理完了後に必ずデフォルト値に復元
- デフォルト値は定数として定義し、プール作成時の設定と一致させる
- 復元処理はRAII/guardパターンで確実に実行（Result型なら`?`演算子後でも実行）

### 16. set_busy_timeout失敗時のエラーハンドリング（高リスク指摘）

**指摘内容**:
- PRAGMA busy_timeout設定失敗を無視して続行していた
- 設定失敗時はプールのデフォルトbusy_timeout（5秒）でブロックされる可能性
- 2秒の総タイムアウト目標を達成できなくなる

**対応**:
```rust
// busy_timeoutを設定（エラー時は適切に処理）
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    if is_sqlite_busy_error(&e) {
        log::debug!("SQLITE_BUSY on set busy_timeout: {:?}", e);
        return TransactionResult::Busy;
    }
    log::warn!("Failed to set busy_timeout: {:?}", e);
    // PRAGMA失敗時は未知のタイムアウトで続行するのは危険なのでエラー
    return TransactionResult::OtherError;
}
```

**今後の対策**:
- PRAGMA設定失敗は無視せず適切にエラーハンドリング
- BUSYエラーならリトライ可能として`Busy`を返す
- その他のエラーなら即座にフォールバックへ移行
- 未知のタイムアウトで続行するとタイムアウト保証が崩れるため避ける

### 17. busy_timeout復元時に定数ではなく元の値を使用する（高リスク指摘）

**指摘内容**:
- `DEFAULT_POOL_BUSY_TIMEOUT_MS`(5000ms)をハードコードして復元していた
- プール設定が異なるbusy_timeout（例: 50ms、150ms）を持つ場合、5000msで上書きしてしまう
- テストで50msのプールを作成しているのに、5000msに変更される

**対応**:
```rust
/// busy_timeout取得失敗時のフォールバック値（ミリ秒）
const FALLBACK_BUSY_TIMEOUT_MS: u64 = 5000;

/// コネクションの現在のbusy_timeoutを取得
async fn get_busy_timeout(conn: &mut SqliteConnection) -> u64 {
    match sqlx::query_scalar::<_, i64>("PRAGMA busy_timeout")
        .fetch_one(&mut *conn)
        .await
    {
        Ok(timeout) => timeout as u64,
        Err(e) => {
            log::debug!("Failed to get busy_timeout, using fallback: {:?}", e);
            FALLBACK_BUSY_TIMEOUT_MS
        }
    }
}

async fn save_chunk_with_transaction_and_timeout(...) -> TransactionResult {
    let mut conn = pool.acquire().await?;

    // ★ 元のbusy_timeoutを取得（復元用）
    let original_timeout = get_busy_timeout(&mut conn).await;

    set_busy_timeout(&mut conn, busy_timeout_ms).await?;
    let result = save_chunk_with_transaction_on_conn(&mut conn, messages).await;

    // ★ 元の値に復元（定数ではなく）
    let _ = set_busy_timeout(&mut conn, original_timeout).await;
    result
}
```

**テスト追加**:
```rust
#[tokio::test]
async fn test_non_default_pool_timeout_restoration() {
    // 非デフォルトbusy_timeout（150ms）でプールを作成
    let connect_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_millis(150));

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(connect_options)
        .await
        .unwrap();

    save_chunk_with_retry(&pool, &messages).await;

    // 元の値（150ms）に復元されていること（5000msではない）
    let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(timeout.0, 150);
}
```

**今後の対策**:
- PRAGMA設定を変更する場合、定数ではなく元の値を取得して復元
- 元の値取得に失敗した場合のフォールバック値を用意
- 単一接続プールで復元が正しく行われることをテスト

### 18. backoff sleepを残り予算で制限する（高リスク指摘）

**指摘内容**:
- backoff sleepが`RETRY_TOTAL_TIMEOUT_MS`の予算に制限されていない
- pool.acquire()とbackoff sleepの合計で2秒を超える可能性
- 総タイムアウトがend-to-endで強制されていない

**対応**:
```rust
TransactionResult::Busy => {
    attempt += 1;
    // ...試行回数チェック...

    // 残り時間を再計算してbackoffを制限
    let remaining_ms = (total_timeout - elapsed).as_millis() as u64;

    // 残り時間がbackoff + 最小試行時間(50ms)未満なら諦める
    if remaining_ms < backoff_ms + 50 {
        log::warn!(
            "SQLITE_BUSY: Remaining time ({}ms) too short for backoff ({}ms) + retry, giving up",
            remaining_ms,
            backoff_ms
        );
        return false;
    }

    // backoffを残り時間で制限（次の試行のための余裕を残す）
    let clamped_backoff = backoff_ms.min(remaining_ms.saturating_sub(50));

    sleep(Duration::from_millis(clamped_backoff)).await;
    backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
}
```

**今後の対策**:
- リトライ処理のsleepは残り時間で制限する
- 残り時間が次の試行に十分でない場合は早めに諦める
- 総タイムアウトはbusy_timeout + backoff + 試行時間の合計で強制

### 19. get_busy_timeout()失敗時のOption型対応（高リスク指摘）

**指摘内容**:
- `get_busy_timeout()`が失敗時にフォールバック値（5000ms）を返していた
- 一時的な障害時にプール設定とは異なる値で復元してしまう
- 非デフォルトbusy_timeout（例: 150ms）のプールが5000msに上書きされる

**対応**:
```rust
/// コネクションの現在のbusy_timeoutを取得
/// 取得失敗時はNoneを返す（呼び出し元で復元をスキップする判断材料として）
async fn get_busy_timeout(conn: &mut SqliteConnection) -> Option<u64> {
    match sqlx::query_scalar::<_, i64>("PRAGMA busy_timeout")
        .fetch_one(&mut *conn)
        .await
    {
        Ok(timeout) => Some(timeout as u64),
        Err(e) => {
            log::debug!("Failed to get busy_timeout: {:?}", e);
            None
        }
    }
}

// 呼び出し側
let original_timeout = get_busy_timeout(&mut conn).await;
// ...トランザクション...
// 復元: Noneの場合はスキップして未知の値での上書きを回避
if let Some(timeout) = original_timeout {
    if let Err(e) = set_busy_timeout(&mut conn, timeout).await {
        log::warn!("Failed to restore busy_timeout: {:?}", e);
    }
} else {
    log::debug!("Skipping busy_timeout restoration (original value unknown)");
}
```

**今後の対策**:
- PRAGMA取得失敗時はフォールバック値を使わずOption::Noneを返す
- 元の値が不明な場合は復元をスキップして既存設定を維持
- 確実に復元できる値がある場合のみ復元処理を実行

### 20. BUSY時のrollback失敗をOtherErrorとして扱う（高リスク指摘）

**指摘内容**:
- BUSY発生時のrollback失敗を無視して`TransactionResult::Busy`を返していた
- rollback失敗時は接続が汚染されている可能性がある
- 汚染された接続でリトライすると予期しない動作になる

**対応**:
```rust
if is_sqlite_busy_error(&e) {
    log::debug!("SQLITE_BUSY during insert: {:?}", e);
    // rollback失敗時は接続が汚染されている可能性があるためOtherErrorとして扱う
    if let Err(rb_err) = tx.rollback().await {
        log::warn!("Rollback failed after BUSY during insert: {:?}", rb_err);
        return TransactionResult::OtherError;
    }
    return TransactionResult::Busy;
}
```

**今後の対策**:
- rollback失敗はトランザクション状態が不定のため深刻なエラーとして扱う
- rollback失敗時はリトライ不可（OtherError）として、フォールバック処理へ移行
- 汚染された可能性のある接続でのリトライは避ける

### 21. ws.onopenで再接続タイマーをクリア（高リスク指摘）

**指摘内容**:
- `ws.onopen`で`reconnectTimerId`をクリアしていない
- 手動/早期再接続で`onopen`に到達した場合、予約済みタイマーが残っている可能性
- タイマーが発火すると二重WebSocket接続が発生

**対応**:
```javascript
ws.onopen = () => {
  console.log('WebSocket connected');
  reconnectDelay = 1000;
  // ペンディング中の再接続タイマーをクリア（二重接続防止）
  // 手動/早期再接続でonopenに到達した場合、予約済みタイマーが残っている可能性がある
  if (reconnectTimerId) {
    clearTimeout(reconnectTimerId);
    reconnectTimerId = null;
  }
  // ...
};
```

**今後の対策**:
- WebSocketの`onopen`では必ず再接続タイマーをクリア
- `pageshow`だけでなく`onopen`でもタイマー管理を行う
- 接続成功時点で予約済みタイマーは不要になるため、即座にクリア

### 22. pool.acquire()のタイムアウト制限（高リスク指摘）

**指摘内容**:
- `pool.acquire().await`が残り予算を超えてブロックする可能性
- プール枯渇時に無期限待機となり、総タイムアウト保証が崩れる
- `save_comments_to_db`が2秒を大幅に超えて停滞する可能性

**対応**:
```rust
// pool.acquire()をtokio::time::timeoutでラップ
let acquire_timeout = Duration::from_millis(busy_timeout_ms);
let mut conn = match timeout(acquire_timeout, pool.acquire()).await {
    Ok(Ok(conn)) => conn,
    Ok(Err(e)) => {
        if is_sqlite_busy_error(&e) {
            return TransactionResult::Busy;
        }
        return TransactionResult::OtherError;
    }
    Err(_) => {
        // タイムアウト: プール接続待ちが予算を超過
        log::debug!("Connection acquire timed out after {}ms", busy_timeout_ms);
        return TransactionResult::Busy;
    }
};
```

**今後の対策**:
- `pool.acquire()`は必ず`tokio::time::timeout`でラップ
- タイムアウト値は残り予算から計算
- タイムアウト時は`Busy`として扱い、リトライ可能にする

### 23. get_busy_timeout()失敗時は処理を中止（高リスク指摘）

**指摘内容**:
- `get_busy_timeout()`失敗時に短い`busy_timeout`を設定後、復元をスキップ
- 接続が劣化した状態でプールに戻され、他の呼び出し元に影響
- SQLITE_BUSY発生率が増加する可能性

**対応**:
```rust
// 元のbusy_timeoutを取得（復元用）
// 取得失敗時は復元不能なため、OtherErrorを返して続行しない
let original_timeout = match get_busy_timeout(&mut conn).await {
    Some(timeout) => timeout,
    None => {
        log::warn!("Cannot proceed: failed to get original busy_timeout for restoration");
        return TransactionResult::OtherError;
    }
};

// ここで初めてbusy_timeoutを設定
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await { ... }
```

**今後の対策**:
- PRAGMA値の変更前に必ず元の値を取得
- 取得失敗時は変更を行わずにエラー終了
- 「変更 → 復元スキップ」の状態を作らない

### 24. フォールバックパスの予算制限（高リスク指摘）

**指摘内容**:
- `save_chunk_individually`がプールのデフォルト`busy_timeout`を使用
- フォールバックパスで2秒を超える可能性
- end-to-endのタイムアウト保証が崩れる

**対応**:
```rust
// save_comments_to_dbからremainingを渡す
save_chunk_individually(pool, chunk, remaining).await;

// save_chunk_individually内でも予算を制限
async fn save_chunk_individually(pool: &SqlitePool, messages: &[ChatMessage], remaining: Duration) {
    // 接続取得にもタイムアウトを適用
    let acquire_timeout_ms = (remaining.as_millis() as u64 / 2).min(500);
    let mut conn = match timeout(acquire_timeout, pool.acquire()).await { ... };

    // busy_timeoutも残り予算で制限
    let busy_timeout_ms = ((remaining - start_time.elapsed()).as_millis() as u64).min(500);
    set_busy_timeout(&mut conn, busy_timeout_ms).await?;

    for msg in messages {
        // 予算チェック
        if start_time.elapsed() >= remaining {
            break;
        }
        insert_comment(&mut *conn, msg).await;
    }
}
```

**今後の対策**:
- フォールバックパスにも同じ予算制限を適用
- `remaining`パラメータで予算を明示的に渡す
- 予算超過時は処理を中断し、残りをスキップ

### 25. Duration::saturating_subでアンダーフロー防止（高リスク指摘）

**指摘内容**:
- `remaining - start_time.elapsed()`でsaturationなし
- スケジューラ遅延等でelapsedがremainingを超えるとパニック
- フォールバックパスがクラッシュする可能性

**対応**:
```rust
// saturating_subでアンダーフローを防止
let remaining_after_acquire = remaining.saturating_sub(start_time.elapsed());
if remaining_after_acquire.as_millis() < 50 {
    log::debug!("...");
    return;
}
let busy_timeout_ms = (remaining_after_acquire.as_millis() as u64).min(500);
```

**今後の対策**:
- Duration間の減算は必ず`saturating_sub()`を使用
- アンダーフロー後のゼロチェックを必ず行う
- 時間計算ではスケジューラ遅延を考慮

### 26. フォールバックパスのbusy_timeout復元（高リスク指摘）

**指摘内容**:
- フォールバックパスで短い`busy_timeout`を設定後、復元していない
- 接続がプールに戻され、他の呼び出し元に影響
- SQLITE_BUSY発生率が増加、ドロップされる書き込みが増加

**対応**:
```rust
// 元のbusy_timeoutを取得
let original_timeout = get_busy_timeout(&mut conn).await;

// original_timeoutがSomeの場合のみbusy_timeoutを変更
let should_restore = if let Some(_) = original_timeout {
    if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
        false // 設定失敗時は復元不要
    } else {
        true // 設定成功時は後で復元
    }
} else {
    false // 元の値が不明なので変更しない
};

// ... INSERT処理 ...

// busy_timeoutを元の値に復元（変更した場合のみ）
if should_restore {
    if let Some(timeout) = original_timeout {
        let _ = set_busy_timeout(&mut conn, timeout).await;
    }
}
```

**今後の対策**:
- フォールバックパスでもリトライパスと同じ復元ロジックを適用
- `original_timeout`がNoneの場合は変更しない（復元不能なため）
- `should_restore`フラグで変更の有無を追跡

### 27. 総予算によるデータスキップの設計判断（高リスク指摘 → 将来対応）

**指摘内容**:
- 2秒の総予算でSQLITE_BUSYなしでもメッセージをスキップする可能性
- 従来の「全て書き込む」動作からの行動回帰
- データ損失リスク

**対応**:
- `docs/900_tasks.md`に将来タスクとして追記
- 現状はリアルタイム性を優先し、2秒以内の完了を保証
- 対応案:
  - 予算をチャンクごとにスコープ
  - スキップ数をログ/メトリクスで報告
  - 上流でリキュー可能に

**今後の対策**:
- タイムアウト導入時はデータ損失リスクを検討
- スキップ発生時のログ/メトリクスを確保
- 本番運用後にフィードバックを収集して判断

### 28. save_chunk_with_retryのend-to-end予算管理（高リスク指摘）

**指摘内容**:
- `save_chunk_with_retry`が独自の2秒タイマーを持っている
- 外側の`save_comments_to_db`の総予算を無視して、遅いチャンクが2秒を消費
- end-to-endのタイムアウト保証が破れる
- 例: 外側で残り500msしかないのに、チャンクは2秒まで使える

**対応**:
```rust
// Before（問題あり）:
async fn save_chunk_with_retry(pool: &SqlitePool, messages: &[ChatMessage]) -> bool {
    let start_time = Instant::now();
    let total_timeout = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS); // 独自タイマー
    // ...
}

// After（修正済み）:
async fn save_chunk_with_retry(
    pool: &SqlitePool,
    messages: &[ChatMessage],
    remaining: Duration,  // 外側から予算を受け取る
) -> bool {
    let start_time = Instant::now();
    let total_timeout = remaining;  // 外側の予算を使用
    // ...
}

// 呼び出し側:
let remaining = total_timeout.saturating_sub(elapsed);
if !save_chunk_with_retry(pool, chunk, remaining).await {
    // ...
}
```

**追加テスト**:
- `test_tiny_budget_skips_gracefully`: 極小予算（10ms）でパニックしないことを確認
- `test_budget_exhaustion_mid_chunk`: 100件のメッセージが予算内で処理されることを確認
- `test_remaining_budget_passed_to_retry`: 500ms予算内で完了することを確認

**今後の対策**:
- 内部関数は独自のタイムアウトを持たず、外側から予算を受け取る
- end-to-end予算管理を意識した設計
- チェーンされた関数間で予算を渡すパターンを標準化

### 29. save_chunk_individuallyのbusy_timeout取得/設定失敗時の即時終了（高リスク指摘）

**指摘内容**:
- `save_chunk_individually`で`get_busy_timeout`がNoneまたは`set_busy_timeout`失敗時に続行
- 接続の以前のbusy_timeout（5秒など）で長時間ブロックする可能性
- 予算を超過してリアルタイム保証が破れる

**対応**:
```rust
// Before（問題あり）:
let original_timeout = get_busy_timeout(&mut conn).await;
// Noneの場合も続行 → 5秒でブロック可能

let should_restore = if let Some(_) = original_timeout {
    if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
        false // 設定失敗時も続行 → 5秒でブロック可能
    } else {
        true
    }
} else {
    false // 続行してしまう
};

// After（修正済み）:
let original_timeout = match get_busy_timeout(&mut conn).await {
    Some(timeout) => timeout,
    None => {
        log::debug!("Skipping individual insert fallback: failed to get original busy_timeout");
        return; // 即座に終了
    }
};

if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    log::debug!("Skipping individual insert fallback: failed to set busy_timeout: {:?}", e);
    return; // 即座に終了
}
```

**追加テスト**:
- `test_fallback_restores_busy_timeout`: フォールバックパスでもbusy_timeoutが復元されることを確認
- `test_fallback_tiny_budget_exits_without_blocking`: 極小予算で長時間ブロックせずに終了することを確認

**今後の対策**:
- 予算を強制できない場合は続行せず即座に終了
- 「続行」よりも「安全に終了」を優先
- フォールバックパスでもリトライパスと同じ厳密さを維持

### 30. deadline-based予算管理とpoisoned connection対応（高リスク指摘）

**指摘内容**:
1. **acquire後の予算再計算**:
   - `busy_timeout_ms`が`pool.acquire()`前に計算されていた
   - acquire待ちに時間がかかると、acquire+busy_timeoutで予算を超過
   - 例: 残り500msで、acquire=300ms、busy_timeout=500ms → 800ms消費

2. **rollback失敗時のpoisoned connection**:
   - rollback失敗時も接続をプールに戻していた
   - 汚染されたトランザクション状態が他の呼び出し元に影響
   - SQLITE_BUSYが増幅される可能性

3. **busy_timeout復元失敗時のpool poisoning**:
   - 復元失敗時も接続をプールに戻していた
   - 短いbusy_timeoutが他の呼び出し元に影響

**対応**:

1. **deadline-based予算管理**:
```rust
// Before（問題あり）:
async fn save_chunk_with_transaction_and_timeout(
    pool: &SqlitePool,
    messages: &[ChatMessage],
    busy_timeout_ms: u64,  // 事前計算された値
) -> TransactionResult {
    let mut conn = pool.acquire().await?;  // acquire時間は考慮されない
    set_busy_timeout(&mut conn, busy_timeout_ms).await?;
    // ...
}

// After（修正済み）:
async fn save_chunk_with_transaction_and_timeout(
    pool: &SqlitePool,
    messages: &[ChatMessage],
    deadline: Instant,  // 絶対的なデッドライン
) -> TransactionResult {
    // デッドラインまでの残り時間を計算
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.as_millis() < 50 {
        return TransactionResult::Busy;
    }

    // acquire用のタイムアウト（残り時間の半分、最大500ms）
    let acquire_timeout = Duration::from_millis(
        (remaining.as_millis() as u64 / 2).min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS)
    );

    let mut conn = match timeout(acquire_timeout, pool.acquire()).await { ... };

    // ★ acquire後に残り時間を再計算（acquire時間を差し引く）
    let remaining_after_acquire = deadline.saturating_duration_since(Instant::now());
    if remaining_after_acquire.as_millis() < 50 {
        return TransactionResult::Busy;
    }

    let busy_timeout_ms = (remaining_after_acquire.as_millis() as u64)
        .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS);
    // ...
}
```

2. **TransactionResult::Poisoned追加**:
```rust
enum TransactionResult {
    Success,
    Busy,
    OtherError,
    /// 接続が汚染された状態（rollback失敗等）
    /// この場合、接続をプールに戻さずに破棄すべき
    Poisoned,
}

// rollback失敗時:
if let Err(rb_err) = tx.rollback().await {
    log::warn!("Rollback failed after insert error: {:?}", rb_err);
    return TransactionResult::Poisoned;
}
```

3. **conn.detach()によるpoisoned connection排除**:
```rust
// Poisoned状態の場合は接続を切り離す
if result == TransactionResult::Poisoned {
    log::warn!("Transaction resulted in poisoned connection, detaching from pool");
    conn.detach();
    return TransactionResult::OtherError;
}

// busy_timeout復元失敗時も切り離し
if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
    log::warn!("Failed to restore busy_timeout, detaching connection from pool");
    conn.detach();
}
```

**今後の対策**:
- タイムアウト計算は事前ではなく、操作直前に行う
- `deadline: Instant`パターンで絶対的な期限を渡す
- rollback/復元失敗時は`conn.detach()`でプールから排除
- `TransactionResult::Poisoned`で汚染状態を明示的に表現

### 31. フォールバックパスでもbusy_timeout復元失敗時にconn.detach()（高リスク指摘）

**指摘内容**:
- `save_chunk_individually`でbusy_timeout復元失敗時に`conn.detach()`を呼んでいない
- 短いbusy_timeoutの接続がプールに戻り、他の呼び出し元でSQLITE_BUSYが増加
- リトライパスでは`conn.detach()`を呼んでいるのにフォールバックパスでは未対応

**対応**:
```rust
// Before（問題あり）:
if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
    log::warn!(
        "Failed to restore busy_timeout in fallback to original ({}ms): {:?}",
        original_timeout,
        e
    );
    // ← conn.detach()がない
}

// After（修正済み）:
if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
    log::warn!(
        "Failed to restore busy_timeout in fallback to original ({}ms), detaching connection: {:?}",
        original_timeout,
        e
    );
    conn.detach(); // プールから切り離し
}
```

**今後の対策**:
- 同じエラー処理はリトライパスとフォールバックパスで一貫させる
- busy_timeout復元失敗時は常に`conn.detach()`でプールから排除
- コードレビュー時に「同様の処理が他にないか」を網羅的に確認

### 32. bfcache復元時のマネージャー再初期化（高リスク指摘）

**指摘内容**:
- `cleanup()`で`densityManager.destroy()`と`updateBatcher.destroy()`を呼び出し
- bfcache復元時（`pageshow`で`event.persisted`がtrue）にマネージャーが使用不能
- バッチ処理と過密検出が無効になり、パフォーマンス低下の可能性
- OBSブラウザソースではbfcacheは使われないが、通常ブラウザでのテストに影響

**対応**:
```javascript
// 1. const → let に変更（再代入を許可）
let updateBatcher = new UpdateBatcher({ batchInterval: 150 });
let densityManager = new DensityManager();

// 2. pageshowハンドラでマネージャーを再初期化
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    // ... 既存の再接続処理 ...

    // DensityManagerのタイマーが停止している場合は再作成
    if (!densityManager || densityManager._cleanupTimerId === null) {
      densityManager = new DensityManager();
      console.log('DensityManager reinitialized after bfcache restore');
    }
    // UpdateBatcherも再作成
    if (!updateBatcher) {
      updateBatcher = new UpdateBatcher({ batchInterval: 150 });
      console.log('UpdateBatcher reinitialized after bfcache restore');
    }
    // ...
  }
});
```

**今後の対策**:
- `destroy()`で停止したリソースは`pageshow`で再初期化
- bfcache対応が必要な変数は`const`ではなく`let`で宣言
- タイマーIDの存在チェックでdestroy済みかを判定
- OBSブラウザソース以外でのテストも考慮した設計

## 未対応（将来対応）

### 1. EXCLUSIVEトランザクションを使用したデッドロックテスト
- 一方の接続がEXCLUSIVEロックを保持した状態で、他方からsave_chunk_with_retryを呼び出す
- リトライロジックが正しく動作し、ロック解放後に成功することを検証
- 複雑なテストセットアップが必要なため、優先度低

### 2. cleanup()が再接続をスケジュールしないことのテスト
- JSテストハーネスを追加した場合に検証
- 現状は手動QAで対応

### 3. pagehide/pageshowのbfcache動作テスト
- 通常ブラウザで戻る/進むした場合の動作を手動QAで確認
- OBSブラウザソースではbfcacheは使われない

### 4. save_comments_to_dbの戻り値構造化（設計判断）
- 現在: `save_comments_to_db`は`()`を返し、呼び出し元に成功/失敗/スキップを通知しない
- 問題: 2s予算超過でサイレントにメッセージをドロップ（データ損失リスク）
- 対応案:
  - `{ saved: usize, failed: usize, skipped: usize }`を返す
  - 予算を設定可能に（メッセージ数/チャンク数に比例）
  - スキップ発生時は呼び出し元でリキュー可能に
- 現状はリアルタイム性を優先し、本番運用後にフィードバックを収集

### 5. busy_timeoutのタイムアウト保証の限界（設計判断）
- 現在: `busy_timeout`はロック待ち時間のみ制限
- 問題: 遅いディスクI/Oで2s保証が破れる可能性
- 対応案:
  - ドキュメントで「ロック待ちのみ制限」を明記
  - または`spawn_blocking` + `timeout`でキャンセル可能に
  - SQLite interrupt/progress handlerの使用を検討
- 優先度: 低（遅いディスクI/Oは稀なケース）

### 33. per-attempt busy_timeoutがプール設定を超える問題（高リスク指摘）

**指摘内容**:
- per-attempt `busy_timeout`を`MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS`（500ms）で制限していた
- プールが意図的に低い値（例: 50ms）に設定されていても、500msまでブロックする可能性
- オペレータが設定した応答性の制約が無視される

**対応**:
```rust
// Before（問題あり）:
let busy_timeout_ms = (remaining_after_acquire.as_millis() as u64)
    .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS);

// After（修正済み）:
// 3つの制約でクランプ:
// 1. original_timeout: プール設定を超えない（オペレータ意図を尊重）
// 2. MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS: 1回の試行での最大待ち時間
// 3. remaining_after_acquire: 残り予算を超えない
let busy_timeout_ms = original_timeout
    .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS)
    .min(remaining_after_acquire.as_millis() as u64);
```

**フォールバックパスも同様に修正**:
```rust
// save_chunk_individually内
let busy_timeout_ms = original_timeout
    .min(500)
    .min(remaining_after_acquire.as_millis() as u64);
```

**今後の対策**:
- 動的にタイムアウトを設定する場合は、元の設定値を超えないようにクランプ
- オペレータが意図的に設定した制約は尊重する
- リトライパスとフォールバックパスで一貫したロジックを使用

### 34. original_timeout==0で無限ブロックする問題（高リスク指摘）

**指摘内容**:
- `original_timeout`が0の場合（SQLiteデフォルトまたは設定ミス）
- `busy_timeout_ms`が0になり、SQLiteが無限にブロックする可能性
- 2秒の予算保証が破れる

**対応**:
```rust
// Before（問題あり）:
let busy_timeout_ms = original_timeout
    .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS)
    .min(remaining_after_acquire.as_millis() as u64);
// original_timeout == 0 なら busy_timeout_ms == 0 → 無限ブロック

// After（修正済み）:
// original_timeout == 0 は「制限なし」を意味するため、u64::MAXとして扱う
let effective_original = if original_timeout == 0 {
    u64::MAX
} else {
    original_timeout
};

let busy_timeout_ms = effective_original
    .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS)
    .min(remaining_after_acquire.as_millis() as u64);
// original_timeout == 0 でも MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS と remaining で適切にクランプ
```

**フォールバックパスも同様に修正**。

**今後の対策**:
- タイムアウト値0は「制限なし」として扱うのがSQLiteの仕様
- 0をそのまま使用すると無限ブロックになるため、u64::MAXに変換
- MIN関数でクランプする際は、0の特殊な意味を考慮

### 35. get_busy_timeout失敗時のBUSYエラー対応（将来検討）

**指摘内容**:
- `get_busy_timeout()`失敗時に`OtherError`としてリトライを停止
- 一時的なBUSYエラーの場合、リトライすべきでは

**対応**:
- PRAGMA busy_timeoutは通常SQLITE_BUSYにならない（データベースファイルにアクセスしない）
- 現状はログレベルを`warn`に変更し、エラー内容を可視化
- 将来的に`Result<u64, sqlx::Error>`に変更し、BUSYならリトライ可能にすることを検討
- `docs/900_tasks.md`に将来タスクとして記載

### 36. フォールバックパスの早期リターン時のログレベル昇格（高リスク指摘）

**指摘内容**:
- `save_chunk_individually`で予算不足やPRAGMA失敗で早期リターンする箇所が`log::debug!`
- 本番環境のデフォルトログレベルでは見えず、サイレントデータ損失になる
- スキップされたメッセージ数を運用者が検出できない

**対応箇所**:
1. 残り時間が50ms未満でスキップ（L560-567）
2. 接続取得タイムアウト（L581-587）
3. `get_busy_timeout`失敗（L596-602）
4. 残り時間（acquire後）が50ms未満（L609-615）
5. `set_busy_timeout`失敗（L637-644）

**対応**:
すべての早期リターン箇所を`log::warn!`に昇格し、スキップされたメッセージ数を出力:
```rust
// Before（問題あり）:
log::debug!(
    "Skipping individual insert fallback: remaining time ({}ms) too short",
    remaining.as_millis()
);

// After（修正済み）:
log::warn!(
    "Skipping individual insert fallback: remaining time ({}ms) too short, {} messages not saved",
    remaining.as_millis(),
    messages.len()
);
```

**今後の対策**:
- データ損失の可能性がある箇所は`log::warn!`以上を使用
- スキップ件数を含めて運用者が問題を検出できるようにする
- 途中経過の`log::debug!`と最終結果の`log::warn!`を区別
- サマリーログに到達しない早期リターンパスに注意

### 37. get_busy_timeout/set_busy_timeout失敗時にconn.detach()（高リスク指摘）

**指摘内容**:
- `get_busy_timeout`や`set_busy_timeout`が失敗した場合に接続を`detach`せずプールに戻していた
- PRAGMA失敗はIO障害等の深刻な問題の可能性があり、接続が汚染されているかもしれない
- 汚染された接続がプール内に残り、後続の書き込みで問題が発生する可能性

**対応箇所**:
1. `save_chunk_with_transaction_and_timeout`内の`get_busy_timeout`失敗時:
```rust
// Before（問題あり）:
let original_timeout = match get_busy_timeout(&mut conn).await {
    Some(timeout) => timeout,
    None => {
        log::warn!("Cannot proceed: failed to get original busy_timeout");
        return TransactionResult::OtherError;
        // ← conn.detach()がない
    }
};

// After（修正済み）:
let original_timeout = match get_busy_timeout(&mut conn).await {
    Some(timeout) => timeout,
    None => {
        log::warn!("Cannot proceed: failed to get original busy_timeout, detaching connection");
        conn.detach();  // プールから切り離し
        return TransactionResult::OtherError;
    }
};
```

2. `save_chunk_with_transaction_and_timeout`内の`set_busy_timeout`失敗時:
```rust
// Before（問題あり）:
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    if is_sqlite_busy_error(&e) {
        return TransactionResult::Busy;
    }
    return TransactionResult::OtherError;
    // ← conn.detach()がない
}

// After（修正済み）:
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    if is_sqlite_busy_error(&e) {
        log::debug!("SQLITE_BUSY on set busy_timeout, detaching connection: {:?}", e);
        conn.detach();
        return TransactionResult::Busy;
    }
    log::warn!("Failed to set busy_timeout, detaching connection: {:?}", e);
    conn.detach();
    return TransactionResult::OtherError;
}
```

3. `save_chunk_individually`内の`get_busy_timeout`失敗時:
```rust
// After（修正済み）:
let original_timeout = match get_busy_timeout(&mut conn).await {
    Some(timeout) => timeout,
    None => {
        log::debug!("Skipping individual insert fallback: failed to get original busy_timeout, detaching connection");
        conn.detach();
        return;
    }
};
```

4. `save_chunk_individually`内の`set_busy_timeout`失敗時:
```rust
// After（修正済み）:
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    log::debug!("Skipping individual insert fallback: failed to set busy_timeout, detaching connection: {:?}");
    conn.detach();
    return;
}
```

**今後の対策**:
- PRAGMA操作の失敗はIO障害等の深刻な問題の可能性がある
- 失敗時は接続を`detach()`でプールから切り離し、汚染を防止
- 「エラー時は接続を使い捨てる」というセーフティネット思想
- リトライパスとフォールバックパスで一貫した対応を行う

### 38. set_busy_timeout BUSY時はconn.detach()しない（高リスク指摘）

**指摘内容**:
- `set_busy_timeout`でBUSYエラーが発生した場合に`conn.detach()`を呼んでいた
- BUSYエラーは一時的なロック競合であり、接続自体は正常
- 一時的なロック競合で接続を破棄するとプールが枯渇する可能性

**対応**:
```rust
// Before（問題あり）:
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    if is_sqlite_busy_error(&e) {
        log::debug!("SQLITE_BUSY on set busy_timeout, detaching connection: {:?}", e);
        conn.detach();  // ← BUSYでもdetach
        return TransactionResult::Busy;
    }
    log::warn!("Failed to set busy_timeout, detaching connection: {:?}", e);
    conn.detach();
    return TransactionResult::OtherError;
}

// After（修正済み）:
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    if is_sqlite_busy_error(&e) {
        log::debug!("SQLITE_BUSY on set busy_timeout: {:?}", e);
        // 一時的なロック競合なのでdetachせずBusyを返す（リトライ可能）
        return TransactionResult::Busy;
    }
    log::warn!("Failed to set busy_timeout, detaching connection: {:?}", e);
    conn.detach();  // 非BUSYエラーはIO障害等の可能性があるためdetach
    return TransactionResult::OtherError;
}
```

**フォールバックパスも同様に修正**:
```rust
// save_chunk_individually内
if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
    if is_sqlite_busy_error(&e) {
        log::warn!(
            "Skipping individual insert fallback: SQLITE_BUSY on set busy_timeout, {} messages not saved: {:?}",
            messages.len(),
            e
        );
        // 一時的なロック競合なのでdetachしない
        return;
    }
    log::warn!(
        "Skipping individual insert fallback: failed to set busy_timeout, detaching connection, {} messages not saved: {:?}",
        messages.len(),
        e
    );
    conn.detach();  // 非BUSYエラーはdetach
    return;
}
```

**今後の対策**:
- BUSYエラーは一時的なロック競合であり、接続は正常
- BUSYエラー時は`conn.detach()`を呼ばずにリトライ可能として返す
- 非BUSYエラー（IO障害等）は接続が汚染されている可能性があるため`conn.detach()`
- 「一時的エラー」と「永続的エラー」を区別し、適切に対応する

### 39. 内部フィールドへの直接参照を避けてpublicメソッドを使用（コード品質）

**指摘内容**:
- `combined-v2.html`で`densityManager._cleanupTimerId === null`と内部フィールドを直接参照していた
- 内部実装への結合が強くなり、将来の変更に弱くなる

**対応**:
```javascript
// Before（問題あり）:
if (!densityManager || densityManager._cleanupTimerId === null) {
  densityManager = new DensityManager();
}

// After（修正済み）:
// density-manager.jsにpublicメソッドを追加
isDestroyed() {
  return this._cleanupTimerId === null;
}

// combined-v2.html
if (!densityManager || densityManager.isDestroyed()) {
  densityManager = new DensityManager();
}
```

**今後の対策**:
- `_`プレフィックスのフィールドは内部実装として扱い、外部から直接参照しない
- 状態チェックが必要な場合はpublicメソッド（`isDestroyed()`等）を追加
- 内部実装の変更が外部に影響しないようカプセル化を維持

### 40. テストでの接続復元検証は単一接続プールを使用（テスト品質）

**指摘内容**:
- `test_fallback_restores_busy_timeout`で2接続プールを使用していた
- 2接続プールでは、検証時に取得した接続が`save_chunk_individually`で使用した接続と同一かどうかが不明確
- 復元検証が確実ではない

**対応**:
```rust
// 単一接続バリアントを追加: test_fallback_restores_busy_timeout_single_connection
let pool = SqlitePoolOptions::new()
    .max_connections(1) // 単一接続: 同じコネクションが再利用される
    .acquire_timeout(std::time::Duration::from_secs(5))
    .connect_with(connect_options)
    .await
    .unwrap();

// ... save_chunk_individually呼び出し ...

// 同じ接続を取得してbusy_timeoutを確認（単一接続なので必ず同一接続）
let mut conn = pool.acquire().await.unwrap();
let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
    .fetch_one(&mut *conn)
    .await
    .unwrap();

assert_eq!(timeout.0, 2500, "..."); // 確実に復元を検証
```

**今後の対策**:
- 接続状態の復元テストは単一接続プールを使用して確実に検証
- 複数接続プールでの汎用的なテストも併用し、実運用シナリオもカバー
- 「同一接続であること」が重要な場合は、テスト設計段階で意識する

### 41. original_timeout==0のクランプロジックをテスト（テスト品質）

**指摘内容**:
- `original_timeout == 0`（SQLiteデフォルト: 制限なし）の場合のクランプロジックがテストされていなかった
- 0がu64::MAXとして扱われ、適切にクランプされることを検証する必要がある

**対応**:
```rust
// test_original_timeout_zero_is_clamped を追加
// busy_timeoutを0に設定（SQLiteデフォルト: 制限なし）
sqlx::query("PRAGMA busy_timeout = 0")
    .execute(&pool)
    .await
    .unwrap();

// 500msの予算で呼び出し
// original_timeout=0がu64::MAXとして扱われ、MIN(u64::MAX, 500, remaining)=500ms以内で完了すること
let budget = Duration::from_millis(500);
let result = save_chunk_with_retry(&pool, &messages, budget).await;

// 600ms以内に完了すること（無限ブロックしていない）
assert!(elapsed < Duration::from_millis(600), "...");
```

**今後の対策**:
- タイムアウト値の特殊ケース（0、最大値、負数など）は明示的にテスト
- SQLiteでは0が「制限なし」を意味するため、この特殊な意味をドキュメントとテストで明確化
- MIN/MAX関数でクランプする際は、エッジケースを考慮

### 42. 戻り値なしの関数でデータ損失が発生する場合の設計（設計判断）

**指摘内容**:
- `save_comments_to_db`が`()`を返し、固定2秒予算でスキップされたメッセージが呼び出し元に通知されない
- サイレントなデータ損失リスクがある

**対応（将来タスクとして記録）**:
- `docs/900_tasks.md`に「save_comments_to_dbの総予算によるデータスキップ検討」として記録済み
- 対応案:
  - 戻り値を構造化: `{ saved: usize, failed: usize, skipped: usize }`
  - 予算を設定可能に: メッセージ数/チャンク数に比例、または設定ファイルで変更可能
  - テスト用に予算を注入可能に

**今後の対策**:
- データを処理する関数では、処理結果（成功/失敗/スキップ数）を戻り値で返す
- タイムアウトや予算による早期終了がある場合は、呼び出し元にその事実を通知
- ログ出力だけでなく、プログラム的に検出・対応できる設計を検討

### 43. テスト用一時ファイルの分離とクリーンアップ（テスト品質）

**指摘内容**:
- PIDベースのテンポラリファイル命名（`test_{pid}.db`）では、テスト中断時に古いファイルが残る
- 再実行時に古いデータがテスト結果に影響する可能性

**対応（将来タスクとして記録）**:
- `docs/900_tasks.md`に「テスト用DBファイルの分離改善」として追記
- 対応案:
  - `tempfile`クレートを使用して自動削除されるテンポラリファイルを使用
  - テスト開始前にファイル削除するヘルパー関数を追加
  - PID + UUIDでユニークなパス生成

**今後の対策**:
- テスト用の一時ファイルは`tempfile`クレートまたは同等の仕組みを使用
- テスト開始時に前回の残留ファイルを削除するか、ユニークなパスを使用
- `DROP TABLE IF EXISTS`ではなく`CREATE TABLE IF NOT EXISTS`に頼らない設計

### 44. busy_timeout復元時の一時的BUSYエラーによるプール枯渇（高リスク指摘）

**指摘内容**:
- `set_busy_timeout`でbusy_timeoutを復元する際、**任意の**エラーで`conn.detach()`していた
- 一時的なSQLITE_BUSYエラー（他のトランザクションがロック中）でも接続を切り離す
- 高負荷時にプール接続が急速に枯渇する可能性

**問題のあるコード**:
```rust
// Before（問題あり）:
if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
    log::warn!(
        "Failed to restore busy_timeout to original ({}ms): {:?}, detaching connection from pool",
        original_timeout,
        e
    );
    conn.detach();  // BUSYエラーでも即座にdetach → プール枯渇
}
```

**対応**:
```rust
// After（修正済み）:
// busy_timeoutを元の値に復元（プール内接続への影響を防ぐ）
if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
    if is_sqlite_busy_error(&e) {
        // BUSYエラーの場合は短いbackoff後にリトライ（プール枯渇防止）
        log::debug!(
            "SQLITE_BUSY on restore busy_timeout, retrying after 20ms: {:?}",
            e
        );
        sleep(Duration::from_millis(20)).await;

        // リトライ
        if let Err(retry_err) = set_busy_timeout(&mut conn, original_timeout).await {
            log::warn!(
                "Failed to restore busy_timeout to original ({}ms) after retry, detaching: {:?}",
                original_timeout,
                retry_err
            );
            conn.detach();
        }
    } else {
        // 非BUSYエラー（IO障害等）は即座にdetach
        log::warn!(
            "Failed to restore busy_timeout to original ({}ms): {:?}, detaching connection from pool",
            original_timeout,
            e
        );
        conn.detach();
    }
}
```

**適用箇所**:
- `save_chunk_with_transaction_and_timeout` (リトライパス): L405-435
- `save_chunk_individually` (フォールバックパス): L697-726

**今後の対策**:
- エラー発生時は種類を区別する: 一時的（BUSY、ネットワーク）vs 恒久的（IO障害、poisoned）
- 一時的エラーには短いバックオフ + リトライで対応
- リトライ後も失敗する場合のみ接続を切り離す
- `conn.detach()`は「接続が完全に使用不能」な場合にのみ使用

### 45. beforeunloadとpagehideの二重発火によるbfcache無効化（高リスク指摘）

**指摘内容**:
- `pagehide`で`event.persisted`がtrueの場合は`cleanup()`をスキップするが、`beforeunload`は常に`cleanup()`を呼ぶ
- 両方発火するブラウザではbfcacheの意図が無効になる
- cleanup()後にwsがnullにならず、後続のロジックで閉じたソケットを"生きている"と誤認

**対応**:
```javascript
// Before（問題あり）:
window.addEventListener('pagehide', (event) => {
  if (!event.persisted) { cleanup(); }
});
window.addEventListener('beforeunload', cleanup); // 常に実行 → bfcache無効化

// After（修正済み）:
let cleanupCalled = false; // 二重実行防止フラグ

function cleanup() {
  if (cleanupCalled) return; // 二重実行防止
  cleanupCalled = true;
  isShuttingDown = true;
  // ...
  if (ws) {
    ws.onclose = null;
    ws.close();
    ws = null; // 閉じたソケットを"生きている"と誤認しないようnull化
  }
  // ...
}

// pageshow復元時
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    isShuttingDown = false;
    cleanupCalled = false; // 次回のcleanup()を有効にする
    // ...
  }
});
```

**今後の対策**:
- イベントリスナーが複数発火する可能性を考慮し、フラグで二重実行を防止
- cleanup()後はリソースをnull化して後続のロジックでの誤認を防止
- bfcache復元時はフラグをリセットして次回のcleanup()を有効にする

### 46. OBSリロード時のisShuttingDownリセット漏れ（高リスク指摘）

**指摘内容**:
- OBSブラウザソースのリロード時、ページは完全に再読み込みされるがJSの状態がリセットされない場合がある
- `isShuttingDown = true`のままだと再接続が抑制される

**対応**:
```javascript
function connectWebSocket() {
  // OBSリロード時などでisShuttingDownがtrueのまま呼ばれる場合に備えてリセット
  // ※ connectWebSocket()が呼ばれる = 接続を意図しているため、シャットダウン状態を解除
  isShuttingDown = false;
  cleanupCalled = false;

  ws = new WebSocket(WS_URL);
  // ...
}
```

**今後の対策**:
- 接続開始時に状態フラグをリセットする（接続意図が明確なため）
- シャットダウンフラグはグローバル状態であることを意識し、適切なタイミングでリセット

### 47. スキップログへのメッセージ数追加（ログ品質）

**指摘内容**:
- `save_chunk_with_retry`のスキップログにメッセージ数が含まれていなかった
- サイレントなデータ損失の診断が困難

**対応**:
```rust
// Before（問題あり）:
log::warn!(
    "SQLITE_BUSY: Max attempts ({}) exceeded, giving up",
    MAX_ATTEMPTS
);

// After（修正済み）:
log::warn!(
    "SQLITE_BUSY: Max attempts ({}) exceeded, giving up ({} messages skipped)",
    MAX_ATTEMPTS,
    messages.len()
);
```

**適用箇所**:
- save_chunk_with_retry内のすべてのスキップログ（5箇所）

**今後の対策**:
- データを処理する関数のスキップログには必ず処理対象の件数を含める
- スキップの原因（タイムアウト、リトライ超過等）と影響（件数）を明確にログ出力
- ログレベルはwarnを使用し、運用時に検知可能にする

### 48. beforeunloadがbfcacheを無効化する問題（高リスク指摘）

**指摘内容**:
- `beforeunload`イベントで常に`cleanup()`を呼ぶと、bfcache対応ブラウザでもbfcacheが無効化される
- `pagehide`で`event.persisted`をチェックしてもbeforeunloadが先に発火するため意味がない
- bfcacheからの復元後にシャットダウン状態のまま再接続できなくなる

**対応**:
```javascript
// Before（問題あり）:
window.addEventListener('pagehide', (event) => {
  if (!event.persisted) { cleanup(); }
});
window.addEventListener('beforeunload', cleanup); // ← bfcacheを無効化

// After（修正済み）:
window.addEventListener('pagehide', (event) => {
  // event.persisted=true: bfcacheに保存される → クリーンアップしない
  // event.persisted=false: 通常のページ離脱（OBS含む） → クリーンアップ
  if (!event.persisted) {
    cleanup();
  }
});
// beforeunloadは登録しない（bfcacheを無効化しないため）
// OBSブラウザソース等のbfcache非対応環境でもpagehide(persisted=false)で確実にクリーンアップ
```

**今後の対策**:
- bfcache対応が必要な場合、`beforeunload`でのリソース解放は避ける
- `pagehide`イベントの`event.persisted`でbfcache判定し、falseの場合のみクリーンアップ
- bfcache非対応環境（OBS等）では`pagehide`のpersisted=falseで確実にクリーンアップされる

### 49. destroy()後のインスタンスがinertになる問題（高リスク指摘）

**指摘内容**:
- `UpdateBatcher.destroy()`後、インスタンスはnullにならないが機能しなくなる（inert状態）
- `pageshow`で`!updateBatcher`チェックでは検出できず、バッチ処理が再開されない
- `DensityManager`には`isDestroyed()`があるが、`UpdateBatcher`にはなかった

**対応**:
```javascript
// UpdateBatcherにisDestroyed()を追加
destroy() {
  this.clear();
  this._destroyed = true;
}

isDestroyed() {
  return this._destroyed === true;
}

// pageshow復元時のチェック
if (!updateBatcher || updateBatcher.isDestroyed()) {
  updateBatcher = new UpdateBatcher({ batchInterval: 150 });
}
```

**今後の対策**:
- `destroy()`メソッドを持つクラスには必ず`isDestroyed()`も実装する
- 外部からインスタンスの状態を確認できるようにする（カプセル化）
- 復元処理ではnullチェックだけでなく`isDestroyed()`も確認する

### 50. トランザクション内での遅いI/Oによるデッドライン超過（将来タスク）

**指摘内容**:
- 2秒の総予算は`save_chunk_with_retry`ループで管理されているが、トランザクション内の遅いI/Oは制限されない
- 遅いディスクI/O（SQLITE_BUSYなし）で2秒を超える可能性がある

**対応（将来タスクとして記録）**:
- `docs/900_tasks.md`に「トランザクション内でのデッドライン強制」として記録
- 優先度: 低（遅いディスクI/Oは稀なケース）

**今後の対策**:
- リアルタイム保証が必要な場合は、トランザクション内でもデッドラインをチェックする
- 各操作前にデッドライン超過チェックを挿入し、超過時はrollbackして早期終了
- SQLite progress handlerの使用も検討

### 51. save_comments_to_dbの構造化戻り値（繰り返し指摘）

**指摘内容**（複数回のレビューで指摘）:
- `save_comments_to_db`が2秒の総予算でスキップされたチャンクを呼び出し元に通知せず`()`を返す
- 上流が損失を検出・補償できない
- 非BUSYパスでもデータがドロップされる可能性がある

**対応状況**:
- `docs/900_tasks.md`の「save_comments_to_dbの総予算によるデータスキップ検討」として記録済み
- 本PRではリアルタイム性を優先し、本番運用後にフィードバックを収集する設計判断
- スキップ発生時は`log::warn`でメッセージ数を出力済み

**今後の対策**:
- 本番運用でスキップが頻発する場合は構造化戻り値を実装
- 戻り値: `SaveResult { saved: usize, failed: usize, skipped: usize }`
- 予算を設定可能にする（メッセージ数/チャンク数に比例）

### 52. isShuttingDownの永続化によるリコネクト抑制（高リスク指摘）

**指摘内容**:
- `cleanup()`で`isShuttingDown=true`を設定し、`connectWebSocket()`でリセットするが、`pageshow`はbfcache時のみ
- 通常のunload/reloadで`connectWebSocket()`より前にエラーが発生すると、シャットダウン状態が永続化
- 再接続が永久に抑制される可能性

**対応**:
```javascript
// 初期化セクションの先頭で防御的リセット
// ===== 初期化 =====
// 防御的リセット: 初期化前にシャットダウン状態をクリア
// リロード時やbfcache非対応環境で、前回のシャットダウン状態が残っている可能性がある
// connectWebSocket()より前にエラーが発生しても、再接続が抑制されないようにする
isShuttingDown = false;

applyUrlSettings();
fetchAndApplySettings();
applyComponentSettings();
connectWebSocket();
```

**今後の対策**:
- グローバル状態フラグは初期化時にリセットする
- 関数呼び出しに依存したリセットは避ける（エラー時に呼ばれない可能性）
- 防御的プログラミング: 最悪のケース（早期エラー）を考慮

### 53. busy_timeout復元のリトライ回数（将来タスク）

**指摘内容**:
- 現在: 20msバックオフ後1回リトライ、失敗時はdetach
- 問題: 高負荷時にBUSYが連続すると接続をchurnしてプール容量が減少

**対応（将来タスクとして記録）**:
- `docs/900_tasks.md`に「busy_timeout復元のBUSYリトライ回数増加検討」として記録
- 2-3回のリトライループに変更を検討（例: 20ms, 40ms, 80ms）
- 優先度: 低（本番運用でBUSY頻発時に検討）

**今後の対策**:
- リトライ回数は本番運用のメトリクスに基づいて調整
- detachによるプール容量減少を監視

### 54. 繰り返しレビュー指摘のまとめ（PR Approve後）

**状況**:
- PRは5件のレビューコメント全てでApprove推奨
- `.codex/review.md`で繰り返し指摘される2つの高リスク問題は将来タスクとして記録済み

**繰り返し指摘される内容**:
1. **`save_comments_to_db`の構造化戻り値**: `{ saved, failed, skipped }`を返すべき
   - 現状: `()`を返し、スキップ時はwarnログのみ
   - 将来タスク: `docs/900_tasks.md`に記録済み（優先度: 中）

2. **busy_timeout復元のBUSYリトライ回数**: 1回→2-3回に増加すべき
   - 現状: 20msバックオフ後1回リトライ、失敗時detach
   - 将来タスク: `docs/900_tasks.md`に記録済み（優先度: 低）

**軽微な改善提案（PRコメントより）**:
- `original_timeout == 0`のコメント追加（設計判断の明記）
- `UpdateBatcher._destroyed`の明示的初期化（`= false`をコンストラクタに）
- `DensityManager.isDestroyed()`の一貫性（`_destroyed`フラグ使用を検討）
- テスト用DBファイルの`tempfile`クレート使用（将来タスクに記録済み）

**今後の対策**:
- 将来タスクとして記録済みの指摘は、対応時期まで繰り返し指摘される可能性がある
- 優先度と本番運用での影響を考慮して対応タイミングを判断
- 本番リリース前チェックリストで優先度を再評価

### 55. busy_timeout=0のfail-fast動作変更（設計判断）

**指摘内容**:
- SQLiteの`busy_timeout=0`は「即座にBUSYエラーを返す（fail-fast）」という意味
- 現在の実装では`original_timeout == 0`を「制限なし」として`u64::MAX`扱い → 500msにクランプ
- オペレーターが意図的に0を設定した場合、動作が変わる（fail-fast → 500ms待機）

**コード**:
```rust
// 現在の実装
let effective_original = if original_timeout == 0 {
    u64::MAX  // ← 0は「制限なし」として扱い、MAXに変換
} else {
    original_timeout
};

let busy_timeout_ms = effective_original
    .min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS)  // 500ms
    .min(remaining_after_acquire.as_millis() as u64);
```

**設計判断**:
- 現在の実装は「リトライを優先する」設計に基づく
- プール設定で`busy_timeout=0`を使うケースは稀
- このアプリはリアルタイム性よりデータ保存を優先する設計
- 将来タスクとして記録し、必要に応じて対応

**対応（将来タスクとして記録）**:
- `docs/900_tasks.md`に「busy_timeout=0のfail-fast動作保持検討」として記録
- Option A: 0は0として扱い、busy_timeout=0で即座にBUSYを返す
- Option B: 設定可能な`min_busy_timeout_ms`を追加し、0を上書きするかを制御
- 優先度: 低

**今後の対策**:
- 元の値の意味（semantic）を変更する場合は設計判断として明示する
- SQLiteのPRAGMA設定値は、その本来の意味を理解した上で扱う
- 0は「制限なし」か「即座にエラー」かはコンテキスト依存（busy_timeoutでは後者）

## 参照
- PR #56: https://github.com/manaca0923/vtuber-overlay-suite/pull/56
- SQLite Result Codes: https://www.sqlite.org/rescode.html
- SQLITE_BUSY (5): https://www.sqlite.org/rescode.html#busy
- SQLITE_LOCKED (6): https://www.sqlite.org/rescode.html#locked
- SQLite PRAGMA busy_timeout: https://www.sqlite.org/pragma.html#pragma_busy_timeout
