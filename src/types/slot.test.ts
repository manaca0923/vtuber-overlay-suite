/**
 * slot.ts のユニットテスト
 */

import { describe, it, expect } from 'vitest';
import {
  SLOT_IDS,
  slotIdToCssId,
  cssIdToSlotId,
  isValidSlotId,
  getSlotsByColumn,
} from './slot';

describe('slot.ts', () => {
  describe('slotIdToCssId', () => {
    it('ドット区切りをハイフン区切りに変換する', () => {
      expect(slotIdToCssId('left.top')).toBe('slot-left-top');
      expect(slotIdToCssId('center.full')).toBe('slot-center-full');
      expect(slotIdToCssId('right.kpi')).toBe('slot-right-kpi');
    });

    it('全てのSLOT_IDを正しく変換できる', () => {
      for (const slotId of SLOT_IDS) {
        const cssId = slotIdToCssId(slotId);
        expect(cssId).toMatch(/^slot-[a-z]+-[a-zA-Z]+$/);
      }
    });
  });

  describe('cssIdToSlotId', () => {
    it('ハイフン区切りをドット区切りに変換する', () => {
      expect(cssIdToSlotId('slot-left-top')).toBe('left.top');
      expect(cssIdToSlotId('slot-center-full')).toBe('center.full');
      expect(cssIdToSlotId('slot-right-kpi')).toBe('right.kpi');
    });

    it('無効なCSS IDに対してnullを返す', () => {
      expect(cssIdToSlotId('invalid')).toBeNull();
      expect(cssIdToSlotId('left-top')).toBeNull(); // slot-プレフィックスなし
      expect(cssIdToSlotId('slot-')).toBeNull();
      expect(cssIdToSlotId('slot-left')).toBeNull(); // slot名なし
    });

    it('存在しないslot IDに対してnullを返す', () => {
      expect(cssIdToSlotId('slot-left-invalid')).toBeNull();
      expect(cssIdToSlotId('slot-unknown-top')).toBeNull();
    });

    it('全てのSLOT_IDを往復変換できる', () => {
      for (const slotId of SLOT_IDS) {
        const cssId = slotIdToCssId(slotId);
        const converted = cssIdToSlotId(cssId);
        expect(converted).toBe(slotId);
      }
    });

    it('全slotのCSS ID変換をテスト', () => {
      // 全てのslot IDを往復変換
      expect(cssIdToSlotId('slot-right-kpi')).toBe('right.kpi');
      expect(cssIdToSlotId('slot-right-tanzaku')).toBe('right.tanzaku');
    });
  });

  describe('isValidSlotId', () => {
    it('有効なslot IDに対してtrueを返す', () => {
      for (const slotId of SLOT_IDS) {
        expect(isValidSlotId(slotId)).toBe(true);
      }
    });

    it('無効なslot IDに対してfalseを返す', () => {
      expect(isValidSlotId('invalid')).toBe(false);
      expect(isValidSlotId('left.invalid')).toBe(false);
      expect(isValidSlotId('unknown.top')).toBe(false);
      expect(isValidSlotId('')).toBe(false);
    });
  });

  describe('getSlotsByColumn', () => {
    it('左カラムのslotを取得できる', () => {
      const leftSlots = getSlotsByColumn('left');
      expect(leftSlots).toContain('left.top');
      expect(leftSlots).toContain('left.topBelow');
      expect(leftSlots).toContain('left.middle');
      expect(leftSlots).toContain('left.lower');
      expect(leftSlots).toContain('left.bottom');
      expect(leftSlots).toHaveLength(5);
    });

    it('中央カラムのslotを取得できる', () => {
      const centerSlots = getSlotsByColumn('center');
      expect(centerSlots).toContain('center.full');
      expect(centerSlots).toHaveLength(1);
    });

    it('右カラムのslotを取得できる', () => {
      const rightSlots = getSlotsByColumn('right');
      expect(rightSlots).toContain('right.top');
      expect(rightSlots).toContain('right.upper');
      expect(rightSlots).toContain('right.kpi');
      expect(rightSlots).toContain('right.tanzaku');
      expect(rightSlots).toContain('right.bottom');
      expect(rightSlots).toHaveLength(5);
    });
  });

  describe('SLOT_IDS', () => {
    it('11個のslot IDが定義されている', () => {
      expect(SLOT_IDS).toHaveLength(11);
    });

    it('全て一意である', () => {
      const uniqueIds = new Set(SLOT_IDS);
      expect(uniqueIds.size).toBe(SLOT_IDS.length);
    });
  });
});
