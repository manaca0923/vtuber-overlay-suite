/**
 * 過密検出マネージャー
 *
 * 右下エリアのコンポーネント更新頻度を監視し、高負荷時に縮退処理を適用。
 * ComponentRegistry.dispatch()を通じて各コンポーネントにdensityイベントを送信。
 *
 * 使用例:
 *   const densityManager = new DensityManager();
 *   densityManager.recordUpdate('right.lowerLeft');
 *   // 高負荷検出時に自動で ComponentRegistry.dispatch('density:high', {...}) が呼ばれる
 *   // 終了時には densityManager.destroy() を呼び出してタイマーをクリア
 */
(function () {
  'use strict';

  /** 定期クリーンアップのデフォルト間隔（ms） */
  const DEFAULT_CLEANUP_INTERVAL_MS = 5000;

  class DensityManager {
    /**
     * コンストラクタ
     * @param {object} options - オプション
     * @param {string[]} options.slots - 監視対象のslot ID配列
     * @param {number} options.windowMs - 監視ウィンドウ（ms、デフォルト: 2000）
     * @param {number} options.threshold - 高負荷判定閾値（デフォルト: 5）
     * @param {number} options.cleanupIntervalMs - 定期クリーンアップ間隔（ms、デフォルト: 5000）
     */
    constructor(options = {}) {
      this.monitoredSlots = options.slots || [
        'right.lowerLeft', // KPIBlock
        'right.lowerRight', // QueueList
        'right.bottom', // PromoPanel
      ];

      this.windowMs = options.windowMs || 2000;
      this.highDensityThreshold = options.threshold || 5;
      this.cleanupIntervalMs = options.cleanupIntervalMs || DEFAULT_CLEANUP_INTERVAL_MS;

      // 更新履歴（slotId -> タイムスタンプ配列）
      this.updateHistory = new Map();
      this.monitoredSlots.forEach((slot) => this.updateHistory.set(slot, []));

      // 現在の状態
      this.isDense = false;

      // 縮退時の設定
      this.degradedSettings = {
        updateThrottle: 4000, // KPIBlock: 2秒 → 4秒
        maxItems: 4, // QueueList: 6 → 4
        showSec: 10, // PromoPanel: 6秒 → 10秒
      };

      // 通常時の設定（復元用）
      this.normalSettings = {
        updateThrottle: 2000,
        maxItems: 6,
        showSec: 6,
      };

      // 定期クリーンアップタイマーを開始
      // recordUpdate()が呼ばれない場合でも古いエントリを除去し、メモリリークを防止
      this._cleanupTimerId = setInterval(() => {
        this._cleanupOldEntries();
      }, this.cleanupIntervalMs);
    }

    /**
     * 更新を記録
     * @param {string} slotId - 更新されたslot ID
     */
    recordUpdate(slotId) {
      if (!this.updateHistory.has(slotId)) return;

      const now = Date.now();
      const cutoff = now - this.windowMs;

      // 古いエントリを除去しつつ新しいエントリを追加
      // filter() + push() で配列操作を効率化（shift()のO(n)を回避）
      const history = this.updateHistory.get(slotId);
      const filtered = history.filter((t) => t >= cutoff);
      filtered.push(now);
      this.updateHistory.set(slotId, filtered);

      this.checkDensity();
    }

    /**
     * 過密状態をチェック
     */
    checkDensity() {
      let totalUpdates = 0;
      this.updateHistory.forEach((history) => {
        totalUpdates += history.length;
      });

      const wasDense = this.isDense;
      this.isDense = totalUpdates >= this.highDensityThreshold;

      if (this.isDense !== wasDense) {
        this.applyDensityState();
      }
    }

    /**
     * 過密状態に応じた処理を適用
     */
    applyDensityState() {
      if (typeof ComponentRegistry === 'undefined') return;

      if (this.isDense) {
        console.log(
          `[DensityManager] High density detected (threshold: ${this.highDensityThreshold})`
        );
        ComponentRegistry.dispatch('density:high', this.degradedSettings);
      } else {
        console.log('[DensityManager] Density normalized');
        ComponentRegistry.dispatch('density:normal', this.normalSettings);
      }
    }

    /**
     * 強制的に縮退モードを適用
     */
    forceDegraded() {
      this.isDense = true;
      this.applyDensityState();
    }

    /**
     * 強制的に通常モードを適用
     */
    forceNormal() {
      this.isDense = false;
      this.applyDensityState();
    }

    /**
     * 現在の状態を取得
     * @returns {boolean}
     */
    isHighDensity() {
      return this.isDense;
    }

    /**
     * 更新履歴をクリア
     */
    clearHistory() {
      this.updateHistory.forEach((history) => {
        history.length = 0;
      });
      this.isDense = false;
    }

    /**
     * 古いエントリを削除（定期クリーンアップ）
     * @private
     */
    _cleanupOldEntries() {
      const now = Date.now();
      const cutoff = now - this.windowMs;
      let cleaned = false;

      this.updateHistory.forEach((history, slotId) => {
        const originalLength = history.length;
        const filtered = history.filter((t) => t >= cutoff);

        if (filtered.length !== originalLength) {
          this.updateHistory.set(slotId, filtered);
          cleaned = true;
        }
      });

      // クリーンアップ後に過密状態をチェック
      // 古いエントリが消えたことで過密状態が解消される可能性がある
      if (cleaned) {
        this.checkDensity();
      }
    }

    /**
     * DensityManagerを破棄
     * 定期クリーンアップタイマーを停止し、リソースを解放
     */
    destroy() {
      if (this._cleanupTimerId) {
        clearInterval(this._cleanupTimerId);
        this._cleanupTimerId = null;
      }
      this.clearHistory();
    }

    /**
     * DensityManagerが破棄されたかどうかを返す
     * @returns {boolean} 破棄済みならtrue
     */
    isDestroyed() {
      return this._cleanupTimerId === null;
    }

    /**
     * 縮退設定を更新
     * @param {object} settings - 新しい縮退設定
     */
    setDegradedSettings(settings) {
      Object.assign(this.degradedSettings, settings);
    }

    /**
     * 閾値を更新
     * @param {number} threshold - 新しい閾値
     */
    setThreshold(threshold) {
      if (typeof threshold !== 'number' || isNaN(threshold)) return;
      this.highDensityThreshold = Math.max(1, Math.min(20, threshold));
    }

    /**
     * デバッグ情報を取得
     * @returns {object}
     */
    getDebugInfo() {
      const info = {
        isDense: this.isDense,
        threshold: this.highDensityThreshold,
        windowMs: this.windowMs,
        slots: {},
      };

      this.updateHistory.forEach((history, slotId) => {
        info.slots[slotId] = {
          count: history.length,
          timestamps: [...history],
        };
      });

      return info;
    }
  }

  // グローバルに公開
  window.DensityManager = DensityManager;
})();
