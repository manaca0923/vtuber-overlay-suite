/**
 * PromoPanel - 告知パネルコンポーネント
 *
 * 配置: right.bottom
 * 機能: 複数の告知をサイクル表示
 *
 * style設定:
 *   - items: Array<{text: string, icon?: string}> (告知アイテム)
 *
 * rules設定:
 *   - cycleSec: number (サイクル間隔秒、デフォルト: 30、範囲: 10-120)
 *   - showSec: number (各アイテム表示秒、デフォルト: 6、範囲: 3-15)
 *
 * update()で受け取るデータ:
 *   - items: Array<{text: string, icon?: string}>
 */
class PromoPanel extends BaseComponent {
  constructor(config) {
    super(config);

    // 表示アイテム
    this.items = this.style.items || [];

    // サイクル設定（クランプ適用）
    // cycleSec: 将来的にサイクル間の休憩時間として使用予定（現在未使用）
    this.cycleSec = this.clampByKey(this.rules.cycleSec || 30, 'cycleSec', 10, 120);
    // showSec: 各アイテムの表示間隔
    this.showSec = this.clampByKey(this.rules.showSec || 6, 'showSec', 3, 15);
    this.originalShowSec = this.showSec; // 縮退モードからの復元用

    this.currentIndex = 0;
    this.cycleTimerId = null;
    this.showTimeoutId = null;
  }

  /**
   * イベントハンドラ（density対応）
   * @param {string} eventType
   * @param {object} payload
   */
  onEvent(eventType, payload) {
    if (eventType === 'density:high') {
      this.applyDegradedMode(payload);
    } else if (eventType === 'density:normal') {
      this.restoreNormalMode(payload);
    }
  }

  /**
   * 縮退モードを適用
   * @param {object} settings
   */
  applyDegradedMode(settings) {
    if (settings.showSec) {
      this.showSec = this.clampByKey(settings.showSec, 'showSec', 3, 15);
      this.startCycle(); // 間隔変更時は再スタート
    }
  }

  /**
   * 通常モードに復元
   * @param {object} settings
   */
  restoreNormalMode(settings) {
    if (settings.showSec) {
      this.showSec = this.clampByKey(settings.showSec, 'showSec', 3, 15);
    } else {
      this.showSec = this.originalShowSec;
    }
    this.startCycle(); // 間隔変更時は再スタート
  }

  render() {
    const container = this.createElement('div', {
      className: 'promo-panel panel',
    });

    this.contentEl = this.createElement('div', {
      className: 'promo-content dt-text-shadow',
    });

    container.appendChild(this.contentEl);

    return container;
  }

  afterMount() {
    // デフォルトアイテムがなければスタブデータを使用
    if (this.items.length === 0) {
      this.items = [
        { text: 'チャンネル登録よろしくお願いします!' },
        { text: '次回配信: 毎週土曜 21:00〜' },
        { text: 'SNSフォロー歓迎!' },
      ];
    }

    this.showItem(0);
    this.startCycle();
  }

  /**
   * サイクル開始
   */
  startCycle() {
    if (this.cycleTimerId) {
      this.clearInterval(this.cycleTimerId);
    }

    if (this.items.length <= 1) return;

    this.cycleTimerId = this.setInterval(() => {
      this.nextItem();
    }, this.showSec * 1000);
  }

  /**
   * 次のアイテムへ
   */
  nextItem() {
    this.showItem(this.currentIndex + 1);
  }

  /**
   * 指定アイテムを表示
   * @param {number} index
   */
  showItem(index) {
    if (this.items.length === 0) {
      this.contentEl.textContent = '';
      return;
    }

    this.currentIndex = index % this.items.length;
    const item = this.items[this.currentIndex];

    // フェードアウト
    this.contentEl.style.opacity = '0';

    // 既存のタイムアウトをクリア
    if (this.showTimeoutId) {
      clearTimeout(this.showTimeoutId);
    }

    this.showTimeoutId = setTimeout(() => {
      // コンテンツ更新
      this.contentEl.innerHTML = '';

      if (item.icon) {
        const iconEl = this.createElement('span', {
          className: 'promo-icon',
          textContent: item.icon,
          style: { marginRight: '8px' },
        });
        this.contentEl.appendChild(iconEl);
      }

      // textContentはHTMLを解釈しないため、escapeHtmlは不要
      const textEl = this.createElement('span', {
        className: 'promo-text',
        textContent: item.text || '',
      });
      this.contentEl.appendChild(textEl);

      // フェードイン
      this.contentEl.style.opacity = '1';
    }, 300);
  }

  update(data) {
    if (data.items && Array.isArray(data.items)) {
      this.items = data.items;
      this.currentIndex = 0;
      this.showItem(0);
      this.startCycle();
    }

    if (data.cycleSec !== undefined) {
      this.cycleSec = this.clampByKey(data.cycleSec, 'cycleSec', 10, 120);
    }

    if (data.showSec !== undefined) {
      this.showSec = this.clampByKey(data.showSec, 'showSec', 3, 15);
      this.startCycle(); // 間隔変更時は再スタート
    }
  }

  destroy() {
    if (this.cycleTimerId) {
      this.clearInterval(this.cycleTimerId);
      this.cycleTimerId = null;
    }
    if (this.showTimeoutId) {
      clearTimeout(this.showTimeoutId);
      this.showTimeoutId = null;
    }
    super.destroy();
  }
}

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('PromoPanel', PromoPanel);
}
