import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ApiKeySetup } from './components/ApiKeySetup';
import { CommentControlPanel } from './components/CommentControlPanel';
import { SongList } from './components/SongList';
import { SetlistList } from './components/SetlistList';
import { TestModeButton } from './components/TestModeButton';
import { OverlaySettings } from './components/settings';
import Wizard from './components/wizard/Wizard';

type Tab = 'comment' | 'setlist' | 'settings';
type AppMode = 'wizard' | 'main';

interface WizardSettings {
  video_id: string;
  live_chat_id: string;
  saved_at: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('comment');
  const [appMode, setAppMode] = useState<AppMode>('wizard');
  const [isCheckingFirstLaunch, setIsCheckingFirstLaunch] = useState(true);
  const [apiKey, setApiKey] = useState<string>('');
  const [wizardSettings, setWizardSettings] = useState<WizardSettings | null>(null);

  // 初回起動判定 & 設定読み込み
  useEffect(() => {
    async function initialize() {
      try {
        const hasKey = await invoke<boolean>('has_api_key');
        if (hasKey) {
          // APIキーを読み込む
          try {
            const key = await invoke<string | null>('get_api_key');
            if (key) {
              setApiKey(key);
            }
          } catch (err) {
            console.error('Failed to load API key:', err);
          }
          // ウィザード設定を読み込む
          try {
            const settings = await invoke<WizardSettings | null>('load_wizard_settings');
            if (settings) {
              setWizardSettings(settings);
            }
          } catch (err) {
            console.error('Failed to load wizard settings:', err);
          }
          setAppMode('main');
        } else {
          setAppMode('wizard');
        }
      } catch (err) {
        console.error('Failed to check API key:', err);
        setAppMode('wizard');
      } finally {
        setIsCheckingFirstLaunch(false);
      }
    }
    initialize();
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

  // ウィザード完了ハンドラ
  const handleWizardComplete = async () => {
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
  };

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
            <div className="flex justify-end">
              <TestModeButton />
            </div>
            <CommentControlPanel
              apiKey={apiKey}
              videoId={wizardSettings?.video_id ?? ''}
              liveChatId={wizardSettings?.live_chat_id ?? ''}
            />
            <ApiKeySetup />
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
    </div>
  );
}

export default App;
