/**
 * SuperchatCard - スーパーチャット専用表示コンポーネント
 *
 * 配置: left.lower
 * 機能: スパチャを専用ウィジェットで目立たせて表示
 *
 * WebSocket連携:
 *   - superchat:add: スパチャを表示キューに追加
 *   - superchat:remove: 表示完了したスパチャを削除
 *
 * style設定:
 *   - maxDisplay: number (同時表示最大数、デフォルト: 1)
 */

// デバッグモード: URLパラメータ ?debug=true で有効化
const SUPERCHAT_DEBUG = new URLSearchParams(window.location.search).get('debug') === 'true';

class SuperchatCard extends BaseComponent {
  constructor(config) {
    super(config);
    this.maxDisplay = this.style.maxDisplay || 1;
    // 表示中のスパチャ（IDをキーにしたMap）
    this.displayedSuperchats = new Map();
    // 待機中のスパチャキュー
    this.queue = [];
  }

  render() {
    const container = this.createElement('div', {
      className: 'superchat-card-container',
    });
    return container;
  }

  /**
   * スパチャを追加（superchat:addイベントで呼び出される）
   * @param {object} data - SuperchatPayload
   */
  addSuperchat(data) {
    if (!data || !data.id) {
      console.warn('[SuperchatCard] Invalid superchat data:', data);
      return;
    }

    if (SUPERCHAT_DEBUG) console.log('[SuperchatCard] addSuperchat:', {
      id: data.id,
      amount: data.amount,
      maxDisplay: this.maxDisplay,
      currentDisplayed: this.displayedSuperchats.size,
      queueLength: this.queue.length,
    });

    // 既に表示中または待機中の場合はスキップ
    if (this.displayedSuperchats.has(data.id)) return;
    if (this.queue.some((s) => s.id === data.id)) return;

    // 表示枠に空きがあれば即表示、なければキューに追加
    if (this.displayedSuperchats.size < this.maxDisplay) {
      if (SUPERCHAT_DEBUG) console.log('[SuperchatCard] Displaying immediately');
      this._displaySuperchat(data);
    } else {
      if (SUPERCHAT_DEBUG) console.log('[SuperchatCard] Adding to queue');
      this.queue.push(data);
    }
  }

  /**
   * スパチャを削除（superchat:removeイベントで呼び出される）
   * @param {string} id - スパチャID
   */
  removeSuperchat(id) {
    const el = this.displayedSuperchats.get(id);
    if (el) {
      this._animateOut(el, () => {
        el.remove();
        this.displayedSuperchats.delete(id);
        // キューから次のスパチャを表示
        this._processQueue();
      });
    }
  }

  /**
   * キューから次のスパチャを表示
   */
  _processQueue() {
    if (
      this.queue.length > 0 &&
      this.displayedSuperchats.size < this.maxDisplay
    ) {
      const next = this.queue.shift();
      this._displaySuperchat(next);
    }
  }

  /**
   * スパチャを表示
   * @param {object} data - SuperchatPayload
   */
  _displaySuperchat(data) {
    const card = this._createCard(data);
    this.element.appendChild(card);
    this.displayedSuperchats.set(data.id, card);

    // スライドインアニメーション
    requestAnimationFrame(() => {
      card.classList.add('visible');
    });
  }

  /**
   * スパチャカードを生成
   * @param {object} data - SuperchatPayload
   * @returns {HTMLElement}
   */
  _createCard(data) {
    const card = this.createElement('div', {
      className: `superchat-card tier-${data.tier}`,
    });
    card.dataset.id = data.id;

    // 背景色をTierに応じて設定
    card.style.setProperty('--sc-bg-color', this._getTierColor(data.tier));

    // ヘッダー（アイコン + 名前 + 金額）
    const header = this.createElement('div', {
      className: 'superchat-header',
    });

    const avatar = this.createElement('img', {
      className: 'superchat-avatar',
      attrs: {
        src: data.authorImageUrl || '',
        alt: data.authorName || '',
      },
    });
    avatar.onerror = () => {
      avatar.style.display = 'none';
    };

    const nameAmount = this.createElement('div', {
      className: 'superchat-name-amount',
    });

    const name = this.createElement('span', {
      className: 'superchat-author',
      textContent: data.authorName || 'Anonymous',
    });

    const amount = this.createElement('span', {
      className: 'superchat-amount',
      textContent: data.amount || '',
    });

    nameAmount.appendChild(name);
    nameAmount.appendChild(amount);
    header.appendChild(avatar);
    header.appendChild(nameAmount);

    // メッセージ本文
    const message = this.createElement('div', {
      className: 'superchat-message',
      textContent: data.message || '',
    });

    card.appendChild(header);
    if (data.message) {
      card.appendChild(message);
    }

    return card;
  }

  /**
   * Tierに応じた背景色を取得
   * @param {number} tier - 1-7
   * @returns {string} 色コード
   */
  _getTierColor(tier) {
    // YouTube公式のスパチャカラー
    const tierColors = {
      1: '#1565C0', // Blue - ¥100-199
      2: '#00B8D4', // Cyan - ¥200-499
      3: '#00BFA5', // Teal - ¥500-999
      4: '#FFB300', // Yellow - ¥1,000-1,999
      5: '#F57C00', // Orange - ¥2,000-4,999
      6: '#E91E63', // Pink - ¥5,000-9,999
      7: '#E62117', // Red - ¥10,000+
    };
    return tierColors[tier] || tierColors[1];
  }

  /**
   * フェードアウトアニメーション
   * @param {HTMLElement} el
   * @param {Function} callback
   */
  _animateOut(el, callback) {
    el.classList.remove('visible');
    el.classList.add('removing');

    // 二重実行防止フラグ
    let callbackExecuted = false;
    const safeCallback = () => {
      if (callbackExecuted) return;
      callbackExecuted = true;
      callback();
    };

    el.addEventListener(
      'animationend',
      () => {
        safeCallback();
      },
      { once: true }
    );

    // フォールバック（アニメーションが発火しなかった場合）
    setTimeout(() => {
      if (el.parentNode) {
        safeCallback();
      }
    }, 500);
  }

  update(data) {
    // update()はWebSocketハンドラから直接呼ばれる可能性があるため
    // addSuperchatへの転送用
    if (data && data.id) {
      this.addSuperchat(data);
    }
  }

  /**
   * 設定を更新
   * @param {object} settings - SuperchatSettings
   */
  updateSettings(settings) {
    if (SUPERCHAT_DEBUG) console.log('[SuperchatCard] updateSettings called:', settings);
    if (!settings) return;

    if (typeof settings.maxDisplay === 'number' && settings.maxDisplay >= 1) {
      if (SUPERCHAT_DEBUG) console.log('[SuperchatCard] Updating maxDisplay:', this.maxDisplay, '->', settings.maxDisplay);
      this.maxDisplay = settings.maxDisplay;
      // 設定変更後、キューに空きができた場合は処理
      this._processQueue();
    }
  }

  destroy() {
    this.displayedSuperchats.clear();
    this.queue = [];
    super.destroy();
  }
}

// CSSを動的に追加
(function () {
  if (document.getElementById('superchat-card-styles')) return;

  const style = document.createElement('style');
  style.id = 'superchat-card-styles';
  style.textContent = `
    .superchat-card-container {
      display: flex;
      flex-direction: column;
      gap: 8px;
      width: 100%;
    }

    .superchat-card {
      background: var(--sc-bg-color, #1565C0);
      border-radius: 8px;
      padding: 12px;
      color: white;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
      opacity: 0;
      transform: translateX(-100%);
      transition: opacity 0.3s ease, transform 0.3s ease;
    }

    .superchat-card.visible {
      opacity: 1;
      transform: translateX(0);
    }

    .superchat-card.removing {
      animation: superchat-slide-out 0.3s ease forwards;
    }

    @keyframes superchat-slide-out {
      from {
        opacity: 1;
        transform: translateX(0);
      }
      to {
        opacity: 0;
        transform: translateX(-100%);
      }
    }

    .superchat-header {
      display: flex;
      align-items: center;
      gap: 10px;
      margin-bottom: 8px;
    }

    .superchat-avatar {
      width: 40px;
      height: 40px;
      border-radius: 50%;
      object-fit: cover;
      border: 2px solid rgba(255, 255, 255, 0.5);
    }

    .superchat-name-amount {
      display: flex;
      flex-direction: column;
      gap: 2px;
      flex: 1;
      min-width: 0;
    }

    .superchat-author {
      font-size: 14px;
      font-weight: bold;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .superchat-amount {
      font-size: 18px;
      font-weight: bold;
    }

    .superchat-message {
      font-size: 14px;
      line-height: 1.4;
      word-break: break-word;
      background: rgba(0, 0, 0, 0.15);
      padding: 8px;
      border-radius: 4px;
      max-height: 80px;
      overflow: hidden;
    }

    /* Tier別の微調整 */
    .superchat-card.tier-4,
    .superchat-card.tier-5 {
      color: #000;
    }

    .superchat-card.tier-4 .superchat-avatar,
    .superchat-card.tier-5 .superchat-avatar {
      border-color: rgba(0, 0, 0, 0.3);
    }

    .superchat-card.tier-4 .superchat-message,
    .superchat-card.tier-5 .superchat-message {
      background: rgba(0, 0, 0, 0.1);
    }
  `;
  document.head.appendChild(style);
})();

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('SuperchatCard', SuperchatCard);
}
