# PR#116: Promo機能のバリデーションと境界チェック

## 問題

### 1. save_promo_state の入力検証不足

`save_promo_state`はTauriコマンドとして公開されているため、任意の値が渡される可能性がある。
`show_sec=0`や`cycle_sec=9999`など極端な値が保存されると、オーバーレイ表示に異常が発生する。

**問題のあるコード:**
```rust
#[tauri::command]
pub async fn save_promo_state(
    promo_state: PromoState,  // 検証なしで保存
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // そのまま保存...
}
```

**修正後:**
```rust
#[tauri::command]
pub async fn save_promo_state(
    mut promo_state: PromoState,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 保存時にも範囲検証を行う
    if let Some(sec) = promo_state.cycle_sec {
        promo_state.cycle_sec = Some(sec.clamp(10, 120));
    }
    if let Some(sec) = promo_state.show_sec {
        promo_state.show_sec = Some(sec.clamp(3, 15));
    }
    // 保存...
}
```

### 2. インデックス境界外での無変更成功

`remove_promo_item`/`update_promo_item`が範囲外インデックスで呼ばれた場合、
エラーを返さず無変更で成功扱いになっていた。

フロントエンドでは成功メッセージが表示されるため、操作が反映されないのに「成功」になる問題があった。

**問題のあるコード:**
```rust
if index < promo_state.items.len() {
    promo_state.items.remove(index);
    save_promo_state(promo_state.clone(), state).await?;
}
// 範囲外でも Ok(promo_state) を返す
Ok(promo_state)
```

**修正後:**
```rust
if index >= promo_state.items.len() {
    return Err(format!(
        "Index out of range: {} (items count: {})",
        index,
        promo_state.items.len()
    ));
}
promo_state.items.remove(index);
save_promo_state(promo_state.clone(), state).await?;
Ok(promo_state)
```

### 3. ブロードキャスト時のOption値のNullチェック

`show_sec`/`cycle_sec`がNoneの状態でブロードキャストすると、
オーバーレイ側（JavaScript）で`null`がクランプされて最小値（3秒）になる問題があった。

**問題のあるコード:**
```rust
let payload = PromoUpdatePayload {
    items: promo_state.items,
    cycle_sec: promo_state.cycle_sec,  // None → null
    show_sec: promo_state.show_sec,    // None → null → JS側で3秒に
};
```

**JavaScript側の問題:**
```javascript
if (data.showSec !== undefined) {  // null !== undefined は true
    this.showSec = this.clampByKey(data.showSec, 'showSec', 3, 15);  // null → 3
}
```

**修正後:**
```rust
let payload = PromoUpdatePayload {
    items: promo_state.items,
    cycle_sec: Some(promo_state.cycle_sec.unwrap_or(DEFAULT_CYCLE_SEC)),
    show_sec: Some(promo_state.show_sec.unwrap_or(DEFAULT_SHOW_SEC)),
};
```

### 4. JSON破損時の修復保存

バックアップ保存だけでは破損データが残り続け、毎回フォールバックが発生する。
破損キーを削除して次回以降のフォールバックを防止する必要がある。

**問題のあるコード:**
```rust
// バックアップ保存後、デフォルト値を返すだけ
// → 次回呼び出しでも同じ破損JSONが読み込まれる
if let Err(backup_err) = sqlx::query(...).execute(pool).await {
    log::error!("Failed to backup: {}", backup_err);
}
Ok(PromoState::default())  // 破損キーはそのまま
```

**修正後:**
```rust
// バックアップ保存後、破損キーを削除
if let Err(backup_err) = sqlx::query(...).execute(pool).await {
    log::error!("Failed to backup: {}", backup_err);
}

// 破損した promo_state キーを削除
if let Err(delete_err) = sqlx::query("DELETE FROM settings WHERE key = 'promo_state'")
    .execute(pool).await {
    log::error!("Failed to delete corrupted promo state: {}", delete_err);
} else {
    log::info!("Deleted corrupted promo_state key to prevent repeated fallback");
}

Ok(PromoState::default())
```

## 教訓

### 公開APIには防御的バリデーションを

Tauriコマンドとして外部公開される関数は、任意の入力が渡される前提で設計する。

1. **クランプ対象のフィールドを持つ保存関数**: 入力値を範囲内にクランプしてから保存
2. **インデックスベースの操作**: 境界チェックを行い、範囲外ならエラーを返す
3. **フロントエンドとの整合性**: バックエンドがエラーを返さないと、UIは成功と判断する
4. **ブロードキャスト時のOption値**: Noneをそのまま送信するとJSで`null`になり、想定外の挙動を招く。デフォルト値を適用する
5. **JSON破損時の修復保存**: バックアップ保存だけでなく、破損キーを削除して次回以降のフォールバックを防止する

### 複数の経路で同じデータを保存する場合

`set_promo_settings`と`save_promo_state`のように、同じデータを保存する経路が複数ある場合：

- **どちらの経路でも同じバリデーションを適用する**
- または`save_promo_state`を内部専用にして、検証済み経路のみを公開する設計に変更する

## 関連ファイル

- `src-tauri/src/commands/promo.rs`
- `src/components/settings/PromoSettingsPanel.tsx`

## 関連ノウハウ

- [013_pr68-accessibility-defensive-coding.md](013_pr68-accessibility-defensive-coding.md): 入力検証パターン
