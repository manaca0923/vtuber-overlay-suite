/**
 * QueueList - 待機キューコンポーネント
 *
 * 配置: right.lowerRight
 * 機能: リクエスト曲などの待機キューを表示
 *
 * style設定:
 *   - title: string (タイトル、デフォルト: 'リクエスト')
 *   - showNumber: boolean (番号表示、デフォルト: true)
 *
 * rules設定:
 *   - maxItems: number (最大アイテム数、デフォルト: 6、範囲: 3-10)
 *   - showWhenEmpty: boolean (空でも表示、デフォルト: false)
 *
 * update()で受け取るデータ:
 *   - items: Array<{id?: string, text: string}>
 *   - title: string
 */
class QueueList extends BaseComponent {
  constructor(config) {
    super(config);

    // 表示設定
    this.title = this.style.title || 'リクエスト';
    this.showNumber = this.style.showNumber !== false;

    // ルール設定（クランプ適用）
    this.maxItems = this.clampByKey(this.rules.maxItems || 6, 'queueMaxItems', 3, 10);
    this.originalMaxItems = this.maxItems; // 縮退モードからの復元用
    this.showWhenEmpty = this.rules.showWhenEmpty || false;

    // アイテム
    this.items = [];
  }

  /**
   * イベントハンドラ（density対応）
   * @param {string} eventType
   * @param {object} payload
   */
  onEvent(eventType, payload) {
    if (eventType === 'density:high') {
      this.applyDegradedMode(payload);
    } else if (eventType === 'density:normal') {
      this.restoreNormalMode(payload);
    }
  }

  /**
   * 縮退モードを適用
   * @param {object} settings
   */
  applyDegradedMode(settings) {
    if (settings.maxItems) {
      this.maxItems = this.clampByKey(settings.maxItems, 'queueMaxItems', 3, 10);
      // 現在の表示を更新
      if (this.items.length > this.maxItems) {
        this.items = this.items.slice(0, this.maxItems);
        this.renderList();
      }
    }
  }

  /**
   * 通常モードに復元
   * @param {object} settings
   */
  restoreNormalMode(settings) {
    if (settings.maxItems) {
      this.maxItems = this.clampByKey(settings.maxItems, 'queueMaxItems', 3, 10);
    } else {
      this.maxItems = this.originalMaxItems;
    }
  }

  render() {
    const container = this.createElement('div', {
      className: 'queue-list panel clamp-box',
    });

    this.titleEl = this.createElement('div', {
      className: 'queue-title dt-text-shadow',
      textContent: this.title,
    });

    this.listEl = this.createElement('ul', {
      className: 'queue-items',
    });

    container.appendChild(this.titleEl);
    container.appendChild(this.listEl);

    return container;
  }

  afterMount() {
    this.updateVisibility();
  }

  update(data) {
    if (data.items && Array.isArray(data.items)) {
      this.items = data.items.slice(0, this.maxItems);
      this.renderList();
      this.updateVisibility();
    }

    if (data.title !== undefined) {
      this.title = data.title;
      this.titleEl.textContent = data.title;
    }
  }

  /**
   * リストをレンダリング
   */
  renderList() {
    this.listEl.innerHTML = '';

    this.items.forEach((item, idx) => {
      const li = this.createElement('li', {
        className: 'queue-item dt-text-shadow dt-ellipsis',
      });

      // アイテムIDがあれば data-id として設定
      if (item.id) {
        li.setAttribute('data-id', item.id);
      }

      // テキスト構築（textContentはHTMLを解釈しないため、escapeHtmlは不要）
      const text = this.showNumber ? `${idx + 1}. ${item.text || item}` : item.text || item;
      li.textContent = text;

      this.listEl.appendChild(li);
    });
  }

  /**
   * 表示/非表示を更新
   */
  updateVisibility() {
    if (this.items.length === 0 && !this.showWhenEmpty) {
      this.element.style.display = 'none';
    } else {
      this.element.style.display = '';
    }
  }

  /**
   * アイテムを追加
   * @param {object} item - {id?: string, text: string}
   */
  addItem(item) {
    if (this.items.length >= this.maxItems) {
      // 最大数を超えたら古いものを削除
      this.items.shift();
    }
    this.items.push(item);
    this.renderList();
    this.updateVisibility();
  }

  /**
   * アイテムを削除
   * @param {string} id - 削除するアイテムのID
   */
  removeItem(id) {
    const index = this.items.findIndex((item) => item.id === id);
    if (index !== -1) {
      this.items.splice(index, 1);
      this.renderList();
      this.updateVisibility();
    }
  }

  /**
   * 先頭アイテムを削除して返す
   * @returns {object|null}
   */
  dequeue() {
    if (this.items.length === 0) return null;

    const item = this.items.shift();
    this.renderList();
    this.updateVisibility();
    return item;
  }

  /**
   * すべてクリア
   */
  clear() {
    this.items = [];
    this.renderList();
    this.updateVisibility();
  }

  /**
   * 現在のアイテム数を取得
   * @returns {number}
   */
  getCount() {
    return this.items.length;
  }
}

// レジストリに登録
if (typeof ComponentRegistry !== 'undefined') {
  ComponentRegistry.register('QueueList', QueueList);
}
