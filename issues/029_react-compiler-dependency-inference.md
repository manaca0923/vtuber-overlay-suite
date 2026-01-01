# React Compiler依存配列推論とuseCallback

## 概要
React Compilerは、`useCallback`や`useMemo`の依存配列を自動的に推論し、手動で指定した依存配列と一致しない場合はエラーを報告する。

## 問題

### ケース: オブジェクトのプロパティのみを使用する場合

```typescript
// 悪い例: Compilerエラーになる
const skipVersion = useCallback(() => {
  if (state.update?.version) {
    localStorage.setItem(KEY, state.update.version);
  }
}, [state.update?.version]); // Compilerは state.update 全体を期待
```

React Compilerは関数本体で`state.update`オブジェクトへのアクセスを検出し、`state.update`全体を依存配列に入れることを期待する。

### エラーメッセージ
```
Compilation Skipped: Existing memoization could not be preserved
The inferred dependency was `state.update`, but the source dependencies were [state.update?.version].
Inferred less specific property than source.
```

## 解決方法

### 方法1: Compilerの推論に従う（推奨）
```typescript
// NOTE: versionのみ使用しているが、React Compilerはstate.update全体を依存として推論する
// state.updateは更新情報が変わったときのみ変化するため、実用上問題なし
const skipVersion = useCallback(() => {
  if (state.update?.version) {
    localStorage.setItem(KEY, state.update.version);
  }
}, [state.update]);
```

### 方法2: プロパティを事前に抽出（特殊なケース向け）
```typescript
// state.updateの変更頻度が高く、version以外の変更で再生成を避けたい場合
const version = state.update?.version;
const skipVersion = useCallback(() => {
  if (version) {
    localStorage.setItem(KEY, version);
  }
}, [version]);
```

## 設計判断

React Compilerに従うのが推奨される理由:
1. Compilerは関数本体のアクセスパターンを正確に分析している
2. プリミティブ値を抽出する方法は、追加の変数が必要でコードが複雑になる
3. 多くの場合、オブジェクト全体が変わる頻度は問題にならない

## 関連PR
- PR#114: ESLint警告修正とReact Compilerルール対応

## 関連ファイル
- `src/components/UpdateChecker.tsx` - `skipVersion`コールバック

## タグ
react, react-compiler, useCallback, useMemo, dependency-array, memoization
