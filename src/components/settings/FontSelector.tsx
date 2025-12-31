import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FONT_PRESETS, type FontPresetName, type ThemeSettings } from '../../types/overlaySettings';

interface FontSelectorProps {
  themeSettings: ThemeSettings;
  onChange: (settings: ThemeSettings) => void;
}

// Google Fontsの読み込み済みフラグ（重複読み込み防止）
const loadedGoogleFonts = new Set<string>();

/**
 * フォントファミリー名をサニタイズ（XSS対策）
 * issues/002: オーバーレイセキュリティ対応
 * @param fontFamily フォントファミリー名
 * @returns サニタイズ済みのフォントファミリー名、無効な場合はnull
 */
function sanitizeFontFamily(fontFamily: string | null | undefined): string | null {
  if (typeof fontFamily !== 'string' || fontFamily.length === 0 || fontFamily.length > 200) {
    return null;
  }
  // 危険な文字を除去（comment-renderer.jsと同じロジック）
  return fontFamily.replace(/[<>"'`;{}]/g, '');
}

/**
 * Google Fontsを動的に読み込む
 * @param fontSpec フォント指定（例: 'Noto+Sans+JP:wght@400;500;700'）
 */
function loadGoogleFont(fontSpec: string): void {
  if (!fontSpec || loadedGoogleFonts.has(fontSpec)) return;

  // fontSpecは既にURLエンコード済みの形式（+はスペース、:と;はそのまま）
  // encodeURIComponentを使うと+が%2Bに変換されてしまうので直接使用
  const url = `https://fonts.googleapis.com/css2?family=${fontSpec}&display=swap`;

  // セキュリティチェック
  try {
    const parsed = new URL(url);
    if (parsed.hostname !== 'fonts.googleapis.com') return;
  } catch {
    return;
  }

  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = url;
  document.head.appendChild(link);
  loadedGoogleFonts.add(fontSpec);
}

/**
 * フォント設定コンポーネント
 * - プリセット選択（Noto Sans JP, M PLUS 1, 游ゴシック, メイリオ）
 * - システムフォント選択（Rust動的取得）
 * - プレビュー表示
 *
 * issues/013: アクセシビリティ対応（id, htmlFor, aria-label）
 */
export function FontSelector({ themeSettings, onChange }: FontSelectorProps) {
  const [systemFonts, setSystemFonts] = useState<string[]>([]);
  const [isLoadingFonts, setIsLoadingFonts] = useState(false);
  const [fontError, setFontError] = useState<string | null>(null);
  const fontsLoadedRef = useRef(false);

  // システムフォント取得（useCallbackでメモ化）
  const loadSystemFonts = useCallback(async () => {
    if (fontsLoadedRef.current) return; // 読み込み済み

    setIsLoadingFonts(true);
    setFontError(null);

    try {
      const fonts = await invoke<string[]>('get_system_fonts');
      setSystemFonts(fonts);
      fontsLoadedRef.current = true;
    } catch (error) {
      console.error('Failed to load system fonts:', error);
      setFontError('システムフォントの取得に失敗しました');
    } finally {
      setIsLoadingFonts(false);
    }
  }, []);

  // システムフォント取得（system選択時）
  useEffect(() => {
    if (themeSettings.fontPreset === 'system') {
      loadSystemFonts();
    }
  }, [themeSettings.fontPreset, loadSystemFonts]);

  // Google Fontsを読み込む（プリセット変更時）
  useEffect(() => {
    const preset = FONT_PRESETS[themeSettings.fontPreset];
    if (preset?.googleFont) {
      loadGoogleFont(preset.googleFont);
    }
  }, [themeSettings.fontPreset]);

  // フォントプリセット変更ハンドラ
  const handlePresetChange = (preset: FontPresetName) => {
    const updates: Partial<ThemeSettings> = {
      fontPreset: preset,
    };

    // system以外の場合はcustomFontFamilyをクリア
    if (preset !== 'system') {
      updates.customFontFamily = null;
    }

    onChange({
      ...themeSettings,
      ...updates,
    });

    // systemの場合はフォント一覧を読み込む
    if (preset === 'system') {
      loadSystemFonts();
    }
  };

  // システムフォント選択ハンドラ
  const handleSystemFontChange = (fontFamily: string) => {
    onChange({
      ...themeSettings,
      fontPreset: 'system',
      customFontFamily: fontFamily,
    });
  };

  // 現在のフォントファミリーを取得（サニタイズ済み）
  const getCurrentFontFamily = (): string => {
    if (themeSettings.fontPreset === 'system' && themeSettings.customFontFamily) {
      // カスタムフォントファミリーはサニタイズして使用
      const sanitized = sanitizeFontFamily(themeSettings.customFontFamily);
      return sanitized || FONT_PRESETS['yu-gothic'].fontFamily;
    }
    return FONT_PRESETS[themeSettings.fontPreset]?.fontFamily || FONT_PRESETS['yu-gothic'].fontFamily;
  };

  return (
    <div className="space-y-4">
      {/* セクションタイトル */}
      <h3 className="text-lg font-bold text-gray-900">フォント設定</h3>

      {/* プリセット選択 */}
      <div className="space-y-2">
        <label htmlFor="font-preset" className="text-sm font-medium text-gray-700">
          フォントプリセット
        </label>
        <select
          id="font-preset"
          aria-label="フォントプリセットを選択"
          value={themeSettings.fontPreset}
          onChange={(e) => handlePresetChange(e.target.value as FontPresetName)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          {(Object.entries(FONT_PRESETS) as [FontPresetName, (typeof FONT_PRESETS)[FontPresetName]][]).map(
            ([key, preset]) => (
              <option key={key} value={key}>
                {preset.name}
              </option>
            )
          )}
        </select>
      </div>

      {/* システムフォント選択（system選択時のみ表示） */}
      {themeSettings.fontPreset === 'system' && (
        <div className="space-y-2">
          <label htmlFor="system-font" className="text-sm font-medium text-gray-700">
            システムフォント
          </label>
          {isLoadingFonts ? (
            <p className="text-sm text-gray-500">フォント一覧を読み込み中...</p>
          ) : fontError ? (
            <p className="text-sm text-red-500">{fontError}</p>
          ) : (
            <select
              id="system-font"
              aria-label="システムフォントを選択"
              value={themeSettings.customFontFamily || ''}
              onChange={(e) => handleSystemFontChange(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="">選択してください</option>
              {systemFonts.map((font) => (
                <option key={font} value={font}>
                  {font}
                </option>
              ))}
            </select>
          )}
        </div>
      )}

      {/* プレビュー */}
      <div className="space-y-2">
        <p className="text-sm font-medium text-gray-700">プレビュー</p>
        <div
          className="p-4 border border-gray-200 rounded-lg bg-gray-900 text-white"
          style={{ fontFamily: getCurrentFontFamily() }}
        >
          <p className="text-2xl mb-2">あいうえお ABC 12345</p>
          <p className="text-sm opacity-70">The quick brown fox jumps over the lazy dog.</p>
        </div>
      </div>

      {/* 情報テキスト */}
      <p className="text-xs text-gray-500">
        ※ Noto Sans JP、M PLUS 1はGoogle Fontsから動的に読み込まれます。
        <br />
        ※ 游ゴシック、メイリオはWindows専用フォントです（macOSでは代替フォントで表示）。
      </p>
    </div>
  );
}
