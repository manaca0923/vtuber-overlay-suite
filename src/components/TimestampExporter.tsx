import { useState } from 'react';
import type { SetlistWithSongs } from '../types/setlist';

interface TimestampExporterProps {
  setlist: SetlistWithSongs;
}

export function TimestampExporter({ setlist }: TimestampExporterProps) {
  const [copied, setCopied] = useState(false);

  /**
   * 配信開始時刻を取得（最初に開始された曲の開始時刻）
   */
  const getStreamStartTime = (): Date | null => {
    const startedSongs = setlist.songs
      .filter(s => s.startedAt)
      .sort((a, b) => {
        const timeA = new Date(a.startedAt!).getTime();
        const timeB = new Date(b.startedAt!).getTime();
        return timeA - timeB;
      });

    if (startedSongs.length === 0) {
      return null;
    }

    const firstSong = startedSongs[0];
    if (!firstSong || !firstSong.startedAt) {
      return null;
    }

    return new Date(firstSong.startedAt);
  };

  /**
   * ISO時刻文字列から配信開始からの経過時間を計算
   * @param isoString - ISO 8601形式の時刻文字列
   * @returns "3:45" または "1:23:45" 形式の文字列
   */
  const formatTime = (isoString: string): string => {
    const streamStart = getStreamStartTime();
    if (!streamStart) {
      return '0:00';
    }

    const eventTime = new Date(isoString);
    const elapsedMs = eventTime.getTime() - streamStart.getTime();
    const elapsedSeconds = Math.floor(elapsedMs / 1000);

    const hours = Math.floor(elapsedSeconds / 3600);
    const minutes = Math.floor((elapsedSeconds % 3600) / 60);
    const seconds = elapsedSeconds % 60;

    if (hours > 0) {
      return `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
    } else {
      return `${minutes}:${String(seconds).padStart(2, '0')}`;
    }
  };

  /**
   * YouTube概要欄用タイムスタンプテキストを生成
   */
  const generateTimestamps = (): string => {
    const startedSongs = setlist.songs
      .filter(s => s.startedAt)
      .sort((a, b) => a.position - b.position);

    if (startedSongs.length === 0) {
      return '（まだ曲が再生されていません）';
    }

    return startedSongs
      .map(song => {
        const time = formatTime(song.startedAt!);
        const artist = song.song.artist ? ` / ${song.song.artist}` : '';
        return `${time} ${song.song.title}${artist}`;
      })
      .join('\n');
  };

  /**
   * クリップボードにコピー
   */
  const handleCopy = async () => {
    const text = generateTimestamps();
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const timestamps = generateTimestamps();
  const streamStart = getStreamStartTime();

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-4">
      <div className="flex justify-between items-center mb-3">
        <h3 className="text-lg font-semibold text-gray-900">YouTube概要欄用タイムスタンプ</h3>
        <button
          onClick={handleCopy}
          disabled={!streamStart}
          className={`px-4 py-2 rounded-lg transition-colors ${
            copied
              ? 'bg-green-600 text-white'
              : streamStart
              ? 'bg-blue-600 text-white hover:bg-blue-700'
              : 'bg-gray-300 text-gray-500 cursor-not-allowed'
          }`}
        >
          {copied ? 'コピーしました！' : 'コピー'}
        </button>
      </div>

      {streamStart && (
        <div className="mb-2 text-sm text-gray-600">
          配信開始時刻: {streamStart.toLocaleString('ja-JP')}
        </div>
      )}

      <textarea
        value={timestamps}
        readOnly
        rows={10}
        className="w-full px-3 py-2 border border-gray-300 rounded-lg bg-gray-50 text-sm font-mono resize-y focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="曲を再生するとタイムスタンプが表示されます"
      />

      <div className="mt-2 text-xs text-gray-500">
        {streamStart ? (
          <>再生済みの曲（{setlist.songs.filter(s => s.startedAt).length}曲）のタイムスタンプが表示されます</>
        ) : (
          <>曲を再生すると、YouTube概要欄にコピー＆ペーストできるタイムスタンプが生成されます</>
        )}
      </div>
    </div>
  );
}
