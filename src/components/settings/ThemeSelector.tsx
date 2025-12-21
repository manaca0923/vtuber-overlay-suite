import { THEME_PRESETS, type ThemeName } from '../../types/overlaySettings';

interface ThemeSelectorProps {
  theme: ThemeName;
  primaryColor: string;
  onChange: (theme: ThemeName, primaryColor: string) => void;
}

export function ThemeSelector({ theme, primaryColor, onChange }: ThemeSelectorProps) {
  return (
    <div className="space-y-4">
      <h3 className="text-lg font-bold text-gray-900">テーマ選択</h3>
      <div className="grid grid-cols-3 gap-4">
        {(Object.entries(THEME_PRESETS) as [ThemeName, typeof THEME_PRESETS.default][]).map(
          ([key, preset]) => (
            <div
              key={key}
              role="button"
              tabIndex={0}
              onClick={() => onChange(key, preset.primaryColor)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onChange(key, preset.primaryColor);
                }
              }}
              className={`p-4 border-2 rounded-lg cursor-pointer transition-all ${
                theme === key
                  ? 'border-blue-600 bg-blue-50'
                  : 'border-gray-300 hover:border-gray-400'
              }`}
            >
              <div
                className="w-full h-8 rounded mb-2"
                style={{ backgroundColor: preset.primaryColor }}
              />
              <p className="font-medium text-gray-900">{preset.name}</p>
              <p className="text-sm text-gray-500">{preset.description}</p>
            </div>
          )
        )}
      </div>

      {/* カスタムカラー */}
      <div className="flex items-center gap-4 pt-2">
        <label className="text-sm font-medium text-gray-700">カスタムカラー:</label>
        <input
          type="color"
          value={primaryColor}
          onChange={(e) => onChange('custom', e.target.value)}
          className="w-10 h-10 rounded cursor-pointer border border-gray-300"
        />
        <span className="text-sm text-gray-500 font-mono">{primaryColor}</span>
      </div>
    </div>
  );
}
