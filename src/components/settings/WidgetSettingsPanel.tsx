import type { WidgetVisibilitySettings } from '../../types/overlaySettings';

interface WidgetSettingsPanelProps {
  settings: WidgetVisibilitySettings;
  onChange: (settings: WidgetVisibilitySettings) => void;
}

type WidgetKey = keyof WidgetVisibilitySettings;

interface WidgetOption {
  key: WidgetKey;
  label: string;
  slot: string;
  column: 'left' | 'right';
}

const WIDGET_OPTIONS: WidgetOption[] = [
  { key: 'clock', label: '時計', slot: 'left.top', column: 'left' },
  { key: 'weather', label: '天気', slot: 'left.topBelow', column: 'left' },
  { key: 'comment', label: 'コメント', slot: 'left.middle', column: 'left' },
  { key: 'superchat', label: 'スパチャ', slot: 'left.lower', column: 'left' },
  { key: 'logo', label: 'ロゴ', slot: 'left.bottom', column: 'left' },
  { key: 'setlist', label: 'セトリ', slot: 'right.upper', column: 'right' },
  { key: 'kpi', label: 'KPI', slot: 'right.lowerLeft', column: 'right' },
  { key: 'tanzaku', label: '短冊', slot: 'right.lowerRight', column: 'right' },
  { key: 'announcement', label: '告知', slot: 'right.bottom', column: 'right' },
];

export function WidgetSettingsPanel({ settings, onChange }: WidgetSettingsPanelProps) {
  const handleToggle = (key: WidgetKey) => {
    onChange({
      ...settings,
      [key]: !settings[key],
    });
  };

  const leftWidgets = WIDGET_OPTIONS.filter((w) => w.column === 'left');
  const rightWidgets = WIDGET_OPTIONS.filter((w) => w.column === 'right');

  const renderWidget = (widget: WidgetOption) => (
    <div
      key={widget.key}
      className="flex items-center justify-between py-2 px-3 bg-gray-50 rounded-lg"
    >
      <div className="flex items-center gap-3">
        <span className="text-sm font-medium text-gray-700">{widget.label}</span>
        <span className="text-xs text-gray-400">{widget.slot}</span>
      </div>
      <button
        type="button"
        onClick={() => handleToggle(widget.key)}
        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
          settings[widget.key] ? 'bg-blue-600' : 'bg-gray-300'
        }`}
      >
        <span
          className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
            settings[widget.key] ? 'translate-x-6' : 'translate-x-1'
          }`}
        />
      </button>
    </div>
  );

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-gray-900">ウィジェット表示設定</h3>
      <p className="text-sm text-gray-500">
        各ウィジェットの表示/非表示を切り替えます。詳細設定は各タブで行えます。
      </p>

      <div className="grid grid-cols-2 gap-6">
        {/* 左カラム */}
        <div className="space-y-3">
          <h4 className="text-sm font-medium text-gray-600 border-b pb-1">左カラム</h4>
          <div className="space-y-2">{leftWidgets.map(renderWidget)}</div>
        </div>

        {/* 右カラム */}
        <div className="space-y-3">
          <h4 className="text-sm font-medium text-gray-600 border-b pb-1">右カラム</h4>
          <div className="space-y-2">{rightWidgets.map(renderWidget)}</div>
        </div>
      </div>
    </div>
  );
}
