import { useState, useEffect, useCallback, useRef, type ChangeEvent, type KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** 表示間隔保存のdebounce間隔（ミリ秒） */
const SETTINGS_SAVE_DEBOUNCE_MS = 500;

interface PromoItem {
  text: string;
  icon: string | null;
}

interface PromoState {
  items: PromoItem[];
  cycleSec: number | null;
  showSec: number | null;
}

export function PromoSettingsPanel() {
  const [promoState, setPromoState] = useState<PromoState>({ items: [], cycleSec: null, showSec: null });
  // 新規アイテム入力用のローカル状態
  const [newItemText, setNewItemText] = useState('');
  const [newItemIcon, setNewItemIcon] = useState('');
  // 表示間隔入力用のローカル状態（debounce用）
  const [localShowSec, setLocalShowSec] = useState<number>(6);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  // 編集中のアイテムインデックス
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editText, setEditText] = useState('');
  const [editIcon, setEditIcon] = useState('');
  const settingsSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // 設定保存のシーケンス番号（非同期競合対策）
  const settingsSaveSeqRef = useRef(0);

  // 告知状態を読み込み
  const loadPromoState = useCallback(async () => {
    try {
      const state = await invoke<PromoState>('get_promo_state');
      setPromoState(state);
      setLocalShowSec(state.showSec ?? 6);
    } catch (err) {
      console.error('Failed to load promo state:', err);
      setError('告知状態の読み込みに失敗しました');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPromoState();
  }, [loadPromoState]);

  // タイマークリーンアップ
  useEffect(() => {
    return () => {
      if (settingsSaveTimerRef.current) {
        clearTimeout(settingsSaveTimerRef.current);
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
      const updated = await invoke<PromoState>('add_promo_item', {
        text: newItemText.trim(),
        icon: newItemIcon.trim() || null,
      });
      setPromoState(updated);
      setNewItemText('');
      setNewItemIcon('');
      // ブロードキャスト
      await invoke('broadcast_promo_update', { promo_state: updated });
      setSuccess('告知を追加しました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to add promo item:', err);
      setError('告知の追加に失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // アイテムを削除
  const handleRemoveItem = async (index: number) => {
    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<PromoState>('remove_promo_item', { index });
      setPromoState(updated);
      // ブロードキャスト
      await invoke('broadcast_promo_update', { promo_state: updated });
      setSuccess('告知を削除しました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to remove promo item:', err);
      setError('告知の削除に失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // 編集開始
  const handleStartEdit = (index: number) => {
    const item = promoState.items[index];
    if (!item) return;
    setEditingIndex(index);
    setEditText(item.text);
    setEditIcon(item.icon ?? '');
  };

  // 編集キャンセル
  const handleCancelEdit = () => {
    setEditingIndex(null);
    setEditText('');
    setEditIcon('');
  };

  // 編集保存
  const handleSaveEdit = async () => {
    if (editingIndex === null || !editText.trim()) return;

    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<PromoState>('update_promo_item', {
        index: editingIndex,
        text: editText.trim(),
        icon: editIcon.trim() || null,
      });
      setPromoState(updated);
      setEditingIndex(null);
      setEditText('');
      setEditIcon('');
      // ブロードキャスト
      await invoke('broadcast_promo_update', { promo_state: updated });
      setSuccess('告知を更新しました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to update promo item:', err);
      setError('告知の更新に失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // 告知をクリア
  const handleClearPromo = async () => {
    if (promoState.items.length === 0) return;

    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<PromoState>('clear_promo');
      setPromoState(updated);
      // ブロードキャスト
      await invoke('broadcast_promo_update', { promo_state: updated });
      setSuccess('告知をクリアしました');
      setTimeout(() => setSuccess(''), 2000);
    } catch (err) {
      console.error('Failed to clear promo:', err);
      setError('告知のクリアに失敗しました');
    } finally {
      setSaving(false);
    }
  };

  // 表示間隔の変更（ローカル状態のみ更新、debounceで保存）
  const handleShowSecChange = (e: ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value, 10);
    if (isNaN(value)) return;

    setLocalShowSec(value);

    // 既存のタイマーをクリア
    if (settingsSaveTimerRef.current) {
      clearTimeout(settingsSaveTimerRef.current);
    }

    // debounce後に保存
    settingsSaveTimerRef.current = setTimeout(() => {
      handleSettingsSave(value);
    }, SETTINGS_SAVE_DEBOUNCE_MS);
  };

  // 設定を実際に保存
  const handleSettingsSave = async (showSec: number) => {
    // シーケンス番号をインクリメント
    settingsSaveSeqRef.current += 1;
    const currentSeq = settingsSaveSeqRef.current;

    clearMessages();
    setSaving(true);
    try {
      const updated = await invoke<PromoState>('set_promo_settings', { show_sec: showSec });

      // 最新のリクエストのみ状態を更新
      if (currentSeq === settingsSaveSeqRef.current) {
        setPromoState(updated);
        // ブロードキャスト
        await invoke('broadcast_promo_update', { promo_state: updated });
      }
    } catch (err) {
      // 最新のリクエストのみエラーを表示
      if (currentSeq === settingsSaveSeqRef.current) {
        console.error('Failed to save promo settings:', err);
        setError('設定の保存に失敗しました');
      }
    } finally {
      // 最新のリクエストのみsaving状態を解除
      if (currentSeq === settingsSaveSeqRef.current) {
        setSaving(false);
      }
    }
  };

  // キーボードイベント（新規追加）
  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
      handleAddItem();
    }
  };

  // キーボードイベント（編集）
  const handleEditKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
      handleSaveEdit();
    } else if (e.key === 'Escape') {
      handleCancelEdit();
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
        <h3 className="text-lg font-medium text-gray-900 mb-2">告知管理</h3>
        <p className="text-sm text-gray-500">
          配信中に表示する告知メッセージを管理できます。複数の告知がサイクル表示されます。
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

      {/* 表示間隔設定 */}
      <div>
        <label htmlFor="show-sec" className="block text-sm font-medium text-gray-700 mb-1">
          表示間隔（秒）
        </label>
        <div className="flex items-center gap-3">
          <input
            id="show-sec"
            type="range"
            min={3}
            max={15}
            value={localShowSec}
            onChange={handleShowSecChange}
            className="flex-1"
          />
          <span className="w-12 text-center text-gray-700 font-medium">{localShowSec}秒</span>
        </div>
        <p className="text-xs text-gray-500 mt-1">各告知の表示時間（3〜15秒）</p>
      </div>

      {/* 新規追加フォーム */}
      <div className="space-y-3">
        <label className="block text-sm font-medium text-gray-700">
          新しい告知
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            value={newItemIcon}
            onChange={(e) => setNewItemIcon(e.target.value)}
            placeholder="アイコン"
            disabled={saving}
            className="w-16 px-3 py-2 border border-gray-300 rounded-md text-gray-900 placeholder:text-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100 text-center"
            title="絵文字やアイコン文字を入力（省略可）"
          />
          <input
            type="text"
            value={newItemText}
            onChange={(e) => setNewItemText(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="告知テキストを入力してEnter"
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

      {/* 告知リスト */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <label className="block text-sm font-medium text-gray-700">
            告知リスト ({promoState.items.length}件)
          </label>
          {promoState.items.length > 0 && (
            <button
              type="button"
              onClick={handleClearPromo}
              disabled={saving}
              className="text-sm text-red-600 hover:text-red-700 disabled:text-gray-400"
            >
              すべてクリア
            </button>
          )}
        </div>
        {promoState.items.length === 0 ? (
          <div className="text-center py-8 text-gray-500 bg-gray-50 rounded-lg border border-gray-200">
            告知はありません
          </div>
        ) : (
          <ul className="space-y-2">
            {promoState.items.map((item, index) => (
              <li
                key={index}
                className="flex items-center justify-between px-4 py-3 bg-white border border-gray-200 rounded-lg"
              >
                {editingIndex === index ? (
                  // 編集モード
                  <div className="flex-1 flex items-center gap-2">
                    <input
                      type="text"
                      value={editIcon}
                      onChange={(e) => setEditIcon(e.target.value)}
                      onKeyDown={handleEditKeyDown}
                      placeholder="アイコン"
                      className="w-16 px-2 py-1 border border-gray-300 rounded text-center text-gray-900"
                    />
                    <input
                      type="text"
                      value={editText}
                      onChange={(e) => setEditText(e.target.value)}
                      onKeyDown={handleEditKeyDown}
                      className="flex-1 px-2 py-1 border border-gray-300 rounded text-gray-900"
                      autoFocus
                    />
                    <button
                      type="button"
                      onClick={handleSaveEdit}
                      disabled={saving || !editText.trim()}
                      className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:bg-gray-400"
                    >
                      保存
                    </button>
                    <button
                      type="button"
                      onClick={handleCancelEdit}
                      className="px-3 py-1 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300"
                    >
                      取消
                    </button>
                  </div>
                ) : (
                  // 表示モード
                  <>
                    <span className="flex items-center gap-3">
                      <span className="w-6 h-6 flex items-center justify-center bg-purple-100 text-purple-700 text-xs font-medium rounded-full">
                        {index + 1}
                      </span>
                      {item.icon && (
                        <span className="text-lg" title="アイコン">{item.icon}</span>
                      )}
                      <span className="text-gray-900">{item.text}</span>
                    </span>
                    <div className="flex items-center gap-2">
                      <button
                        type="button"
                        onClick={() => handleStartEdit(index)}
                        disabled={saving}
                        className="text-gray-400 hover:text-blue-600 disabled:text-gray-300 transition-colors"
                        title="編集"
                      >
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                          />
                        </svg>
                      </button>
                      <button
                        type="button"
                        onClick={() => handleRemoveItem(index)}
                        disabled={saving}
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
                    </div>
                  </>
                )}
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* ヒント */}
      <div className="p-4 bg-purple-50 border border-purple-200 rounded-lg">
        <h4 className="text-sm font-medium text-purple-900 mb-1">使い方のヒント</h4>
        <ul className="text-sm text-purple-800 space-y-1">
          <li>• 告知は設定した間隔で順番に切り替わります</li>
          <li>• アイコンには絵文字を使用できます（例: {'\uD83C\uDF89'} {'\uD83D\uDCE2'} {'\u2B50'}）</li>
          <li>• 表示ON/OFFは「ウィジェット」タブで設定できます</li>
        </ul>
      </div>
    </div>
  );
}
