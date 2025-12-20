interface WizardNavigationProps {
  currentStep: number;
  totalSteps: number;
  canGoNext: boolean;
  canGoPrevious: boolean;
  onNext: () => void;
  onPrevious: () => void;
  onComplete: () => void;
  loading?: boolean;
}

export default function WizardNavigation({
  currentStep,
  totalSteps,
  canGoNext,
  canGoPrevious,
  onNext,
  onPrevious,
  onComplete,
  loading = false,
}: WizardNavigationProps) {
  const isLastStep = currentStep === totalSteps;

  return (
    <div className="flex items-center justify-between">
      {/* 前へボタン */}
      <button
        onClick={onPrevious}
        disabled={!canGoPrevious || loading}
        className={`px-4 py-2 rounded-lg font-medium transition-colors ${
          canGoPrevious && !loading
            ? 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            : 'bg-gray-100 text-gray-400 cursor-not-allowed'
        }`}
      >
        ← 前へ
      </button>

      {/* ステップ表示 */}
      <div className="text-sm text-gray-600">
        Step {currentStep} / {totalSteps}
      </div>

      {/* 次へ/完了ボタン */}
      {isLastStep ? (
        <button
          onClick={onComplete}
          disabled={loading}
          className={`px-6 py-2 rounded-lg font-medium transition-colors ${
            loading
              ? 'bg-gray-400 text-white cursor-not-allowed'
              : 'bg-blue-600 text-white hover:bg-blue-700'
          }`}
        >
          {loading ? '処理中...' : '完了'}
        </button>
      ) : (
        <button
          onClick={onNext}
          disabled={!canGoNext || loading}
          className={`px-6 py-2 rounded-lg font-medium transition-colors ${
            canGoNext && !loading
              ? 'bg-blue-600 text-white hover:bg-blue-700'
              : 'bg-gray-400 text-white cursor-not-allowed'
          }`}
        >
          {loading ? '処理中...' : '次へ →'}
        </button>
      )}
    </div>
  );
}
