import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { handleTauriError } from '../../utils/errorMessages';

interface WizardStep1Props {
  apiKey: string;
  onApiKeyChange: (apiKey: string) => void;
  onValidationChange: (isValid: boolean) => void;
}

export default function WizardStep1({
  apiKey,
  onApiKeyChange,
  onValidationChange,
}: WizardStep1Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const isMountedRef = useRef(true);

  // マウント状態を追跡（非同期操作後のstate更新を防ぐ）
  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const handleValidate = async () => {
    if (!apiKey.trim()) {
      setError('APIキーを入力してください');
      return;
    }

    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const isValid = await invoke<boolean>('validate_api_key', {
        apiKey: apiKey,
      });

      // アンマウント後はstate更新をスキップ
      if (!isMountedRef.current) return;

      if (isValid) {
        // APIキーを保存
        await invoke('save_api_key', { apiKey: apiKey });
        if (!isMountedRef.current) return;
        setSuccess('APIキーが有効です。保存しました。');
        onValidationChange(true);
      } else {
        setError('APIキーが無効です');
        onValidationChange(false);
      }
    } catch (err) {
      if (!isMountedRef.current) return;
      const errorMessage = handleTauriError(err, 'APIキーの検証に失敗しました');
      setError(errorMessage);
      onValidationChange(false);
      console.error('API key validation error:', err);
    } finally {
      if (isMountedRef.current) {
        setLoading(false);
      }
    }
  };

  return (
    <div>
      <h2 className="text-xl font-bold mb-4">Step 1: YouTube API キーの設定</h2>
      <p className="text-gray-600 mb-6">
        YouTube Data API v3のAPIキーを入力してください。
        <a
          href="https://console.cloud.google.com/apis/credentials"
          target="_blank"
          rel="noopener noreferrer"
          className="text-blue-600 hover:underline ml-1"
        >
          Google Cloud Console
        </a>
        で取得できます。
      </p>

      <div className="space-y-4">
        <div>
          <label htmlFor="apiKey" className="block text-sm font-medium text-gray-700 mb-2">
            APIキー
          </label>
          <div className="relative">
            <input
              id="apiKey"
              type={showApiKey ? 'text' : 'password'}
              value={apiKey}
              onChange={(e) => onApiKeyChange(e.target.value)}
              placeholder="AIzaSy..."
              className="w-full px-4 py-2 pr-12 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-gray-900 placeholder:text-gray-400"
              disabled={loading}
            />
            <button
              type="button"
              onClick={() => setShowApiKey(!showApiKey)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-700 transition-colors"
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

        <button
          onClick={handleValidate}
          disabled={loading || !apiKey.trim()}
          className={`w-full px-4 py-2 rounded-lg font-medium transition-colors ${
            loading || !apiKey.trim()
              ? 'bg-gray-400 text-white cursor-not-allowed'
              : 'bg-blue-600 text-white hover:bg-blue-700'
          }`}
        >
          {loading ? '検証中...' : 'APIキーを検証'}
        </button>

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
      </div>
    </div>
  );
}
