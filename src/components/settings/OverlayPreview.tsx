import { useMemo, useRef, useState, useEffect } from 'react';
import type { OverlaySettings } from '../../types/overlaySettings';

type PreviewMode = 'combined' | 'individual';

interface OverlayPreviewProps {
  settings: OverlaySettings;
  activePanel: 'comment' | 'setlist';
  mode?: PreviewMode;
}

// OBS推奨サイズ
const OBS_WIDTH = 1920;
const OBS_HEIGHT = 1080;

export function OverlayPreview({ settings, activePanel, mode = 'combined' }: OverlayPreviewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(0.3);

  // コンテナサイズに基づいてスケールを計算
  useEffect(() => {
    const updateScale = () => {
      if (containerRef.current) {
        const containerWidth = containerRef.current.clientWidth;
        const containerHeight = containerRef.current.clientHeight;

        // 幅と高さの両方に収まるスケールを計算
        const scaleX = containerWidth / OBS_WIDTH;
        const scaleY = containerHeight / OBS_HEIGHT;
        const newScale = Math.min(scaleX, scaleY, 1); // 1を超えないように

        setScale(newScale);
      }
    };

    updateScale();

    // ResizeObserverでコンテナサイズの変更を監視
    const resizeObserver = new ResizeObserver(updateScale);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => resizeObserver.disconnect();
  }, []);

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
        <span className="text-gray-400 text-xs">{displayMode} ({Math.round(scale * 100)}%)</span>
      </div>
      <div
        ref={containerRef}
        className="flex-1 bg-gray-800 min-h-0 flex items-center justify-center overflow-hidden"
      >
        <div
          style={{
            width: OBS_WIDTH,
            height: OBS_HEIGHT,
            transform: `scale(${scale})`,
            transformOrigin: 'center center',
          }}
        >
          <iframe
            src={previewUrl}
            style={{
              width: OBS_WIDTH,
              height: OBS_HEIGHT,
              border: 'none',
            }}
            title="Overlay Preview"
            sandbox="allow-scripts allow-same-origin"
          />
        </div>
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
