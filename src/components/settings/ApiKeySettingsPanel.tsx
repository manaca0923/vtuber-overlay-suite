import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * YouTube APIキー設定パネル
 * 設定画面からAPIキーを管理するためのコンポーネント
 */
export function ApiKeySettingsPanel() {
  const [apiKey, setApiKey] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [isLoaded, setIsLoaded] = useState(false);

  // 保存済みAPIキーを読み込み
  useEffect(() => {
    async function loadApiKey() {
      try {
        const savedKey = await invoke<string | null>('get_api_key');
        if (savedKey) {
          setApiKey(savedKey);
          setIsLoaded(true);
        }
      } catch (err) {
        console.error('Failed to load API key:', err);
      }
    }
    loadApiKey();
  }, []);

  const handleValidateAndSave = async () => {
    if (!apiKey.trim()) {
      setError('APIキーを入力してください');
      return;
    }

    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const isValid = await invoke<boolean>('validate_api_key', {
        apiKey: apiKey.trim(),
      });

      if (isValid) {
        await invoke('save_api_key', { apiKey: apiKey.trim() });
        setIsLoaded(true);
        setSuccess('APIキーを保存しました');
        setTimeout(() => setSuccess(''), 3000);
      } else {
        setError('APIキーが無効です');
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(`エラー: ${message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    try {
      await invoke('delete_api_key');
      setApiKey('');
      setIsLoaded(false);
      setSuccess('APIキーを削除しました');
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(`削除エラー: ${message}`);
    }
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h3 className="text-lg font-semibold mb-4">YouTube APIキー</h3>

      <p className="text-sm text-gray-600 mb-4">
        公式API（ポーリング）モードで使用するAPIキーを設定します。
        gRPCモードまたはInnerTubeモードでは不要です。
      </p>

      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            APIキー
          </label>
          <div className="relative">
            <input
              type={showApiKey ? 'text' : 'password'}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="AIza..."
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent pr-10"
            />
            <button
              type="button"
              onClick={() => setShowApiKey(!showApiKey)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700"
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
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleValidateAndSave}
            disabled={loading || !apiKey.trim()}
            className={`flex-1 py-2 px-4 rounded-lg font-medium transition-colors ${
              loading || !apiKey.trim()
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : 'bg-blue-600 text-white hover:bg-blue-700'
            }`}
          >
            {loading ? '検証中...' : '検証・保存'}
          </button>
          {isLoaded && (
            <button
              onClick={handleDelete}
              className="py-2 px-4 rounded-lg font-medium text-red-600 border border-red-300 hover:bg-red-50 transition-colors"
            >
              削除
            </button>
          )}
        </div>

        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
            {error}
          </div>
        )}
        {success && (
          <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
            {success}
          </div>
        )}
      </div>
    </div>
  );
}
