import { useState, useEffect, useCallback, useRef, type ChangeEvent, type KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** タイトル保存のdebounce間隔（ミリ秒） */
const TITLE_SAVE_DEBOUNCE_MS = 500;

interface QueueItem {
  id: string | null;
  text: string;
}

interface QueueState {
  title: string | null;
  items: QueueItem[];
}

export function QueueSettingsPanel() {
  const [queueState, setQueueState] = useState<QueueState>({ title: null, items: [] });
  // タイトル入力用のローカル状態（debounce用）
  const [localTitle, setLocalTitle] = useState<string>('');
  const [newItemText, setNewItemText] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const titleSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // タイトル保存のシーケンス番号（非同期競合対策）
  const titleSaveSeqRef = useRef(0);

  // キュー状態を読み込み
  const loadQueueState = useCallback(async () => {
    try {
      const state = await invoke<QueueState>('get_queue_state');
      setQueueState(state);
      setLocalTitle(state.title ?? '');
    } catch (err) {
      console.error('Failed to load queue state:', err);
      setError('キュー状態の読み込みに失敗しました');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadQueueState();
  }, [loadQueueState]);

  // タイマークリーンアップ
  useEffect(() => {
    return () => {
      if (titleSaveTimerRef.current) {
        clearTimeout(titleSaveTimerRef.current);
      }
    };
  }, []);

  // ステータスメッセージのクリア
  const clearMessages = useCallback(() => {
    setError('');
    setSuccess('');
  }, []);

  // アイテムを追加
  const handleAddItem = async () => {
    if (!newItemText.trim()) return;

    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<QueueState>('add_queue_item', { text: newItemText.trim() });
      setQueueState(updated);
      setNewItemText('');
      // ブロードキャスト
      await invoke('broadcast_queue_update', { queue_state: updated });
      setSuccess('アイテムを追加しました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to add queue item:', err);
      setError('アイテムの追加に失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // アイテムを削除
  const handleRemoveItem = async (id: string) => {
    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<QueueState>('remove_queue_item', { id });
      setQueueState(updated);
      // ブロードキャスト
      await invoke('broadcast_queue_update', { queue_state: updated });
      setSuccess('アイテムを削除しました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to remove queue item:', err);
      setError('アイテムの削除に失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // キューをクリア
  const handleClearQueue = async () => {
    if (queueState.items.length === 0) return;

    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<QueueState>('clear_queue');
      setQueueState(updated);
      // ブロードキャスト
      await invoke('broadcast_queue_update', { queue_state: updated });
      setSuccess('キューをクリアしました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to clear queue:', err);
      setError('キューのクリアに失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // タイトル入力の変更（ローカル状態のみ更新、debounceで保存）
  const handleLocalTitleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setLocalTitle(value);

    // 既存のタイマーをクリア
    if (titleSaveTimerRef.current) {
      clearTimeout(titleSaveTimerRef.current);
    }

    // debounce後に保存
    titleSaveTimerRef.current = setTimeout(() => {
      handleTitleSave(value);
    }, TITLE_SAVE_DEBOUNCE_MS);
  };

  // タイトルを実際に保存
  // NOTE: 非同期競合対策としてシーケンス番号を使用。
  //       古いリクエストの完了が最新のタイトルを上書きしないようにガード。
  const handleTitleSave = async (title: string) => {
    const newTitle = title.trim() || null;
    // シーケンス番号をインクリメント
    titleSaveSeqRef.current += 1;
    const currentSeq = titleSaveSeqRef.current;

    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<QueueState>('set_queue_title', { title: newTitle });

      // 最新のリクエストのみ状態を更新
      if (currentSeq === titleSaveSeqRef.current) {
        setQueueState(updated);
        // ブロードキャスト
        await invoke('broadcast_queue_update', { queue_state: updated });
      }
    } catch (err) {
      // 最新のリクエストのみエラーを表示
      if (currentSeq === titleSaveSeqRef.current) {
        console.error('Failed to set queue title:', err);
        setError('タイトルの変更に失敗しました');
      }
    } finally {
      // 最新のリクエストのみsaving状態を解除
      if (currentSeq === titleSaveSeqRef.current) {
        setSaving(false);
      }
    }
  };

  // キーボードイベント
  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
      handleAddItem();
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600 text-sm">読み込み中...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-2">短冊（キュー）管理</h3>
        <p className="text-sm text-gray-500">
          リクエスト曲の待ち行列や参加者リストなどを管理できます。
        </p>
      </div>

      {/* エラー・成功メッセージ */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {error}
        </div>
      )}
      {success && (
        <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
          {success}
        </div>
      )}

      {/* タイトル設定 */}
      <div>
        <label htmlFor="queue-title" className="block text-sm font-medium text-gray-700 mb-1">
          キュータイトル
        </label>
        <input
          id="queue-title"
          type="text"
          value={localTitle}
          onChange={handleLocalTitleChange}
          placeholder="例: リクエスト曲, 待機リスト"
          className="w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 placeholder:text-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        />
      </div>

      {/* アイテム追加 */}
      <div>
        <label htmlFor="new-item" className="block text-sm font-medium text-gray-700 mb-1">
          新しいアイテム
        </label>
        <div className="flex gap-2">
          <input
            id="new-item"
            type="text"
            value={newItemText}
            onChange={(e) => setNewItemText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="テキストを入力してEnter"
            disabled={saving}
            className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-gray-900 placeholder:text-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
          />
          <button
            type="button"
            onClick={handleAddItem}
            disabled={saving || !newItemText.trim()}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
          >
            追加
          </button>
        </div>
      </div>

      {/* キューリスト */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <label className="block text-sm font-medium text-gray-700">
            キュー ({queueState.items.length}件)
          </label>
          {queueState.items.length > 0 && (
            <button
              type="button"
              onClick={handleClearQueue}
              disabled={saving}
              className="text-sm text-red-600 hover:text-red-700 disabled:text-gray-400"
            >
              すべてクリア
            </button>
          )}
        </div>
        {queueState.items.length === 0 ? (
          <div className="text-center py-8 text-gray-500 bg-gray-50 rounded-lg border border-gray-200">
            キューは空です
          </div>
        ) : (
          <ul className="space-y-2">
            {queueState.items.map((item, index) => (
              <li
                key={item.id ?? index}
                className="flex items-center justify-between px-4 py-3 bg-white border border-gray-200 rounded-lg"
              >
                <span className="flex items-center gap-3">
                  <span className="w-6 h-6 flex items-center justify-center bg-blue-100 text-blue-700 text-xs font-medium rounded-full">
                    {index + 1}
                  </span>
                  <span className="text-gray-900">{item.text}</span>
                </span>
                <button
                  type="button"
                  onClick={() => item.id && handleRemoveItem(item.id)}
                  disabled={saving || !item.id}
                  className="text-gray-400 hover:text-red-600 disabled:text-gray-300 transition-colors"
                  title="削除"
                >
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* ヒント */}
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h4 className="text-sm font-medium text-blue-900 mb-1">使い方のヒント</h4>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• キューに追加したアイテムはオーバーレイにリアルタイムで反映されます</li>
          <li>• 最大6件まで表示されます（それ以上は非表示）</li>
          <li>• 表示ON/OFFは「ウィジェット」タブで設定できます</li>
        </ul>
      </div>
    </div>
  );
}
