# オーバーレイ仕様書

## 概要

OBS Studioのブラウザソースとして動作するオーバーレイの仕様。

---

## エンドポイント

| URL | 用途 |
|-----|------|
| `http://localhost:19800/overlay/comment` | コメント表示（個別） |
| `http://localhost:19800/overlay/setlist` | セットリスト表示（個別） |
| `http://localhost:19800/overlay/combined` | 統合オーバーレイ（コメント+セットリスト） |
| `http://localhost:19800/api/overlay/settings` | オーバーレイ設定取得（初期化用） |
| `ws://localhost:19801/ws` | リアルタイム更新 |

---

## WebSocket プロトコル

### 接続

```javascript
const ws = new WebSocket('ws://localhost:19801/ws');
ws.onopen = () => {
  console.log('WebSocket connected');
  // 接続完了時に自動で全メッセージを受信開始
  // NOTE: subscribe機能は未実装のため、接続するだけで全種別のメッセージを受信
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
  },
  instant: boolean,            // true: 即時表示（gRPC/キャッシュ）, false: バッファリング表示
  buffer_interval_ms?: number  // バッファ間隔（ミリ秒）。InnerTube: 1000, 公式API: 省略時デフォルト5000
}

// コメント削除（モデレーション）
{
  type: 'comment:remove',
  payload: { id: string }
}

// スパチャ追加（T25: スパチャ専用ウィジェット）
{
  type: 'superchat:add',
  payload: {
    id: string,
    authorName: string,
    authorImageUrl: string,
    amount: string,           // 表示用金額文字列（例: "¥1,000"）
    amountMicros: number,     // マイクロ単位の金額（1円 = 1,000,000マイクロ）
    currency: string,         // 通貨コード（"JPY", "USD", "EUR"等）
    message: string,          // スパチャメッセージ
    tier: number,             // 金額帯 1-7（YouTube公式Tier準拠）
    displayDurationMs: number // 表示時間（ミリ秒）
  }
}

// スパチャ削除（表示時間終了）
{
  type: 'superchat:remove',
  payload: { id: string }
}

// スパチャTierとカラー（YouTube公式準拠）
// | Tier | 金額 (JPY) | 表示時間 | 背景色 |
// |------|-----------|---------|--------|
// | 1 | ¥100-199   | 10秒 | #1565C0 (Blue) |
// | 2 | ¥200-499   | 20秒 | #00B8D4 (Cyan) |
// | 3 | ¥500-999   | 30秒 | #00BFA5 (Teal) |
// | 4 | ¥1,000-1,999 | 1分 | #FFB300 (Yellow) |
// | 5 | ¥2,000-4,999 | 2分 | #F57C00 (Orange) |
// | 6 | ¥5,000-9,999 | 3分 | #E91E63 (Pink) |
// | 7 | ¥10,000+   | 5分 | #E62117 (Red) |

// セットリスト更新
{
  type: 'setlist:update',
  payload: {
    setlistId: string,        // セットリストID（フィルタリング用）
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
      showAvatar: boolean,     // アバター表示
      fontSize: number         // フォントサイズ（px）
      // NOTE: maxCountは画面高さベースの自動調整に統一したため削除
    },
    setlist: {
      enabled: boolean,        // 表示ON/OFF
      position: 'top' | 'bottom' | 'left' | 'right',
      showArtist: boolean,     // アーティスト表示
      fontSize: number         // フォントサイズ（px）
    },
    widget?: {                 // ウィジェット表示設定（オプショナル、v2レイアウト用）
      clock: boolean,          // 時計（left.top）
      weather: boolean,        // 天気（left.topBelow）
      comment: boolean,        // コメント（left.middle）
      superchat: boolean,      // スパチャ（left.lower）- T21以降で実装予定
      logo: boolean,           // ロゴ（left.bottom）
      setlist: boolean,        // セトリ（right.upper）
      kpi: boolean,            // KPI（right.lowerLeft）
      tanzaku: boolean,        // 短冊（right.lowerRight）
      announcement: boolean    // 告知（right.bottom）
    },
    themeSettings?: {          // テーマ設定（issues/016対応）
      globalTheme: 'white' | 'purple' | 'sakura' | 'ocean' | 'custom',
      globalPrimaryColor: string,      // #RRGGBB形式
      customColors: Array<{            // 保存済みカスタムカラー（最大3件）
        id: string,
        name: string,
        color: string
      }>,
      widgetColorOverrides: {          // ウィジェット個別カラー（省略時はglobalを使用）
        clock?: string,
        weather?: string,
        comment?: string,
        superchat?: string,
        logo?: string,
        setlist?: string,
        kpi?: string,
        tanzaku?: string,
        announcement?: string
      },
      fontPreset: 'noto-sans-jp' | 'm-plus-1' | 'yu-gothic' | 'meiryo' | 'system',
      customFontFamily: string | null  // fontPreset='system'時のフォント名
    }
  }
}
```

### 拡張メッセージ形式（3カラムレイアウト用）- 将来実装予定

```typescript
// テンプレート適用
{
  type: 'template:apply',
  payload: {
    templateId: string,
    config: TemplateConfig  // 後述の型定義参照
  }
}

// コンポーネント更新（個別）
{
  type: 'component:update',
  payload: {
    componentId: string,
    data: {
      // ClockWidget用
      time?: string,
      date?: string,
      // WeatherWidget用
      weather?: {
        temp: number,
        condition: string,
        icon: string
      },
      // KPIBlock用
      kpi?: {
        primary: { value: number, label: string },
        secondary?: { value: number, label: string }
      },
      // PromoPanel用
      promo?: {
        items: Array<{ text: string, url?: string }>,
        currentIndex: number
      },
      // QueueList用
      queue?: {
        items: Array<{ id: string, name: string, message?: string }>,
        totalCount: number
      }
    }
  }
}

// slot可視性変更
{
  type: 'slot:visibility',
  payload: {
    slotId: SlotId,  // 後述の型定義参照
    visible: boolean
  }
}

// レイアウト動的変更
{
  type: 'layout:update',
  payload: {
    leftPct?: number,    // 0.18-0.28
    centerPct?: number,  // 0.44-0.64
    rightPct?: number,   // 0.18-0.28
    gutterPx?: number    // 0-64
  }
}
```

---

## postMessage プロトコル（プレビュー用）

設定画面のプレビューiframe用のpostMessage通信仕様。

### 送信元（React設定画面）

```typescript
// OverlayPreview.tsx
const PREVIEW_ORIGIN = 'http://localhost:19800';

// 設定変更時にiframeへ送信
iframeRef.current.contentWindow.postMessage({
  type: 'preview:settings:update',
  payload: {
    widget?: WidgetVisibilitySettings,
    comment?: CommentSettings,
    setlist?: SetlistSettings,
    weather?: WeatherSettings,
    themeSettings?: ThemeSettings
  }
}, PREVIEW_ORIGIN);
```

### 受信側（オーバーレイHTML）

```javascript
// overlay-core.js - PostMessageHandler
const TRUSTED_ORIGINS = [
  'http://localhost:1420',   // Vite開発サーバー
  'http://localhost:19800',  // プレビュー用
  'tauri://localhost'        // Tauri WebView
];

class PostMessageHandler {
  constructor(trustedOrigins = TRUSTED_ORIGINS) {
    this.trustedOrigins = trustedOrigins;
    this.handlers = {};
    window.addEventListener('message', (e) => this._handleMessage(e));
  }

  _handleMessage(event) {
    // 1. origin検証（issues/002参照）
    if (!this.trustedOrigins.includes(event.origin)) {
      console.warn('Untrusted origin:', event.origin);
      return;
    }

    // 2. ペイロード検証（issues/013参照）
    const data = event.data;
    if (!data || typeof data !== 'object') return;
    if (typeof data.type !== 'string') return;

    // 3. ハンドラ実行
    const handler = this.handlers[data.type];
    if (handler) {
      handler(data.payload);
    }
  }

  on(type, callback) {
    this.handlers[type] = callback;
  }
}

// 使用例
const postMessageHandler = new PostMessageHandler();
postMessageHandler.on('preview:settings:update', (payload) => {
  applySettingsUpdate(payload);
});
```

### セキュリティ考慮事項

- **targetOrigin明示**: 送信側は`'*'`ではなく具体的なオリジンを指定（issues/002参照）
- **origin検証**: 受信側はTRUSTED_ORIGINSでホワイトリスト検証（issues/002参照）
- **ペイロード検証**: 型チェックで不正なデータを排除（issues/013参照）

---

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
  --primary-color: #ffffff;        /* デフォルトは白（issues/016対応） */
  --background-color: transparent;
  --text-color: #ffffff;
  --text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);

  /* ウィジェット個別カラー（issues/016対応） */
  --widget-clock-color: var(--primary-color, #ffffff);
  --widget-weather-color: var(--primary-color, #ffffff);
  --widget-comment-color: var(--primary-color, #ffffff);
  --widget-superchat-color: var(--primary-color, #ffffff);
  --widget-logo-color: var(--primary-color, #ffffff);
  --widget-setlist-color: var(--primary-color, #ffffff);
  --widget-kpi-color: var(--primary-color, #ffffff);
  --widget-tanzaku-color: var(--primary-color, #ffffff);
  --widget-announcement-color: var(--primary-color, #ffffff);

  /* フォント */
  --font-family: 'Yu Gothic', 'YuGothic', sans-serif;

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

## 統合オーバーレイ（combined.html）

コメントとセットリストを1つのブラウザソースで表示する統合オーバーレイ。

### 用途

- OBSのブラウザソースを1つで済ませたい場合
- コメントとセットリストの配置を連動させたい場合
- レイアウトプリセットを活用したい場合

### レイアウトプリセット（v1）

| プリセット | 用途 | コメント位置 | セットリスト位置 |
|-----------|------|-------------|-----------------|
| `streaming` | 配信向け | bottom-left | bottom |
| `talk` | 雑談向け | bottom-right | left |
| `music` | 歌配信向け | top-right | bottom |
| `gaming` | ゲーム配信向け | top-left | right |
| `custom` | カスタム | ユーザー指定 | ユーザー指定 |

### HTML構造（v1）

```html
<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <title>Combined Overlay</title>
</head>
<body>
  <div id="overlay-root" class="layout-streaming">
    <!-- コメントエリア -->
    <div id="comment-area">
      <div id="comment-container">
        <!-- コメントが動的に追加される -->
      </div>
    </div>
    <!-- セットリストエリア -->
    <div id="setlist-area">
      <div id="setlist-container">
        <!-- セットリストが動的に追加される -->
      </div>
    </div>
  </div>
</body>
</html>
```

### CSS Grid レイアウト（v1）

```css
/* レイアウトプリセット: streaming */
#overlay-root.layout-streaming {
  grid-template-columns: 1fr 2fr;
  grid-template-rows: 1fr auto;
  grid-template-areas:
    "comment main"
    "setlist setlist";
}

/* レイアウトプリセット: talk */
#overlay-root.layout-talk {
  grid-template-columns: auto 1fr 1fr;
  grid-template-rows: 1fr;
  grid-template-areas: "setlist main comment";
}

/* レイアウトプリセット: music */
#overlay-root.layout-music {
  grid-template-columns: 2fr 1fr;
  grid-template-rows: auto 1fr;
  grid-template-areas:
    "main comment"
    "setlist setlist";
}

/* レイアウトプリセット: gaming */
#overlay-root.layout-gaming {
  grid-template-columns: 1fr 1fr auto;
  grid-template-rows: 1fr;
  grid-template-areas: "comment main setlist";
}
```

---

## 3カラムレイアウト（v2）- 将来実装予定

> **ステータス**: 設計完了、実装予定
>
> v1のプリセットレイアウトに加えて、より柔軟な3カラム固定レイアウトを提供する。

### 概要

- **3カラム固定比率**: 左22% / 中央56% / 右22%
- **セーフエリア**: 上4% / 右4% / 下5% / 左4%
- **slot配置**: 11個の固定配置領域
- **OBS推奨**: 1920×1080（16:9）

### CSS変数（3カラム用）

```css
:root {
  /* レイアウト */
  --gutter: 24px;
  --safe-top: 4vh;
  --safe-right: 4vw;
  --safe-bottom: 5vh;
  --safe-left: 4vw;
  --left-col: 22%;
  --center-col: 56%;
  --right-col: 22%;
  --row-gap: 16px;

  /* パネル */
  --radius: 14px;
  --panel-bg: rgba(0,0,0,0.25);
  --panel-blur: 10px;
}
```

### slot配置（11個）

| slot | 役割 | 推奨高さ | 破綻防止ルール |
|------|------|---------|---------------|
| `left.top` | 時刻/日付 | auto | 2行固定 |
| `left.topBelow` | 天気 | auto | 情報量最小 |
| `left.middle` | コメント | 1fr | `maxLines`必須、fade/ellipsis |
| `left.lower` | スパチャ | auto | 最小表示秒数＋キュー |
| `left.bottom` | ロゴ/注意 | auto | 小さめ、透過 |
| `center.full` | 主役ステージ | 1fr | 左右侵食禁止 |
| `right.top` | ラベル | auto | 1行固定 |
| `right.upper` | セトリ | 1fr | `maxItems`必須、ellipsis |
| `right.lowerLeft` | KPI | auto | 更新頻度抑制（2〜5秒） |
| `right.lowerRight` | 短冊 | auto〜中 | `maxItems`必須、空なら非表示 |
| `right.bottom` | 告知 | auto | cycle表示 |

### HTML構造（3カラム）

```html
<div class="layout-root">
  <div class="col-left">
    <section id="left.top"></section>
    <section id="left.topBelow" class="panel"></section>
    <section id="left.middle" class="clamp-box"></section>
    <section id="left.lower"></section>
    <section id="left.bottom"></section>
  </div>

  <div class="col-center">
    <section id="center.full"></section>
  </div>

  <div class="col-right">
    <section id="right.top"></section>
    <section id="right.upper" class="clamp-box"></section>
    <div class="right-lower-grid">
      <section id="right.lowerLeft"></section>
      <section id="right.lowerRight" class="clamp-box"></section>
    </div>
    <section id="right.bottom"></section>
  </div>
</div>
```

### CSS Grid（3カラム）

```css
.layout-root {
  position: relative;
  width: 100%;
  height: 100%;
  padding: var(--safe-top) var(--safe-right) var(--safe-bottom) var(--safe-left);
  box-sizing: border-box;
  display: grid;
  grid-template-columns: var(--left-col) var(--center-col) var(--right-col);
  column-gap: var(--gutter);
}

.col-left, .col-center, .col-right {
  height: 100%;
  display: grid;
  row-gap: var(--row-gap);
}

.col-left {
  grid-template-rows: auto auto 1fr auto auto;
}

.col-center {
  grid-template-rows: 1fr;
}

.col-right {
  grid-template-rows: auto 1fr auto auto;
}

.right-lower-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  column-gap: 16px;
  align-items: end;
}

.panel {
  background: var(--panel-bg);
  backdrop-filter: blur(var(--panel-blur));
  -webkit-backdrop-filter: blur(var(--panel-blur));
  border-radius: var(--radius);
  padding: 12px 14px;
  box-sizing: border-box;
}

.clamp-box { overflow: hidden; }
```

### コンポーネント定義（11種類）

| type | 説明 | 推奨slot |
|------|------|---------|
| `ClockWidget` | 時刻/日付表示 | left.top |
| `WeatherWidget` | 天気情報 | left.topBelow |
| `ChatLog` | コメント表示 | left.middle |
| `SuperChatCard` | スパチャ表示 | left.lower |
| `BrandBlock` | ロゴ/ブランド | left.bottom |
| `MainAvatarStage` | 主役ステージ | center.full |
| `ChannelBadge` | チャンネルバッジ | right.top |
| `SetList` | セットリスト | right.upper |
| `KPIBlock` | KPI数値表示 | right.lowerLeft |
| `QueueList` | 待機キュー | right.lowerRight |
| `PromoPanel` | 告知/プロモ | right.bottom |

### 実装ルール

#### 更新間引き（必須）
- コメント高頻度時は100〜200ms単位でまとめて反映
- requestAnimationFrameまたはバッチ反映

#### 上限値の強制（必須）
- ChatLog: `maxLines` 必須（推奨10 / 範囲 4〜14）
- SetList: `maxItems` 必須（推奨14 / 範囲 6〜20）
- QueueList: `maxItems` 必須（推奨6 / 範囲 3〜10）

#### 右下過密対策（必須）
- PromoPanel: `displayMode=cycle` デフォルト（cycleSec=30 / showSec=6）
- QueueList: `showWhenNotEmpty=true` デフォルト

### URLパラメータ（統合オーバーレイ専用）

```
http://localhost:19800/overlay/combined?layout=streaming&commentEnabled=true&setlistEnabled=true
```

| パラメータ | 型 | デフォルト | 説明 |
|------------|-----|------------|------|
| layout | string | 'streaming' | レイアウトプリセット |
| commentEnabled | boolean | true | コメント表示ON/OFF |
| setlistEnabled | boolean | true | セットリスト表示ON/OFF |
| primaryColor | string | '#6366f1' | プライマリカラー（#RRGGBB） |
| commentFontSize | number | 16 | コメントフォントサイズ（px） |
| setlistFontSize | number | 24 | セットリストフォントサイズ（px） |
| showAvatar | boolean | true | アバター表示 |
| showArtist | boolean | true | アーティスト表示 |

### 推奨設定（OBS）

| 項目 | 値 |
|------|-----|
| 幅 | 1920px |
| 高さ | 1080px |
| FPS | 30 |

統合オーバーレイはOBS画面全体（1920x1080）を前提とした設計。

---

## OBS設定ガイド

### 推奨設定

| 項目 | コメント | セットリスト | 統合オーバーレイ |
|------|----------|--------------|------------------|
| 幅 | 400px | 350px | 1920px |
| 高さ | 600px | 200px | 1080px |
| FPS | 30 | 30 | 30 |
| CSS | カスタム可 | カスタム可 | カスタム可 |

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
http://localhost:19800/overlay/comment?theme=dark&fontSize=16
http://localhost:19800/overlay/setlist?showArtist=false&position=bottom
```

| パラメータ | 型 | デフォルト | 説明 |
|------------|-----|------------|------|
| theme | string | 'default' | テーマ名 |
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
    showAvatar: boolean;
    showBadges: boolean;
    avatarSize: number;
    fontSize: number;
    animationSpeed: 'slow' | 'normal' | 'fast';
    // NOTE: maxCountは画面高さベースの自動調整に統一したため削除
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
