# PR #52 レビュー指摘事項ノウハウ

## 概要
T23: 新コンポーネント追加（8コンポーネント + コンポーネント管理システム）で受けた指摘と解決方法

---

## 1. スクリプト読み込み順序

### 指摘内容
`base-component.js`は他のコンポーネントが継承するクラスなので、`component-registry.js`より**先に**読み込むべき。

### 解決方法
```html
<!-- 正しい順序 -->
<script src="components/base-component.js"></script>
<script src="shared/component-registry.js"></script>
```

### 今後の対策
- 継承関係のあるクラスは、親クラスを先に読み込む
- コメントで読み込み順序の理由を明記する

---

## 2. setTimeout/setIntervalの管理

### 指摘内容
`setTimeout`が`destroy()`時にクリアされないと、コンポーネント破棄後にコールバックが実行される可能性がある。

### 解決方法
```javascript
// コンストラクタで初期化
this.showTimeoutId = null;

// 使用時に保存
this.showTimeoutId = setTimeout(() => { ... }, 300);

// destroy()でクリア
destroy() {
  if (this.showTimeoutId) {
    clearTimeout(this.showTimeoutId);
    this.showTimeoutId = null;
  }
  super.destroy();
}
```

### 今後の対策
- タイマーを使用する場合は必ずIDを保存
- `destroy()`で確実にクリアする
- BaseComponentの`setInterval()`ヘルパーを活用（自動クリーンアップ付き）

---

## 3. URLバリデーション（セキュリティ）

### 指摘内容
画像URLに`javascript:`スキームなど危険なURLが設定される可能性がある。

### 解決方法
```javascript
validateUrl(url) {
  if (!url || typeof url !== 'string') return '';
  try {
    const parsed = new URL(url, window.location.href);
    // http, https, data スキームのみ許可
    if (['http:', 'https:', 'data:'].includes(parsed.protocol)) {
      return url;
    }
    console.warn('無効なURLスキーム:', parsed.protocol);
    return '';
  } catch (e) {
    // 相対パスの場合は許可
    if (url.startsWith('/') || url.startsWith('./') || url.startsWith('../')) {
      return url;
    }
    console.warn('無効なURL:', url);
    return '';
  }
}
```

### 今後の対策
- 外部から受け取るURLは必ずバリデーション
- 許可するスキームを明示的にホワイトリスト化
- `javascript:`, `vbscript:`, `data:text/html`等を拒否

---

## 4. escapeHtmlの二重エスケープ

### 指摘内容
`textContent`への代入はHTMLを解釈しないため、`escapeHtml()`は不要。二重エスケープの原因になる。

### 解決方法
```javascript
// NG: 二重エスケープになる
li.textContent = this.escapeHtml(text);

// OK: textContentはHTMLを解釈しない
li.textContent = text;

// OK: innerHTMLを使う場合のみescapeHtml
li.innerHTML = this.escapeHtml(text);
```

### 今後の対策
- `textContent`を使用する場合は`escapeHtml`不要
- `innerHTML`を使用する場合のみ`escapeHtml`を適用
- コメントで理由を明記する

---

## 5. toLocaleTimeStringのhour12オプション

### 指摘内容
`hour12`オプションが未指定だと、ブラウザ/ロケールによって12時間/24時間表示が変わる。

### 解決方法
```javascript
const timeOptions = {
  hour: '2-digit',
  minute: '2-digit',
  hour12: false,  // 24時間表示を保証
};
```

### 今後の対策
- 時刻表示では`hour12`を明示的に指定
- ロケール依存の挙動は避ける

---

## 6. DOM要素のnullチェック

### 指摘内容
条件によってDOM要素が生成されない場合、アクセス前にnullチェックが必要。

### 解決方法
```javascript
update(data) {
  if (data.showDate !== undefined) {
    this.showDate = data.showDate;
    if (this.dateEl) {  // nullチェック追加
      this.dateEl.style.display = data.showDate ? '' : 'none';
    }
  }
}
```

### 今後の対策
- オプショナルなDOM要素はアクセス前にnullチェック
- TypeScriptの場合は`?.`オプショナルチェーンを活用

---

## チェックリスト（新規コンポーネント作成時）

- [ ] スクリプト読み込み順序は正しいか
- [ ] setTimeout/setIntervalはdestroy()でクリアされるか
- [ ] 外部URLはバリデーションされているか
- [ ] textContentにescapeHtmlを使っていないか
- [ ] 時刻表示でhour12は明示されているか
- [ ] オプショナルなDOM要素にnullチェックがあるか
