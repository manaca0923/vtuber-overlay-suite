import { useState } from 'react';
import type { SuperchatSettings } from '../../types/overlaySettings';
import { sendTestComment } from '../../types/commands';
import { SUPERCHAT_PREVIEW_EVENT, SUPERCHAT_REMOVE_PREVIEW_EVENT } from './OverlayPreview';

interface SuperchatSettingsPanelProps {
  settings: SuperchatSettings;
  onChange: (settings: SuperchatSettings) => void;
}

// スパチャプレビュー用ペイロード
interface PreviewSuperchatPayload {
  id: string;
  authorName: string;
  authorImageUrl: string;
  amount: string;
  message: string;
  tier: number;
}

// テスト送信用のプリセット（YouTube公式のスパチャTier）
const TEST_PRESETS = [
  { label: '¥100', amount: '¥100', tier: 1, color: 'blue' },      // Blue: ¥100-199
  { label: '¥200', amount: '¥200', tier: 2, color: 'cyan' },      // Cyan: ¥200-499
  { label: '¥500', amount: '¥500', tier: 3, color: 'teal' },      // Teal: ¥500-999
  { label: '¥1,000', amount: '¥1,000', tier: 4, color: 'yellow' }, // Yellow: ¥1,000-1,999
  { label: '¥2,000', amount: '¥2,000', tier: 5, color: 'orange' }, // Orange: ¥2,000-4,999
  { label: '¥5,000', amount: '¥5,000', tier: 6, color: 'pink' },   // Pink: ¥5,000-9,999
  { label: '¥10,000', amount: '¥10,000', tier: 7, color: 'red' },  // Red: ¥10,000+
] as const;

// Tier別のボタンスタイル
const TIER_BUTTON_STYLES: Record<string, string> = {
  blue: 'bg-blue-100 border-blue-400 text-blue-800 hover:bg-blue-200',
  cyan: 'bg-cyan-100 border-cyan-400 text-cyan-800 hover:bg-cyan-200',
  teal: 'bg-teal-100 border-teal-400 text-teal-800 hover:bg-teal-200',
  yellow: 'bg-yellow-100 border-yellow-400 text-yellow-800 hover:bg-yellow-200',
  orange: 'bg-orange-100 border-orange-400 text-orange-800 hover:bg-orange-200',
  pink: 'bg-pink-100 border-pink-400 text-pink-800 hover:bg-pink-200',
  red: 'bg-red-100 border-red-400 text-red-800 hover:bg-red-200',
};

export function SuperchatSettingsPanel({ settings, onChange }: SuperchatSettingsPanelProps) {
  const [sending, setSending] = useState(false);
  const [testMessage, setTestMessage] = useState('');

  const updateSettings = (updates: Partial<SuperchatSettings>) => {
    onChange({ ...settings, ...updates });
  };

  // テストスパチャを送信
  const handleTestSend = async (preset: typeof TEST_PRESETS[number]) => {
    setSending(true);
    setTestMessage('');

    try {
      const testText = 'スパチャありがとうございます！';
      const testAuthor = 'スパチャ太郎';

      // WebSocket経由で送信（本体オーバーレイ用）
      await sendTestComment(testText, testAuthor, 'superChat', preset.amount);

      // プレビューiframeにも通知
      const superchatPayload: PreviewSuperchatPayload = {
        id: `test-superchat-${Date.now()}`,
        authorName: testAuthor,
        authorImageUrl: '',
        amount: preset.amount,
        message: testText,
        tier: preset.tier,
      };

      // カスタムイベントを発火
      window.dispatchEvent(new CustomEvent(SUPERCHAT_PREVIEW_EVENT, { detail: superchatPayload }));

      // 設定された表示時間後にremoveイベント
      const displayDurationMs = settings.displayDurationSec * 1000;
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent(SUPERCHAT_REMOVE_PREVIEW_EVENT, { detail: { id: superchatPayload.id } }));
      }, displayDurationMs);

      setTestMessage(`✓ ${preset.label}を送信しました`);
      setTimeout(() => setTestMessage(''), 2000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setTestMessage(`エラー: ${errorMessage}`);
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="space-y-6">
      <p className="text-sm text-gray-500">
        表示ON/OFFは「ウィジェット」タブで設定できます
      </p>

      {/* 同時表示数 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          同時表示数: {settings.maxDisplay}件
        </label>
        <input
          type="range"
          min={1}
          max={3}
          value={settings.maxDisplay}
          onChange={(e) => updateSettings({ maxDisplay: parseInt(e.target.value, 10) })}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>1件</span>
          <span>3件</span>
        </div>
      </div>

      {/* 表示時間 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          表示時間: {settings.displayDurationSec}秒
        </label>
        <input
          type="range"
          min={10}
          max={120}
          step={5}
          value={settings.displayDurationSec}
          onChange={(e) => updateSettings({ displayDurationSec: parseInt(e.target.value, 10) })}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>10秒</span>
          <span>120秒</span>
        </div>
      </div>

      {/* テスト送信 */}
      <div className="border-t pt-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          テスト送信
        </label>
        <div className="grid grid-cols-4 gap-2">
          {TEST_PRESETS.map((preset) => (
            <button
              key={preset.tier}
              type="button"
              onClick={() => handleTestSend(preset)}
              disabled={sending}
              className={`px-2 py-1.5 text-xs font-medium rounded-lg border transition-colors ${
                sending
                  ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                  : TIER_BUTTON_STYLES[preset.color]
              }`}
            >
              {preset.label}
            </button>
          ))}
        </div>
        {testMessage && (
          <p className={`text-xs mt-2 ${testMessage.startsWith('✓') ? 'text-green-600' : 'text-red-600'}`}>
            {testMessage}
          </p>
        )}
      </div>
    </div>
  );
}
