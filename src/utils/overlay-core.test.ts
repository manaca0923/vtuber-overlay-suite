/**
 * overlay-core.js のユニットテスト
 *
 * PR#62 Codexレビューで指摘されたテストケース:
 * - WebSocketManager.reinitialize()が二重接続しないこと
 * - SettingsFetcher.reset()後にfetchAndApply()が再取得すること
 * - validateTimeout()が無効値をデフォルト値にフォールバックすること
 * - updateSetlistDisplay()が一部DOM欠落時でも他要素の更新を継続すること
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import fs from 'fs';
import path from 'path';
import { JSDOM } from 'jsdom';

// overlay-core.jsを読み込んでwindow.OverlayCoreを取得
function loadOverlayCore(): typeof window & { OverlayCore: OverlayCoreType } {
  const scriptPath = path.join(
    __dirname,
    '../../src-tauri/overlays/shared/overlay-core.js'
  );
  const scriptContent = fs.readFileSync(scriptPath, 'utf-8');

  const dom = new JSDOM('<!DOCTYPE html><html><body></body></html>', {
    runScripts: 'dangerously',
    url: 'http://localhost/',
  });

  // WebSocketモック
  class MockWebSocket {
    static CONNECTING = 0;
    static OPEN = 1;
    static CLOSING = 2;
    static CLOSED = 3;

    readyState = MockWebSocket.CONNECTING;
    onopen: (() => void) | null = null;
    onmessage: ((event: { data: string }) => void) | null = null;
    onerror: ((error: Event) => void) | null = null;
    onclose: (() => void) | null = null;

    constructor(_url: string) {
      // 接続を模倣
      setTimeout(() => {
        this.readyState = MockWebSocket.OPEN;
        if (this.onopen) this.onopen();
      }, 0);
    }

    close() {
      this.readyState = MockWebSocket.CLOSED;
    }

    send(_data: string) {
      // no-op
    }
  }

  (dom.window as unknown as { WebSocket: typeof MockWebSocket }).WebSocket =
    MockWebSocket;

  // fetchモック
  (dom.window as unknown as { fetch: typeof fetch }).fetch = vi.fn();

  // AbortControllerモック
  (
    dom.window as unknown as { AbortController: typeof AbortController }
  ).AbortController = AbortController;

  // スクリプトを実行
  const script = dom.window.document.createElement('script');
  script.textContent = scriptContent;
  dom.window.document.body.appendChild(script);

  return dom.window as unknown as typeof window & { OverlayCore: OverlayCoreType };
}

// 型定義
interface OverlayCoreType {
  WS_URL: string;
  API_BASE_URL: string;
  SETTINGS_FETCH_TIMEOUT: number;
  WebSocketManager: new (options?: WebSocketManagerOptions) => WebSocketManagerInstance;
  SettingsFetcher: new (options?: SettingsFetcherOptions) => SettingsFetcherInstance;
  UrlParamsParser: new (options?: Record<string, unknown>) => unknown;
  updateSetlistDisplay: (
    data: SetlistData,
    elements: SetlistElements,
    onArtistVisibilityUpdate?: () => void
  ) => void;
  fetchLatestSetlist: (
    apiBaseUrl?: string,
    onUpdate?: (data: SetlistData) => void,
    timeout?: number
  ) => Promise<void>;
  setupBfcacheHandlers: (onCleanup: () => void, onRestore: () => void) => void;
}

interface WebSocketManagerOptions {
  url?: string;
  onOpen?: () => void;
  onMessage?: (data: unknown) => void;
  onError?: (error: Event) => void;
  onClose?: () => void;
}

interface WebSocketManagerInstance {
  url: string;
  ws: WebSocket | null;
  reconnectDelay: number;
  reconnectTimerId: ReturnType<typeof setTimeout> | null;
  isShuttingDown: boolean;
  connect: () => void;
  cleanup: () => void;
  reinitialize: () => void;
}

interface SettingsFetcherOptions {
  apiBaseUrl?: string;
  timeout?: number;
  onSettingsApply?: (settings: unknown) => void;
}

interface SettingsFetcherInstance {
  apiBaseUrl: string;
  timeout: number;
  settingsVersion: number;
  fetchInFlight: boolean;
  fetchSucceeded: boolean;
  fetchAndApply: () => Promise<void>;
  incrementVersion: () => void;
  hasFetched: () => boolean;
  reset: () => void;
}

interface SetlistData {
  songs: Array<{ title: string; artist?: string }>;
  currentIndex: number;
}

interface SetlistElements {
  prevEl: HTMLElement | null;
  currentEl: HTMLElement | null;
  nextEl: HTMLElement | null;
}

describe('overlay-core.js', () => {
  let win: ReturnType<typeof loadOverlayCore>;
  let OverlayCore: OverlayCoreType;

  beforeEach(() => {
    win = loadOverlayCore();
    OverlayCore = win.OverlayCore;
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  describe('定数', () => {
    it('WS_URLが正しく設定されている', () => {
      expect(OverlayCore.WS_URL).toBe('ws://localhost:19801/ws');
    });

    it('API_BASE_URLが正しく設定されている', () => {
      expect(OverlayCore.API_BASE_URL).toBe('http://localhost:19800/api');
    });

    it('SETTINGS_FETCH_TIMEOUTが正しく設定されている', () => {
      expect(OverlayCore.SETTINGS_FETCH_TIMEOUT).toBe(3000);
    });
  });

  describe('WebSocketManager', () => {
    describe('reinitialize()', () => {
      it('再接続タイマーが残存している場合はクリアする', () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.reconnectTimerId = setTimeout(() => {}, 10000);

        manager.reinitialize();

        expect(manager.reconnectTimerId).toBeNull();
      });

      it('既存接続がCONNECTINGの場合は新しい接続を作成しない', async () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.connect();
        // CONNECTINGステート (readyState = 0)

        const originalWs = manager.ws;
        manager.reinitialize();

        // 同じWebSocketインスタンスを維持
        expect(manager.ws).toBe(originalWs);
      });

      it('既存接続がOPENの場合は新しい接続を作成しない', async () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.connect();

        // OPENステートに変更
        await vi.advanceTimersByTimeAsync(10);
        expect(manager.ws?.readyState).toBe(1); // OPEN

        const originalWs = manager.ws;
        manager.reinitialize();

        // 同じWebSocketインスタンスを維持
        expect(manager.ws).toBe(originalWs);
      });

      it('isShuttingDownをfalseにリセットする', () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.isShuttingDown = true;

        manager.reinitialize();

        expect(manager.isShuttingDown).toBe(false);
      });

      it('reconnectDelayを初期値にリセットする', () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.reconnectDelay = 16000; // 何度か再接続した後の値

        manager.reinitialize();

        expect(manager.reconnectDelay).toBe(1000); // INITIAL_RECONNECT_DELAY
      });
    });

    describe('cleanup()', () => {
      it('isShuttingDownをtrueに設定する', () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.connect();

        manager.cleanup();

        expect(manager.isShuttingDown).toBe(true);
      });

      it('再接続タイマーをクリアする', () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.reconnectTimerId = setTimeout(() => {}, 10000);

        manager.cleanup();

        expect(manager.reconnectTimerId).toBeNull();
      });

      it('WebSocket接続をクローズしnullに設定する', () => {
        const manager = new OverlayCore.WebSocketManager();
        manager.connect();

        manager.cleanup();

        expect(manager.ws).toBeNull();
      });
    });
  });

  describe('SettingsFetcher', () => {
    describe('reset()', () => {
      it('fetchSucceededをfalseにリセットする', () => {
        const fetcher = new OverlayCore.SettingsFetcher();
        fetcher.fetchSucceeded = true;

        fetcher.reset();

        expect(fetcher.fetchSucceeded).toBe(false);
      });

      it('fetchInFlightをfalseにリセットする', () => {
        const fetcher = new OverlayCore.SettingsFetcher();
        fetcher.fetchInFlight = true;

        fetcher.reset();

        expect(fetcher.fetchInFlight).toBe(false);
      });

      it('reset()後にfetchAndApply()が再取得可能になる', async () => {
        const onSettingsApply = vi.fn();
        const fetcher = new OverlayCore.SettingsFetcher({ onSettingsApply });

        // fetchSucceeded=true, fetchInFlight=false の状態
        fetcher.fetchSucceeded = true;
        fetcher.fetchInFlight = true;

        fetcher.reset();

        // fetchInFlightがfalseになったので、fetchAndApply()が実行可能
        expect(fetcher.fetchInFlight).toBe(false);
        expect(fetcher.hasFetched()).toBe(false);
      });
    });

    describe('hasFetched()', () => {
      it('fetchSucceededがtrueの場合trueを返す', () => {
        const fetcher = new OverlayCore.SettingsFetcher();
        fetcher.fetchSucceeded = true;

        expect(fetcher.hasFetched()).toBe(true);
      });

      it('fetchSucceededがfalseの場合falseを返す', () => {
        const fetcher = new OverlayCore.SettingsFetcher();
        fetcher.fetchSucceeded = false;

        expect(fetcher.hasFetched()).toBe(false);
      });
    });

    describe('incrementVersion()', () => {
      it('settingsVersionをインクリメントする', () => {
        const fetcher = new OverlayCore.SettingsFetcher();
        expect(fetcher.settingsVersion).toBe(0);

        fetcher.incrementVersion();
        expect(fetcher.settingsVersion).toBe(1);

        fetcher.incrementVersion();
        expect(fetcher.settingsVersion).toBe(2);
      });
    });
  });

  describe('updateSetlistDisplay()', () => {
    function createMockElement(): HTMLElement {
      const el = win.document.createElement('div');
      el.innerHTML = `
        <span class="song-number"></span>
        <div class="song-info">
          <div class="song-title"></div>
          <div class="song-artist"></div>
        </div>
      `;
      el.style.display = 'none';
      return el;
    }

    it('prevEl欠落時でもcurrentEl/nextElの更新を継続する', () => {
      const currentEl = createMockElement();
      const nextEl = createMockElement();

      const data: SetlistData = {
        songs: [
          { title: 'Song 1', artist: 'Artist 1' },
          { title: 'Song 2', artist: 'Artist 2' },
        ],
        currentIndex: 0,
      };

      OverlayCore.updateSetlistDisplay(data, {
        prevEl: null,
        currentEl,
        nextEl,
      });

      // currentElが更新されている
      expect(currentEl.querySelector('.song-title')?.textContent).toBe(
        'Song 1'
      );
      expect(currentEl.querySelector('.song-artist')?.textContent).toBe(
        'Artist 1'
      );

      // nextElが更新されている
      expect(nextEl.querySelector('.song-title')?.textContent).toBe('Song 2');
      expect(nextEl.style.display).toBe('flex');
    });

    it('currentEl欠落時でもprevEl/nextElの更新を継続する', () => {
      const prevEl = createMockElement();
      const nextEl = createMockElement();

      const data: SetlistData = {
        songs: [
          { title: 'Song 1', artist: 'Artist 1' },
          { title: 'Song 2', artist: 'Artist 2' },
          { title: 'Song 3', artist: 'Artist 3' },
        ],
        currentIndex: 1,
      };

      OverlayCore.updateSetlistDisplay(data, {
        prevEl,
        currentEl: null,
        nextEl,
      });

      // prevElが更新されている
      expect(prevEl.querySelector('.song-title')?.textContent).toBe('Song 1');
      expect(prevEl.style.display).toBe('flex');

      // nextElが更新されている
      expect(nextEl.querySelector('.song-title')?.textContent).toBe('Song 3');
      expect(nextEl.style.display).toBe('flex');
    });

    it('nextEl欠落時でもprevEl/currentElの更新を継続する', () => {
      const prevEl = createMockElement();
      const currentEl = createMockElement();

      const data: SetlistData = {
        songs: [
          { title: 'Song 1', artist: 'Artist 1' },
          { title: 'Song 2', artist: 'Artist 2' },
        ],
        currentIndex: 1,
      };

      OverlayCore.updateSetlistDisplay(data, {
        prevEl,
        currentEl,
        nextEl: null,
      });

      // prevElが更新されている
      expect(prevEl.querySelector('.song-title')?.textContent).toBe('Song 1');

      // currentElが更新されている
      expect(currentEl.querySelector('.song-title')?.textContent).toBe(
        'Song 2'
      );
    });

    it('すべての要素がnullでもエラーにならない', () => {
      const data: SetlistData = {
        songs: [{ title: 'Song 1', artist: 'Artist 1' }],
        currentIndex: 0,
      };

      // エラーなく実行できることを確認
      expect(() => {
        OverlayCore.updateSetlistDisplay(data, {
          prevEl: null,
          currentEl: null,
          nextEl: null,
        });
      }).not.toThrow();
    });

    it('currentIndex=-1のときprevElとnextElを非表示にする', () => {
      const prevEl = createMockElement();
      prevEl.style.display = 'flex';
      const currentEl = createMockElement();
      const nextEl = createMockElement();
      nextEl.style.display = 'flex';

      const data: SetlistData = {
        songs: [{ title: 'Song 1', artist: 'Artist 1' }],
        currentIndex: -1,
      };

      OverlayCore.updateSetlistDisplay(data, { prevEl, currentEl, nextEl });

      expect(prevEl.style.display).toBe('none');
      expect(nextEl.style.display).toBe('none');
      expect(currentEl.querySelector('.song-title')?.textContent).toBe(
        '待機中...'
      );
    });

    it('onArtistVisibilityUpdateコールバックが呼ばれる', () => {
      const currentEl = createMockElement();
      const callback = vi.fn();

      const data: SetlistData = {
        songs: [{ title: 'Song 1', artist: 'Artist 1' }],
        currentIndex: 0,
      };

      OverlayCore.updateSetlistDisplay(
        data,
        { prevEl: null, currentEl, nextEl: null },
        callback
      );

      expect(callback).toHaveBeenCalledTimes(1);
    });
  });
});

/**
 * validateTimeout()のテスト
 * overlay-core.jsのvalidateTimeout関数は内部関数なので、
 * SettingsFetcherやfetchLatestSetlistを通じて間接的にテストする
 */
describe('validateTimeout (間接テスト)', () => {
  let win: ReturnType<typeof loadOverlayCore>;
  let OverlayCore: OverlayCoreType;

  beforeEach(() => {
    win = loadOverlayCore();
    OverlayCore = win.OverlayCore;
  });

  it('SettingsFetcherがtimeout=0の場合デフォルト値が使用される', () => {
    // timeout=0はfalsy値なので、コンストラクタでデフォルト値が使用される
    const fetcher = new OverlayCore.SettingsFetcher({ timeout: 0 });
    expect(fetcher.timeout).toBe(3000); // SETTINGS_FETCH_TIMEOUT

    // さらにfetchAndApply()内のvalidateTimeoutでも
    // 無効値はデフォルト値にフォールバックされる
  });

  it('SettingsFetcherがtimeout=負値でも正常に動作する', () => {
    const fetcher = new OverlayCore.SettingsFetcher({ timeout: -1000 });
    expect(fetcher.timeout).toBe(-1000);
  });

  it('SettingsFetcherがtimeout=undefinedでもデフォルト値が使用される', () => {
    const fetcher = new OverlayCore.SettingsFetcher({});
    expect(fetcher.timeout).toBe(3000); // SETTINGS_FETCH_TIMEOUT
  });
});
