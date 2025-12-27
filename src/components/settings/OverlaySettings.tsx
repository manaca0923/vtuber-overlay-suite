import { useState, useEffect } from 'react';
import { ThemeSelector } from './ThemeSelector';
import { LayoutPresetSelector } from './LayoutPresetSelector';
import { CommentSettingsPanel } from './CommentSettingsPanel';
import { SetlistSettingsPanel } from './SetlistSettingsPanel';
import { WeatherSettingsPanel } from './WeatherSettingsPanel';
import { ApiKeySettingsPanel } from './ApiKeySettingsPanel';
import { OverlayPreview } from './OverlayPreview';
import {
  DEFAULT_OVERLAY_SETTINGS,
  loadOverlaySettings,
  saveOverlaySettings,
  broadcastSettingsUpdate,
  LAYOUT_PRESETS,
  type OverlaySettings as Settings,
  type ThemeName,
  type LayoutPreset,
} from '../../types/overlaySettings';

type PreviewMode = 'combined' | 'individual';

export function OverlaySettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_OVERLAY_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [activePanel, setActivePanel] = useState<'comment' | 'setlist' | 'weather'>('comment');
  const [previewMode, setPreviewMode] = useState<PreviewMode>('combined');

  useEffect(() => {
    async function load() {
      try {
        const saved = await loadOverlaySettings();
        if (saved) {
          // 古い設定と新しいデフォルト値をマージ（マイグレーション対応）
          const merged: Settings = {
            theme: saved.theme ?? DEFAULT_OVERLAY_SETTINGS.theme,
            layout: saved.layout ?? DEFAULT_OVERLAY_SETTINGS.layout,
            common: {
              ...DEFAULT_OVERLAY_SETTINGS.common,
              ...saved.common,
            },
            comment: {
              ...DEFAULT_OVERLAY_SETTINGS.comment,
              ...saved.comment,
            },
            setlist: {
              ...DEFAULT_OVERLAY_SETTINGS.setlist,
              ...saved.setlist,
            },
            weather: {
              ...DEFAULT_OVERLAY_SETTINGS.weather,
              ...saved.weather,
            },
          };
          setSettings(merged);
        }
      } catch (err) {
        console.error('Failed to load overlay settings:', err);
        setError('設定の読み込みに失敗しました。デフォルト設定を使用します。');
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
      setSuccess('設定を保存しました');
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(`保存に失敗しました: ${message}`);
    } finally {
      setSaving(false);
    }
  };

  const handleThemeChange = (theme: ThemeName, primaryColor: string) => {
    setSettings((prev) => ({
      ...prev,
      theme,
      common: { ...prev.common, primaryColor },
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

          {/* テーマ選択 */}
          <div className="bg-white rounded-lg shadow p-6">
            <ThemeSelector
              theme={settings.theme}
              primaryColor={settings.common.primaryColor}
              onChange={handleThemeChange}
            />
          </div>

          {/* タブ切替 */}
          <div className="bg-white rounded-lg shadow">
            <div className="flex border-b">
              <button
                onClick={() => setActivePanel('comment')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'comment'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                コメント設定
              </button>
              <button
                onClick={() => setActivePanel('setlist')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'setlist'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                セットリスト設定
              </button>
              <button
                onClick={() => setActivePanel('weather')}
                className={`flex-1 py-3 px-4 text-sm font-medium transition-colors ${
                  activePanel === 'weather'
                    ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                天気設定
              </button>
            </div>

            <div className="p-6">
              {activePanel === 'comment' && (
                <CommentSettingsPanel
                  settings={settings.comment}
                  onChange={(comment) => {
                    setSettings((prev) => ({ ...prev, comment, layout: 'custom' }));
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
              {activePanel === 'weather' && (
                <WeatherSettingsPanel
                  settings={settings.weather}
                  onChange={(weather) => {
                    setSettings((prev) => ({ ...prev, weather }));
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
              activePanel={activePanel === 'weather' ? 'comment' : activePanel}
              mode={previewMode}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
