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
 */
(function () {
  'use strict';

  class DensityManager {
    /**
     * コンストラクタ
     * @param {object} options - オプション
     * @param {string[]} options.slots - 監視対象のslot ID配列
     * @param {number} options.windowMs - 監視ウィンドウ（ms、デフォルト: 2000）
     * @param {number} options.threshold - 高負荷判定閾値（デフォルト: 5）
     */
    constructor(options = {}) {
      this.monitoredSlots = options.slots || [
        'right.lowerLeft', // KPIBlock
        'right.lowerRight', // QueueList
        'right.bottom', // PromoPanel
      ];

      this.windowMs = options.windowMs || 2000;
      this.highDensityThreshold = options.threshold || 5;

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
    }

    /**
     * 更新を記録
     * @param {string} slotId - 更新されたslot ID
     */
    recordUpdate(slotId) {
      if (!this.updateHistory.has(slotId)) return;

      const now = Date.now();
      const history = this.updateHistory.get(slotId);
      history.push(now);

      // 古いエントリを削除
      const cutoff = now - this.windowMs;
      while (history.length > 0 && history[0] < cutoff) {
        history.shift();
      }

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
