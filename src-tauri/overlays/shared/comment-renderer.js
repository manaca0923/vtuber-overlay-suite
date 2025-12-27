/**
 * コメントレンダリング共通モジュール
 * comment.html と combined.html で共有するJavaScript関数
 */

(function() {
'use strict';

// =============================================================================
// バリデーション関数
// =============================================================================

/**
 * 16進数カラーコードのバリデーション
 * @param {string} color - カラーコード
 * @returns {boolean}
 */
function isValidHexColor(color) {
  return typeof color === 'string' && /^#[0-9A-Fa-f]{6}$/.test(color);
}

/**
 * 数値のバリデーション
 * @param {string|number} value - 値
 * @param {number} min - 最小値
 * @param {number} max - 最大値
 * @returns {boolean}
 */
function isValidNumber(value, min, max) {
  const num = parseInt(value, 10);
  return !isNaN(num) && num >= min && num <= max;
}

/**
 * フォントファミリーのサニタイズ（XSS対策）
 * @param {string} fontFamily - フォントファミリー
 * @returns {string|null}
 */
function sanitizeFontFamily(fontFamily) {
  if (typeof fontFamily !== 'string' || fontFamily.length === 0 || fontFamily.length > 200) {
    return null;
  }
  // 危険な文字を除去
  return fontFamily.replace(/[<>"'`\;{}]/g, '');
}

// =============================================================================
// URL処理
// =============================================================================

/**
 * URLを正規化（//形式をhttps:に変換）
 * @param {string} url - URL
 * @returns {string}
 */
function normalizeUrl(url) {
  if (!url || typeof url !== 'string') return url;
  if (url.startsWith('//')) {
    return 'https:' + url;
  }
  return url;
}

/**
 * 絵文字URLがYouTubeドメインかどうかを検証
 * @param {string} url - URL
 * @returns {boolean}
 */
function isValidEmojiUrl(url) {
  if (!url || typeof url !== 'string') return false;
  try {
    const parsed = new URL(normalizeUrl(url));
    const hostname = parsed.hostname;
    // YouTube関連ドメインのみ許可
    const exactHosts = ['youtube.com', 'www.youtube.com', 'fonts.gstatic.com'];
    if (exactHosts.includes(hostname)) return true;
    // サフィックスマッチ
    const allowedSuffixes = ['.ggpht.com', '.googleusercontent.com', '.ytimg.com', '.gstatic.com'];
    return allowedSuffixes.some(suffix => hostname.endsWith(suffix));
  } catch {
    return false;
  }
}

// =============================================================================
// スーパーチャット色分け
// =============================================================================

/**
 * 金額文字列から数値を抽出
 * @param {string} amountString - 金額文字列
 * @returns {number}
 */
function parseAmount(amountString) {
  if (!amountString) return 0;

  // カンマが小数点として使われているか判定（欧州形式: "1.000,50"）
  const hasCommaDecimal = /\d,\d{1,2}$/.test(amountString);

  let cleaned = amountString.replace(/[^\d.,]/g, '');

  if (hasCommaDecimal) {
    // 欧州形式: ピリオドを除去（千の区切り）、カンマをピリオドに（小数点）
    cleaned = cleaned.replace(/\./g, '').replace(',', '.');
  } else {
    // 英語/日本形式: カンマを除去（千の区切り）
    cleaned = cleaned.replace(/,/g, '');
  }

  return parseFloat(cleaned) || 0;
}

/**
 * 通貨を日本円相当額に変換
 * TODO: 将来的に為替レートはAPIから取得することを検討
 * @param {string} amountString - 金額文字列
 * @returns {number}
 */
function convertToJPY(amountString) {
  if (!amountString) return 0;
  const amount = parseAmount(amountString);

  // 通貨を判定して日本円に換算
  if (amountString.includes('$') || amountString.includes('USD') || amountString.includes('CA$') || amountString.includes('A$')) {
    return amount * 150; // USD/CAD/AUD -> JPY
  } else if (amountString.includes('€') || amountString.includes('EUR')) {
    return amount * 160; // EUR -> JPY
  } else if (amountString.includes('£') || amountString.includes('GBP')) {
    return amount * 190; // GBP -> JPY
  } else if (amountString.includes('₩') || amountString.includes('KRW')) {
    return amount * 0.11; // KRW -> JPY
  } else if (amountString.includes('NT$') || amountString.includes('TWD')) {
    return amount * 4.7; // TWD -> JPY
  } else if (amountString.includes('¥') || amountString.includes('JPY') || amountString.includes('円')) {
    return amount; // JPY
  } else {
    // その他の通貨は金額をそのまま使用
    return amount;
  }
}

/**
 * 金額に応じたスーパーチャットの色を返す（YouTube公式準拠）
 * @param {string} amountString - 金額文字列
 * @returns {string}
 */
function getSuperChatColor(amountString) {
  const jpy = convertToJPY(amountString);

  if (jpy >= 10000) {
    return '#f44336'; // 赤 - ¥10,000以上
  } else if (jpy >= 5000) {
    return '#e91e63'; // マゼンタ - ¥5,000-9,999
  } else if (jpy >= 2000) {
    return '#e65100'; // オレンジ - ¥2,000-4,999
  } else if (jpy >= 1000) {
    return '#ffb300'; // 黄色 - ¥1,000-1,999
  } else if (jpy >= 500) {
    return '#00c853'; // 緑 - ¥500-999
  } else if (jpy >= 200) {
    return '#00e5ff'; // 水色 - ¥200-499
  } else {
    return '#1e88e5'; // 青 - ¥100-199（最低額）
  }
}

// =============================================================================
// 絵文字レンダリング
// =============================================================================

/**
 * 絵文字を含むメッセージをレンダリング
 * @param {HTMLElement} container - コンテナ要素
 * @param {Array} messageRuns - メッセージランの配列
 */
function renderMessageWithEmoji(container, messageRuns) {
  messageRuns.forEach(run => {
    if (run.text !== undefined) {
      // テキスト部分
      container.appendChild(document.createTextNode(run.text));
    } else if (run.emoji) {
      // 絵文字部分
      const thumbnails = run.emoji.image?.thumbnails || [];
      const thumb = thumbnails.find(t => t.width >= 24) || thumbnails[0];
      const shortcut = run.emoji.shortcuts?.[0] || ':emoji:';

      // セキュリティ: YouTubeドメインのURLのみ許可
      if (thumb && isValidEmojiUrl(thumb.url)) {
        const img = document.createElement('img');
        img.className = 'inline-emoji';
        img.alt = shortcut;
        img.title = shortcut;
        img.loading = 'lazy';
        const normalizedUrl = normalizeUrl(thumb.url);
        img.onerror = function () {
          const text = document.createTextNode(shortcut);
          this.parentNode?.replaceChild(text, this);
        };
        img.src = normalizedUrl;
        container.appendChild(img);
      } else {
        // URLが無効またはサムネイルがない場合はショートカットをテキスト表示
        container.appendChild(document.createTextNode(shortcut));
      }
    }
  });
}

// =============================================================================
// コメント要素生成
// =============================================================================

/**
 * コメント要素を生成
 * @param {Object} comment - コメントデータ
 * @param {boolean} showAvatar - アバター表示フラグ
 * @returns {HTMLElement}
 */
function createCommentElement(comment, showAvatar) {
  const div = document.createElement('div');
  div.className = 'comment';
  div.dataset.id = comment.id;

  // messageTypeはタグ付きenum: { type: "text" | "superChat" | ... }
  const messageType = comment.messageType?.type;
  if (messageType === 'superChat') {
    div.classList.add('superchat');
    div.style.setProperty('--sc-color', getSuperChatColor(comment.messageType.amount));
  } else if (messageType === 'superSticker') {
    div.classList.add('supersticker');
  } else if (messageType === 'membership') {
    div.classList.add('membership');
  } else if (messageType === 'membershipGift') {
    div.classList.add('membershipgift');
  }

  const avatar = document.createElement('img');
  avatar.className = 'avatar';
  avatar.src = comment.authorImageUrl;
  avatar.alt = comment.authorName;
  if (!showAvatar) {
    avatar.style.display = 'none';
  }

  const content = document.createElement('div');
  content.className = 'content';

  const header = document.createElement('div');
  header.className = 'header';

  const name = document.createElement('span');
  name.className = 'name';
  name.textContent = comment.authorName;
  header.appendChild(name);

  if (comment.isOwner) {
    const badge = document.createElement('span');
    badge.className = 'badge badge-owner';
    badge.textContent = 'Owner';
    header.appendChild(badge);
  }

  if (comment.isModerator) {
    const badge = document.createElement('span');
    badge.className = 'badge badge-moderator';
    badge.textContent = 'Mod';
    header.appendChild(badge);
  }

  if (comment.isMember) {
    const badge = document.createElement('span');
    badge.className = 'badge badge-member';
    badge.textContent = 'Member';
    header.appendChild(badge);
  }

  // スーパーチャットの金額表示
  if (messageType === 'superChat' && comment.messageType.amount) {
    const amount = document.createElement('span');
    amount.className = 'amount';
    amount.textContent = comment.messageType.amount;
    header.appendChild(amount);
  }

  // スーパーステッカーのバッジ
  if (messageType === 'superSticker') {
    const badge = document.createElement('span');
    badge.className = 'badge';
    badge.style.background = '#f59e0b';
    badge.style.color = '#000';
    badge.textContent = 'Sticker';
    header.appendChild(badge);
  }

  // メンバーシップのバッジ（レベル表示対応）
  if (messageType === 'membership') {
    const badge = document.createElement('span');
    badge.className = 'badge';
    badge.style.background = '#10b981';
    badge.style.color = '#fff';
    const level = comment.messageType.level || 'New Member';
    badge.textContent = level;
    header.appendChild(badge);
  }

  // メンバーシップギフトのバッジ
  if (messageType === 'membershipGift' && comment.messageType.count) {
    const badge = document.createElement('span');
    badge.className = 'badge';
    badge.style.background = '#8b5cf6';
    badge.style.color = '#fff';
    badge.textContent = `Gift x${comment.messageType.count}`;
    header.appendChild(badge);
  }

  const message = document.createElement('div');
  message.className = 'message';

  // messageRunsがある場合は絵文字を含むレンダリング
  if (comment.messageRuns && comment.messageRuns.length > 0) {
    renderMessageWithEmoji(message, comment.messageRuns);
  } else {
    message.textContent = comment.message;
  }

  content.appendChild(header);
  content.appendChild(message);

  div.appendChild(avatar);
  div.appendChild(content);

  return div;
}

// =============================================================================
// アニメーション
// =============================================================================

/**
 * フェードアウトアニメーション付きでコメントを削除
 * @param {HTMLElement} element - 削除する要素
 */
function removeCommentWithAnimation(element) {
  if (!element || element.classList.contains('removing')) return;

  element.classList.add('removing');
  element.addEventListener('animationend', () => {
    element.remove();
  }, { once: true });

  // フォールバック: アニメーションが発火しなかった場合のタイムアウト
  setTimeout(() => {
    if (element.parentNode) {
      element.remove();
    }
  }, 500);
}

// =============================================================================
// コメントキューシステム
// =============================================================================

/**
 * コメントキューマネージャー
 * - instant=false: 5秒間に溜まったコメントを5秒間で等間隔表示（公式APIポーリング用）
 * - instant=true: 即座に表示（gRPC/InnerTube用）
 *
 * 重要: 即時キューとバッファキューは完全に分離されており、
 * それぞれ独立したキューと処理フラグを持つ。
 * これにより、バッファ処理中でも即時コメントは短間隔で処理される。
 */
class CommentQueueManager {
  constructor(options = {}) {
    // バッファモード用（公式APIポーリング）
    this.commentBuffer = [];
    this.bufferQueue = [];
    this.isProcessingBuffer = false;
    this.currentDisplayInterval = 300;
    this.BUFFER_INTERVAL = options.bufferInterval || 5000;
    this.MIN_DISPLAY_INTERVAL = options.minInterval || 100;
    this.MAX_DISPLAY_INTERVAL = options.maxInterval || 1000;

    // 即時モード用（gRPC/InnerTube）
    this.instantQueue = [];
    this.isProcessingInstant = false;
    // 即時表示時の表示間隔（連続で来た場合のスロットリング）
    this.INSTANT_DISPLAY_INTERVAL = options.instantInterval || 150;

    this.onAddComment = options.onAddComment || (() => {});
    // 処理済みIDのセット（重複防止用）
    this.processedIds = new Set();
    this.MAX_PROCESSED_IDS = 1000;

    // 定期フラッシュ開始（バッファモード用）
    setInterval(() => this.flushBuffer(), this.BUFFER_INTERVAL);
  }

  /**
   * 処理済みIDを記録（メモリリーク防止のため古いIDを削除）
   * @param {string} id - コメントID
   */
  _markProcessed(id) {
    this.processedIds.add(id);
    if (this.processedIds.size > this.MAX_PROCESSED_IDS) {
      // 古いIDを削除（Setは挿入順を保持）
      const iterator = this.processedIds.values();
      this.processedIds.delete(iterator.next().value);
    }
  }

  /**
   * 重複チェック
   * @param {string} id - コメントID
   * @returns {boolean} 重複している場合true
   */
  _isDuplicate(id) {
    if (this.processedIds.has(id)) return true;
    if (this.commentBuffer.some(c => c.id === id)) return true;
    if (this.bufferQueue.some(c => c.id === id)) return true;
    if (this.instantQueue.some(c => c.id === id)) return true;
    if (document.querySelector(`[data-id="${CSS.escape(id)}"]`)) return true;
    return false;
  }

  /**
   * コメントをバッファに追加（バッファモード）
   * @param {Object} comment - コメントデータ
   */
  queue(comment) {
    if (!comment || !comment.id) {
      console.warn('Invalid comment: missing id', comment);
      return;
    }

    // 重複チェック
    if (this._isDuplicate(comment.id)) return;

    this.commentBuffer.push(comment);
  }

  /**
   * コメントを即座に表示（即時モード）
   * gRPC/InnerTube用：バッファをスキップして即座に表示
   * Note: バッファ処理とは独立したキュー・フラグを使用するため、
   * バッファ処理中でも即時コメントは短間隔で処理される
   * @param {Object} comment - コメントデータ
   */
  addInstant(comment) {
    if (!comment || !comment.id) {
      console.warn('Invalid comment: missing id', comment);
      return;
    }

    // 重複チェック
    if (this._isDuplicate(comment.id)) return;

    // 処理済みとしてマーク
    this._markProcessed(comment.id);

    // 即時キューに追加（バッファキューとは別）
    this.instantQueue.push(comment);

    // 即時処理中でなければ開始
    if (!this.isProcessingInstant) {
      this._processInstantQueue();
    }
  }

  /**
   * 即時モード用のキュー処理
   * バッファ処理とは独立して動作
   */
  _processInstantQueue() {
    if (this.instantQueue.length === 0) {
      this.isProcessingInstant = false;
      return;
    }

    this.isProcessingInstant = true;
    const comment = this.instantQueue.shift();
    this.onAddComment(comment);

    // 次のコメントがあれば短い間隔で処理
    if (this.instantQueue.length > 0) {
      setTimeout(() => this._processInstantQueue(), this.INSTANT_DISPLAY_INTERVAL);
    } else {
      this.isProcessingInstant = false;
    }
  }

  /**
   * バッファを表示キューに移動（バッファモード用）
   */
  flushBuffer() {
    if (this.commentBuffer.length > 0) {
      const count = this.commentBuffer.length;
      this.currentDisplayInterval = Math.floor(this.BUFFER_INTERVAL / count);
      this.currentDisplayInterval = Math.max(
        this.MIN_DISPLAY_INTERVAL,
        Math.min(this.MAX_DISPLAY_INTERVAL, this.currentDisplayInterval)
      );

      console.log(`Flushing ${count} comments with ${this.currentDisplayInterval}ms interval`);

      // 処理済みとしてマーク
      this.commentBuffer.forEach(c => this._markProcessed(c.id));

      this.bufferQueue.push(...this.commentBuffer);
      this.commentBuffer.length = 0;

      this._processBufferQueue();
    }
  }

  /**
   * バッファキューを処理
   * 即時処理とは独立して動作
   */
  _processBufferQueue() {
    if (this.isProcessingBuffer || this.bufferQueue.length === 0) return;

    this.isProcessingBuffer = true;
    const comment = this.bufferQueue.shift();
    this.onAddComment(comment);

    setTimeout(() => {
      this.isProcessingBuffer = false;
      this._processBufferQueue();
    }, this.currentDisplayInterval);
  }

  /**
   * 表示キューを処理（後方互換性のため残す）
   * @deprecated _processBufferQueue() を使用してください
   */
  processQueue() {
    this._processBufferQueue();
  }
}

// グローバルに公開（ESモジュール非対応環境向け）
window.CommentRenderer = {
  isValidHexColor,
  isValidNumber,
  sanitizeFontFamily,
  normalizeUrl,
  isValidEmojiUrl,
  parseAmount,
  convertToJPY,
  getSuperChatColor,
  renderMessageWithEmoji,
  createCommentElement,
  removeCommentWithAnimation,
  CommentQueueManager
};

})();
