# Tauriコマンド引数のsnake_caseルール

## 概要
Tauriのinvoke関数で渡す引数名は、Rust側の引数名（snake_case）と完全に一致する必要がある。

## 問題
TypeScript側でcamelCaseの引数名を使用すると、Rust側のコマンドが引数を受け取れずに失敗する。

### 悪い例
```typescript
// ❌ camelCaseを使用 → Rustに引数が届かない
invoke('get_live_stream_stats', { videoId: videoId, useBundledKey: true });
invoke('save_wizard_settings', { videoId: 'xxx', liveChatId: 'yyy' });
invoke('start_unified_polling', { videoId: videoId, useBundledKey: true });
```

### 良い例
```typescript
// ✅ snake_caseを使用 → Rustに正しく引数が届く
invoke('get_live_stream_stats', { video_id: videoId, use_bundled_key: true });
invoke('save_wizard_settings', { video_id: 'xxx', live_chat_id: 'yyy' });
invoke('start_unified_polling', { video_id: videoId, use_bundled_key: true });
```

## 対象ファイル
invokeを使用するすべてのファイル:
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
# invokeの引数にcamelCaseが使われていないか確認
grep -r "invoke(" src/ --include="*.ts" --include="*.tsx" | grep -E "\{ *[a-z]+[A-Z]"
```

## PR#58での修正内容
以下のファイルでsnake_case違反を修正:
- `src/types/weather.ts`: `videoId` → `video_id`, `useBundledKey` → `use_bundled_key`

**注記**: 他のファイル（ApiKeySetup.tsx, App.tsx, WizardStep1.tsx, CommentControlPanel.tsx, Wizard.tsx）は既に別のPRで修正済みか、現在の実装でsnake_caseが使用されている。

## コメント追加
invokeを使用するファイルには以下のコメントを追加して注意喚起:
```typescript
// 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
```

## 関連ドキュメント
- `issues/003_tauri-rust-patterns.md`: Tauri/Rust関連のパターン集
