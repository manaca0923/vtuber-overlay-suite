import { useState } from 'react';
import type { CommentSettings, CommentPosition } from '../../types/overlaySettings';
import { sendTestComment } from '../../types/commands';

interface CommentSettingsPanelProps {
  settings: CommentSettings;
  onChange: (settings: CommentSettings) => void;
}

const POSITION_OPTIONS: { value: CommentPosition; label: string }[] = [
  { value: 'top-left', label: '左上' },
  { value: 'top-right', label: '右上' },
  { value: 'bottom-left', label: '左下' },
  { value: 'bottom-right', label: '右下' },
];

export function CommentSettingsPanel({ settings, onChange }: CommentSettingsPanelProps) {
  const [sending, setSending] = useState(false);
  const [testMessage, setTestMessage] = useState('');

  const updateSettings = (updates: Partial<CommentSettings>) => {
    onChange({ ...settings, ...updates });
  };

  // テストコメントを送信
  const handleTestSend = async () => {
    setSending(true);
    setTestMessage('');

    try {
      await sendTestComment('テストコメントです！', 'テストユーザー', 'text');
      setTestMessage('✓ コメントを送信しました');
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

      {/* 位置選択 */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">表示位置</label>
        <div className="grid grid-cols-2 gap-2">
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
          min={12}
          max={24}
          value={settings.fontSize}
          onChange={(e) => updateSettings({ fontSize: parseInt(e.target.value, 10) })}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>12px</span>
          <span>24px</span>
        </div>
      </div>

      {/* アイコン表示 */}
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-gray-700">アイコン表示</label>
        <button
          type="button"
          onClick={() => updateSettings({ showAvatar: !settings.showAvatar })}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
            settings.showAvatar ? 'bg-blue-600' : 'bg-gray-300'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
              settings.showAvatar ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>

      {/* テスト送信 */}
      <div className="border-t pt-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          テスト送信
        </label>
        <button
          type="button"
          onClick={handleTestSend}
          disabled={sending}
          className={`px-3 py-1.5 text-sm rounded-lg border transition-colors ${
            sending
              ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
              : 'bg-gray-50 border-gray-300 text-gray-700 hover:bg-gray-100'
          }`}
        >
          通常コメント
        </button>
        {testMessage && (
          <p className={`text-xs mt-2 ${testMessage.startsWith('✓') ? 'text-green-600' : 'text-red-600'}`}>
            {testMessage}
          </p>
        )}
      </div>
    </div>
  );
}
