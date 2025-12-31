# PR#68 アクセシビリティ・防御的プログラミングレビュー

## 概要

PR#68（CommentQueueManagerテスト追加 & DensityManager閾値設定UI）で指摘された改善点。

## 指摘事項と対応

### 1. setThreshold()の型安全性（防御的プログラミング）

**問題**: JavaScriptで外部から呼ばれる関数は、予期しない型の入力に対するガードが必要。

**修正前**:
```javascript
setThreshold(threshold) {
  this.highDensityThreshold = Math.max(1, Math.min(20, threshold));
}
```

**修正後**:
```javascript
setThreshold(threshold) {
  if (typeof threshold !== 'number' || isNaN(threshold)) return;
  this.highDensityThreshold = Math.max(1, Math.min(20, threshold));
}
```

**教訓**: 外部から呼ばれる可能性のあるJavaScript関数では、入力値の型チェックを行う。

### 2. フォーム要素のアクセシビリティ

**問題**: スライダーなどのフォーム要素には、スクリーンリーダー対応のための属性が必要。

**修正前**:
```tsx
<label className="...">過密検出閾値</label>
<input type="range" ... />
```

**修正後**:
```tsx
<label htmlFor="density-threshold" className="...">過密検出閾値</label>
<input
  type="range"
  id="density-threshold"
  aria-label="過密検出閾値"
  ...
/>
```

**必要な属性**:
- `id`: 入力要素に一意のIDを付与
- `htmlFor`: ラベル要素とinputを関連付け
- `aria-label`: スクリーンリーダー向けの説明

## 今後の注意点

### フォーム要素を作成する際のチェックリスト

1. [ ] `id`属性を設定しているか
2. [ ] `label`要素に`htmlFor`で関連付けているか
3. [ ] 必要に応じて`aria-label`や`aria-describedby`を設定しているか
4. [ ] キーボード操作でフォーカスが当たるか

### JavaScript関数の防御的プログラミング

1. [ ] 外部から呼ばれる関数は入力値の型チェックを行う
2. [ ] `typeof`と`isNaN`で数値型をチェック
3. [ ] オブジェクト型は `!value || typeof value !== 'object'` でチェック
4. [ ] 不正な入力の場合は早期リターンまたはデフォルト値を使用

**数値型の型ガード例**:
```javascript
setThreshold(threshold) {
  if (typeof threshold !== 'number' || isNaN(threshold)) return;
  this.threshold = Math.max(1, Math.min(20, threshold));
}
```

**上下限ガードのパターン**（PR#100で追加）:
```javascript
// 外部から受け取る値には、最小値と最大値の両方をガード
setBufferInterval(intervalMs) {
  const MAX_BUFFER_INTERVAL = 30000;  // 上限値は定数化

  // Number.isFinite()は型チェック+NaN/Infinity排除を同時に行う
  if (!Number.isFinite(intervalMs) || intervalMs <= 0 || intervalMs > MAX_BUFFER_INTERVAL) return;

  this.bufferInterval = intervalMs;
}
```

> **Note**: `Number.isFinite()` は `typeof === 'number' && !isNaN()` より簡潔で、
> `Infinity` も排除できるため推奨。

**オブジェクト型の型ガード例**:
```javascript
setSettings(settings) {
  if (!settings || typeof settings !== 'object') return;
  Object.assign(this.settings, settings);
}
```

**オブジェクト型の型ガード（配列排除版）**（PR#102で追加）:
```javascript
// typeof [] === 'object' がtrueになるため、配列も排除する必要がある場合
applyWidgetVisibility(widgetSettings) {
  if (!widgetSettings || typeof widgetSettings !== 'object' || Array.isArray(widgetSettings)) {
    console.warn('[Widget] Invalid widgetSettings, skipping');
    return;
  }
  // 処理...
}
```

> **Note**: `typeof` だけでは配列とオブジェクトを区別できない。
> 純粋なオブジェクトのみを許可する場合は `Array.isArray()` チェックを追加する。

**設定オブジェクトのフォールバック処理**（PR#102, PR#103で追加）:
```javascript
// 問題: settings.widgetがundefinedの場合、処理がスキップされる
function applySettingsUpdate(settings) {
  if (settings.widget) {
    applyWidgetVisibility(settings.widget);  // settings.widget === undefined でスキップ
  }
}

// 解決: デフォルト値へのフォールバック
const DEFAULT_WIDGET_SETTINGS = {
  clock: true,
  weather: true,
  comment: true,
  setlist: true,
  kpi: true,
  queue: true,
  promo: true,
  brand: true,
  superchat: true,
};

function applySettingsUpdate(settings) {
  // デフォルト値を使用（undefined/nullでも動作）
  applyWidgetVisibility(settings.widget || DEFAULT_WIDGET_SETTINGS);
}
```

> **Note**: デフォルト値オブジェクトはグローバルスコープで定義し、
> 関数呼び出しごとに再作成されないようにする（issues/020参照）。

**postMessageペイロードの検証**（PR#103で追加）:
```javascript
// 受信側（オーバーレイ）でのpostMessage検証
class PostMessageHandler {
  constructor(trustedOrigins = []) {
    this.trustedOrigins = trustedOrigins;
  }

  _handleMessage(event) {
    // 1. origin検証（issues/002参照）
    if (!this.trustedOrigins.includes(event.origin)) {
      console.warn('Untrusted origin:', event.origin);
      return;
    }

    // 2. data構造の検証
    const data = event.data;
    if (!data || typeof data !== 'object') {
      return;
    }

    // 3. 必須フィールドの検証
    if (typeof data.type !== 'string') {
      return;
    }

    // 4. payload検証（存在する場合）
    if (data.payload !== undefined && typeof data.payload !== 'object') {
      return;
    }

    this._dispatch(data);
  }
}
```

> **Note**: postMessage通信は外部からのデータ受信のため、
> 複数レイヤーでの検証が必要（深層防御、issues/002セクション6参照）。
