# ESLint設定：オーバーレイJSファイル向け

## 概要

`src-tauri/overlays/`のJavaScriptファイルは、ブラウザで実行されるコードであり、
Node.js/TypeScript用のデフォルト設定ではlintエラーが発生する。

## 問題

1. **ブラウザグローバル未認識**: `window`, `document`, `console`等が未定義エラー
2. **共有グローバル未認識**: `BaseComponent`, `SlotManager`等の共有クラスが未定義エラー
3. **TypeScriptルールの誤適用**: `@typescript-eslint/no-unused-vars`がJSファイルにも適用される

## 解決策

### ESLint設定（eslint.config.js）

```javascript
{
  files: ['src-tauri/overlays/**/*.js'],
  languageOptions: {
    ecmaVersion: 2020,
    sourceType: 'script',
    globals: {
      ...globals.browser,
      // オーバーレイ共通グローバル
      BaseComponent: 'readonly',
      ComponentRegistry: 'readonly',
      SlotManager: 'readonly',
      CLAMP_RANGES: 'readonly',
      UpdateBatcher: 'readonly',
      DensityManager: 'readonly',
    },
  },
  rules: {
    // 未使用変数：_プレフィックスで許可（引数と変数の両方）
    'no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
    // JSファイルにはTypeScriptルールを適用しない
    '@typescript-eslint/no-unused-vars': 'off',
  },
},
{
  // グローバルを定義するファイルはno-redeclareを無効化
  files: [
    'src-tauri/overlays/components/base-component.js',
    'src-tauri/overlays/shared/component-registry.js',
    // ...他のグローバル定義ファイル
  ],
  rules: {
    'no-redeclare': 'off',
  },
},
```

## ベストプラクティス

### 1. 未使用パラメータの命名

```javascript
// ✅ 良い例：_プレフィックスで意図を明示
update(_data) {}
onEvent(_eventType, _payload) {}
catch (_e) { ... }

// ❌ 悪い例：警告が発生
update(data) {}
catch (e) { ... }
```

### 2. `varsIgnorePattern`と`argsIgnorePattern`の両方を設定

一貫性のために、両方のパターンを設定する：

```javascript
'no-unused-vars': ['error', {
  argsIgnorePattern: '^_',   // 関数引数
  varsIgnorePattern: '^_'    // 変数
}],
```

### 3. グローバル定義ファイルの明示的な除外

新しい共有グローバルを定義するファイルを追加する場合は、
`no-redeclare: 'off'`の対象ファイルリストに追加すること。

## 関連PR

- PR#97: ESLint設定を改善しオーバーレイJSのlintエラーを解消
