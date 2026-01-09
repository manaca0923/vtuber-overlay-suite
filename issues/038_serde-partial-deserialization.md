# 038: serde部分的デシリアライズパターン

## 概要

保存されたJSONに新しいフィールドが追加された場合や、一部のフィールドが欠損している場合でも、デシリアライズが失敗しないようにする方法。

## 問題

1. 新しいフィールドを追加すると、古いJSONデータではデシリアライズが失敗する
2. フィールドが欠損しているとデシリアライズエラーになる
3. 毎回デフォルト値を手動で設定するのは冗長でエラーの原因になる

## 解決策

### 1. struct全体に`#[serde(default)]`を付与

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]  // <- default追加
pub struct MySettings {
    pub field1: String,
    pub field2: u32,
}
```

### 2. `Default`トレイトを実装

```rust
impl Default for MySettings {
    fn default() -> Self {
        Self {
            field1: "default_value".to_string(),
            field2: 42,
        }
    }
}
```

### 3. フィールド個別に`#[serde(default = "fn")]`を使う場合

String型など、`Default::default()`が空文字を返す型には明示的なデフォルト関数が必要：

```rust
const DEFAULT_VALUE: &str = "my_default";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MySettings {
    #[serde(default = "default_value")]  // 明示的なデフォルト関数
    pub field1: String,
}

fn default_value() -> String {
    DEFAULT_VALUE.to_string()
}
```

## 注意点

### `#[derive(Default)]`と`#[serde(default = "fn")]`の違い

- `#[derive(Default)]`: `Default::default()`を呼んだときに使われる
- `#[serde(default = "fn")]`: デシリアライズ時にフィールドが欠損している場合に使われる

**問題**: `#[derive(Default)]`を使うと、`String::default()`は空文字を返す。しかし`#[serde(default = "fn")]`は`serde_json::from_str`でのデシリアライズ時のみ適用される。

```rust
// BAD: Default::default()では空文字になる
#[derive(Default)]
#[serde(default)]
pub struct Config {
    #[serde(default = "default_color")]
    pub color: String,  // Default::default()では "" になる
}

// GOOD: 手動でDefaultを実装
impl Default for Config {
    fn default() -> Self {
        Self {
            color: default_color(),  // 正しいデフォルト値
        }
    }
}
```

詳細は `issues/037_derive-default-serde-conflict.md` を参照。

### enumにも`Default`を実装

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Position {
    Top,
    #[default]  // Rust 1.62+
    Bottom,
    Left,
    Right,
}
```

### Option<T>フィールドの欠損時挙動（重要）

`Option<T>`フィールドの欠損時挙動には注意が必要：

```rust
// パターン1: 欠損時にNone（旧データ互換性を優先）
#[serde(default)]  // Option::default() = None
pub weather: Option<WeatherSettings>,

// パターン2: 欠損時にSome(default)（新規作成時と同じ挙動）
#[serde(default = "default_weather")]
pub weather: Option<WeatherSettings>,

fn default_weather() -> Option<WeatherSettings> {
    Some(WeatherSettings::default())
}
```

**推奨**: 旧データとの互換性を維持したい場合は`#[serde(default)]`を使い、
新規作成時のみ`Some(...)`にしたい場合は`Default`トレイト実装で区別する：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlaySettings {
    /// 欠損時はNone（旧データ互換性）、Default::default()ではSome(...)
    #[serde(default)]  // 欠損時は None
    pub weather: Option<WeatherSettings>,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            weather: Some(WeatherSettings::default()),  // 新規作成時は Some(...)
        }
    }
}
```

この設計により：
- **旧JSON（フィールド欠損）**: `weather` は `None`（未設定扱い）
- **新規作成（`Default::default()`）**: `weather` は `Some(default)`（有効化）
- **「未設定」と「明示的に無効」を区別可能**

## 実装例

### OverlaySettings（PR#120対応）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlaySettings {
    #[serde(default = "OverlaySettings::default_theme")]
    pub theme: String,
    #[serde(default)]
    pub layout: LayoutPreset,  // enumにDefaultあり
    #[serde(default)]
    pub comment: CommentSettings,  // structにDefaultあり
    /// 欠損時はNone（旧データ互換性）
    #[serde(default)]
    pub weather: Option<WeatherSettings>,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            layout: LayoutPreset::default(),
            comment: CommentSettings::default(),
            weather: Some(WeatherSettings::default()),  // 新規作成時はSome
        }
    }
}
```

## デフォルト値の一元管理

デフォルト値を定数化して、`Default`実装とserde関数で共有：

```rust
const DEFAULT_FONT_SIZE: u32 = 16;

impl CommentSettings {
    fn default_font_size() -> u32 {
        DEFAULT_FONT_SIZE
    }
}

impl Default for CommentSettings {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,  // 同じ定数を使用
            // ...
        }
    }
}
```

## 関連Issue

- `issues/037_derive-default-serde-conflict.md`: `#[derive(Default)]`と`#[serde(default)]`の競合
- `issues/036_enum-unknown-normalization.md`: enum unknown値の正規化

## 適用対象

- DB保存される設定struct全般
- 後方互換性が必要なWebSocket/HTTP APIペイロード
- 新しいフィールドが追加される可能性があるstruct
