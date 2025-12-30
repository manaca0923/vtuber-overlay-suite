import { useState } from 'react';
import { sendTestComment } from '../types/commands';
import type { TestMessageType } from '../types/commands';
import { SUPERCHAT_PREVIEW_EVENT, SUPERCHAT_REMOVE_PREVIEW_EVENT } from './settings/OverlayPreview';

// プレビュー用スパチャペイロード型
interface PreviewSuperchatPayload {
  id: string;
  authorName: string;
  authorImageUrl: string;
  amount: string;
  message: string;
  tier: number;
}

// 金額とTierの対応（Tier4: ¥1,000-1,999）
const DEFAULT_SUPERCHAT_TIER = 4;
const DEFAULT_SUPERCHAT_AMOUNT = '¥1,000';
const DEFAULT_SUPERCHAT_DISPLAY_DURATION_MS = 60_000; // Tier4: 1分

// プレビュー用スパチャイベントを発火
function dispatchSuperchatPreviewEvent(payload: PreviewSuperchatPayload): void {
  console.log('[TestModeButton] dispatching superchat preview event:', payload);
  window.dispatchEvent(new CustomEvent(SUPERCHAT_PREVIEW_EVENT, { detail: payload }));

  // 表示完了後にremoveイベントを発火
  setTimeout(() => {
    console.log('[TestModeButton] dispatching superchat remove event:', payload.id);
    window.dispatchEvent(new CustomEvent(SUPERCHAT_REMOVE_PREVIEW_EVENT, { detail: { id: payload.id } }));
  }, DEFAULT_SUPERCHAT_DISPLAY_DURATION_MS);
}

const MESSAGE_TYPES: { value: TestMessageType; label: string; color: string }[] = [
  { value: 'text', label: '通常コメント', color: 'bg-gray-100 text-gray-700' },
  { value: 'superChat', label: 'スーパーチャット', color: 'bg-red-100 text-red-700' },
  { value: 'superSticker', label: 'スーパーステッカー', color: 'bg-orange-100 text-orange-700' },
  { value: 'membership', label: 'メンバーシップ', color: 'bg-green-100 text-green-700' },
  { value: 'membershipGift', label: 'メンバーシップギフト', color: 'bg-purple-100 text-purple-700' },
];

const PRESETS = {
  short: { text: 'こんにちは！', author: 'テストユーザー', messageType: 'text' as TestMessageType },
  long: {
    text: 'これは長文コメントのテストです。'.repeat(10),
    author: '長文太郎',
    messageType: 'text' as TestMessageType,
  },
  superchat: {
    text: 'スパチャありがとうございます！',
    author: 'スパチャ太郎',
    messageType: 'superChat' as TestMessageType,
  },
  sticker: {
    text: 'ステッカー送ります！',
    author: 'ステッカー太郎',
    messageType: 'superSticker' as TestMessageType,
  },
  membership: {
    text: 'メンバーになりました！',
    author: 'メンバー太郎',
    messageType: 'membership' as TestMessageType,
  },
  gift: {
    text: 'メンバーシップギフト5件！',
    author: 'ギフト太郎',
    messageType: 'membershipGift' as TestMessageType,
  },
} as const;

export function TestModeButton() {
  const [showDialog, setShowDialog] = useState(false);
  const [commentText, setCommentText] = useState('');
  const [authorName, setAuthorName] = useState('テストユーザー');
  const [messageType, setMessageType] = useState<TestMessageType>('text');
  const [sending, setSending] = useState(false);
  const [message, setMessage] = useState('');

  const handleSend = async () => {
    if (!commentText.trim()) {
      setMessage('コメントを入力してください');
      return;
    }

    setSending(true);
    setMessage('');

    try {
      await sendTestComment(commentText, authorName || 'テストユーザー', messageType);
      const typeLabel = MESSAGE_TYPES.find(t => t.value === messageType)?.label || 'コメント';
      setMessage(`✓ ${typeLabel}を送信しました`);

      // スパチャの場合はプレビューiframeにも通知
      if (messageType === 'superChat') {
        const superchatPayload: PreviewSuperchatPayload = {
          id: `test-superchat-${Date.now()}`,
          authorName: authorName || 'テストユーザー',
          authorImageUrl: '', // テストではアバター無し
          amount: DEFAULT_SUPERCHAT_AMOUNT,
          message: commentText,
          tier: DEFAULT_SUPERCHAT_TIER,
        };
        dispatchSuperchatPreviewEvent(superchatPayload);
      }

      setTimeout(() => {
        setMessage('');
        setCommentText('');
      }, 2000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setMessage(`エラー: ${errorMessage}`);
    } finally {
      setSending(false);
    }
  };

  const handlePreset = (preset: keyof typeof PRESETS) => {
    const { text, author, messageType: presetType } = PRESETS[preset];
    setCommentText(text);
    setAuthorName(author);
    setMessageType(presetType);
  };

  return (
    <>
      <button
        onClick={() => setShowDialog(true)}
        className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors font-medium"
      >
        🧪 テストモード
      </button>

      {showDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-2xl w-full p-6">
            <h2 className="text-2xl font-bold mb-4">テストモード</h2>
            <p className="text-gray-600 mb-6">
              ダミーコメントを送信して、オーバーレイの動作を確認できます。
            </p>

            <div className="space-y-4">
              {/* プリセットボタン */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  プリセット
                </label>
                <div className="flex flex-wrap gap-2">
                  <button
                    onClick={() => handlePreset('short')}
                    className="px-3 py-1.5 bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors text-sm"
                  >
                    通常
                  </button>
                  <button
                    onClick={() => handlePreset('long')}
                    className="px-3 py-1.5 bg-purple-100 text-purple-700 rounded hover:bg-purple-200 transition-colors text-sm"
                  >
                    長文
                  </button>
                  <button
                    onClick={() => handlePreset('superchat')}
                    className="px-3 py-1.5 bg-red-100 text-red-700 rounded hover:bg-red-200 transition-colors text-sm"
                  >
                    スパチャ
                  </button>
                  <button
                    onClick={() => handlePreset('sticker')}
                    className="px-3 py-1.5 bg-orange-100 text-orange-700 rounded hover:bg-orange-200 transition-colors text-sm"
                  >
                    ステッカー
                  </button>
                  <button
                    onClick={() => handlePreset('membership')}
                    className="px-3 py-1.5 bg-green-100 text-green-700 rounded hover:bg-green-200 transition-colors text-sm"
                  >
                    メンバー
                  </button>
                  <button
                    onClick={() => handlePreset('gift')}
                    className="px-3 py-1.5 bg-violet-100 text-violet-700 rounded hover:bg-violet-200 transition-colors text-sm"
                  >
                    ギフト
                  </button>
                </div>
              </div>

              {/* メッセージタイプ選択 */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  メッセージタイプ
                </label>
                <div className="flex flex-wrap gap-2">
                  {MESSAGE_TYPES.map((type) => (
                    <button
                      key={type.value}
                      onClick={() => setMessageType(type.value)}
                      className={`px-3 py-1.5 rounded text-sm transition-colors ${
                        messageType === type.value
                          ? `${type.color} ring-2 ring-offset-1 ring-gray-400 font-medium`
                          : `${type.color} opacity-60 hover:opacity-100`
                      }`}
                    >
                      {type.label}
                    </button>
                  ))}
                </div>
              </div>

              {/* 名前入力 */}
              <div>
                <label htmlFor="authorName" className="block text-sm font-medium text-gray-700 mb-2">
                  投稿者名
                </label>
                <input
                  id="authorName"
                  type="text"
                  value={authorName}
                  onChange={(e) => setAuthorName(e.target.value)}
                  placeholder="テストユーザー"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-gray-900 placeholder:text-gray-400"
                  disabled={sending}
                />
              </div>

              {/* コメント入力 */}
              <div>
                <label htmlFor="commentText" className="block text-sm font-medium text-gray-700 mb-2">
                  コメント
                </label>
                <textarea
                  id="commentText"
                  value={commentText}
                  onChange={(e) => setCommentText(e.target.value)}
                  placeholder="コメントを入力してください"
                  rows={4}
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                  disabled={sending}
                />
              </div>

              {/* メッセージ表示 */}
              {message && (
                <div
                  className={`p-3 rounded-lg text-sm ${
                    message.startsWith('✓')
                      ? 'bg-green-50 border border-green-200 text-green-700'
                      : 'bg-red-50 border border-red-200 text-red-700'
                  }`}
                >
                  {message}
                </div>
              )}

              {/* ボタン */}
              <div className="flex gap-3">
                <button
                  onClick={handleSend}
                  disabled={sending || !commentText.trim()}
                  className={`flex-1 px-6 py-2 rounded-lg font-medium transition-colors ${
                    sending || !commentText.trim()
                      ? 'bg-gray-400 text-white cursor-not-allowed'
                      : 'bg-blue-600 text-white hover:bg-blue-700'
                  }`}
                >
                  {sending ? '送信中...' : 'コメントを送信'}
                </button>
                <button
                  onClick={() => {
                    setShowDialog(false);
                    setMessage('');
                  }}
                  disabled={sending}
                  className="px-6 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition-colors font-medium"
                >
                  閉じる
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
