import { useState } from 'react';
import { open } from '@tauri-apps/plugin-shell';

export default function WizardStep4() {
  const [copiedUrl, setCopiedUrl] = useState<string | null>(null);

  const overlayUrls = [
    {
      name: 'コメント表示オーバーレイ',
      url: 'http://localhost:19800/overlay/comment',
      description: 'ライブチャットのコメントを表示します',
    },
    {
      name: 'セットリスト表示オーバーレイ',
      url: 'http://localhost:19800/overlay/setlist',
      description: '演奏曲のセットリストを表示します',
    },
  ];

  const handleCopy = async (url: string) => {
    try {
      await navigator.clipboard.writeText(url);
      setCopiedUrl(url);
      setTimeout(() => setCopiedUrl(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const handleOpenBrowser = async (url: string) => {
    try {
      await open(url);
    } catch (err) {
      console.error('Failed to open URL:', err);
    }
  };

  return (
    <div>
      <h2 className="text-xl font-bold mb-4">Step 4: OBS設定ガイド</h2>
      <p className="text-gray-600 mb-6">
        OBS Studioでオーバーレイを表示するための設定方法です。
      </p>

      {/* オーバーレイURL一覧 */}
      <div className="space-y-4 mb-6">
        {overlayUrls.map((overlay) => (
          <div key={overlay.url} className="p-4 border border-gray-300 rounded-lg">
            <h3 className="font-bold mb-2">{overlay.name}</h3>
            <p className="text-sm text-gray-600 mb-3">{overlay.description}</p>
            <div className="flex gap-2">
              <input
                type="text"
                value={overlay.url}
                readOnly
                className="flex-1 px-3 py-2 bg-gray-50 border border-gray-300 rounded text-sm"
              />
              <button
                onClick={() => handleCopy(overlay.url)}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors text-sm"
              >
                {copiedUrl === overlay.url ? 'コピー済み!' : 'コピー'}
              </button>
              <button
                onClick={() => handleOpenBrowser(overlay.url)}
                className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors text-sm"
              >
                開く
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* OBS設定手順 */}
      <div className="p-6 bg-gray-50 border border-gray-200 rounded-lg">
        <h3 className="font-bold text-lg mb-4">OBS Studio 設定手順</h3>
        <ol className="space-y-3 text-sm">
          <li className="flex gap-3">
            <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold">
              1
            </span>
            <div>
              <strong>OBS Studioを起動</strong>
              <p className="text-gray-600 mt-1">
                まだインストールしていない場合は、
                <a
                  href="https://obsproject.com/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 hover:underline"
                >
                  こちら
                </a>
                からダウンロードしてください。
              </p>
            </div>
          </li>
          <li className="flex gap-3">
            <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold">
              2
            </span>
            <div>
              <strong>ソースを追加</strong>
              <p className="text-gray-600 mt-1">
                「ソース」パネルの「+」ボタンをクリックし、「ブラウザ」を選択します。
              </p>
            </div>
          </li>
          <li className="flex gap-3">
            <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold">
              3
            </span>
            <div>
              <strong>URLを貼り付け</strong>
              <p className="text-gray-600 mt-1">
                上記のURLをコピーして、「URL」フィールドに貼り付けます。
              </p>
            </div>
          </li>
          <li className="flex gap-3">
            <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold">
              4
            </span>
            <div>
              <strong>推奨設定</strong>
              <p className="text-gray-600 mt-1">
                幅: 400px、高さ: 600px<br />
                「カスタムCSS」で背景を透過にする場合は不要です。
              </p>
            </div>
          </li>
          <li className="flex gap-3">
            <span className="flex-shrink-0 w-6 h-6 rounded-full bg-blue-600 text-white flex items-center justify-center text-xs font-bold">
              5
            </span>
            <div>
              <strong>配置を調整</strong>
              <p className="text-gray-600 mt-1">
                OBSのプレビュー画面でオーバーレイの位置とサイズを調整します。
              </p>
            </div>
          </li>
        </ol>
      </div>

      <div className="mt-6 p-4 bg-green-50 border border-green-200 rounded-lg">
        <p className="text-green-800 text-sm">
          ✓ 設定が完了したら、「完了」ボタンをクリックしてウィザードを終了してください。
        </p>
      </div>
    </div>
  );
}
