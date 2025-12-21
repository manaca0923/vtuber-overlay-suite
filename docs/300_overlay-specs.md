# オーバーレイ仕様書

## 概要

OBS Studioのブラウザソースとして動作するオーバーレイの仕様。

---

## エンドポイント

| URL | 用途 |
|-----|------|
| `http://localhost:19800/overlay/comment` | コメント表示 |
| `http://localhost:19800/overlay/setlist` | セットリスト表示 |
| `http://localhost:19800/api/overlay/settings` | オーバーレイ設定取得（初期化用） |
| `ws://localhost:19801/ws` | リアルタイム更新 |

---

## WebSocket プロトコル

### 接続

```javascript
const ws = new WebSocket('ws://localhost:19801/ws');
ws.onopen = () => {
  // 購読するチャンネルを指定
  ws.send(JSON.stringify({ type: 'subscribe', channel: 'comments' }));
};
```

### メッセージ形式

**サーバー → クライアント**

```typescript
// コメント追加
{
  type: 'comment:add',
  payload: {
    id: string,
    message: string,
    authorName: string,
    authorImageUrl: string,
    isOwner: boolean,
    isModerator: boolean,
    isMember: boolean,
    messageType: MessageType,
    publishedAt: string
  }
}

// コメント削除（モデレーション）
{
  type: 'comment:remove',
  payload: { id: string }
}

// セットリスト更新
{
  type: 'setlist:update',
  payload: {
    currentIndex: number,
    songs: Array<{
      id: string,
      title: string,
      artist: string,
      status: 'pending' | 'current' | 'done'
    }>
  }
}

// 設定更新
{
  type: 'settings:update',
  payload: {
    theme: string,
    primaryColor: string,      // #RRGGBB形式
    fontFamily: string,        // フォントファミリー
    borderRadius: number,      // 角丸（px）
    comment: {
      enabled: boolean,        // 表示ON/OFF
      position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right',
      maxCount: number,        // 最大表示件数
      showAvatar: boolean,     // アバター表示
      fontSize: number         // フォントサイズ（px）
    },
    setlist: {
      enabled: boolean,        // 表示ON/OFF
      position: 'top' | 'bottom' | 'left' | 'right',
      showArtist: boolean,     // アーティスト表示
      fontSize: number         // フォントサイズ（px）
    }
  }
}
```

### 再接続ロジック

```javascript
class WebSocketManager {
  constructor(url) {
    this.url = url;
    this.reconnectDelay = 1000;
    this.maxDelay = 30000;
  }

  connect() {
    this.ws = new WebSocket(this.url);

    this.ws.onclose = () => {
      setTimeout(() => {
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxDelay);
        this.connect();
      }, this.reconnectDelay);
    };

    this.ws.onopen = () => {
      this.reconnectDelay = 1000; // リセット
    };
  }
}
```

---

## コメントオーバーレイ

### HTML構造

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div id="comment-container">
    <!-- コメントが動的に追加される -->
  </div>
  <script src="script.js"></script>
</body>
</html>
```

### コメント要素構造

```html
<div class="comment" data-id="xxx" data-type="text">
  <img class="avatar" src="..." alt="">
  <div class="content">
    <div class="header">
      <span class="name">ユーザー名</span>
      <span class="badge badge-moderator">モデ</span>
      <span class="badge badge-member">メンバー</span>
    </div>
    <div class="message">コメント内容</div>
  </div>
</div>
```

### スーパーチャット

```html
<div class="comment superchat" data-id="xxx" data-type="superchat" style="--sc-color: #ff0000">
  <img class="avatar" src="..." alt="">
  <div class="content">
    <div class="header">
      <span class="name">ユーザー名</span>
      <span class="amount">¥1,000</span>
    </div>
    <div class="message">スパチャメッセージ</div>
  </div>
</div>
```

### CSS変数（テーマ対応）

```css
:root {
  /* カラー */
  --primary-color: #6366f1;
  --background-color: transparent;
  --text-color: #ffffff;
  --text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);

  /* サイズ */
  --avatar-size: 48px;
  --font-size-name: 14px;
  --font-size-message: 16px;
  --comment-gap: 8px;
  --max-comments: 10;

  /* アニメーション */
  --animation-duration: 0.3s;
  --animation-easing: ease-out;
}
```

### アニメーション

```css
/* フェードイン */
@keyframes comment-enter {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* フェードアウト */
@keyframes comment-exit {
  from {
    opacity: 1;
    transform: translateY(0);
  }
  to {
    opacity: 0;
    transform: translateY(-20px);
  }
}

.comment {
  animation: comment-enter var(--animation-duration) var(--animation-easing);
}

.comment.removing {
  animation: comment-exit var(--animation-duration) var(--animation-easing);
}
```

---

## セットリストオーバーレイ

### HTML構造

```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <link rel="stylesheet" href="style.css">
</head>
<body>
  <div id="setlist-container">
    <div class="song previous">前の曲</div>
    <div class="song current">
      <span class="indicator">♪</span>
      <span class="title">現在の曲</span>
      <span class="artist">アーティスト</span>
    </div>
    <div class="song next">次の曲</div>
  </div>
  <script src="script.js"></script>
</body>
</html>
```

### CSS変数

```css
:root {
  --primary-color: #6366f1;
  --current-bg: rgba(99, 102, 241, 0.8);
  --other-bg: rgba(0, 0, 0, 0.5);
  --text-color: #ffffff;

  --font-size-title: 24px;
  --font-size-artist: 14px;

  --transition-duration: 0.5s;
}
```

### 曲切替アニメーション

```css
@keyframes slide-up {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.song.entering {
  animation: slide-up var(--transition-duration) ease-out;
}
```

---

## OBS設定ガイド

### 推奨設定

| 項目 | コメント | セットリスト |
|------|----------|--------------|
| 幅 | 400px | 350px |
| 高さ | 600px | 200px |
| FPS | 30 | 30 |
| CSS | カスタム可 | カスタム可 |

### カスタムCSS（OBS）

```css
/* 背景を完全透過 */
body {
  background: transparent !important;
}

/* スクロールバー非表示 */
::-webkit-scrollbar {
  display: none;
}
```

### URLパラメータ

オーバーレイはURLパラメータでカスタマイズ可能。

```
http://localhost:19800/overlay/comment?theme=dark&maxCount=15
http://localhost:19800/overlay/setlist?showArtist=false&position=bottom
```

| パラメータ | 型 | デフォルト | 説明 |
|------------|-----|------------|------|
| theme | string | 'default' | テーマ名 |
| maxCount | number | 10 | 最大表示件数 |
| showAvatar | boolean | true | アバター表示 |
| showBadge | boolean | true | バッジ表示 |
| position | string | 'bottom' | 表示位置 |
| showArtist | boolean | true | アーティスト表示 |
| showPrevNext | boolean | true | 前後曲表示 |

---

## パフォーマンス要件

### DOM管理

```javascript
const MAX_COMMENTS = 10;

function addComment(comment) {
  const container = document.getElementById('comment-container');
  const el = createCommentElement(comment);
  container.appendChild(el);

  // 古いコメントを削除
  while (container.children.length > MAX_COMMENTS) {
    const oldest = container.firstChild;
    oldest.classList.add('removing');
    setTimeout(() => oldest.remove(), 300);
  }
}
```

### メモリリーク防止

- イベントリスナーの適切な解除
- setIntervalの適切なクリア
- DOM要素の明示的な削除

### 負荷制御

- アニメーションは `transform` と `opacity` のみ使用（GPU最適化）
- 大量コメント時は間引き表示
- requestAnimationFrame での描画最適化

---

## テンプレート設定スキーマ

```typescript
interface TemplateSettings {
  // 共通
  theme: 'default' | 'dark' | 'light' | 'custom';
  primaryColor: string;      // #RRGGBB
  secondaryColor: string;
  fontFamily: string;

  // コメント固有
  comment: {
    enabled: boolean;
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
    maxCount: number;
    showAvatar: boolean;
    showBadges: boolean;
    avatarSize: number;
    fontSize: number;
    animationSpeed: 'slow' | 'normal' | 'fast';
  };

  // セットリスト固有
  setlist: {
    enabled: boolean;
    position: 'top' | 'bottom' | 'left' | 'right';
    showArtist: boolean;
    showPrevNext: boolean;
    fontSize: number;
  };
}
```

---

## 神テンプレ デザインガイドライン

### 品質基準
- **ミナトデザイン・唐揚丸レベル**を目標
- 視認性とデザイン性のバランス
- 配信画面を邪魔しない透過感

### カラーバリアント
1. **Default** - パープル系（#6366f1）
2. **Sakura** - ピンク系（#ec4899）
3. **Ocean** - ブルー系（#0ea5e9）

### 構成要素
- 背景: グラデーション＋グラスモーフィズム
- 枠線: 微細なグロー効果
- 角丸: 統一した丸み（8-12px）
- 影: 自然なドロップシャドウ
