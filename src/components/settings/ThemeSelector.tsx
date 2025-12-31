import { useState } from 'react';
import {
  THEME_PRESETS,
  WIDGET_IDS,
  type ThemeSettings,
  type CustomColorEntry,
  type WidgetColorOverrides,
  type WidgetId,
} from '../../types/overlaySettings';

// プリセットの値型を抽出（issues/020: マジックナンバー定数化対応）
type ThemePresetValue = (typeof THEME_PRESETS)[keyof typeof THEME_PRESETS];

// ウィジェット表示名マップ（issues/020: 定数化）
const WIDGET_DISPLAY_NAMES: Record<WidgetId, string> = {
  clock: '時計',
  weather: '天気',
  comment: 'コメント',
  superchat: 'スパチャ',
  logo: 'ロゴ',
  setlist: 'セトリ',
  kpi: 'KPI',
  tanzaku: '短冊',
  announcement: '告知',
};

interface ThemeSelectorProps {
  themeSettings: ThemeSettings;
  onChange: (settings: ThemeSettings) => void;
}

/**
 * テーマ設定コンポーネント
 * - プリセット選択（4種: white, purple, sakura, ocean）
 * - カスタムカラー（最大3件）
 * - ウィジェット個別カラー設定
 *
 * issues/013: アクセシビリティ対応（id, htmlFor, aria-label）
 */
export function ThemeSelector({ themeSettings, onChange }: ThemeSelectorProps) {
  const [showWidgetColors, setShowWidgetColors] = useState(false);

  // プリセット選択ハンドラ
  const handlePresetSelect = (presetKey: keyof typeof THEME_PRESETS) => {
    const preset = THEME_PRESETS[presetKey];
    onChange({
      ...themeSettings,
      globalTheme: presetKey,
      globalPrimaryColor: preset.primaryColor,
    });
  };

  // カスタムカラー変更ハンドラ
  const handleCustomColorChange = (color: string) => {
    onChange({
      ...themeSettings,
      globalTheme: 'custom',
      globalPrimaryColor: color,
    });
  };

  // カスタムカラーエントリ追加
  const handleAddCustomColor = () => {
    if (themeSettings.customColors.length >= 3) return;

    const newEntry: CustomColorEntry = {
      id: crypto.randomUUID(),
      name: `カラー${themeSettings.customColors.length + 1}`,
      color: themeSettings.globalPrimaryColor,
    };

    onChange({
      ...themeSettings,
      customColors: [...themeSettings.customColors, newEntry],
    });
  };

  // カスタムカラーエントリ更新
  const handleUpdateCustomColor = (
    id: string,
    updates: Partial<Pick<CustomColorEntry, 'name' | 'color'>>
  ) => {
    const updatedColors = themeSettings.customColors.map((entry) =>
      entry.id === id ? { ...entry, ...updates } : entry
    );
    onChange({
      ...themeSettings,
      customColors: updatedColors,
    });
  };

  // カスタムカラーエントリ削除
  const handleRemoveCustomColor = (id: string) => {
    onChange({
      ...themeSettings,
      customColors: themeSettings.customColors.filter((entry) => entry.id !== id),
    });
  };

  // カスタムカラーを適用
  const handleApplyCustomColor = (color: string) => {
    onChange({
      ...themeSettings,
      globalTheme: 'custom',
      globalPrimaryColor: color,
    });
  };

  // ウィジェット個別カラー変更
  const handleWidgetColorChange = (widgetId: WidgetId, color: string | null) => {
    const newOverrides: WidgetColorOverrides = { ...themeSettings.widgetColorOverrides };
    if (color === null) {
      delete newOverrides[widgetId];
    } else {
      newOverrides[widgetId] = color;
    }
    onChange({
      ...themeSettings,
      widgetColorOverrides: newOverrides,
    });
  };

  return (
    <div className="space-y-6">
      {/* セクションタイトル */}
      <h3 className="text-lg font-bold text-gray-900">テーマ設定</h3>

      {/* プリセット選択 */}
      <div className="space-y-2">
        <p className="text-sm font-medium text-gray-700">プリセット</p>
        <div className="grid grid-cols-4 gap-3">
          {(
            Object.entries(THEME_PRESETS) as [keyof typeof THEME_PRESETS, ThemePresetValue][]
          ).map(([key, preset]) => (
            <button
              key={key}
              type="button"
              id={`theme-preset-${key}`}
              aria-label={`${preset.name}テーマを選択`}
              onClick={() => handlePresetSelect(key)}
              className={`p-3 border-2 rounded-lg cursor-pointer transition-all text-left ${
                themeSettings.globalTheme === key
                  ? 'border-blue-600 bg-blue-50'
                  : 'border-gray-200 hover:border-gray-400'
              }`}
            >
              <div
                className="w-full h-6 rounded mb-2 border border-gray-200"
                style={{ backgroundColor: preset.primaryColor }}
              />
              <p className="text-sm font-medium text-gray-900">{preset.name}</p>
            </button>
          ))}
        </div>
      </div>

      {/* カスタムカラー（現在選択中） */}
      <div className="space-y-2">
        <label htmlFor="custom-color-picker" className="text-sm font-medium text-gray-700">
          カスタムカラー
        </label>
        <div className="flex items-center gap-3">
          <input
            type="color"
            id="custom-color-picker"
            aria-label="カスタムカラーを選択"
            value={themeSettings.globalPrimaryColor}
            onChange={(e) => handleCustomColorChange(e.target.value)}
            className="w-10 h-10 rounded cursor-pointer border border-gray-300"
          />
          <span className="text-sm text-gray-500 font-mono">
            {themeSettings.globalPrimaryColor}
          </span>
          {themeSettings.globalTheme === 'custom' && (
            <span className="text-xs text-blue-600 bg-blue-100 px-2 py-1 rounded">選択中</span>
          )}
        </div>
      </div>

      {/* 保存済みカスタムカラー（最大3件） */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <p className="text-sm font-medium text-gray-700">保存済みカラー</p>
          {themeSettings.customColors.length < 3 && (
            <button
              type="button"
              onClick={handleAddCustomColor}
              className="text-sm text-blue-600 hover:text-blue-800"
              aria-label="カスタムカラーを追加"
            >
              + 追加
            </button>
          )}
        </div>
        {themeSettings.customColors.length === 0 ? (
          <p className="text-sm text-gray-400">保存済みカラーはありません</p>
        ) : (
          <div className="space-y-2">
            {themeSettings.customColors.map((entry, index) => (
              <div key={entry.id} className="flex items-center gap-2">
                <input
                  type="color"
                  id={`custom-color-${index}`}
                  value={entry.color}
                  onChange={(e) => handleUpdateCustomColor(entry.id, { color: e.target.value })}
                  className="w-8 h-8 rounded cursor-pointer border border-gray-300"
                  aria-label={`カスタムカラー${index + 1}の色`}
                />
                <input
                  type="text"
                  id={`custom-color-name-${index}`}
                  value={entry.name}
                  onChange={(e) => handleUpdateCustomColor(entry.id, { name: e.target.value })}
                  className="flex-1 px-2 py-1 text-sm border border-gray-300 rounded"
                  aria-label={`カスタムカラー${index + 1}の名前`}
                  maxLength={20}
                />
                <button
                  type="button"
                  onClick={() => handleApplyCustomColor(entry.color)}
                  className="text-xs text-blue-600 hover:text-blue-800 px-2 py-1"
                  aria-label={`${entry.name}を適用`}
                >
                  適用
                </button>
                <button
                  type="button"
                  onClick={() => handleRemoveCustomColor(entry.id)}
                  className="text-xs text-red-600 hover:text-red-800 px-2 py-1"
                  aria-label={`${entry.name}を削除`}
                >
                  削除
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* ウィジェット個別カラー設定 */}
      <div className="space-y-2 border-t pt-4">
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="widget-color-toggle"
            checked={showWidgetColors}
            onChange={(e) => setShowWidgetColors(e.target.checked)}
            className="w-4 h-4 text-blue-600 rounded"
          />
          <label htmlFor="widget-color-toggle" className="text-sm font-medium text-gray-700">
            ウィジェット個別にカラーを設定
          </label>
        </div>

        {showWidgetColors && (
          <div className="grid grid-cols-3 gap-3 mt-3 p-3 bg-gray-50 rounded-lg">
            {WIDGET_IDS.filter((id) => id !== 'superchat').map((widgetId) => {
              const currentColor =
                themeSettings.widgetColorOverrides[widgetId] || themeSettings.globalPrimaryColor;
              const hasOverride = widgetId in themeSettings.widgetColorOverrides;

              return (
                <div key={widgetId} className="flex items-center gap-2">
                  <input
                    type="color"
                    id={`widget-color-${widgetId}`}
                    value={currentColor}
                    onChange={(e) => handleWidgetColorChange(widgetId, e.target.value)}
                    className="w-6 h-6 rounded cursor-pointer border border-gray-300"
                    aria-label={`${WIDGET_DISPLAY_NAMES[widgetId]}の色`}
                  />
                  <span className="text-xs text-gray-700">{WIDGET_DISPLAY_NAMES[widgetId]}</span>
                  {hasOverride && (
                    <button
                      type="button"
                      onClick={() => handleWidgetColorChange(widgetId, null)}
                      className="text-xs text-gray-400 hover:text-gray-600"
                      aria-label={`${WIDGET_DISPLAY_NAMES[widgetId]}の色をリセット`}
                    >
                      ✕
                    </button>
                  )}
                </div>
              );
            })}
            <p className="col-span-3 text-xs text-gray-500 mt-2">
              ※ スパチャはYouTube Tierカラーを使用するため個別設定不可
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
