import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ChatMessage } from '../types/chat';
import type { ApiMode, InnerTubeStatusEvent, GrpcStatusEvent, OfficialStatusEvent } from '../types/api';
import { API_MODE_INFO } from '../types/api';

// YouTube API クォータ定数
const DAILY_QUOTA_LIMIT = 10000; // 1日のクォータ上限
const QUOTA_PER_REQUEST = 5; // 1リクエストあたりのコスト
const SAVED_STATE_EXPIRY_HOURS = 24; // 保存状態の有効期限（時間）- バックエンドと同期

interface PollingState {
  next_page_token: string | null;
  polling_interval_millis: number;
  live_chat_id: string;
  quota_used: number;
  poll_count: number;
}

/**
 * ポーリングイベント型（Rust側のPollingEventと同期）
 * @see src-tauri/src/youtube/poller.rs
 */
type PollingEventType =
  | { type: 'started'; live_chat_id: string }
  | { type: 'stopped'; reason: string }
  | { type: 'messages'; messages: ChatMessage[] }
  | {
      type: 'stateUpdate';
      quota_used: number;
      remaining_quota: number;
      poll_count: number;
      next_page_token: string | null;
      polling_interval_millis: number;
    }
  | { type: 'error'; message: string; retrying: boolean }
  | { type: 'quotaExceeded' }
  | { type: 'streamEnded' };

interface SavedPollingState {
  live_chat_id: string;
  next_page_token: string | null;
  quota_used: number;
  polling_interval_millis?: number;
  saved_at: string;
}

/**
 * 保存された状態が有効期限内かどうかを判定
 * @param savedAt ISO 8601形式の日付文字列
 * @returns 有効期限内ならtrue、無効な日付や期限切れの場合はfalse
 */
function isSavedStateValid(savedAt: string): boolean {
  const savedDate = new Date(savedAt);
  // 無効な日付の場合はfalseを返す
  if (isNaN(savedDate.getTime())) {
    console.warn('Invalid saved_at date:', savedAt);
    return false;
  }
  const hoursOld = (Date.now() - savedDate.getTime()) / (1000 * 60 * 60);
  return hoursOld < SAVED_STATE_EXPIRY_HOURS;
}

interface CommentControlPanelProps {
  apiKey: string;
  videoId: string;
  liveChatId: string;
  isPolling: boolean;
  setIsPolling: React.Dispatch<React.SetStateAction<boolean>>;
  onSettingsChange?: (settings: { videoId: string; liveChatId: string }) => void;
}

export function CommentControlPanel({
  apiKey,
  videoId,
  liveChatId,
  isPolling,
  setIsPolling,
  onSettingsChange,
}: CommentControlPanelProps) {
  // isPollingは親コンポーネント（App.tsx）から受け取る
  // これによりタブ切り替え時も状態が維持される

  const [pollingState, setPollingState] = useState<PollingState | null>(null);
  // NOTE: savedState は「続きから開始」ボタン用（現在統合ポーラー未サポートのためUI非表示）
  // 将来的に保存状態復元機能を実装する際に復活させる
  // TODO: 統合ポーラーで保存状態復元をサポートしたら [savedState, setSavedState] に変更
  const [, setSavedState] = useState<SavedPollingState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastEvent, setLastEvent] = useState<string>('');
  const isMountedRef = useRef(true);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // 編集用の動画ID/URL入力
  const [editVideoInput, setEditVideoInput] = useState('');
  const [isUpdating, setIsUpdating] = useState(false);

  // APIモード関連
  const [apiMode, setApiMode] = useState<ApiMode>('innertube');
  const [useBundledKey, setUseBundledKey] = useState(true);
  const [connectionStatus, setConnectionStatus] = useState<'disconnected' | 'connected' | 'error'>('disconnected');

  // コンポーネントのマウント状態を管理
  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  // isPollingが変化したらconnectionStatusを同期
  useEffect(() => {
    if (isPolling) {
      setConnectionStatus('connected');
      // ポーリング中のAPIモードを取得
      invoke<ApiMode | null>('get_unified_polling_mode')
        .then((mode) => {
          if (isMountedRef.current && mode) {
            setApiMode(mode);
          }
        })
        .catch((err) => console.error('Failed to get unified polling mode:', err));
    }
  }, [isPolling]);

  // ポーリング中は30秒ごとに視聴者数・高評価数を取得してブロードキャスト
  // InnerTubeモードではAPIキー不要のためスキップ
  useEffect(() => {
    if (!isPolling || !videoId) return;
    // InnerTubeモードではYouTube Data APIを使用しないためスキップ
    if (apiMode === 'innertube') return;

    // 初回は即座に取得
    const fetchViewerCount = async () => {
      try {
        await invoke('fetch_and_broadcast_viewer_count', {
          video_id: videoId,
          use_bundled_key: useBundledKey,
        });
      } catch (err) {
        console.warn('Failed to fetch viewer count:', err);
        // 取得失敗はログのみ（KPI表示は必須機能ではない）
      }
    };

    fetchViewerCount();

    // 30秒ごとに定期取得（クォータ節約とリアルタイム性のバランス）
    const VIEWER_COUNT_INTERVAL_MS = 30_000;
    const intervalId = setInterval(fetchViewerCount, VIEWER_COUNT_INTERVAL_MS);

    return () => {
      clearInterval(intervalId);
    };
  }, [isPolling, videoId, apiMode, useBundledKey]);

  // 初期設定を読み込み（APIモード、useBundledKey）
  // NOTE: このuseEffectは初回マウント時のみ実行する意図のため、依存配列は空のままにする。
  // isPollingはマウント時の初期値（親から受け取った値）を参照している。
  // マウント時にポーリング中であれば保存されたモードで上書きしない（現在のモードを維持）。
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => {
    async function loadInitialSettings() {
      // APIモードを読み込み（ポーリング中でない場合のみ更新）
      try {
        const savedMode = await invoke<ApiMode>('load_api_mode');
        if (isMountedRef.current && !isPolling) {
          setApiMode(savedMode);
        }
      } catch (err) {
        console.error('Failed to load API mode:', err);
      }

      // useBundledKeyを読み込み（wizard_settingsから）
      try {
        const settings = await invoke<{
          video_id: string;
          live_chat_id: string;
          saved_at: string;
          use_bundled_key?: boolean | null;
        } | null>('load_wizard_settings');
        // use_bundled_keyがbooleanの場合のみ設定（nullやundefinedはデフォルト値を維持）
        // WizardやApiKeySetupはnullを渡すため、nullをデフォルト値(true)として扱う
        if (isMountedRef.current && typeof settings?.use_bundled_key === 'boolean') {
          setUseBundledKey(settings.use_bundled_key);
        }
      } catch (err) {
        console.error('Failed to load useBundledKey:', err);
      }
    }
    loadInitialSettings();
  }, []);

  // InnerTube/gRPC/Officialステータスイベントを監視
  useEffect(() => {
    let unlistenInnerTube: UnlistenFn | null = null;
    let unlistenGrpc: UnlistenFn | null = null;
    let unlistenOfficial: UnlistenFn | null = null;

    async function setupStatusListeners() {
      try {
        unlistenInnerTube = await listen<InnerTubeStatusEvent>('innertube-status', (event) => {
          if (!isMountedRef.current) return;
          const { connected, error: statusError, stopped } = event.payload;
          if (connected) {
            setConnectionStatus('connected');
            setError(null);
            setLastEvent('InnerTube接続成功');
          } else if (stopped) {
            setConnectionStatus('disconnected');
            setLastEvent('InnerTube停止');
          } else if (statusError) {
            setConnectionStatus('error');
            setError(statusError);
            setLastEvent('InnerTubeエラー');
          }
        });

        unlistenGrpc = await listen<GrpcStatusEvent>('grpc-status', (event) => {
          if (!isMountedRef.current) return;
          const { connected, error: statusError } = event.payload;
          if (connected) {
            setConnectionStatus('connected');
            setError(null);
            setLastEvent('gRPC接続成功');
          } else if (statusError) {
            setConnectionStatus('error');
            setError(statusError);
            setLastEvent('gRPCエラー');
          } else {
            setConnectionStatus('disconnected');
            setLastEvent('gRPC切断');
          }
        });

        unlistenOfficial = await listen<OfficialStatusEvent>('official-status', (event) => {
          if (!isMountedRef.current) return;
          const { connected, error: statusError, stopped, quotaExceeded, streamEnded, retrying } = event.payload;
          if (connected) {
            setConnectionStatus('connected');
            setError(null);
            setLastEvent('公式API接続成功');
          } else if (stopped) {
            setConnectionStatus('disconnected');
            setLastEvent('公式API停止');
          } else if (quotaExceeded) {
            setConnectionStatus('error');
            setError('クォータ超過 - 翌日まで待機してください');
            setLastEvent('クォータ超過');
          } else if (streamEnded) {
            setConnectionStatus('disconnected');
            setLastEvent('配信終了');
          } else if (statusError) {
            setConnectionStatus(retrying ? 'connected' : 'error');
            if (!retrying) setError(statusError);
            setLastEvent(retrying ? `リトライ中: ${statusError}` : `公式APIエラー: ${statusError}`);
          } else {
            setConnectionStatus('disconnected');
            setLastEvent('公式API切断');
          }
        });
      } catch (err) {
        console.error('Failed to setup status listeners:', err);
      }
    }

    setupStatusListeners();

    return () => {
      unlistenInnerTube?.();
      unlistenGrpc?.();
      unlistenOfficial?.();
    };
  }, []);

  // ポーリング状態を監視
  useEffect(() => {
    async function setupListener() {
      try {
        unlistenRef.current = await listen<PollingEventType>('polling-event', (event) => {
          if (!isMountedRef.current) return;

          const payload = event.payload;
          switch (payload.type) {
            case 'started':
              setIsPolling(true);
              setError(null);
              setLastEvent('ポーリング開始');
              break;
            case 'stopped':
              setIsPolling(false);
              setLastEvent(`ポーリング停止: ${payload.reason}`);
              break;
            case 'messages':
              setLastEvent(`${payload.messages.length}件のコメントを取得`);
              break;
            case 'stateUpdate':
              // Rust側のStateUpdateイベントからPollingStateを構築（全フィールドを更新）
              setPollingState((prev) => ({
                next_page_token: payload.next_page_token,
                polling_interval_millis: payload.polling_interval_millis,
                live_chat_id: prev?.live_chat_id ?? liveChatId,
                quota_used: payload.quota_used,
                poll_count: payload.poll_count,
              }));
              break;
            case 'error':
              setError(payload.message);
              if (payload.retrying) {
                setLastEvent(`エラー: ${payload.message} (再試行中)`);
              } else {
                setLastEvent(`エラー: ${payload.message}`);
              }
              break;
            case 'quotaExceeded':
              setError('クォータ超過: 本日のAPI使用量が上限に達しました');
              setIsPolling(false);
              setLastEvent('クォータ超過');
              break;
            case 'streamEnded':
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
      unlistenRef.current?.();
    };
  }, [liveChatId]);

  // NOTE: ポーリング状態はApp.tsxで管理（is_unified_polling_running）
  // 以下の古いuseEffectは削除：is_polling_running（旧API）を使用していたためバグの原因だった

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

  // 現在のモードでAPIキーが必要かどうか
  const currentModeRequiresApiKey = API_MODE_INFO[apiMode].requiresApiKey;
  const hasValidApiKey = apiKey && apiKey.length > 0;
  // APIキーが必要なモードでは、ユーザーAPIキーまたは同梱キー使用のいずれかがあれば開始可能
  const canStartPolling = videoId && (apiMode === 'innertube' || hasValidApiKey || (currentModeRequiresApiKey && useBundledKey));

  // 統合ポーラーを使った開始処理
  const handleStartPolling = useCallback(
    async (_useSavedState: boolean = false) => {
      // NOTE: 保存状態からの再開は公式APIポーリングモードでのみサポート
      // 現在は統合ポーラーを使用するため、この機能は将来的に再実装予定
      if (!videoId) {
        setError('動画IDが必要です');
        return;
      }

      // 公式APIモードではAPIキーが必要
      if (currentModeRequiresApiKey && !useBundledKey && !hasValidApiKey) {
        setError('APIキーが設定されていません');
        return;
      }

      setLoading(true);
      setError(null);
      setConnectionStatus('disconnected');

      try {
        // 統合ポーラーを使用
        // 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
        await invoke('start_unified_polling', {
          video_id: videoId,
          mode: apiMode,
          use_bundled_key: useBundledKey,
          user_api_key: hasValidApiKey ? apiKey : null,
        });
        if (isMountedRef.current) {
          setIsPolling(true);
          setSavedState(null); // 開始後は保存された状態をクリア
        }
      } catch (err) {
        if (isMountedRef.current) {
          const errorMessage = err instanceof Error ? err.message : String(err);
          setError(`ポーリング開始エラー: ${errorMessage}`);
          setConnectionStatus('error');
        }
      } finally {
        if (isMountedRef.current) {
          setLoading(false);
        }
      }
    },
    [videoId, apiMode, useBundledKey, apiKey, hasValidApiKey, currentModeRequiresApiKey]
  );

  // 統合ポーラーを使った停止処理
  const handleStopPolling = useCallback(async () => {
    setLoading(true);

    try {
      // 統合ポーラーを停止
      await invoke('stop_unified_polling');
      if (isMountedRef.current) {
        setIsPolling(false);
        setConnectionStatus('disconnected');
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
  }, []);

  // APIモード変更ハンドラ
  const handleApiModeChange = useCallback(async (newMode: ApiMode) => {
    if (isPolling) {
      setError('取得中はモードを変更できません。停止してから変更してください。');
      return;
    }

    try {
      await invoke('save_api_mode', { mode: newMode });
      setApiMode(newMode);
      setError(null);
      setLastEvent(`モードを${API_MODE_INFO[newMode].label}に変更しました`);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`モード変更エラー: ${errorMessage}`);
    }
  }, [isPolling]);

  // 動画ID/URLから動画IDを抽出
  const extractVideoId = (input: string): string => {
    const urlMatch = input.match(/(?:v=|youtu\.be\/)([a-zA-Z0-9_-]{11})/);
    if (urlMatch && urlMatch[1]) {
      return urlMatch[1];
    }
    // 11文字の動画IDとして扱う
    if (/^[a-zA-Z0-9_-]{11}$/.test(input)) {
      return input;
    }
    return input;
  };

  // 動画ID変更処理
  const handleUpdateVideoId = useCallback(async () => {
    const newVideoId = extractVideoId(editVideoInput.trim());
    if (!newVideoId) {
      setError('動画IDまたはURLを入力してください');
      return;
    }

    setIsUpdating(true);
    setError(null);

    try {
      // InnerTube経由でChat IDを取得（APIキー不要）
      // または公式API経由で取得
      let newLiveChatId: string;

      if (apiKey) {
        // 公式API経由
        // 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
        newLiveChatId = await invoke<string>('get_live_chat_id', {
          api_key: apiKey,
          video_id: newVideoId,
        });
      } else {
        // InnerTube経由（APIキー不要）
        // InnerTubeClientはstart_polling_innertube内で使われる
        // ここでは公式APIが使えない場合のフォールバックとして空文字を設定
        newLiveChatId = '';
      }

      // 設定を保存
      // 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
      await invoke('save_wizard_settings', {
        video_id: newVideoId,
        live_chat_id: newLiveChatId,
        use_bundled_key: useBundledKey,
      });

      // 親コンポーネントに通知
      onSettingsChange?.({ videoId: newVideoId, liveChatId: newLiveChatId });

      setEditVideoInput('');
      setLastEvent('動画IDを更新しました');
    } catch (err) {
      if (isMountedRef.current) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`動画ID更新エラー: ${errorMessage}`);
      }
    } finally {
      if (isMountedRef.current) {
        setIsUpdating(false);
      }
    }
  }, [apiKey, editVideoInput, onSettingsChange, useBundledKey]);

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

      {/* APIモード選択 */}
      <div className="mb-4 p-4 bg-gray-50 rounded-lg">
        <h3 className="text-sm font-semibold text-gray-700 mb-3">取得モード</h3>
        <div className="space-y-2">
          {(Object.keys(API_MODE_INFO) as ApiMode[]).map((mode) => {
            const info = API_MODE_INFO[mode];
            return (
              <label
                key={mode}
                className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                  apiMode === mode
                    ? 'border-blue-500 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300'
                } ${isPolling ? 'opacity-50 cursor-not-allowed' : ''}`}
              >
                <input
                  type="radio"
                  name="apiMode"
                  value={mode}
                  checked={apiMode === mode}
                  onChange={() => handleApiModeChange(mode)}
                  disabled={isPolling}
                  className="mt-1"
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-gray-800">{info.label}</span>
                    {info.recommended && (
                      <span className="px-2 py-0.5 text-xs bg-green-100 text-green-700 rounded">
                        推奨
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-gray-500 mt-0.5">{info.description}</p>
                </div>
              </label>
            );
          })}
        </div>

        {/* APIキー設定（公式APIモード時） */}
        {currentModeRequiresApiKey && (
          <div className="mt-3 pt-3 border-t border-gray-200">
            <div className="flex items-center gap-2 mb-2">
              <input
                type="checkbox"
                id="useBundledKey"
                checked={useBundledKey}
                onChange={async (e) => {
                  const newValue = e.target.checked;
                  const oldValue = useBundledKey;
                  setUseBundledKey(newValue);
                  // 設定を永続化
                  try {
                    // videoIdやliveChatIdが空でも保存を試みる
                    // （Rust側でマージ処理を行うため）
                    await invoke('save_wizard_settings', {
                      video_id: videoId,
                      live_chat_id: liveChatId,
                      use_bundled_key: newValue,
                    });
                  } catch (err) {
                    console.error('Failed to save useBundledKey:', err);
                    // 保存失敗時はUIを元の状態に戻す
                    setUseBundledKey(oldValue);
                  }
                }}
                disabled={isPolling}
                className="rounded"
              />
              <label htmlFor="useBundledKey" className="text-sm text-gray-700">
                アプリ同梱キーを使用
              </label>
            </div>
            {!useBundledKey && !hasValidApiKey && (
              <p className="text-xs text-orange-600">
                ※ APIキーが設定されていません。設定画面でAPIキーを入力してください。
              </p>
            )}
          </div>
        )}
      </div>

      {/* 接続情報 */}
      <div className="mb-4 p-3 bg-gray-50 rounded-lg text-sm">
        <div className="grid grid-cols-2 gap-2 mb-3">
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
        {/* 動画ID変更フォーム */}
        <div className="flex gap-2">
          <input
            type="text"
            value={editVideoInput}
            onChange={(e) => setEditVideoInput(e.target.value)}
            placeholder="新しい動画ID または URL"
            disabled={isPolling || isUpdating}
            className="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
            onKeyDown={(e) => {
              if (e.key === 'Enter' && editVideoInput.trim()) {
                handleUpdateVideoId();
              }
            }}
          />
          <button
            onClick={handleUpdateVideoId}
            disabled={isPolling || isUpdating || !editVideoInput.trim()}
            className={`px-3 py-1.5 text-sm font-medium rounded transition-colors ${
              isPolling || isUpdating || !editVideoInput.trim()
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : 'bg-blue-600 text-white hover:bg-blue-700'
            }`}
          >
            {isUpdating ? '更新中...' : '変更'}
          </button>
        </div>
        {isPolling && (
          <p className="mt-1 text-xs text-orange-600">
            ※取得中は変更できません。停止してから変更してください。
          </p>
        )}
      </div>

      {/* 接続状態 */}
      <div className="mb-4 flex items-center gap-3">
        <div
          className={`w-3 h-3 rounded-full ${
            connectionStatus === 'connected'
              ? 'bg-green-500 animate-pulse'
              : connectionStatus === 'error'
                ? 'bg-red-500'
                : 'bg-gray-400'
          }`}
        />
        <span className="font-medium">
          {connectionStatus === 'connected'
            ? '接続中'
            : connectionStatus === 'error'
              ? 'エラー'
              : '停止中'}
        </span>
        <span className="text-sm text-gray-500">
          [{API_MODE_INFO[apiMode].label}]
        </span>
        {lastEvent && <span className="text-sm text-gray-500">- {lastEvent}</span>}
      </div>

      {/* NOTE: 「続きから開始」ボタンは統合ポーラーでは非サポート
          統合ポーラー（start_unified_polling）が保存状態復元をサポートするまで非表示。
          将来的に保存状態復元機能を実装する場合はここを復活させる。
          関連: docs/900_tasks.md - T16: 技術的負債「公式APIモードの保存状態再開機能」
      {savedState && !isPolling && apiMode === 'official' && (
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
              disabled={loading || !canStartPolling}
              className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                loading || !canStartPolling
                  ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                  : 'bg-yellow-600 text-white hover:bg-yellow-700'
              }`}
            >
              続きから開始
            </button>
          </div>
        </div>
      )}
      */}

      {/* 制御ボタン */}
      <div className="flex gap-3 mb-4">
        <button
          onClick={() => handleStartPolling(false)}
          disabled={loading || isPolling || !canStartPolling}
          className={`px-4 py-2 rounded-lg font-medium transition-colors ${
            loading || isPolling || !canStartPolling
              ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
              : 'bg-green-600 text-white hover:bg-green-700'
          }`}
        >
          {loading ? '処理中...' : '開始'}
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

      {/* クォータ情報（公式APIモードのみ） */}
      {pollingState && apiMode === 'official' && (
        <div className="mb-4 p-3 bg-blue-50 rounded-lg">
          <div className="flex justify-between items-center mb-2">
            <span className="text-sm font-medium text-blue-800">
              クォータ使用量
            </span>
            <span className="text-sm text-blue-600">
              {pollingState.quota_used.toLocaleString()} / {DAILY_QUOTA_LIMIT.toLocaleString()} units
            </span>
          </div>
          <div
            className="w-full bg-blue-200 rounded-full h-2"
            role="progressbar"
            aria-valuenow={pollingState.quota_used}
            aria-valuemin={0}
            aria-valuemax={DAILY_QUOTA_LIMIT}
            aria-label="クォータ使用量"
          >
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
