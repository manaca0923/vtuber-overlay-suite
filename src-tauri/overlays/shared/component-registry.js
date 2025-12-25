/**
 * コンポーネントレジストリ
 * コンポーネントクラスの登録、slotへのマウント/アンマウント管理、ライフサイクル制御
 *
 * 使用例:
 *   ComponentRegistry.register('ClockWidget', ClockWidget);
 *   ComponentRegistry.mount('left.top', 'ClockWidget', { rules: { ... } });
 *   ComponentRegistry.broadcast('KPIBlock', { main: 1234 });
 */
(function () {
  'use strict';

  // コンポーネントクラスレジストリ
  const componentClasses = {};

  // マウント済みインスタンス（slotId → { type, instance }）
  const mountedComponents = {};

  const ComponentRegistry = {
    /**
     * コンポーネントクラスを登録
     * @param {string} type - コンポーネントタイプ名（例: 'ClockWidget'）
     * @param {Function} ComponentClass - コンポーネントクラス
     */
    register(type, ComponentClass) {
      if (componentClasses[type]) {
        console.warn(`Component type "${type}" is already registered, overwriting.`);
      }
      componentClasses[type] = ComponentClass;
    },

    /**
     * 登録済みコンポーネントタイプを取得
     * @returns {string[]}
     */
    getRegisteredTypes() {
      return Object.keys(componentClasses);
    },

    /**
     * コンポーネントをslotにマウント
     * @param {string} slotId - slot ID（例: 'left.top'）
     * @param {string} type - コンポーネントタイプ
     * @param {object} config - 設定（rules, tuning, style）
     * @returns {object|null} コンポーネントインスタンス
     */
    mount(slotId, type, config = {}) {
      // 既存コンポーネントのアンマウント
      if (mountedComponents[slotId]) {
        this.unmount(slotId);
      }

      const ComponentClass = componentClasses[type];
      if (!ComponentClass) {
        console.error(`[ComponentRegistry] Unknown component type: ${type}`);
        return null;
      }

      // SlotManagerが存在しない場合はDOMから直接取得
      let slot;
      if (typeof SlotManager !== 'undefined') {
        slot = SlotManager.getSlot(slotId);
      } else {
        const cssId = 'slot-' + slotId.replace('.', '-');
        slot = document.getElementById(cssId);
      }

      if (!slot) {
        console.error(`[ComponentRegistry] Invalid slotId: ${slotId}`);
        return null;
      }

      try {
        const instance = new ComponentClass(config);
        instance.init(slot);
        mountedComponents[slotId] = { type, instance };
        console.log(`[ComponentRegistry] Mounted ${type} to ${slotId}`);
        return instance;
      } catch (e) {
        console.error(`[ComponentRegistry] Failed to mount ${type}:`, e);
        return null;
      }
    },

    /**
     * コンポーネントをアンマウント
     * @param {string} slotId
     */
    unmount(slotId) {
      const mounted = mountedComponents[slotId];
      if (mounted) {
        try {
          if (typeof mounted.instance.destroy === 'function') {
            mounted.instance.destroy();
          }
        } catch (e) {
          console.error(`[ComponentRegistry] Error destroying component at ${slotId}:`, e);
        }

        // SlotManagerがあれば使用、なければ直接クリア
        if (typeof SlotManager !== 'undefined') {
          SlotManager.unmountComponent(slotId);
        } else {
          const cssId = 'slot-' + slotId.replace('.', '-');
          const slot = document.getElementById(cssId);
          if (slot) slot.innerHTML = '';
        }

        delete mountedComponents[slotId];
        console.log(`[ComponentRegistry] Unmounted component from ${slotId}`);
      }
    },

    /**
     * 全コンポーネントをアンマウント
     */
    unmountAll() {
      Object.keys(mountedComponents).forEach((slotId) => this.unmount(slotId));
    },

    /**
     * 指定タイプの全インスタンスにデータを送信
     * @param {string} type - コンポーネントタイプ
     * @param {object} data - 送信データ
     */
    broadcast(type, data) {
      Object.entries(mountedComponents).forEach(([slotId, mounted]) => {
        if (mounted.type === type && typeof mounted.instance.update === 'function') {
          try {
            mounted.instance.update(data);
          } catch (e) {
            console.error(`[ComponentRegistry] Error updating ${type} at ${slotId}:`, e);
          }
        }
      });
    },

    /**
     * 全コンポーネントにイベントをディスパッチ
     * @param {string} eventType - イベントタイプ
     * @param {object} payload - ペイロード
     */
    dispatch(eventType, payload) {
      Object.entries(mountedComponents).forEach(([slotId, mounted]) => {
        if (typeof mounted.instance.onEvent === 'function') {
          try {
            mounted.instance.onEvent(eventType, payload);
          } catch (e) {
            console.error(`[ComponentRegistry] Error dispatching to ${slotId}:`, e);
          }
        }
      });
    },

    /**
     * slotIdからマウント済みインスタンスを取得
     * @param {string} slotId
     * @returns {object|null}
     */
    getInstance(slotId) {
      return mountedComponents[slotId]?.instance || null;
    },

    /**
     * slotIdからマウント済みコンポーネントタイプを取得
     * @param {string} slotId
     * @returns {string|null}
     */
    getType(slotId) {
      return mountedComponents[slotId]?.type || null;
    },

    /**
     * 全マウント済みコンポーネントの情報を取得
     * @returns {Array<{slotId: string, type: string, instance: object}>}
     */
    getMountedComponents() {
      return Object.entries(mountedComponents).map(([slotId, mounted]) => ({
        slotId,
        type: mounted.type,
        instance: mounted.instance,
      }));
    },

    /**
     * テンプレート設定から一括マウント
     * @param {Array} components - TemplateComponent[]
     */
    applyTemplate(components) {
      if (!Array.isArray(components)) {
        console.error('[ComponentRegistry] applyTemplate: components must be an array');
        return;
      }

      // 先に全アンマウント
      this.unmountAll();

      // enabledなコンポーネントをマウント
      components
        .filter((comp) => comp.enabled)
        .forEach((comp) => {
          this.mount(comp.slot, comp.type, {
            rules: comp.rules,
            tuning: comp.tuning,
            style: comp.style,
          });
        });
    },
  };

  // グローバルに公開
  window.ComponentRegistry = ComponentRegistry;
})();
