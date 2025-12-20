import { useState } from 'react';
import { sendTestComment } from '../types/commands';

const PRESETS = {
  short: { text: 'こんにちは！', author: 'テストユーザー' },
  long: {
    text: 'これは長文コメントのテストです。'.repeat(10),
    author: '長文太郎',
  },
  superchat: {
    text: 'スパチャありがとうございます！',
    author: 'スパチャ太郎',
  },
};

export function TestModeButton() {
  const [showDialog, setShowDialog] = useState(false);
  const [commentText, setCommentText] = useState('');
  const [authorName, setAuthorName] = useState('テストユーザー');
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
      await sendTestComment(commentText, authorName || 'テストユーザー');
      setMessage('✓ テストコメントを送信しました');
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
    const { text, author } = PRESETS[preset];
    setCommentText(text);
    setAuthorName(author);
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
                <div className="flex gap-2">
                  <button
                    onClick={() => handlePreset('short')}
                    className="px-4 py-2 bg-blue-100 text-blue-700 rounded hover:bg-blue-200 transition-colors text-sm"
                  >
                    通常コメント
                  </button>
                  <button
                    onClick={() => handlePreset('long')}
                    className="px-4 py-2 bg-purple-100 text-purple-700 rounded hover:bg-purple-200 transition-colors text-sm"
                  >
                    長文テスト
                  </button>
                  <button
                    onClick={() => handlePreset('superchat')}
                    className="px-4 py-2 bg-yellow-100 text-yellow-700 rounded hover:bg-yellow-200 transition-colors text-sm"
                  >
                    スーパーチャット風
                  </button>
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
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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
