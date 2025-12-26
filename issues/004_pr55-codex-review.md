# PR #55 Codexレビュー指摘事項

## 概要
PR #55「本番リリース前チェックリスト対応（6項目）」のCodexレビューで受けた指摘と対応をまとめる。

## 指摘事項と対応

### 1. docコメント更新漏れ（template_types.rs:167）

**指摘内容**:
`max_items`のdocコメントが「6〜20」のままで、実際の範囲「3〜20」と不整合

**原因**:
- Rust/TypeScript/JSON Schemaの値は更新したが、docコメントの更新を漏らした
- コード変更時にdocコメントの整合性確認を怠った

**対応**:
```rust
// Before:
/// 最大アイテム数（6〜20）

// After:
/// 最大アイテム数（3〜20, SetList推奨:14, QueueList推奨:6）
```

**今後の対策**:
- 数値範囲を変更する際は、必ず以下を確認:
  1. 実装コード
  2. docコメント
  3. JSON Schema
  4. ドキュメント（docs/）
  5. テスト

### 2. NaN/非有限数の非決定論的動作（clamp関数）

**指摘内容**:
TypeScriptのclamp関数がNaN/Infinityを受け取った場合、結果が非決定論的
- `Math.round(NaN)` → `NaN`
- `Math.max(3, NaN)` → `NaN`
- `Math.min(20, NaN)` → `NaN`

JSONパース時に不正なデータが渡されるとNaNが伝播し、予期しない動作につながる可能性

**対応**:
```typescript
// Before:
function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

// After:
function clamp(value: number, min: number, max: number): number {
  // NaN/Infinityなど非有限数は最小値にフォールバック
  if (!Number.isFinite(value)) {
    return min;
  }
  return Math.max(min, Math.min(max, value));
}
```

**今後の対策**:
- 外部入力を扱う数値処理では、必ず`Number.isFinite()`で検証
- clamp/validate関数では、非有限数に対する決定論的なフォールバック値を定義

### 3. SQLITE_BUSY対策（db.rs, db/mod.rs）

**指摘内容**:
- バッチ処理でトランザクションを使用すると、並行書き込み時にSQLITE_BUSYエラーが発生しやすい
- 現状のフォールバック処理はリトライなしでログ出力のみ
- 高負荷時にコメントが消失する可能性

**対応（v1）**: URIパラメータでbusy_timeout設定
```rust
let pool = SqlitePoolOptions::new()
    .connect(&format!("sqlite:{}?mode=rwc&busy_timeout={}", db_path, SQLITE_BUSY_TIMEOUT_MS))
    .await?;
```

**追加指摘（v2）**: URIパラメータではsqlxが正しく解釈しない可能性
- `SqliteConnectOptions`を使用して明示的にbusy_timeoutを設定すべき

**最終対応（v2）**:
```rust
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use std::time::Duration;

const SQLITE_BUSY_TIMEOUT_MS: u64 = 5000;

let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}?mode=rwc", db_path))?
    .busy_timeout(Duration::from_millis(SQLITE_BUSY_TIMEOUT_MS));

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(connect_options)
    .await?;
```

**スモークテスト追加**:
- `test_create_pool_with_busy_timeout`: busy_timeout設定でプール作成が成功することを検証

**今後の対策**:
- SQLite接続設定には`SqliteConnectOptions`を使用（URIパラメータは避ける）
- 設定変更時はスモークテストで検証

### 3.1 LRUキャッシュのロック競合（parser.rs）

**指摘内容**:
- `Mutex<LruCache>`で毎回`get()`を呼ぶとLRU順序が更新される（書き込み操作）
- 高スループット時にロック競合が発生し、レイテンシが悪化する可能性

**対応**:
- `get()`を`peek()`に変更（読み取り専用、LRU順序を更新しない）
```rust
// Before:
let emoji_info = cache.get(shortcut).cloned();

// After:
let emoji_info = cache.peek(shortcut).cloned();
```

**今後の対策**:
- キャッシュからの読み取りは`peek()`を優先
- LRU更新が必要な場合のみ`get()`を使用

### 3.2 Number.isFinite()の文字列入力対応（template.ts, clamp-constants.js）

**指摘内容**:
- `Number.isFinite()`は文字列を変換しない
- `Number.isFinite("10")` → `false`（文字列はfalse）
- クエリパラメータやシリアライズされた設定から文字列で渡された場合、最小値にクランプされてしまう
- 例: `maxItems`に`"14"`が渡されると`3`になる

**対応**:
```typescript
// Before:
function clamp(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) {
    return min;
  }
  return Math.max(min, Math.min(max, value));
}

// After:
function clamp(value: number, min: number, max: number): number {
  // 数値文字列対応: Number()で変換（Number.isFiniteは文字列を変換しない）
  const num = Number(value);
  // NaN/Infinityなど非有限数は最小値にフォールバック
  if (!Number.isFinite(num)) {
    return min;
  }
  return Math.max(min, Math.min(max, num));
}
```

**修正箇所**:
- `src/types/template.ts`: clamp関数
- `src-tauri/overlays/shared/clamp-constants.js`: clamp関数

**テスト追加**:
- `clampMaxItems`/`clampMaxLines`に対する文字列入力テスト
- 数値文字列（`"10"`）→正しくパース
- 非数値文字列（`"abc"`, `""`）→最小値にフォールバック

**今後の対策**:
- 外部入力を扱う関数では、`Number()`で明示的に変換してから`Number.isFinite()`でチェック
- `Number.isFinite()`は型変換を行わないことを意識する

### 4. ログレベルの適切な使用

**指摘内容（複数のPRレビューで言及）**:
- トランザクション失敗時のログが`debug`レベルで、運用時に気づきにくい

**対応**:
- 致命的エラー（トランザクション開始失敗、コミット失敗）は`log::warn!`
- 詳細なデバッグ情報は`log::debug!`
- 既存コードは適切に設定済み

### 5. テスト用DBファイル名のユニーク化（db/mod.rs）

**指摘内容**:
- テストで固定のDBファイル名を使用すると、並行テスト実行時にプロセス間で衝突する可能性

**対応**:
```rust
/// ユニークなテスト用DBパスを生成
/// プロセスIDとタイムスタンプを組み合わせて衝突を回避
fn unique_test_db_path(prefix: &str) -> std::path::PathBuf {
    let temp_dir = env::temp_dir();
    let pid = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temp_dir.join(format!("{}_{}_{}_{}.db", prefix, pid, timestamp, rand_suffix()))
}
```

**今後の対策**:
- テスト用の一時ファイルには必ずプロセスID+タイムスタンプ+ランダムサフィックスを含める
- 共有リソースへのアクセスは衝突を考慮した設計にする

### 6. PRAGMA busy_timeout検証テスト追加（db/mod.rs）

**指摘内容**:
- busy_timeout設定が実際にSQLiteセッションに反映されているか検証するテストがない

**対応**:
```rust
#[tokio::test]
async fn test_busy_timeout_pragma_is_set() {
    let db_path = unique_test_db_path("test_pragma_busy_timeout");
    let pool = create_pool(db_path_str).await.expect("...");

    // PRAGMA busy_timeoutの値を確認
    let row: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
        .fetch_one(&pool)
        .await
        .expect("PRAGMA query should succeed");

    assert_eq!(row.0 as u64, SQLITE_BUSY_TIMEOUT_MS);
}
```

**今後の対策**:
- 接続オプションで設定した値はPRAGMAで検証できることを覚えておく
- 重要な設定は「設定したつもり」ではなく「設定されている」ことをテストで確認

### 7. peek()からget()への変更（parser.rs） ※設計判断の変更

**指摘内容**:
- peek()を使用すると長時間配信でホット絵文字がevictされ、`:_emoji:`テキストに戻る
- ユーザー体験が悪化するため、真のLRU動作（get()）に変更すべき

**対応（v1: peek()使用）**:
当初はロック競合軽減のためpeek()を選択し、FIFO-likeなeviction動作を許容

**対応（v2: get()に変更）**:
レビュー指摘を受け、ユーザー体験を優先してget()に変更
```rust
// get()を使用してLRU順序を更新（頻繁にアクセスされる絵文字は残る）
//
// get()を選択した理由:
// 1. 長時間配信で頻繁に使われる絵文字（ホット絵文字）がevictされると
//    :_emoji:テキストに戻りユーザー体験が悪化
// 2. ロックは既に取得済みなので追加のロックオーバーヘッドはない
// 3. get()はLRU順序の更新のみで、内部的にはポインタ操作のみ（軽量）
let emoji_info = cache.get(shortcut).cloned();
```

**テスト更新**:
```rust
/// get()による真のLRU eviction動作のテスト
#[test]
fn test_hot_emoji_survives_with_lru() {
    // ホットエントリを定期的にアクセスしながらキャッシュを埋める
    // → get()によりLRU順序が更新されホットエントリは残る
}
```

**今後の対策**:
- キャッシュ実装の選択は「性能」と「ユーザー体験」のトレードオフ
- VTuber配信アプリではユーザー体験を優先すべき
- パフォーマンス最適化は実測データに基づいて行う

## 未対応（将来対応）

### 1. SQLITE_BUSY時の並行書き込みテスト
- 2接続で同時書き込みし、busy_timeoutが正しく動作するかを検証するテスト
- 現状はbusy_timeout設定とスモークテストで対応済みのため、優先度低
- `docs/900_tasks.md`に追加済み

### 2. parking_lot::Mutex検討
- 高並行性が必要になった場合の検討事項
- 現状のstd::sync::Mutexで問題なし

### 3. db.rsのSQLITE_BUSYリトライ/backoff
- トランザクション開始/コミット時にSQLITE_BUSYが発生した場合のリトライロジック
- 現状はbusy_timeoutで軽減、フォールバック処理（個別INSERT）で対応
- 高負荷環境で問題が発生したら検討
- `docs/900_tasks.md`に追加済み

## 参照
- PR #55: https://github.com/manaca0923/vtuber-overlay-suite/pull/55
- SQLite busy handling: https://www.sqlite.org/c3ref/busy_timeout.html
- lru crate peek vs get: https://docs.rs/lru/latest/lru/struct.LruCache.html#method.peek
