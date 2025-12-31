/**
 * WeatherWidget - 天気情報コンポーネント
 *
 * 配置: left.topBelow
 * 機能: 天気情報を表示（Open-Meteo API連携済み）
 *
 * バックエンド連携:
 *   - src-tauri/src/weather/mod.rs - Open-Meteo API連携
 *   - src-tauri/src/commands/weather.rs - Tauriコマンド
 *   - WebSocket: weather:update メッセージで単一都市更新
 *   - WebSocket: weather:multi-update メッセージでマルチシティ更新
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
 *
 * updateMulti()で受け取るデータ:
 *   - cities: CityWeatherData[]
 *   - rotationIntervalSec: number
 */
class WeatherWidget extends BaseComponent {
  constructor(config) {
    super(config);
    // スタブ用デフォルト値
    this.icon = this.style.icon || '☀️';
    this.temp = this.style.temp ?? 25;
    this.description = this.style.description || '晴れ';
    this.location = this.style.location || '';

    // マルチシティモード用
    this.multiMode = false;
    this.cities = [];
    this.currentIndex = 0;
    this.rotationInterval = 5000; // デフォルト5秒
    this.rotationTimer = null;
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

  /**
   * 単一都市モードの更新
   * weather:update WebSocketメッセージで呼び出される
   */
  update(data) {
    // マルチシティモードを無効化
    this._stopRotation();
    this.multiMode = false;

    this._updateDisplay(data);
  }

  /**
   * マルチシティモードの更新
   * weather:multi-update WebSocketメッセージで呼び出される
   * @param {Object} data - { cities: CityWeatherData[], rotationIntervalSec: number }
   */
  updateMulti(data) {
    console.log('[WeatherWidget] updateMulti called:', JSON.stringify(data));

    this.cities = data.cities || [];
    this.rotationInterval = (data.rotationIntervalSec || 5) * 1000;
    this.multiMode = true;

    console.log('[WeatherWidget] cities count:', this.cities.length, 'interval:', this.rotationInterval);

    // 既存のタイマーをクリア
    this._stopRotation();

    if (this.cities.length === 0) {
      console.warn('[WeatherWidget] No cities to display');
      return;
    }

    // 最初の都市を表示
    this.currentIndex = 0;
    this._displayCity(this.cities[0]);
    console.log('[WeatherWidget] Displayed first city:', this.cities[0]?.cityName);

    // ローテーション開始（2都市以上の場合）
    if (this.cities.length > 1) {
      console.log('[WeatherWidget] Starting rotation timer');
      this.rotationTimer = setInterval(() => {
        this._rotateNext();
      }, this.rotationInterval);
    }
  }

  /**
   * 次の都市にローテーション
   */
  _rotateNext() {
    this.currentIndex = (this.currentIndex + 1) % this.cities.length;
    const city = this.cities[this.currentIndex];
    console.log('[WeatherWidget] Rotating to city:', this.currentIndex, city?.cityName);
    this._displayCityWithFade(city);
  }

  /**
   * 都市の天気を表示（フェードアニメーション付き）
   * @param {Object} cityData
   */
  _displayCityWithFade(cityData) {
    // BaseComponentでは this.element を使用
    if (!this.element) {
      console.warn('[WeatherWidget] element is not initialized');
      this._displayCity(cityData);
      return;
    }

    // フェードアウト
    this.element.classList.add('weather-fade-out');

    setTimeout(() => {
      // データ更新
      this._displayCity(cityData);

      // フェードイン
      this.element.classList.remove('weather-fade-out');
      this.element.classList.add('weather-fade-in');

      // フェードインクラスを削除
      setTimeout(() => {
        this.element.classList.remove('weather-fade-in');
      }, 300);
    }, 200);
  }

  /**
   * 都市の天気を即座に表示
   * @param {Object} cityData
   */
  _displayCity(cityData) {
    this._updateDisplay({
      icon: cityData.icon,
      temp: cityData.temp,
      description: cityData.description,
      location: cityData.cityName, // 表示名を使用
    });
  }

  /**
   * 表示を更新（共通処理）
   * @param {Object} data
   */
  _updateDisplay(data) {
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
   * ローテーションを停止
   */
  _stopRotation() {
    if (this.rotationTimer) {
      clearInterval(this.rotationTimer);
      this.rotationTimer = null;
    }
  }

  /**
   * コンポーネント破棄時
   */
  destroy() {
    this._stopRotation();
    if (super.destroy) {
      super.destroy();
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
