import { useState, useEffect, useCallback } from 'react';
import {
  getWeatherCity,
  getWeatherCacheTtl,
  refreshWeather,
  broadcastWeather,
  setWeatherCityAndBroadcast,
  broadcastWeatherMulti,
  setMultiCityMode,
  type WeatherData,
} from '../../types/weather';
import type {
  WeatherSettings,
  WeatherPosition,
  CityEntry,
  MultiCitySettings,
} from '../../types/overlaySettings';
import {
  DEFAULT_MULTI_CITIES,
  DEFAULT_MULTI_CITY_SETTINGS,
  ROTATION_INTERVAL_OPTIONS,
} from '../../types/overlaySettings';

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
  const multiCitySettings = settings?.multiCity ?? DEFAULT_MULTI_CITY_SETTINGS;

  // 単一都市モード用
  const [city, setCityValue] = useState('Tokyo');
  const [weather, setWeather] = useState<WeatherData | null>(null);
  const [cacheTtl, setCacheTtl] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // マルチシティモード用
  const [newCityName, setNewCityName] = useState('');
  const [newCityDisplayName, setNewCityDisplayName] = useState('');
  const [showAddCity, setShowAddCity] = useState(false);

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

  // マルチシティ設定変更ハンドラ
  const handleMultiCityChange = useCallback(
    (updates: Partial<MultiCitySettings>) => {
      const newMultiCity = {
        ...multiCitySettings,
        ...updates,
      };
      onChange?.({
        enabled: weatherEnabled,
        position: weatherPosition,
        multiCity: newMultiCity,
      });
    },
    [multiCitySettings, weatherEnabled, weatherPosition, onChange]
  );

  // 都市の有効/無効切替
  const handleToggleCity = useCallback(
    (cityId: string) => {
      const updatedCities = multiCitySettings.cities.map((c) =>
        c.id === cityId ? { ...c, enabled: !c.enabled } : c
      );
      handleMultiCityChange({ cities: updatedCities });
    },
    [multiCitySettings.cities, handleMultiCityChange]
  );

  // 都市の並び替え（上へ）
  const handleMoveUp = useCallback(
    (index: number) => {
      if (index === 0) return;
      const cities = [...multiCitySettings.cities];
      const prev = cities[index - 1];
      const curr = cities[index];
      if (prev && curr) {
        cities[index - 1] = curr;
        cities[index] = prev;
      }
      // orderを更新
      const updatedCities = cities.map((c, i) => ({ ...c, order: i }));
      handleMultiCityChange({ cities: updatedCities });
    },
    [multiCitySettings.cities, handleMultiCityChange]
  );

  // 都市の並び替え（下へ）
  const handleMoveDown = useCallback(
    (index: number) => {
      if (index === multiCitySettings.cities.length - 1) return;
      const cities = [...multiCitySettings.cities];
      const curr = cities[index];
      const next = cities[index + 1];
      if (curr && next) {
        cities[index] = next;
        cities[index + 1] = curr;
      }
      // orderを更新
      const updatedCities = cities.map((c, i) => ({ ...c, order: i }));
      handleMultiCityChange({ cities: updatedCities });
    },
    [multiCitySettings.cities, handleMultiCityChange]
  );

  // 都市の削除
  const handleRemoveCity = useCallback(
    (cityId: string) => {
      const updatedCities = multiCitySettings.cities
        .filter((c) => c.id !== cityId)
        .map((c, i) => ({ ...c, order: i }));
      handleMultiCityChange({ cities: updatedCities });
    },
    [multiCitySettings.cities, handleMultiCityChange]
  );

  // カスタム都市の追加
  const handleAddCity = useCallback(() => {
    if (!newCityName.trim() || !newCityDisplayName.trim()) return;

    const newCity: CityEntry = {
      id: `custom-${Date.now()}`,
      name: newCityName.trim(),
      displayName: newCityDisplayName.trim(),
      enabled: true,
      order: multiCitySettings.cities.length,
    };

    handleMultiCityChange({
      cities: [...multiCitySettings.cities, newCity],
    });

    setNewCityName('');
    setNewCityDisplayName('');
    setShowAddCity(false);
  }, [newCityName, newCityDisplayName, multiCitySettings.cities, handleMultiCityChange]);

  // デフォルト都市リストにリセット
  const handleResetCities = useCallback(() => {
    handleMultiCityChange({ cities: DEFAULT_MULTI_CITIES });
  }, [handleMultiCityChange]);

  // 都市名設定 + 更新 + 配信（単一都市モード）
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

  // 配信（オーバーレイに送信）- 単一都市モード
  const handleBroadcast = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // 自動更新を単一都市モードに設定
      await setMultiCityMode(false, [], 5);

      await broadcastWeather();
    } catch (err) {
      setError('配信に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, []);

  // マルチシティ配信
  const handleBroadcastMulti = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const enabledCities = multiCitySettings.cities
        .filter((c) => c.enabled)
        .sort((a, b) => a.order - b.order);

      if (enabledCities.length === 0) {
        setError('有効な都市がありません');
        return;
      }

      const cityTuples: Array<[string, string, string]> = enabledCities.map((c) => [
        c.id,
        c.name,
        c.displayName,
      ]);

      // 自動更新にマルチシティモードを反映（15分ごとの更新で使用）
      await setMultiCityMode(true, cityTuples, multiCitySettings.rotationIntervalSec);

      // 今すぐ配信
      await broadcastWeatherMulti(cityTuples, multiCitySettings.rotationIntervalSec);
    } catch (err) {
      setError('マルチシティ配信に失敗しました');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }, [multiCitySettings]);

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
      multiCity: multiCitySettings,
    });
  };

  const handleModeChange = (enabled: boolean) => {
    handleMultiCityChange({ enabled });
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

      {/* モード切替 */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-700">表示モード</label>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => handleModeChange(false)}
            className={`flex-1 px-4 py-2 text-sm rounded-lg border transition-colors ${
              !multiCitySettings.enabled
                ? 'border-blue-600 bg-blue-50 text-blue-700'
                : 'border-gray-300 hover:border-gray-400'
            }`}
          >
            単一都市
          </button>
          <button
            type="button"
            onClick={() => handleModeChange(true)}
            className={`flex-1 px-4 py-2 text-sm rounded-lg border transition-colors ${
              multiCitySettings.enabled
                ? 'border-blue-600 bg-blue-50 text-blue-700'
                : 'border-gray-300 hover:border-gray-400'
            }`}
          >
            マルチシティ
          </button>
        </div>
        <p className="text-xs text-gray-500">
          {multiCitySettings.enabled
            ? '身バレ防止: 複数都市の天気を自動ローテーション表示します'
            : '設定した都市の天気のみを表示します'}
        </p>
      </div>

      {/* 単一都市モード設定 */}
      {!multiCitySettings.enabled && (
        <>
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
              都市名を入力して「設定」を押すと、天気を取得してオーバーレイに反映します。
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
        </>
      )}

      {/* マルチシティモード設定 */}
      {multiCitySettings.enabled && (
        <>
          {/* ローテーション間隔 */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700">ローテーション間隔</label>
            <select
              value={multiCitySettings.rotationIntervalSec}
              onChange={(e) => handleMultiCityChange({ rotationIntervalSec: Number(e.target.value) })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              {ROTATION_INTERVAL_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <p className="text-xs text-gray-500">
              各都市の天気を表示する時間を設定します
            </p>
          </div>

          {/* 都市リスト */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="block text-sm font-medium text-gray-700">都市リスト</label>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={handleResetCities}
                  className="text-xs text-gray-500 hover:text-gray-700"
                >
                  リセット
                </button>
                <button
                  type="button"
                  onClick={() => setShowAddCity(true)}
                  className="text-xs text-blue-600 hover:text-blue-800"
                >
                  + 都市を追加
                </button>
              </div>
            </div>

            <div className="border border-gray-200 rounded-lg divide-y divide-gray-200 max-h-64 overflow-y-auto">
              {multiCitySettings.cities
                .sort((a, b) => a.order - b.order)
                .map((cityItem, index) => (
                  <div
                    key={cityItem.id}
                    className="flex items-center gap-2 px-3 py-2 hover:bg-gray-50"
                  >
                    <input
                      type="checkbox"
                      checked={cityItem.enabled}
                      onChange={() => handleToggleCity(cityItem.id)}
                      className="w-4 h-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                    />
                    <span
                      className={`flex-1 text-sm ${cityItem.enabled ? 'text-gray-900' : 'text-gray-400'}`}
                    >
                      {cityItem.displayName}
                    </span>
                    <span className="text-xs text-gray-400">{cityItem.name}</span>
                    <div className="flex gap-1">
                      <button
                        type="button"
                        onClick={() => handleMoveUp(index)}
                        disabled={index === 0}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
                        title="上へ"
                      >
                        ↑
                      </button>
                      <button
                        type="button"
                        onClick={() => handleMoveDown(index)}
                        disabled={index === multiCitySettings.cities.length - 1}
                        className="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
                        title="下へ"
                      >
                        ↓
                      </button>
                      {cityItem.id.startsWith('custom-') && (
                        <button
                          type="button"
                          onClick={() => handleRemoveCity(cityItem.id)}
                          className="p-1 text-red-400 hover:text-red-600"
                          title="削除"
                        >
                          ×
                        </button>
                      )}
                    </div>
                  </div>
                ))}
            </div>

            <p className="text-xs text-gray-500">
              チェックを入れた都市が北から南へ順に表示されます（{multiCitySettings.cities.filter((c) => c.enabled).length}都市有効）
            </p>
          </div>

          {/* カスタム都市追加フォーム */}
          {showAddCity && (
            <div className="p-4 bg-gray-50 rounded-lg space-y-3">
              <h4 className="text-sm font-medium text-gray-700">カスタム都市を追加</h4>
              <div className="space-y-2">
                <input
                  type="text"
                  value={newCityName}
                  onChange={(e) => setNewCityName(e.target.value)}
                  placeholder="都市名（英語）: Kyoto"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
                <input
                  type="text"
                  value={newCityDisplayName}
                  onChange={(e) => setNewCityDisplayName(e.target.value)}
                  placeholder="表示名（日本語）: 京都"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={handleAddCity}
                  disabled={!newCityName.trim() || !newCityDisplayName.trim()}
                  className="flex-1 px-3 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 disabled:opacity-50"
                >
                  追加
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setShowAddCity(false);
                    setNewCityName('');
                    setNewCityDisplayName('');
                  }}
                  className="flex-1 px-3 py-2 bg-gray-200 text-gray-700 text-sm rounded-lg hover:bg-gray-300"
                >
                  キャンセル
                </button>
              </div>
            </div>
          )}

          {/* マルチシティプレビュー・配信 */}
          <div className="space-y-3">
            <button
              type="button"
              onClick={handleBroadcastMulti}
              disabled={loading || multiCitySettings.cities.filter((c) => c.enabled).length === 0}
              className="w-full px-4 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
            >
              {loading ? '配信中...' : 'マルチシティ天気を配信'}
            </button>
            <p className="text-xs text-gray-400 text-center">
              有効な都市の天気を取得してオーバーレイに配信します
            </p>
          </div>
        </>
      )}
    </div>
  );
}
