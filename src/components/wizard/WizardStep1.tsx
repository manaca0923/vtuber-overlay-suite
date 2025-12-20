import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

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
  const isMountedRef = useRef(true);

  useEffect(() => {
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
        api_key: apiKey,
      });

      if (!isMountedRef.current) return;

      if (isValid) {
        // APIキーを保存
        await invoke('save_api_key', { api_key: apiKey });
        if (isMountedRef.current) {
          setSuccess('APIキーが有効です。保存しました。');
          onValidationChange(true);
        }
      } else {
        if (isMountedRef.current) {
          setError('APIキーが無効です');
          onValidationChange(false);
        }
      }
    } catch (err) {
      if (isMountedRef.current) {
        const errorMessage =
          err instanceof Error ? err.message : String(err);
        setError(`エラー: ${errorMessage}`);
        onValidationChange(false);
      }
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
          <input
            id="apiKey"
            type="text"
            value={apiKey}
            onChange={(e) => onApiKeyChange(e.target.value)}
            placeholder="AIzaSy..."
            className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            disabled={loading}
          />
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
