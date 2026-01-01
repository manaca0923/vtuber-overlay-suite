// =============================================================================
// 定数定義
// =============================================================================

/** デフォルトのローテーション間隔（ミリ秒） */
const DEFAULT_ROTATION_INTERVAL_MS = 5000;

/** デフォルトのローテーション間隔（秒） - updateMulti引数用 */
const DEFAULT_ROTATION_INTERVAL_SEC = 5;

/** フェードアウトアニメーション時間（ミリ秒） */
const FADE_OUT_DURATION_MS = 200;

/** フェードインアニメーション時間（ミリ秒） */
const FADE_IN_DURATION_MS = 300;

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
    this.rotationInterval = DEFAULT_ROTATION_INTERVAL_MS;
    this.rotationTimer = null;

    // visibilitychange対応（メモリリーク防止）
    this._boundVisibilityHandler = this._handleVisibilityChange.bind(this);
    document.addEventListener('visibilitychange', this._boundVisibilityHandler);
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
    // 型安全性チェック: 配列以外のtruthyな値（オブジェクト等）への防御
    this.cities = Array.isArray(data.cities) ? data.cities : [];
    this.rotationInterval = (data.rotationIntervalSec || DEFAULT_ROTATION_INTERVAL_SEC) * 1000;
    this.multiMode = true;

    // 既存のタイマーをクリア
    this._stopRotation();

    if (this.cities.length === 0) {
      return;
    }

    // 最初の都市を表示
    this.currentIndex = 0;
    this._displayCity(this.cities[0]);

    // ローテーション開始（2都市以上の場合）
    if (this.cities.length > 1) {
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
    this._displayCityWithFade(city);
  }

  /**
   * 都市の天気を表示（フェードアニメーション付き）
   * @param {Object} cityData
   */
  _displayCityWithFade(cityData) {
    // BaseComponentでは this.element を使用
    if (!this.element) {
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
      }, FADE_IN_DURATION_MS);
    }, FADE_OUT_DURATION_MS);
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
   * ローテーションを再開（マルチシティモードで2都市以上の場合のみ）
   */
  _resumeRotation() {
    if (this.multiMode && this.cities.length > 1 && !this.rotationTimer) {
      this.rotationTimer = setInterval(() => {
        this._rotateNext();
      }, this.rotationInterval);
    }
  }

  /**
   * ページの可視状態変更ハンドラ（メモリリーク防止）
   * タブがバックグラウンドに行った際にタイマーを停止し、
   * フォアグラウンドに戻った際に再開する
   */
  _handleVisibilityChange() {
    if (document.hidden) {
      this._stopRotation();
    } else {
      this._resumeRotation();
    }
  }

  /**
   * コンポーネント破棄時
   */
  destroy() {
    this._stopRotation();
    // visibilitychangeリスナーを解除
    if (this._boundVisibilityHandler) {
      document.removeEventListener('visibilitychange', this._boundVisibilityHandler);
      this._boundVisibilityHandler = null;
    }
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
