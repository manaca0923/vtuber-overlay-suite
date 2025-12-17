import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

type MessageType =
  | { type: 'text' }
  | { type: 'superChat'; amount: string; currency: string }
  | { type: 'superSticker'; stickerId: string }
  | { type: 'membership'; level: string }
  | { type: 'membershipGift'; count: number };

interface ChatMessage {
  id: string;
  message: string;
  authorName: string;
  authorChannelId: string;
  authorImageUrl: string;
  publishedAt: string;
  isOwner: boolean;
  isModerator: boolean;
  isMember: boolean;
  isVerified: boolean;
  messageType: MessageType;
}

export function ApiKeySetup() {
  const [apiKey, setApiKey] = useState('');
  const [videoId, setVideoId] = useState('');
  const [liveChatId, setLiveChatId] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);

  const handleValidate = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const isValid = await invoke<boolean>('validate_api_key', {
        api_key: apiKey,
      });

      if (isValid) {
        setSuccess('APIキーが有効です');
      } else {
        setError('APIキーが無効です');
      }
    } catch (err) {
      setError(`エラー: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleGetChatId = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const chatId = await invoke<string>('get_live_chat_id', {
        api_key: apiKey,
        video_id: videoId,
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
      const [newMessages, _nextPageToken, pollingInterval] = await invoke<
        [ChatMessage[], string | null, number]
      >('get_chat_messages', {
        api_key: apiKey,
        live_chat_id: liveChatId,
        page_token: null,
      });

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
        <label className="block mb-2 font-semibold">APIキー</label>
        <input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          className="w-full p-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="AIza..."
        />
        <button
          onClick={handleValidate}
          disabled={loading || !apiKey}
          className="mt-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
        >
          {loading ? '検証中...' : 'APIキーを検証'}
        </button>
      </div>

      {/* 動画ID入力 */}
      <div className="mb-6">
        <label className="block mb-2 font-semibold">動画ID</label>
        <input
          type="text"
          value={videoId}
          onChange={(e) => setVideoId(e.target.value)}
          className="w-full p-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
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

      {/* チャットID表示＆メッセージ取得 */}
      {liveChatId && (
        <div className="mb-6">
          <label className="block mb-2 font-semibold">Live Chat ID</label>
          <div className="p-2 bg-gray-100 rounded border border-gray-300 break-all">
            {liveChatId}
          </div>
          <button
            onClick={handleGetMessages}
            disabled={loading || !apiKey}
            className="mt-2 px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
          >
            {loading ? '取得中...' : 'メッセージを取得'}
          </button>
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
