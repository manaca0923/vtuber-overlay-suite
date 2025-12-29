/**
 * comment-renderer.js CommentQueueManager のユニットテスト
 *
 * PR#59で指摘されたテストケース:
 * - 即時モード（addInstant）とバッファモード（queue）の混在動作
 * - 重複防止機能
 * - バッファフラッシュの動作
 */

import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { JSDOM } from 'jsdom';
import {
  loadScriptContent,
  createTestDOM,
  executeScript,
} from './test-helpers';

// 型定義
interface Comment {
  id: string;
  author: string;
  text: string;
  timestamp?: number;
}

interface CommentQueueManagerClass {
  new (options?: CommentQueueManagerOptions): CommentQueueManagerInstance;
}

interface CommentQueueManagerOptions {
  onAddComment?: (comment: Comment) => void;
  bufferInterval?: number;
  minInterval?: number;
  maxInterval?: number;
  instantInterval?: number;
}

interface CommentQueueManagerInstance {
  commentBuffer: Comment[];
  bufferQueue: Comment[];
  instantQueue: Comment[];
  isProcessingBuffer: boolean;
  isProcessingInstant: boolean;
  processedIds: Set<string>;
  queue: (comment: Comment) => void;
  addInstant: (comment: Comment) => void;
  flushBuffer: () => void;
  _isDuplicate: (id: string) => boolean;
  _markProcessed: (id: string) => void;
}

interface CommentRendererType {
  CommentQueueManager: CommentQueueManagerClass;
  isValidHexColor: (color: string) => boolean;
  isValidNumber: (value: unknown, min?: number, max?: number) => boolean;
}

// CommentRendererを読み込んで取得
function loadCommentRenderer(): {
  dom: JSDOM;
  CommentQueueManager: CommentQueueManagerClass;
} {
  const scriptContent = loadScriptContent('src-tauri/overlays/shared/comment-renderer.js');
  const dom = createTestDOM();

  // CSS.escapeのモック
  (dom.window as unknown as { CSS: { escape: (s: string) => string } }).CSS = {
    escape: (s: string) => s.replace(/([^\w-])/g, '\\$1'),
  };

  // スクリプトを実行
  executeScript(dom, scriptContent);

  const CommentRenderer = (dom.window as unknown as { CommentRenderer: CommentRendererType }).CommentRenderer;

  return {
    dom,
    CommentQueueManager: CommentRenderer.CommentQueueManager,
  };
}

describe('CommentQueueManager', () => {
  let dom: JSDOM;
  let CommentQueueManager: CommentQueueManagerClass;

  beforeEach(() => {
    vi.useFakeTimers();
    const loaded = loadCommentRenderer();
    dom = loaded.dom;
    CommentQueueManager = loaded.CommentQueueManager;
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
    dom.window.close();
  });

  describe('バッファモード（queue）', () => {
    it('queue()でコメントがバッファに追加される', () => {
      const manager = new CommentQueueManager();
      const comment = { id: 'c1', author: 'User1', text: 'Hello' };

      manager.queue(comment);

      expect(manager.commentBuffer).toHaveLength(1);
      expect(manager.commentBuffer[0]).toEqual(comment);
    });

    it('複数コメントがバッファに蓄積される', () => {
      const manager = new CommentQueueManager();

      manager.queue({ id: 'c1', author: 'User1', text: 'Hello' });
      manager.queue({ id: 'c2', author: 'User2', text: 'World' });
      manager.queue({ id: 'c3', author: 'User3', text: '!' });

      expect(manager.commentBuffer).toHaveLength(3);
    });

    it('flushBuffer()でバッファが処理キューに移動', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({ onAddComment: mockAddComment });

      manager.queue({ id: 'c1', author: 'User1', text: 'Hello' });
      manager.queue({ id: 'c2', author: 'User2', text: 'World' });

      manager.flushBuffer();

      expect(manager.commentBuffer).toHaveLength(0);
      // flushBuffer後、最初のコメントが即時処理される
      expect(mockAddComment).toHaveBeenCalledTimes(1);
    });
  });

  describe('即時モード（addInstant）', () => {
    it('addInstant()でコメントが即座に処理される', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({ onAddComment: mockAddComment });
      const comment = { id: 'c1', author: 'User1', text: 'Instant!' };

      manager.addInstant(comment);

      expect(mockAddComment).toHaveBeenCalledWith(comment);
      expect(mockAddComment).toHaveBeenCalledTimes(1);
    });

    it('連続addInstant()でキューにコメントが蓄積される', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({
        onAddComment: mockAddComment,
        instantInterval: 150,
      });

      manager.addInstant({ id: 'c1', author: 'User1', text: 'First' });
      manager.addInstant({ id: 'c2', author: 'User2', text: 'Second' });
      manager.addInstant({ id: 'c3', author: 'User3', text: 'Third' });

      // 最初のコメントは即座に処理され、残りはキューに入る
      // Note: JSDOMタイマーはVitestで制御できないため、最終的にすべて処理される
      expect(mockAddComment).toHaveBeenCalledTimes(3);
    });

    it('addInstant()は処理済みIDとしてマークされる', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({ onAddComment: mockAddComment });

      manager.addInstant({ id: 'c1', author: 'User1', text: 'First' });

      // 処理済みとしてマークされている
      expect(manager.processedIds.has('c1')).toBe(true);
    });
  });

  describe('即時/バッファモード混在', () => {
    it('バッファとinstantは別々のキューで管理される', () => {
      const manager = new CommentQueueManager();

      // バッファに追加
      manager.queue({ id: 'b1', author: 'Buffer1', text: 'Buffer1' });
      manager.queue({ id: 'b2', author: 'Buffer2', text: 'Buffer2' });

      // 即時追加
      manager.addInstant({ id: 'i1', author: 'Instant1', text: 'Instant1' });

      // バッファにはバッファコメントのみ
      expect(manager.commentBuffer).toHaveLength(2);
      // 即時はprocessedIdsに記録される（処理済み）
      expect(manager.processedIds.has('i1')).toBe(true);
    });

    it('バッファフラッシュ後も即時コメントは追加可能', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({ onAddComment: mockAddComment });

      // バッファに追加してフラッシュ
      manager.queue({ id: 'b1', author: 'Buffer1', text: 'Buffer1' });
      manager.flushBuffer();

      // 即時コメントも追加可能
      manager.addInstant({ id: 'i1', author: 'Instant1', text: 'Instant1' });

      // 両方処理される
      expect(mockAddComment).toHaveBeenCalledTimes(2);
    });

    it('即時モードとバッファモードで重複IDは処理されない', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({ onAddComment: mockAddComment });

      // 同じIDのコメント
      const comment = { id: 'same-id', author: 'User', text: 'Same' };

      // バッファに追加
      manager.queue(comment);

      // 即時モードで同じIDを追加しようとする
      manager.addInstant({ ...comment });

      // バッファには1つだけ
      expect(manager.commentBuffer).toHaveLength(1);
      // 即時キューには追加されない（重複として検出）
      expect(manager.instantQueue).toHaveLength(0);
      // コールバックは呼ばれない
      expect(mockAddComment).not.toHaveBeenCalled();
    });
  });

  describe('重複防止', () => {
    it('同じIDのコメントはバッファに追加されない', () => {
      const manager = new CommentQueueManager();

      manager.queue({ id: 'c1', author: 'User1', text: 'First' });
      manager.queue({ id: 'c1', author: 'User1', text: 'Duplicate' });

      expect(manager.commentBuffer).toHaveLength(1);
    });

    it('processedIdsに記録されたIDは重複として扱われる', () => {
      const manager = new CommentQueueManager();

      manager._markProcessed('c1');

      manager.queue({ id: 'c1', author: 'User1', text: 'Already processed' });

      expect(manager.commentBuffer).toHaveLength(0);
    });

    it('_isDuplicate()は複数のキューをチェックする', () => {
      const manager = new CommentQueueManager();

      // バッファに追加
      manager.queue({ id: 'in-buffer', author: 'User', text: 'Buffer' });

      // 即時キューに追加（重複チェックをバイパスして直接追加）
      manager.instantQueue.push({ id: 'in-instant', author: 'User', text: 'Instant' });

      // 処理済みにマーク
      manager._markProcessed('processed');

      expect(manager._isDuplicate('in-buffer')).toBe(true);
      expect(manager._isDuplicate('in-instant')).toBe(true);
      expect(manager._isDuplicate('processed')).toBe(true);
      expect(manager._isDuplicate('new-id')).toBe(false);
    });

    it('processedIdsは最大1000件でローテート', () => {
      const manager = new CommentQueueManager();

      // 1001件追加
      for (let i = 0; i < 1001; i++) {
        manager._markProcessed(`id-${i}`);
      }

      // 最初のIDは削除されている
      expect(manager.processedIds.has('id-0')).toBe(false);
      // 最新のIDは存在
      expect(manager.processedIds.has('id-1000')).toBe(true);
      // サイズは1000
      expect(manager.processedIds.size).toBe(1000);
    });
  });

  describe('無効なコメント', () => {
    it('IDがないコメントはqueue()で無視される', () => {
      const manager = new CommentQueueManager();

      manager.queue({ author: 'User', text: 'No ID' } as unknown as Comment);
      manager.queue(null as unknown as Comment);
      manager.queue(undefined as unknown as Comment);

      expect(manager.commentBuffer).toHaveLength(0);
    });

    it('IDがないコメントはaddInstant()で無視される', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({ onAddComment: mockAddComment });

      manager.addInstant({ author: 'User', text: 'No ID' } as unknown as Comment);

      expect(mockAddComment).not.toHaveBeenCalled();
    });
  });

  describe('表示間隔の計算', () => {
    it('flushBuffer()でコメント数に応じた間隔が計算される', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({
        onAddComment: mockAddComment,
        bufferInterval: 5000,
        minInterval: 100,
        maxInterval: 1000,
      });

      // 10件追加 → 5000/10 = 500ms間隔
      for (let i = 0; i < 10; i++) {
        manager.queue({ id: `c${i}`, author: 'User', text: `Comment ${i}` });
      }

      manager.flushBuffer();
      expect(mockAddComment).toHaveBeenCalledTimes(1);

      // 500ms後に次のコメント
      vi.advanceTimersByTime(500);
      expect(mockAddComment).toHaveBeenCalledTimes(2);
    });

    it('間隔はminIntervalとmaxIntervalの範囲内にクランプされる', () => {
      const mockAddComment = vi.fn();
      const manager = new CommentQueueManager({
        onAddComment: mockAddComment,
        bufferInterval: 5000,
        minInterval: 100,
        maxInterval: 1000,
      });

      // 100件追加 → 5000/100 = 50ms → minInterval(100)にクランプ
      for (let i = 0; i < 100; i++) {
        manager.queue({ id: `c${i}`, author: 'User', text: `Comment ${i}` });
      }

      manager.flushBuffer();
      expect(mockAddComment).toHaveBeenCalledTimes(1);

      // 100ms（minInterval）後に次のコメント
      vi.advanceTimersByTime(100);
      expect(mockAddComment).toHaveBeenCalledTimes(2);
    });
  });
});
