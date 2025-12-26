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

## 未対応（将来対応）

### 1. EXCLUSIVEトランザクションを使用したデッドロックテスト
- 一方の接続がEXCLUSIVEロックを保持した状態で、他方からsave_chunk_with_retryを呼び出す
- リトライロジックが正しく動作し、ロック解放後に成功することを検証
- 複雑なテストセットアップが必要なため、優先度低

## 参照
- PR #56: https://github.com/manaca0923/vtuber-overlay-suite/pull/56
- SQLite Result Codes: https://www.sqlite.org/rescode.html
- SQLITE_BUSY (5): https://www.sqlite.org/rescode.html#busy
- SQLITE_LOCKED (6): https://www.sqlite.org/rescode.html#locked
