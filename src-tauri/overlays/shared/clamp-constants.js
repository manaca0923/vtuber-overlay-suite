/**
 * クランプ規約定数（ソース・オブ・トゥルース）
 *
 * オーバーレイ側での値制限に使用。
 * TypeScript（src/types/template.ts）およびRust（src-tauri/src/server/template_types.rs）の
 * CLAMP_RANGESと同期を維持すること。
 *
 * 使用例:
 *   const throttle = window.clampValue(value, 'updateThrottle');
 *   const offset = window.clampValue(x, 'offsetX');
 */
(function () {
  'use strict';

  /**
   * クランプ範囲定義
   * 各キーは { min, max, default } の形式
   */
  const CLAMP_RANGES = {
    // tuning（微調整）
    offsetX: { min: -40, max: 40, default: 0 },
    offsetY: { min: -40, max: 40, default: 0 },

    // rules（ルール）
    maxLines: { min: 4, max: 14, default: 10 },
    maxItems: { min: 3, max: 20, default: 14 }, // SetList推奨:14, QueueList推奨:6
    cycleSec: { min: 10, max: 120, default: 30 },
    showSec: { min: 3, max: 15, default: 6 },

    // layout（レイアウト）
    leftPct: { min: 0.18, max: 0.28, default: 0.22 },
    centerPct: { min: 0.44, max: 0.64, default: 0.56 },
    rightPct: { min: 0.18, max: 0.28, default: 0.22 },
    gutterPx: { min: 0, max: 64, default: 24 },

    // safeArea（セーフエリア）
    safeArea: { min: 0.0, max: 0.1, default: 0.04 },

    // theme.panel（パネル）
    blurPx: { min: 0, max: 24, default: 10 },
    radiusPx: { min: 0, max: 32, default: 14 },

    // theme.shadow（シャドウ）
    shadowBlur: { min: 0, max: 24, default: 8 },
    shadowOpacity: { min: 0.0, max: 1.0, default: 0.55 },
    shadowOffset: { min: -20, max: 20, default: 2 },

    // theme.outline（アウトライン）
    outlineWidth: { min: 0, max: 6, default: 2 },

    // コンポーネント固有（オーバーレイ専用、TypeScript/Rustには未定義）
    // これらはオーバーレイ側でのみ使用されるため、3層同期は不要
    updateThrottle: { min: 1000, max: 10000, default: 2000 }, // KPIBlock用
    // queueMaxItems: QueueListの表示アイテム数制限。docs/300_overlay-specs.md の
    // 「QueueList: 3-10」に基づく。汎用のmaxItems(3-20)とは異なる用途。
    queueMaxItems: { min: 3, max: 10, default: 6 },
  };

  /**
   * 汎用クランプ関数
   * NaN/Infinity/-Infinityなど非有限数は最小値にフォールバック
   * 数値文字列（"10"など）もNumber()で変換してから処理
   * @param {number|string} value - クランプする値
   * @param {number} min - 最小値
   * @param {number} max - 最大値
   * @returns {number} クランプ後の値
   */
  function clamp(value, min, max) {
    // 数値文字列対応: Number()で変換（Number.isFiniteは文字列を変換しない）
    const num = Number(value);
    // NaN/Infinityなど非有限数は最小値にフォールバック
    if (!Number.isFinite(num)) {
      return min;
    }
    return Math.max(min, Math.min(max, num));
  }

  /**
   * キー指定でクランプ
   * @param {number} value - クランプする値
   * @param {string} key - CLAMP_RANGESのキー
   * @returns {number} クランプ後の値（キーが見つからない場合はそのまま返す）
   */
  function clampValue(value, key) {
    const range = CLAMP_RANGES[key];
    if (!range) {
      console.warn(`[clamp-constants] Unknown key: ${key}`);
      return value;
    }
    return clamp(value, range.min, range.max);
  }

  /**
   * キー指定でデフォルト値を取得
   * @param {string} key - CLAMP_RANGESのキー
   * @returns {number|undefined} デフォルト値
   */
  function getDefault(key) {
    const range = CLAMP_RANGES[key];
    return range ? range.default : undefined;
  }

  /**
   * キー指定で範囲を取得
   * @param {string} key - CLAMP_RANGESのキー
   * @returns {{min: number, max: number, default: number}|undefined}
   */
  function getRange(key) {
    return CLAMP_RANGES[key];
  }

  // グローバルに公開
  window.CLAMP_RANGES = CLAMP_RANGES;
  window.clampValue = clampValue;
  window.getClampDefault = getDefault;
  window.getClampRange = getRange;
})();
