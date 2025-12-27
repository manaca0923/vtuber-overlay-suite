import type { LayoutPreset } from '../../types/overlaySettings';
import { LAYOUT_PRESETS } from '../../types/overlaySettings';

interface LayoutPresetSelectorProps {
  selected: LayoutPreset;
  onChange: (preset: LayoutPreset) => void;
}

// 3カラムプレビュー用SVG
function ThreeColumnPreview() {
  return (
    <svg viewBox="0 0 40 40" className="w-full h-full">
      {/* 背景 */}
      <rect x="0" y="0" width="40" height="40" fill="#1f2937" rx="4" />
      {/* 左カラム（22%） */}
      <rect x="2" y="2" width="8" height="36" fill="#6366f1" rx="2" opacity="0.8" />
      {/* 中央カラム（56%）- 空きスペース表示 */}
      <rect x="12" y="2" width="16" height="36" fill="#374151" rx="2" opacity="0.5" />
      {/* 右カラム（22%） */}
      <rect x="30" y="2" width="8" height="36" fill="#10b981" rx="2" opacity="0.8" />
      {/* 中央ラベル */}
      <text x="20" y="22" textAnchor="middle" fill="#9ca3af" fontSize="6">主役</text>
    </svg>
  );
}

// プリセットのミニチュアプレビュー用SVG
function PresetPreview({ preset }: { preset: LayoutPreset }) {
  // 3カラムは専用プレビュー
  if (preset === 'three-column') {
    return <ThreeColumnPreview />;
  }

  const config = LAYOUT_PRESETS[preset];
  const commentPos = config.comment.position;
  const setlistPos = config.setlist.position;

  // コメントエリアの位置
  const commentRect = {
    'top-left': { x: 2, y: 2, width: 14, height: 20 },
    'top-right': { x: 24, y: 2, width: 14, height: 20 },
    'bottom-left': { x: 2, y: 18, width: 14, height: 20 },
    'bottom-right': { x: 24, y: 18, width: 14, height: 20 },
  }[commentPos] || { x: 24, y: 18, width: 14, height: 20 };

  // セットリストエリアの位置
  const setlistRect = {
    top: { x: 8, y: 2, width: 24, height: 8 },
    bottom: { x: 8, y: 30, width: 24, height: 8 },
    left: { x: 2, y: 10, width: 8, height: 20 },
    right: { x: 30, y: 10, width: 8, height: 20 },
  }[setlistPos] || { x: 8, y: 30, width: 24, height: 8 };

  return (
    <svg viewBox="0 0 40 40" className="w-full h-full">
      {/* 背景 */}
      <rect x="0" y="0" width="40" height="40" fill="#1f2937" rx="4" />
      {/* コメントエリア */}
      <rect
        x={commentRect.x}
        y={commentRect.y}
        width={commentRect.width}
        height={commentRect.height}
        fill="#6366f1"
        rx="2"
        opacity="0.8"
      />
      {/* セットリストエリア */}
      <rect
        x={setlistRect.x}
        y={setlistRect.y}
        width={setlistRect.width}
        height={setlistRect.height}
        fill="#10b981"
        rx="2"
        opacity="0.8"
      />
    </svg>
  );
}

export function LayoutPresetSelector({ selected, onChange }: LayoutPresetSelectorProps) {
  const presetEntries = Object.entries(LAYOUT_PRESETS) as [LayoutPreset, typeof LAYOUT_PRESETS[LayoutPreset]][];

  return (
    <div className="space-y-2">
      <label className="block text-sm font-medium text-gray-700">レイアウトプリセット</label>
      <div className="grid grid-cols-2 gap-3">
        {presetEntries.map(([key, config]) => (
          <button
            key={key}
            type="button"
            onClick={() => onChange(key)}
            className={`relative p-3 rounded-lg border-2 transition-all ${
              selected === key
                ? 'border-blue-600 bg-blue-50'
                : 'border-gray-200 hover:border-gray-300 bg-white'
            }`}
          >
            {/* プレビュー */}
            <div className="w-16 h-16 mx-auto mb-2">
              <PresetPreview preset={key} />
            </div>
            {/* ラベル */}
            <div className="text-center">
              <div className={`text-sm font-medium ${selected === key ? 'text-blue-700' : 'text-gray-700'}`}>
                {config.name}
              </div>
              <div className="text-xs text-gray-500 mt-0.5 line-clamp-2">
                {config.description}
              </div>
            </div>
            {/* 選択インジケーター */}
            {selected === key && (
              <div className="absolute top-1 right-1 w-5 h-5 bg-blue-600 rounded-full flex items-center justify-center">
                <svg className="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                </svg>
              </div>
            )}
          </button>
        ))}
      </div>
      {/* 凡例 */}
      <div className="flex items-center gap-4 text-xs text-gray-500 mt-2">
        <div className="flex items-center gap-1">
          <div className="w-3 h-3 bg-indigo-500 rounded-sm opacity-80" />
          <span>コメント</span>
        </div>
        <div className="flex items-center gap-1">
          <div className="w-3 h-3 bg-emerald-500 rounded-sm opacity-80" />
          <span>セットリスト</span>
        </div>
      </div>
    </div>
  );
}
