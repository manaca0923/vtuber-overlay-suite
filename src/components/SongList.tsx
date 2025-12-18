import { useEffect, useState } from 'react';
import { getSongs, deleteSong } from '../types/commands';
import type { Song } from '../types/song';
import { parseTags } from '../types/song';
import { SongForm } from './SongForm';

export function SongList() {
  const [songs, setSongs] = useState<Song[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [showForm, setShowForm] = useState(false);
  const [editingSong, setEditingSong] = useState<Song | null>(null);

  const loadSongs = async () => {
    try {
      setLoading(true);
      const result = await getSongs();
      setSongs(result);
      setError('');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSongs();
  }, []);

  const handleDelete = async (id: string) => {
    if (!confirm('本当にこの楽曲を削除しますか？')) return;

    try {
      await deleteSong(id);
      await loadSongs();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleEdit = (song: Song) => {
    setEditingSong(song);
    setShowForm(true);
  };

  const handleFormClose = async (saved: boolean) => {
    setShowForm(false);
    setEditingSong(null);
    if (saved) {
      await loadSongs();
    }
  };

  if (loading) {
    return <div className="text-gray-600">読み込み中...</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">楽曲管理</h2>
        <button
          onClick={() => setShowForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          楽曲を追加
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-600">{error}</p>
        </div>
      )}

      {songs.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
          <p className="text-gray-500">楽曲が登録されていません</p>
          <button
            onClick={() => setShowForm(true)}
            className="mt-4 text-blue-600 hover:text-blue-700"
          >
            最初の楽曲を追加
          </button>
        </div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  タイトル
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  アーティスト
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  カテゴリ
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  タグ
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  時間
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  操作
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {songs.map((song) => (
                <tr key={song.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm font-medium text-gray-900">{song.title}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">{song.artist || '-'}</div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">{song.category || '-'}</div>
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex flex-wrap gap-1">
                      {parseTags(song).map((tag, i) => (
                        <span
                          key={i}
                          className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="text-sm text-gray-500">
                      {song.durationSeconds
                        ? `${Math.floor(song.durationSeconds / 60)}:${String(song.durationSeconds % 60).padStart(2, '0')}`
                        : '-'}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <button
                      onClick={() => handleEdit(song)}
                      className="text-blue-600 hover:text-blue-900 mr-4"
                    >
                      編集
                    </button>
                    <button
                      onClick={() => handleDelete(song.id)}
                      className="text-red-600 hover:text-red-900"
                    >
                      削除
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {showForm && (
        <SongForm
          song={editingSong}
          onClose={handleFormClose}
        />
      )}
    </div>
  );
}
