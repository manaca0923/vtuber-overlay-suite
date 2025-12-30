# Tauriコマンド引数のsnake_caseルール

## 概要
Tauriのinvoke関数で渡す引数名は、Rust側の引数名（snake_case）と完全に一致する必要がある。

## Tauri 2.xのデフォルト動作

**重要**: Tauri 2.xはデフォルトでRust側の`snake_case`パラメータをJavaScript側の`camelCase`に変換する。

```
Rust: video_id → JavaScript: videoId（デフォルト）
```

### 本プロジェクトでの対応

フロントエンドが既に`snake_case`を使用しているため、Rust側で`rename_all = "snake_case"`を指定して
両方で`snake_case`を使用する方式を採用している。

```rust
// Rust側: rename_allを指定してsnake_caseを受け付ける
#[tauri::command(rename_all = "snake_case")]
pub async fn start_unified_polling(
    video_id: String,        // ← JavaScript側も video_id で渡す
    use_bundled_key: bool,   // ← JavaScript側も use_bundled_key で渡す
    ...
) -> Result<(), String> { ... }
```

```typescript
// TypeScript側: snake_caseで渡す
invoke('start_unified_polling', { video_id: videoId, use_bundled_key: true });
```

## 修正が必要なコマンド

`#[tauri::command(rename_all = "snake_case")]`を追加すべきコマンド:

- snake_caseパラメータを持つ全てのコマンド
- 例: `api_key`, `video_id`, `live_chat_id`, `use_bundled_key`, `setlist_id`, `song_id` など

## 問題発生時のエラーメッセージ

```
invalid args `videoId` for command `start_unified_polling`:
command start_unified_polling missing required key videoId
```

このエラーは、Tauri側が`videoId`（camelCase）を期待しているが、
フロントエンドが`video_id`（snake_case）を送信している場合に発生する。

## 解決方法

Rustコマンドに`rename_all = "snake_case"`を追加:

```rust
// Before
#[tauri::command]
pub async fn save_wizard_settings(video_id: String, ...) { ... }

// After
#[tauri::command(rename_all = "snake_case")]
pub async fn save_wizard_settings(video_id: String, ...) { ... }
```

## 対象ファイル

### Rust側（rename_all追加済み）
- `src-tauri/src/commands/youtube.rs`
- `src-tauri/src/commands/setlist.rs`
- `src-tauri/src/commands/weather.rs`
- `src-tauri/src/commands/keyring.rs`

### TypeScript側（snake_case使用）
- `src/types/weather.ts`
- `src/types/commands.ts`
- `src/types/overlaySettings.ts`
- `src/components/ApiKeySetup.tsx`
- `src/components/CommentControlPanel.tsx`
- `src/components/wizard/Wizard.tsx`
- `src/components/wizard/WizardStep1.tsx`
- `src/App.tsx`

## 確認方法
```bash
# Rust側: rename_allが設定されていないコマンドを確認
grep -B1 "pub async fn" src-tauri/src/commands/*.rs | grep -A1 "#\[tauri::command\]$"

# TypeScript側: invokeの引数にcamelCaseが使われていないか確認
grep -r "invoke(" src/ --include="*.ts" --include="*.tsx" | grep -E "\{ *[a-z]+[A-Z]"
```

## 関連ドキュメント
- `issues/003_tauri-rust-patterns.md`: Tauri/Rust関連のパターン集
- [Tauri 2.0 Command API](https://v2.tauri.app/develop/calling-rust/)
