# PR#60 useBundledKey永続化のレビュー指摘

## 概要

useBundledKey状態をDBに永続化する機能のPRで受けた指摘と対応。

## 指摘事項

### P1: null値の不適切な処理（Codex Review）

**問題**:
```typescript
// 修正前
if (settings?.use_bundled_key !== undefined) {
  setUseBundledKey(settings.use_bundled_key);
}
```

`use_bundled_key`が`null`の場合、`!== undefined`はtrueになるため、`null`が`setUseBundledKey`に渡される。
これにより後続の`start_unified_polling`で`use_bundled_key=null`がRustに送られ、ポーリングが失敗する。

**対応**:
```typescript
// 修正後
if (typeof settings?.use_bundled_key === 'boolean') {
  setUseBundledKey(settings.use_bundled_key);
}
```

`typeof`を使ってbooleanの場合のみ設定するように修正。

### 中: 保存失敗時のロールバック処理（Claude Review）

**問題**:
```typescript
// 修正前
onChange={async (e) => {
  const newValue = e.target.checked;
  setUseBundledKey(newValue);  // 先にUIを更新
  try {
    await invoke('save_wizard_settings', { ... });
  } catch (err) {
    console.error('Failed to save useBundledKey:', err);
    // 保存失敗してもUIはそのまま → 再起動時に古い値に戻る
  }
}}
```

**対応**:
```typescript
// 修正後
onChange={async (e) => {
  const newValue = e.target.checked;
  const oldValue = useBundledKey;
  setUseBundledKey(newValue);
  try {
    await invoke('save_wizard_settings', { ... });
  } catch (err) {
    console.error('Failed to save useBundledKey:', err);
    setUseBundledKey(oldValue);  // ロールバック
  }
}}
```

### 中: 空のvideoId/liveChatIdで上書きされる問題（Claude Review）

**問題**:
`videoId`や`liveChatId`が空の状態で保存すると、既存の値が空文字列で上書きされる。

**対応**:
Rust側で既存値とのマージ処理を実装:
```rust
// 既存の設定を読み込んでマージ
let existing = load_existing_settings();

let merged_video_id = if video_id.is_empty() {
    existing.video_id.clone()
} else {
    video_id
};
// live_chat_id, use_bundled_key も同様にマージ
```

## 学んだこと

### 1. null vs undefined の区別

TypeScriptでDBから読み込んだ値を扱う際、`null`と`undefined`は区別する必要がある:
- `!== undefined` は `null` を通過させる
- `typeof value === 'boolean'` で明示的に型チェックする

### 2. オプティミスティックUIのロールバック

UIを先に更新する場合（オプティミスティック更新）は、失敗時のロールバック処理を必ず実装する。

### 3. 部分更新時の既存値保持

設定の一部のみを更新する場合、既存値を読み込んでマージする方式が安全:
- 空文字列や`null`は「更新しない」という意図を表す
- バックエンド側でマージ処理を行うことで、フロントエンドのミスを防げる

## チェックリスト（今後の対応）

- [ ] 同様のDB読み込みパターンで`!== undefined`を使っている箇所がないか確認
- [ ] オプティミスティックUIのロールバック漏れがないか確認
