import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ChatMessage } from '../types/chat';

// YouTube API クォータ定数
const DAILY_QUOTA_LIMIT = 10000; // 1日のクォータ上限
const QUOTA_PER_REQUEST = 5; // 1リクエストあたりのコスト
const SAVED_STATE_EXPIRY_HOURS = 6; // 保存状態の有効期限（時間）

interface PollingState {
  next_page_token: string | null;
  polling_interval_millis: number;
  live_chat_id: string;
  quota_used: number;
  poll_count: number;
}

type PollingEventType =
  | { type: 'Started'; live_chat_id: string }
  | { type: 'Stopped' }
  | { type: 'Messages'; messages: ChatMessage[] }
  | { type: 'StateUpdate'; state: PollingState }
  | { type: 'Error'; message: string }
  | { type: 'RateLimited'; retry_after_ms: number }
  | { type: 'QuotaExceeded' }
  | { type: 'LiveEnded' };

interface SavedPollingState {
  live_chat_id: string;
  next_page_token: string | null;
  quota_used: number;
  saved_at: string;
}

/**
 * 保存された状態が有効期限内かどうかを判定
 */
function isSavedStateValid(savedAt: string): boolean {
  const savedDate = new Date(savedAt);
  const hoursOld = (Date.now() - savedDate.getTime()) / (1000 * 60 * 60);
  return hoursOld < SAVED_STATE_EXPIRY_HOURS;
}

interface CommentControlPanelProps {
  apiKey: string;
  videoId: string;
  liveChatId: string;
}

export function CommentControlPanel({
  apiKey,
  videoId,
  liveChatId,
}: CommentControlPanelProps) {
  const [isPolling, setIsPolling] = useState(false);
  const [pollingState, setPollingState] = useState<PollingState | null>(null);
  const [savedState, setSavedState] = useState<SavedPollingState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastEvent, setLastEvent] = useState<string>('');
  const isMountedRef = useRef(true);

  // ポーリング状態を監視
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    async function setupListener() {
      try {
        unlisten = await listen<PollingEventType>('polling-event', (event) => {
          if (!isMountedRef.current) return;

          const payload = event.payload;
          switch (payload.type) {
            case 'Started':
              setIsPolling(true);
              setError(null);
              setLastEvent('ポーリング開始');
              break;
            case 'Stopped':
              setIsPolling(false);
              setLastEvent('ポーリング停止');
              break;
            case 'Messages':
              setLastEvent(`${payload.messages.length}件のコメントを取得`);
              break;
            case 'StateUpdate':
              setPollingState(payload.state);
              break;
            case 'Error':
              setError(payload.message);
              setLastEvent(`エラー: ${payload.message}`);
              break;
            case 'RateLimited':
              setLastEvent(
                `レート制限中 (${Math.round(payload.retry_after_ms / 1000)}秒後に再試行)`
              );
              break;
            case 'QuotaExceeded':
              setError('クォータ超過: 本日のAPI使用量が上限に達しました');
              setIsPolling(false);
              setLastEvent('クォータ超過');
              break;
            case 'LiveEnded':
              setIsPolling(false);
              setLastEvent('配信が終了しました');
              break;
          }
        });
      } catch (err) {
        console.error('Failed to setup event listener:', err);
      }
    }

    setupListener();

    return () => {
      isMountedRef.current = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // 現在のポーリング状態を確認
  useEffect(() => {
    async function checkPollingStatus() {
      try {
        const running = await invoke<boolean>('is_polling_running');
        if (isMountedRef.current) {
          setIsPolling(running);
        }
        if (running) {
          const state = await invoke<PollingState | null>('get_polling_state');
          if (isMountedRef.current && state) {
            setPollingState(state);
          }
        }
      } catch (err) {
        console.error('Failed to check polling status:', err);
      }
    }
    checkPollingStatus();
  }, []);

  // 保存されたポーリング状態を読み込む（有効期限チェック付き）
  useEffect(() => {
    async function loadSavedState() {
      try {
        const saved = await invoke<SavedPollingState | null>('load_polling_state');
        if (
          isMountedRef.current &&
          saved &&
          saved.live_chat_id === liveChatId &&
          isSavedStateValid(saved.saved_at)
        ) {
          setSavedState(saved);
        }
      } catch (err) {
        console.error('Failed to load saved polling state:', err);
      }
    }
    if (liveChatId) {
      loadSavedState();
    }
  }, [liveChatId]);

  const handleStartPolling = useCallback(
    async (useSavedState: boolean = false) => {
      if (!apiKey || !liveChatId) {
        setError('APIキーとLive Chat IDが必要です');
        return;
      }

      setLoading(true);
      setError(null);

      try {
        await invoke('start_polling', {
          api_key: apiKey,
          live_chat_id: liveChatId,
          next_page_token: useSavedState ? savedState?.next_page_token : null,
          quota_used: useSavedState ? savedState?.quota_used : null,
        });
        if (isMountedRef.current) {
          setIsPolling(true);
          setSavedState(null); // 開始後は保存された状態をクリア
        }
      } catch (err) {
        if (isMountedRef.current) {
          const errorMessage = err instanceof Error ? err.message : String(err);
          setError(`ポーリング開始エラー: ${errorMessage}`);
        }
      } finally {
        if (isMountedRef.current) {
          setLoading(false);
        }
      }
    },
    [apiKey, liveChatId, savedState]
  );

  const handleStopPolling = useCallback(async () => {
    setLoading(true);

    try {
      // 現在の状態を保存してから停止
      if (pollingState && liveChatId) {
        await invoke('save_polling_state', {
          live_chat_id: liveChatId,
          next_page_token: pollingState.next_page_token,
          quota_used: pollingState.quota_used,
        });
      }

      await invoke('stop_polling');
      if (isMountedRef.current) {
        setIsPolling(false);
      }
    } catch (err) {
      if (isMountedRef.current) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`ポーリング停止エラー: ${errorMessage}`);
      }
    } finally {
      if (isMountedRef.current) {
        setLoading(false);
      }
    }
  }, [pollingState, liveChatId]);

  // クォータ情報を計算
  const estimatedRemainingQuota = pollingState
    ? DAILY_QUOTA_LIMIT - pollingState.quota_used
    : DAILY_QUOTA_LIMIT;
  const estimatedRemainingPolls = Math.floor(estimatedRemainingQuota / QUOTA_PER_REQUEST);
  const quotaPercentage = pollingState
    ? (pollingState.quota_used / DAILY_QUOTA_LIMIT) * 100
    : 0;

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-xl font-bold mb-4">コメント取得制御</h2>

      {/* 接続情報 */}
      <div className="mb-4 p-3 bg-gray-50 rounded-lg text-sm">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <span className="text-gray-500">動画ID:</span>{' '}
            <span className="font-mono">{videoId || '-'}</span>
          </div>
          <div>
            <span className="text-gray-500">Chat ID:</span>{' '}
            <span className="font-mono text-xs">
              {liveChatId ? liveChatId.substring(0, 20) + '...' : '-'}
            </span>
          </div>
        </div>
      </div>

      {/* ポーリング状態 */}
      <div className="mb-4 flex items-center gap-3">
        <div
          className={`w-3 h-3 rounded-full ${isPolling ? 'bg-green-500 animate-pulse' : 'bg-gray-400'}`}
        />
        <span className="font-medium">
          {isPolling ? '取得中' : '停止中'}
        </span>
        {lastEvent && <span className="text-sm text-gray-500">{lastEvent}</span>}
      </div>

      {/* 保存された状態がある場合の再開オプション */}
      {savedState && !isPolling && (
        <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-yellow-800">
                前回の取得状態があります
              </p>
              <p className="text-xs text-yellow-600">
                保存日時: {new Date(savedState.saved_at).toLocaleString('ja-JP')}
                {' | '}クォータ使用量: {savedState.quota_used.toLocaleString()} units
              </p>
            </div>
            <button
              onClick={() => handleStartPolling(true)}
              disabled={loading || !apiKey || !liveChatId}
              className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                loading || !apiKey || !liveChatId
                  ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                  : 'bg-yellow-600 text-white hover:bg-yellow-700'
              }`}
            >
              続きから開始
            </button>
          </div>
        </div>
      )}

      {/* 制御ボタン */}
      <div className="flex gap-3 mb-4">
        <button
          onClick={() => handleStartPolling(false)}
          disabled={loading || isPolling || !apiKey || !liveChatId}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            loading || isPolling || !apiKey || !liveChatId
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-green-600 text-white hover:bg-green-700'
          }`}
        >
          {loading ? '処理中...' : '最初から開始'}
        </button>
        <button
          onClick={handleStopPolling}
          disabled={loading || !isPolling}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            loading || !isPolling
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-red-600 text-white hover:bg-red-700'
          }`}
        >
          {loading ? '処理中...' : '停止'}
        </button>
      </div>

      {/* クォータ情報 */}
      {pollingState && (
        <div className="mb-4 p-3 bg-blue-50 rounded-lg">
          <div className="flex justify-between items-center mb-2">
            <span className="text-sm font-medium text-blue-800">
              クォータ使用量
            </span>
            <span className="text-sm text-blue-600">
              {pollingState.quota_used.toLocaleString()} / {DAILY_QUOTA_LIMIT.toLocaleString()} units
            </span>
          </div>
          <div className="w-full bg-blue-200 rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all ${
                quotaPercentage > 80
                  ? 'bg-red-500'
                  : quotaPercentage > 50
                    ? 'bg-yellow-500'
                    : 'bg-blue-500'
              }`}
              style={{ width: `${Math.min(quotaPercentage, 100)}%` }}
            />
          </div>
          <div className="mt-2 text-xs text-blue-600">
            残り約 {estimatedRemainingPolls.toLocaleString()} 回取得可能 |
            ポーリング間隔: {pollingState.polling_interval_millis / 1000}秒
          </div>
        </div>
      )}

      {/* エラー表示 */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {error}
        </div>
      )}
    </div>
  );
}
