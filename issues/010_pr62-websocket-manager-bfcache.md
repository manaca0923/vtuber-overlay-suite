# PR#62 WebSocketManager bfcache対応のレビュー指摘

## 概要

WebSocketManagerを共通化した際の、bfcache復元時の二重接続問題。

## 指摘事項

### P1: bfcache復元時の二重接続 (Codex Review)

**問題**:
```javascript
// 修正前のreinitialize()
reinitialize() {
  this.isShuttingDown = false;
  this.reconnectDelay = INITIAL_RECONNECT_DELAY;
  this.connect();  // 既存タイマーや接続状態をチェックせずに接続
}
```

bfcacheからページが復元された時:
1. `ws.onclose`がまだ実行されておらず、再接続タイマーが残っている可能性
2. または既にCONNECTING/OPEN状態の接続が存在する可能性

→ タイマー発火 + `reinitialize()`の両方が接続を試み、二重接続になる

**対応**:
```javascript
// 修正後
reinitialize() {
  this.isShuttingDown = false;
  this.reconnectDelay = INITIAL_RECONNECT_DELAY;

  // ペンディング中の再接続タイマーをクリア（二重接続防止）
  if (this.reconnectTimerId) {
    clearTimeout(this.reconnectTimerId);
    this.reconnectTimerId = null;
  }

  // 既存の接続がある場合はスキップ（二重接続防止）
  if (this.ws && (this.ws.readyState === WebSocket.CONNECTING || this.ws.readyState === WebSocket.OPEN)) {
    console.log('WebSocket already connected or connecting, skipping reinitialize');
    return;
  }

  this.connect();
}
```

### 中: cleanup()でのoncloseハンドラ無効化 (Claude Review)

**問題**:
```javascript
// 修正前
cleanup() {
  this.isShuttingDown = true;
  if (this.ws) {
    this.ws.close();  // oncloseハンドラがまだ有効
    this.ws = null;
  }
}
```

`isShuttingDown`フラグで再接続は防げるが、不要なログ出力が発生する。

**対応**:
```javascript
// 修正後
cleanup() {
  this.isShuttingDown = true;
  if (this.ws) {
    // oncloseハンドラを無効化して不要なログ/再接続を防止
    this.ws.onclose = null;
    this.ws.close();
    this.ws = null;
  }
}
```

## 学んだこと

### 1. bfcacheとWebSocket接続の競合

bfcache（Back/Forward Cache）からページが復元される際:
- JavaScriptの状態は保持される
- しかしWebSocket接続は切断されている可能性がある
- 再接続タイマーが残っている可能性がある

→ 復元時は必ずタイマーと接続状態をチェックする

### 2. WebSocket接続の二重防止パターン

接続を開始する前に:
1. ペンディング中のタイマーをクリア
2. 既存接続の`readyState`をチェック（CONNECTING または OPEN）
3. 上記に該当しない場合のみ接続

```javascript
// 二重接続防止の標準パターン
if (this.reconnectTimerId) {
  clearTimeout(this.reconnectTimerId);
  this.reconnectTimerId = null;
}
if (this.ws && (this.ws.readyState === WebSocket.CONNECTING || this.ws.readyState === WebSocket.OPEN)) {
  return;  // 既に接続中
}
this.connect();
```

### 3. イベントハンドラのクリーンアップ

`close()`を呼ぶ前に`onclose = null`を設定することで:
- 不要なハンドラ実行を防止
- 意図しないログ出力を回避
- 再接続タイマーのスケジューリングを防止

## チェックリスト（今後の対応）

- [x] reinitialize()での二重接続防止
- [x] cleanup()でのoncloseハンドラ無効化
- [ ] combined.htmlへのbfcacheハンドリング追加（低優先度、OBS以外のブラウザ向け）
- [ ] SettingsFetcherのhasFetched()リセット機能（低優先度）
