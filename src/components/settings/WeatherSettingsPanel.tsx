import { useState, useEffect, useCallback } from 'react';
import {
  hasWeatherApiKey,
  setWeatherApiKey,
  getWeatherCity,
  setWeatherCity,
  getWeather,
  fetchWeather,
  broadcastWeatherUpdate,
  getWeatherCacheTtl,
  type WeatherData,
} from '../../types/weather';

interface WeatherSettingsPanelProps {
  className?: string;
}

export function WeatherSettingsPanel({ className = '' }: WeatherSettingsPanelProps) {
  const [apiKey, setApiKeyValue] = useState('');
  const [hasKey, setHasKey] = useState(false);
  const [city, setCityValue] = useState('Tokyo');
  const [weather, setWeather] = useState<WeatherData | null>(null);
  const [cacheTtl, setCacheTtl] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);

  // 初期化
  useEffect(() => {
    const init = async () => {
      try {
        const [hasApiKey, currentCity] = await Promise.all([
          hasWeatherApiKey(),
          getWeatherCity(),
        ]);
        setHasKey(hasApiKey);
        setCityValue(currentCity);
      } catch (err) {
        console.error('Failed to initialize weather settings:', err);
      }
    };
    init();
  }, []);

  // キャッシュTTL更新
  useEffect(() => {
    const updateTtl = async () => {
      try {
        const ttl = await getWeatherCacheTtl();
        setCacheTtl(ttl);
      } catch {
        // ignore
      }
    };
    updateTtl();
    const interval = setInterval(updateTtl, 10000);
    return () => clearInterval(interval);
  }, []);

  // APIキー保存
  const handleSaveApiKey = useCallback(async () => {
    if (!apiKey.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await setWeatherApiKey(apiKey.trim());
      setHasKey(true);
      setApiKeyValue('');
    } catch (err) {
      setError('APIキーの保存に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [apiKey]);

  // 都市名保存
  const handleSaveCity = useCallback(async () => {
    if (!city.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await setWeatherCity(city.trim());
    } catch (err) {
      setError('都市名の保存に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [city]);

  // 天気取得
  const handleFetchWeather = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getWeather();
      setWeather(data);
    } catch (err) {
      setError('天気情報の取得に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  // 強制更新
  const handleForceRefresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchWeather();
      setWeather(data);
    } catch (err) {
      setError('天気情報の更新に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  // ブロードキャスト
  const handleBroadcast = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await broadcastWeatherUpdate();
    } catch (err) {
      setError('ブロードキャストに失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  const formatTtl = (seconds: number) => {
    if (seconds === 0) return 'キャッシュなし';
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className={`space-y-6 ${className}`}>
      <h3 className="text-lg font-semibold text-gray-900">天気設定</h3>

      {/* エラー表示 */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {error}
        </div>
      )}

      {/* APIキー設定 */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-700">
          OpenWeatherMap APIキー
        </label>
        {hasKey ? (
          <div className="flex items-center gap-2">
            <span className="text-sm text-green-600">設定済み</span>
            <button
              type="button"
              onClick={() => setHasKey(false)}
              className="text-sm text-gray-500 hover:text-gray-700 underline"
            >
              変更
            </button>
          </div>
        ) : (
          <div className="flex gap-2">
            <div className="relative flex-1">
              <input
                type={showApiKey ? 'text' : 'password'}
                value={apiKey}
                onChange={(e) => setApiKeyValue(e.target.value)}
                placeholder="APIキーを入力"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
              <button
                type="button"
                onClick={() => setShowApiKey(!showApiKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
              >
                {showApiKey ? '隠す' : '表示'}
              </button>
            </div>
            <button
              type="button"
              onClick={handleSaveApiKey}
              disabled={loading || !apiKey.trim()}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              保存
            </button>
          </div>
        )}
        <p className="text-xs text-gray-500">
          <a
            href="https://openweathermap.org/api"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-600 hover:underline"
          >
            OpenWeatherMap
          </a>
          で無料APIキーを取得できます
        </p>
      </div>

      {/* 都市名設定 */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-700">都市名</label>
        <div className="flex gap-2">
          <input
            type="text"
            value={city}
            onChange={(e) => setCityValue(e.target.value)}
            placeholder="Tokyo"
            className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <button
            type="button"
            onClick={handleSaveCity}
            disabled={loading || !city.trim()}
            className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            設定
          </button>
        </div>
        <p className="text-xs text-gray-500">
          英語で都市名を入力（例: Tokyo, Osaka, New York）
        </p>
      </div>

      {/* 天気プレビュー */}
      {hasKey && (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700">天気プレビュー</span>
            <span className="text-xs text-gray-500">キャッシュ残り: {formatTtl(cacheTtl)}</span>
          </div>

          {weather ? (
            <div className="p-4 bg-gray-50 rounded-lg">
              <div className="flex items-center gap-4">
                <span className="text-4xl">{weather.icon}</span>
                <div>
                  <p className="text-2xl font-bold">{weather.temp.toFixed(1)}°C</p>
                  <p className="text-sm text-gray-600">{weather.description}</p>
                  <p className="text-xs text-gray-500">{weather.location}</p>
                </div>
              </div>
            </div>
          ) : (
            <div className="p-4 bg-gray-50 rounded-lg text-center text-gray-500">
              天気情報なし
            </div>
          )}

          <div className="flex gap-2">
            <button
              type="button"
              onClick={handleFetchWeather}
              disabled={loading}
              className="flex-1 px-4 py-2 bg-blue-100 text-blue-700 rounded-lg hover:bg-blue-200 disabled:opacity-50"
            >
              取得
            </button>
            <button
              type="button"
              onClick={handleForceRefresh}
              disabled={loading}
              className="flex-1 px-4 py-2 bg-orange-100 text-orange-700 rounded-lg hover:bg-orange-200 disabled:opacity-50"
            >
              強制更新
            </button>
            <button
              type="button"
              onClick={handleBroadcast}
              disabled={loading || !weather}
              className="flex-1 px-4 py-2 bg-green-100 text-green-700 rounded-lg hover:bg-green-200 disabled:opacity-50"
            >
              配信
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
