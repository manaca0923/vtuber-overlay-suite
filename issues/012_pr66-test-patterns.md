# PR#66 ユニットテストパターンレビュー

## 概要

PR#66でUpdateBatcher、DensityManager、WeatherWidgetのユニットテストを追加した際のレビュー指摘事項。

## 指摘事項

### 1. テストヘルパー関数の重複

**問題**: `resolveScriptPath()`関数が4つのテストファイルで重複している。

**対象ファイル**:
- `src/utils/update-batcher.test.ts`
- `src/utils/density-manager.test.ts`
- `src/utils/weather-widget.test.ts`
- `src/utils/overlay-core.test.ts`

**推奨対応**:
```typescript
// src/utils/test-helpers.ts として共通化
export function resolveOverlayScriptPath(relativePath: string): string {
  try {
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const rootDir = path.resolve(__dirname, '../..');
    const scriptPath = path.join(rootDir, relativePath);
    if (fs.existsSync(scriptPath)) {
      return scriptPath;
    }
  } catch {
    // fileURLToPathが失敗した場合はフォールバック
  }
  return path.join(process.cwd(), relativePath);
}
```

**優先度**: 低（機能に影響なし）

### 2. JSDOMとVitestのタイマー連携

**問題**: JSDOMで作成された環境のタイマー（`setInterval`/`setTimeout`）は、Vitestの`vi.useFakeTimers()`で制御できない。

**解決方法**:
- タイマーベースのテストは、内部状態を直接操作するアプローチを使用
- 例: 古いタイムスタンプを履歴に直接設定し、`recordUpdate()`で古いエントリがフィルタリングされることを検証

```typescript
// NG: JSDOMタイマーはVitestで制御できない
vi.advanceTimersByTime(1500);
expect(manager.getDebugInfo().slots['right.lowerLeft'].count).toBe(0);

// OK: 内部状態を直接操作
const oldTimestamp = Date.now() - 2000;
manager.updateHistory.set('right.lowerLeft', [oldTimestamp]);
manager.recordUpdate('right.lowerLeft');
expect(manager.getDebugInfo().slots['right.lowerLeft'].count).toBe(1);
```

### 3. read-onlyプロパティのモック

**問題**: JSDOMの`window.performance`はread-onlyプロパティで、直接上書きできない。

**解決方法**: `Object.defineProperty`を使用

```typescript
// NG: Cannot set property performance of [object Window] which has only a getter
dom.window.performance = { now: () => Date.now() };

// OK: Object.definePropertyで上書き
Object.defineProperty(dom.window, 'performance', {
  value: { now: () => Date.now() },
  writable: true,
  configurable: true,
});
```

## 良いパターン

### 1. エッジケースのテスト

```typescript
// falsyな値（0）がデフォルト値にならないことを確認
it('style.temp=0は0として扱われる（falsyでもデフォルトにならない）', () => {
  const widget = new WeatherWidget({ style: { temp: 0 } });
  expect(widget.temp).toBe(0);
});

// nullとundefinedの区別
it('location=nullは空文字にフォールバック', () => {
  widget.update({ location: null });
  expect(widget.location).toBe('');
});

it('location=undefinedは更新しない', () => {
  widget.update({ icon: '🌧️' }); // locationはundefined
  expect(widget.location).toBe('東京'); // 変更されない
});
```

### 2. リソース管理

```typescript
// 各テストでdestroy()を呼び出してタイマーリークを防止
afterEach(() => {
  if (manager) manager.destroy();
});
```

### 3. ComponentRegistryモック

```typescript
// window.ComponentRegistryとして露出
const componentRegistryMock = `
window.ComponentRegistry = {
  _components: new Map(),
  register(name, cls) {
    this._components.set(name, cls);
  },
  get(name) {
    return this._components.get(name);
  }
};
`;
```

## 今後のテスト追加時の注意点

1. テストヘルパー関数を共通化する場合は `src/utils/test-helpers.ts` を使用
2. JSDOMのタイマーはVitestで制御できないため、内部状態操作でテスト
3. read-onlyプロパティは `Object.defineProperty` でモック
4. destroy()などのリソース解放を必ずafterEachで実行
