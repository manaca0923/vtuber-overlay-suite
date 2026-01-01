import { useMemo, useRef, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { OverlaySettings } from '../../types/overlaySettings';
import { LAYOUT_PRESETS } from '../../types/overlaySettings';
import type { LiveStreamStats } from '../../types/weather';

// KPIデータ型（オーバーレイへのpostMessage用）
interface KpiData {
  main: number | null;
  label: string;
  sub: number | null;
  subLabel: string | null;
}

// ウィザード設定型（Rust側でrename_allなしのためsnake_case）
interface WizardSettingsData {
  video_id: string;
  live_chat_id: string;
  use_bundled_key?: boolean;
}

type PreviewMode = 'combined' | 'individual';

// カスタムフック: スライダー操作時のパフォーマンス最適化用デバウンス
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}

interface OverlayPreviewProps {
  settings: OverlaySettings;
  activePanel: 'comment' | 'setlist';
  mode?: PreviewMode;
}

// OBS推奨サイズ
const OBS_WIDTH = 1920;
const OBS_HEIGHT = 1080;

// プレビュー用定数
const PREVIEW_ORIGIN = 'http://localhost:19800';  // iframeのorigin（postMessage送信先）
const DEBOUNCE_DELAY_MS = 50;  // スライダー操作時のデバウンス遅延

// スパチャプレビュー用カスタムイベント名
export const SUPERCHAT_PREVIEW_EVENT = 'preview:superchat:add';
export const SUPERCHAT_REMOVE_PREVIEW_EVENT = 'preview:superchat:remove';

export function OverlayPreview({ settings, activePanel, mode = 'combined' }: OverlayPreviewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [scale, setScale] = useState(0.3);
  // loadedUrlを追跡することで、useEffectでsetStateを呼ぶ必要がなくなる
  const [loadedUrl, setLoadedUrl] = useState<string | null>(null);
  // イベントハンドラからiframeLoaded状態を参照するためのref（クロージャ問題回避）
  const iframeLoadedRef = useRef(false);

  // スライダー操作時のパフォーマンス最適化
  const debouncedSettings = useDebounce(settings, DEBOUNCE_DELAY_MS);

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

  // レイアウトバージョンを判定
  const layoutVersion = LAYOUT_PRESETS[settings.layout]?.version || 'v1';
  const isV2Layout = layoutVersion === 'v2';

  // キャッシュバスター（開発時のiframeキャッシュ対策）
  // コンポーネントマウント時に一度だけ生成
  const cacheKey = useMemo(() => Date.now().toString(), []);

  // previewUrl: iframeの再作成を最小限にするため、URLパラメータは最小限に
  // 他の設定（カラー、フォントサイズ等）はpostMessageで即時反映
  const previewUrl = useMemo(() => {
    if (mode === 'combined') {
      // 統合オーバーレイ: layoutのみURLで指定、他はpostMessageで反映
      const params = new URLSearchParams({
        preview: 'true',
        layout: settings.layout,
        _v: cacheKey, // キャッシュバスター
      });
      // v2レイアウトの場合はcombined-v2を使用
      const endpoint = isV2Layout ? '/overlay/combined-v2' : '/overlay/combined';
      return `http://localhost:19800${endpoint}?${params.toString()}`;
    }

    // 個別オーバーレイ
    const base =
      activePanel === 'comment'
        ? 'http://localhost:19800/overlay/comment'
        : 'http://localhost:19800/overlay/setlist';

    // 個別オーバーレイも最小限のパラメータのみ
    const params = new URLSearchParams({
      preview: 'true',
      _v: cacheKey, // キャッシュバスター
    });

    return `${base}?${params.toString()}`;
  }, [settings.layout, activePanel, mode, isV2Layout, cacheKey]);

  // loadedUrlとpreviewUrlを比較してiframeがロード済みかを判定
  // これによりuseEffect内でsetStateを呼ぶ必要がなくなる（react-hooks/set-state-in-effect回避）
  const iframeLoaded = loadedUrl === previewUrl;

  // iframeLoaded状態をrefに同期（イベントハンドラから参照するため）
  useEffect(() => {
    iframeLoadedRef.current = iframeLoaded;
  }, [iframeLoaded]);

  // 設定変更時にpostMessageを送信（即時プレビュー機能）
  useEffect(() => {
    if (!iframeLoaded || !iframeRef.current?.contentWindow) return;

    // docs/300: settings:update payload形式に準拠
    const message = {
      type: 'preview:settings:update',
      payload: {
        theme: debouncedSettings.theme,
        primaryColor: debouncedSettings.common.primaryColor,
        fontFamily: debouncedSettings.common.fontFamily,
        borderRadius: debouncedSettings.common.borderRadius,
        comment: debouncedSettings.comment,
        setlist: debouncedSettings.setlist,
        weather: debouncedSettings.weather,
        widget: debouncedSettings.widget,
        performance: debouncedSettings.performance,
        superchat: debouncedSettings.superchat,
        themeSettings: debouncedSettings.themeSettings,
      }
    };

    // iframeのcontentWindowに送信（セキュリティ: targetOriginを明示）
    iframeRef.current.contentWindow.postMessage(message, PREVIEW_ORIGIN);
  }, [debouncedSettings, iframeLoaded]);

  // プレビューモード時にKPIデータを取得してiframeに送信
  // Video IDがあれば実データを取得、なければダミーデータを使用
  useEffect(() => {
    if (!iframeLoaded || !iframeRef.current?.contentWindow) return;

    const fetchAndSendKpiData = async () => {
      // デフォルトのダミーデータ
      const dummyKpi: KpiData = {
        main: 1234,
        label: '視聴者',
        sub: 567,
        subLabel: '高評価',
      };

      const sendKpiMessage = (kpiData: KpiData) => {
        iframeRef.current?.contentWindow?.postMessage({
          type: 'preview:kpi:update',
          payload: kpiData,
        }, PREVIEW_ORIGIN);
      };

      try {
        // ウィザード設定からvideo_idを取得
        const wizardSettings = await invoke<WizardSettingsData | null>('load_wizard_settings');

        if (wizardSettings?.video_id) {
          // video_idがあれば実データを取得
          const useBundledKey = wizardSettings.use_bundled_key ?? true;

          try {
            const stats = await invoke<LiveStreamStats>('get_live_stream_stats', {
              video_id: wizardSettings.video_id,
              use_bundled_key: useBundledKey,
            });

            // 実データでKPIを更新
            const realKpi: KpiData = {
              main: stats.concurrentViewers,
              label: '視聴者',
              sub: stats.likeCount,
              subLabel: stats.likeCount !== null ? '高評価' : null,
            };

            sendKpiMessage(realKpi);
            return; // 成功したら終了
          } catch (statsError) {
            // API取得失敗（配信終了、ネットワークエラー等）
            console.warn('[OverlayPreview] KPI取得失敗、ダミーデータを使用:', statsError);
          }
        }

        // video_idがない、または取得失敗した場合はダミーデータを送信
        sendKpiMessage(dummyKpi);

      } catch (error) {
        // ウィザード設定の読み込みに失敗した場合もダミーデータを使用
        console.warn('[OverlayPreview] ウィザード設定読み込み失敗、ダミーデータを使用:', error);
        sendKpiMessage(dummyKpi);
      }
    };

    // iframeロード後に少し遅延させてKPIデータを送信（コンポーネント初期化完了を待つ）
    const timer = setTimeout(fetchAndSendKpiData, 200);
    return () => clearTimeout(timer);
  }, [iframeLoaded]);

  // スパチャテスト送信のカスタムイベントをリッスン（プレビューiframeへ転送）
  // 注: イベントリスナーは一度だけ登録し、refを使って最新の状態を参照
  useEffect(() => {
    const handleSuperchatAdd = (event: Event) => {
      const customEvent = event as CustomEvent;
      // refを使って最新のiframeLoaded状態を参照（クロージャ問題回避）
      if (!iframeLoadedRef.current || !customEvent.detail || !iframeRef.current?.contentWindow) {
        return;
      }

      iframeRef.current.contentWindow.postMessage({
        type: 'preview:superchat:add',
        payload: customEvent.detail,
      }, PREVIEW_ORIGIN);
    };

    const handleSuperchatRemove = (event: Event) => {
      const customEvent = event as CustomEvent;
      // refを使って最新のiframeLoaded状態を参照（クロージャ問題回避）
      if (!iframeLoadedRef.current || !customEvent.detail || !iframeRef.current?.contentWindow) {
        return;
      }

      iframeRef.current.contentWindow.postMessage({
        type: 'preview:superchat:remove',
        payload: { id: customEvent.detail.id },
      }, PREVIEW_ORIGIN);
    };

    window.addEventListener(SUPERCHAT_PREVIEW_EVENT, handleSuperchatAdd);
    window.addEventListener(SUPERCHAT_REMOVE_PREVIEW_EVENT, handleSuperchatRemove);

    return () => {
      window.removeEventListener(SUPERCHAT_PREVIEW_EVENT, handleSuperchatAdd);
      window.removeEventListener(SUPERCHAT_REMOVE_PREVIEW_EVENT, handleSuperchatRemove);
    };
  }, []); // 依存配列を空に - イベントリスナーは一度だけ登録

  // iframeのhandleLoad関数
  // previewUrlが変わると、iframeのkeyも変わるため自動的に再作成される
  // loadedUrlを更新することで、iframeLoadedが自動的にtrueになる
  const handleIframeLoad = () => {
    setLoadedUrl(previewUrl);
  };

  const displayMode = mode === 'combined'
    ? (isV2Layout ? '3カラム統合オーバーレイ' : '統合オーバーレイ')
    : activePanel === 'comment' ? 'コメントオーバーレイ' : 'セットリストオーバーレイ';

  const obsUrl = mode === 'combined'
    ? (isV2Layout ? 'http://localhost:19800/overlay/combined-v2' : 'http://localhost:19800/overlay/combined')
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
            key={previewUrl}
            ref={iframeRef}
            src={previewUrl}
            onLoad={handleIframeLoad}
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
