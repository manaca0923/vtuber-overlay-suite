# 032: Debounce保存とブロードキャストの原子性

## 問題

### 1. Debounce時の古い値参照問題

複数フィールドをdebounce保存する際、コールバック内で`useState`の値を直接参照すると、**スナップショット**（クロージャに閉じ込められた古い値）を保存してしまう可能性がある。

```typescript
// ❌ 悪い例: setTimeoutコールバックがクロージャで古い値を参照
const [localLogoUrl, setLocalLogoUrl] = useState('');
const [localText, setLocalText] = useState('');

const handleLogoUrlChange = (e: ChangeEvent<HTMLInputElement>) => {
  const value = e.target.value;
  setLocalLogoUrl(value);

  saveTimerRef.current = setTimeout(() => {
    // ここで localText は古い値を参照する可能性がある
    handleSave(value, localText);  // ❌ 競合リスク
  }, DEBOUNCE_MS);
};
```

**発生シナリオ**: ユーザーが素早く両フィールドを変更した場合
1. logoUrlを変更 → タイマー開始（500ms後に保存）
2. 100ms後にtextを変更 → 新しいタイマー開始、古いタイマーキャンセル
3. 500ms後に保存実行 → `localLogoUrl`は古い値（1のスナップショット）

### 2. 保存とブロードキャストの分離問題

```typescript
// ❌ 悪い例: 2つの独立した呼び出し
const saved = await invoke('save_brand_settings', { ... });
await invoke('broadcast_brand_update', { brand_settings: saved });
```

問題点:
- 保存は成功したがブロードキャストが失敗する可能性
- 他のクライアントからの更新とレースコンディション
- オーバーレイが最新状態に追随しない

## 解決策

### 1. useRefで最新値を保持

```typescript
// ✅ 良い例: refで最新値を保持
const latestValuesRef = useRef({ logoUrl: '', text: '' });

const handleLogoUrlChange = (e: ChangeEvent<HTMLInputElement>) => {
  const value = e.target.value;
  setLocalLogoUrl(value);

  // refを更新（常に最新値を保持）
  latestValuesRef.current.logoUrl = value;

  saveTimerRef.current = setTimeout(() => {
    // handleSave内でlatestValuesRef.currentから最新値を取得
    handleSave();
  }, DEBOUNCE_MS);
};

const handleSave = async () => {
  // refから最新値を取得（古い値の問題を回避）
  const { logoUrl, text } = latestValuesRef.current;
  // ... 保存処理
};
```

### 2. 保存とブロードキャストを単一コマンドに統一

```rust
// Rust側: 原子的な操作として実装
#[tauri::command]
pub async fn save_and_broadcast_brand(
    brand_settings: BrandSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let validated = save_brand_settings(brand_settings, state.clone()).await?;
    broadcast_brand_update(validated, state).await?;
    Ok(())
}
```

```typescript
// TypeScript側: 単一コマンドを呼び出し
await invoke('save_and_broadcast_brand', { brand_settings: newSettings });
```

## 適用対象

- 複数フィールドを持つ設定パネル（BrandSettings等）
- debounce保存を行うフォーム入力
- 保存後にWebSocket配信が必要な機能

## 関連

- [030](030_pr115-queue-management-review.md): 非同期競合対策
- [007](007_tauri-invoke-snake-case.md): Tauriコマンドパターン

## 参照PR

- PR#117: BrandSettingsPanel実装
