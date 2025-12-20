import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ApiKeySetup } from './components/ApiKeySetup';
import { SongList } from './components/SongList';
import { SetlistList } from './components/SetlistList';
import Wizard from './components/wizard/Wizard';

type Tab = 'comment' | 'setlist';
type AppMode = 'wizard' | 'main';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('comment');
  const [appMode, setAppMode] = useState<AppMode>('wizard');
  const [isCheckingFirstLaunch, setIsCheckingFirstLaunch] = useState(true);

  // 初回起動判定
  useEffect(() => {
    async function checkFirstLaunch() {
      try {
        const hasKey = await invoke<boolean>('has_api_key');
        setAppMode(hasKey ? 'main' : 'wizard');
      } catch (err) {
        console.error('Failed to check API key:', err);
        // エラー時はウィザード表示（安全側に倒す）
        setAppMode('wizard');
      } finally {
        setIsCheckingFirstLaunch(false);
      }
    }
    checkFirstLaunch();
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
    return <Wizard onComplete={() => setAppMode('main')} />;
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
          </nav>
        </div>

        {activeTab === 'comment' && <ApiKeySetup />}
        {activeTab === 'setlist' && (
          <div className="space-y-8">
            <SongList />
            <SetlistList />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
