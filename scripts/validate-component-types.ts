/**
 * コンポーネントタイプの同期検証スクリプト
 *
 * JSON Schema, TypeScript, Rust間でComponentTypeの定義が
 * 一致しているかを検証します。
 *
 * 使用方法:
 *   npx tsx scripts/validate-component-types.ts
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

// === ファイルパス ===
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');
const SCHEMA_PATH = path.join(ROOT_DIR, 'src-tauri/schemas/template-mvp-1.0.json');
const TS_PATH = path.join(ROOT_DIR, 'src/types/template.ts');
const RUST_PATH = path.join(ROOT_DIR, 'src-tauri/src/server/template_types.rs');

// === 色付き出力 ===
const red = (s: string) => `\x1b[31m${s}\x1b[0m`;
const green = (s: string) => `\x1b[32m${s}\x1b[0m`;
const yellow = (s: string) => `\x1b[33m${s}\x1b[0m`;

// === JSON Schemaからコンポーネントタイプを抽出 ===
function extractFromSchema(): string[] {
  const content = fs.readFileSync(SCHEMA_PATH, 'utf-8');
  const schema = JSON.parse(content);

  const typeEnum = schema.properties?.components?.items?.properties?.type?.enum;
  if (!Array.isArray(typeEnum)) {
    throw new Error('JSON Schema: components.items.properties.type.enum が見つかりません');
  }

  return typeEnum;
}

// === TypeScriptからコンポーネントタイプを抽出 ===
function extractFromTypeScript(): string[] {
  const content = fs.readFileSync(TS_PATH, 'utf-8');

  // COMPONENT_TYPES = [...] を抽出
  const match = content.match(/COMPONENT_TYPES\s*=\s*\[([\s\S]*?)\]\s*as\s*const/);
  if (!match) {
    throw new Error('TypeScript: COMPONENT_TYPES の定義が見つかりません');
  }

  const arrayContent = match[1];
  // 文字列リテラルを抽出
  const types = [...arrayContent.matchAll(/'([^']+)'/g)].map((m) => m[1]);

  if (types.length === 0) {
    throw new Error('TypeScript: COMPONENT_TYPES が空です');
  }

  return types;
}

// === Rustからコンポーネントタイプを抽出 ===
function extractFromRust(): string[] {
  const content = fs.readFileSync(RUST_PATH, 'utf-8');

  // pub enum ComponentType { ... } を抽出
  // 構造体バリアント `Variant { field: Type },` の `},` に誤マッチしないよう、
  // 行頭の `}` にのみマッチするパターンを使用
  const match = content.match(/pub enum ComponentType\s*\{([\s\S]*?)^\}/m);
  if (!match) {
    throw new Error('Rust: pub enum ComponentType の定義が見つかりません');
  }

  const enumContent = match[1];
  const types: string[] = [];
  const lines = enumContent.split('\n');

  // 状態変数: 直前のserde(rename)値を保持
  let pendingRename: string | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // 空行やコメント行はスキップ（ただしpendingRenameはリセットしない）
    if (!trimmed || trimmed.startsWith('//')) {
      continue;
    }

    // serde(rename = "XXX") 属性を検出
    // 対応形式:
    // - #[serde(rename = "foo")]         - 通常の文字列 → グループ1
    // - #[serde(rename = r#"foo"#)]      - raw string literal → グループ2
    // - #[serde(rename = r##"foo"##)]    - raw string literal (複数#) → グループ2
    // 非対応（現時点では使用していないため）:
    // - #[cfg_attr(..., serde(rename = "foo"))]
    const renameMatch = trimmed.match(
      /#\[serde\(rename\s*=\s*(?:"([^"]+)"|r#+"([^"]+)"#+)\)\]/
    );
    const renameValue = renameMatch?.[1] ?? renameMatch?.[2];
    if (renameValue) {
      pendingRename = renameValue;
      continue;
    }

    // その他の属性行（#[...])はスキップするがpendingRenameは維持
    if (trimmed.startsWith('#[')) {
      continue;
    }

    // バリアント名を抽出
    // ユニットバリアント: `Variant,` または `Variant`
    // タプルバリアント: `Variant(Type),`
    // 構造体バリアント: `Variant { field: Type },`
    // 判別子付き: `Variant = 1,`
    const variantMatch = trimmed.match(/^(\w+)(?:\s*[({=]|,|$)/);
    if (variantMatch) {
      const variantName = variantMatch[1];
      // pendingRenameがあればその値を使用、なければバリアント名をそのまま使用
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
  }

  if (types.length === 0) {
    throw new Error('Rust: ComponentType のバリアントが空です');
  }

  return types;
}

// === 配列の差分を取得 ===
function getDiff(expected: string[], actual: string[]): { missing: string[]; extra: string[] } {
  const expectedSet = new Set(expected);
  const actualSet = new Set(actual);

  const missing = expected.filter((x) => !actualSet.has(x));
  const extra = actual.filter((x) => !expectedSet.has(x));

  return { missing, extra };
}

// === メイン処理 ===
function main(): number {
  console.log('ComponentType 同期検証を開始します...\n');

  let hasError = false;

  try {
    // 各ソースから抽出
    const schemaTypes = extractFromSchema();
    const tsTypes = extractFromTypeScript();
    const rustTypes = extractFromRust();

    console.log(`JSON Schema: ${schemaTypes.length}個のタイプ`);
    console.log(`TypeScript:  ${tsTypes.length}個のタイプ`);
    console.log(`Rust:        ${rustTypes.length}個のタイプ`);
    console.log();

    // JSON Schemaを正とする比較
    const tsDiff = getDiff(schemaTypes, tsTypes);
    const rustDiff = getDiff(schemaTypes, rustTypes);

    // TypeScriptの検証
    if (tsDiff.missing.length > 0 || tsDiff.extra.length > 0) {
      console.log(red('TypeScript との差分:'));
      if (tsDiff.missing.length > 0) {
        console.log(`  ${yellow('不足:')} ${tsDiff.missing.join(', ')}`);
      }
      if (tsDiff.extra.length > 0) {
        console.log(`  ${yellow('余分:')} ${tsDiff.extra.join(', ')}`);
      }
      hasError = true;
    } else {
      console.log(green('TypeScript: 同期OK'));
    }

    // Rustの検証
    if (rustDiff.missing.length > 0 || rustDiff.extra.length > 0) {
      console.log(red('Rust との差分:'));
      if (rustDiff.missing.length > 0) {
        console.log(`  ${yellow('不足:')} ${rustDiff.missing.join(', ')}`);
      }
      if (rustDiff.extra.length > 0) {
        console.log(`  ${yellow('余分:')} ${rustDiff.extra.join(', ')}`);
      }
      hasError = true;
    } else {
      console.log(green('Rust: 同期OK'));
    }

    // TypeScriptとRustの順序も確認
    const tsOrder = tsTypes.join(',');
    const schemaOrder = schemaTypes.join(',');
    if (tsOrder !== schemaOrder && !hasError) {
      console.log(yellow('\n順序の違い（参考情報）:'));
      console.log(`  Schema: ${schemaTypes.slice(0, 3).join(', ')}...`);
      console.log(`  TS:     ${tsTypes.slice(0, 3).join(', ')}...`);
    }
  } catch (error) {
    console.error(red(`エラー: ${error instanceof Error ? error.message : error}`));
    hasError = true;
  }

  console.log();

  if (hasError) {
    console.log(red('検証失敗: ComponentType の定義を同期してください'));
    return 1;
  }

  console.log(green('検証成功: すべての定義が同期されています'));
  return 0;
}

process.exit(main());
