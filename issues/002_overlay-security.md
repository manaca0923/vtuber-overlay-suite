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

## 5. postMessageのtargetOrigin明示化

### 指摘内容 (PR#103)
`postMessage`の第2引数で`'*'`を使用すると、意図しない受信者にメッセージが送られるリスクがある。

### 解決方法
```typescript
// NG: '*'はセキュリティ上のリスク
iframeRef.current.contentWindow.postMessage(message, '*');

// OK: targetOriginを明示
const PREVIEW_ORIGIN = 'http://localhost:19800';
iframeRef.current.contentWindow.postMessage(message, PREVIEW_ORIGIN);
```

### 今後の対策
- `postMessage`使用時は常にtargetOriginを明示
- 定数として定義し、送信先を明確化
- 受信側（オーバーレイ）では`event.origin`を検証（issues/020参照: `TRUSTED_ORIGINS`）

### 関連
- 受信側のorigin検証: `overlay-core.js`の`PostMessageHandler`クラス
- 信頼できるオリジンリスト: `TRUSTED_ORIGINS`定数

---

## 6. 深層防御（フロントエンド・バックエンド両方でのバリデーション）

### 指摘内容 (PR#106)
フロントエンドでバリデーションしていても、バックエンドでもバリデーションが必要。

### 解決方法

**フロントエンド側（TypeScript）**
```typescript
// FontSelector.tsx
function sanitizeFontFamily(fontFamily: string | null | undefined): string | null {
  if (typeof fontFamily !== 'string' || fontFamily.length === 0 || fontFamily.length > 200) {
    return null;
  }
  // 危険な文字を除去
  return fontFamily.replace(/[<>"'`;{}]/g, '');
}
```

**バックエンド側（Rust）**
```rust
// system.rs - システムフォント取得時のフィルタリング
const MAX_FONT_NAME_LENGTH: usize = 200;

let mut fonts: Vec<String> = families
    .into_iter()
    .filter(|name| {
        !name.is_empty()
            && name.len() <= MAX_FONT_NAME_LENGTH
            && !name.chars().any(|c| c.is_control())
    })
    .collect();
```

**バックエンド検証（Rust）**
```rust
// overlay.rs - 設定保存時の検証
fn validate_overlay_settings(settings: &OverlaySettings) -> Result<(), String> {
    if let Some(ref theme) = settings.theme_settings {
        // カスタムカラーの上限チェック（最大3件）
        const MAX_CUSTOM_COLORS: usize = 3;
        if theme.custom_colors.len() > MAX_CUSTOM_COLORS {
            return Err(format!("Too many custom colors: {}. Maximum is {}.",
                theme.custom_colors.len(), MAX_CUSTOM_COLORS));
        }

        // グローバルプライマリカラーの検証
        if !is_valid_hex_color(&theme.global_primary_color) {
            return Err(format!("Invalid globalPrimaryColor: {}.",
                theme.global_primary_color));
        }
    }
    Ok(())
}
```

### 今後の対策
- フロントエンドでバリデーションしていても、バックエンドでも必ず検証
- 「深層防御」の原則: 複数レイヤーでセキュリティ対策
- 定数（上限値など）はフロントエンド・バックエンド両方で定義

---

## 7. Google Fonts URLのバリデーション

### 指摘内容 (PR#106)
Google Fontsを動的に読み込む際、URLのホスト名を検証する必要がある。

### 解決方法
```typescript
// FontSelector.tsx / combined-v2.html
function loadGoogleFont(fontSpec: string): void {
  if (!fontSpec || loadedGoogleFonts.has(fontSpec)) return;

  const url = `https://fonts.googleapis.com/css2?family=${fontSpec}&display=swap`;

  // セキュリティチェック: ホスト名を検証
  try {
    const parsed = new URL(url);
    if (parsed.hostname !== 'fonts.googleapis.com') return;
  } catch {
    return;
  }

  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = url;
  document.head.appendChild(link);
  loadedGoogleFonts.add(fontSpec);
}
```

### 今後の対策
- 外部リソース読み込み時はホスト名を検証
- 許可するドメインをホワイトリストで管理
- 重複読み込み防止のためSetで管理

---

## チェックリスト（オーバーレイ実装時）

- [ ] URLパラメータはバリデーション済みか
- [ ] 外部URLはドメイン検証済みか
- [ ] 画像には`loading="lazy"`を設定したか
- [ ] 画像読み込み失敗時のフォールバックがあるか
- [ ] `textContent`と`innerHTML`の使い分けは正しいか
- [ ] `escapeHtml`は必要な箇所のみで使用しているか
- [ ] `postMessage`のtargetOriginは明示されているか
- [ ] フロントエンド・バックエンド両方でバリデーションしているか（深層防御）
- [ ] カスタム値の上限はバックエンドでも検証しているか
- [ ] 外部リソース（Google Fonts等）のURLホスト名を検証しているか
