/**
 * ChannelBadge - チャンネルバッジコンポーネント
 *
 * 配置: right.top
 * 機能: チャンネル名やライブラベルを表示
 *
 * style設定:
 *   - label: string (ラベルテキスト、デフォルト: 'LIVE')
 *   - iconUrl: string (アイコン画像URL)
 *   - showIcon: boolean (アイコン表示、デフォルト: false)
 *   - blinking: boolean (点滅アニメーション、デフォルト: false)
 *
 * update()で受け取るデータ:
 *   - label: string
 *   - iconUrl: string
 */
class ChannelBadge extends BaseComponent {
  constructor(config) {
    super(config);
    this.label = this.style.label || 'LIVE';
    this.iconUrl = this.validateUrl(this.style.iconUrl) || '';
    this.showIcon = this.style.showIcon || false;
    this.blinking = this.style.blinking || false;
  }

  /**
   * URLが安全なスキームか検証
   * @param {string} url
   * @returns {string} 安全なURLまたは空文字列
   */
  validateUrl(url) {
    if (!url || typeof url !== 'string') return '';
    try {
      const parsed = new URL(url, window.location.href);
      // http, https, data スキームのみ許可
      if (['http:', 'https:', 'data:'].includes(parsed.protocol)) {
        return url;
      }
      console.warn('ChannelBadge: 無効なURLスキーム:', parsed.protocol);
      return '';
    } catch (e) {
      // 相対パスの場合は許可
      if (url.startsWith('/') || url.startsWith('./') || url.startsWith('../')) {
        return url;
      }
      console.warn('ChannelBadge: 無効なURL:', url);
      return '';
    }
  }

  render() {
    const container = this.createElement('div', {
      className: 'channel-badge panel dt-text-shadow dt-ellipsis',
    });

    if (this.showIcon && this.iconUrl) {
      this.iconEl = this.createElement('img', {
        className: 'channel-badge-icon',
        attrs: {
          src: this.iconUrl,
          alt: '',
        },
      });
      container.appendChild(this.iconEl);
    }

    this.labelEl = this.createElement('span', {
      className: 'channel-badge-label',
      textContent: this.label,
    });
    container.appendChild(this.labelEl);

    if (this.blinking) {
      this.startBlinking();
    }

    return container;
  }

  startBlinking() {
    // CSS アニメーションでの点滅
    if (this.labelEl) {
      this.labelEl.style.animation = 'channel-badge-blink 1s ease-in-out infinite';
    }
  }

  stopBlinking() {
    if (this.labelEl) {
      this.labelEl.style.animation = '';
    }
  }

  update(data) {
    if (data.label !== undefined) {
      this.label = data.label;
      this.labelEl.textContent = data.label;
    }

    if (data.iconUrl !== undefined && this.iconEl) {
      const validatedUrl = this.validateUrl(data.iconUrl);
      this.iconUrl = validatedUrl;
      this.iconEl.src = validatedUrl;
    }

    if (data.blinking !== undefined) {
      this.blinking = data.blinking;
      if (data.blinking) {
        this.startBlinking();
      } else {
        this.stopBlinking();
      }
    }
  }

  destroy() {
    this.stopBlinking();
    super.destroy();
  }
}

// 点滅アニメーションのCSS（動的追加）
(function () {
  if (document.getElementById('channel-badge-styles')) return;

  const style = document.createElement('style');
  style.id = 'channel-badge-styles';
  style.textContent = `
    @keyframes channel-badge-blink {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.5; }
    }
  `;
  document.head.appendChild(style);
})();

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('ChannelBadge', ChannelBadge);
}
