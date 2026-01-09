# enum未知値の正規化パターン

## 概要

Rust enumでフロントエンドとの互換性を保ちつつ、未知の値（旧バージョンからの移行、将来の拡張）を安全に処理するパターン。

## 問題

- 新しいenum値を追加すると、旧データがデシリアライズ失敗する
- 未知の値がそのままフロントエンドに渡ると、UIが不正値を処理できずに崩れる
- 未知値が再保存されると、DBに`"unknown"`等の文字列が残る

## 解決策

### 1. `#[serde(other)]` でUnknownバリアントを定義

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GlobalTheme {
    #[default]
    White,
    Purple,
    Sakura,
    Ocean,
    Custom,
    /// 未知の値（旧バージョンとの互換性用）
    #[serde(other)]
    Unknown,
}
```

### 2. `normalize()` メソッドでUnknownをデフォルトに変換

```rust
impl GlobalTheme {
    /// Unknown値をデフォルト値に正規化
    /// API応答でフロントエンドに渡す前に呼び出す
    pub fn normalize(self) -> Self {
        match self {
            Self::Unknown => Self::default(),
            other => other,
        }
    }
}
```

### 3. API応答前に正規化を適用

```rust
// WebSocket送信時
let payload = SettingsUpdatePayload {
    // ...
    theme_settings: settings.theme_settings.map(|ts| ts.normalize()),
};

// HTTP API応答時
impl From<OverlaySettings> for OverlaySettingsApiResponse {
    fn from(settings: OverlaySettings) -> Self {
        Self {
            // ...
            theme_settings: settings.theme_settings.map(|ts| ts.normalize()),
        }
    }
}
```

### 4. ネスト型の正規化

```rust
impl ThemeSettings {
    pub fn normalize(mut self) -> Self {
        self.global_theme = self.global_theme.normalize();
        self.font_preset = self.font_preset.normalize();
        self
    }
}
```

## チェックリスト

- [ ] enumに `#[serde(other)] Unknown` バリアントがあるか
- [ ] `Default` トレイトが実装されているか
- [ ] `normalize()` メソッドがあるか
- [ ] API応答（HTTP/WebSocket）前に `normalize()` を呼び出しているか
- [ ] ネスト構造を持つ型は内部のenumも正規化しているか

## 関連PR

- PR#119: GlobalTheme/FontPresetのUnknown値正規化

## 関連issues

- [021_serde-field-naming.md](./021_serde-field-naming.md) - serdeのフィールド命名規則
- [034_api-fallback-patterns.md](./034_api-fallback-patterns.md) - APIフォールバックパターン
