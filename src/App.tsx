import { useState } from 'react';
import { ApiKeySetup } from './components/ApiKeySetup';
import { SongList } from './components/SongList';
import { SetlistList } from './components/SetlistList';

type Tab = 'comment' | 'setlist';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('comment');

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
