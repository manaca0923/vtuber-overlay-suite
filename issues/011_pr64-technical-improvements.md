# PR#64 技術的改善タスクのレビュー指摘

## 概要

overlay-core.jsのテスト追加、コンポーネントタイプ同期検証、slotId型安全性改善のPRに対するレビュー指摘。

## 指摘事項

### 中: Rust serde(rename)解析の脆さ (Codex Review)

**問題**:
```javascript
// 修正前: findIndexとincludesを使った直前行検索
const currentIndex = prevLines.findIndex((l) => l.includes(variantName));
if (currentIndex > 0) {
  const prevLine = prevLines[currentIndex - 1];
  // ...
}
```

- `findIndex`で`includes`を使うと、同名の部分文字列がマッチする可能性
- 属性の並びやコメントがあると誤った値を取得

**対応**:
```javascript
// 修正後: 状態変数を使って直前のserde(rename)を保持
let pendingRename: string | null = null;

for (const line of lines) {
  const trimmed = line.trim();

  // 空行やコメント行はスキップ（ただしpendingRenameはリセットしない）
  if (!trimmed || trimmed.startsWith('//')) {
    continue;
  }

  // serde(rename = "XXX") 属性を検出
  const renameMatch = trimmed.match(/#\[serde\(rename\s*=\s*"([^"]+)"\)\]/);
  if (renameMatch) {
    pendingRename = renameMatch[1];
    continue;
  }

  // その他の属性行はスキップするがpendingRenameは維持
  if (trimmed.startsWith('#[')) {
    continue;
  }

  // バリアント名を抽出
  const variantMatch = trimmed.match(/^(\w+),?$/);
  if (variantMatch) {
    if (pendingRename) {
      types.push(pendingRename);
      pendingRename = null;
    } else {
      types.push(variantMatch[1]);
    }
  }
}
```

### 中: ESM/CJS差でのパス解決問題 (Codex Review)

**問題**:
```javascript
// 修正前: __dirnameを使用（CJS専用）
const scriptPath = path.join(
  __dirname,
  '../../src-tauri/overlays/shared/overlay-core.js'
);
```

`package.json`が`"type": "module"`の場合、`__dirname`は未定義でエラーになる。

**対応**:
```javascript
// 検討した案: import.meta.urlベース
const scriptUrl = new URL(
  '../../src-tauri/overlays/shared/overlay-core.js',
  import.meta.url
);
const scriptPath = fileURLToPath(scriptUrl);
```

しかし、Vitestのjsdom環境では`import.meta.url`が正しく動作しないため、最終的にはprocess.cwdベースで解決:

```javascript
// 最終対応: process.cwdベース（Vitestで確実に動作）
const scriptPath = path.join(
  process.cwd(),
  'src-tauri/overlays/shared/overlay-core.js'
);
```

### 不足テスト: fetchLatestSetlistのtimeout引数 (Codex Review)

**問題**:
`timeout=0/負値/undefined`を与えた場合でもAbortが即時発火しないことのテストが不足。

**対応**:
```javascript
describe('fetchLatestSetlist timeout handling', () => {
  it('timeout=0でもAbortが即時発火せずにfetchが実行される', async () => {
    vi.useFakeTimers();
    const promise = OverlayCore.fetchLatestSetlist(
      'http://localhost:19800/api',
      () => {},
      0
    );
    await vi.runAllTimersAsync();
    await promise;
    expect(mockFetch).toHaveBeenCalled();
    expect(abortedSignals[0]).toBe(false);
  });
  // 負値、undefinedのテストも同様に追加
});
```

## 学んだこと

### 1. Rust enum解析での状態変数パターン

属性とバリアントの関係を解析する場合:
- 行ごとにループしながら「直前の属性」を状態変数で保持
- バリアント行に到達したら状態変数を消費してリセット
- `findIndex`/`includes`の組み合わせは誤検知のリスクあり

### 2. ESM環境でのパス解決

Node.jsのESM環境でファイルパスを解決する方法:
- `import.meta.url`ベースが標準的
- ただしテストフレームワークのDOM環境では動作しない場合あり
- `process.cwd()`は汎用的だが、実行ディレクトリ依存

### 3. Vitestのjsdom環境の制限

- `import.meta.url`がファイルパスではなく特殊なURLになる
- `process.cwd()`ベースなら安定動作
- テスト実行時のワーキングディレクトリに依存する点に注意

## 第2回レビュー指摘事項

### 高: jsdom@27のNode.js要件 (Codex Review)

**問題**:
`jsdom@27`の`engines`が`node >=20.19.0`で、既存のCI/開発環境がNode 18/20.9等の場合に`npm install`が失敗する可能性がある。

**対応**:
```json
// package.jsonにenginesフィールドを追加
{
  "engines": {
    "node": ">=20.19.0"
  }
}
```

プロジェクト全体のNode.js要件を明示的に定義することで、CI/開発環境での混乱を防止。

### 中: Rust enum解析が非ユニットバリアントに未対応 (Codex Review)

**問題**:
```javascript
// 修正前: ユニットバリアントのみ対応
const variantMatch = trimmed.match(/^(\w+),?$/);
```

`Variant = 1,`（判別子付き）や`Variant(Type)`（タプルバリアント）、`Variant { field: Type }`（構造体バリアント）に未対応。また、バリアント名を抽出できなかった場合に`pendingRename`がリセットされず、次のバリアントに誤って適用される可能性があった。

**対応**:
```javascript
// 修正後: 複数のバリアント形式に対応
// ユニットバリアント: `Variant,` または `Variant`
// タプルバリアント: `Variant(Type),`
// 構造体バリアント: `Variant { field: Type },`
// 判別子付き: `Variant = 1,`
const variantMatch = trimmed.match(/^(\w+)(?:\s*[({=]|,|$)/);
if (variantMatch) {
  const variantName = variantMatch[1];
  if (pendingRename) {
    types.push(pendingRename);
    pendingRename = null;
  } else {
    types.push(variantName);
  }
} else {
  // バリアント名を抽出できなかった場合はpendingRenameをリセット（誤適用防止）
  pendingRename = null;
}
```

### 中: CSSのscale値の後退 (Codex Review)

**問題**:
`combined.html`の`.setlist-item.current`が`transform: scale(1.05)`だったが、`setlist-common.css`への共通化時に`scale(1.02)`に変わり、意図しない見た目の後退（強調度低下）が発生。

**対応**:
```html
<!-- combined.html内で上書き -->
<style>
  /* セットリストアイテムスタイルは setlist-common.css に共通化 */
  /* combined.html固有: scale値を維持 */
  .setlist-item.current {
    transform: scale(1.05);
  }
</style>
```

共通CSSを使いつつ、ファイル固有の値を維持するための上書きパターン。

## 学んだこと

### 1. Rust enum解析での状態変数パターン

属性とバリアントの関係を解析する場合:
- 行ごとにループしながら「直前の属性」を状態変数で保持
- バリアント行に到達したら状態変数を消費してリセット
- バリアント名を抽出できなかった場合も`pendingRename`をリセット（誤適用防止）
- `findIndex`/`includes`の組み合わせは誤検知のリスクあり

### 2. ESM環境でのパス解決

Node.jsのESM環境でファイルパスを解決する方法:
- `import.meta.url`ベースが標準的
- ただしテストフレームワークのDOM環境では動作しない場合あり
- `process.cwd()`は汎用的だが、実行ディレクトリ依存

### 3. Vitestのjsdom環境の制限

- `import.meta.url`がファイルパスではなく特殊なURLになる
- `process.cwd()`ベースなら安定動作
- テスト実行時のワーキングディレクトリに依存する点に注意

### 4. Rust enum正規表現の網羅性

Rustのenumバリアントには複数の形式がある:
- ユニットバリアント: `Variant,`
- タプルバリアント: `Variant(Type),`
- 構造体バリアント: `Variant { field: Type },`
- 判別子付き: `Variant = 1,`

正規表現で解析する場合は、すべての形式に対応する必要がある。

### 5. CSS共通化時の値の維持

CSSを共通ファイルに抽出する際:
- 元のファイル固有の値が変わっていないか確認
- 変わる場合は、元のファイル内で上書きスタイルを追加

### 6. package.jsonのenginesフィールド

依存パッケージがNode.jsの特定バージョンを要求する場合:
- プロジェクトの`package.json`に`engines`フィールドを追加
- CI/開発環境での混乱を防止

## チェックリスト

### 第1回レビュー対応
- [x] Rust serde(rename)解析を状態変数方式に変更
- [x] overlay-core.test.tsのパス解決をprocess.cwdベースに変更
- [x] fetchLatestSetlistのtimeout=0/負値/undefinedテストを追加

### 第2回レビュー対応
- [x] package.jsonにenginesフィールドを追加（Node >=20.19.0）
- [x] Rust enum解析の正規表現を非ユニットバリアント対応に拡張
- [x] combined.htmlにscale(1.05)のオーバーライドを追加

## 第3回レビュー指摘事項

### 中: Node.js要件のドキュメント明記 (Codex Review)

**問題**:
`package.json`に`engines`フィールドを追加したが、CI/開発環境のドキュメントに明記されていない。

**対応**:
`docs/110_development-environment.md`を更新:
- 開発ツールテーブルにNode.js要件（>=20.19.0必須、jsdom要件）を追記
- セットアップ手順のバージョン確認コマンドに要件を追記
- Windows環境要件にNode.js 20.19.0以上を追記

### 中: serde(rename)正規表現の拡張 (Codex Review)

**問題**:
`#[serde(rename = r#"..."#)]`（raw string literal）形式に未対応。

**対応**:
```javascript
// 修正前
const renameMatch = trimmed.match(/#\[serde\(rename\s*=\s*"([^"]+)"\)\]/);

// 修正後
// 対応形式:
// - #[serde(rename = "foo")]         - 通常の文字列
// - #[serde(rename = r#"foo"#)]      - raw string literal
// - #[serde(rename = r##"foo"##)]    - raw string literal (複数#)
const renameMatch = trimmed.match(/#\[serde\(rename\s*=\s*(?:r#+")?\"([^"]+)\"(?:#+)?\)\]/);
```

### 学んだこと: Rustのraw string literal

Rustでは`r#"..."#`形式でraw string literalを記述できる:
- エスケープ不要で特殊文字を含む文字列を記述可能
- `#`の数は任意で増やせる（`r##"..."##`など）
- 正規表現で解析する場合は両形式に対応する必要がある

### 第3回レビュー対応
- [x] docs/110_development-environment.mdにNode.js要件を明記
- [x] serde(rename)の正規表現をraw string literal対応に拡張

## 第4回レビュー指摘事項

### 中: serde(rename)正規表現がraw stringにマッチしない (Codex Review)

**問題**:
第3回で追加した正規表現 `(?:r#+")?\"([^\"]+)\"(?:#+)?` は、raw string literal `r#"foo"#` に実際はマッチしない構造になっていた。

```javascript
// 問題のある正規表現
/#\[serde\(rename\s*=\s*(?:r#+")?\"([^"]+)\"(?:#+)?\)\]/
// r#" の後に \" を期待するが、raw stringでは " のみ
```

**対応**:
通常の文字列とraw stringを別パターンで分岐し、どちらかを取得する形に修正:

```javascript
// 修正後: 通常文字列(グループ1)とraw string(グループ2)を別パターンで分岐
const renameMatch = trimmed.match(
  /#\[serde\(rename\s*=\s*(?:"([^"]+)"|r#+"([^"]+)"#+)\)\]/
);
const renameValue = renameMatch?.[1] ?? renameMatch?.[2];
if (renameValue) {
  pendingRename = renameValue;
  continue;
}
```

### 学んだこと: 正規表現での複数形式対応

複数の文字列形式（通常/raw string）に対応する正規表現を書く際:
- オプショナルグループ `(?:...)?` で両方をカバーしようとすると、実際にはマッチしないケースがある
- 代わりに、alternation `(?:...|...)` で明確に分岐し、別々のキャプチャグループで取得する
- `??`（nullish coalescing）で最初にマッチしたグループを選択

### 第4回レビュー対応
- [x] serde(rename)正規表現を通常文字列/raw string別パターン分岐に修正

## 第5回レビュー指摘事項

### 中: enum終端の`}`が構造体バリアントの`},`に誤マッチする (Codex Review)

**問題**:
```javascript
// 問題のある正規表現
/pub enum ComponentType\s*\{([\s\S]*?)\n\}/
```

この正規表現は `\n}` にマッチするため、構造体バリアント `Variant { field: Type },` の `},` にも誤マッチする可能性がある。

**対応**:
行頭の`}`にのみマッチするパターンに修正:

```javascript
// 修正後: 行頭の`}`にのみマッチ
/pub enum ComponentType\s*\{([\s\S]*?)^\}/m
```

`^}` と `m`（multiline）フラグにより、行頭の`}`にのみマッチするようになる。

### 学んだこと: 正規表現のmultilineフラグ

- `m`（multiline）フラグを使うと、`^`と`$`が行頭・行末にマッチするようになる
- 構造体やenumの終端`}`を正確に抽出するには、`^}`パターンが有効
- インデントされたバリアント内の`}`（構造体バリアントなど）と区別できる

### 第5回レビュー対応
- [x] enum終端の正規表現を行頭`}`のみにマッチするよう修正（multilineフラグ使用）

## 第6回レビュー指摘事項

### 中: enum終端の`}`がインデント時にマッチしない (Codex Review)

**問題**:
```javascript
// 問題のある正規表現
/pub enum ComponentType\s*\{([\s\S]*?)^\}/m
```

`^}`は行頭の`}`のみにマッチするため、`mod`内などでインデントされている場合にマッチしない。

**対応**:
インデント（空白）を許容するパターンに修正:

```javascript
// 修正後: インデント許容
/pub enum ComponentType\s*\{([\s\S]*?)^\s*\}/m
```

`^\s*}`により、行頭のインデント（空白）があっても`}`にマッチするようになる。

### 後回し: raw string内の`"`対応 (Codex Review → 900_tasks.md)

**問題**:
`r##"foo"bar"##` のように`"`を含むraw stringで途中切れする可能性。

**判断**:
現時点では使用していないため、`docs/900_tasks.md`に追記して後回し。

### 後回し: process.cwd()依存 (Codex Review → 900_tasks.md)

**問題**:
テスト実行ディレクトリがリポジトリ直下でない場合に読み込み失敗。

**判断**:
Vitestの標準実行方法（プロジェクトルートから`npm run test`）で問題なく動作するため、`docs/900_tasks.md`に追記して後回し。

### 第6回レビュー対応
- [x] enum終端の正規表現をインデント許容に修正（`^\s*}`）
- [x] raw string内の`"`対応を900_tasks.mdに追記
- [x] process.cwd()依存を900_tasks.mdに追記
