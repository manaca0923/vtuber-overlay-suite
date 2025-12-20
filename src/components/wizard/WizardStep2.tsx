import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface WizardStep2Props {
  apiKey: string;
  videoId: string;
  onVideoIdChange: (videoId: string) => void;
  onLiveChatIdChange: (liveChatId: string | null) => void;
}

export default function WizardStep2({
  apiKey,
  videoId,
  onVideoIdChange,
  onLiveChatIdChange,
}: WizardStep2Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const isMountedRef = useRef(true);

  useEffect(() => {
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  // URL解析ロジック
  const extractVideoId = (input: string): string => {
    // URLの場合は動画IDを抽出
    const urlMatch = input.match(/(?:watch\?v=|youtu\.be\/)([^&\s]+)/);
    if (urlMatch && urlMatch[1]) {
      return urlMatch[1];
    }
    // そのまま動画IDの場合
    return input;
  };

  const handleGetChatId = useCallback(async (vid: string) => {
    if (!vid.trim()) {
      setError('動画IDを入力してください');
      return;
    }

    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const chatId = await invoke<string>('get_live_chat_id', {
        apiKey: apiKey,
        videoId: vid,
      });

      // コンポーネントがアンマウントされていない場合のみstate更新
      if (isMountedRef.current) {
        onLiveChatIdChange(chatId);
        setSuccess(`チャットIDを取得しました: ${chatId.substring(0, 20)}...`);
      }
    } catch (err) {
      if (isMountedRef.current) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`エラー: ${errorMessage}`);
        onLiveChatIdChange(null);
      }
    } finally {
      if (isMountedRef.current) {
        setLoading(false);
      }
    }
  }, [apiKey, onLiveChatIdChange]);

  // 動画ID変更時に自動でチャットID取得（debounce）
  useEffect(() => {
    if (!videoId.trim()) return;

    const timer = setTimeout(() => {
      handleGetChatId(videoId);
    }, 500);

    return () => clearTimeout(timer);
  }, [videoId, handleGetChatId]);

  const handleInputChange = (value: string) => {
    const vid = extractVideoId(value);
    onVideoIdChange(vid);
  };

  return (
    <div>
      <h2 className="text-xl font-bold mb-4">Step 2: 動画IDの入力</h2>
      <p className="text-gray-600 mb-6">
        ライブ配信中のYouTube動画のURLまたは動画IDを入力してください。
      </p>

      <div className="space-y-4">
        <div>
          <label htmlFor="videoId" className="block text-sm font-medium text-gray-700 mb-2">
            動画URLまたはID
          </label>
          <input
            id="videoId"
            type="text"
            value={videoId}
            onChange={(e) => handleInputChange(e.target.value)}
            placeholder="https://www.youtube.com/watch?v=... または動画ID"
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            disabled={loading}
          />
          <p className="mt-2 text-sm text-gray-500">
            例: https://www.youtube.com/watch?v=dQw4w9WgXcQ
          </p>
        </div>

        {loading && (
          <div className="flex items-center gap-2 text-gray-600">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
            <span className="text-sm">チャットIDを取得中...</span>
          </div>
        )}

        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
            ✓ {success}
          </div>
        )}

        <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <h3 className="font-medium text-blue-900 mb-2">ヒント</h3>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• ライブ配信中の動画である必要があります</li>
            <li>• チャット機能が有効になっている必要があります</li>
            <li>• URLをコピー＆ペーストするだけでOKです</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
