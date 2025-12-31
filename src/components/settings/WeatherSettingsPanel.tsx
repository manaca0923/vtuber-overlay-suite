import { useState, useEffect, useCallback } from 'react';
import {
  getWeatherCity,
  getWeatherCacheTtl,
  refreshWeather,
  broadcastWeather,
  setWeatherCityAndBroadcast,
  type WeatherData,
} from '../../types/weather';
import type { WeatherSettings, WeatherPosition } from '../../types/overlaySettings';

// 天気位置オプション
const WEATHER_POSITION_OPTIONS: { value: WeatherPosition; label: string }[] = [
  { value: 'left-top', label: '左上' },
  { value: 'left-bottom', label: '左下' },
  { value: 'right-top', label: '右上' },
  { value: 'right-bottom', label: '右下' },
];

interface WeatherSettingsPanelProps {
  className?: string;
  settings?: WeatherSettings;
  onChange?: (settings: WeatherSettings) => void;
}

export function WeatherSettingsPanel({ className = '', settings, onChange }: WeatherSettingsPanelProps) {
  // デフォルト値
  const weatherEnabled = settings?.enabled ?? true;
  const weatherPosition = settings?.position ?? 'left-top';
  const [city, setCityValue] = useState('Tokyo');
  const [weather, setWeather] = useState<WeatherData | null>(null);
  const [cacheTtl, setCacheTtl] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 初期化
  useEffect(() => {
    const init = async () => {
      try {
        const currentCity = await getWeatherCity();
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

  // 都市名設定 + 更新 + 配信（一括処理）
  const handleSaveCity = useCallback(async () => {
    if (!city.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const data = await setWeatherCityAndBroadcast(city.trim());
      setWeather(data);
    } catch (err) {
      setError('都市名の設定に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [city]);

  // 更新（キャッシュクリア + 取得 + タイマーリセット）
  const handleRefresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await refreshWeather();
      setWeather(data);
    } catch (err) {
      setError('天気情報の更新に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  // 配信（オーバーレイに送信）
  const handleBroadcast = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await broadcastWeather();
    } catch (err) {
      setError('配信に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  const formatTtl = (seconds: number) => {
    if (seconds === 0) return '次回更新まで: --:--';
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `次回更新まで: ${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  // 設定変更ハンドラ
  const handlePositionChange = (position: WeatherPosition) => {
    onChange?.({
      enabled: weatherEnabled,
      position,
    });
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

      <p className="text-sm text-gray-500">
        表示ON/OFFは「ウィジェット」タブで設定できます。天気は15分ごとに自動更新されます。
      </p>

      {/* 配置位置 */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-700">配置位置</label>
        <div className="grid grid-cols-2 gap-2">
          {WEATHER_POSITION_OPTIONS.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => handlePositionChange(option.value)}
              className={`px-3 py-2 text-sm rounded-lg border transition-colors ${
                weatherPosition === option.value
                  ? 'border-blue-600 bg-blue-50 text-blue-700'
                  : 'border-gray-300 hover:border-gray-400'
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
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
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            設定
          </button>
        </div>
        <p className="text-xs text-gray-500">
          都市名を入力して「設定」を押すと、天気を取得してオーバーレイに反映します。APIキーは不要です。
        </p>
      </div>

      {/* 天気プレビュー */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-gray-700">天気プレビュー</span>
          <span className="text-xs text-gray-500">{formatTtl(cacheTtl)}</span>
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
            天気情報なし（「更新」ボタンを押してください）
          </div>
        )}

        <div className="flex gap-2">
          <button
            type="button"
            onClick={handleRefresh}
            disabled={loading}
            className="flex-1 px-4 py-2 bg-blue-100 text-blue-700 rounded-lg hover:bg-blue-200 disabled:opacity-50"
          >
            更新
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
        <p className="text-xs text-gray-400 text-center">
          更新: 最新の天気を取得 / 配信: オーバーレイに送信
        </p>
      </div>
    </div>
  );
}
