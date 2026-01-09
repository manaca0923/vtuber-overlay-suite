# `#[derive(Default)]` と `#[serde(default = "...")]` の競合

## 概要

Rustで `#[derive(Default)]` と `#[serde(default = "...")]` を同時に使用すると、`Default::default()` 呼び出し時に意図しないデフォルト値が設定される問題。

## 問題

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]  // ← deriveで空文字
#[serde(rename_all = "camelCase", default)]
pub struct ThemeSettings {
    /// グローバルプライマリカラー (#RRGGBB)
    #[serde(default = "default_primary_color")]  // ← serdeでのみ使用
    pub global_primary_color: String,
}

fn default_primary_color() -> String {
    "#6366f1".to_string()
}
```

- `serde_json::from_str::<ThemeSettings>("{}")` → `"#6366f1"` ✅
- `ThemeSettings::default()` → `""` (空文字) ❌

## 原因

- `#[derive(Default)]` は各フィールドに対して `T::default()` を呼び出す
- `String::default()` は空文字列 `""` を返す
- `#[serde(default = "...")]` はdeserialize時にのみ呼び出される

## 解決策

### 手動で `Default` トレイトを実装

```rust
/// デフォルトのプライマリカラー
const DEFAULT_PRIMARY_COLOR: &str = "#6366f1";

#[derive(Debug, Clone, Serialize, Deserialize)]  // ← Defaultを除去
#[serde(rename_all = "camelCase", default)]
pub struct ThemeSettings {
    #[serde(default = "default_primary_color")]
    pub global_primary_color: String,
}

// 手動で実装
impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            global_primary_color: DEFAULT_PRIMARY_COLOR.to_string(),
        }
    }
}

fn default_primary_color() -> String {
    DEFAULT_PRIMARY_COLOR.to_string()
}
```

## チェックリスト

- [ ] `#[derive(Default)]` を使用している構造体に、カスタムデフォルト値を持つ `String` フィールドがないか
- [ ] `#[serde(default = "...")]` と `#[derive(Default)]` を同時に使用していないか
- [ ] カスタムデフォルト値が必要な場合は、手動で `Default` を実装しているか
- [ ] 定数を使用してserde関数とDefault実装の値を一致させているか

## 関連PR

- PR#119: ThemeSettingsのDefault実装を手動化

## 関連issues

- [020_magic-number-constants.md](./020_magic-number-constants.md) - マジックナンバーの定数化
- [036_enum-unknown-normalization.md](./036_enum-unknown-normalization.md) - enum正規化パターン
