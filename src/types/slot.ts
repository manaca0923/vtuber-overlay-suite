/**
 * slot型定義
 * 3カラムレイアウト v2 の11個のslot配置システム
 */

// slot ID定義（Rust SlotIdと同期）
export const SLOT_IDS = [
  'left.top',
  'left.topBelow',
  'left.middle',
  'left.lower',
  'left.bottom',
  'center.full',
  'right.top',
  'right.upper',
  'right.lowerLeft',
  'right.lowerRight',
  'right.bottom',
] as const;

export type SlotId = (typeof SLOT_IDS)[number];

// カラム種別
export type ColumnType = 'left' | 'center' | 'right';

// slot情報
export interface SlotInfo {
  id: SlotId;
  column: ColumnType;
  role: string;
  defaultHeight: 'auto' | '1fr';
  clampBox: boolean;
}

// slot情報マスターデータ
export const SLOT_INFO: Record<SlotId, SlotInfo> = {
  'left.top': {
    id: 'left.top',
    column: 'left',
    role: '時刻',
    defaultHeight: 'auto',
    clampBox: false,
  },
  'left.topBelow': {
    id: 'left.topBelow',
    column: 'left',
    role: '天気',
    defaultHeight: 'auto',
    clampBox: false,
  },
  'left.middle': {
    id: 'left.middle',
    column: 'left',
    role: 'コメント',
    defaultHeight: '1fr',
    clampBox: true,
  },
  'left.lower': {
    id: 'left.lower',
    column: 'left',
    role: 'スパチャ',
    defaultHeight: 'auto',
    clampBox: false,
  },
  'left.bottom': {
    id: 'left.bottom',
    column: 'left',
    role: 'ロゴ',
    defaultHeight: 'auto',
    clampBox: false,
  },
  'center.full': {
    id: 'center.full',
    column: 'center',
    role: '主役',
    defaultHeight: '1fr',
    clampBox: false,
  },
  'right.top': {
    id: 'right.top',
    column: 'right',
    role: 'ラベル',
    defaultHeight: 'auto',
    clampBox: false,
  },
  'right.upper': {
    id: 'right.upper',
    column: 'right',
    role: 'セトリ',
    defaultHeight: '1fr',
    clampBox: true,
  },
  'right.lowerLeft': {
    id: 'right.lowerLeft',
    column: 'right',
    role: 'KPI',
    defaultHeight: 'auto',
    clampBox: false,
  },
  'right.lowerRight': {
    id: 'right.lowerRight',
    column: 'right',
    role: '短冊',
    defaultHeight: 'auto',
    clampBox: true,
  },
  'right.bottom': {
    id: 'right.bottom',
    column: 'right',
    role: '告知',
    defaultHeight: 'auto',
    clampBox: false,
  },
};

/**
 * slotId → CSS ID変換
 * 例: 'left.top' → 'slot-left-top'
 */
export function slotIdToCssId(slotId: SlotId): string {
  return `slot-${slotId.replace('.', '-')}`;
}

/**
 * CSS ID → slotId変換
 * 例: 'slot-left-top' → 'left.top'
 */
export function cssIdToSlotId(cssId: string): SlotId | null {
  const match = cssId.match(/^slot-(.+)-(.+)$/);
  if (!match) return null;
  const slotId = `${match[1]}.${match[2]}` as SlotId;
  return SLOT_IDS.includes(slotId) ? slotId : null;
}

/**
 * slotIdが有効かどうか
 */
export function isValidSlotId(id: string): id is SlotId {
  return SLOT_IDS.includes(id as SlotId);
}

/**
 * カラムごとのslotリストを取得
 */
export function getSlotsByColumn(column: ColumnType): SlotId[] {
  return SLOT_IDS.filter((id) => SLOT_INFO[id].column === column);
}
