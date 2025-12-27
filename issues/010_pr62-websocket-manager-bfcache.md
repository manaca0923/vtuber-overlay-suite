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

### 中: clearTimeoutをtry/finallyで確実に実行 (Codex Review 2回目)

**問題**:
```javascript
// 修正前
async function fetchLatestSetlist(...) {
  try {
    const timeoutId = setTimeout(...);
    const response = await fetch(...);
    clearTimeout(timeoutId);  // 例外発生時はここに到達しない
    ...
  } catch (e) {
    // clearTimeoutが呼ばれない
  }
}
```

例外やAbort時にタイマーが残り、連続呼び出しでタイマーが増殖する。

**対応**:
```javascript
// 修正後
async function fetchLatestSetlist(...) {
  const timeoutId = setTimeout(...);
  try {
    const response = await fetch(...);
    ...
  } catch (e) {
    ...
  } finally {
    clearTimeout(timeoutId);  // 必ず実行される
  }
}
```

### 中: updateSetlistDisplayの要素単位ガード (Codex Review 2回目)

**問題**:
```javascript
// 修正前
if (!prevEl || !currentEl || !nextEl) {
  return;  // いずれか欠落で全体が停止
}
```

prev/nextがないレイアウトでも現在曲表示まで停止する。

**対応**:
```javascript
// 修正後
if (prevEl) { /* prev更新 */ }
if (currentEl) { /* current更新 */ }
if (nextEl) { /* next更新 */ }
```

各要素を独立してガードすることで、部分的なレイアウトでも動作する。

## 学んだこと

### 4. タイマーのtry/finallyパターン

AbortControllerとsetTimeoutを組み合わせる場合:
- `clearTimeout`は必ず`finally`ブロックで実行
- 例外やAbort時もタイマーをクリーンアップ

```javascript
const timeoutId = setTimeout(() => controller.abort(), timeout);
try {
  await fetch(...);
} finally {
  clearTimeout(timeoutId);  // 必ず実行
}
```

### 5. DOM要素の部分欠落への対応

レイアウトによって一部のDOM要素が存在しない場合:
- 全体をreturnするのではなく、要素単位でガード
- 各要素の更新を独立させて部分的な動作を維持

### 中: SettingsFetcherのbfcache復元時リセット (Codex Review 3回目)

**問題**:
```javascript
// combined-v2.html (修正前)
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    wsManager.reinitialize();  // settingsFetcherはhasFetched=trueのまま
  }
});
```

bfcache復元時に`settingsFetcher.hasFetched()`が`true`のままだと、`onOpen`での再取得がスキップされる可能性がある。

**対応**:
```javascript
// overlay-core.js - reset()メソッド追加
reset() {
  this.fetchSucceeded = false;
}

// combined-v2.html - pageshow時にreset()を呼ぶ
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    settingsFetcher.reset();  // 追加
    wsManager.reinitialize();
  }
});
```

### 中: fetchLatestSetlistのtimeoutバリデーション (Codex Review 3回目)

**問題**:
```javascript
// 修正前
async function fetchLatestSetlist(apiBaseUrl = API_BASE_URL, onUpdate, timeout = SETTINGS_FETCH_TIMEOUT) {
  const timeoutId = setTimeout(() => controller.abort(), timeout);
  // timeout=0やundefinedで即時Abort
}
```

呼び出し元が`timeout`を明示的に渡さない場合、または0/負値を渡した場合に即時Abortになる。

**対応**:
```javascript
// 修正後
async function fetchLatestSetlist(apiBaseUrl = API_BASE_URL, onUpdate, timeout = SETTINGS_FETCH_TIMEOUT) {
  const timeoutMs = Number.isFinite(timeout) && timeout > 0 ? timeout : SETTINGS_FETCH_TIMEOUT;
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
}
```

### 6. 状態管理クラスのbfcache対応

bfcacheからの復元時、状態管理クラスの内部フラグも適切にリセットする必要がある:
- `hasFetched`などの「完了済み」フラグはリセット
- 次回のフェッチで必ず再取得されるようにする

### 7. 引数のデフォルト値とフォールバック

デフォルト引数だけでは不十分な場合がある:
- 呼び出し元が明示的に`undefined`や無効値を渡す可能性
- `Number.isFinite()`と正値チェックでフォールバック

```javascript
// 安全なフォールバックパターン
const value = Number.isFinite(input) && input > 0 ? input : DEFAULT_VALUE;
```

### 中: SettingsFetcherのtimeoutバリデーション漏れ (Codex Review 4回目)

**問題**:
```javascript
// fetchLatestSetlistにはバリデーションを追加したが、SettingsFetcherには未対応だった
class SettingsFetcher {
  async fetchAndApply() {
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);
    // this.timeoutが0/undefined/負値の場合に即時Abort
  }
}
```

**対応**:
共通関数`validateTimeout()`を追加し、両方の箇所で使用するように統一:
```javascript
// 共通バリデーション関数
function validateTimeout(timeout, defaultValue = SETTINGS_FETCH_TIMEOUT) {
  return Number.isFinite(timeout) && timeout > 0 ? timeout : defaultValue;
}

// SettingsFetcher.fetchAndApply
const timeoutMs = validateTimeout(this.timeout);

// fetchLatestSetlist
const timeoutMs = validateTimeout(timeout);
```

### 8. バリデーションロジックの共通化

同じバリデーションを複数箇所で行う場合:
- 共通関数に抽出して一貫性を保つ
- 修正漏れを防止
- コードの重複を削減

### 中: reset()でのfetchInFlightリセット漏れ (Codex Review 5回目)

**問題**:
```javascript
// reset()がfetchSucceededのみリセットしていた
reset() {
  this.fetchSucceeded = false;
  // fetchInFlightがtrueのままだと、fetchAndApply()が即returnして再取得されない
}
```

bfcacheからの復元時、fetchリクエストが中断されて`fetchInFlight=true`のままになっている可能性がある。

**対応**:
```javascript
reset() {
  this.fetchSucceeded = false;
  this.fetchInFlight = false;  // 追加
}
```

### 9. 状態リセット時のフラグ完全性

状態をリセットするメソッドでは、関連するすべてのフラグをリセットする:
- 「実行済み」フラグ（`fetchSucceeded`）
- 「実行中」フラグ（`fetchInFlight`）
- 片方だけリセットすると、もう片方の状態が残って不整合が発生する

## チェックリスト（今後の対応）

- [x] reinitialize()での二重接続防止
- [x] cleanup()でのoncloseハンドラ無効化
- [x] clearTimeoutをtry/finallyに移動
- [x] updateSetlistDisplayの要素単位ガード
- [x] SettingsFetcherのreset()メソッド追加
- [x] combined-v2.htmlでのbfcache復元時settingsFetcher.reset()呼び出し
- [x] fetchLatestSetlistのtimeoutバリデーション
- [x] SettingsFetcherのtimeoutバリデーション
- [x] validateTimeout()共通関数の追加
- [x] SettingsFetcher.reset()でfetchInFlightもリセット
- [ ] combined.htmlへのbfcacheハンドリング追加（低優先度、OBS以外のブラウザ向け）
