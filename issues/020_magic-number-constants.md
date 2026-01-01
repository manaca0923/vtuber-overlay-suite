# マジックナンバーの定数化

## 概要

同じ意味を持つ数値が複数箇所に散らばっている場合、定数として一元化すべき。
変更時の漏れを防ぎ、コードの意図を明確にする。

## 問題例

```rust
// 悪い例: 同じ値が複数箇所にハードコード
// unified_poller.rs
buffer_interval_ms: Some(1000), // InnerTubeは1秒バッファ

// youtube.rs
buffer_interval_ms: Some(1000), // InnerTubeは1秒バッファ
```

## 解決方法

```rust
// 良い例: 定数として一元化
// innertube/types.rs
/// InnerTubeモードのバッファ間隔（1秒）
/// オーバーレイ側で1秒間に取得したコメントを等間隔で表示するための設定
pub const INNERTUBE_BUFFER_INTERVAL_MS: u32 = 1000;

// 使用箇所
use crate::youtube::innertube::INNERTUBE_BUFFER_INTERVAL_MS;
buffer_interval_ms: Some(INNERTUBE_BUFFER_INTERVAL_MS),
```

## 定数化の判断基準

| 状況 | 定数化すべきか |
|------|--------------|
| 同じ値が2箇所以上で使用 | ✅ 必須 |
| 値の意味を説明するコメントが必要 | ✅ 推奨 |
| 将来変更される可能性がある | ✅ 推奨 |
| 1箇所のみで使用、変更の可能性低い | ❌ 不要 |

## 定数の命名規則

```rust
// モジュール/機能 + 用途 + 単位
const INNERTUBE_BUFFER_INTERVAL_MS: u32 = 1000;
const MAX_POLLING_INTERVAL_MS: u64 = 30000;
const MIN_DISPLAY_INTERVAL: u32 = 100;
```

## 関連ノウハウ

- **issues/013**: 防御的プログラミング（上下限ガード）
- **issues/019**: InnerTubeポーリング間隔

## 関連PR

- PR#100: InnerTubeコメント表示のリアルタイム体験改善（レビューで指摘）
- PR#102: ウィジェット表示設定トグル修正（デフォルト設定オブジェクトの配置）

## JavaScript固有のパターン

### 関数内で毎回作成される定数オブジェクト

```javascript
// 悪い例: 呼び出しごとにオブジェクトが再作成される
function applySettingsUpdate(settings) {
  const DEFAULT_WIDGET_SETTINGS = {
    clock: true, weather: true, comment: true, ...
  };
  applyWidgetVisibility(settings.widget || DEFAULT_WIDGET_SETTINGS);
}

// 良い例: グローバルスコープで一度だけ定義
const DEFAULT_WIDGET_SETTINGS = {
  clock: true, weather: true, comment: true, ...
};

function applySettingsUpdate(settings) {
  applyWidgetVisibility(settings.widget || DEFAULT_WIDGET_SETTINGS);
}
```

### 判断基準

| 状況 | グローバル化すべきか |
|------|---------------------|
| 頻繁に呼ばれる関数内の定数 | ✅ 必須 |
| 関数のロジックと密接に関連するマッピング | ❌ 関数内でOK |
| 将来的に複数箇所で参照される可能性 | ✅ 推奨 |

## 教訓

1. **2箇所以上で使う値は定数化**: コピペミスや変更漏れを防ぐ
2. **コメントで根拠を明記**: なぜこの値を選んだか後からわかるように
3. **適切なモジュールに配置**: 関連する型や定数と同じ場所に置く
4. **publicにしてエクスポート**: 複数ファイルから参照できるように

## デバウンス・スロットル値の定数化（PR#103で追加）

### 問題例

```typescript
// 悪い例: デバウンス値がハードコード
useEffect(() => {
  const timer = setTimeout(() => {
    sendUpdate(settings);
  }, 50);  // 50msの根拠が不明
  return () => clearTimeout(timer);
}, [settings]);
```

### 解決方法

```typescript
// 良い例: 定数化して意図を明確に
/**
 * 設定変更のデバウンス間隔（ミリ秒）
 * スライダー操作時の過剰なpostMessage送信を防止
 * 50ms: 人間の操作速度を考慮した値（連続操作でも最後の値のみ送信）
 */
const SETTINGS_UPDATE_DEBOUNCE_MS = 50;

useEffect(() => {
  const timer = setTimeout(() => {
    sendUpdate(settings);
  }, SETTINGS_UPDATE_DEBOUNCE_MS);
  return () => clearTimeout(timer);
}, [settings]);
```

### よく使うデバウンス・スロットル値

| 用途 | 推奨値 | 根拠 |
|------|--------|------|
| スライダー操作 | 50ms | 連続操作の最後のみ反映 |
| 入力フィールド | 300ms | 入力完了を待つ |
| ウィンドウリサイズ | 100-200ms | 過剰なレイアウト再計算を防止 |
| スクロール | 16ms (1フレーム) | 60FPS相当 |
| API呼び出し | 1000ms | レート制限対策 |

## TypeScript/Rust間の定数同期パターン（PR#106で追加）

### 問題

フロントエンドとバックエンドで同じ定数を使う場合、一方を変更すると不整合が発生。

### 解決方法

```typescript
// types/overlaySettings.ts - TypeScript側
export const MAX_CUSTOM_COLORS = 3;
export const MAX_FONT_NAME_LENGTH = 200;

// コメントでRust側との対応を明記
/**
 * カスタムカラーの最大数
 * @see src-tauri/src/commands/overlay.rs MAX_CUSTOM_COLORS
 */
```

```rust
// overlay.rs - Rust側
// コメントでTypeScript側との対応を明記
/// カスタムカラーの最大数
/// @see src/types/overlaySettings.ts MAX_CUSTOM_COLORS
const MAX_CUSTOM_COLORS: usize = 3;
```

### チェックリスト

- [ ] 定数をフロントエンド・バックエンド両方で定義したか
- [ ] コメントで相互参照を明記したか
- [ ] 値を変更する場合、両方を更新したか

## 最小値/最大値ガードの同期パターン（PR#112で追加）

### 問題

バックエンドで最小値ガードを追加したが、フロントエンドでは異なるフォールバック処理をしていたため挙動が不一致。

```rust
// バックエンド (weather.rs)
const MIN_ROTATION_INTERVAL_SEC: u32 = 1;
let rotation_interval_sec = rotation_interval_sec.max(MIN_ROTATION_INTERVAL_SEC);
// → 0が渡されると1秒に
```

```javascript
// フロントエンド (weather-widget.js) - 修正前
this.rotationInterval = (data.rotationIntervalSec || DEFAULT_ROTATION_INTERVAL_SEC) * 1000;
// → 0が渡されるとfalsy扱いで5秒にフォールバック
```

### 解決方法

```javascript
// フロントエンド (weather-widget.js) - 修正後
/** ローテーション間隔の最小値（秒） - バックエンド(weather.rs)のMIN_ROTATION_INTERVAL_SECと同値 */
const MIN_ROTATION_INTERVAL_SEC = 1;

// 数値チェック＋最小値クランプでバックエンドと挙動を統一
const intervalSec = Number.isFinite(data.rotationIntervalSec)
  ? data.rotationIntervalSec
  : DEFAULT_ROTATION_INTERVAL_SEC;
this.rotationInterval = Math.max(intervalSec, MIN_ROTATION_INTERVAL_SEC) * 1000;
```

### チェックリスト

- [ ] バックエンドとフロントエンドで同じ最小値/最大値を使っているか
- [ ] `|| default` ではなく `Number.isFinite()` + `Math.max/min` を使っているか
- [ ] コメントで相互参照を明記したか
