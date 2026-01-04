import { useState, useEffect, useCallback, useRef, type ChangeEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** 保存のdebounce間隔（ミリ秒） */
const SAVE_DEBOUNCE_MS = 500;

/** ロゴURL最大長（バイト） - Rust側と同期 */
const MAX_LOGO_URL_LENGTH = 2048;

/** テキスト最大長（文字） - Rust側と同期 */
const MAX_TEXT_LENGTH = 100;

/**
 * 許可するdata: URLのMIMEタイプ（プレフィックス）- Rust側と同期
 * NOTE: SVGはスクリプト/外部参照によるセキュリティリスクがあるため除外
 */
const ALLOWED_DATA_IMAGE_PREFIXES = [
  'data:image/png',
  'data:image/jpeg',
  'data:image/gif',
  'data:image/webp',
] as const;

interface BrandSettings {
  logoUrl: string | null;
  text: string | null;
}

export function BrandSettingsPanel() {
  // 入力用のローカル状態（debounce用）
  const [localLogoUrl, setLocalLogoUrl] = useState('');
  const [localText, setLocalText] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  // プレビュー画像のエラー状態
  const [previewError, setPreviewError] = useState(false);

  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // 保存のシーケンス番号（非同期競合対策）
  const saveSeqRef = useRef(0);
  // 最新値を保持するref（debounce内で最新値を参照するため）
  const latestValuesRef = useRef({ logoUrl: '', text: '' });

  // ブランド設定を読み込み
  const loadBrandSettings = useCallback(async () => {
    try {
      const settings = await invoke<BrandSettings>('get_brand_settings');
      const logoUrl = settings.logoUrl ?? '';
      const text = settings.text ?? '';
      setLocalLogoUrl(logoUrl);
      setLocalText(text);
      // refも同期
      latestValuesRef.current = { logoUrl, text };
    } catch (err) {
      console.error('Failed to load brand settings:', err);
      setError('ブランド設定の読み込みに失敗しました');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadBrandSettings();
  }, [loadBrandSettings]);

  // タイマークリーンアップ
  useEffect(() => {
    return () => {
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current);
      }
    };
  }, []);

  // ステータスメッセージのクリア
  const clearMessages = useCallback(() => {
    setError('');
    setSuccess('');
  }, []);

  // UTF-8バイト長を計算（Rust側の.len()と同期）
  const getUtf8ByteLength = (str: string): number => {
    return new TextEncoder().encode(str).length;
  };

  // URL検証（フロントエンド側）
  const validateUrl = (url: string): boolean => {
    if (!url) return true; // 空は許可
    // バイト長でチェック（Rust側のurl.len()と同期）
    if (getUtf8ByteLength(url) > MAX_LOGO_URL_LENGTH) return false;
    // http, https, data:image/(許可リスト) スキームのみ許可
    // NOTE: SVGはスクリプト/外部参照によるセキュリティリスクがあるため除外
    const isHttp = url.startsWith('http://') || url.startsWith('https://');
    const isAllowedData = ALLOWED_DATA_IMAGE_PREFIXES.some((prefix) => url.startsWith(prefix));
    return isHttp || isAllowedData;
  };

  // 設定を保存（latestValuesRefから最新値を取得）
  const handleSave = async () => {
    // 最新値をrefから取得し、トリム済み値を作成
    // NOTE: 検証と保存で同じトリム済み値を使用（検証→保存の順序を揃える）
    const trimmedLogoUrl = latestValuesRef.current.logoUrl.trim();
    const trimmedText = latestValuesRef.current.text.trim();

    // シーケンス番号をインクリメント
    saveSeqRef.current += 1;
    const currentSeq = saveSeqRef.current;

    // URL検証（トリム後の値で検証）
    if (trimmedLogoUrl && !validateUrl(trimmedLogoUrl)) {
      setError(`無効なURLです。http://, https://, または data:image/(png|jpeg|gif|webp) で始まり、${MAX_LOGO_URL_LENGTH}バイト以内のURLを入力してください。`);
      return;
    }

    // テキスト長検証（サロゲートペア対応 - Rust側のtext.chars().count()と同期、トリム後の値で検証）
    if ([...trimmedText].length > MAX_TEXT_LENGTH) {
      setError(`テキストが長すぎます（最大${MAX_TEXT_LENGTH}文字）`);
      return;
    }

    clearMessages();
    setSaving(true);
    try {
      const newSettings: BrandSettings = {
        logoUrl: trimmedLogoUrl || null,
        text: trimmedText || null,
      };

      // 保存とブロードキャストを単一コマンドで実行（原子性を担保）
      await invoke('save_and_broadcast_brand', { brand_settings: newSettings });

      // 最新のリクエストのみ成功メッセージを表示
      if (currentSeq === saveSeqRef.current) {
        setSuccess('設定を保存しました');
        setTimeout(() => setSuccess(''), 2000);
      }
    } catch (err) {
      // 最新のリクエストのみエラーを表示
      if (currentSeq === saveSeqRef.current) {
        console.error('Failed to save brand settings:', err);
        setError(typeof err === 'string' ? err : '設定の保存に失敗しました');
      }
    } finally {
      // 最新のリクエストのみsaving状態を解除
      if (currentSeq === saveSeqRef.current) {
        setSaving(false);
      }
    }
  };

  // ロゴURL入力の変更（debounce）
  const handleLogoUrlChange = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setLocalLogoUrl(value);
    setPreviewError(false); // プレビューエラーをリセット

    // refを更新（debounceコールバック内で最新値を参照するため）
    latestValuesRef.current.logoUrl = value;

    // 既存のタイマーをクリア
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
    }

    // debounce後に保存（handleSaveはlatestValuesRefから最新値を取得）
    saveTimerRef.current = setTimeout(() => {
      handleSave();
    }, SAVE_DEBOUNCE_MS);
  };

  // テキスト入力の変更（debounce）
  const handleTextChange = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setLocalText(value);

    // refを更新（debounceコールバック内で最新値を参照するため）
    latestValuesRef.current.text = value;

    // 既存のタイマーをクリア
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
    }

    // debounce後に保存（handleSaveはlatestValuesRefから最新値を取得）
    saveTimerRef.current = setTimeout(() => {
      handleSave();
    }, SAVE_DEBOUNCE_MS);
  };

  // クリアボタン
  const handleClear = async () => {
    setLocalLogoUrl('');
    setLocalText('');
    setPreviewError(false);
    clearMessages();

    // refも更新
    latestValuesRef.current = { logoUrl: '', text: '' };

    // タイマーをキャンセル
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
    }

    await handleSave();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600 text-sm">読み込み中...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-2">ロゴ設定</h3>
        <p className="text-sm text-gray-500">
          配信画面に表示するロゴ画像またはテキストを設定できます。
        </p>
      </div>

      {/* エラー・成功メッセージ */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {error}
        </div>
      )}
      {success && (
        <div className="p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 text-sm">
          {success}
        </div>
      )}

      {/* ロゴURL入力 */}
      <div>
        <label htmlFor="logo-url" className="block text-sm font-medium text-gray-700 mb-1">
          ロゴ画像URL
        </label>
        <input
          id="logo-url"
          type="url"
          value={localLogoUrl}
          onChange={handleLogoUrlChange}
          placeholder="https://example.com/logo.png"
          disabled={saving}
          className="w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 placeholder:text-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
        />
        <p className="text-xs text-gray-500 mt-1">
          http://, https://, または data:image/(png|jpeg|gif|webp) で始まるURLを入力してください
        </p>
      </div>

      {/* プレビュー */}
      {/* NOTE: トリム済み＆検証済みのURLのみ表示（SVG等の拒否対象をUI上で読み込まない） */}
      {(() => {
        const trimmedUrl = localLogoUrl.trim();
        const isValidForPreview = trimmedUrl && validateUrl(trimmedUrl) && !previewError;
        return isValidForPreview ? (
          <div className="p-4 bg-gray-800 rounded-lg">
            <p className="text-xs text-gray-400 mb-2">プレビュー</p>
            <div className="flex items-center justify-center min-h-16">
              <img
                src={trimmedUrl}
                alt="Logo preview"
                onError={() => setPreviewError(true)}
                className="max-h-16 max-w-full object-contain"
              />
            </div>
          </div>
        ) : null;
      })()}

      {/* プレビューエラー */}
      {previewError && localLogoUrl.trim() && validateUrl(localLogoUrl.trim()) && (
        <div className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg text-yellow-700 text-sm">
          画像の読み込みに失敗しました。URLが正しいか確認してください。
        </div>
      )}

      {/* 代替テキスト入力 */}
      <div>
        <label htmlFor="brand-text" className="block text-sm font-medium text-gray-700 mb-1">
          代替テキスト
        </label>
        <input
          id="brand-text"
          type="text"
          value={localText}
          onChange={handleTextChange}
          placeholder="例: チャンネル名, @username"
          disabled={saving}
          className="w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 placeholder:text-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
        />
        {/* NOTE: maxLength属性はUTF-16コードユニット単位でサロゲートペアを正しくカウントしないため削除。
            文字数制限はバックエンド（Rust側chars().count()）で行い、フロントでは[...text].lengthで事前検証 */}
        <p className="text-xs text-gray-500 mt-1">
          ロゴ画像がない場合や読み込みエラー時に表示されます（最大{MAX_TEXT_LENGTH}文字）
        </p>
      </div>

      {/* クリアボタン */}
      {(localLogoUrl || localText) && (
        <div className="flex justify-end">
          <button
            type="button"
            onClick={handleClear}
            disabled={saving}
            className="px-4 py-2 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 rounded-md transition-colors disabled:text-gray-400"
          >
            設定をクリア
          </button>
        </div>
      )}

      {/* ヒント */}
      <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h4 className="text-sm font-medium text-blue-900 mb-1">使い方のヒント</h4>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• 推奨画像サイズ: 高さ60px程度（幅は自動調整）</li>
          <li>• 透過PNG/WebP形式を推奨</li>
          <li>• 表示ON/OFFは「ウィジェット」タブで設定できます</li>
        </ul>
      </div>
    </div>
  );
}
