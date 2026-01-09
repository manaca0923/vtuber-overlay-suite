# APIフォールバックパターン

## 概要

DBからの設定読み込みやAPIレスポンスのパース時、デシリアライズ失敗でもサービス継続できるようフォールバック設計が必要。

## 問題

- 旧スキーマのデータが残っている場合、新しい型へのデシリアライズが失敗
- enumの未知値（`theme_settings`のenum不一致など）でパースエラー
- 欠損フィールドでのデシリアライズ失敗
- 500エラーを返すとオーバーレイ初期化が失敗しUX悪化

## 対応パターン

### 1. デシリアライズ失敗時のデフォルト値フォールバック

```rust
match serde_json::from_str::<OverlaySettings>(&json_str) {
    Ok(settings) => {
        let response: OverlaySettingsApiResponse = settings.into();
        Json(response).into_response()
    }
    Err(e) => {
        // デシリアライズ失敗時はデフォルト値でフォールバック
        log::warn!(
            "Failed to parse overlay settings, using defaults: {}",
            e
        );
        let response = default_overlay_settings();
        Json(response).into_response()
    }
}
```

### 2. `#[serde(default)]`の活用

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlaySettings {
    pub theme: String,
    pub layout: LayoutPreset,
    #[serde(default)]  // 欠損時はNone
    pub weather: Option<WeatherSettings>,
    #[serde(default)]  // 欠損時はNone
    pub widget: Option<WidgetVisibilitySettings>,
}
```

### 3. ログレベルの選択

- **error**: DB接続エラーなど復旧不可能な問題
- **warn**: デシリアライズ失敗でフォールバック（運用上は問題なし）
- **info**: 正常動作のログ

## チェックリスト

- [ ] DB設定読み込みにフォールバックがあるか
- [ ] HTTPエンドポイントが500を返さずデフォルト値を返すか
- [ ] 新しいフィールド追加時に`#[serde(default)]`を付けているか
- [ ] enum型にunknown値対策があるか（`#[serde(other)]`等）

## 関連PR

- PR#119: `get_overlay_settings_api`のフォールバック追加
