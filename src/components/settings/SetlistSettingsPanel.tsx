import type { SetlistSettings, SetlistPosition } from '../../types/overlaySettings';

interface SetlistSettingsPanelProps {
  settings: SetlistSettings;
  onChange: (settings: SetlistSettings) => void;
}

const POSITION_OPTIONS: { value: SetlistPosition; label: string }[] = [
  { value: 'top', label: '上' },
  { value: 'bottom', label: '下' },
  { value: 'left', label: '左' },
  { value: 'right', label: '右' },
];

export function SetlistSettingsPanel({ settings, onChange }: SetlistSettingsPanelProps) {
  const updateSettings = (updates: Partial<SetlistSettings>) => {
    onChange({ ...settings, ...updates });
  };

  return (
    <div className="space-y-6">
      {/* 表示ON/OFF */}
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-gray-700">セットリスト表示</label>
        <button
          type="button"
          onClick={() => updateSettings({ enabled: !settings.enabled })}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
            settings.enabled ? 'bg-blue-600' : 'bg-gray-300'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              settings.enabled ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>

      {/* 位置選択 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">表示位置</label>
        <div className="grid grid-cols-4 gap-2">
          {POSITION_OPTIONS.map((option) => (
            <button
              key={option.value}
              type="button"
              onClick={() => updateSettings({ position: option.value })}
              className={`px-4 py-2 text-sm rounded-lg border transition-colors ${
                settings.position === option.value
                  ? 'border-blue-600 bg-blue-50 text-blue-700'
                  : 'border-gray-300 hover:border-gray-400 text-gray-700'
              }`}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      {/* フォントサイズ */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          フォントサイズ: {settings.fontSize}px
        </label>
        <input
          type="range"
          min={16}
          max={36}
          value={settings.fontSize}
          onChange={(e) => updateSettings({ fontSize: parseInt(e.target.value, 10) })}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>16px</span>
          <span>36px</span>
        </div>
      </div>

      {/* アーティスト表示 */}
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-gray-700">アーティスト名表示</label>
        <button
          type="button"
          onClick={() => updateSettings({ showArtist: !settings.showArtist })}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
            settings.showArtist ? 'bg-blue-600' : 'bg-gray-300'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              settings.showArtist ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>
    </div>
  );
}
