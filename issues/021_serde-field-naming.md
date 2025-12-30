# Rust serde のフィールド命名規則

## 概要

Rustの`serde`でJSONシリアライズする際、enum variantのフィールド名は自動でcamelCaseに変換されない。
この問題により、Rust側とJavaScript側でフィールド名が一致しないバグが発生した。

## 問題例

```rust
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    CommentAdd {
        payload: ChatMessage,
        instant: bool,
        buffer_interval_ms: Option<u32>,  // ← snake_caseのまま送信される
    },
}
```

```javascript
// 誤: camelCaseでアクセス
commentQueue.queue(data.payload, data.bufferIntervalMs);  // undefined

// 正: snake_caseでアクセス
commentQueue.queue(data.payload, data.buffer_interval_ms);  // OK
```

## 原因

`#[serde(rename_all = "camelCase")]` をenumに適用しても：
- **tag値**と**variant名**はcamelCaseに変換される
- **struct variant内のフィールド名**には適用されない

## 解決方法

### 方法1: JavaScript側でsnake_caseを使用（推奨）

Rust側の命名規則に合わせてJavaScript側を修正：

```javascript
data.buffer_interval_ms  // Rustのフィールド名に合わせる
```

### 方法2: Rust側で個別にrenameを指定

```rust
CommentAdd {
    payload: ChatMessage,
    instant: bool,
    #[serde(rename = "bufferIntervalMs")]
    buffer_interval_ms: Option<u32>,
},
```

### 方法3: variant内のstructにrename_allを指定

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommentAddPayload {
    payload: ChatMessage,
    instant: bool,
    buffer_interval_ms: Option<u32>,
}
```

## チェックリスト

新しいWebSocketメッセージフィールドを追加する際：

1. [ ] Rust側のフィールド名を確認
2. [ ] JavaScript側のアクセス名がRust側と一致するか確認
3. [ ] ドキュメント（`docs/300_overlay-specs.md`）を更新

## 関連PR

- PR#100: `buffer_interval_ms`フィールド追加
- PR#101: フィールド名不一致の修正

## 教訓

1. **serdeのrename_allはvariant内フィールドに自動適用されない**
2. **Rust側とJavaScript側でフィールド名の一貫性を確認する**
3. **新フィールド追加時はドキュメントも同時に更新する**
