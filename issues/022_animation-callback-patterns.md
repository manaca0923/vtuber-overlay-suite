# アニメーションコールバックのパターン

## 概要

PR#104（スパチャウィジェット実装）で指摘されたアニメーションコールバックの二重実行リスクと対策。

## 問題

`animationend` イベントリスナーとフォールバックの `setTimeout` の両方が同じコールバックを実行する可能性がある。

```javascript
// 問題のあるパターン
el.addEventListener('animationend', () => {
  callback();  // 1回目の実行
}, { once: true });

setTimeout(() => {
  if (el.parentNode) {
    callback();  // 2回目の実行（アニメーション完了後500ms以内に発火）
  }
}, 500);
```

## 解決方法

### パターン1: 二重実行防止フラグ（推奨）

複雑なコールバック処理（状態更新、キュー処理など）がある場合に使用。

```javascript
_animateOut(el, callback) {
  el.classList.remove('visible');
  el.classList.add('removing');

  // 二重実行防止フラグ
  let callbackExecuted = false;
  const safeCallback = () => {
    if (callbackExecuted) return;
    callbackExecuted = true;
    callback();
  };

  el.addEventListener(
    'animationend',
    () => safeCallback(),
    { once: true }
  );

  // フォールバック
  setTimeout(() => {
    if (el.parentNode) {
      safeCallback();
    }
  }, 500);
}
```

### パターン2: 冪等な操作（単純な場合）

コールバックが `element.remove()` のみのような冪等な操作の場合は、フラグ不要。

```javascript
// element.remove()は2回呼んでも安全
function removeCommentWithAnimation(element) {
  if (!element || element.classList.contains('removing')) return;

  element.classList.add('removing');
  element.addEventListener('animationend', () => {
    element.remove();  // 冪等な操作
  }, { once: true });

  setTimeout(() => {
    if (element.parentNode) {
      element.remove();  // 既に削除済みなら何も起きない
    }
  }, 500);
}
```

## 使い分け

| 状況 | 推奨パターン |
|------|------------|
| コールバックが状態を変更する | パターン1（フラグ使用） |
| コールバックがキュー処理を行う | パターン1（フラグ使用） |
| コールバックが`element.remove()`のみ | パターン2（そのまま） |
| コールバックが冪等な操作のみ | パターン2（そのまま） |

## 対象ファイル

- `src-tauri/overlays/components/superchat-card.js` - パターン1を適用済み
- `src-tauri/overlays/shared/comment-renderer.js` - パターン2（冪等操作）

## 関連PR

- PR#104: スパチャ専用ウィジェット実装
