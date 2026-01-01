# PR#115 キュー管理機能レビュー

## 概要
PR#115で受けた指摘と解決方法

---

## 1. 非同期リクエストの競合対策（シーケンス番号パターン）

### 指摘内容
debounceで保存するタイトル入力において、古いリクエストの完了が最新のタイトルを上書きする可能性がある。

### 問題のあるコード
```typescript
// ❌ 危険: 古いリクエストが後から完了すると最新値を上書き
const handleTitleSave = async (title: string) => {
  const updated = await invoke('set_queue_title', { title });
  setQueueState(updated);  // 古いリクエストの結果で上書きされる可能性
};
```

### 解決方法: シーケンス番号パターン
```typescript
const titleSaveSeqRef = useRef(0);

const handleTitleSave = async (title: string) => {
  // シーケンス番号をインクリメント
  titleSaveSeqRef.current += 1;
  const currentSeq = titleSaveSeqRef.current;

  try {
    const updated = await invoke('set_queue_title', { title });

    // 最新のリクエストのみ状態を更新
    if (currentSeq === titleSaveSeqRef.current) {
      setQueueState(updated);
      await invoke('broadcast_queue_update', { queueState: updated });
    }
  } catch (err) {
    // 最新のリクエストのみエラーを表示
    if (currentSeq === titleSaveSeqRef.current) {
      setError('タイトルの変更に失敗しました');
    }
  } finally {
    // 最新のリクエストのみsaving状態を解除
    if (currentSeq === titleSaveSeqRef.current) {
      setSaving(false);
    }
  }
};
```

### 適用場面
- debounce付き入力の保存処理
- 連続して発生する非同期リクエスト
- 最新の結果のみをUIに反映したい場合

---

## 2. 旧データ互換性のマイグレーションパターン

### 指摘内容
既存データに`id: None`が含まれる場合、`remove_queue_item`で削除不能になる。

### 解決方法: 読み込み時の正規化マイグレーション
```rust
pub async fn get_queue_state(state: tauri::State<'_, AppState>) -> Result<QueueState, String> {
    let mut queue_state: QueueState = /* DBから読み込み */;

    // 旧データ互換性: id: None のアイテムにUUIDを付与
    let mut needs_save = false;
    for item in &mut queue_state.items {
        if item.id.is_none() {
            item.id = Some(Uuid::new_v4().to_string());
            needs_save = true;
        }
    }

    // 正規化したデータを保存（マイグレーション）
    if needs_save {
        log::info!("Migrating queue items: assigning UUIDs to items without id");
        // DBに保存
    }

    Ok(queue_state)
}
```

### 設計ノート
- 読み込み時に自動マイグレーション（lazy migration）
- ユーザーに意識させずにデータを正規化
- マイグレーション発生時はログで記録

---

## チェックリスト

### 非同期リクエスト実装時
- [ ] 連続リクエストで競合が発生しないか
- [ ] 古いリクエストの結果で最新値が上書きされないか
- [ ] シーケンス番号パターンの適用を検討

### データスキーマ変更時
- [ ] 旧データとの互換性は保たれるか
- [ ] `None`/`null`値の扱いは明確か
- [ ] 必要に応じてマイグレーション処理を実装
