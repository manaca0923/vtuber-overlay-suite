/**
 * MainAvatarStage - 中央ステージコンポーネント
 *
 * 配置: center.full
 * 機能: アバター表示用のプレースホルダー
 *       実際のアバターはOBSの別レイヤーで表示することを想定
 *
 * style設定:
 *   - showFrame: boolean (フレーム表示、デフォルト: false)
 *   - frameColor: string (フレーム色、デフォルト: 透明)
 *   - backgroundColor: string (背景色、デフォルト: 透明)
 *
 * 注意:
 *   - このコンポーネントは基本的に透明なプレースホルダー
 *   - 左右カラムへの侵食を防ぐためのスペーサーとして機能
 */
class MainAvatarStage extends BaseComponent {
  constructor(config) {
    super(config);
    this.showFrame = this.style.showFrame || false;
    this.frameColor = this.style.frameColor || 'transparent';
    this.backgroundColor = this.style.backgroundColor || 'transparent';
  }

  render() {
    const container = this.createElement('div', {
      className: 'main-avatar-stage',
      style: {
        width: '100%',
        height: '100%',
        backgroundColor: this.backgroundColor,
      },
    });

    if (this.showFrame) {
      container.style.border = `2px solid ${this.frameColor}`;
      container.style.borderRadius = 'var(--radius, 14px)';
    }

    return container;
  }

  update(data) {
    if (data.showFrame !== undefined) {
      this.showFrame = data.showFrame;
      if (data.showFrame) {
        this.element.style.border = `2px solid ${this.frameColor}`;
        this.element.style.borderRadius = 'var(--radius, 14px)';
      } else {
        this.element.style.border = 'none';
      }
    }

    if (data.frameColor !== undefined) {
      this.frameColor = data.frameColor;
      if (this.showFrame) {
        this.element.style.borderColor = data.frameColor;
      }
    }

    if (data.backgroundColor !== undefined) {
      this.backgroundColor = data.backgroundColor;
      this.element.style.backgroundColor = data.backgroundColor;
    }
  }
}

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('MainAvatarStage', MainAvatarStage);
}
