# Rust serde のフィールド命名規則

## 概要

Rustの`serde`でJSONシリアライズする際のフィールド命名に関するルール。
このルールは**WebSocketメッセージ**と**Tauriコマンド戻り値**の両方に適用される。

## 重要ルール

### 1. Tauriコマンドの引数 vs 戻り値

| 種類 | 命名規則 | 参照 |
|------|---------|------|
| **引数（パラメータ）** | `snake_case` | [issues/007](007_tauri-invoke-snake-case.md) |
| **戻り値（レスポンス）** | Rust側のserde設定に依存 | 本ドキュメント |

### 2. 本プロジェクトの規約

**ほとんどの構造体で `#[serde(rename_all = "camelCase")]` を使用**
→ TypeScript側も**camelCase**で定義する

```rust
// Rust側
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStreamStats {
    pub concurrent_viewers: Option<i64>,  // → concurrentViewers
    pub like_count: Option<i64>,          // → likeCount
}
```

```typescript
// TypeScript側 ✅ 正しい
interface LiveStreamStats {
  concurrentViewers: number | null;
  likeCount: number | null;
}

// TypeScript側 ❌ 間違い
interface LiveStreamStats {
  concurrent_viewers: number | null;  // Rustのフィールド名をそのまま使っている
  like_count: number | null;
}
```

### 3. 例外: rename_allがない場合

一部の構造体（例: `WizardSettingsData`）では`rename_all`なし
→ TypeScript側も**snake_case**のまま

```rust
// Rust側（rename_allなし）
pub struct WizardSettingsData {
    pub video_id: String,     // → video_id
    pub live_chat_id: String, // → live_chat_id
}
```

```typescript
// TypeScript側（snake_caseを使用）
interface WizardSettingsData {
  video_id: string;
  live_chat_id: string;
}
```

---

## 問題パターン1: enum variantのフィールド

enum variantのフィールド名は自動でcamelCaseに変換されない。
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
4. **Tauriコマンド戻り値の型定義時は、Rust側のserde設定を必ず確認する**
5. **既存の型定義を再利用し、重複定義を避ける**

---

## オプショナル/必須の整合性

### ルール

TypeScript型を定義する際、フィールドのオプショナル/必須はRust側と一致させる：

| Rust | TypeScript | 説明 |
|------|------------|------|
| `pub field: String` | `field: string` | 必須フィールド |
| `pub field: Option<String>` | `field?: string` または `field: string \| null` | オプショナル |
| `#[serde(default)]` | `field?: Type` | デフォルト値あり（オプショナル可） |

### 問題例（PR#111で発見）

```typescript
// ❌ 悪い例: Rust側は必須なのにTypeScript側がオプショナル
interface WizardSettingsData {
  saved_at?: string;  // TypeScript: オプショナル
}

// Rust側
pub struct WizardSettingsData {
  pub saved_at: String,  // Rust: 必須
}

// ✅ 良い例: 整合性が取れている
interface WizardSettingsData {
  saved_at: string;  // TypeScript: 必須（Rust側と一致）
}
```

### 確認手順

1. Rust側の型定義を確認（`Option<T>`かどうか）
2. `#[serde(default)]`があるか確認
3. TypeScript側の型定義を同じルールで設定

---

## 重複定義の防止

### ルール

新しくTypeScript型を定義する前に、以下を確認：

1. `src/types/` に同じ型が既に存在しないか検索
2. 存在する場合はインポートして再利用
3. 新規定義が必要な場合は、Rust側のserde設定を確認
4. **オプショナル/必須の整合性を確認**

### 例: LiveStreamStats

```typescript
// ❌ 悪い例: ローカルファイルで重複定義
// src/components/settings/OverlayPreview.tsx
interface LiveStreamStats {
  concurrent_viewers: number | null;  // 間違った命名
}

// ✅ 良い例: 既存の型定義を再利用
// src/components/settings/OverlayPreview.tsx
import type { LiveStreamStats } from '../../types/weather';
```

### 確認コマンド

```bash
# 型定義の重複を確認
grep -r "interface LiveStreamStats" src/
grep -r "type LiveStreamStats" src/
```

---

## 関連PR

- PR#100: `buffer_interval_ms`フィールド追加
- PR#101: フィールド名不一致の修正
- PR#110: LiveStreamStats型の重複定義修正
