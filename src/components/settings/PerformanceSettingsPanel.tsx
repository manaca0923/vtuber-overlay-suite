import type { PerformanceSettings } from '../../types/overlaySettings';

interface PerformanceSettingsPanelProps {
  className?: string;
  settings?: PerformanceSettings;
  onChange?: (settings: PerformanceSettings) => void;
}

export function PerformanceSettingsPanel({
  className = '',
  settings,
  onChange,
}: PerformanceSettingsPanelProps) {
  // デフォルト値
  const densityThreshold = settings?.densityThreshold ?? 5;

  const handleThresholdChange = (value: number) => {
    // 1-20の範囲にクランプ
    const clamped = Math.max(1, Math.min(20, value));
    onChange?.({
      densityThreshold: clamped,
    });
  };

  return (
    <div className={`space-y-6 ${className}`}>
      <h3 className="text-lg font-semibold text-gray-900">詳細設定</h3>

      {/* 過密検出閾値 */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <label
            htmlFor="density-threshold"
            className="block text-sm font-medium text-gray-700"
          >
            過密検出閾値
          </label>
          <span className="text-sm font-mono text-gray-600">
            {densityThreshold}回/2秒
          </span>
        </div>

        <input
          type="range"
          id="density-threshold"
          aria-label="過密検出閾値"
          min={1}
          max={20}
          value={densityThreshold}
          onChange={(e) => handleThresholdChange(parseInt(e.target.value, 10))}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
        />

        <div className="flex justify-between text-xs text-gray-500">
          <span>敏感（1）</span>
          <span>標準（5）</span>
          <span>鈍感（20）</span>
        </div>

        <p className="text-xs text-gray-500 mt-2">
          2秒間に右下エリア（KPI・待機キュー・告知）の更新がこの回数を超えると、
          表示間隔を自動調整して負荷を軽減します。
        </p>
      </div>

      {/* 説明 */}
      <div className="p-3 bg-blue-50 border border-blue-100 rounded-lg">
        <h4 className="text-sm font-medium text-blue-800 mb-1">
          過密検出について
        </h4>
        <p className="text-xs text-blue-700">
          配信中にKPI・待機キュー・告知が頻繁に更新されると、
          視聴者にとって見づらくなります。過密検出により、
          高負荷時は自動的に更新間隔が長くなり、
          見やすさが維持されます。
        </p>
      </div>
    </div>
  );
}
