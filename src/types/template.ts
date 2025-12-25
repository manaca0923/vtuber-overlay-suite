/**
 * テンプレート型定義
 * 3カラムレイアウト v2 のテンプレート設定
 *
 * 参照: docs/3カラム_要件仕様書_追記版_v1.1.md
 * Schema: src-tauri/schemas/template-mvp-1.0.json
 */

import type { SlotId } from './slot';
import { SLOT_IDS } from './slot';

// ===== コンポーネントタイプ =====

export const COMPONENT_TYPES = [
  'ClockWidget',
  'WeatherWidget',
  'ChatLog',
  'SuperChatCard',
  'BrandBlock',
  'MainAvatarStage',
  'ChannelBadge',
  'SetList',
  'KPIBlock',
  'PromoPanel',
  'QueueList',
] as const;

export type ComponentType = (typeof COMPONENT_TYPES)[number];

/**
 * コンポーネントタイプが有効かどうか
 */
export function isValidComponentType(type: string): type is ComponentType {
  return COMPONENT_TYPES.includes(type as ComponentType);
}

// ===== レイアウト設定 =====

export interface TemplateLayout {
  type: 'threeColumn';
  leftPct: number;
  centerPct: number;
  rightPct: number;
  gutterPx: number;
}

// ===== セーフエリア設定 =====

export interface TemplateSafeArea {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

// ===== テーマ設定 =====

export interface TemplatePanelStyle {
  bg?: string;
  blurPx?: number;
  radiusPx?: number;
}

export interface TemplateShadowStyle {
  enabled?: boolean;
  blur?: number;
  opacity?: number;
  offsetX?: number;
  offsetY?: number;
}

export interface TemplateOutlineStyle {
  enabled?: boolean;
  width?: number;
  color?: string;
}

export interface TemplateTheme {
  fontFamily?: string;
  textColor?: string;
  panel?: TemplatePanelStyle;
  shadow?: TemplateShadowStyle;
  outline?: TemplateOutlineStyle;
}

// ===== コンポーネント設定 =====

export interface ComponentRules {
  maxLines?: number;
  maxItems?: number;
  cycleSec?: number;
  showSec?: number;
}

export interface ComponentTuning {
  offsetX?: number;
  offsetY?: number;
}

export interface TemplateComponent {
  id: string;
  type: ComponentType;
  slot: SlotId;
  enabled: boolean;
  style?: Record<string, unknown>;
  rules?: ComponentRules;
  tuning?: ComponentTuning;
}

// ===== テンプレート全体 =====

export interface Template {
  layout: TemplateLayout;
  safeAreaPct: TemplateSafeArea;
  theme?: TemplateTheme;
  components: TemplateComponent[];
}

// ===== クランプ規約（14. クランプ規約） =====

/**
 * クランプ範囲定義
 */
export const CLAMP_RANGES = {
  // tuning
  offsetX: { min: -40, max: 40, default: 0 },
  offsetY: { min: -40, max: 40, default: 0 },
  // rules
  maxLines: { min: 4, max: 14, default: 10 },
  maxItems: { min: 6, max: 20, default: 14 },
  cycleSec: { min: 10, max: 120, default: 30 },
  showSec: { min: 3, max: 15, default: 6 },
  // layout
  leftPct: { min: 0.18, max: 0.28, default: 0.22 },
  centerPct: { min: 0.44, max: 0.64, default: 0.56 },
  rightPct: { min: 0.18, max: 0.28, default: 0.22 },
  gutterPx: { min: 0, max: 64, default: 24 },
  // safeArea
  safeArea: { min: 0.0, max: 0.1, default: 0.04 },
  // theme.panel
  blurPx: { min: 0, max: 24, default: 10 },
  radiusPx: { min: 0, max: 32, default: 14 },
  // theme.shadow
  shadowBlur: { min: 0, max: 24, default: 8 },
  shadowOpacity: { min: 0.0, max: 1.0, default: 0.55 },
  shadowOffset: { min: -20, max: 20, default: 2 },
  // theme.outline
  outlineWidth: { min: 0, max: 6, default: 2 },
} as const;

/**
 * 汎用クランプ関数
 */
function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

// ===== クランプ関数（tuning） =====

export function clampOffsetX(value: number): number {
  return clamp(value, CLAMP_RANGES.offsetX.min, CLAMP_RANGES.offsetX.max);
}

export function clampOffsetY(value: number): number {
  return clamp(value, CLAMP_RANGES.offsetY.min, CLAMP_RANGES.offsetY.max);
}

// ===== クランプ関数（rules） =====

export function clampMaxLines(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.maxLines.min, CLAMP_RANGES.maxLines.max);
}

export function clampMaxItems(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.maxItems.min, CLAMP_RANGES.maxItems.max);
}

export function clampCycleSec(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.cycleSec.min, CLAMP_RANGES.cycleSec.max);
}

export function clampShowSec(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.showSec.min, CLAMP_RANGES.showSec.max);
}

// ===== クランプ関数（layout） =====

export function clampLeftPct(value: number): number {
  return clamp(value, CLAMP_RANGES.leftPct.min, CLAMP_RANGES.leftPct.max);
}

export function clampCenterPct(value: number): number {
  return clamp(value, CLAMP_RANGES.centerPct.min, CLAMP_RANGES.centerPct.max);
}

export function clampRightPct(value: number): number {
  return clamp(value, CLAMP_RANGES.rightPct.min, CLAMP_RANGES.rightPct.max);
}

export function clampGutterPx(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.gutterPx.min, CLAMP_RANGES.gutterPx.max);
}

// ===== クランプ関数（safeArea） =====

export function clampSafeArea(value: number): number {
  return clamp(value, CLAMP_RANGES.safeArea.min, CLAMP_RANGES.safeArea.max);
}

// ===== クランプ関数（theme.panel） =====

export function clampBlurPx(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.blurPx.min, CLAMP_RANGES.blurPx.max);
}

export function clampRadiusPx(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.radiusPx.min, CLAMP_RANGES.radiusPx.max);
}

// ===== クランプ関数（theme.shadow） =====

export function clampShadowBlur(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.shadowBlur.min, CLAMP_RANGES.shadowBlur.max);
}

export function clampShadowOpacity(value: number): number {
  return clamp(value, CLAMP_RANGES.shadowOpacity.min, CLAMP_RANGES.shadowOpacity.max);
}

export function clampShadowOffset(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.shadowOffset.min, CLAMP_RANGES.shadowOffset.max);
}

// ===== クランプ関数（theme.outline） =====

export function clampOutlineWidth(value: number): number {
  return clamp(Math.round(value), CLAMP_RANGES.outlineWidth.min, CLAMP_RANGES.outlineWidth.max);
}

// ===== バリデーション =====

/**
 * themeをクランプ
 */
function clampTheme(theme: TemplateTheme | undefined): TemplateTheme | undefined {
  if (!theme) return undefined;

  const clamped: TemplateTheme = {
    fontFamily: theme.fontFamily,
    textColor: theme.textColor,
  };

  // panel クランプ
  if (theme.panel) {
    clamped.panel = {
      bg: theme.panel.bg,
    };
    if (theme.panel.blurPx !== undefined) {
      clamped.panel.blurPx = clampBlurPx(theme.panel.blurPx);
    }
    if (theme.panel.radiusPx !== undefined) {
      clamped.panel.radiusPx = clampRadiusPx(theme.panel.radiusPx);
    }
  }

  // shadow クランプ
  if (theme.shadow) {
    clamped.shadow = {
      enabled: theme.shadow.enabled,
    };
    if (theme.shadow.blur !== undefined) {
      clamped.shadow.blur = clampShadowBlur(theme.shadow.blur);
    }
    if (theme.shadow.opacity !== undefined) {
      clamped.shadow.opacity = clampShadowOpacity(theme.shadow.opacity);
    }
    if (theme.shadow.offsetX !== undefined) {
      clamped.shadow.offsetX = clampShadowOffset(theme.shadow.offsetX);
    }
    if (theme.shadow.offsetY !== undefined) {
      clamped.shadow.offsetY = clampShadowOffset(theme.shadow.offsetY);
    }
  }

  // outline クランプ
  if (theme.outline) {
    clamped.outline = {
      enabled: theme.outline.enabled,
      color: theme.outline.color,
    };
    if (theme.outline.width !== undefined) {
      clamped.outline.width = clampOutlineWidth(theme.outline.width);
    }
  }

  return clamped;
}

/**
 * テンプレートをバリデーション＆クランプ
 * 不正な値はクランプして適用
 *
 * 注: フロントエンドでの簡易バリデーション用。
 * 本番ではRust側のvalidate_templateコマンドを使用することを推奨。
 */
export function validateAndClampTemplate(template: Template): Template {
  // layout クランプ（typeは'threeColumn'に強制）
  const layout: TemplateLayout = {
    type: 'threeColumn',
    leftPct: clampLeftPct(template.layout.leftPct),
    centerPct: clampCenterPct(template.layout.centerPct),
    rightPct: clampRightPct(template.layout.rightPct),
    gutterPx: clampGutterPx(template.layout.gutterPx),
  };

  // safeAreaPct クランプ
  const safeAreaPct: TemplateSafeArea = {
    top: clampSafeArea(template.safeAreaPct.top),
    right: clampSafeArea(template.safeAreaPct.right),
    bottom: clampSafeArea(template.safeAreaPct.bottom),
    left: clampSafeArea(template.safeAreaPct.left),
  };

  // theme クランプ
  const theme = clampTheme(template.theme);

  // components クランプ
  const components = template.components.map((comp) => {
    const clamped: TemplateComponent = {
      id: comp.id,
      type: comp.type,
      slot: comp.slot,
      enabled: comp.enabled,
      style: comp.style,
    };

    // rules クランプ
    if (comp.rules) {
      clamped.rules = {};
      if (comp.rules.maxLines !== undefined) {
        clamped.rules.maxLines = clampMaxLines(comp.rules.maxLines);
      }
      if (comp.rules.maxItems !== undefined) {
        clamped.rules.maxItems = clampMaxItems(comp.rules.maxItems);
      }
      if (comp.rules.cycleSec !== undefined) {
        clamped.rules.cycleSec = clampCycleSec(comp.rules.cycleSec);
      }
      if (comp.rules.showSec !== undefined) {
        clamped.rules.showSec = clampShowSec(comp.rules.showSec);
      }
    }

    // tuning クランプ
    if (comp.tuning) {
      clamped.tuning = {};
      if (comp.tuning.offsetX !== undefined) {
        clamped.tuning.offsetX = clampOffsetX(comp.tuning.offsetX);
      }
      if (comp.tuning.offsetY !== undefined) {
        clamped.tuning.offsetY = clampOffsetY(comp.tuning.offsetY);
      }
    }

    return clamped;
  });

  return {
    layout,
    safeAreaPct,
    theme,
    components,
  };
}

/**
 * コンポーネントのslotが有効かチェック
 */
export function isValidComponentSlot(comp: TemplateComponent): boolean {
  return SLOT_IDS.includes(comp.slot);
}

/**
 * テンプレート内でslotの重複がないかチェック
 * （enabled=trueのコンポーネントのみ対象）
 */
export function hasSlotDuplicates(components: TemplateComponent[]): boolean {
  const usedSlots = new Set<SlotId>();
  for (const comp of components) {
    if (comp.enabled) {
      if (usedSlots.has(comp.slot)) {
        return true;
      }
      usedSlots.add(comp.slot);
    }
  }
  return false;
}

/**
 * テンプレート内でコンポーネントIDの重複がないかチェック
 */
export function hasIdDuplicates(components: TemplateComponent[]): boolean {
  const usedIds = new Set<string>();
  for (const comp of components) {
    if (usedIds.has(comp.id)) {
      return true;
    }
    usedIds.add(comp.id);
  }
  return false;
}

// ===== デフォルトテンプレート =====

/**
 * デフォルトテンプレート
 *
 * 注: componentsは空配列で初期化。
 * これはJSON Schema（minItems: 1）に違反しますが、
 * テンプレート作成UIの初期状態として使用するため意図的です。
 * 実際のテンプレート保存時はcomponentsが追加されることを想定しています。
 */
export const DEFAULT_TEMPLATE: Template = {
  layout: {
    type: 'threeColumn',
    leftPct: CLAMP_RANGES.leftPct.default,
    centerPct: CLAMP_RANGES.centerPct.default,
    rightPct: CLAMP_RANGES.rightPct.default,
    gutterPx: CLAMP_RANGES.gutterPx.default,
  },
  safeAreaPct: {
    top: CLAMP_RANGES.safeArea.default,
    right: CLAMP_RANGES.safeArea.default,
    bottom: 0.05,
    left: CLAMP_RANGES.safeArea.default,
  },
  components: [],
};
