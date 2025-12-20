import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect, useRef } from 'react';
import { CommentControlPanel } from './CommentControlPanel';
import type { ChatMessage } from '../types/chat';

export function ApiKeySetup() {
  const [apiKey, setApiKey] = useState('');
  const [videoId, setVideoId] = useState('');
  const [liveChatId, setLiveChatId] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isApiKeyLoaded, setIsApiKeyLoaded] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const isMountedRef = useRef(true);

  // 保存済みAPIキーとウィザード設定を自動読み込み
  useEffect(() => {
    async function loadSavedSettings() {
      try {
        // APIキー読み込み
        const hasKey = await invoke<boolean>('has_api_key');
        if (hasKey) {
          const savedKey = await invoke<string>('get_api_key');
          if (isMountedRef.current && savedKey) {
            setApiKey(savedKey);
            setIsApiKeyLoaded(true);
          }
        }

        // ウィザード設定読み込み
        const wizardSettings = await invoke<{
          video_id: string;
          live_chat_id: string;
          saved_at: string;
        } | null>('load_wizard_settings');
        if (isMountedRef.current && wizardSettings) {
          setVideoId(wizardSettings.video_id);
          setLiveChatId(wizardSettings.live_chat_id);
          setSuccess('保存済み設定を読み込みました');
        } else if (isMountedRef.current && hasKey) {
          setSuccess('保存済みAPIキーを読み込みました');
        }
      } catch (err) {
        console.error('Failed to load saved settings:', err);
      }
    }
    loadSavedSettings();

    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const handleValidate = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const isValid = await invoke<boolean>('validate_api_key', {
        apiKey: apiKey,
      });

      if (!isMountedRef.current) return;

      if (isValid) {
        // APIキーを保存
        await invoke('save_api_key', { apiKey: apiKey });
        if (isMountedRef.current) {
          setIsApiKeyLoaded(true);
          setSuccess('APIキーが有効です。保存しました。');
        }
      } else {
        setError('APIキーが無効です');
      }
    } catch (err) {
      if (isMountedRef.current) {
        // Tauri 2.0のエラーハンドリング
        let errorMessage = 'APIキーの検証に失敗しました';
        if (err instanceof Error) {
          errorMessage = err.message;
        } else if (typeof err === 'string') {
          errorMessage = err;
        } else if (err && typeof err === 'object' && 'message' in err) {
          errorMessage = String((err as any).message);
        } else {
          errorMessage = String(err);
        }
        
        // エラーメッセージをユーザーフレンドリーに変換
        if (errorMessage.includes('API key is invalid') || errorMessage.includes('InvalidApiKey')) {
          errorMessage = 'APIキーが無効です。正しいAPIキーを入力してください。';
        } else if (errorMessage.includes('Quota exceeded')) {
          errorMessage = 'APIクォータが超過しています。明日再度お試しください。';
        } else if (errorMessage.includes('Rate limit exceeded')) {
          errorMessage = 'レート制限に達しました。しばらく待ってから再度お試しください。';
        } else if (errorMessage.includes('HTTP request failed')) {
          errorMessage = 'ネットワークエラーが発生しました。インターネット接続を確認してください。';
        }
        
        setError(errorMessage);
        console.error('API key validation error:', err);
      }
    } finally {
      if (isMountedRef.current) {
        setLoading(false);
      }
    }
  };

  const handleGetChatId = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const chatId = await invoke<string>('get_live_chat_id', {
        apiKey: apiKey,
        videoId: videoId,
      });
      setLiveChatId(chatId);
      setSuccess(`Live Chat ID: ${chatId}`);
    } catch (err) {
      setError(`エラー: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleGetMessages = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const result = await invoke<[ChatMessage[], string | null, number]>(
        'get_chat_messages',
        {
          apiKey: apiKey,
          liveChatId: liveChatId,
          pageToken: null,
        }
      );
      const newMessages = result[0];
      const pollingInterval = result[2];

      setMessages(newMessages);
      setSuccess(
        `${newMessages.length}件のメッセージを取得しました（ポーリング間隔: ${pollingInterval}ms）`
      );
    } catch (err) {
      setError(`エラー: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-4 max-w-4xl">
      <h2 className="text-2xl font-bold mb-6">YouTube API テスト</h2>

      {/* APIキー入力 */}
      <div className="mb-6">
        <div className="flex items-center gap-2 mb-2">
          <label className="font-semibold">APIキー</label>
          {isApiKeyLoaded && (
            <span className="text-xs bg-green-100 text-green-700 px-2 py-0.5 rounded">
              保存済み
            </span>
          )}
        </div>
        <div className="relative">
          <input
            type={showApiKey ? 'text' : 'password'}
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            className="w-full p-2 pr-12 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 placeholder:text-gray-400"
            placeholder="AIza..."
          />
          <button
            type="button"
            onClick={() => setShowApiKey(!showApiKey)}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-1 rounded"
            aria-label={showApiKey ? 'APIキーを隠す' : 'APIキーを表示'}
          >
            {showApiKey ? (
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
              </svg>
            ) : (
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              </svg>
            )}
          </button>
        </div>
        <button
          onClick={handleValidate}
          disabled={loading || !apiKey}
          className="mt-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
        >
          {loading ? '検証中...' : 'APIキーを検証・保存'}
        </button>
      </div>

      {/* 動画ID入力 */}
      <div className="mb-6">
        <label className="block mb-2 font-semibold">動画ID</label>
        <input
          type="text"
          value={videoId}
          onChange={(e) => setVideoId(e.target.value)}
          className="w-full p-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500 text-gray-900 placeholder:text-gray-400"
          placeholder="dQw4w9WgXcQ"
        />
        <button
          onClick={handleGetChatId}
          disabled={loading || !apiKey || !videoId}
          className="mt-2 px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
        >
          {loading ? '取得中...' : 'チャットIDを取得'}
        </button>
      </div>

      {/* チャットID表示＆コメント制御パネル */}
      {liveChatId && (
        <div className="mb-6 space-y-4">
          <div>
            <label className="block mb-2 font-semibold">Live Chat ID</label>
            <div className="p-2 bg-gray-100 rounded border border-gray-300 break-all text-sm">
              {liveChatId}
            </div>
          </div>

          {/* コメント取得制御パネル */}
          <CommentControlPanel
            apiKey={apiKey}
            videoId={videoId}
            liveChatId={liveChatId}
          />

          {/* 手動メッセージ取得（デバッグ用） */}
          <details className="bg-gray-50 rounded-lg p-4">
            <summary className="cursor-pointer font-medium text-gray-700">
              デバッグ: 手動メッセージ取得
            </summary>
            <div className="mt-3">
              <button
                onClick={handleGetMessages}
                disabled={loading || !apiKey}
                className="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
              >
                {loading ? '取得中...' : 'メッセージを取得'}
              </button>
            </div>
          </details>
        </div>
      )}

      {/* 成功メッセージ */}
      {success && (
        <div className="mb-4 p-3 bg-green-100 text-green-700 rounded border border-green-300">
          {success}
        </div>
      )}

      {/* エラーメッセージ */}
      {error && (
        <div className="mb-4 p-3 bg-red-100 text-red-700 rounded border border-red-300">
          {error}
        </div>
      )}

      {/* メッセージリスト */}
      {messages.length > 0 && (
        <div className="mt-6">
          <h3 className="text-xl font-bold mb-3">
            取得したメッセージ ({messages.length}件)
          </h3>
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {messages.map((msg) => (
              <div
                key={msg.id}
                className="p-3 border border-gray-300 rounded bg-white"
              >
                <div className="flex items-center gap-2 mb-1">
                  <img
                    src={msg.authorImageUrl}
                    alt={msg.authorName}
                    className="w-8 h-8 rounded-full"
                  />
                  <span className="font-semibold">{msg.authorName}</span>
                  {msg.isOwner && (
                    <span className="px-2 py-0.5 bg-red-500 text-white text-xs rounded">
                      オーナー
                    </span>
                  )}
                  {msg.isModerator && (
                    <span className="px-2 py-0.5 bg-blue-500 text-white text-xs rounded">
                      モデレーター
                    </span>
                  )}
                  {msg.isMember && (
                    <span className="px-2 py-0.5 bg-green-500 text-white text-xs rounded">
                      メンバー
                    </span>
                  )}
                </div>
                <div className="text-gray-800">{msg.message}</div>
                <div className="text-xs text-gray-500 mt-1">
                  {new Date(msg.publishedAt).toLocaleString('ja-JP')}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
