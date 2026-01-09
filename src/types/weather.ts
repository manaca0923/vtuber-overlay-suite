import { invoke } from '@tauri-apps/api/core';

// 天気データ
export interface WeatherData {
  icon: string;
  temp: number;
  description: string;
  location: string;
  humidity: number;
  weatherCode: number;
  fetchedAt: number;
}

// ライブ配信統計情報
export interface LiveStreamStats {
  concurrentViewers: number | null;
  likeCount: number | null;
  viewCount: number | null;
}

// Tauri Commands - 天気API（Open-Meteo - APIキー不要）
export const setWeatherCity = (city: string) =>
  invoke<void>('set_weather_city', { city });

export const getWeatherCity = () =>
  invoke<string>('get_weather_city');

export const getWeather = () =>
  invoke<WeatherData>('get_weather');

export const fetchWeather = () =>
  invoke<WeatherData>('fetch_weather');

export const broadcastWeatherUpdate = (forceRefresh?: boolean) =>
  invoke<void>('broadcast_weather_update', { force_refresh: forceRefresh });

export const clearWeatherCache = () =>
  invoke<void>('clear_weather_cache');

export const getWeatherCacheTtl = () =>
  invoke<number>('get_weather_cache_ttl');

// 新UI用コマンド（2ボタン化対応）

/** 天気を手動更新（キャッシュクリア + 取得 + タイマーリセット） */
export const refreshWeather = () =>
  invoke<WeatherData>('refresh_weather');

/** 天気をオーバーレイに配信 */
export const broadcastWeather = () =>
  invoke<void>('broadcast_weather');

/** 都市名設定 + 更新 + 配信（一括処理） */
export const setWeatherCityAndBroadcast = (city: string) =>
  invoke<WeatherData>('set_weather_city_and_broadcast', { city });

// マルチシティモード用の型と関数

/** 都市タプル: [cityId, cityName, displayName] */
export type CityTuple = [id: string, name: string, displayName: string];

/** 都市ごとの天気データ */
export interface CityWeatherData {
  cityId: string;
  cityName: string;
  icon: string;
  temp: number;
  description: string;
  location: string;
  humidity: number | null;
}

/** マルチシティ配信結果 */
export interface BroadcastMultiResult {
  /** 成功した都市数 */
  success_count: number;
  /** 総都市数 */
  total_count: number;
}

/** 複数都市の天気を取得 */
export const getWeatherMulti = (cities: CityTuple[]) =>
  invoke<CityWeatherData[]>('get_weather_multi', { cities });

/** 複数都市の天気をオーバーレイに配信（成功/失敗カウントを返す） */
export const broadcastWeatherMulti = (
  cities: CityTuple[],
  rotationIntervalSec: number
) =>
  invoke<BroadcastMultiResult>('broadcast_weather_multi', {
    cities,
    rotation_interval_sec: rotationIntervalSec,
  });

/** マルチシティモードを自動更新に反映 */
export const setMultiCityMode = (
  enabled: boolean,
  cities: CityTuple[],
  rotationIntervalSec: number
) =>
  invoke<void>('set_multi_city_mode', {
    enabled,
    cities,
    rotation_interval_sec: rotationIntervalSec,
  });

// Tauri Commands - KPI/視聴者数
// 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
export const getLiveStreamStats = (videoId: string, useBundledKey: boolean) =>
  invoke<LiveStreamStats>('get_live_stream_stats', { video_id: videoId, use_bundled_key: useBundledKey });

export const broadcastKpiUpdate = (
  main: number | null,
  label: string | null,
  sub: number | null,
  subLabel: string | null
) =>
  invoke<void>('broadcast_kpi_update', { main, label, sub, sub_label: subLabel });
