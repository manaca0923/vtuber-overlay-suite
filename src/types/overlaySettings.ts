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

// 共通設定
export interface CommonSettings {
  primaryColor: string;
  fontFamily: string;
  borderRadius: number;
}

// コメントオーバーレイ設定
export interface CommentSettings {
  enabled: boolean;
  position: CommentPosition;
  maxCount: number;
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

// オーバーレイ設定全体
export interface OverlaySettings {
  theme: ThemeName;
  common: CommonSettings;
  comment: CommentSettings;
  setlist: SetlistSettings;
}

// デフォルト設定
export const DEFAULT_OVERLAY_SETTINGS: OverlaySettings = {
  theme: 'default',
  common: {
    primaryColor: '#6366f1',
    fontFamily: "'Yu Gothic', 'Meiryo', sans-serif",
    borderRadius: 8,
  },
  comment: {
    enabled: true,
    position: 'bottom-right',
    maxCount: 10,
    showAvatar: true,
    fontSize: 16,
  },
  setlist: {
    enabled: true,
    position: 'bottom',
    showArtist: true,
    fontSize: 24,
  },
};

// Tauri Commands
export const saveOverlaySettings = (settings: OverlaySettings) =>
  invoke<void>('save_overlay_settings', { settings });

export const loadOverlaySettings = () =>
  invoke<OverlaySettings | null>('load_overlay_settings');

export const broadcastSettingsUpdate = (settings: OverlaySettings) =>
  invoke<void>('broadcast_settings_update', { settings });
