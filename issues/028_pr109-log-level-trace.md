# PR#109 定期実行ログのログレベル指針

## 概要

定期的に実行される関数（30秒ごとなど）では、ログが大量になる可能性があるため、適切なログレベルを選択する必要がある。

## 指摘内容

PR#109のレビューで以下の指摘を受けた:

> `log::debug!` が複数箇所にありますが、30秒ごとに呼ばれるためログが多くなる可能性があります。本番前に `trace!` レベルへの変更を検討してください。

## ログレベルの使い分け

| レベル | 用途 | 例 |
|--------|------|-----|
| `trace` | 非常に詳細なデバッグ情報、定期実行の開始/終了 | 30秒ごとの視聴者数取得ログ |
| `debug` | デバッグ情報、重要な状態変化 | 初期化完了、モード切替 |
| `info` | 通常運用で有用な情報 | サーバー起動、設定変更 |
| `warn` | 警告（継続可能だが注意が必要） | リトライ発生、一時的エラー |
| `error` | エラー（処理失敗） | 致命的エラー、回復不能な状態 |

## 修正内容

```rust
// Before（debug - 30秒ごとにログ大量発生）
log::debug!("Fetching viewer count via InnerTube: video_id={}", video_id);

// After（trace - 必要な時のみ表示）
log::trace!("Fetching viewer count via InnerTube: video_id={}", video_id);
```

## 判断基準

1. **定期実行（< 1分間隔）の開始/終了ログ** → `trace`
2. **状態変化や重要なイベント** → `debug` または `info`
3. **エラーや警告** → `warn` または `error`

## 対象ファイル

- `src-tauri/src/commands/youtube.rs`
  - `fetch_and_broadcast_viewer_count` (30秒ごと)
  - `fetch_viewer_count_innertube` (30秒ごと)

## チェックリスト

新しい定期実行関数を追加する際:
- [ ] 定期実行の開始/終了ログは`trace`レベルを使用
- [ ] エラー発生時は`warn`以上を使用
- [ ] 重要な状態変化は`debug`または`info`を使用

## 関連Issue

- `019_innertube-polling-interval.md` - ポーリング間隔のログレベル指針
