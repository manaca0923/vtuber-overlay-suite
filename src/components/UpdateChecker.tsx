import { useEffect, useState, useCallback } from 'react';
import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

interface UpdateState {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  downloaded: boolean;
  progress: number;
  totalDownloaded: number;
  contentLength: number;
  error: string | null;
  update: Update | null;
}

export function UpdateChecker() {
  const [state, setState] = useState<UpdateState>({
    checking: false,
    available: false,
    downloading: false,
    downloaded: false,
    progress: 0,
    totalDownloaded: 0,
    contentLength: 0,
    error: null,
    update: null,
  });
  const [dismissed, setDismissed] = useState(false);

  const checkForUpdates = useCallback(async () => {
    setState((prev) => ({ ...prev, checking: true, error: null }));
    try {
      const update = await check();
      if (update) {
        setState((prev) => ({
          ...prev,
          checking: false,
          available: true,
          update,
        }));
      } else {
        setState((prev) => ({ ...prev, checking: false, available: false }));
      }
    } catch (error) {
      console.error('Update check failed:', error);
      setState((prev) => ({
        ...prev,
        checking: false,
        error: error instanceof Error ? error.message : '更新確認に失敗しました',
      }));
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    if (!state.update) return;

    setState((prev) => ({ ...prev, downloading: true, error: null }));
    try {
      let totalDownloaded = 0;
      let contentLength = 0;
      await state.update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            totalDownloaded = 0;
            contentLength = event.data.contentLength ?? 0;
            setState((prev) => ({ ...prev, progress: 0, totalDownloaded: 0, contentLength }));
            break;
          case 'Progress':
            totalDownloaded += event.data.chunkLength;
            if (contentLength > 0) {
              const progress = Math.round((totalDownloaded / contentLength) * 100);
              setState((prev) => ({ ...prev, progress, totalDownloaded }));
            }
            break;
          case 'Finished':
            setState((prev) => ({
              ...prev,
              downloading: false,
              downloaded: true,
              progress: 100,
            }));
            break;
        }
      });
    } catch (error) {
      console.error('Update download failed:', error);
      setState((prev) => ({
        ...prev,
        downloading: false,
        error: error instanceof Error ? error.message : 'ダウンロードに失敗しました',
      }));
    }
  }, [state.update]);

  const handleRelaunch = useCallback(async () => {
    try {
      await relaunch();
    } catch (error) {
      console.error('Relaunch failed:', error);
    }
  }, []);

  // 起動時に更新チェック
  useEffect(() => {
    // 開発環境ではスキップ
    if (import.meta.env.DEV) {
      console.log('Skipping update check in development mode');
      return;
    }
    checkForUpdates();
  }, [checkForUpdates]);

  // 非表示の場合は何も表示しない
  if (dismissed || (!state.available && !state.error)) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 max-w-sm">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-4">
        {/* ヘッダー */}
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-medium text-gray-900 dark:text-white">
            {state.downloaded
              ? 'アップデート準備完了'
              : state.downloading
                ? 'ダウンロード中...'
                : '新しいバージョンがあります'}
          </h3>
          {!state.downloading && !state.downloaded && (
            <button
              onClick={() => setDismissed(true)}
              className="text-gray-400 hover:text-gray-500"
              aria-label="閉じる"
            >
              <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          )}
        </div>

        {/* バージョン情報 */}
        {state.update && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">
            v{state.update.version}
          </p>
        )}

        {/* プログレスバー */}
        {state.downloading && (
          <div className="mb-3">
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div
                className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                style={{ width: `${state.progress}%` }}
              />
            </div>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              {state.progress}%
            </p>
          </div>
        )}

        {/* エラー表示 */}
        {state.error && (
          <p className="text-xs text-red-500 mb-3">{state.error}</p>
        )}

        {/* アクションボタン */}
        <div className="flex gap-2">
          {state.downloaded ? (
            <button
              onClick={handleRelaunch}
              className="flex-1 px-3 py-1.5 text-sm bg-green-500 text-white rounded hover:bg-green-600 transition-colors"
            >
              再起動して更新
            </button>
          ) : state.downloading ? (
            <button
              disabled
              className="flex-1 px-3 py-1.5 text-sm bg-gray-300 text-gray-500 rounded cursor-not-allowed"
            >
              ダウンロード中...
            </button>
          ) : (
            <>
              <button
                onClick={downloadAndInstall}
                className="flex-1 px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
              >
                更新する
              </button>
              <button
                onClick={() => setDismissed(true)}
                className="px-3 py-1.5 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
              >
                後で
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
