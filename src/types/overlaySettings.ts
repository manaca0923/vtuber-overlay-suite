import { invoke } from '@tauri-apps/api/core';

// テーマプリセット定義
export const THEME_PRESETS = {
  default: {
    name: 'デフォルト',
    primaryColor: '#6366f1',
    description: 'パープル系の落ち着いたデザイン',
  },
  sakura: {
    name: 'Sakura',
    primaryColor: '#ec4899',
    description: 'ピンク系の可愛らしいデザイン',
  },
  ocean: {
    name: 'Ocean',
    primaryColor: '#0ea5e9',
    description: 'ブルー系の爽やかなデザイン',
  },
} as const;

export type ThemeName = keyof typeof THEME_PRESETS | 'custom';

// コメント位置
export type CommentPosition = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

// セットリスト位置
export type SetlistPosition = 'top' | 'bottom' | 'left' | 'right';

// 天気ウィジェット位置（3カラムレイアウト用）
export type WeatherPosition = 'left-top' | 'left-bottom' | 'right-top' | 'right-bottom';

// レイアウトプリセット
export type LayoutPreset = 'custom' | 'three-column';

// レイアウトバージョン
export type LayoutVersion = 'v1' | 'v2';

// レイアウトプリセット設定
export interface LayoutPresetConfig {
  name: string;
  description: string;
  version?: LayoutVersion; // v1（デフォルト）または v2（3カラム）
  comment: {
    position: CommentPosition;
    enabled: boolean;
  };
  setlist: {
    position: SetlistPosition;
    enabled: boolean;
  };
}

// レイアウトプリセット定義
export const LAYOUT_PRESETS: Record<LayoutPreset, LayoutPresetConfig> = {
  'three-column': {
    name: '3カラム',
    description: '左22%/中央56%/右22%の固定レイアウト',
    version: 'v2',
    comment: { position: 'bottom-left', enabled: true },
    setlist: { position: 'right', enabled: true },
  },
  custom: {
    name: 'カスタム',
    description: '個別に設定をカスタマイズ',
    comment: { position: 'bottom-right', enabled: true },
    setlist: { position: 'bottom', enabled: true },
  },
};

// 共通設定
export interface CommonSettings {
  primaryColor: string;
  fontFamily: string;
  borderRadius: number;
}

// コメントオーバーレイ設定
// NOTE: maxCountは画面高さベースの自動調整に統一したため削除
export interface CommentSettings {
  enabled: boolean;
  position: CommentPosition;
  showAvatar: boolean;
  fontSize: number;
}

// セットリストオーバーレイ設定
export interface SetlistSettings {
  enabled: boolean;
  position: SetlistPosition;
  showArtist: boolean;
  fontSize: number;
}

// 天気ウィジェット設定
export interface WeatherSettings {
  enabled: boolean;
  position: WeatherPosition;
}

// パフォーマンス設定
export interface PerformanceSettings {
  densityThreshold: number; // 過密検出閾値（1-20、デフォルト: 5）
}

// スパチャ設定
export interface SuperchatSettings {
  maxDisplay: number; // 同時表示数（1-3、デフォルト: 1）
  displayDurationSec: number; // 表示時間（秒、10-120、デフォルト: 60）
  queueEnabled: boolean; // キュー表示ON/OFF（待機中のスパチャを順次表示）
}

// ウィジェット表示設定
export interface WidgetVisibilitySettings {
  clock: boolean; // left.top: 時計
  weather: boolean; // left.topBelow: 天気
  comment: boolean; // left.middle: コメント
  superchat: boolean; // left.lower: スパチャ
  logo: boolean; // left.bottom: ロゴ
  setlist: boolean; // right.upper: セトリ
  kpi: boolean; // right.lowerLeft: KPI
  tanzaku: boolean; // right.lowerRight: 短冊
  announcement: boolean; // right.bottom: 告知
}

// オーバーレイ設定全体
export interface OverlaySettings {
  theme: ThemeName;
  layout: LayoutPreset;
  common: CommonSettings;
  comment: CommentSettings;
  setlist: SetlistSettings;
  weather?: WeatherSettings; // オプショナル（後方互換性のため）
  performance?: PerformanceSettings; // オプショナル（後方互換性のため）
  widget?: WidgetVisibilitySettings; // オプショナル（後方互換性のため）
  superchat?: SuperchatSettings; // オプショナル（後方互換性のため）
}

// デフォルト設定
export const DEFAULT_OVERLAY_SETTINGS: OverlaySettings = {
  theme: 'default',
  layout: 'three-column',
  common: {
    primaryColor: '#6366f1',
    fontFamily: "'Yu Gothic', 'Meiryo', sans-serif",
    borderRadius: 8,
  },
  comment: {
    enabled: true,
    position: 'bottom-right',
    showAvatar: true,
    fontSize: 16,
  },
  setlist: {
    enabled: true,
    position: 'bottom',
    showArtist: true,
    fontSize: 24,
  },
  weather: {
    enabled: true,
    position: 'left-top',
  },
  performance: {
    densityThreshold: 5, // デフォルト: 2秒間に5回更新で高負荷と判定
  },
  widget: {
    clock: true,
    weather: true,
    comment: true,
    superchat: true,
    logo: true,
    setlist: true,
    kpi: true,
    tanzaku: true,
    announcement: true,
  },
  superchat: {
    maxDisplay: 1, // 同時表示1件
    displayDurationSec: 60, // 60秒表示
    queueEnabled: true, // キュー表示ON
  },
};

// Tauri Commands
export const saveOverlaySettings = (settings: OverlaySettings) =>
  invoke<void>('save_overlay_settings', { settings });

export const loadOverlaySettings = () =>
  invoke<OverlaySettings | null>('load_overlay_settings');

export const broadcastSettingsUpdate = (settings: OverlaySettings) =>
  invoke<void>('broadcast_settings_update', { settings });
