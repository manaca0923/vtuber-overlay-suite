/**
 * KPIBlock - KPI数値表示コンポーネント
 *
 * 配置: right.lowerLeft
 * 機能: 視聴者数、高評価数などのKPI数値を表示
 *
 * style設定:
 *   - mainLabel: string (主数字のラベル、デフォルト: '視聴者')
 *   - subLabel: string (副数字のラベル、デフォルト: '')
 *   - showSub: boolean (副数字表示、デフォルト: true)
 *
 * rules設定:
 *   - updateThrottle: number (更新間隔ms、デフォルト: 2000)
 *
 * update()で受け取るデータ:
 *   - main: number (主数字)
 *   - label: string (ラベル)
 *   - sub: number (副数字)
 *   - subLabel: string (副ラベル)
 */
class KPIBlock extends BaseComponent {
  constructor(config) {
    super(config);
    this.mainLabel = this.style.mainLabel || '視聴者';
    this.subLabel = this.style.subLabel || '';
    this.showSub = this.style.showSub !== false;

    // 更新スロットリング（2秒デフォルト）
    this.updateThrottle = this.clamp(this.rules.updateThrottle || 2000, 1000, 10000);
    this.lastUpdate = 0;

    // 現在値
    this.mainValue = null;
    this.subValue = null;
  }

  render() {
    const container = this.createElement('div', {
      className: 'kpi-block panel',
    });

    this.mainEl = this.createElement('div', {
      className: 'kpi-main dt-text-shadow',
      textContent: '--',
    });

    this.labelEl = this.createElement('div', {
      className: 'kpi-label',
      textContent: this.mainLabel,
    });

    container.appendChild(this.mainEl);
    container.appendChild(this.labelEl);

    if (this.showSub) {
      this.subEl = this.createElement('div', {
        className: 'kpi-sub dt-text-shadow',
        textContent: '',
      });
      container.appendChild(this.subEl);

      if (this.subLabel) {
        this.subLabelEl = this.createElement('div', {
          className: 'kpi-sub-label',
          textContent: this.subLabel,
        });
        container.appendChild(this.subLabelEl);
      }
    }

    return container;
  }

  afterMount() {
    // モックデータでデモ表示（初期値）
    this.update({
      main: Math.floor(Math.random() * 500) + 100,
      sub: Math.floor(Math.random() * 50) + 10,
    });
  }

  update(data) {
    const now = Date.now();

    // スロットリング（短時間での連続更新を防止）
    if (now - this.lastUpdate < this.updateThrottle) {
      return;
    }
    this.lastUpdate = now;

    // 主数字の更新
    if (data.main !== undefined) {
      this.mainValue = data.main;
      this.mainEl.textContent = this.formatNumber(data.main);
      this.animateUpdate(this.mainEl);
    }

    // ラベルの更新
    if (data.label !== undefined) {
      this.labelEl.textContent = data.label;
    }

    // 副数字の更新
    if (data.sub !== undefined && this.subEl) {
      this.subValue = data.sub;
      this.subEl.textContent = this.formatNumber(data.sub);
      this.animateUpdate(this.subEl);
    }

    // 副ラベルの更新
    if (data.subLabel !== undefined && this.subLabelEl) {
      this.subLabelEl.textContent = data.subLabel;
    }
  }

  /**
   * 数値フォーマット
   * @param {number} num
   * @returns {string}
   */
  formatNumber(num) {
    if (typeof num !== 'number' || isNaN(num)) return '--';

    if (num >= 10000) {
      return (num / 10000).toFixed(1) + '万';
    }
    if (num >= 1000) {
      return num.toLocaleString();
    }
    return num.toString();
  }

  /**
   * 更新時のアニメーション
   * @param {HTMLElement} el
   */
  animateUpdate(el) {
    el.classList.remove('kpi-updated');
    // 強制リフロー
    void el.offsetWidth;
    el.classList.add('kpi-updated');
  }
}

// 更新アニメーションのCSS（動的追加）
(function () {
  if (document.getElementById('kpi-block-styles')) return;

  const style = document.createElement('style');
  style.id = 'kpi-block-styles';
  style.textContent = `
    @keyframes kpi-pulse {
      0% { transform: scale(1); }
      50% { transform: scale(1.05); }
      100% { transform: scale(1); }
    }
    .kpi-updated {
      animation: kpi-pulse 0.3s ease-out;
    }
  `;
  document.head.appendChild(style);
})();

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('KPIBlock', KPIBlock);
}
