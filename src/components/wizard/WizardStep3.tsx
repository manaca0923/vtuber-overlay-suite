interface WizardStep3Props {
  selectedTemplate: 'default';
  onTemplateChange: (template: 'default') => void;
}

export default function WizardStep3({
  selectedTemplate,
  onTemplateChange,
}: WizardStep3Props) {
  return (
    <div>
      <h2 className="text-xl font-bold mb-4">Step 3: テンプレートの選択</h2>
      <p className="text-gray-600 mb-6">
        オーバーレイのデザインテンプレートを選択してください。
      </p>

      <div className="space-y-4">
        {/* デフォルトテンプレート */}
        <div
          role="button"
          tabIndex={0}
          onClick={() => onTemplateChange('default')}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              onTemplateChange('default');
            }
          }}
          className={`p-6 border-2 rounded-lg cursor-pointer transition-all ${
            selectedTemplate === 'default'
              ? 'border-blue-600 bg-blue-50'
              : 'border-gray-300 hover:border-gray-400'
          }`}
        >
          <div className="flex items-start gap-4">
            <div
              className={`w-6 h-6 rounded-full border-2 flex items-center justify-center ${
                selectedTemplate === 'default'
                  ? 'border-blue-600 bg-blue-600'
                  : 'border-gray-300'
              }`}
            >
              {selectedTemplate === 'default' && (
                <svg
                  className="w-4 h-4 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              )}
            </div>
            <div className="flex-1">
              <h3 className="font-bold text-lg mb-2">デフォルトテンプレート</h3>
              <p className="text-gray-600 mb-3">
                シンプルで見やすいデザイン。すべての機能が利用できます。
              </p>
              <ul className="text-sm text-gray-500 space-y-1">
                <li>✓ コメント表示</li>
                <li>✓ スーパーチャット対応</li>
                <li>✓ セットリスト表示</li>
                <li>✓ カスタマイズ可能</li>
              </ul>
            </div>
          </div>
        </div>

        {/* 将来のテンプレート（グレーアウト） */}
        <div className="p-6 border-2 border-gray-200 rounded-lg opacity-60 cursor-not-allowed bg-gray-50">
          <div className="flex items-start gap-4">
            <div className="w-6 h-6 rounded-full border-2 border-gray-300"></div>
            <div className="flex-1">
              <h3 className="font-bold text-lg mb-2 text-gray-500">
                プレミアムテンプレート
              </h3>
              <p className="text-gray-400 mb-3">
                今後のアップデートで追加予定
              </p>
              <span className="inline-block px-3 py-1 bg-gray-200 text-gray-500 text-sm rounded-full">
                近日公開
              </span>
            </div>
          </div>
        </div>

        <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <h3 className="font-medium text-blue-900 mb-2">今後の予定</h3>
          <p className="text-sm text-blue-800">
            今後のアップデートで、さまざまなデザインテンプレートを追加予定です。
            カスタマイズ機能も拡充していきます。
          </p>
        </div>
      </div>
    </div>
  );
}
