/**
 * 全コンポーネントの基底クラス
 *
 * 使用例:
 *   class ClockWidget extends BaseComponent {
 *     render() {
 *       const el = document.createElement('div');
 *       el.textContent = 'Clock';
 *       return el;
 *     }
 *   }
 */
class BaseComponent {
  /**
   * コンストラクタ
   * @param {object} config - 設定オブジェクト
   * @param {object} config.rules - 動作ルール（maxLines, maxItems, cycleSec, showSec）
   * @param {object} config.tuning - 微調整（offsetX, offsetY）
   * @param {object} config.style - スタイル設定
   */
  constructor(config = {}) {
    this.config = config;
    this.rules = config.rules || {};
    this.tuning = config.tuning || {};
    this.style = config.style || {};
    this.element = null;
    this.slotElement = null;
    this._timers = [];
    this._animationFrames = [];
  }

  /**
   * 初期化（slotにマウント）
   * @param {HTMLElement} slotElement
   */
  init(slotElement) {
    this.slotElement = slotElement;
    this.element = this.render();

    if (!this.element) {
      console.warn(`[${this.constructor.name}] render() returned null/undefined`);
      return;
    }

    this.applyTuning();
    this.applyStyle();

    slotElement.innerHTML = '';
    slotElement.appendChild(this.element);

    this.afterMount();
  }

  /**
   * 要素を生成（サブクラスで実装必須）
   * @returns {HTMLElement}
   */
  render() {
    throw new Error(`[${this.constructor.name}] render() must be implemented`);
  }

  /**
   * マウント後の処理（サブクラスでオーバーライド）
   */
  afterMount() {}

  /**
   * データ更新（サブクラスでオーバーライド）
   * @param {object} data
   */
  update(data) {}

  /**
   * イベントハンドラ（サブクラスでオーバーライド）
   * @param {string} eventType
   * @param {object} payload
   */
  onEvent(eventType, payload) {}

  /**
   * 破棄処理
   */
  destroy() {
    // タイマーをクリア
    this._timers.forEach((t) => clearInterval(t));
    this._timers = [];

    // アニメーションフレームをキャンセル
    this._animationFrames.forEach((id) => cancelAnimationFrame(id));
    this._animationFrames = [];

    // 要素を削除
    if (this.element && this.element.parentNode) {
      this.element.remove();
    }

    this.element = null;
    this.slotElement = null;
  }

  /**
   * tuning（offset）の適用
   */
  applyTuning() {
    if (!this.element) return;

    const offsetX = this.clampByKey(this.tuning.offsetX || 0, 'offsetX', -40, 40);
    const offsetY = this.clampByKey(this.tuning.offsetY || 0, 'offsetY', -40, 40);

    if (offsetX !== 0 || offsetY !== 0) {
      this.element.style.transform = `translate(${offsetX}px, ${offsetY}px)`;
    }
  }

  /**
   * style適用（サブクラスで拡張可能）
   */
  applyStyle() {
    // サブクラスでオーバーライドして独自スタイルを適用
  }

  /**
   * 要素取得
   * @returns {HTMLElement|null}
   */
  getElement() {
    return this.element;
  }

  /**
   * 周期タイマー登録（破棄時に自動クリア）
   * @param {Function} callback
   * @param {number} ms
   * @returns {number} タイマーID
   */
  setInterval(callback, ms) {
    const id = setInterval(callback, ms);
    this._timers.push(id);
    return id;
  }

  /**
   * タイマーをクリア
   * @param {number} id
   */
  clearInterval(id) {
    clearInterval(id);
    this._timers = this._timers.filter((t) => t !== id);
  }

  /**
   * requestAnimationFrame登録（破棄時に自動キャンセル）
   * @param {Function} callback
   * @returns {number} フレームID
   */
  requestAnimationFrame(callback) {
    const id = requestAnimationFrame(callback);
    this._animationFrames.push(id);
    return id;
  }

  /**
   * クランプユーティリティ
   * @param {number} value
   * @param {number} min
   * @param {number} max
   * @returns {number}
   */
  clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
  }

  /**
   * キー指定でクランプ（共有定数を参照）
   * clamp-constants.jsが読み込まれている場合はそちらを使用、
   * なければフォールバック値を使用
   *
   * @param {number} value - クランプする値
   * @param {string} key - CLAMP_RANGESのキー
   * @param {number} fallbackMin - フォールバック最小値
   * @param {number} fallbackMax - フォールバック最大値
   * @returns {number}
   */
  clampByKey(value, key, fallbackMin = 0, fallbackMax = 100) {
    if (typeof window.clampValue === 'function') {
      return window.clampValue(value, key);
    }
    // フォールバック
    return this.clamp(value, fallbackMin, fallbackMax);
  }

  /**
   * HTML エスケープ
   * @param {string} str
   * @returns {string}
   */
  escapeHtml(str) {
    if (typeof str !== 'string') return '';
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  /**
   * DOM要素を生成するヘルパー
   * @param {string} tag - タグ名
   * @param {object} options - オプション
   * @param {string} options.className - クラス名
   * @param {string} options.textContent - テキスト内容
   * @param {object} options.style - インラインスタイル
   * @param {object} options.attrs - 属性
   * @returns {HTMLElement}
   */
  createElement(tag, options = {}) {
    const el = document.createElement(tag);

    if (options.className) {
      el.className = options.className;
    }

    if (options.textContent !== undefined) {
      el.textContent = options.textContent;
    }

    if (options.style) {
      Object.assign(el.style, options.style);
    }

    if (options.attrs) {
      Object.entries(options.attrs).forEach(([key, value]) => {
        el.setAttribute(key, value);
      });
    }

    return el;
  }
}

// グローバルに公開
window.BaseComponent = BaseComponent;
