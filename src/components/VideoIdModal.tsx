import { useState, useCallback, useEffect, useRef, type FormEvent, type KeyboardEvent } from 'react';

interface VideoIdModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (videoId: string) => void;
  defaultValue?: string;
}

export function VideoIdModal({ isOpen, onClose, onSubmit, defaultValue = '' }: VideoIdModalProps) {
  const [videoId, setVideoId] = useState(defaultValue);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // モーダルが開いたときにフォームをリセットしてフォーカス
  useEffect(() => {
    if (isOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- モーダル開閉時のフォームリセット
      setVideoId(defaultValue);
      setError(null);
      // 少し遅延させてフォーカス（アニメーション考慮）
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [isOpen, defaultValue]);

  // URLからVideo IDを抽出
  const extractVideoId = useCallback((input: string): string | null => {
    const trimmed = input.trim();

    // 既にVideo IDの形式（11文字の英数字+ハイフン+アンダースコア）
    if (/^[a-zA-Z0-9_-]{11}$/.test(trimmed)) {
      return trimmed;
    }

    // YouTube URLパターン
    const patterns = [
      /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/live\/)([a-zA-Z0-9_-]{11})/,
      /youtube\.com\/embed\/([a-zA-Z0-9_-]{11})/,
    ];

    for (const pattern of patterns) {
      const match = trimmed.match(pattern);
      if (match && match[1]) {
        return match[1];
      }
    }

    return null;
  }, []);

  const handleSubmit = useCallback((e: FormEvent) => {
    e.preventDefault();

    const extracted = extractVideoId(videoId);
    if (!extracted) {
      setError('有効なVideo IDまたはYouTube URLを入力してください');
      return;
    }

    onSubmit(extracted);
    onClose();
  }, [videoId, extractVideoId, onSubmit, onClose]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
    }
  }, [onClose]);

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={(e) => e.target === e.currentTarget && onClose()}
      onKeyDown={handleKeyDown}
    >
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md mx-4 p-6">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Video IDを入力
        </h2>

        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label
              htmlFor="videoId"
              className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
            >
              Video ID または YouTube URL
            </label>
            <input
              ref={inputRef}
              type="text"
              id="videoId"
              value={videoId}
              onChange={(e) => {
                setVideoId(e.target.value);
                setError(null);
              }}
              placeholder="例: dQw4w9WgXcQ または https://youtube.com/watch?v=..."
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md
                         bg-white dark:bg-gray-700 text-gray-900 dark:text-white
                         focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                         placeholder:text-gray-400 dark:placeholder:text-gray-500"
            />
            {error && (
              <p className="mt-1 text-sm text-red-500">{error}</p>
            )}
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              ライブ配信のURLまたはVideo IDを入力してください
            </p>
          </div>

          <div className="flex justify-end gap-3">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300
                         hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md transition-colors"
            >
              キャンセル
            </button>
            <button
              type="submit"
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600
                         hover:bg-blue-700 rounded-md transition-colors"
            >
              開始
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
