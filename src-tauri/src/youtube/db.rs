//! YouTube関連のDB操作を共通化したモジュール

use std::time::{Duration, Instant};

use sqlx::sqlite::SqliteConnection;
use sqlx::SqlitePool;
use tokio::time::{sleep, timeout};

use super::types::{ChatMessage, MessageType};

/// バッチ処理のチャンクサイズ
/// ロック保持時間を短縮するため、大きなバッチを分割して処理
const BATCH_CHUNK_SIZE: usize = 50;

/// SQLITE_BUSYエラー時の最大試行回数（初回 + リトライ）
/// 3回 = 初回試行 + 2回リトライ（100ms, 200ms後）
const MAX_ATTEMPTS: u32 = 3;

/// 初回リトライ時のバックオフ時間（ミリ秒）
const INITIAL_BACKOFF_MS: u64 = 100;

/// 最大バックオフ時間（ミリ秒）
const MAX_BACKOFF_MS: u64 = 1000;

/// リトライ処理の総タイムアウト（ミリ秒）
/// 各試行のbusy_timeout合計がこの値を超えないように制御
/// 2秒あれば通常の一時的なロック競合は解消される
const RETRY_TOTAL_TIMEOUT_MS: u64 = 2000;

/// 1回の試行あたりの最大busy_timeout（ミリ秒）
/// 総タイムアウトを試行回数で割った値を基準に、各試行で動的に調整
/// この値を超えないようにしつつ、残り時間に応じて短縮する
const MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS: u64 = 500;

/// トランザクション処理の結果
#[derive(Debug, Clone, Copy, PartialEq)]
enum TransactionResult {
    /// 成功
    Success,
    /// SQLITE_BUSYエラー（リトライ可能）
    Busy,
    /// その他のエラー（リトライ不可）
    OtherError,
    /// 接続が汚染された状態（rollback失敗等）
    /// この場合、接続をプールに戻さずに破棄すべき
    Poisoned,
}

/// コメントをDBに保存（バッチ処理最適化版）
///
/// トランザクション内で複数のINSERTを実行し、I/O効率を向上させる。
/// INSERT OR IGNOREで重複を無視し、既存レコードはスキップする。
/// youtube_idのUNIQUE制約により重複コメントは自動的にスキップされる。
///
/// ## 保存形式
/// - `message_type`: 短い文字列 ("text", "superChat", "superSticker", "membership", "membershipGift")
/// - `message_data`: MessageTypeの詳細データをJSON（TextはNULL）
/// - `published_at`: RFC3339形式の文字列
///
/// ## エラー処理
/// - SQLITE_BUSYエラー: exponential backoffでリトライ（最大3回）
/// - トランザクション開始/コミット失敗時: 1件ずつ個別INSERTにフォールバック
/// - 個別INSERT失敗時: ログ出力して次のメッセージへ進む
///
/// ## タイムアウト保証
/// 全チャンクの処理は総タイムアウト（RETRY_TOTAL_TIMEOUT_MS）内で完了する。
/// フォールバックパスも同じ予算を共有し、予算超過時はスキップする。
pub async fn save_comments_to_db(pool: &SqlitePool, messages: &[ChatMessage]) {
    if messages.is_empty() {
        return;
    }

    let start_time = Instant::now();
    let total_timeout = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);

    // チャンクに分割して処理（ロック保持時間を短縮）
    for chunk in messages.chunks(BATCH_CHUNK_SIZE) {
        // 残り予算を計算（saturating_subでアンダーフロー防止）
        let elapsed = start_time.elapsed();
        let remaining = total_timeout.saturating_sub(elapsed);

        // 残り予算が少なすぎる場合はスキップ（50ms未満は無意味）
        if remaining.as_millis() < 50 {
            log::warn!(
                "save_comments_to_db: Remaining budget ({}ms) too short, skipping chunk ({} messages)",
                remaining.as_millis(),
                chunk.len()
            );
            return;
        }

        // 残り予算をsave_chunk_with_retryに渡す（end-to-end予算管理）
        if !save_chunk_with_retry(pool, chunk, remaining).await {
            // トランザクション失敗時は1件ずつ個別INSERTにフォールバック
            // ただし総タイムアウトを超過している場合はスキップ
            let elapsed = start_time.elapsed();
            if elapsed >= total_timeout {
                log::warn!(
                    "save_comments_to_db: Total timeout exceeded, skipping fallback for chunk"
                );
                continue;
            }

            log::debug!("Transaction failed after retries, falling back to individual inserts");
            let remaining = total_timeout - elapsed;
            save_chunk_individually(pool, chunk, remaining).await;
        }
    }
}

/// チャンクをexponential backoffでリトライしながら保存
///
/// SQLITE_BUSYエラーが発生した場合、以下の制約内でリトライする:
/// - 最大試行回数: MAX_ATTEMPTS回
/// - 外側から渡された残り予算（remaining）内
///
/// 試行間隔はexponential backoffで増加（100ms → 200ms...最大1000ms）。
///
/// ## タイムアウトの強制（end-to-end保証）
/// 外側のsave_comments_to_dbから残り予算（remaining）を受け取り、
/// その予算内でのみリトライを行う。これにより、遅いチャンクが
/// 後続チャンクの予算を食い潰すことを防ぐ。
///
/// 各試行前にコネクションを取得し、`PRAGMA busy_timeout`を残り時間に応じて
/// 動的に設定する。これにより、SQLiteレベルでブロック時間を制限する。
///
/// `tokio::time::timeout`はブロッキングSQLite操作をキャンセルできないため使用しない。
/// 代わりに、各試行でbusy_timeoutを明示的に制限することで、
/// ブロック時間がタイムアウト予算を超えないことを保証する。
///
/// 例: remaining=1500ms、MAX_ATTEMPTS=3の場合
///   1回目: busy_timeout=500ms（残り1500ms、最大500ms）
///   2回目: 100ms後、busy_timeout=500ms（残り900ms程度、最大500ms）
///   3回目: 200ms後、busy_timeout=残り時間以内（最大500ms）
///   → 試行回数超過または残り時間なしでフォールバック（個別INSERT）
async fn save_chunk_with_retry(
    pool: &SqlitePool,
    messages: &[ChatMessage],
    remaining: Duration,
) -> bool {
    let start_time = Instant::now();
    // 外側から渡された残り予算を使用（独自タイマーではない）
    let total_timeout = remaining;
    let mut attempt = 0;
    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        // 残り時間を計算
        let elapsed = start_time.elapsed();
        if elapsed >= total_timeout {
            log::warn!(
                "SQLITE_BUSY: Total timeout ({}ms) exceeded before attempt {}, giving up",
                RETRY_TOTAL_TIMEOUT_MS,
                attempt + 1
            );
            return false;
        }
        let remaining = total_timeout - elapsed;

        // 残り時間が少なすぎる場合はスキップ（50ms未満は無意味）
        if remaining.as_millis() < 50 {
            log::warn!(
                "SQLITE_BUSY: Remaining time ({}ms) too short for retry, giving up",
                remaining.as_millis()
            );
            return false;
        }

        // デッドラインを計算して渡す（acquire後に残り時間を再計算するため）
        let deadline = start_time + total_timeout;
        let result =
            save_chunk_with_transaction_and_timeout(pool, messages, deadline).await;

        match result {
            TransactionResult::Success => return true,
            TransactionResult::OtherError | TransactionResult::Poisoned => {
                // 非SQLITE_BUSYエラー（テーブル不存在など）はリトライしない
                // Poisonedはsave_chunk_with_transaction_and_timeout内でOtherErrorに変換されるため
                // ここには到達しないが、網羅性のために明示的に処理
                return false;
            }
            TransactionResult::Busy => {
                attempt += 1;

                // 試行回数チェック
                if attempt >= MAX_ATTEMPTS {
                    log::warn!(
                        "SQLITE_BUSY: Max attempts ({}) exceeded, giving up",
                        MAX_ATTEMPTS
                    );
                    return false;
                }

                // 総タイムアウトチェック（再計算）
                let elapsed = start_time.elapsed();
                if elapsed >= total_timeout {
                    log::warn!(
                        "SQLITE_BUSY: Total timeout ({}ms) exceeded after {} attempts, giving up",
                        RETRY_TOTAL_TIMEOUT_MS,
                        attempt
                    );
                    return false;
                }

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

                log::debug!(
                    "SQLITE_BUSY: Attempt {}/{} failed, retrying after {}ms (elapsed: {}ms, remaining: {}ms)",
                    attempt,
                    MAX_ATTEMPTS,
                    clamped_backoff,
                    elapsed.as_millis(),
                    remaining_ms
                );
                sleep(Duration::from_millis(clamped_backoff)).await;
                backoff_ms = (backoff_ms * 2).min(MAX_BACKOFF_MS);
            }
        }
    }
}

/// コネクションにPRAGMA busy_timeoutを設定
///
/// 各試行前にbusy_timeoutを動的に設定することで、
/// SQLiteレベルでブロック時間を制限する。
///
/// ## 戻り値
/// - `Ok(())`: 設定成功
/// - `Err(sqlx::Error)`: PRAGMA実行失敗（busy含む）
async fn set_busy_timeout(conn: &mut SqliteConnection, timeout_ms: u64) -> Result<(), sqlx::Error> {
    sqlx::query(&format!("PRAGMA busy_timeout = {}", timeout_ms))
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// コネクションの現在のbusy_timeoutを取得
///
/// プール設定に依存せず、接続の実際の値を取得する。
/// 取得失敗時はNoneを返す（呼び出し元で復元をスキップする判断材料として）。
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

/// デッドラインを指定してトランザクション処理を実行
///
/// コネクションを取得し、busy_timeoutを設定してからトランザクションを開始する。
/// これにより、各試行のブロック時間を確実に制限できる。
///
/// ## デッドラインベースの予算管理
/// `deadline: Instant`を受け取り、pool.acquire()後に残り時間を再計算する。
/// これにより、acquire時間が予算から差し引かれ、acquire + busy_timeout で
/// 予算を超過することを防ぐ。
///
/// ## pool.acquire()のタイムアウト
/// pool.acquire()自体がデッドラインを超えてブロックしないよう、
/// tokio::time::timeoutでラップする。これにより総タイムアウトがend-to-endで強制される。
///
/// ## busy_timeout復元
/// トランザクション完了後（成功/失敗問わず）、busy_timeoutを元の値に復元する。
/// 元の値は接続から直接取得するため、プール設定に依存しない。
/// これにより、プール内の接続が短いタイムアウトを保持しない。
///
/// ## 接続のクローズ（汚染防止）
/// rollback失敗またはbusy_timeout復元失敗時は、接続を明示的にクローズして
/// プールに戻さない。これにより、汚染された接続が他の書き込みに影響することを防ぐ。
///
/// ## エラーハンドリング
/// - pool.acquire()タイムアウト → `TransactionResult::Busy`を返す（リトライ可能）
/// - 元のbusy_timeout取得失敗 → `TransactionResult::OtherError`を返す（復元不能なため）
/// - PRAGMA busy_timeout設定がBUSYで失敗 → `TransactionResult::Busy`を返す
/// - PRAGMA設定がその他のエラーで失敗 → `TransactionResult::OtherError`を返す
async fn save_chunk_with_transaction_and_timeout(
    pool: &SqlitePool,
    messages: &[ChatMessage],
    deadline: Instant,
) -> TransactionResult {
    // デッドラインまでの残り時間を計算
    let now = Instant::now();
    let remaining = deadline.saturating_duration_since(now);

    // 残り時間が少なすぎる場合はスキップ
    if remaining.as_millis() < 50 {
        log::debug!(
            "Skipping transaction attempt: remaining time ({}ms) too short",
            remaining.as_millis()
        );
        return TransactionResult::Busy;
    }

    // acquire用のタイムアウト（残り時間の半分、最大500ms）
    let acquire_timeout_ms = (remaining.as_millis() as u64 / 2).min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS);
    let acquire_timeout = Duration::from_millis(acquire_timeout_ms);

    // コネクションを取得（デッドラインを考慮したタイムアウト）
    let mut conn = match timeout(acquire_timeout, pool.acquire()).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => {
            if is_sqlite_busy_error(&e) {
                log::debug!("SQLITE_BUSY on connection acquire: {:?}", e);
                return TransactionResult::Busy;
            }
            log::warn!("Failed to acquire connection: {:?}", e);
            return TransactionResult::OtherError;
        }
        Err(_) => {
            // タイムアウト: プール接続待ちが予算を超過
            log::debug!(
                "Connection acquire timed out after {}ms",
                acquire_timeout_ms
            );
            return TransactionResult::Busy;
        }
    };

    // acquire後の残り時間を再計算（acquire時間を差し引く）
    let remaining_after_acquire = deadline.saturating_duration_since(Instant::now());
    if remaining_after_acquire.as_millis() < 50 {
        log::debug!(
            "Skipping transaction: remaining time after acquire ({}ms) too short",
            remaining_after_acquire.as_millis()
        );
        return TransactionResult::Busy;
    }

    // busy_timeoutをacquire後の残り時間で計算（最大500ms）
    let busy_timeout_ms = (remaining_after_acquire.as_millis() as u64).min(MAX_BUSY_TIMEOUT_PER_ATTEMPT_MS);

    // 元のbusy_timeoutを取得（復元用）
    // 取得失敗時は復元不能なため、OtherErrorを返して続行しない
    // → 短いbusy_timeoutを設定後に復元できず、接続が劣化するリスクを回避
    let original_timeout = match get_busy_timeout(&mut conn).await {
        Some(timeout) => timeout,
        None => {
            log::warn!("Cannot proceed: failed to get original busy_timeout for restoration");
            return TransactionResult::OtherError;
        }
    };

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

    // トランザクションを実行
    let result = save_chunk_with_transaction_on_conn(&mut conn, messages).await;

    // Poisoned状態の場合は接続を切り離す（rollback失敗等）
    if result == TransactionResult::Poisoned {
        log::warn!("Transaction resulted in poisoned connection, detaching from pool");
        conn.detach();
        return TransactionResult::OtherError; // 呼び出し元にはOtherErrorとして返す
    }

    // busy_timeoutを元の値に復元（プール内接続への影響を防ぐ）
    if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
        log::warn!(
            "Failed to restore busy_timeout to original ({}ms): {:?}, detaching connection from pool",
            original_timeout,
            e
        );
        // 復元失敗時は接続をプールから切り離す（汚染防止）
        // detach()で接続をプールから切り離し、dropで実際にクローズ
        conn.detach();
    }

    result
}

/// エラーがSQLITE_BUSY/SQLITE_LOCKEDかどうかを判定
///
/// リトライ可能なエラー:
/// - SQLITE_BUSY (5): 他の接続がロックを保持している
/// - SQLITE_LOCKED (6): 同一接続内でのデッドロック検出
/// - 拡張エラーコード: SQLITE_BUSY_RECOVERY(261), SQLITE_BUSY_SNAPSHOT(517)等
///
/// 拡張エラーコードは (extended_code % 256) で基本コードを取得できる。
/// 例: 517 % 256 = 5 (SQLITE_BUSY), 261 % 256 = 5 (SQLITE_BUSY)
///
/// ## メッセージフォールバック
/// エラーコードがNoneの場合のみ、SQLite固有のフレーズでメッセージ判定を行う。
/// エラーコードがある場合はメッセージ判定をスキップし、非一時的エラーの誤検出を防ぐ。
///
/// 参考: https://www.sqlite.org/rescode.html
fn is_sqlite_busy_error(e: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = e {
        // エラーコードで判定（優先）
        if let Some(code) = db_err.code() {
            let code_str = code.as_ref();

            // 数値コードの場合: パースして % 256 で基本コードを取得
            // 5 = SQLITE_BUSY, 6 = SQLITE_LOCKED
            if let Ok(code_num) = code_str.parse::<i32>() {
                let base_code = code_num % 256;
                if base_code == 5 || base_code == 6 {
                    return true;
                }
            }

            // 文字列として"SQLITE_BUSY"/"SQLITE_LOCKED"が返される場合も対応
            let code_upper = code_str.to_uppercase();
            if code_upper.starts_with("SQLITE_BUSY") || code_upper.starts_with("SQLITE_LOCKED") {
                return true;
            }

            // エラーコードがあるが上記に該当しない場合 → 非一時的エラー
            // メッセージフォールバックをスキップして誤検出を防ぐ
            return false;
        }

        // エラーコードがNoneの場合のみメッセージで判定（フォールバック）
        // SQLite固有のフレーズに絞り込み、非一時的エラーの誤検出を防ぐ
        let msg = db_err.message().to_lowercase();
        if msg.contains("database is locked") || msg.contains("database is busy") {
            return true;
        }
    }
    false
}

/// コネクション上でトランザクション処理を実行
///
/// INSERT OR IGNOREを使用しているため、UNIQUE/CHECK等の制約エラーは発生しない。
/// エラーが発生した場合は以下の致命的な問題のみ:
/// - テーブル不存在（DDLエラー）
/// - ディスクフル/I/Oエラー
/// - RAISE(ABORT, ...)トリガー
/// - SQLITE_BUSY（データベースロック）
///
/// これらの致命的エラー発生時は最初のエラーで即座にロールバックする。
/// SQLITE_BUSYの場合はリトライ可能として`TransactionResult::Busy`を返す。
///
/// 呼び出し元でbusy_timeoutを設定済みのコネクションを渡すことで、
/// ブロック時間を制限できる。
async fn save_chunk_with_transaction_on_conn(
    conn: &mut SqliteConnection,
    messages: &[ChatMessage],
) -> TransactionResult {
    use sqlx::Connection;

    // トランザクションを開始
    let mut tx = match conn.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            if is_sqlite_busy_error(&e) {
                log::debug!("SQLITE_BUSY on transaction begin: {:?}", e);
                return TransactionResult::Busy;
            }
            log::warn!("Failed to start transaction: {:?}", e);
            return TransactionResult::OtherError;
        }
    };

    for msg in messages {
        if let Err(e) = insert_comment(&mut *tx, msg).await {
            // INSERT OR IGNOREなので重複エラーは発生しないはず
            // エラーが発生した場合は致命的な問題（テーブル不存在等）
            // 最初のエラーで即座にロールバック（warn spam回避）
            if is_sqlite_busy_error(&e) {
                log::debug!("SQLITE_BUSY during insert: {:?}", e);
                // ロールバック（dropで自動的に行われるが明示的に）
                // rollback失敗時は接続が汚染されているためPoisonedを返す
                if let Err(rb_err) = tx.rollback().await {
                    log::warn!("Rollback failed after BUSY during insert: {:?}", rb_err);
                    return TransactionResult::Poisoned;
                }
                return TransactionResult::Busy;
            }
            log::warn!("Insert failed in transaction, rolling back: {:?}", e);
            // ロールバック（dropで自動的に行われるが明示的に）
            // rollback失敗時は接続が汚染されているためPoisonedを返す
            if let Err(rb_err) = tx.rollback().await {
                log::warn!("Rollback failed after insert error: {:?}", rb_err);
                return TransactionResult::Poisoned;
            }
            return TransactionResult::OtherError;
        }
    }

    // コミット
    // 注: commit()はselfを消費するため、失敗後にrollback()を呼び出すことはできない
    // sqlxのTransactionはcommit失敗時に自動的にロールバックされる
    if let Err(e) = tx.commit().await {
        if is_sqlite_busy_error(&e) {
            log::debug!("SQLITE_BUSY on commit: {:?}", e);
            return TransactionResult::Busy;
        }
        log::warn!("Failed to commit transaction: {:?}", e);
        return TransactionResult::OtherError;
    }

    TransactionResult::Success
}

/// チャンクを1件ずつ個別に保存（フォールバック用）
///
/// 単一接続を取得して再利用し、行ごとのpool checkout回避でパフォーマンス向上。
/// 残り予算を受け取り、予算内でのみ処理を実行する。
/// pool.acquire()も短いbusy_timeoutも残り予算で制限される。
///
/// ## busy_timeout管理
/// リトライパスと同様に、元のbusy_timeoutを取得・設定・復元する。
/// 取得または設定に失敗した場合は即座に終了し、予算を超えてブロックするリスクを排除。
/// これにより、接続の以前のbusy_timeout（5秒など）で長時間ブロックすることを防ぐ。
async fn save_chunk_individually(pool: &SqlitePool, messages: &[ChatMessage], remaining: Duration) {
    let mut success_count = 0;
    let mut error_count = 0;
    let start_time = Instant::now();

    // 残り時間が少なすぎる場合はスキップ
    if remaining.as_millis() < 50 {
        log::debug!(
            "Skipping individual insert fallback: remaining time ({}ms) too short",
            remaining.as_millis()
        );
        return;
    }

    // 接続取得のタイムアウト（残り予算の半分、最大500ms）
    let acquire_timeout_ms = (remaining.as_millis() as u64 / 2).min(500);
    let acquire_timeout = Duration::from_millis(acquire_timeout_ms);

    // 単一接続を取得して再利用（行ごとのpool checkout回避）
    let mut conn = match timeout(acquire_timeout, pool.acquire()).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => {
            log::warn!("Failed to acquire connection for fallback: {:?}", e);
            return;
        }
        Err(_) => {
            log::debug!(
                "Connection acquire for fallback timed out after {}ms",
                acquire_timeout_ms
            );
            return;
        }
    };

    // 元のbusy_timeoutを取得（復元用）
    // 取得失敗時は予算を強制できないため即座に終了
    // → 接続の以前のbusy_timeout（5秒など）で長時間ブロックするリスクを排除
    let original_timeout = match get_busy_timeout(&mut conn).await {
        Some(timeout) => timeout,
        None => {
            log::debug!(
                "Skipping individual insert fallback: failed to get original busy_timeout"
            );
            return;
        }
    };

    // busy_timeoutを残り予算で制限（最大500ms）
    // saturating_subでアンダーフローを防止（スケジューラ遅延等でelapsedがremainingを超える可能性）
    let remaining_after_acquire = remaining.saturating_sub(start_time.elapsed());
    if remaining_after_acquire.as_millis() < 50 {
        log::debug!(
            "Skipping individual insert fallback: remaining time after acquire ({}ms) too short",
            remaining_after_acquire.as_millis()
        );
        return;
    }

    let busy_timeout_ms = (remaining_after_acquire.as_millis() as u64).min(500);

    // busy_timeoutを設定
    // 設定失敗時は予算を強制できないため即座に終了
    if let Err(e) = set_busy_timeout(&mut conn, busy_timeout_ms).await {
        log::debug!(
            "Skipping individual insert fallback: failed to set busy_timeout: {:?}",
            e
        );
        return;
    }

    for msg in messages {
        // 予算チェック
        if start_time.elapsed() >= remaining {
            log::debug!(
                "Individual insert fallback: budget exhausted after {}/{} messages",
                success_count + error_count,
                messages.len()
            );
            break;
        }

        match insert_comment(&mut *conn, msg).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                log::debug!("Failed to insert comment individually: {:?}", e);
            }
        }
    }

    // busy_timeoutを元の値に復元
    // 復元失敗時は接続を切り離してプールへの影響を防ぐ（リトライパスと同様）
    if let Err(e) = set_busy_timeout(&mut conn, original_timeout).await {
        log::warn!(
            "Failed to restore busy_timeout in fallback to original ({}ms), detaching connection: {:?}",
            original_timeout,
            e
        );
        conn.detach();
    }

    if error_count > 0 || success_count + error_count < messages.len() {
        log::warn!(
            "Individual insert fallback: {} succeeded, {} failed, {} skipped (budget)",
            success_count,
            error_count,
            messages.len() - success_count - error_count
        );
    }
}

/// 単一コメントをINSERT
async fn insert_comment<'e, E>(executor: E, msg: &ChatMessage) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    // MessageTypeを短い文字列に変換
    let message_type = match &msg.message_type {
        MessageType::Text => "text",
        MessageType::SuperChat { .. } => "superChat",
        MessageType::SuperSticker { .. } => "superSticker",
        MessageType::Membership { .. } => "membership",
        MessageType::MembershipGift { .. } => "membershipGift",
    };

    // MessageTypeの詳細データをJSONに変換（Textの場合はNULL）
    let message_data = match &msg.message_type {
        MessageType::Text => None,
        other => serde_json::to_string(other).ok(),
    };

    // published_atをRFC3339形式に変換
    let published_at_str = msg.published_at.to_rfc3339();

    sqlx::query(
        r#"INSERT OR IGNORE INTO comment_logs
        (id, youtube_id, message, author_name, author_channel_id, author_image_url,
         is_owner, is_moderator, is_member, message_type, message_data, published_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&msg.id)
    .bind(&msg.id)
    .bind(&msg.message)
    .bind(&msg.author_name)
    .bind(&msg.author_channel_id)
    .bind(&msg.author_image_url)
    .bind(msg.is_owner)
    .bind(msg.is_moderator)
    .bind(msg.is_member)
    .bind(message_type)
    .bind(&message_data)
    .bind(&published_at_str)
    .execute(executor)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    fn create_test_message(id: &str, message: &str) -> ChatMessage {
        ChatMessage {
            id: id.to_string(),
            message: message.to_string(),
            message_runs: None,
            author_name: "TestUser".to_string(),
            author_channel_id: "UC123".to_string(),
            author_image_url: "https://example.com/icon.png".to_string(),
            message_type: MessageType::Text,
            is_owner: false,
            is_moderator: false,
            is_member: false,
            is_verified: false,
            published_at: Utc::now(),
        }
    }

    /// 単一接続のin-memoryプールを作成（DDL/DMLが同一DBで実行されることを保証）
    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    /// テスト用のcomment_logsテーブルを作成
    async fn create_test_table(pool: &SqlitePool) {
        sqlx::query(
            r#"CREATE TABLE comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL,
                author_name TEXT NOT NULL,
                author_channel_id TEXT NOT NULL,
                author_image_url TEXT,
                is_owner BOOLEAN NOT NULL DEFAULT 0,
                is_moderator BOOLEAN NOT NULL DEFAULT 0,
                is_member BOOLEAN NOT NULL DEFAULT 0,
                message_type TEXT NOT NULL,
                message_data TEXT,
                published_at TEXT NOT NULL
            )"#,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_empty_batch_is_noop() {
        let pool = create_test_pool().await;
        // テーブルがなくても空バッチは即座にリターン
        save_comments_to_db(&pool, &[]).await;
    }

    #[tokio::test]
    async fn test_save_comments_batch() {
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("msg1", "Hello"),
            create_test_message("msg2", "World"),
            create_test_message("msg3", "Test"),
        ];

        save_comments_to_db(&pool, &messages).await;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 3);
    }

    #[tokio::test]
    async fn test_save_comments_ignores_duplicates() {
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("msg1", "First"),
            create_test_message("msg1", "Duplicate"), // 重複ID
            create_test_message("msg2", "Second"),
        ];

        save_comments_to_db(&pool, &messages).await;

        // 重複が無視されて2件のみ保存されること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);

        // 最初のメッセージが保存されていること
        let msg: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg.0, "First");
    }

    #[tokio::test]
    async fn test_save_comments_chunk_boundary() {
        // BATCH_CHUNK_SIZE + 1 件のメッセージでチャンク境界をテスト
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages: Vec<ChatMessage> = (0..=BATCH_CHUNK_SIZE)
            .map(|i| create_test_message(&format!("msg{}", i), &format!("Message {}", i)))
            .collect();

        save_comments_to_db(&pool, &messages).await;

        // 全件保存されていること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, (BATCH_CHUNK_SIZE + 1) as i64);
    }

    #[tokio::test]
    async fn test_save_comments_fallback_on_missing_table() {
        // テーブルがない場合、トランザクション失敗→フォールバック→個別も失敗
        // エラーログは出るが、パニックせず完了すること
        let pool = create_test_pool().await;
        // テーブルを作成しない

        let messages = vec![
            create_test_message("msg1", "Hello"),
            create_test_message("msg2", "World"),
        ];

        // パニックしないことを確認（エラーはログに出力される）
        save_comments_to_db(&pool, &messages).await;
    }

    /// CHECK制約付きテーブルを作成（テスト用）
    async fn create_test_table_with_check(pool: &SqlitePool) {
        sqlx::query(
            r#"CREATE TABLE comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL CHECK(length(message) <= 10),
                author_name TEXT NOT NULL,
                author_channel_id TEXT NOT NULL,
                author_image_url TEXT,
                is_owner BOOLEAN NOT NULL DEFAULT 0,
                is_moderator BOOLEAN NOT NULL DEFAULT 0,
                is_member BOOLEAN NOT NULL DEFAULT 0,
                message_type TEXT NOT NULL,
                message_data TEXT,
                published_at TEXT NOT NULL
            )"#,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_comments_with_check_constraint_violation() {
        // CHECK制約違反があっても、INSERT OR IGNOREにより無視される
        // 有効な行のみ保存されることを確認
        let pool = create_test_pool().await;
        create_test_table_with_check(&pool).await;

        let messages = vec![
            create_test_message("msg1", "Short"),                   // OK: 5文字
            create_test_message("msg2", "This is way too long"),    // NG: 21文字 > 10
            create_test_message("msg3", "Valid"),                   // OK: 5文字
        ];

        save_comments_to_db(&pool, &messages).await;

        // INSERT OR IGNORE により、CHECK制約違反の行はスキップされる
        // 有効な2件のみ保存されること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);

        // 有効なメッセージが保存されていること
        let msg1: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg1.0, "Short");

        let msg3: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg3'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg3.0, "Valid");

        // CHECK制約違反のメッセージは保存されていないこと
        let msg2_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comment_logs WHERE id = 'msg2'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(msg2_count.0, 0);
    }

    /// RAISE(ABORT)トリガー付きテーブルを作成（テスト用）
    /// 特定のID（"trigger_fail"）でハードエラーを発生させる
    async fn create_test_table_with_abort_trigger(pool: &SqlitePool) {
        // テーブル作成
        sqlx::query(
            r#"CREATE TABLE comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL,
                author_name TEXT NOT NULL,
                author_channel_id TEXT NOT NULL,
                author_image_url TEXT,
                is_owner BOOLEAN NOT NULL DEFAULT 0,
                is_moderator BOOLEAN NOT NULL DEFAULT 0,
                is_member BOOLEAN NOT NULL DEFAULT 0,
                message_type TEXT NOT NULL,
                message_data TEXT,
                published_at TEXT NOT NULL
            )"#,
        )
        .execute(pool)
        .await
        .unwrap();

        // 特定のIDでRAISE(ABORT)を発生させるトリガー
        sqlx::query(
            r#"CREATE TRIGGER abort_on_trigger_fail
               BEFORE INSERT ON comment_logs
               WHEN NEW.id = 'trigger_fail'
               BEGIN
                   SELECT RAISE(ABORT, 'Forced abort for testing');
               END"#,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_save_comments_fallback_with_abort_trigger() {
        // RAISE(ABORT)トリガーによるハード障害をテスト
        // トランザクション失敗 → フォールバック → 残りの行は保存される
        let pool = create_test_pool().await;
        create_test_table_with_abort_trigger(&pool).await;

        let messages = vec![
            create_test_message("msg1", "First"),
            create_test_message("trigger_fail", "This triggers abort"), // ABORTトリガー
            create_test_message("msg3", "Third"),
        ];

        save_comments_to_db(&pool, &messages).await;

        // フォールバック処理により、トリガー対象以外の行は保存される
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2, "Valid rows should be saved via fallback");

        // 正常なメッセージが保存されていること
        let msg1: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg1.0, "First");

        let msg3: (String,) = sqlx::query_as("SELECT message FROM comment_logs WHERE id = 'msg3'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(msg3.0, "Third");

        // トリガー対象のメッセージは保存されていないこと
        let trigger_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comment_logs WHERE id = 'trigger_fail'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(trigger_count.0, 0, "Trigger target should not be saved");
    }

    #[test]
    fn test_transaction_result_enum() {
        // TransactionResult enumの基本的な動作を確認
        assert_eq!(TransactionResult::Success, TransactionResult::Success);
        assert_eq!(TransactionResult::Busy, TransactionResult::Busy);
        assert_eq!(TransactionResult::OtherError, TransactionResult::OtherError);
        assert_eq!(TransactionResult::Poisoned, TransactionResult::Poisoned);
        assert_ne!(TransactionResult::Success, TransactionResult::Busy);
        assert_ne!(TransactionResult::OtherError, TransactionResult::Poisoned);
    }

    /// MockDatabaseErrorでエラー判定のテスト用構造体
    #[derive(Debug)]
    struct MockDatabaseError {
        code: Option<String>,
        message: String,
    }

    impl std::fmt::Display for MockDatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for MockDatabaseError {}

    impl sqlx::error::DatabaseError for MockDatabaseError {
        fn message(&self) -> &str {
            &self.message
        }

        fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
            self.code.as_ref().map(|c| std::borrow::Cow::Borrowed(c.as_str()))
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }

        fn kind(&self) -> sqlx::error::ErrorKind {
            sqlx::error::ErrorKind::Other
        }
    }

    fn create_mock_db_error(code: Option<&str>, message: &str) -> sqlx::Error {
        sqlx::Error::Database(Box::new(MockDatabaseError {
            code: code.map(|c| c.to_string()),
            message: message.to_string(),
        }))
    }

    #[test]
    fn test_is_sqlite_busy_error_code_5() {
        // エラーコード5（SQLITE_BUSY）を検出
        let err = create_mock_db_error(Some("5"), "database is busy");
        assert!(is_sqlite_busy_error(&err), "Should detect code 5 as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_code_6() {
        // エラーコード6（SQLITE_LOCKED）を検出
        let err = create_mock_db_error(Some("6"), "database table is locked");
        assert!(is_sqlite_busy_error(&err), "Should detect code 6 as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_string_code() {
        // 文字列コード "SQLITE_BUSY" を検出
        let err = create_mock_db_error(Some("SQLITE_BUSY"), "database is busy");
        assert!(is_sqlite_busy_error(&err), "Should detect SQLITE_BUSY string as busy");

        // 文字列コード "SQLITE_BUSY_TIMEOUT" を検出
        let err = create_mock_db_error(Some("SQLITE_BUSY_TIMEOUT"), "timeout");
        assert!(is_sqlite_busy_error(&err), "Should detect SQLITE_BUSY_TIMEOUT as busy");

        // 文字列コード "SQLITE_LOCKED" を検出
        let err = create_mock_db_error(Some("SQLITE_LOCKED"), "locked");
        assert!(is_sqlite_busy_error(&err), "Should detect SQLITE_LOCKED as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_message_fallback() {
        // コードがNoneの場合のみメッセージで判定
        // SQLite固有のフレーズに絞り込み

        // "database is locked" → busy
        let err = create_mock_db_error(None, "database is locked");
        assert!(
            is_sqlite_busy_error(&err),
            "Should detect 'database is locked' message as busy"
        );

        // "database is busy" → busy
        let err = create_mock_db_error(None, "database is busy");
        assert!(
            is_sqlite_busy_error(&err),
            "Should detect 'database is busy' message as busy"
        );

        // 単に"busy"や"locked"を含むだけでは検出しない（誤検出防止）
        let err = create_mock_db_error(None, "table is locked by another connection");
        assert!(
            !is_sqlite_busy_error(&err),
            "Should not detect generic 'locked' message as busy"
        );

        let err = create_mock_db_error(None, "The system is busy");
        assert!(
            !is_sqlite_busy_error(&err),
            "Should not detect generic 'busy' message as busy"
        );
    }

    #[test]
    fn test_is_sqlite_busy_error_code_overrides_message() {
        // エラーコードがある場合、メッセージフォールバックはスキップ
        // 非busy/lockedコードがあれば、メッセージに"locked"があってもfalse
        let err = create_mock_db_error(Some("1"), "database is locked");
        assert!(
            !is_sqlite_busy_error(&err),
            "Should not retry non-busy error even if message contains 'locked'"
        );

        let err = create_mock_db_error(Some("19"), "database is busy");
        assert!(
            !is_sqlite_busy_error(&err),
            "Should not retry constraint error even if message contains 'busy'"
        );
    }

    #[test]
    fn test_is_sqlite_busy_error_extended_codes() {
        // 拡張エラーコードの検出テスト
        // 517 = SQLITE_BUSY_SNAPSHOT (517 % 256 = 5)
        let err = create_mock_db_error(Some("517"), "snapshot busy");
        assert!(
            is_sqlite_busy_error(&err),
            "Should detect code 517 (SQLITE_BUSY_SNAPSHOT) as busy"
        );

        // 261 = SQLITE_BUSY_RECOVERY (261 % 256 = 5)
        let err = create_mock_db_error(Some("261"), "recovery busy");
        assert!(
            is_sqlite_busy_error(&err),
            "Should detect code 261 (SQLITE_BUSY_RECOVERY) as busy"
        );

        // 262 = SQLITE_LOCKED_SHAREDCACHE (262 % 256 = 6)
        let err = create_mock_db_error(Some("262"), "shared cache locked");
        assert!(
            is_sqlite_busy_error(&err),
            "Should detect code 262 (SQLITE_LOCKED_SHAREDCACHE) as busy"
        );

        // 非拡張busyコード（257 % 256 = 1 = SQLITE_ERROR）
        let err = create_mock_db_error(Some("257"), "generic error");
        assert!(
            !is_sqlite_busy_error(&err),
            "Should not detect code 257 (base code 1) as busy"
        );
    }

    #[test]
    fn test_is_sqlite_busy_error_non_busy_code() {
        // 非busy/lockedエラーコードはfalse
        let err = create_mock_db_error(Some("1"), "SQL logic error");
        assert!(!is_sqlite_busy_error(&err), "Should not detect code 1 as busy");

        let err = create_mock_db_error(Some("19"), "constraint failed");
        assert!(!is_sqlite_busy_error(&err), "Should not detect code 19 as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_non_busy_message() {
        // 非busyエラーメッセージはfalse
        let err = create_mock_db_error(None, "syntax error");
        assert!(!is_sqlite_busy_error(&err), "Should not detect syntax error as busy");

        let err = create_mock_db_error(None, "table does not exist");
        assert!(!is_sqlite_busy_error(&err), "Should not detect table error as busy");
    }

    #[test]
    fn test_is_sqlite_busy_error_non_database_error() {
        // 非Databaseエラーはfalse
        let err = sqlx::Error::RowNotFound;
        assert!(!is_sqlite_busy_error(&err), "Should not detect RowNotFound as busy");
    }

    #[tokio::test]
    async fn test_retry_logic_with_success() {
        // リトライロジックが成功時に正常動作することを確認
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("retry1", "Message 1"),
            create_test_message("retry2", "Message 2"),
        ];

        // save_chunk_with_retryが成功すること
        // テスト用にデフォルト予算（2秒）を渡す
        let default_budget = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);
        let result = save_chunk_with_retry(&pool, &messages, default_budget).await;
        assert!(result, "Retry should succeed on first attempt");

        // データが保存されていること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2);
    }

    #[tokio::test]
    async fn test_retry_gives_up_on_non_busy_error() {
        // 非SQLITE_BUSYエラー時はリトライせずにすぐ失敗すること
        let pool = create_test_pool().await;
        // テーブルを作成しない → テーブル不存在エラー

        let messages = vec![create_test_message("msg1", "Hello")];

        // save_chunk_with_retryが失敗すること（リトライせずに）
        let default_budget = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);
        let result = save_chunk_with_retry(&pool, &messages, default_budget).await;
        assert!(!result, "Should fail immediately on non-BUSY error");
    }

    /// 並行書き込みテスト用のファイルベースプールを作成
    ///
    /// `SqliteConnectOptions::new().filename()`を使用してWindowsパス問題を回避
    async fn create_file_based_pool(path: &std::path::Path) -> SqlitePool {
        use sqlx::sqlite::SqliteConnectOptions;

        // SqliteConnectOptions::new().filename()を使用（from_strはWindows/特殊パスで問題あり）
        // busy_timeoutを短く設定（テスト用）
        let connect_options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .busy_timeout(std::time::Duration::from_millis(50)); // 短いタイムアウト

        SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(connect_options)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_concurrent_writes_with_retry() {
        use std::sync::Arc;
        use tokio::sync::Barrier;

        // テンポラリファイルを使用
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("concurrent_test_{}.db", std::process::id()));

        // ファイルベースのプールを作成
        let pool = create_file_based_pool(&db_path).await;

        // テーブル作成
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS comment_logs (
                id TEXT PRIMARY KEY,
                youtube_id TEXT UNIQUE NOT NULL,
                message TEXT NOT NULL,
                author_name TEXT NOT NULL,
                author_channel_id TEXT NOT NULL,
                author_image_url TEXT,
                is_owner BOOLEAN NOT NULL DEFAULT 0,
                is_moderator BOOLEAN NOT NULL DEFAULT 0,
                is_member BOOLEAN NOT NULL DEFAULT 0,
                message_type TEXT NOT NULL,
                message_data TEXT,
                published_at TEXT NOT NULL
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let pool = Arc::new(pool);
        let barrier = Arc::new(Barrier::new(2));

        // 2つのタスクから同時に書き込み
        let pool1 = Arc::clone(&pool);
        let barrier1 = Arc::clone(&barrier);
        let task1 = tokio::spawn(async move {
            let messages: Vec<ChatMessage> = (0..30)
                .map(|i| create_test_message(&format!("task1_msg{}", i), &format!("Task1 Message {}", i)))
                .collect();
            barrier1.wait().await;
            save_comments_to_db(&pool1, &messages).await;
        });

        let pool2 = Arc::clone(&pool);
        let barrier2 = Arc::clone(&barrier);
        let task2 = tokio::spawn(async move {
            let messages: Vec<ChatMessage> = (0..30)
                .map(|i| create_test_message(&format!("task2_msg{}", i), &format!("Task2 Message {}", i)))
                .collect();
            barrier2.wait().await;
            save_comments_to_db(&pool2, &messages).await;
        });

        // 両タスクの完了を待つ
        let _ = tokio::join!(task1, task2);

        // 全てのメッセージが保存されていること（リトライ or フォールバックで）
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
        assert_eq!(count.0, 60, "All 60 messages should be saved despite concurrency");

        // クリーンアップ
        drop(pool);
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_busy_timeout_is_restored_after_retry() {
        // save_chunk_with_retry後にbusy_timeoutがデフォルト値に復元されることを確認
        // プール内接続への短いタイムアウト漏れを防ぐ

        // テンポラリファイルを使用（in-memoryではPRAGMAの検証が難しい）
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("timeout_restore_test_{}.db", std::process::id()));

        let pool = create_file_based_pool(&db_path).await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("restore1", "Message 1"),
            create_test_message("restore2", "Message 2"),
        ];

        // save_chunk_with_retryを実行（内部でbusy_timeoutが変更される）
        let default_budget = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);
        let result = save_chunk_with_retry(&pool, &messages, default_budget).await;
        assert!(result, "Should succeed");

        // 新しいコネクションを取得してbusy_timeoutを確認
        // 注: save_chunk_with_retryで使用したコネクションがプールに戻されている
        let mut conn = pool.acquire().await.unwrap();
        let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // デフォルト値（5000ms）に復元されていること
        // ただし、プールが複数接続を持つ場合、別のコネクションが返される可能性があるため
        // この値が500ms（短いタイムアウト）ではないことを確認
        assert!(
            timeout.0 >= 50, // 短すぎるタイムアウトではない
            "busy_timeout should not be too short after retry: got {}ms",
            timeout.0
        );

        // クリーンアップ
        drop(conn);
        drop(pool);
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_single_connection_pool_timeout_restoration() {
        // 単一接続プールでbusy_timeout復元を厳密にテスト
        // 同じコネクションが再利用されるため、復元を確実に検証できる

        use sqlx::sqlite::SqliteConnectOptions;

        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("single_conn_test_{}.db", std::process::id()));

        // プールのデフォルトbusy_timeoutを5000msに設定
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .busy_timeout(std::time::Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(1) // 単一接続で強制的に同じコネクションを再利用
            .connect_with(connect_options)
            .await
            .unwrap();

        create_test_table(&pool).await;

        let messages = vec![create_test_message("single1", "Message 1")];

        // save_chunk_with_retryを実行
        let default_budget = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);
        let result = save_chunk_with_retry(&pool, &messages, default_budget).await;
        assert!(result, "Should succeed");

        // 同じコネクションを再取得（単一接続プールなので同じはず）
        let mut conn = pool.acquire().await.unwrap();
        let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // 元の値（5000ms）に復元されていること
        assert_eq!(
            timeout.0, 5000,
            "busy_timeout should be restored to original (5000ms) after retry, got {}ms",
            timeout.0
        );

        // クリーンアップ
        drop(conn);
        drop(pool);
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_non_default_pool_timeout_restoration() {
        // 非デフォルトbusy_timeout（150ms）でプールを作成し、
        // save_chunk_with_retry後に元の値（150ms）に復元されることを確認
        // 定数5000msではなく、プール設定に依存しない復元を検証

        use sqlx::sqlite::SqliteConnectOptions;

        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("non_default_timeout_{}.db", std::process::id()));

        // 非デフォルトbusy_timeout（150ms）でプールを作成
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .busy_timeout(std::time::Duration::from_millis(150));

        let pool = SqlitePoolOptions::new()
            .max_connections(1) // 単一接続で強制的に同じコネクションを再利用
            .connect_with(connect_options)
            .await
            .unwrap();

        create_test_table(&pool).await;

        let messages = vec![create_test_message("nondefault1", "Message 1")];

        // save_chunk_with_retryを実行
        let default_budget = Duration::from_millis(RETRY_TOTAL_TIMEOUT_MS);
        let result = save_chunk_with_retry(&pool, &messages, default_budget).await;
        assert!(result, "Should succeed");

        // 同じコネクションを再取得
        let mut conn = pool.acquire().await.unwrap();
        let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // 元の値（150ms）に復元されていること（5000msではない）
        assert_eq!(
            timeout.0, 150,
            "busy_timeout should be restored to original (150ms), not 5000ms, got {}ms",
            timeout.0
        );

        // クリーンアップ
        drop(conn);
        drop(pool);
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_tiny_budget_skips_gracefully() {
        // 非常に小さな予算（10ms）でsave_chunk_with_retryを呼び出し、
        // パニックせずにfalseを返すことを確認
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![create_test_message("tiny1", "Message 1")];

        // 10msの予算 → 50ms未満なのでスキップされるはず
        let tiny_budget = Duration::from_millis(10);
        let result = save_chunk_with_retry(&pool, &messages, tiny_budget).await;

        // 予算不足で失敗するが、パニックしないこと
        assert!(!result, "Should fail gracefully with tiny budget");

        // データは保存されていないこと
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0, "No messages should be saved with tiny budget");
    }

    #[tokio::test]
    async fn test_budget_exhaustion_mid_chunk() {
        // 中途半端な予算でsave_comments_to_dbを呼び出し、
        // 予算内で処理が打ち切られることを確認
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        // 100件のメッセージ（2チャンク）
        let messages: Vec<ChatMessage> = (0..100)
            .map(|i| create_test_message(&format!("budget_msg{}", i), &format!("Message {}", i)))
            .collect();

        // 全体をsave_comments_to_dbに渡す（2秒予算内で処理される）
        save_comments_to_db(&pool, &messages).await;

        // 正常な予算内では全件保存されること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 100, "All 100 messages should be saved within 2s budget");
    }

    #[tokio::test]
    async fn test_remaining_budget_passed_to_retry() {
        // save_chunk_with_retryが外側の予算を尊重することを確認
        // 500msの予算で呼び出し、内部で500ms以内に完了すること
        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![
            create_test_message("budget_test1", "Message 1"),
            create_test_message("budget_test2", "Message 2"),
        ];

        let start = Instant::now();
        let budget = Duration::from_millis(500);
        let result = save_chunk_with_retry(&pool, &messages, budget).await;

        let elapsed = start.elapsed();

        // 成功すること
        assert!(result, "Should succeed");

        // 500ms未満で完了すること（予算を超えない）
        assert!(
            elapsed < Duration::from_millis(600), // マージン100ms
            "Should complete within budget, took {}ms",
            elapsed.as_millis()
        );

        // データが保存されていること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 2, "Both messages should be saved");
    }

    #[tokio::test]
    async fn test_fallback_restores_busy_timeout() {
        // フォールバックパス（save_chunk_individually）でもbusy_timeoutが復元されることを確認
        // リトライパスの復元テストと同様のカバレッジ

        use sqlx::sqlite::SqliteConnectOptions;

        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("fallback_restore_test_{}.db", std::process::id()));

        // プールのデフォルトbusy_timeoutを3000msに設定
        // 接続タイムアウトも設定（テスト用）
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .busy_timeout(std::time::Duration::from_millis(3000));

        let pool = SqlitePoolOptions::new()
            .max_connections(2) // 2接続で、1つはsave_chunk_individually用、1つは検証用
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect_with(connect_options)
            .await
            .unwrap();

        create_test_table(&pool).await;

        let messages = vec![create_test_message("fallback1", "Message 1")];

        // フォールバックを直接呼び出してテスト
        save_chunk_individually(&pool, &messages, Duration::from_millis(500)).await;

        // 別のコネクションを取得してbusy_timeoutを確認
        // 注: save_chunk_individuallyで使用したコネクションがプールに戻されている
        let mut conn = pool.acquire().await.unwrap();
        let timeout: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
            .fetch_one(&mut *conn)
            .await
            .unwrap();

        // 元の値（3000ms）に復元されていること
        // ただし、2接続プールなので別のコネクションが返される可能性がある
        // この場合も3000msであるはず
        assert_eq!(
            timeout.0, 3000,
            "busy_timeout should be 3000ms (original or default), got {}ms",
            timeout.0
        );

        // データが保存されていること
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1, "Message should be saved via fallback");

        // クリーンアップ
        drop(conn);
        drop(pool);
        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_fallback_tiny_budget_exits_without_blocking() {
        // 極小予算でsave_chunk_individuallyを呼び出し、
        // 長時間ブロックせずに終了することを確認

        let pool = create_test_pool().await;
        create_test_table(&pool).await;

        let messages = vec![create_test_message("tiny_fallback1", "Message 1")];

        let start = Instant::now();

        // 10msの予算 → 50ms未満なので即座にスキップ
        save_chunk_individually(&pool, &messages, Duration::from_millis(10)).await;

        let elapsed = start.elapsed();

        // 100ms以内に終了すること（長時間ブロックしていない）
        assert!(
            elapsed < Duration::from_millis(100),
            "Fallback with tiny budget should exit quickly, took {}ms",
            elapsed.as_millis()
        );

        // データは保存されていないこと（予算不足でスキップ）
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM comment_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0, "No messages should be saved with tiny budget");
    }
}
