import { useEffect, useState } from 'react';
import { getSetlists, deleteSetlist } from '../types/commands';
import type { Setlist } from '../types/setlist';
import { SetlistEditor } from './SetlistEditor';
import { SetlistForm } from './SetlistForm';

export function SetlistList() {
  const [setlists, setSetlists] = useState<Setlist[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [showForm, setShowForm] = useState(false);
  const [editingSetlistId, setEditingSetlistId] = useState<string | null>(null);

  const loadSetlists = async () => {
    try {
      setLoading(true);
      const result = await getSetlists();
      setSetlists(result);
      setError('');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSetlists();
  }, []);

  const handleDelete = async (id: string) => {
    if (!confirm('本当にこのセットリストを削除しますか？')) return;

    try {
      await deleteSetlist(id);
      await loadSetlists();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleFormClose = async (saved: boolean) => {
    setShowForm(false);
    if (saved) {
      await loadSetlists();
    }
  };

  const handleEditorClose = () => {
    setEditingSetlistId(null);
  };

  if (loading) {
    return <div className="text-gray-600">読み込み中...</div>;
  }

  if (editingSetlistId) {
    return <SetlistEditor setlistId={editingSetlistId} onClose={handleEditorClose} />;
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">セットリスト管理</h2>
        <button
          onClick={() => setShowForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          セットリストを作成
        </button>
      </div>

      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-600">{error}</p>
        </div>
      )}

      {setlists.length === 0 ? (
        <div className="text-center py-12 bg-white rounded-lg border border-gray-200">
          <p className="text-gray-500">セットリストが作成されていません</p>
          <button
            onClick={() => setShowForm(true)}
            className="mt-4 text-blue-600 hover:text-blue-700"
          >
            最初のセットリストを作成
          </button>
        </div>
      ) : (
        <div className="grid gap-4">
          {setlists.map((setlist) => (
            <div
              key={setlist.id}
              className="bg-white rounded-lg border border-gray-200 p-6 hover:shadow-md transition-shadow"
            >
              <div className="flex justify-between items-start">
                <div className="flex-1">
                  <h3 className="text-xl font-semibold text-gray-900 mb-2">
                    {setlist.name}
                  </h3>
                  {setlist.description && (
                    <p className="text-gray-600 mb-4">{setlist.description}</p>
                  )}
                  <p className="text-sm text-gray-400">
                    作成日: {new Date(setlist.createdAt).toLocaleString('ja-JP')}
                  </p>
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => setEditingSetlistId(setlist.id)}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                  >
                    編集
                  </button>
                  <button
                    onClick={() => handleDelete(setlist.id)}
                    className="px-4 py-2 border border-red-600 text-red-600 rounded-lg hover:bg-red-50 transition-colors"
                  >
                    削除
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {showForm && <SetlistForm onClose={handleFormClose} />}
    </div>
  );
}
