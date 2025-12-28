/**
 * update-batcher.js のユニットテスト
 *
 * PR#54で指摘されたUpdateBatcherのテストケース:
 * - queue()が同一タイプを上書きすること
 * - forceFlush()が即時フラッシュすること
 * - clear()がキューをクリアすること
 * - setBatchInterval()が範囲内にクランプすること
 * - destroy()がタイマーを停止すること
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { JSDOM } from 'jsdom';

// スクリプトパスを解決
function resolveScriptPath(): string {
  const relativePath = 'src-tauri/overlays/shared/update-batcher.js';

  try {
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);
    const rootDir = path.resolve(__dirname, '../..');
    const scriptPath = path.join(rootDir, relativePath);
    if (fs.existsSync(scriptPath)) {
      return scriptPath;
    }
  } catch {
    // fileURLToPathが失敗した場合はフォールバック
  }

  return path.join(process.cwd(), relativePath);
}

// UpdateBatcherを読み込んでwindow.UpdateBatcherを取得
function loadUpdateBatcher(): typeof window & { UpdateBatcher: UpdateBatcherClass } {
  const scriptPath = resolveScriptPath();
  const scriptContent = fs.readFileSync(scriptPath, 'utf-8');

  const dom = new JSDOM('<!DOCTYPE html><html><body></body></html>', {
    runScripts: 'dangerously',
    url: 'http://localhost/',
  });

  // performanceモック（read-onlyプロパティなのでObject.definePropertyを使用）
  Object.defineProperty(dom.window, 'performance', {
    value: {
      now: () => Date.now(),
    },
    writable: true,
    configurable: true,
  });

  // requestAnimationFrameモック
  (dom.window as unknown as { requestAnimationFrame: (cb: () => void) => number }).requestAnimationFrame = (cb) => {
    return setTimeout(cb, 0) as unknown as number;
  };

  // ComponentRegistryモック
  const mockBroadcast = vi.fn();
  (dom.window as unknown as { ComponentRegistry: { broadcast: typeof mockBroadcast } }).ComponentRegistry = {
    broadcast: mockBroadcast,
  };

  // スクリプトを実行
  const script = dom.window.document.createElement('script');
  script.textContent = scriptContent;
  dom.window.document.body.appendChild(script);

  return dom.window as unknown as typeof window & { UpdateBatcher: UpdateBatcherClass };
}

// 型定義
interface UpdateBatcherClass {
  new (options?: { batchInterval?: number }): UpdateBatcherInstance;
}

interface UpdateBatcherInstance {
  batchInterval: number;
  pendingUpdates: Map<string, unknown>;
  isScheduled: boolean;
  timeoutId: ReturnType<typeof setTimeout> | null;
  queue: (type: string, data: unknown) => void;
  scheduleFlush: () => void;
  flush: () => void;
  forceFlush: () => void;
  clear: () => void;
  getPendingCount: () => number;
  setBatchInterval: (ms: number) => void;
  destroy: () => void;
  isDestroyed: () => boolean;
}

describe('UpdateBatcher', () => {
  let win: ReturnType<typeof loadUpdateBatcher>;
  let UpdateBatcher: UpdateBatcherClass;

  beforeEach(() => {
    win = loadUpdateBatcher();
    UpdateBatcher = win.UpdateBatcher;
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('デフォルトのbatchIntervalは150ms', () => {
      const batcher = new UpdateBatcher();
      expect(batcher.batchInterval).toBe(150);
    });

    it('オプションでbatchIntervalを設定できる', () => {
      const batcher = new UpdateBatcher({ batchInterval: 200 });
      expect(batcher.batchInterval).toBe(200);
    });

    it('初期状態でpendingUpdatesは空', () => {
      const batcher = new UpdateBatcher();
      expect(batcher.getPendingCount()).toBe(0);
    });
  });

  describe('queue()', () => {
    it('更新をキューに追加する', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      expect(batcher.getPendingCount()).toBe(1);
    });

    it('同一タイプは最新のみ保持（上書き）', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      batcher.queue('KPIBlock', { main: 5678 });
      expect(batcher.getPendingCount()).toBe(1);
      expect(batcher.pendingUpdates.get('KPIBlock')).toEqual({ main: 5678 });
    });

    it('異なるタイプは別々に保持', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      batcher.queue('QueueList', { items: [] });
      expect(batcher.getPendingCount()).toBe(2);
    });
  });

  describe('forceFlush()', () => {
    it('タイマーを待たずに即時フラッシュする', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      expect(batcher.getPendingCount()).toBe(1);

      batcher.forceFlush();
      expect(batcher.getPendingCount()).toBe(0);
    });

    it('isScheduledをfalseにリセットする', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      expect(batcher.isScheduled).toBe(true);

      batcher.forceFlush();
      expect(batcher.isScheduled).toBe(false);
    });
  });

  describe('clear()', () => {
    it('キューをクリアする', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      batcher.queue('QueueList', { items: [] });

      batcher.clear();
      expect(batcher.getPendingCount()).toBe(0);
    });

    it('isScheduledをfalseにリセットする', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });

      batcher.clear();
      expect(batcher.isScheduled).toBe(false);
    });

    it('タイムアウトをクリアする', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      expect(batcher.timeoutId).not.toBeNull();

      batcher.clear();
      expect(batcher.timeoutId).toBeNull();
    });
  });

  describe('setBatchInterval()', () => {
    it('バッチ間隔を変更できる', () => {
      const batcher = new UpdateBatcher();
      batcher.setBatchInterval(200);
      expect(batcher.batchInterval).toBe(200);
    });

    it('50ms未満は50msにクランプ', () => {
      const batcher = new UpdateBatcher();
      batcher.setBatchInterval(10);
      expect(batcher.batchInterval).toBe(50);
    });

    it('500ms超過は500msにクランプ', () => {
      const batcher = new UpdateBatcher();
      batcher.setBatchInterval(1000);
      expect(batcher.batchInterval).toBe(500);
    });
  });

  describe('destroy()', () => {
    it('タイマーを停止しリソースを解放する', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });

      batcher.destroy();
      expect(batcher.isDestroyed()).toBe(true);
      expect(batcher.getPendingCount()).toBe(0);
    });
  });

  describe('isDestroyed()', () => {
    it('destroyされていない場合はfalse', () => {
      const batcher = new UpdateBatcher();
      expect(batcher.isDestroyed()).toBe(false);
    });

    it('destroy後はtrue', () => {
      const batcher = new UpdateBatcher();
      batcher.destroy();
      expect(batcher.isDestroyed()).toBe(true);
    });
  });
});
