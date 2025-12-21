import { useMemo } from 'react';
import type { OverlaySettings } from '../../types/overlaySettings';

type PreviewMode = 'combined' | 'individual';

interface OverlayPreviewProps {
  settings: OverlaySettings;
  activePanel: 'comment' | 'setlist';
  mode?: PreviewMode;
}

export function OverlayPreview({ settings, activePanel, mode = 'combined' }: OverlayPreviewProps) {
  const previewUrl = useMemo(() => {
    if (mode === 'combined') {
      // 統合オーバーレイ
      const params = new URLSearchParams({
        preview: 'true',
        layout: settings.layout,
        primaryColor: settings.common.primaryColor,
        commentFontSize: String(settings.comment.fontSize),
        showAvatar: String(settings.comment.showAvatar),
        commentEnabled: String(settings.comment.enabled),
        setlistFontSize: String(settings.setlist.fontSize),
        showArtist: String(settings.setlist.showArtist),
        setlistEnabled: String(settings.setlist.enabled),
      });
      return `http://localhost:19800/overlay/combined?${params.toString()}`;
    }

    // 個別オーバーレイ
    const base =
      activePanel === 'comment'
        ? 'http://localhost:19800/overlay/comment'
        : 'http://localhost:19800/overlay/setlist';

    const params = new URLSearchParams({
      preview: 'true',
      primaryColor: settings.common.primaryColor,
      borderRadius: String(settings.common.borderRadius),
    });

    if (activePanel === 'comment') {
      params.set('fontSize', String(settings.comment.fontSize));
      params.set('maxCount', String(settings.comment.maxCount));
      params.set('showAvatar', String(settings.comment.showAvatar));
      params.set('position', settings.comment.position);
      params.set('enabled', String(settings.comment.enabled));
    } else {
      params.set('fontSize', String(settings.setlist.fontSize));
      params.set('showArtist', String(settings.setlist.showArtist));
      params.set('position', settings.setlist.position);
      params.set('enabled', String(settings.setlist.enabled));
    }

    return `${base}?${params.toString()}`;
  }, [settings, activePanel, mode]);

  const displayMode = mode === 'combined' ? '統合オーバーレイ' :
    activePanel === 'comment' ? 'コメントオーバーレイ' : 'セットリストオーバーレイ';

  const obsUrl = mode === 'combined'
    ? 'http://localhost:19800/overlay/combined'
    : activePanel === 'comment'
      ? 'http://localhost:19800/overlay/comment'
      : 'http://localhost:19800/overlay/setlist';

  return (
    <div className="bg-gray-900 rounded-lg overflow-hidden h-full flex flex-col">
      <div className="p-2 bg-gray-800 text-white text-sm flex justify-between items-center">
        <span>プレビュー</span>
        <span className="text-gray-400 text-xs">{displayMode}</span>
      </div>
      <div className="flex-1 bg-gray-800 min-h-0">
        <iframe
          src={previewUrl}
          className="w-full h-full border-0"
          title="Overlay Preview"
          sandbox="allow-scripts allow-same-origin"
        />
      </div>
      <div className="p-3 bg-gray-800 border-t border-gray-700">
        <p className="text-xs text-gray-400 mb-1">OBSブラウザソースURL:</p>
        <div className="flex items-center gap-2">
          <code className="text-xs text-blue-400 break-all flex-1">{obsUrl}</code>
          <button
            type="button"
            onClick={() => {
              navigator.clipboard.writeText(obsUrl);
            }}
            className="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 text-white rounded transition-colors"
          >
            コピー
          </button>
        </div>
        {mode === 'combined' && (
          <p className="text-xs text-gray-500 mt-2">
            統合オーバーレイは1つのブラウザソースでコメント＋セットリストを表示します。
            画面サイズは1920x1080推奨。
          </p>
        )}
      </div>
    </div>
  );
}
