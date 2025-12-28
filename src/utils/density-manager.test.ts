/**
 * density-manager.js のユニットテスト
 *
 * PR#54で指摘されたDensityManagerのテストケース:
 * - recordUpdate()が履歴を記録すること
 * - 閾値超過で高密度状態になること
 * - 古いエントリがクリーンアップされること
 * - forceDegraded()/forceNormal()が状態を強制変更すること
 * - destroy()がタイマーを停止すること
 * - setThreshold()が範囲内にクランプすること
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
  loadScriptContent,
  createTestDOM,
  executeScript,
} from './test-helpers';

// DensityManagerを読み込んでwindow.DensityManagerを取得
function loadDensityManager(): typeof window & {
  DensityManager: DensityManagerClass;
  ComponentRegistry: { dispatch: ReturnType<typeof vi.fn> };
} {
  const scriptContent = loadScriptContent('src-tauri/overlays/shared/density-manager.js');
  const dom = createTestDOM();

  // ComponentRegistryモック
  const mockDispatch = vi.fn();
  (dom.window as unknown as { ComponentRegistry: { dispatch: typeof mockDispatch } }).ComponentRegistry = {
    dispatch: mockDispatch,
  };

  // スクリプトを実行
  executeScript(dom, scriptContent);

  return dom.window as unknown as typeof window & {
    DensityManager: DensityManagerClass;
    ComponentRegistry: { dispatch: typeof mockDispatch };
  };
}

// 型定義
interface DensityManagerClass {
  new (options?: DensityManagerOptions): DensityManagerInstance;
}

interface DensityManagerOptions {
  slots?: string[];
  windowMs?: number;
  threshold?: number;
  cleanupIntervalMs?: number;
}

interface DensityManagerInstance {
  monitoredSlots: string[];
  windowMs: number;
  highDensityThreshold: number;
  isDense: boolean;
  updateHistory: Map<string, number[]>;
  recordUpdate: (slotId: string) => void;
  checkDensity: () => void;
  applyDensityState: () => void;
  forceDegraded: () => void;
  forceNormal: () => void;
  isHighDensity: () => boolean;
  clearHistory: () => void;
  destroy: () => void;
  isDestroyed: () => boolean;
  setDegradedSettings: (settings: Record<string, unknown>) => void;
  setThreshold: (threshold: number) => void;
  getDebugInfo: () => DebugInfo;
}

interface DebugInfo {
  isDense: boolean;
  threshold: number;
  windowMs: number;
  slots: Record<string, { count: number; timestamps: number[] }>;
}

describe('DensityManager', () => {
  let win: ReturnType<typeof loadDensityManager>;
  let DensityManager: DensityManagerClass;
  let mockDispatch: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.useFakeTimers();
    win = loadDensityManager();
    DensityManager = win.DensityManager;
    mockDispatch = win.ComponentRegistry.dispatch;
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('デフォルトのslots設定', () => {
      const manager = new DensityManager();
      expect(manager.monitoredSlots).toEqual([
        'right.lowerLeft',
        'right.lowerRight',
        'right.bottom',
      ]);
      manager.destroy();
    });

    it('デフォルトのwindowMsは2000ms', () => {
      const manager = new DensityManager();
      expect(manager.windowMs).toBe(2000);
      manager.destroy();
    });

    it('デフォルトのthresholdは5', () => {
      const manager = new DensityManager();
      expect(manager.highDensityThreshold).toBe(5);
      manager.destroy();
    });

    it('オプションでカスタム設定が可能', () => {
      const manager = new DensityManager({
        slots: ['custom.slot'],
        windowMs: 3000,
        threshold: 10,
      });
      expect(manager.monitoredSlots).toEqual(['custom.slot']);
      expect(manager.windowMs).toBe(3000);
      expect(manager.highDensityThreshold).toBe(10);
      manager.destroy();
    });

    it('初期状態でisHighDensity()はfalse', () => {
      const manager = new DensityManager();
      expect(manager.isHighDensity()).toBe(false);
      manager.destroy();
    });
  });

  describe('recordUpdate()', () => {
    it('監視対象slotの更新を記録する', () => {
      const manager = new DensityManager({ threshold: 10 });
      manager.recordUpdate('right.lowerLeft');
      const debugInfo = manager.getDebugInfo();
      expect(debugInfo.slots['right.lowerLeft']!.count).toBe(1);
      manager.destroy();
    });

    it('監視対象外slotの更新は無視する', () => {
      const manager = new DensityManager({ threshold: 10 });
      manager.recordUpdate('unknown.slot');
      const debugInfo = manager.getDebugInfo();
      expect(debugInfo.slots['unknown.slot']).toBeUndefined();
      manager.destroy();
    });

    it('複数回の更新が記録される', () => {
      const manager = new DensityManager({ threshold: 10 });
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerLeft');
      const debugInfo = manager.getDebugInfo();
      expect(debugInfo.slots['right.lowerLeft']!.count).toBe(3);
      manager.destroy();
    });
  });

  describe('閾値と高密度状態', () => {
    it('閾値未満では高密度にならない', () => {
      const manager = new DensityManager({ threshold: 5 });
      for (let i = 0; i < 4; i++) {
        manager.recordUpdate('right.lowerLeft');
      }
      expect(manager.isHighDensity()).toBe(false);
      manager.destroy();
    });

    it('閾値以上で高密度状態になる', () => {
      const manager = new DensityManager({ threshold: 5 });
      for (let i = 0; i < 5; i++) {
        manager.recordUpdate('right.lowerLeft');
      }
      expect(manager.isHighDensity()).toBe(true);
      expect(mockDispatch).toHaveBeenCalledWith('density:high', expect.any(Object));
      manager.destroy();
    });

    it('複数slotの更新が合算される', () => {
      const manager = new DensityManager({ threshold: 5 });
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerRight');
      manager.recordUpdate('right.lowerRight');
      manager.recordUpdate('right.bottom');
      expect(manager.isHighDensity()).toBe(true);
      manager.destroy();
    });
  });

  describe('forceDegraded() / forceNormal()', () => {
    it('forceDegraded()で強制的に高密度状態にする', () => {
      const manager = new DensityManager();
      expect(manager.isHighDensity()).toBe(false);
      manager.forceDegraded();
      expect(manager.isHighDensity()).toBe(true);
      expect(mockDispatch).toHaveBeenCalledWith('density:high', expect.any(Object));
      manager.destroy();
    });

    it('forceNormal()で強制的に通常状態にする', () => {
      const manager = new DensityManager();
      manager.forceDegraded();
      mockDispatch.mockClear();

      manager.forceNormal();
      expect(manager.isHighDensity()).toBe(false);
      expect(mockDispatch).toHaveBeenCalledWith('density:normal', expect.any(Object));
      manager.destroy();
    });
  });

  describe('clearHistory()', () => {
    it('履歴をクリアする', () => {
      const manager = new DensityManager({ threshold: 10 });
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerRight');

      manager.clearHistory();
      const debugInfo = manager.getDebugInfo();
      expect(debugInfo.slots['right.lowerLeft']!.count).toBe(0);
      expect(debugInfo.slots['right.lowerRight']!.count).toBe(0);
      manager.destroy();
    });

    it('高密度状態をfalseにリセットする', () => {
      const manager = new DensityManager({ threshold: 2 });
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerLeft');
      expect(manager.isHighDensity()).toBe(true);

      manager.clearHistory();
      expect(manager.isHighDensity()).toBe(false);
      manager.destroy();
    });
  });

  describe('destroy()', () => {
    it('タイマーを停止しリソースを解放する', () => {
      const manager = new DensityManager();
      manager.destroy();
      expect(manager.isDestroyed()).toBe(true);
    });

    it('履歴をクリアする', () => {
      const manager = new DensityManager({ threshold: 10 });
      manager.recordUpdate('right.lowerLeft');
      manager.destroy();

      const debugInfo = manager.getDebugInfo();
      expect(debugInfo.slots['right.lowerLeft']!.count).toBe(0);
    });
  });

  describe('setThreshold()', () => {
    it('閾値を変更できる', () => {
      const manager = new DensityManager();
      manager.setThreshold(10);
      expect(manager.highDensityThreshold).toBe(10);
      manager.destroy();
    });

    it('1未満は1にクランプ', () => {
      const manager = new DensityManager();
      manager.setThreshold(0);
      expect(manager.highDensityThreshold).toBe(1);
      manager.destroy();
    });

    it('20超過は20にクランプ', () => {
      const manager = new DensityManager();
      manager.setThreshold(100);
      expect(manager.highDensityThreshold).toBe(20);
      manager.destroy();
    });
  });

  describe('setDegradedSettings()', () => {
    it('縮退設定を更新できる', () => {
      const manager = new DensityManager();
      manager.setDegradedSettings({ updateThrottle: 5000 });

      manager.forceDegraded();
      expect(mockDispatch).toHaveBeenCalledWith('density:high', expect.objectContaining({
        updateThrottle: 5000,
      }));
      manager.destroy();
    });
  });

  describe('getDebugInfo()', () => {
    it('デバッグ情報を返す', () => {
      const manager = new DensityManager({ threshold: 10 });
      manager.recordUpdate('right.lowerLeft');

      const info = manager.getDebugInfo();
      expect(info.isDense).toBe(false);
      expect(info.threshold).toBe(10);
      expect(info.windowMs).toBe(2000);
      expect(info.slots['right.lowerLeft']!.count).toBe(1);
      manager.destroy();
    });
  });

  describe('古いエントリのクリーンアップ', () => {
    it('recordUpdate時に古いエントリがフィルタリングされる', () => {
      const manager = new DensityManager({
        windowMs: 1000,
        threshold: 10,
      });

      // 履歴に古いタイムスタンプを直接設定
      const oldTimestamp = Date.now() - 2000; // 2秒前（windowMs超過）
      manager.updateHistory.set('right.lowerLeft', [oldTimestamp]);

      // 新しいrecordUpdateで古いエントリがフィルタリングされる
      manager.recordUpdate('right.lowerLeft');
      const debugInfo = manager.getDebugInfo();

      // 古いエントリは削除され、新しいエントリのみ残る
      expect(debugInfo.slots['right.lowerLeft']!.count).toBe(1);
      manager.destroy();
    });

    it('windowMs内のエントリは保持される', () => {
      const manager = new DensityManager({
        windowMs: 2000,
        threshold: 10,
      });

      // 履歴に新しいタイムスタンプを直接設定
      const recentTimestamp = Date.now() - 500; // 0.5秒前（windowMs内）
      manager.updateHistory.set('right.lowerLeft', [recentTimestamp]);

      // 新しいrecordUpdateで新しいエントリは保持される
      manager.recordUpdate('right.lowerLeft');
      const debugInfo = manager.getDebugInfo();

      // 新しいエントリ + recordUpdateのエントリ = 2つ
      expect(debugInfo.slots['right.lowerLeft']!.count).toBe(2);
      manager.destroy();
    });

    it('clearHistory()で履歴クリア後に高密度状態が解消される', () => {
      const manager = new DensityManager({
        threshold: 3,
      });

      // 高密度状態にする
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerLeft');
      manager.recordUpdate('right.lowerLeft');
      expect(manager.isHighDensity()).toBe(true);

      // 履歴をクリア
      manager.clearHistory();

      expect(manager.isHighDensity()).toBe(false);
      manager.destroy();
    });
  });
});
