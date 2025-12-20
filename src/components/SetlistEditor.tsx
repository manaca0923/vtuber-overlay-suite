import { useEffect, useState, useCallback } from 'react';
import { getSetlistWithSongs, removeSongFromSetlist, getSongs, addSongToSetlist, setCurrentSong, nextSong, previousSong, reorderSetlistSongs, broadcastSetlistUpdate } from '../types/commands';
import type { SetlistWithSongs, SetlistSong } from '../types/setlist';
import type { Song } from '../types/song';
import { parseTags } from '../types/song';
import { TimestampExporter } from './TimestampExporter';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

interface SetlistEditorProps {
  setlistId: string;
  onClose: () => void;
}

export function SetlistEditor({ setlistId, onClose }: SetlistEditorProps) {
  const [setlistData, setSetlistData] = useState<SetlistWithSongs | null>(null);
  const [allSongs, setAllSongs] = useState<Song[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [showAddSong, setShowAddSong] = useState(false);
  const [isReordering, setIsReordering] = useState(false);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      const [setlist, songs] = await Promise.all([
        getSetlistWithSongs(setlistId),
        getSongs(),
      ]);
      setSetlistData(setlist);
      setAllSongs(songs);
      setError('');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [setlistId]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleRemoveSong = async (setlistSongId: string) => {
    if (!confirm('この曲をセットリストから削除しますか？')) return;

    setError('');
    try {
      await removeSongFromSetlist(setlistId, setlistSongId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleAddSong = async (songId: string) => {
    setError('');
    try {
      await addSongToSetlist(setlistId, songId);
      await loadData();
      setShowAddSong(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSetCurrent = async (position: number) => {
    setError('');
    try {
      await setCurrentSong(setlistId, position);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleNext = async () => {
    setError('');
    try {
      await nextSong(setlistId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handlePrevious = async () => {
    setError('');
    try {
      await previousSong(setlistId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleBroadcast = async () => {
    setError('');
    try {
      await broadcastSetlistUpdate(setlistId);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  // ドラッグ&ドロップ設定
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // 8px動かしてからドラッグ開始（誤操作防止）
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;

    if (!over || active.id === over.id || !setlistData || isReordering) {
      return;
    }

    const oldIndex = setlistData.songs.findIndex((song) => song.id === active.id);
    const newIndex = setlistData.songs.findIndex((song) => song.id === over.id);

    if (oldIndex === -1 || newIndex === -1) {
      return;
    }

    // 楽観的UI更新（positionフィールドも更新）
    const newSongs = arrayMove(setlistData.songs, oldIndex, newIndex).map((song, index) => ({
      ...song,
      position: index,
    }));
    setSetlistData({
      ...setlistData,
      songs: newSongs,
    });

    // サーバーに送信
    setError('');
    setIsReordering(true);
    try {
      const newOrderIds = newSongs.map((song) => song.id);
      await reorderSetlistSongs(setlistId, newOrderIds);
      await loadData(); // 最新データを再取得
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      await loadData(); // エラー時は元に戻す
    } finally {
      setIsReordering(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="text-gray-600">読み込み中...</div>
      </div>
    );
  }

  if (!setlistData) {
    return (
      <div className="text-center py-12">
        <p className="text-red-600">セットリストが見つかりません</p>
        <button
          onClick={onClose}
          className="mt-4 text-blue-600 hover:text-blue-700"
        >
          戻る
        </button>
      </div>
    );
  }

  const availableSongs = allSongs.filter(
    (song) => !setlistData.songs.some((ss) => ss.song.id === song.id)
  );

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div>
          <button
            onClick={onClose}
            className="text-blue-600 hover:text-blue-700 mb-2"
          >
            ← セットリスト一覧に戻る
          </button>
          <h2 className="text-2xl font-bold text-gray-900">{setlistData.setlist.name}</h2>
          {setlistData.setlist.description && (
            <p className="text-gray-600 mt-1">{setlistData.setlist.description}</p>
          )}
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowAddSong(!showAddSong)}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            {showAddSong ? 'キャンセル' : '楽曲を追加'}
          </button>
        </div>
      </div>

      {/* 曲切替コントロール */}
      {setlistData && setlistData.songs.length > 0 && (
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <button
                onClick={handlePrevious}
                disabled={setlistData.currentIndex <= 0}
                className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                ← 前の曲
              </button>
              <button
                onClick={handleNext}
                disabled={setlistData.currentIndex >= setlistData.songs.length - 1}
                className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                次の曲 →
              </button>
              <div className="border-l border-gray-300 h-8 mx-2"></div>
              <button
                onClick={handleBroadcast}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
                title="OBSオーバーレイにセットリストを送信"
              >
                オーバーレイに送信
              </button>
            </div>
            <div className="text-sm text-gray-600">
              {setlistData.currentIndex >= 0 ? (
                <span>
                  現在: {setlistData.currentIndex + 1} / {setlistData.songs.length}
                </span>
              ) : (
                <span>開始前</span>
              )}
            </div>
          </div>
        </div>
      )}

      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-600">{error}</p>
        </div>
      )}

      {isReordering && (
        <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-blue-600">曲順を保存中...</p>
        </div>
      )}

      {showAddSong && (
        <div className="bg-white rounded-lg border border-gray-200 p-4">
          <h3 className="text-lg font-semibold text-gray-900 mb-3">楽曲を選択</h3>
          {availableSongs.length === 0 ? (
            <p className="text-gray-500">追加できる楽曲がありません</p>
          ) : (
            <div className="space-y-2 max-h-96 overflow-y-auto">
              {availableSongs.map((song) => (
                <div
                  key={song.id}
                  className="flex justify-between items-center p-3 border border-gray-200 rounded-lg hover:bg-gray-50"
                >
                  <div>
                    <div className="font-medium text-gray-900">{song.title}</div>
                    {song.artist && (
                      <div className="text-sm text-gray-500">{song.artist}</div>
                    )}
                  </div>
                  <button
                    onClick={() => handleAddSong(song.id)}
                    className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors"
                  >
                    追加
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <div className="bg-white rounded-lg border border-gray-200">
        {setlistData.songs.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-gray-500">楽曲が追加されていません</p>
            <button
              onClick={() => setShowAddSong(true)}
              className="mt-4 text-blue-600 hover:text-blue-700"
            >
              最初の楽曲を追加
            </button>
          </div>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={setlistData.songs.map((s) => s.id)}
              strategy={verticalListSortingStrategy}
            >
              <div className="divide-y divide-gray-200">
                {setlistData.songs.map((setlistSong, index) => (
                  <SortableSetlistSongItem
                    key={setlistSong.id}
                    setlistSong={setlistSong}
                    index={index}
                    onRemove={() => handleRemoveSong(setlistSong.id)}
                    onSetCurrent={() => handleSetCurrent(setlistSong.position)}
                    isReordering={isReordering}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}
      </div>

      {/* タイムスタンプ出力 */}
      {setlistData && setlistData.songs.length > 0 && (
        <TimestampExporter setlist={setlistData} />
      )}
    </div>
  );
}

interface SetlistSongItemProps {
  setlistSong: SetlistSong;
  index: number;
  onRemove: () => void;
  onSetCurrent: () => void;
}

interface SortableSetlistSongItemProps extends SetlistSongItemProps {
  isReordering?: boolean;
}

function SortableSetlistSongItem(props: SortableSetlistSongItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
  } = useSortable({ id: props.setlistSong.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  // attributesとlistenersを結合（保存中は無効化）
  const dragHandleProps = props.isReordering
    ? {}
    : {
        ...attributes,
        ...listeners,
      };

  return (
    <div ref={setNodeRef} style={style}>
      <SetlistSongItem {...props} dragHandleProps={dragHandleProps} />
    </div>
  );
}

interface SetlistSongItemPropsWithDrag extends SetlistSongItemProps {
  dragHandleProps?: Record<string, unknown>;
  isReordering?: boolean;
}

function SetlistSongItem({ setlistSong, index, onRemove, onSetCurrent, dragHandleProps, isReordering }: SetlistSongItemPropsWithDrag) {
  const { song, status } = setlistSong;

  // parseTags を1回だけ呼び出してキャッシュ
  const tags = parseTags(song);

  const statusColors = {
    pending: 'bg-gray-100 text-gray-600',
    current: 'bg-green-100 text-green-800 font-semibold',
    done: 'bg-blue-100 text-blue-600',
  };

  const statusLabels = {
    pending: '未再生',
    current: '再生中',
    done: '再生済',
  };

  return (
    <div
      className={`p-4 flex items-center gap-4 ${
        status === 'current' ? 'bg-green-50' : 'hover:bg-gray-50'
      }`}
    >
      {/* ドラッグハンドル */}
      {dragHandleProps && (
        <div
          {...dragHandleProps}
          className={`flex-shrink-0 ${
            isReordering
              ? 'cursor-not-allowed opacity-50'
              : 'cursor-grab active:cursor-grabbing'
          } text-gray-400 hover:text-gray-600`}
          title={isReordering ? '保存中...' : 'ドラッグして並び替え'}
          aria-label="曲順を並び替えるドラッグハンドル"
          role="button"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 8h16M4 16h16"
            />
          </svg>
        </div>
      )}
      <div className="flex-shrink-0 w-8 text-center text-gray-500 font-medium">
        {index + 1}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <h4 className="font-medium text-gray-900 truncate">{song.title}</h4>
          <span
            className={`inline-flex items-center px-2 py-0.5 rounded text-xs ${statusColors[status]}`}
          >
            {statusLabels[status]}
          </span>
        </div>
        <div className="flex items-center gap-4 text-sm text-gray-500">
          {song.artist && <span>{song.artist}</span>}
          {song.category && <span className="text-gray-400">| {song.category}</span>}
          {song.durationSeconds && (
            <span className="text-gray-400">
              | {Math.floor(song.durationSeconds / 60)}:{String(song.durationSeconds % 60).padStart(2, '0')}
            </span>
          )}
        </div>
        {tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-2">
            {tags.map((tag, i) => (
              <span
                key={i}
                className="inline-flex items-center px-2 py-0.5 rounded text-xs bg-gray-100 text-gray-600"
              >
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>
      <div className="flex flex-shrink-0 gap-2">
        {status === 'pending' && (
          <button
            onClick={onSetCurrent}
            className="px-3 py-2 bg-green-600 text-white text-sm rounded hover:bg-green-700 transition-colors"
          >
            この曲から開始
          </button>
        )}
        <button
          onClick={onRemove}
          className="px-3 py-2 text-red-600 hover:bg-red-50 rounded transition-colors"
        >
          削除
        </button>
      </div>
    </div>
  );
}
