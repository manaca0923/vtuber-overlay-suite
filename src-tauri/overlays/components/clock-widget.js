/**
 * ClockWidget - 時刻/日付表示コンポーネント
 *
 * 配置: left.top
 * 機能: 現在時刻と日付を表示（毎秒更新）
 *
 * style設定:
 *   - showSeconds: boolean (秒表示、デフォルト: true)
 *   - showDate: boolean (日付表示、デフォルト: true)
 *   - dateFormat: 'long' | 'short' (日付フォーマット、デフォルト: 'long')
 */
class ClockWidget extends BaseComponent {
  constructor(config) {
    super(config);
    this.showSeconds = this.style.showSeconds !== false;
    this.showDate = this.style.showDate !== false;
    this.dateFormat = this.style.dateFormat || 'long';
  }

  render() {
    const container = this.createElement('div', {
      className: 'clock-widget dt-text-shadow',
    });

    this.timeEl = this.createElement('div', {
      className: 'clock-time',
    });

    this.dateEl = this.createElement('div', {
      className: 'clock-date',
    });

    container.appendChild(this.timeEl);
    if (this.showDate) {
      container.appendChild(this.dateEl);
    }

    return container;
  }

  afterMount() {
    this.updateTime();
    this.setInterval(() => this.updateTime(), 1000);
  }

  updateTime() {
    const now = new Date();

    // 時刻フォーマット（hour12: falseで24時間表示を保証）
    const timeOptions = {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    };
    if (this.showSeconds) {
      timeOptions.second = '2-digit';
    }
    this.timeEl.textContent = now.toLocaleTimeString('ja-JP', timeOptions);

    // 日付フォーマット
    if (this.showDate && this.dateEl) {
      const dateOptions =
        this.dateFormat === 'short'
          ? { month: 'numeric', day: 'numeric', weekday: 'short' }
          : { year: 'numeric', month: 'long', day: 'numeric', weekday: 'short' };
      this.dateEl.textContent = now.toLocaleDateString('ja-JP', dateOptions);
    }
  }

  update(data) {
    // 外部データによる更新（通常は使用しない）
    if (data.showSeconds !== undefined) {
      this.showSeconds = data.showSeconds;
    }
    if (data.showDate !== undefined) {
      this.showDate = data.showDate;
      if (this.dateEl) {
        this.dateEl.style.display = data.showDate ? '' : 'none';
      }
    }
  }
}

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('ClockWidget', ClockWidget);
}
