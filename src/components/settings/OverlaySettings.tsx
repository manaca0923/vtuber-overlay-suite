import { useState, useEffect } from 'react';
import { ThemeSelector } from './ThemeSelector';
import { CommentSettingsPanel } from './CommentSettingsPanel';
import { SetlistSettingsPanel } from './SetlistSettingsPanel';
import { OverlayPreview } from './OverlayPreview';
import {
  DEFAULT_OVERLAY_SETTINGS,
  loadOverlaySettings,
  saveOverlaySettings,
  broadcastSettingsUpdate,
  type OverlaySettings as Settings,
  type ThemeName,
} from '../../types/overlaySettings';

export function OverlaySettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_OVERLAY_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [activePanel, setActivePanel] = useState<'comment' | 'setlist'>('comment');

  useEffect(() => {
    async function load() {
      try {
        const saved = await loadOverlaySettings();
        if (saved) {
          // 古い設定と新しいデフォルト値をマージ（マイグレーション対応）
          const merged: Settings = {
            theme: saved.theme ?? DEFAULT_OVERLAY_SETTINGS.theme,
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
            </div>

            <div className="p-6">
              {activePanel === 'comment' && (
                <CommentSettingsPanel
                  settings={settings.comment}
                  onChange={(comment) => setSettings((prev) => ({ ...prev, comment }))}
                />
              )}
              {activePanel === 'setlist' && (
                <SetlistSettingsPanel
                  settings={settings.setlist}
                  onChange={(setlist) => setSettings((prev) => ({ ...prev, setlist }))}
                />
              )}
            </div>
          </div>

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
        <div className="lg:sticky lg:top-4">
          <OverlayPreview settings={settings} activePanel={activePanel} />
        </div>
      </div>
    </div>
  );
}
