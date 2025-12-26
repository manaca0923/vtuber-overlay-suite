/**
 * テンプレートバリデーション関数のテスト
 */

import { describe, it, expect } from 'vitest';
import {
  hasSlotDuplicates,
  hasIdDuplicates,
  clampMaxItems,
  clampMaxLines,
  CLAMP_RANGES,
  type TemplateComponent,
} from './template';
import type { SlotId } from './slot';

// テスト用のヘルパー関数
function createComponent(
  id: string,
  slot: SlotId,
  enabled: boolean = true
): TemplateComponent {
  return {
    id,
    type: 'ChatLog',
    slot,
    enabled,
    style: undefined,
    rules: undefined,
    tuning: undefined,
  };
}

describe('hasSlotDuplicates', () => {
  it('空の配列で false を返す', () => {
    expect(hasSlotDuplicates([])).toBe(false);
  });

  it('1つのコンポーネントで false を返す', () => {
    const components = [createComponent('comp1', 'left.top')];
    expect(hasSlotDuplicates(components)).toBe(false);
  });

  it('異なるslotを使うコンポーネントで false を返す', () => {
    const components = [
      createComponent('comp1', 'left.top'),
      createComponent('comp2', 'left.middle'),
      createComponent('comp3', 'right.upper'),
    ];
    expect(hasSlotDuplicates(components)).toBe(false);
  });

  it('同じslotを使う有効なコンポーネントで true を返す', () => {
    const components = [
      createComponent('comp1', 'left.top'),
      createComponent('comp2', 'left.top'), // duplicate!
    ];
    expect(hasSlotDuplicates(components)).toBe(true);
  });

  it('同じslotでも無効なコンポーネントはカウントしない', () => {
    const components = [
      createComponent('comp1', 'left.top', true),
      createComponent('comp2', 'left.top', false), // disabled - OK
    ];
    expect(hasSlotDuplicates(components)).toBe(false);
  });

  it('両方が無効なコンポーネントはカウントしない', () => {
    const components = [
      createComponent('comp1', 'left.top', false),
      createComponent('comp2', 'left.top', false),
    ];
    expect(hasSlotDuplicates(components)).toBe(false);
  });

  it('3つ以上のコンポーネントで重複がある場合 true を返す', () => {
    const components = [
      createComponent('comp1', 'left.top'),
      createComponent('comp2', 'left.middle'),
      createComponent('comp3', 'left.top'), // duplicate with comp1!
    ];
    expect(hasSlotDuplicates(components)).toBe(true);
  });
});

describe('hasIdDuplicates', () => {
  it('空の配列で false を返す', () => {
    expect(hasIdDuplicates([])).toBe(false);
  });

  it('1つのコンポーネントで false を返す', () => {
    const components = [createComponent('comp1', 'left.top')];
    expect(hasIdDuplicates(components)).toBe(false);
  });

  it('異なるIDを持つコンポーネントで false を返す', () => {
    const components = [
      createComponent('comp1', 'left.top'),
      createComponent('comp2', 'left.middle'),
      createComponent('comp3', 'right.upper'),
    ];
    expect(hasIdDuplicates(components)).toBe(false);
  });

  it('同じIDを持つコンポーネントで true を返す', () => {
    const components = [
      createComponent('same-id', 'left.top'),
      createComponent('same-id', 'left.middle'), // duplicate ID!
    ];
    expect(hasIdDuplicates(components)).toBe(true);
  });

  it('無効なコンポーネントでもIDは重複チェックされる', () => {
    // hasIdDuplicatesはenabledを考慮しない
    const components = [
      createComponent('same-id', 'left.top', true),
      createComponent('same-id', 'left.middle', false), // disabled but still duplicate ID!
    ];
    expect(hasIdDuplicates(components)).toBe(true);
  });

  it('3つ以上のコンポーネントで重複がある場合 true を返す', () => {
    const components = [
      createComponent('comp1', 'left.top'),
      createComponent('comp2', 'left.middle'),
      createComponent('comp1', 'right.upper'), // duplicate with first comp1!
    ];
    expect(hasIdDuplicates(components)).toBe(true);
  });

  it('空文字列のIDでも重複チェックされる', () => {
    const components = [
      createComponent('', 'left.top'),
      createComponent('', 'left.middle'), // duplicate empty ID!
    ];
    expect(hasIdDuplicates(components)).toBe(true);
  });
});

describe('clampMaxItems', () => {
  it('範囲内の値はそのまま返す', () => {
    expect(clampMaxItems(10)).toBe(10);
    expect(clampMaxItems(6)).toBe(6);
    expect(clampMaxItems(14)).toBe(14);
  });

  it('最小値3に設定されている（QueueList対応）', () => {
    expect(CLAMP_RANGES.maxItems.min).toBe(3);
  });

  it('最大値20に設定されている', () => {
    expect(CLAMP_RANGES.maxItems.max).toBe(20);
  });

  it('最小値未満は3にクランプされる', () => {
    expect(clampMaxItems(1)).toBe(3);
    expect(clampMaxItems(2)).toBe(3);
    expect(clampMaxItems(0)).toBe(3);
    expect(clampMaxItems(-5)).toBe(3);
  });

  it('最大値超過は20にクランプされる', () => {
    expect(clampMaxItems(21)).toBe(20);
    expect(clampMaxItems(100)).toBe(20);
  });

  it('小数点は丸められる', () => {
    expect(clampMaxItems(10.4)).toBe(10);
    expect(clampMaxItems(10.6)).toBe(11);
  });

  it('NaNは決定論的に最小値にクランプされる', () => {
    // Number.isFinite(NaN) === false なので最小値にフォールバック
    expect(clampMaxItems(NaN)).toBe(CLAMP_RANGES.maxItems.min);
  });

  it('Infinityは決定論的に最小値にクランプされる', () => {
    // Number.isFinite(Infinity) === false なので最小値にフォールバック
    expect(clampMaxItems(Infinity)).toBe(CLAMP_RANGES.maxItems.min);
  });

  it('-Infinityは決定論的に最小値にクランプされる', () => {
    // Number.isFinite(-Infinity) === false なので最小値にフォールバック
    expect(clampMaxItems(-Infinity)).toBe(CLAMP_RANGES.maxItems.min);
  });
});

describe('clampMaxLines', () => {
  it('範囲内の値はそのまま返す', () => {
    expect(clampMaxLines(10)).toBe(10);
    expect(clampMaxLines(4)).toBe(4);
    expect(clampMaxLines(14)).toBe(14);
  });

  it('最小値は4', () => {
    expect(CLAMP_RANGES.maxLines.min).toBe(4);
  });

  it('最大値は14', () => {
    expect(CLAMP_RANGES.maxLines.max).toBe(14);
  });

  it('最小値未満は4にクランプされる', () => {
    expect(clampMaxLines(1)).toBe(4);
    expect(clampMaxLines(3)).toBe(4);
  });

  it('最大値超過は14にクランプされる', () => {
    expect(clampMaxLines(15)).toBe(14);
    expect(clampMaxLines(100)).toBe(14);
  });

  it('NaNは決定論的に最小値にクランプされる', () => {
    // Number.isFinite(NaN) === false なので最小値にフォールバック
    expect(clampMaxLines(NaN)).toBe(CLAMP_RANGES.maxLines.min);
  });

  it('Infinityは決定論的に最小値にクランプされる', () => {
    // Number.isFinite(Infinity) === false なので最小値にフォールバック
    expect(clampMaxLines(Infinity)).toBe(CLAMP_RANGES.maxLines.min);
  });

  it('-Infinityは決定論的に最小値にクランプされる', () => {
    // Number.isFinite(-Infinity) === false なので最小値にフォールバック
    expect(clampMaxLines(-Infinity)).toBe(CLAMP_RANGES.maxLines.min);
  });
});
