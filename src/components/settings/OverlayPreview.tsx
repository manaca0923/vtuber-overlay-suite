import { useMemo } from 'react';
import type { OverlaySettings } from '../../types/overlaySettings';

interface OverlayPreviewProps {
  settings: OverlaySettings;
  activePanel: 'comment' | 'setlist';
}

export function OverlayPreview({ settings, activePanel }: OverlayPreviewProps) {
  const previewUrl = useMemo(() => {
    const base =
      activePanel === 'comment'
        ? 'http://localhost:19800/overlay/comment'
        : 'http://localhost:19800/overlay/setlist';

    // 共通パラメータ
    const params = new URLSearchParams({
      preview: 'true',
      primaryColor: settings.common.primaryColor,
      borderRadius: String(settings.common.borderRadius),
    });

    // パネル固有のパラメータを追加
    if (activePanel === 'comment') {
      params.set('fontSize', String(settings.comment.fontSize));
      params.set('maxCount', String(settings.comment.maxCount));
      params.set('showAvatar', String(settings.comment.showAvatar));
    } else {
      params.set('fontSize', String(settings.setlist.fontSize));
      params.set('showArtist', String(settings.setlist.showArtist));
    }

    return `${base}?${params.toString()}`;
  }, [settings, activePanel]);

  return (
    <div className="bg-gray-900 rounded-lg overflow-hidden h-full">
      <div className="p-2 bg-gray-800 text-white text-sm flex justify-between items-center">
        <span>プレビュー</span>
        <span className="text-gray-400 text-xs">
          {activePanel === 'comment' ? 'コメントオーバーレイ' : 'セットリストオーバーレイ'}
        </span>
      </div>
      <div className="aspect-video bg-gray-800">
        <iframe
          src={previewUrl}
          className="w-full h-full border-0"
          title="Overlay Preview"
        />
      </div>
      <div className="p-3 bg-gray-800 border-t border-gray-700">
        <p className="text-xs text-gray-400">
          OBSブラウザソースURL:
        </p>
        <code className="text-xs text-blue-400 break-all">
          {activePanel === 'comment'
            ? 'http://localhost:19800/overlay/comment'
            : 'http://localhost:19800/overlay/setlist'}
        </code>
      </div>
    </div>
  );
}
