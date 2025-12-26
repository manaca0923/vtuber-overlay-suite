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

**対応**:
1. `busy_timeout`設定を追加（db/mod.rs）
   - ロック競合時に即座にエラーを返さず、5秒間待機してリトライ
   ```rust
   const SQLITE_BUSY_TIMEOUT_MS: u32 = 5000;

   let pool = SqlitePoolOptions::new()
       .max_connections(5)
       .connect(&format!(
           "sqlite:{}?mode=rwc&busy_timeout={}",
           db_path, SQLITE_BUSY_TIMEOUT_MS
       ))
       .await?;
   ```

2. フォールバック処理は既存のまま維持
   - busy_timeoutでほとんどのロック競合は解消される
   - それでも失敗した場合は個別INSERTにフォールバック

**今後の対策**:
- SQLite接続設定には常にbusy_timeoutを設定
- 高負荷環境では追加のリトライロジック検討

### 4. ログレベルの適切な使用

**指摘内容（複数のPRレビューで言及）**:
- トランザクション失敗時のログが`debug`レベルで、運用時に気づきにくい

**対応**:
- 致命的エラー（トランザクション開始失敗、コミット失敗）は`log::warn!`
- 詳細なデバッグ情報は`log::debug!`
- 既存コードは適切に設定済み

## 未対応（将来対応）

### 1. SQLITE_BUSY時のリトライ/backoffテスト
- 2接続で同時書き込みし、リトライが正しく動作するかを検証するテスト
- 現状はbusy_timeout設定で対応済みのため、優先度低

### 2. parking_lot::Mutex検討
- 高並行性が必要になった場合の検討事項
- 現状のstd::sync::Mutexで問題なし

## 参照
- PR #55: https://github.com/manaca0923/vtuber-overlay-suite/pull/55
- SQLite busy handling: https://www.sqlite.org/c3ref/busy_timeout.html
