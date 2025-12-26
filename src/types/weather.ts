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

// Tauri Commands - 天気API
export const setWeatherApiKey = (apiKey: string) =>
  invoke<void>('set_weather_api_key', { apiKey });

export const hasWeatherApiKey = () =>
  invoke<boolean>('has_weather_api_key');

export const setWeatherCity = (city: string) =>
  invoke<void>('set_weather_city', { city });

export const getWeatherCity = () =>
  invoke<string>('get_weather_city');

export const getWeather = () =>
  invoke<WeatherData>('get_weather');

export const fetchWeather = () =>
  invoke<WeatherData>('fetch_weather');

export const broadcastWeatherUpdate = () =>
  invoke<void>('broadcast_weather_update');

export const clearWeatherCache = () =>
  invoke<void>('clear_weather_cache');

export const getWeatherCacheTtl = () =>
  invoke<number>('get_weather_cache_ttl');

// Tauri Commands - KPI/視聴者数
export const getLiveStreamStats = (videoId: string, useBundledKey: boolean) =>
  invoke<LiveStreamStats>('get_live_stream_stats', { videoId, useBundledKey });

export const broadcastKpiUpdate = (
  main: number | null,
  label: string | null,
  sub: number | null,
  subLabel: string | null
) =>
  invoke<void>('broadcast_kpi_update', { main, label, sub, subLabel });
