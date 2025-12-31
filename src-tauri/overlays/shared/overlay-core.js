/**
 * オーバーレイ共通コアモジュール
 * combined.html と combined-v2.html で共有するWebSocket/設定管理ロジック
 */

(function() {
'use strict';

// =============================================================================
// 定数
// =============================================================================

// デバッグモード: URLパラメータ ?debug=true で有効化
const DEBUG = new URLSearchParams(window.location.search).get('debug') === 'true';

const WS_URL = 'ws://localhost:19801/ws';
const API_BASE_URL = 'http://localhost:19800/api';
const SETTINGS_FETCH_TIMEOUT = 3000;
const MAX_RECONNECT_DELAY = 30000;
const INITIAL_RECONNECT_DELAY = 1000;

// issues/020: postMessage用のtrusted origins定数
const TRUSTED_ORIGINS = [
  'http://localhost:5173',  // Vite dev server
  'http://localhost:1420',  // Tauri dev
  'tauri://localhost'       // Tauri production
];

// =============================================================================
// ユーティリティ関数
// =============================================================================

/**
 * タイムアウト値をバリデートし、無効な場合はデフォルト値を返す
 * @param {*} timeout - タイムアウト値
 * @param {number} defaultValue - デフォルト値
 * @returns {number} - 有効なタイムアウト値
 */
function validateTimeout(timeout, defaultValue = SETTINGS_FETCH_TIMEOUT) {
  return Number.isFinite(timeout) && timeout > 0 ? timeout : defaultValue;
}

// =============================================================================
// WebSocket接続マネージャー
// =============================================================================

/**
 * WebSocket接続を管理するクラス
 * 自動再接続、指数バックオフ、bfcache対応を提供
 */
class WebSocketManager {
  constructor(options = {}) {
    this.url = options.url || WS_URL;
    this.ws = null;
    this.reconnectDelay = INITIAL_RECONNECT_DELAY;
    this.reconnectTimerId = null;
    this.isShuttingDown = false;

    // コールバック
    this.onOpen = options.onOpen || (() => {});
    this.onMessage = options.onMessage || (() => {});
    this.onError = options.onError || ((error) => console.error('WebSocket error:', error));
    this.onClose = options.onClose || (() => {});
  }

  /**
   * WebSocket接続を開始
   */
  connect() {
    if (this.isShuttingDown) return;

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      if (DEBUG) console.log('WebSocket connected');
      this.reconnectDelay = INITIAL_RECONNECT_DELAY;
      this.onOpen();
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.onMessage(data);
      } catch (e) {
        console.error('WebSocket message handling error:', e);
      }
    };

    this.ws.onerror = (error) => {
      this.onError(error);
    };

    this.ws.onclose = () => {
      this.onClose();

      // シャットダウン中は再接続しない
      if (this.isShuttingDown) {
        if (DEBUG) console.log('WebSocket closed (shutdown)');
        return;
      }

      if (DEBUG) console.log('WebSocket closed, reconnecting...');
      this.reconnectTimerId = setTimeout(() => {
        this.reconnectTimerId = null;
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, MAX_RECONNECT_DELAY);
        this.connect();
      }, this.reconnectDelay);
    };
  }

  /**
   * 接続をクリーンアップ
   */
  cleanup() {
    this.isShuttingDown = true;

    if (this.reconnectTimerId) {
      clearTimeout(this.reconnectTimerId);
      this.reconnectTimerId = null;
    }

    if (this.ws) {
      // oncloseハンドラを無効化して不要なログ/再接続を防止
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * bfcache復元時に再初期化
   */
  reinitialize() {
    this.isShuttingDown = false;
    this.reconnectDelay = INITIAL_RECONNECT_DELAY;

    // ペンディング中の再接続タイマーをクリア（二重接続防止）
    if (this.reconnectTimerId) {
      clearTimeout(this.reconnectTimerId);
      this.reconnectTimerId = null;
    }

    // 既存の接続がある場合はスキップ（二重接続防止）
    if (this.ws && (this.ws.readyState === WebSocket.CONNECTING || this.ws.readyState === WebSocket.OPEN)) {
      if (DEBUG) console.log('WebSocket already connected or connecting, skipping reinitialize');
      return;
    }

    this.connect();
  }
}

// =============================================================================
// 設定フェッチャー
// =============================================================================

/**
 * API設定をフェッチして適用するクラス
 */
class SettingsFetcher {
  constructor(options = {}) {
    this.apiBaseUrl = options.apiBaseUrl || API_BASE_URL;
    this.timeout = options.timeout || SETTINGS_FETCH_TIMEOUT;
    this.settingsVersion = 0;
    this.fetchInFlight = false;
    this.fetchSucceeded = false;

    // コールバック
    this.onSettingsApply = options.onSettingsApply || (() => {});
  }

  /**
   * 設定をフェッチして適用
   */
  async fetchAndApply() {
    if (this.fetchInFlight) return;
    this.fetchInFlight = true;
    const requestVersion = this.settingsVersion;

    const controller = new AbortController();
    const timeoutMs = validateTimeout(this.timeout);
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const response = await fetch(`${this.apiBaseUrl}/overlay/settings`, {
        signal: controller.signal
      });

      if (response.ok) {
        const settings = await response.json();
        // 新しいリクエストが来ていたらスキップ
        if (this.settingsVersion > requestVersion) return;
        this.onSettingsApply(settings);
        this.fetchSucceeded = true;
      }
    } catch (_e) {
      if (DEBUG) console.log('Settings API not available, using defaults');
    } finally {
      clearTimeout(timeoutId);
      this.fetchInFlight = false;
    }
  }

  /**
   * 設定バージョンをインクリメント（WebSocket経由の更新時）
   */
  incrementVersion() {
    this.settingsVersion++;
  }

  /**
   * フェッチ済みかどうか
   */
  hasFetched() {
    return this.fetchSucceeded;
  }

  /**
   * 状態をリセット（bfcache復元時に使用）
   * 次回のfetchAndApplyで必ず再取得される
   */
  reset() {
    this.fetchSucceeded = false;
    this.fetchInFlight = false;
  }
}

// =============================================================================
// URLパラメータパーサー
// =============================================================================

/**
 * URLパラメータをパースしてCSS変数やフラグに適用するユーティリティ
 */
class UrlParamsParser {
  constructor(options = {}) {
    this.validators = options.validators || {};
  }

  /**
   * URLパラメータを取得
   */
  getParams() {
    return new URLSearchParams(window.location.search);
  }

  /**
   * CSS変数を適用
   * @param {string} paramName - URLパラメータ名
   * @param {string} cssVar - CSS変数名
   * @param {Function} validator - バリデーション関数
   * @param {Function} transform - 変換関数（オプション）
   */
  applyCssVar(paramName, cssVar, validator, transform = (v) => v) {
    const params = this.getParams();
    const value = params.get(paramName);
    if (value && validator(value)) {
      document.documentElement.style.setProperty(cssVar, transform(value));
      return true;
    }
    return false;
  }

  /**
   * ブールパラメータを取得
   * @param {string} paramName - URLパラメータ名
   * @param {boolean} defaultValue - デフォルト値
   */
  getBoolParam(paramName, defaultValue = true) {
    const params = this.getParams();
    const value = params.get(paramName);
    if (value !== null) {
      return value === 'true';
    }
    return defaultValue;
  }

  /**
   * 文字列パラメータを取得（バリデーション付き）
   * @param {string} paramName - URLパラメータ名
   * @param {Array} validValues - 有効な値のリスト
   * @param {string} defaultValue - デフォルト値
   */
  getStringParam(paramName, validValues, defaultValue = null) {
    const params = this.getParams();
    const value = params.get(paramName);
    if (value && validValues.includes(value)) {
      return value;
    }
    return defaultValue;
  }
}

// =============================================================================
// PostMessageハンドラ
// =============================================================================

/**
 * プレビュー専用のpostMessageハンドラ
 * issues/010: WebSocketManagerと同様のcleanup()/reinitialize()パターンを適用
 */
class PostMessageHandler {
  constructor(options = {}) {
    this.trustedOrigins = options.trustedOrigins || TRUSTED_ORIGINS;
    this.onSettingsUpdate = options.onSettingsUpdate || (() => {});
    this.onSuperchatAdd = options.onSuperchatAdd || (() => {});
    this.onSuperchatRemove = options.onSuperchatRemove || (() => {});
    this._handleMessage = this._handleMessage.bind(this);
    this.isActive = true;
    window.addEventListener('message', this._handleMessage);
  }

  _handleMessage(event) {
    // issues/010: シャットダウン中は処理しない
    if (!this.isActive) return;

    // issues/002: origin検証（セキュリティ）
    if (!this.trustedOrigins.includes(event.origin)) {
      // 開発時のデバッグ用（プロダクションでは削除可能）
      // console.warn('[PostMessage] Untrusted origin:', event.origin);
      return;
    }

    const data = event.data;

    // issues/013: 防御的プログラミング - オブジェクト型のガード
    if (!data || typeof data !== 'object' || Array.isArray(data)) {
      return;
    }

    if (data.type === 'preview:settings:update') {
      // issues/013: payloadの存在チェック
      if (!data.payload || typeof data.payload !== 'object' || Array.isArray(data.payload)) {
        console.warn('[PostMessage] Invalid payload');
        return;
      }
      this.onSettingsUpdate(data.payload);
    } else if (data.type === 'preview:superchat:add') {
      // スパチャ追加（プレビュー用）
      if (!data.payload || typeof data.payload !== 'object' || Array.isArray(data.payload)) {
        console.warn('[PostMessage] Invalid superchat payload');
        return;
      }
      this.onSuperchatAdd(data.payload);
    } else if (data.type === 'preview:superchat:remove') {
      // スパチャ削除（プレビュー用）
      if (!data.payload || typeof data.payload !== 'object' || Array.isArray(data.payload)) {
        console.warn('[PostMessage] Invalid superchat remove payload');
        return;
      }
      this.onSuperchatRemove(data.payload.id);
    }
  }

  /**
   * クリーンアップ（ページ遷移時）
   * issues/010: WebSocketManagerと同様のcleanup()パターン
   */
  cleanup() {
    this.isActive = false;
    window.removeEventListener('message', this._handleMessage);
  }

  /**
   * 再初期化（bfcache復元時）
   * issues/010: bfcache復元時の再有効化
   */
  reinitialize() {
    if (this.isActive) return; // 既にアクティブならスキップ
    this.isActive = true;
    window.addEventListener('message', this._handleMessage);
  }
}

// =============================================================================
// セットリスト更新ヘルパー
// =============================================================================

/**
 * セットリスト表示を更新するヘルパー関数
 * @param {Object} data - セットリストデータ { songs, currentIndex }
 * @param {Object} elements - DOM要素 { prevEl, currentEl, nextEl }
 * @param {Function} onArtistVisibilityUpdate - アーティスト表示更新コールバック
 */
function updateSetlistDisplay(data, elements, onArtistVisibilityUpdate = () => {}) {
  const songs = data.songs || [];
  const currentIndex = data.currentIndex ?? -1;
  const { prevEl, currentEl, nextEl } = elements;

  // 前の曲（要素が存在する場合のみ更新）
  if (prevEl) {
    if (currentIndex > 0) {
      const prevSong = songs[currentIndex - 1];
      const prevNumber = prevEl.querySelector('.song-number');
      const prevTitle = prevEl.querySelector('.song-title');
      const prevArtist = prevEl.querySelector('.song-artist');
      if (prevNumber) prevNumber.textContent = currentIndex;
      if (prevTitle) prevTitle.textContent = prevSong.title;
      if (prevArtist) prevArtist.textContent = prevSong.artist || '';
      prevEl.style.display = 'flex';
    } else {
      prevEl.style.display = 'none';
    }
  }

  // 現在の曲（要素が存在する場合のみ更新）
  if (currentEl) {
    const currentNumber = currentEl.querySelector('.song-number');
    const currentTitle = currentEl.querySelector('.song-title');
    const currentArtist = currentEl.querySelector('.song-artist');
    if (currentIndex >= 0 && currentIndex < songs.length) {
      const currentSong = songs[currentIndex];
      if (currentNumber) currentNumber.textContent = currentIndex + 1;
      if (currentTitle) currentTitle.textContent = currentSong.title;
      if (currentArtist) currentArtist.textContent = currentSong.artist || '';
      currentEl.style.display = 'flex';
    } else {
      if (currentNumber) currentNumber.textContent = '-';
      if (currentTitle) currentTitle.textContent = '待機中...';
      if (currentArtist) currentArtist.textContent = '';
    }
  }

  // 次の曲（要素が存在する場合のみ更新）
  if (nextEl) {
    if (currentIndex >= 0 && currentIndex < songs.length - 1) {
      const nextSong = songs[currentIndex + 1];
      const nextNumber = nextEl.querySelector('.song-number');
      const nextTitle = nextEl.querySelector('.song-title');
      const nextArtist = nextEl.querySelector('.song-artist');
      if (nextNumber) nextNumber.textContent = currentIndex + 2;
      if (nextTitle) nextTitle.textContent = nextSong.title;
      if (nextArtist) nextArtist.textContent = nextSong.artist || '';
      nextEl.style.display = 'flex';
    } else {
      nextEl.style.display = 'none';
    }
  }

  onArtistVisibilityUpdate();
}

/**
 * 最新セットリストをフェッチ
 * @param {string} apiBaseUrl - APIベースURL
 * @param {Function} onUpdate - 更新コールバック
 * @param {number} timeout - タイムアウト時間（ミリ秒）
 */
async function fetchLatestSetlist(apiBaseUrl = API_BASE_URL, onUpdate, timeout = SETTINGS_FETCH_TIMEOUT) {
  const timeoutMs = validateTimeout(timeout);
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(`${apiBaseUrl}/setlist/latest`, {
      signal: controller.signal
    });
    if (response.ok) {
      const data = await response.json();
      if (data.songs) {
        onUpdate({
          songs: data.songs.map(s => ({ title: s.title, artist: s.artist })),
          currentIndex: data.setlist.currentIndex
        });
      }
    }
  } catch (_e) {
    if (DEBUG) console.log('Failed to fetch setlist');
  } finally {
    clearTimeout(timeoutId);
  }
}

// =============================================================================
// bfcache対応ヘルパー
// =============================================================================

/**
 * bfcache対応のイベントリスナーを設定
 * @param {Function} onCleanup - クリーンアップ時のコールバック
 * @param {Function} onRestore - 復元時のコールバック
 */
function setupBfcacheHandlers(onCleanup, onRestore) {
  // ページ非表示時
  window.addEventListener('pagehide', (event) => {
    if (!event.persisted) {
      // bfcacheに保存されない場合はクリーンアップ
      onCleanup();
    }
  });

  // ページ表示時（bfcacheからの復元時）
  window.addEventListener('pageshow', (event) => {
    if (event.persisted) {
      // bfcacheから復元された場合は再初期化
      onRestore();
    }
  });
}

// =============================================================================
// エクスポート
// =============================================================================

window.OverlayCore = {
  // 定数
  DEBUG,
  WS_URL,
  API_BASE_URL,
  SETTINGS_FETCH_TIMEOUT,
  TRUSTED_ORIGINS,

  // クラス
  WebSocketManager,
  SettingsFetcher,
  UrlParamsParser,
  PostMessageHandler,

  // ヘルパー関数
  updateSetlistDisplay,
  fetchLatestSetlist,
  setupBfcacheHandlers
};

})();
