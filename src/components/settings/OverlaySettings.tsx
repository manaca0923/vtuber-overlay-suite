import { useState, useEffect, useMemo } from 'react';
import { ThemeSelector } from './ThemeSelector';
import { FontSelector } from './FontSelector';
import { LayoutPresetSelector } from './LayoutPresetSelector';
import { CommentSettingsPanel } from './CommentSettingsPanel';
import { SuperchatSettingsPanel } from './SuperchatSettingsPanel';
import { SetlistSettingsPanel } from './SetlistSettingsPanel';
import { WeatherSettingsPanel } from './WeatherSettingsPanel';
import { PerformanceSettingsPanel } from './PerformanceSettingsPanel';
import { ApiKeySettingsPanel } from './ApiKeySettingsPanel';
import { WidgetSettingsPanel } from './WidgetSettingsPanel';
import { QueueSettingsPanel } from './QueueSettingsPanel';
import { PromoSettingsPanel } from './PromoSettingsPanel';
import { OverlayPreview } from './OverlayPreview';
import {
  DEFAULT_OVERLAY_SETTINGS,
  DEFAULT_THEME_SETTINGS,
  THEME_PRESETS,
  loadOverlaySettings,
  saveOverlaySettings,
  broadcastSettingsUpdate,
  LAYOUT_PRESETS,
  type OverlaySettings as Settings,
  type ThemeSettings,
  type LayoutPreset,
  type WidgetVisibilitySettings,
} from '../../types/overlaySettings';

type PreviewMode = 'combined' | 'individual';

export function OverlaySettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_OVERLAY_SETTINGS);
  const [originalSettings, setOriginalSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [activePanel, setActivePanel] = useState<'widget' | 'comment' | 'superchat' | 'setlist' | 'queue' | 'promo' | 'weather' | 'performance'>('widget');
  const [previewMode, setPreviewMode] = useState<PreviewMode>('combined');

  // 変更検出（リセットボタンの表示/非表示制御用）
  const hasChanges = useMemo(() => {
    if (!originalSettings) return false;
    return JSON.stringify(settings) !== JSON.stringify(originalSettings);
  }, [settings, originalSettings]);

  // リセット機能
  const handleReset = () => {
    if (originalSettings) {
      setSettings(originalSettings);
    }
  };

  useEffect(() => {
    async function load() {
      try {
        const saved = await loadOverlaySettings();
        if (saved) {
          // 旧レイアウトプリセット（streaming/talk/music/gaming）を新プリセットにマイグレーション
          const validLayouts: LayoutPreset[] = ['custom', 'three-column'];
          const migratedLayout: LayoutPreset = validLayouts.includes(saved.layout as LayoutPreset)
            ? (saved.layout as LayoutPreset)
            : 'three-column'; // 旧プリセットはthree-columnにフォールバック

          // 古い設定と新しいデフォルト値をマージ（マイグレーション対応）
          // widget設定のマイグレーション（既存のenabled設定から生成）
          const migratedWidget: WidgetVisibilitySettings = saved.widget
            ? { ...DEFAULT_OVERLAY_SETTINGS.widget, ...saved.widget }
            : {
                clock: true,
                weather: saved.weather?.enabled ?? true,
                comment: saved.comment?.enabled ?? true,
                superchat: true,
                logo: true,
                setlist: saved.setlist?.enabled ?? true,
                kpi: true,
                tanzaku: true,
                announcement: true,
              };

          // themeSettingsのマイグレーション（issues/016: 後方互換性）
          // themeSettingsがない場合は既存のtheme/primaryColorから生成
          const migratedThemeSettings: ThemeSettings = saved.themeSettings
            ? { ...DEFAULT_THEME_SETTINGS, ...saved.themeSettings }
            : {
                // 旧themeをglobalThemeにマッピング
                // 'default'は旧プリセット（現在は'purple'に相当）
                globalTheme: saved.theme && saved.theme in THEME_PRESETS
                  ? (saved.theme as keyof typeof THEME_PRESETS)
                  : (saved.theme as string) === 'default' ? 'purple' : 'white',
                globalPrimaryColor: saved.common?.primaryColor || DEFAULT_THEME_SETTINGS.globalPrimaryColor,
                customColors: [],
                widgetColorOverrides: {},
                fontPreset: 'yu-gothic',
                customFontFamily: null,
              };

          const merged: Settings = {
            theme: migratedThemeSettings.globalTheme,
            layout: migratedLayout,
            common: {
              ...DEFAULT_OVERLAY_SETTINGS.common,
              ...saved.common,
              // themeSettingsからprimaryColorを同期
              primaryColor: migratedThemeSettings.globalPrimaryColor,
            },
            comment: {
              ...DEFAULT_OVERLAY_SETTINGS.comment,
              ...saved.comment,
            },
            setlist: {
              ...DEFAULT_OVERLAY_SETTINGS.setlist,
              ...saved.setlist,
            },
            // saved.weatherがundefinedの場合はデフォルト値を使用
            weather: saved.weather
              ? { ...DEFAULT_OVERLAY_SETTINGS.weather, ...saved.weather }
              : DEFAULT_OVERLAY_SETTINGS.weather,
            // saved.performanceがundefinedの場合はデフォルト値を使用
            performance: saved.performance
              ? { ...DEFAULT_OVERLAY_SETTINGS.performance, ...saved.performance }
              : DEFAULT_OVERLAY_SETTINGS.performance,
            // widget設定
            widget: migratedWidget,
            // saved.superchatがundefinedの場合はデフォルト値を使用
            superchat: saved.superchat
              ? { ...DEFAULT_OVERLAY_SETTINGS.superchat, ...saved.superchat }
              : DEFAULT_OVERLAY_SETTINGS.superchat,
            // テーマ設定
            themeSettings: migratedThemeSettings,
          };
          setSettings(merged);
          setOriginalSettings(merged); // 元設定を保存（リセット機能用）
        }
      } catch (err) {
        console.error('Failed to load overlay settings:', err);
        setError('設定の読み込みに失敗しました。デフォルト設定を使用します。');
        // エラー時もデフォルト設定を元設定として保存
        setOriginalSettings(DEFAULT_OVERLAY_SETTINGS);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setError('');
    setSuccess('');

    try {
      await saveOverlaySettings(settings);
      await broadcastSettingsUpdate(settings);
      setOriginalSettings(settings); // 保存成功後に元設定を更新
      setSuccess('設定を保存しました');
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(`保存に失敗しました: ${message}`);
    } finally {
      setSaving(false);
    }
  };

  // テーマ設定変更ハンドラ（ThemeSettings統合版）
  const handleThemeSettingsChange = (themeSettings: ThemeSettings) => {
    setSettings((prev) => ({
      ...prev,
      theme: themeSettings.globalTheme,
      common: { ...prev.common, primaryColor: themeSettings.globalPrimaryColor },
      themeSettings,
    }));
  };

  const handleLayoutChange = (layout: LayoutPreset) => {
    // プリセット以外（custom）の場合はレイアウトのみ変更
    if (layout === 'custom') {
      setSettings((prev) => ({ ...prev, layout }));
      return;
    }

    // プリセットの設定を適用
    const presetConfig = LAYOUT_PRESETS[layout];
    setSettings((prev) => ({
      ...prev,
      layout,
      comment: {
        ...prev.comment,
        position: presetConfig.comment.position,
        enabled: presetConfig.comment.enabled,
      },
      setlist: {
        ...prev.setlist,
        position: presetConfig.setlist.position,
        enabled: presetConfig.setlist.enabled,
      },
    }));
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600">読み込み中...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-gray-900">オーバーレイ設定</h2>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 左: 設定パネル */}
        <div className="space-y-6">
          {/* レイアウトプリセット */}
          <div className="bg-white rounded-lg shadow p-6">
            <LayoutPresetSelector
              selected={settings.layout}
              onChange={handleLayoutChange}
            />
          </div>

          {/* テーマ設定 */}
          <div className="bg-white rounded-lg shadow p-6">
            <ThemeSelector
              themeSettings={settings.themeSettings ?? DEFAULT_THEME_SETTINGS}
              onChange={handleThemeSettingsChange}
            />
          </div>

          {/* フォント設定 */}
          <div className="bg-white rounded-lg shadow p-6">
            <FontSelector
              themeSettings={settings.themeSettings ?? DEFAULT_THEME_SETTINGS}
              onChange={handleThemeSettingsChange}
            />
          </div>

          {/* タブ切替 */}
          <div className="bg-white rounded-lg shadow">
            <div className="flex border-b">
              <button
                onClick={() => setActivePanel('widget')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'widget'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                ウィジェット
              </button>
              <button
                onClick={() => setActivePanel('comment')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'comment'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                コメント
              </button>
              <button
                onClick={() => setActivePanel('superchat')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'superchat'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                スパチャ
              </button>
              <button
                onClick={() => setActivePanel('setlist')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'setlist'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                セトリ
              </button>
              <button
                onClick={() => setActivePanel('queue')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'queue'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                短冊
              </button>
              <button
                onClick={() => setActivePanel('promo')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'promo'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                告知
              </button>
              <button
                onClick={() => setActivePanel('weather')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'weather'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                天気
              </button>
              <button
                onClick={() => setActivePanel('performance')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'performance'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                詳細
              </button>
            </div>

            <div className="p-6">
              {activePanel === 'widget' && settings.widget && (
                <WidgetSettingsPanel
                  settings={settings.widget}
                  onChange={(widget) => {
                    // widget設定と各enabled設定を同期
                    setSettings((prev) => ({
                      ...prev,
                      widget,
                      comment: { ...prev.comment, enabled: widget.comment },
                      setlist: { ...prev.setlist, enabled: widget.setlist },
                      weather: prev.weather
                        ? { ...prev.weather, enabled: widget.weather }
                        : { enabled: widget.weather, position: 'left-top' },
                    }));
                  }}
                />
              )}
              {activePanel === 'comment' && (
                <CommentSettingsPanel
                  settings={settings.comment}
                  onChange={(comment) => {
                    setSettings((prev) => ({ ...prev, comment, layout: 'custom' }));
                  }}
                />
              )}
              {activePanel === 'superchat' && (
                <SuperchatSettingsPanel
                  settings={settings.superchat ?? DEFAULT_OVERLAY_SETTINGS.superchat!}
                  onChange={(superchat) => {
                    setSettings((prev) => ({ ...prev, superchat }));
                  }}
                />
              )}
              {activePanel === 'setlist' && (
                <SetlistSettingsPanel
                  settings={settings.setlist}
                  onChange={(setlist) => {
                    setSettings((prev) => ({ ...prev, setlist, layout: 'custom' }));
                  }}
                />
              )}
              {activePanel === 'queue' && (
                <QueueSettingsPanel />
              )}
              {activePanel === 'promo' && (
                <PromoSettingsPanel />
              )}
              {activePanel === 'weather' && (
                <WeatherSettingsPanel
                  settings={settings.weather}
                  onChange={(weather) => {
                    setSettings((prev) => ({ ...prev, weather }));
                  }}
                />
              )}
              {activePanel === 'performance' && (
                <PerformanceSettingsPanel
                  settings={settings.performance}
                  onChange={(performance) => {
                    setSettings((prev) => ({ ...prev, performance }));
                  }}
                />
              )}
            </div>
          </div>

          {/* YouTube APIキー設定 */}
          <ApiKeySettingsPanel />

          {/* メッセージ表示 */}
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

          {/* リセットボタン（変更がある場合のみ表示） */}
          {hasChanges && (
            <button
              onClick={handleReset}
              className="w-full py-2 rounded-lg font-medium bg-gray-200 text-gray-700 hover:bg-gray-300 transition-colors"
            >
              変更を元に戻す
            </button>
          )}

          {/* 保存ボタン */}
          <button
            onClick={handleSave}
            disabled={saving}
            className={`w-full py-3 rounded-lg font-medium transition-colors ${
              saving
                ? 'bg-gray-400 text-white cursor-not-allowed'
                : 'bg-blue-600 text-white hover:bg-blue-700'
            }`}
          >
            {saving ? '保存中...' : '設定を保存'}
          </button>
        </div>

        {/* 右: プレビュー */}
        <div className="lg:sticky lg:top-4 space-y-4">
          {/* プレビューモード切替 */}
          <div className="flex items-center gap-2 bg-white rounded-lg shadow p-3">
            <span className="text-sm text-gray-600">プレビュー:</span>
            <button
              onClick={() => setPreviewMode('combined')}
              className={`px-3 py-1 text-sm rounded-lg transition-colors ${
                previewMode === 'combined'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              統合
            </button>
            <button
              onClick={() => setPreviewMode('individual')}
              className={`px-3 py-1 text-sm rounded-lg transition-colors ${
                previewMode === 'individual'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              個別
            </button>
          </div>

          {/* プレビュー */}
          <div className="h-[500px]">
            <OverlayPreview
              settings={settings}
              activePanel={activePanel === 'widget' || activePanel === 'weather' || activePanel === 'performance' || activePanel === 'superchat' || activePanel === 'queue' || activePanel === 'promo' ? 'comment' : activePanel}
              mode={previewMode}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
