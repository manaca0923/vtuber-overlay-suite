/**
 * BrandBlock - ロゴ表示コンポーネント
 *
 * 配置: left.bottom
 * 機能: ブランドロゴや注意書きを表示
 *
 * style設定:
 *   - logoUrl: string (ロゴ画像URL)
 *   - text: string (テキスト、logoUrlがない場合に表示)
 *   - maxHeight: number (最大高さ、デフォルト: 60px)
 *   - opacity: number (透明度、デフォルト: 1)
 *
 * update()で受け取るデータ:
 *   - logoUrl: string
 *   - text: string
 */
class BrandBlock extends BaseComponent {
  constructor(config) {
    super(config);
    this.logoUrl = this.style.logoUrl || '';
    this.text = this.style.text || '';
    this.maxHeight = this.style.maxHeight || 60;
    this.opacity = this.style.opacity ?? 1;
  }

  render() {
    const container = this.createElement('div', {
      className: 'brand-block',
      style: { opacity: this.opacity.toString() },
    });

    if (this.logoUrl) {
      this.logoEl = this.createElement('img', {
        className: 'brand-logo',
        attrs: {
          src: this.logoUrl,
          alt: 'Logo',
        },
        style: {
          maxWidth: '100%',
          maxHeight: `${this.maxHeight}px`,
          objectFit: 'contain',
        },
      });
      // 画像読み込みエラー時のフォールバック
      this.logoEl.onerror = () => {
        this.logoEl.style.display = 'none';
        // フォールバックテキストがあれば表示
        if (this.text && !this.textEl) {
          this.textEl = this.createElement('div', {
            className: 'brand-text dt-text-shadow',
            textContent: this.text,
          });
          container.appendChild(this.textEl);
        }
      };
      container.appendChild(this.logoEl);
    } else if (this.text) {
      this.textEl = this.createElement('div', {
        className: 'brand-text dt-text-shadow',
        textContent: this.text,
      });
      container.appendChild(this.textEl);
    }

    return container;
  }

  update(data) {
    if (data.logoUrl !== undefined) {
      this.logoUrl = data.logoUrl;
      if (this.logoEl) {
        this.logoEl.src = data.logoUrl;
      } else if (data.logoUrl) {
        // テキストからロゴへの切り替え
        this.element.innerHTML = '';
        this.textEl = null;
        this.logoEl = this.createElement('img', {
          className: 'brand-logo',
          attrs: {
            src: data.logoUrl,
            alt: 'Logo',
          },
          style: {
            maxWidth: '100%',
            maxHeight: `${this.maxHeight}px`,
            objectFit: 'contain',
          },
        });
        // 画像読み込みエラー時のフォールバック
        this.logoEl.onerror = () => {
          this.logoEl.style.display = 'none';
          if (this.text && !this.textEl) {
            this.textEl = this.createElement('div', {
              className: 'brand-text dt-text-shadow',
              textContent: this.text,
            });
            this.element.appendChild(this.textEl);
          }
        };
        this.element.appendChild(this.logoEl);
      }
    }

    if (data.text !== undefined && !this.logoUrl) {
      this.text = data.text;
      if (this.textEl) {
        this.textEl.textContent = data.text;
      }
    }
  }

  applyStyle() {
    if (this.element && this.opacity !== 1) {
      this.element.style.opacity = this.opacity.toString();
    }
  }
}

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('BrandBlock', BrandBlock);
}
