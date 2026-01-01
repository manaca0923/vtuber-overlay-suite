/**
 * Slot管理モジュール
 * 11個のslot配置とコンポーネントマウントを管理
 *
 * 使用例:
 *   const slot = SlotManager.getSlot('left.middle');
 *   SlotManager.setVisibility('right.upper', false);
 *   SlotManager.mountComponent('left.top', clockElement);
 */
(function () {
  'use strict';

  // slot ID一覧（TypeScript SlotIdと同期）
  const SLOT_IDS = [
    'left.top',
    'left.topBelow',
    'left.middle',
    'left.lower',
    'left.bottom',
    'center.full',
    'right.top',
    'right.upper',
    'right.kpi',
    'right.tanzaku',
    'right.bottom',
  ];

  // slot情報（役割と設定）
  const SLOT_INFO = {
    'left.top': { column: 'left', role: '時刻', clampBox: false },
    'left.topBelow': { column: 'left', role: '天気', clampBox: false },
    'left.middle': { column: 'left', role: 'コメント', clampBox: true },
    'left.lower': { column: 'left', role: 'スパチャ', clampBox: false },
    'left.bottom': { column: 'left', role: 'ロゴ', clampBox: false },
    'center.full': { column: 'center', role: '主役', clampBox: false },
    'right.top': { column: 'right', role: 'ラベル', clampBox: false },
    'right.upper': { column: 'right', role: 'セトリ', clampBox: true },
    'right.kpi': { column: 'right', role: 'KPI', clampBox: false },
    'right.tanzaku': { column: 'right', role: '短冊', clampBox: true },
    'right.bottom': { column: 'right', role: '告知', clampBox: false },
  };

  /**
   * slotId → CSS ID変換
   * 例: 'left.top' → 'slot-left-top'
   * @param {string} slotId
   * @returns {string}
   */
  function toCssId(slotId) {
    return 'slot-' + slotId.replace('.', '-');
  }

  /**
   * CSS ID → slotId変換
   * 例: 'slot-left-top' → 'left.top'
   *
   * カラム名（left/center/right）を明示的にチェックすることで
   * 複数ハイフンを含むslot名（例: lowerLeft）も正しく変換
   *
   * @param {string} cssId
   * @returns {string|null}
   */
  const COLUMN_NAMES = ['left', 'center', 'right'];

  function toSlotId(cssId) {
    const prefix = 'slot-';
    if (!cssId.startsWith(prefix)) return null;

    const rest = cssId.slice(prefix.length);

    // カラム名を明示的にチェック
    for (const column of COLUMN_NAMES) {
      if (rest.startsWith(column + '-')) {
        const name = rest.slice(column.length + 1);
        const slotId = column + '.' + name;
        return SLOT_IDS.includes(slotId) ? slotId : null;
      }
    }

    return null;
  }

  /**
   * slotIdが有効かどうか
   * @param {string} id
   * @returns {boolean}
   */
  function isValidSlotId(id) {
    return SLOT_IDS.includes(id);
  }

  // SlotManager API
  const SlotManager = {
    /**
     * slot要素を取得
     * @param {string} slotId - 例: 'left.top'
     * @returns {HTMLElement|null}
     */
    getSlot(slotId) {
      if (!isValidSlotId(slotId)) {
        console.warn(`Invalid slotId: ${slotId}`);
        return null;
      }
      return document.getElementById(toCssId(slotId));
    },

    /**
     * slotの表示/非表示を設定
     * @param {string} slotId
     * @param {boolean} visible
     */
    setVisibility(slotId, visible) {
      const el = this.getSlot(slotId);
      if (el) {
        el.classList.toggle('hidden', !visible);
      }
    },

    /**
     * slotが表示されているか
     * @param {string} slotId
     * @returns {boolean}
     */
    isVisible(slotId) {
      const el = this.getSlot(slotId);
      return el && !el.classList.contains('hidden');
    },

    /**
     * コンポーネントをslotにマウント
     * @param {string} slotId
     * @param {HTMLElement} componentEl
     */
    mountComponent(slotId, componentEl) {
      const slot = this.getSlot(slotId);
      if (slot) {
        // 既存コンテンツをクリアしてマウント
        slot.innerHTML = '';
        slot.appendChild(componentEl);
      }
    },

    /**
     * slotからコンポーネントをアンマウント
     * @param {string} slotId
     */
    unmountComponent(slotId) {
      const slot = this.getSlot(slotId);
      if (slot) {
        slot.innerHTML = '';
      }
    },

    /**
     * 全slotの状態を取得
     * @returns {Array<{id: string, element: HTMLElement|null, isEmpty: boolean, isVisible: boolean}>}
     */
    getAllSlots() {
      return SLOT_IDS.map((id) => ({
        id,
        element: this.getSlot(id),
        isEmpty: this.isSlotEmpty(id),
        isVisible: this.isVisible(id),
        info: SLOT_INFO[id],
      }));
    },

    /**
     * slotが空かどうか
     * @param {string} slotId
     * @returns {boolean}
     */
    isSlotEmpty(slotId) {
      const slot = this.getSlot(slotId);
      if (!slot) return true;
      // 空白ノードを除いた子要素があるか
      return (
        Array.from(slot.childNodes).filter((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) return true;
          if (node.nodeType === Node.TEXT_NODE)
            return node.textContent.trim() !== '';
          return false;
        }).length === 0
      );
    },

    /**
     * カラムごとのslotを取得
     * @param {'left'|'center'|'right'} column
     * @returns {string[]}
     */
    getSlotsByColumn(column) {
      return SLOT_IDS.filter((id) => SLOT_INFO[id].column === column);
    },

    /**
     * slot情報を取得
     * @param {string} slotId
     * @returns {object|null}
     */
    getSlotInfo(slotId) {
      return SLOT_INFO[slotId] || null;
    },

    // 定数をエクスポート
    SLOT_IDS,
    SLOT_INFO,

    // ユーティリティ関数をエクスポート
    toCssId,
    toSlotId,
    isValidSlotId,
  };

  // グローバルに公開
  window.SlotManager = SlotManager;
})();
