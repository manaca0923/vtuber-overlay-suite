import { useEffect, useState, useCallback, useRef } from 'react';
import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

// リトライ設定
const MAX_RETRY_COUNT = 3;
const INITIAL_RETRY_DELAY_MS = 1000;
const MAX_RETRY_DELAY_MS = 30000;

// スキップされたバージョンを保存するlocalStorageキー
const SKIPPED_VERSION_KEY = 'vtuber-overlay-suite-skipped-version';

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
  retryCount: number;
  nextRetryDelay: number;
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
    retryCount: 0,
    nextRetryDelay: INITIAL_RETRY_DELAY_MS,
  });
  const [dismissed, setDismissed] = useState(false);
  // localStorageからスキップされたバージョンを遅延初期化で読み込み
  const [skippedVersion, setSkippedVersion] = useState<string | null>(() => {
    try {
      return localStorage.getItem(SKIPPED_VERSION_KEY);
    } catch {
      // localStorageが使用できない環境では無視
      return null;
    }
  });
  const downloadCancelledRef = useRef(false);

  const checkForUpdates = useCallback(async (isRetry = false) => {
    setState((prev) => ({ ...prev, checking: true, error: null }));
    try {
      const update = await check();
      if (update) {
        setState((prev) => ({
          ...prev,
          checking: false,
          available: true,
          update,
          retryCount: 0,
          nextRetryDelay: INITIAL_RETRY_DELAY_MS,
        }));
      } else {
        setState((prev) => ({
          ...prev,
          checking: false,
          available: false,
          retryCount: 0,
          nextRetryDelay: INITIAL_RETRY_DELAY_MS,
        }));
      }
    } catch (error) {
      console.error('Update check failed:', error);
      setState((prev) => {
        const newRetryCount = isRetry ? prev.retryCount + 1 : 1;
        const canRetry = newRetryCount < MAX_RETRY_COUNT;
        const nextDelay = Math.min(prev.nextRetryDelay * 2, MAX_RETRY_DELAY_MS);

        return {
          ...prev,
          checking: false,
          error: canRetry
            ? `更新確認に失敗しました（リトライ ${newRetryCount}/${MAX_RETRY_COUNT}）`
            : '更新確認に失敗しました（リトライ上限に達しました）',
          retryCount: newRetryCount,
          nextRetryDelay: nextDelay,
        };
      });
    }
  }, []);

  const downloadAndInstall = useCallback(async (isRetry = false) => {
    if (!state.update) return;

    downloadCancelledRef.current = false;
    setState((prev) => ({ ...prev, downloading: true, error: null }));
    try {
      let totalDownloaded = 0;
      let contentLength = 0;
      await state.update.downloadAndInstall((event) => {
        // キャンセルされた場合は処理を中断
        if (downloadCancelledRef.current) {
          throw new Error('ダウンロードがキャンセルされました');
        }

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
              retryCount: 0,
              nextRetryDelay: INITIAL_RETRY_DELAY_MS,
            }));
            break;
        }
      });
    } catch (error) {
      console.error('Update download failed:', error);
      const isCancelled = downloadCancelledRef.current;
      setState((prev) => {
        if (isCancelled) {
          return {
            ...prev,
            downloading: false,
            error: null,
            progress: 0,
            totalDownloaded: 0,
          };
        }

        const newRetryCount = isRetry ? prev.retryCount + 1 : 1;
        const canRetry = newRetryCount < MAX_RETRY_COUNT;
        const nextDelay = Math.min(prev.nextRetryDelay * 2, MAX_RETRY_DELAY_MS);

        return {
          ...prev,
          downloading: false,
          error: canRetry
            ? `ダウンロードに失敗しました（リトライ ${newRetryCount}/${MAX_RETRY_COUNT}）`
            : 'ダウンロードに失敗しました（リトライ上限に達しました）',
          retryCount: newRetryCount,
          nextRetryDelay: nextDelay,
        };
      });
    }
  }, [state.update]);

  const cancelDownload = useCallback(() => {
    downloadCancelledRef.current = true;
    console.log('Download cancelled by user');
  }, []);

  const handleRelaunch = useCallback(async () => {
    try {
      await relaunch();
    } catch (error) {
      console.error('Relaunch failed:', error);
    }
  }, []);

  // このバージョンをスキップ
  // NOTE: versionのみ使用しているが、React Compilerはstate.update全体を依存として推論する
  // state.updateは更新情報が変わったときのみ変化するため、実用上問題なし
  const skipVersion = useCallback(() => {
    if (state.update?.version) {
      try {
        localStorage.setItem(SKIPPED_VERSION_KEY, state.update.version);
        setSkippedVersion(state.update.version);
        setDismissed(true);
        console.log(`Skipping version: ${state.update.version}`);
      } catch {
        // localStorageが使用できない場合は単に閉じる
        setDismissed(true);
      }
    }
  }, [state.update]);

  // スキップをクリア（将来的に設定画面から使用可能）
  const clearSkippedVersion = useCallback(() => {
    try {
      localStorage.removeItem(SKIPPED_VERSION_KEY);
      setSkippedVersion(null);
    } catch {
      // localStorageが使用できない環境では無視
    }
  }, []);

  // 起動時に更新チェック
  // NOTE: コンポーネントマウント時の初期データ取得は正当なパターン
  useEffect(() => {
    // 開発環境ではスキップ
    if (import.meta.env.DEV) {
      console.log('Skipping update check in development mode');
      return;
    }
    // eslint-disable-next-line react-hooks/set-state-in-effect -- 初期マウント時の更新チェック
    checkForUpdates();
  }, [checkForUpdates]);

  // スキップされたバージョンの場合は非表示
  const isVersionSkipped = state.update?.version && skippedVersion === state.update.version;

  // 非表示の場合は何も表示しない
  if (dismissed || isVersionSkipped || (!state.available && !state.error)) {
    return null;
  }

  // リトライ可能かどうか
  const canRetry = state.retryCount < MAX_RETRY_COUNT;

  // clearSkippedVersion を将来使用するために保持（lintエラー回避）
  void clearSkippedVersion;

  return (
    <div className="fixed bottom-4 right-4 z-50 max-w-sm">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-4">
        {/* ヘッダー */}
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-medium text-gray-900 dark:text-white">
            {state.error
              ? '更新エラー'
              : state.downloaded
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
        <div className="flex gap-2 flex-wrap">
          {state.error ? (
            <>
              {canRetry && (
                <button
                  onClick={() =>
                    state.update ? downloadAndInstall(true) : checkForUpdates(true)
                  }
                  className="flex-1 px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
                >
                  再試行
                </button>
              )}
              <button
                onClick={() => setDismissed(true)}
                className="px-3 py-1.5 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
              >
                閉じる
              </button>
            </>
          ) : state.downloaded ? (
            <button
              onClick={handleRelaunch}
              className="flex-1 px-3 py-1.5 text-sm bg-green-500 text-white rounded hover:bg-green-600 transition-colors"
            >
              再起動して更新
            </button>
          ) : state.downloading ? (
            <>
              <div className="flex-1 px-3 py-1.5 text-sm bg-gray-300 text-gray-500 rounded text-center">
                ダウンロード中...
              </div>
              <button
                onClick={cancelDownload}
                className="px-3 py-1.5 text-sm text-red-500 hover:text-red-600 transition-colors"
              >
                キャンセル
              </button>
            </>
          ) : (
            <>
              <button
                onClick={() => downloadAndInstall()}
                className="flex-1 px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
              >
                更新する
              </button>
              <button
                onClick={skipVersion}
                className="px-3 py-1.5 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
                title="このバージョンをスキップ"
              >
                スキップ
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
