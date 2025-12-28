/**
 * slot定義の同期検証スクリプト
 *
 * JSON Schema, TypeScript, Rust, JavaScript間でSlotIdの定義が
 * 一致しているかを検証します。
 *
 * 使用方法:
 *   npx tsx scripts/validate-slot-types.ts
 */

import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

// === ファイルパス ===
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');
const SCHEMA_PATH = path.join(ROOT_DIR, 'src-tauri/schemas/template-mvp-1.0.json');
const TS_PATH = path.join(ROOT_DIR, 'src/types/slot.ts');
const RUST_PATH = path.join(ROOT_DIR, 'src-tauri/src/server/types.rs');
const JS_PATH = path.join(ROOT_DIR, 'src-tauri/overlays/shared/slots.js');

// === 色付き出力 ===
const red = (s: string) => `\x1b[31m${s}\x1b[0m`;
const green = (s: string) => `\x1b[32m${s}\x1b[0m`;
const yellow = (s: string) => `\x1b[33m${s}\x1b[0m`;

// === JSON Schemaからslotを抽出 ===
function extractFromSchema(): string[] {
  const content = fs.readFileSync(SCHEMA_PATH, 'utf-8');
  const schema = JSON.parse(content);

  const slotEnum = schema.properties?.components?.items?.properties?.slot?.enum;
  if (!Array.isArray(slotEnum)) {
    throw new Error('JSON Schema: components.items.properties.slot.enum が見つかりません');
  }

  return slotEnum;
}

// === TypeScriptからslotを抽出 ===
function extractFromTypeScript(): string[] {
  const content = fs.readFileSync(TS_PATH, 'utf-8');

  // SLOT_IDS = [...] を抽出
  const match = content.match(/SLOT_IDS\s*=\s*\[([\s\S]*?)\]\s*as\s*const/);
  if (!match) {
    throw new Error('TypeScript: SLOT_IDS の定義が見つかりません');
  }

  const arrayContent = match[1];
  // 文字列リテラルを抽出
  const slots = [...arrayContent.matchAll(/'([^']+)'/g)].map((m) => m[1]);

  if (slots.length === 0) {
    throw new Error('TypeScript: SLOT_IDS が空です');
  }

  return slots;
}

// === Rustからslotを抽出 ===
function extractFromRust(): string[] {
  const content = fs.readFileSync(RUST_PATH, 'utf-8');

  // pub enum SlotId { ... } を抽出
  const match = content.match(/pub enum SlotId\s*\{([\s\S]*?)^\s*\}/m);
  if (!match) {
    throw new Error('Rust: pub enum SlotId の定義が見つかりません');
  }

  const enumContent = match[1];
  const slots: string[] = [];
  const lines = enumContent.split('\n');

  // 状態変数: 直前のserde(rename)値を保持
  let pendingRename: string | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // 空行やコメント行はスキップ
    if (!trimmed || trimmed.startsWith('//')) {
      continue;
    }

    // serde属性からrename値を検出
    const renameMatch = trimmed.match(/#\[serde\([^)]*rename\s*=\s*"([^"]+)"/);
    if (renameMatch) {
      pendingRename = renameMatch[1];
      continue;
    }

    // その他の属性行はスキップ
    if (trimmed.startsWith('#[')) {
      continue;
    }

    // バリアント名を抽出
    const variantMatch = trimmed.match(/^(\w+)(?:\s*[({=]|,|$)/);
    if (variantMatch) {
      if (pendingRename) {
        slots.push(pendingRename);
        pendingRename = null;
      }
    }
  }

  if (slots.length === 0) {
    throw new Error('Rust: SlotId のバリアントが空です');
  }

  return slots;
}

// === JavaScriptからslotを抽出 ===
function extractFromJavaScript(): string[] {
  const content = fs.readFileSync(JS_PATH, 'utf-8');

  // SLOT_IDS = [...] を抽出
  const match = content.match(/const SLOT_IDS\s*=\s*\[([\s\S]*?)\];/);
  if (!match) {
    throw new Error('JavaScript: SLOT_IDS の定義が見つかりません');
  }

  const arrayContent = match[1];
  // 文字列リテラルを抽出
  const slots = [...arrayContent.matchAll(/'([^']+)'/g)].map((m) => m[1]);

  if (slots.length === 0) {
    throw new Error('JavaScript: SLOT_IDS が空です');
  }

  return slots;
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
  console.log('SlotId 同期検証を開始します...\n');

  let hasError = false;

  try {
    // 各ソースから抽出
    const schemaSlots = extractFromSchema();
    const tsSlots = extractFromTypeScript();
    const rustSlots = extractFromRust();
    const jsSlots = extractFromJavaScript();

    console.log(`JSON Schema: ${schemaSlots.length}個のslot`);
    console.log(`TypeScript:  ${tsSlots.length}個のslot`);
    console.log(`Rust:        ${rustSlots.length}個のslot`);
    console.log(`JavaScript:  ${jsSlots.length}個のslot`);
    console.log();

    // JSON Schemaを正とする比較
    const tsDiff = getDiff(schemaSlots, tsSlots);
    const rustDiff = getDiff(schemaSlots, rustSlots);
    const jsDiff = getDiff(schemaSlots, jsSlots);

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

    // JavaScriptの検証
    if (jsDiff.missing.length > 0 || jsDiff.extra.length > 0) {
      console.log(red('JavaScript との差分:'));
      if (jsDiff.missing.length > 0) {
        console.log(`  ${yellow('不足:')} ${jsDiff.missing.join(', ')}`);
      }
      if (jsDiff.extra.length > 0) {
        console.log(`  ${yellow('余分:')} ${jsDiff.extra.join(', ')}`);
      }
      hasError = true;
    } else {
      console.log(green('JavaScript: 同期OK'));
    }
  } catch (error) {
    console.error(red(`エラー: ${error instanceof Error ? error.message : error}`));
    hasError = true;
  }

  console.log();

  if (hasError) {
    console.log(red('検証失敗: SlotId の定義を同期してください'));
    return 1;
  }

  console.log(green('検証成功: すべての定義が同期されています'));
  return 0;
}

process.exit(main());
