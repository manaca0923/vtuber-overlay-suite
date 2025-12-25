# オーバーレイセキュリティのノウハウ

## 概要
PR #23, #24, #33, #36 で受けたオーバーレイのセキュリティ関連指摘と解決方法

---

## 1. URLパラメータのバリデーション

### 指摘内容 (PR#23)
URLパラメータをそのままCSSプロパティに設定するとCSSインジェクションの可能性がある。

### 解決方法
```javascript
// カラーコードのバリデーション
function isValidHexColor(color) {
  return /^#[0-9A-Fa-f]{6}$/.test(color);
}

// 数値のバリデーション
function isValidNumber(value, min, max) {
  const num = parseInt(value, 10);
  return !isNaN(num) && num >= min && num <= max;
}

// フォントファミリーのサニタイズ
function sanitizeFontFamily(font) {
  return font.replace(/[;{}()]/g, '');
}

// 使用例
if (params.get('primaryColor') && isValidHexColor(params.get('primaryColor'))) {
  root.style.setProperty('--primary-color', params.get('primaryColor'));
}
```

### 今後の対策
- URLパラメータは必ずバリデーションしてから使用
- 許可するパターンを正規表現で明示
- CSS値として使う場合は特に注意

---

## 2. 絵文字URLのドメイン検証

### 指摘内容 (PR#24)
InnerTube APIから取得したURLを直接使用すると、悪意のあるURLがレスポンスに含まれる可能性がある。

### 解決方法
```javascript
function isValidEmojiUrl(url) {
  if (!url || typeof url !== 'string') return false;

  // プロトコル相対URLの正規化
  let normalizedUrl = url;
  if (url.startsWith('//')) {
    normalizedUrl = 'https:' + url;
  }

  try {
    const parsed = new URL(normalizedUrl);

    // 許可するドメインのホワイトリスト
    const allowedHosts = [
      'yt3.ggpht.com',
      'yt4.ggpht.com',
      'lh3.googleusercontent.com',
      'fonts.gstatic.com',
    ];

    // サフィックスマッチ（*.ggpht.com など）
    const allowedSuffixes = [
      '.ggpht.com',
      '.googleusercontent.com',
      '.ytimg.com',
    ];

    const isAllowedHost = allowedHosts.includes(parsed.host);
    const isAllowedSuffix = allowedSuffixes.some(suffix =>
      parsed.host.endsWith(suffix)
    );

    return isAllowedHost || isAllowedSuffix;
  } catch {
    return false;
  }
}
```

### 今後の対策
- 外部APIから取得するURLは必ずドメイン検証
- ホワイトリスト方式で許可ドメインを明示
- 画像読み込み失敗時のフォールバック処理を用意

---

## 3. 絵文字画像の遅延読み込み

### 指摘内容 (PR#24)
多数の絵文字画像リクエストが集中する可能性がある。

### 解決方法
```javascript
const img = document.createElement('img');
img.loading = 'lazy';  // 遅延読み込み
img.src = thumb.url;
img.alt = emoji.shortcuts[0] || '';

// 画像読み込み失敗時のフォールバック
img.onerror = function() {
  // テキスト表示にフォールバック
  const text = document.createElement('span');
  text.textContent = emoji.shortcuts[0] || '';
  img.replaceWith(text);
};
```

### 今後の対策
- 画像には`loading="lazy"`を設定
- `onerror`ハンドラでフォールバック処理を実装
- 可能であれば画像のプリロードを検討

---

## 4. XSS対策（textContent vs innerHTML）

### 指摘内容 (PR#52)
`textContent`への代入はHTMLを解釈しないため、`escapeHtml()`は不要。二重エスケープの原因になる。

### 解決方法
```javascript
// NG: 二重エスケープになる
li.textContent = this.escapeHtml(text);

// OK: textContentはHTMLを解釈しない
li.textContent = text;

// OK: innerHTMLを使う場合のみescapeHtml
li.innerHTML = this.escapeHtml(text);

// escapeHtml関数
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
```

### 今後の対策
- `textContent`を使用する場合は`escapeHtml`不要
- `innerHTML`を使用する場合のみ`escapeHtml`を適用
- コメントで理由を明記する

---

## チェックリスト（オーバーレイ実装時）

- [ ] URLパラメータはバリデーション済みか
- [ ] 外部URLはドメイン検証済みか
- [ ] 画像には`loading="lazy"`を設定したか
- [ ] 画像読み込み失敗時のフォールバックがあるか
- [ ] `textContent`と`innerHTML`の使い分けは正しいか
- [ ] `escapeHtml`は必要な箇所のみで使用しているか
