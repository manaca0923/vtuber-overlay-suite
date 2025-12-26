/**
 * 更新バッチャー
 *
 * WebSocket受信を100-200msでバッチ処理し、requestAnimationFrameで適用。
 * 同一コンポーネントタイプへの複数更新は最新のみ保持（上書き）。
 *
 * 使用例:
 *   const batcher = new UpdateBatcher({ batchInterval: 150 });
 *   batcher.queue('KPIBlock', { main: 1234 });
 *   batcher.queue('QueueList', { items: [...] });
 *   // 150ms後にまとめてComponentRegistry.broadcast()が呼ばれる
 */
(function () {
  'use strict';

  class UpdateBatcher {
    /**
     * コンストラクタ
     * @param {object} options - オプション
     * @param {number} options.batchInterval - バッチ間隔（ms、デフォルト: 150）
     */
    constructor(options = {}) {
      this.batchInterval = options.batchInterval || 150; // 100-200msの中間値
      this.pendingUpdates = new Map(); // type -> latestData
      this.isScheduled = false;
      this.lastFlush = 0;
      this.timeoutId = null;
    }

    /**
     * 更新をキューに追加
     * 同一タイプは最新のみ保持（上書き）
     *
     * @param {string} type - コンポーネントタイプ
     * @param {object} data - 更新データ
     */
    queue(type, data) {
      this.pendingUpdates.set(type, data);
      this.scheduleFlush();
    }

    /**
     * フラッシュをスケジュール
     * 前回フラッシュからの経過時間を考慮して遅延を計算
     */
    scheduleFlush() {
      if (this.isScheduled) return;

      const now = performance.now();
      const elapsed = now - this.lastFlush;
      const delay = Math.max(0, this.batchInterval - elapsed);

      this.isScheduled = true;
      this.timeoutId = setTimeout(() => {
        requestAnimationFrame(() => this.flush());
      }, delay);
    }

    /**
     * キュー内の更新を一括適用
     */
    flush() {
      this.isScheduled = false;
      this.timeoutId = null;
      this.lastFlush = performance.now();

      if (this.pendingUpdates.size === 0) return;

      // バッチ処理: 全更新を1フレームで適用
      // ComponentRegistryの存在チェックはループ外で1回のみ実行
      if (typeof ComponentRegistry !== 'undefined') {
        this.pendingUpdates.forEach((data, type) => {
          ComponentRegistry.broadcast(type, data);
        });
      }

      this.pendingUpdates.clear();
    }

    /**
     * 強制フラッシュ（タイマーを待たずに即時適用）
     */
    forceFlush() {
      if (this.timeoutId) {
        clearTimeout(this.timeoutId);
        this.timeoutId = null;
      }
      this.isScheduled = false;
      this.flush();
    }

    /**
     * キューをクリア（フラッシュせずに破棄）
     */
    clear() {
      if (this.timeoutId) {
        clearTimeout(this.timeoutId);
        this.timeoutId = null;
      }
      this.isScheduled = false;
      this.pendingUpdates.clear();
    }

    /**
     * キュー内の更新数を取得
     * @returns {number}
     */
    getPendingCount() {
      return this.pendingUpdates.size;
    }

    /**
     * バッチ間隔を変更
     * @param {number} ms - 新しいバッチ間隔（ms）
     */
    setBatchInterval(ms) {
      this.batchInterval = Math.max(50, Math.min(500, ms)); // 50-500msの範囲
    }

    /**
     * UpdateBatcherを破棄
     * タイマーを停止し、リソースを解放
     */
    destroy() {
      this.clear();
      this._destroyed = true;
    }

    /**
     * インスタンスが破棄されているかを確認
     * @returns {boolean}
     */
    isDestroyed() {
      return this._destroyed === true;
    }
  }

  // グローバルに公開
  window.UpdateBatcher = UpdateBatcher;
})();
