/**
 * WeatherWidget - 天気情報コンポーネント
 *
 * 配置: left.topBelow
 * 機能: 天気情報を表示（Open-Meteo API連携済み）
 *
 * バックエンド連携:
 *   - src-tauri/src/weather/mod.rs - Open-Meteo API連携
 *   - src-tauri/src/commands/weather.rs - Tauriコマンド
 *   - WebSocket: weather:update メッセージで更新
 *
 * style設定:
 *   - icon: string (天気アイコン、デフォルト: '☀️')
 *   - temp: number (気温、デフォルト: 25)
 *   - description: string (説明、デフォルト: '晴れ')
 *   - location: string (地域名、デフォルト: '')
 *
 * update()で受け取るデータ:
 *   - icon: string
 *   - temp: number
 *   - description: string
 *   - location: string
 */
class WeatherWidget extends BaseComponent {
  constructor(config) {
    super(config);
    // スタブ用デフォルト値
    this.icon = this.style.icon || '☀️';
    this.temp = this.style.temp ?? 25;
    this.description = this.style.description || '晴れ';
    this.location = this.style.location || '';
  }

  render() {
    const container = this.createElement('div', {
      className: 'weather-widget panel dt-text-shadow',
    });

    this.iconEl = this.createElement('span', {
      className: 'weather-icon',
      textContent: this.icon,
    });

    this.tempEl = this.createElement('span', {
      className: 'weather-temp',
      textContent: `${this.temp}°C`,
    });

    this.descEl = this.createElement('span', {
      className: 'weather-desc',
      textContent: this.description,
    });

    container.appendChild(this.iconEl);
    container.appendChild(this.tempEl);
    container.appendChild(this.descEl);

    // locationElは常に作成（後からupdate()で設定される可能性があるため）
    this.locationEl = this.createElement('span', {
      className: 'weather-location',
      textContent: this.location,
      style: { opacity: '0.7', marginLeft: '8px' },
    });
    container.appendChild(this.locationEl);

    return container;
  }

  update(data) {
    // weather:update WebSocketメッセージで呼び出される
    if (data.icon !== undefined) {
      this.icon = data.icon;
      this.iconEl.textContent = data.icon;
    }
    if (data.temp !== undefined) {
      this.temp = data.temp;
      this.tempEl.textContent = `${data.temp}°C`;
    }
    if (data.description !== undefined) {
      this.description = data.description;
      this.descEl.textContent = data.description;
    }
    if (data.location !== undefined) {
      // nullやundefinedが渡された場合は空文字にフォールバック
      this.location = data.location ?? '';
      if (this.locationEl) {
        this.locationEl.textContent = this.location;
      }
    }
  }

  /**
   * 天気アイコンマッピング（将来のAPI連携用）
   */
  static WEATHER_ICONS = {
    clear: '☀️',
    sunny: '☀️',
    cloudy: '☁️',
    partlyCloudy: '⛅',
    rain: '🌧️',
    heavyRain: '⛈️',
    snow: '❄️',
    thunder: '⚡',
    fog: '🌫️',
    wind: '💨',
  };

  /**
   * 天気コードからアイコンを取得（将来のAPI連携用）
   * @param {string} code
   * @returns {string}
   */
  static getIconForCode(code) {
    return WeatherWidget.WEATHER_ICONS[code] || '🌡️';
  }
}

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('WeatherWidget', WeatherWidget);
}
