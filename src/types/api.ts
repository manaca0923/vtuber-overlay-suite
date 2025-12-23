/**
 * YouTube API関連の型定義
 * @see src-tauri/src/commands/youtube.rs ApiMode
 */

/**
 * APIモード
 * - innertube: 非公式API（APIキー不要、クォータ無制限）
 * - official: 公式API ポーリング（APIキー必須）
 * - grpc: 公式API gRPCストリーミング（推奨、低遅延）
 */
export type ApiMode = 'innertube' | 'official' | 'grpc';

/**
 * API設定
 */
export interface ApiSettings {
  mode: ApiMode;
  useBundledKey: boolean;
  userApiKey?: string;
}

/**
 * InnerTubeステータスイベント
 * @see src-tauri/src/youtube/unified_poller.rs run_innertube_loop
 */
export interface InnerTubeStatusEvent {
  connected: boolean;
  error?: string;
  stopped?: boolean;
}

/**
 * gRPCステータスイベント
 * @see src-tauri/src/youtube/grpc/poller.rs run_grpc_stream
 */
export interface GrpcStatusEvent {
  connected: boolean;
  liveChatId?: string;
  error?: string;
}

/**
 * Officialステータスイベント
 * @see src-tauri/src/youtube/unified_poller.rs start_official
 */
export interface OfficialStatusEvent {
  connected: boolean;
  liveChatId?: string;
  error?: string;
  stopped?: boolean;
  reason?: string;
  retrying?: boolean;
  quotaExceeded?: boolean;
  streamEnded?: boolean;
  quotaUsed?: number;
  remainingQuota?: number;
  pollCount?: number;
}

/**
 * APIモードの表示情報
 */
export const API_MODE_INFO: Record<ApiMode, {
  label: string;
  description: string;
  requiresApiKey: boolean;
  recommended?: boolean;
}> = {
  innertube: {
    label: 'InnerTube (非公式)',
    description: 'APIキー不要、クォータ無制限、仕様変更リスクあり',
    requiresApiKey: false,
  },
  grpc: {
    label: '公式API (gRPC)',
    description: 'リアルタイム、低遅延、安定',
    requiresApiKey: true,
    recommended: true,
  },
  official: {
    label: '公式API (ポーリング)',
    description: 'gRPC非対応環境向け',
    requiresApiKey: true,
  },
};
