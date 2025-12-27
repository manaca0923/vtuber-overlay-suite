import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import WizardNavigation from './WizardNavigation';
import WizardStep1 from './WizardStep1';
import WizardStep2 from './WizardStep2';
import WizardStep3 from './WizardStep3';
import WizardStep4 from './WizardStep4';

interface WizardData {
  apiKey: string;
  apiKeyValid: boolean;
  videoId: string;
  liveChatId: string | null;
  selectedTemplate: 'default';
  setupComplete: boolean;
}

interface WizardProps {
  onComplete: () => void;
}

export default function Wizard({ onComplete }: WizardProps) {
  const [currentStep, setCurrentStep] = useState<1 | 2 | 3 | 4>(1);
  const [wizardData, setWizardData] = useState<WizardData>({
    apiKey: '',
    apiKeyValid: false,
    videoId: '',
    liveChatId: null,
    selectedTemplate: 'default',
    setupComplete: false,
  });
  const [error, setError] = useState('');
  const [warning, setWarning] = useState('');

  const canProceedToNextStep = (): boolean => {
    switch (currentStep) {
      case 1:
        return wizardData.apiKeyValid;
      case 2:
        return wizardData.liveChatId !== null;
      case 3:
        return wizardData.selectedTemplate === 'default';
      case 4:
        return true;
      default:
        return false;
    }
  };

  const handleNext = () => {
    if (canProceedToNextStep() && currentStep < 4) {
      setCurrentStep((currentStep + 1) as 1 | 2 | 3 | 4);
      setError('');
    }
  };

  const handlePrevious = () => {
    if (currentStep > 1) {
      setCurrentStep((currentStep - 1) as 1 | 2 | 3 | 4);
      setError('');
    }
  };

  const handleComplete = async () => {
    // ウィザード設定を保存
    if (wizardData.videoId && wizardData.liveChatId) {
      try {
        // 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
        // use_bundled_key: nullで既存値を維持（ウィザードではuseBundledKeyを管理しない）
        await invoke('save_wizard_settings', {
          video_id: wizardData.videoId,
          live_chat_id: wizardData.liveChatId,
          use_bundled_key: null,
        });
      } catch (err) {
        console.error('Failed to save wizard settings:', err);
        // 保存失敗時は警告を表示し、2秒後に完了
        setWarning('設定の保存に失敗しましたが、セットアップは完了します。次回起動時に再設定が必要な場合があります。');
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }
    }
    setWizardData({ ...wizardData, setupComplete: true });
    onComplete();
  };

  // APIキーをスキップしてInnerTubeモードで続行
  const handleSkipApiKey = async () => {
    // ダミーの設定を保存してウィザード完了扱いにする
    try {
      // 注意: Tauriコマンド引数はRust側のsnake_caseに合わせる必要がある
      // use_bundled_key: nullで既存値を維持
      await invoke('save_wizard_settings', {
        video_id: '',
        live_chat_id: '',
        use_bundled_key: null,
      });
    } catch (err) {
      console.error('Failed to save wizard settings:', err);
    }
    onComplete();
  };

  const updateWizardData = useCallback((updates: Partial<WizardData>) => {
    setWizardData((prev) => ({ ...prev, ...updates }));
  }, []);

  // Step2用のコールバックをメモ化
  const handleVideoIdChange = useCallback((videoId: string) => {
    updateWizardData({ videoId });
  }, [updateWizardData]);

  const handleLiveChatIdChange = useCallback((liveChatId: string | null) => {
    updateWizardData({ liveChatId });
  }, [updateWizardData]);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg max-w-2xl w-full p-6 max-h-[90vh] overflow-y-auto">
        {/* ヘッダー */}
        <div className="mb-6">
          <h1 className="text-2xl font-bold text-gray-900">初期設定ウィザード</h1>
          <p className="text-gray-600 mt-2">
            VTuber配信支援ツールへようこそ！簡単な設定を行います。
          </p>
        </div>

        {/* ステップインジケーター */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            {[1, 2, 3, 4].map((step) => (
              <div key={step} className="flex-1">
                <div className="flex items-center">
                  <div
                    className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                      step === currentStep
                        ? 'bg-blue-600 text-white'
                        : step < currentStep
                        ? 'bg-green-600 text-white'
                        : 'bg-gray-300 text-gray-600'
                    }`}
                  >
                    {step < currentStep ? '✓' : step}
                  </div>
                  {step < 4 && (
                    <div
                      className={`flex-1 h-1 mx-2 ${
                        step < currentStep ? 'bg-green-600' : 'bg-gray-300'
                      }`}
                    />
                  )}
                </div>
                <div className="mt-2 text-xs text-center text-gray-600">
                  {step === 1 && 'APIキー'}
                  {step === 2 && '動画ID'}
                  {step === 3 && 'テンプレート'}
                  {step === 4 && 'OBS設定'}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* エラー表示 */}
        {error && (
          <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
            {error}
          </div>
        )}

        {/* 警告表示 */}
        {warning && (
          <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg text-yellow-700 text-sm">
            {warning}
          </div>
        )}

        {/* ステップコンテンツ */}
        <div className="mb-6">
          {currentStep === 1 && (
            <WizardStep1
              apiKey={wizardData.apiKey}
              onApiKeyChange={(apiKey) =>
                updateWizardData({ apiKey })
              }
              onValidationChange={(isValid) =>
                updateWizardData({ apiKeyValid: isValid })
              }
              onSkip={handleSkipApiKey}
            />
          )}
          {currentStep === 2 && (
            <WizardStep2
              apiKey={wizardData.apiKey}
              videoId={wizardData.videoId}
              onVideoIdChange={handleVideoIdChange}
              onLiveChatIdChange={handleLiveChatIdChange}
            />
          )}
          {currentStep === 3 && (
            <WizardStep3
              selectedTemplate={wizardData.selectedTemplate}
              onTemplateChange={(selectedTemplate) =>
                updateWizardData({ selectedTemplate })
              }
            />
          )}
          {currentStep === 4 && <WizardStep4 />}
        </div>

        {/* ナビゲーション */}
        <WizardNavigation
          currentStep={currentStep}
          totalSteps={4}
          canGoNext={canProceedToNextStep()}
          canGoPrevious={currentStep > 1}
          onNext={handleNext}
          onPrevious={handlePrevious}
          onComplete={handleComplete}
        />
      </div>
    </div>
  );
}
