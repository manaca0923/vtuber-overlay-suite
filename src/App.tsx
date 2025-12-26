import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ApiKeySetup } from './components/ApiKeySetup';
import { CommentControlPanel } from './components/CommentControlPanel';
import { SongList } from './components/SongList';
import { SetlistList } from './components/SetlistList';
import { TestModeButton } from './components/TestModeButton';
import { OverlaySettings } from './components/settings';
import Wizard from './components/wizard/Wizard';
import { UpdateChecker } from './components/UpdateChecker';
import { VideoIdModal } from './components/VideoIdModal';

type Tab = 'comment' | 'setlist' | 'settings';
type AppMode = 'wizard' | 'main';

interface WizardSettings {
  video_id: string;
  live_chat_id: string;
  saved_at: string;
}

interface StatusMessage {
  type: 'success' | 'error';
  text: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('comment');
  const [appMode, setAppMode] = useState<AppMode>('wizard');
  const [isCheckingFirstLaunch, setIsCheckingFirstLaunch] = useState(true);
  const [apiKey, setApiKey] = useState<string>('');
  const [wizardSettings, setWizardSettings] = useState<WizardSettings | null>(null);
  const [isVideoIdModalOpen, setIsVideoIdModalOpen] = useState(false);
  const [statusMessage, setStatusMessage] = useState<StatusMessage | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const statusTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ステータスメッセージを表示（3秒後に自動消去）
  const showStatus = useCallback((type: 'success' | 'error', text: string) => {
    // 既存のタイマーをクリア
    if (statusTimerRef.current) {
      clearTimeout(statusTimerRef.current);
    }
    setStatusMessage({ type, text });
    statusTimerRef.current = setTimeout(() => {
      setStatusMessage(null);
      statusTimerRef.current = null;
    }, 3000);
  }, []);

  // コメント取得開始ハンドラ
  const handleStartPolling = useCallback(async (videoId: string) => {
    try {
      await invoke('start_polling_innertube', { video_id: videoId });
      setIsPolling(true);
      showStatus('success', 'コメント取得を開始しました');
    } catch (e) {
      showStatus('error', 'エラー: ' + (e instanceof Error ? e.message : String(e)));
    }
  }, [showStatus]);

  // コメント取得停止ハンドラ
  const handleStopPolling = useCallback(async () => {
    try {
      await invoke('stop_polling_innertube');
      setIsPolling(false);
      showStatus('success', 'コメント取得を停止しました');
    } catch (e) {
      showStatus('error', 'エラー: ' + (e instanceof Error ? e.message : String(e)));
    }
  }, [showStatus]);

  // 開始ボタンクリック時の処理
  const handleStartClick = useCallback(() => {
    if (wizardSettings?.video_id) {
      // 設定済みのVideo IDがあればそのまま開始
      handleStartPolling(wizardSettings.video_id);
    } else {
      // なければモーダルを表示
      setIsVideoIdModalOpen(true);
    }
  }, [wizardSettings?.video_id, handleStartPolling]);

  // ウィザード完了ハンドラ
  const handleWizardComplete = useCallback(async () => {
    // 設定を再読み込み
    try {
      const key = await invoke<string | null>('get_api_key');
      if (key) {
        setApiKey(key);
      }
      const settings = await invoke<WizardSettings | null>('load_wizard_settings');
      if (settings) {
        setWizardSettings(settings);
      }
    } catch (err) {
      console.error('Failed to reload settings after wizard:', err);
    }
    setAppMode('main');
  }, []);

  // 初回起動判定 & 設定読み込み
  useEffect(() => {
    async function initialize() {
      try {
        // APIキーを読み込む（あれば）
        const hasKey = await invoke<boolean>('has_api_key');
        if (hasKey) {
          try {
            const key = await invoke<string | null>('get_api_key');
            if (key) {
              setApiKey(key);
            }
          } catch (err) {
            console.error('Failed to load API key:', err);
          }
        }

        // ウィザード設定を読み込む
        let hasWizardSettings = false;
        try {
          const settings = await invoke<WizardSettings | null>('load_wizard_settings');
          if (settings) {
            setWizardSettings(settings);
            hasWizardSettings = true;
          }
        } catch (err) {
          console.error('Failed to load wizard settings:', err);
        }

        // APIキーがあるか、ウィザード設定があればメイン画面へ
        if (hasKey || hasWizardSettings) {
          setAppMode('main');
        } else {
          setAppMode('wizard');
        }
      } catch (err) {
        console.error('Failed to initialize:', err);
        setAppMode('wizard');
      } finally {
        setIsCheckingFirstLaunch(false);
      }
    }
    initialize();
  }, []);

  // クリーンアップ：アンマウント時にタイマーをクリア
  useEffect(() => {
    return () => {
      if (statusTimerRef.current) {
        clearTimeout(statusTimerRef.current);
      }
    };
  }, []);

  // ローディング画面
  if (isCheckingFirstLaunch) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">読み込み中...</p>
        </div>
      </div>
    );
  }

  // ウィザードモード
  if (appMode === 'wizard') {
    return <Wizard onComplete={handleWizardComplete} />;
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="container mx-auto py-8">
        <h1 className="text-4xl font-bold mb-2 text-gray-900">
          VTuber Overlay Suite
        </h1>
        <p className="text-gray-600 mb-8">VTuber streaming support tool</p>

        <div className="mb-6 border-b border-gray-200">
          <nav className="flex gap-4">
            <button
              onClick={() => setActiveTab('comment')}
              className={`pb-4 px-2 font-medium transition-colors ${
                activeTab === 'comment'
                  ? 'text-blue-600 border-b-2 border-blue-600'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              コメント管理
            </button>
            <button
              onClick={() => setActiveTab('setlist')}
              className={`pb-4 px-2 font-medium transition-colors ${
                activeTab === 'setlist'
                  ? 'text-blue-600 border-b-2 border-blue-600'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              セットリスト管理
            </button>
            <button
              onClick={() => setActiveTab('settings')}
              className={`pb-4 px-2 font-medium transition-colors ${
                activeTab === 'settings'
                  ? 'text-blue-600 border-b-2 border-blue-600'
                  : 'text-gray-500 hover:text-gray-700'
              }`}
            >
              オーバーレイ設定
            </button>
          </nav>
        </div>

        {activeTab === 'comment' && (
          <div className="space-y-6">
            {/* ステータスメッセージ */}
            {statusMessage && (
              <div
                className={`p-3 rounded-md text-sm ${
                  statusMessage.type === 'success'
                    ? 'bg-green-100 text-green-800 border border-green-200'
                    : 'bg-red-100 text-red-800 border border-red-200'
                }`}
              >
                {statusMessage.text}
              </div>
            )}

            <div className="flex justify-end gap-2 items-center">
              {/* ポーリング状態表示 */}
              {isPolling && (
                <span className="text-sm text-green-600 flex items-center gap-1">
                  <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                  取得中
                </span>
              )}

              {/* InnerTube コメント取得（メイン機能） */}
              <button
                onClick={handleStartClick}
                disabled={isPolling}
                className={`px-3 py-1 text-sm rounded transition-colors ${
                  isPolling
                    ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                    : 'bg-green-600 text-white hover:bg-green-700'
                }`}
              >
                コメント取得開始
              </button>
              <button
                onClick={handleStopPolling}
                disabled={!isPolling}
                className={`px-3 py-1 text-sm rounded transition-colors ${
                  !isPolling
                    ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                    : 'bg-red-600 text-white hover:bg-red-700'
                }`}
              >
                コメント取得停止
              </button>
              <TestModeButton />
            </div>
            {/* 公式API操作パネル（デバッグモードのみ） */}
            {import.meta.env.DEV && (
              <>
                <CommentControlPanel
                  apiKey={apiKey}
                  videoId={wizardSettings?.video_id ?? ''}
                  liveChatId={wizardSettings?.live_chat_id ?? ''}
                  onSettingsChange={(settings) => {
                    setWizardSettings((prev) => ({
                      video_id: settings.videoId ?? prev?.video_id ?? '',
                      live_chat_id: settings.liveChatId ?? prev?.live_chat_id ?? '',
                      saved_at: new Date().toISOString(),
                    }));
                  }}
                />
                <ApiKeySetup
                  onSettingsChange={(settings) => {
                    if (settings.apiKey) {
                      setApiKey(settings.apiKey);
                    }
                    if (settings.videoId || settings.liveChatId) {
                      setWizardSettings((prev) => ({
                        video_id: settings.videoId ?? prev?.video_id ?? '',
                        live_chat_id: settings.liveChatId ?? prev?.live_chat_id ?? '',
                        saved_at: new Date().toISOString(),
                      }));
                    }
                  }}
                />
              </>
            )}
          </div>
        )}
        {activeTab === 'setlist' && (
          <div className="space-y-8">
            <SongList />
            <SetlistList />
          </div>
        )}
        {activeTab === 'settings' && (
          <OverlaySettings />
        )}
      </div>

      {/* アップデートチェッカー */}
      <UpdateChecker />

      {/* Video ID入力モーダル */}
      <VideoIdModal
        isOpen={isVideoIdModalOpen}
        onClose={() => setIsVideoIdModalOpen(false)}
        onSubmit={handleStartPolling}
        defaultValue={wizardSettings?.video_id}
      />
    </div>
  );
}

export default App;
