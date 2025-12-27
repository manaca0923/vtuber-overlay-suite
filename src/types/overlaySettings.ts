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

// オーバーレイ設定全体
export interface OverlaySettings {
  theme: ThemeName;
  layout: LayoutPreset;
  common: CommonSettings;
  comment: CommentSettings;
  setlist: SetlistSettings;
  weather?: WeatherSettings; // オプショナル（後方互換性のため）
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
};

// Tauri Commands
export const saveOverlaySettings = (settings: OverlaySettings) =>
  invoke<void>('save_overlay_settings', { settings });

export const loadOverlaySettings = () =>
  invoke<OverlaySettings | null>('load_overlay_settings');

export const broadcastSettingsUpdate = (settings: OverlaySettings) =>
  invoke<void>('broadcast_settings_update', { settings });
