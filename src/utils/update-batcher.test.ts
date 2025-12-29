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
import {
  loadScriptContent,
  createTestDOM,
  mockPerformance,
  mockRequestAnimationFrame,
  executeScript,
} from './test-helpers';

// UpdateBatcherを読み込んでwindow.UpdateBatcherを取得
function loadUpdateBatcher(): {
  win: typeof window & { UpdateBatcher: UpdateBatcherClass };
  mockBroadcast: ReturnType<typeof vi.fn>;
} {
  const scriptContent = loadScriptContent('src-tauri/overlays/shared/update-batcher.js');
  const dom = createTestDOM();

  // performanceモック（read-onlyプロパティなのでObject.definePropertyを使用）
  mockPerformance(dom.window);

  // requestAnimationFrameモック
  mockRequestAnimationFrame(dom.window);

  // ComponentRegistryモック
  const mockBroadcast = vi.fn();
  (dom.window as unknown as { ComponentRegistry: { broadcast: typeof mockBroadcast } }).ComponentRegistry = {
    broadcast: mockBroadcast,
  };

  // スクリプトを実行
  executeScript(dom, scriptContent);

  return {
    win: dom.window as unknown as typeof window & { UpdateBatcher: UpdateBatcherClass },
    mockBroadcast,
  };
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
  let win: ReturnType<typeof loadUpdateBatcher>['win'];
  let UpdateBatcher: UpdateBatcherClass;
  let mockBroadcast: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    const loaded = loadUpdateBatcher();
    win = loaded.win;
    UpdateBatcher = win.UpdateBatcher;
    mockBroadcast = loaded.mockBroadcast;
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

    it('非数値は無視される（型ガード）', () => {
      const batcher = new UpdateBatcher();
      const originalInterval = batcher.batchInterval;
      batcher.setBatchInterval('invalid' as unknown as number);
      expect(batcher.batchInterval).toBe(originalInterval);
    });

    it('NaNは無視される（型ガード）', () => {
      const batcher = new UpdateBatcher();
      const originalInterval = batcher.batchInterval;
      batcher.setBatchInterval(NaN);
      expect(batcher.batchInterval).toBe(originalInterval);
    });

    it('undefinedは無視される（型ガード）', () => {
      const batcher = new UpdateBatcher();
      const originalInterval = batcher.batchInterval;
      batcher.setBatchInterval(undefined as unknown as number);
      expect(batcher.batchInterval).toBe(originalInterval);
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

  describe('flush() / forceFlush() のbroadcast呼び出し', () => {
    it('forceFlush()後にComponentRegistry.broadcast()が呼ばれる', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });
      batcher.queue('QueueList', { items: ['a', 'b'] });

      batcher.forceFlush();

      // 各タイプについてbroadcastが呼ばれる
      expect(mockBroadcast).toHaveBeenCalledWith('KPIBlock', { main: 1234 });
      expect(mockBroadcast).toHaveBeenCalledWith('QueueList', { items: ['a', 'b'] });
      expect(mockBroadcast).toHaveBeenCalledTimes(2);
    });

    it('forceFlush()後にpendingUpdatesがクリアされる', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 1234 });

      batcher.forceFlush();

      expect(batcher.getPendingCount()).toBe(0);
      expect(mockBroadcast).toHaveBeenCalledTimes(1);
    });

    it('空のキューでforceFlush()してもbroadcastは呼ばれない', () => {
      const batcher = new UpdateBatcher();
      batcher.forceFlush();

      expect(mockBroadcast).not.toHaveBeenCalled();
    });

    it('同一タイプを複数回queueしても最新のデータのみbroadcast', () => {
      const batcher = new UpdateBatcher();
      batcher.queue('KPIBlock', { main: 100 });
      batcher.queue('KPIBlock', { main: 200 });
      batcher.queue('KPIBlock', { main: 300 });

      batcher.forceFlush();

      // 最新の値のみがbroadcastされる
      expect(mockBroadcast).toHaveBeenCalledTimes(1);
      expect(mockBroadcast).toHaveBeenCalledWith('KPIBlock', { main: 300 });
    });
  });
});
