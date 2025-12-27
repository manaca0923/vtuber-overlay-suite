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

### P2: video_id変更時にlive_chat_idがマージされる問題（Codex Review 2回目）

**問題**:
InnerTubeモードで動画を切り替えた場合、`live_chat_id`が空文字列として送られるが、
マージロジックにより古い`live_chat_id`が保持されてしまう。
これにより、新しい動画に古い動画のchat_idが紐づく不整合が発生。

**対応**:
video_idが変更された場合は、live_chat_idをマージせず新しい値（空でも）を使用:
```rust
// video_idが変更された場合、live_chat_idは新しい値を使用（古いchat_idは無効）
let video_id_changed = existing.as_ref()
    .map(|e| e.video_id != merged_video_id)
    .unwrap_or(true);

let merged_live_chat_id = if video_id_changed {
    // video_idが変更された場合は新しいlive_chat_idを使用（空でもOK）
    live_chat_id
} else if live_chat_id.is_empty() {
    // video_idが同じで、live_chat_idが空の場合は既存値を維持
    existing.live_chat_id.clone()
} else {
    live_chat_id
};
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

### 4. 関連フィールドのマージ戦略

設定のマージ時、関連するフィールド間の整合性を考慮する:
- `video_id`と`live_chat_id`は1対1の関係
- `video_id`が変更されたら、`live_chat_id`は古い値をマージすべきではない
- 関連フィールドの変更を検知して適切にマージする

## チェックリスト（今後の対応）

- [x] 同様のDB読み込みパターンで`!== undefined`を使っている箇所がないか確認（template.tsは数値のみで問題なし）
- [x] オプティミスティックUIのロールバック漏れがないか確認（対応済み）
- [x] 関連フィールドのマージ戦略を考慮（video_id/live_chat_id）
