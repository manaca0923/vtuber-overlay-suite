# InnerTube APIのContinuation型とポーリング間隔

## 概要

InnerTube APIのポーリング間隔を最適化する際に発見した知見。
APIレスポンスの`timeoutMs`は一律に扱うべきではなく、Continuation種別によって意味が異なる。

## 調査結果

### Continuation種別

InnerTube APIは3種類のContinuationデータを返す:

| 種別 | 構造 | timeoutMsの意味 | 短縮可否 |
|------|------|-----------------|----------|
| `invalidationContinuationData` | `timeoutMs` はOptional | 推奨ポーリング間隔 | 可能（1〜5秒程度） |
| `timedContinuationData` | `timeoutMs` は必須フィールド | 明示的な待機指示 | 不可（仕様違反リスク） |
| `liveChatReplayContinuationData` | `timeUntilLastMessageMsec` | リプレイ用タイミング | 可能（初期化用途） |

### timedContinuationDataを短縮してはいけない理由

1. **フィールドが必須**: `timeoutMs`がOptionalではなく必須フィールドとして定義されている
2. **明示的な指示**: YouTubeサーバーが「この時間待て」と明示している
3. **仕様違反リスク**: 短縮するとレート制限やブロックの可能性がある
4. **参考実装**: `pytchat`, `chat-downloader`などの実装でも`timedContinuationData`は尊重している

### invalidationContinuationDataは短縮可能な理由

1. **フィールドがOptional**: `timeoutMs`が存在しない場合もある
2. **推奨値**: あくまで「この間隔を推奨する」という意味合い
3. **サーバーサイド無効化**: サーバーがデータを無効化した時に新しいデータを取得する仕組み
4. **実績**: 多くの非公式クライアントで短縮されている

## 実装

```rust
// src-tauri/src/youtube/innertube/types.rs
pub enum ContinuationType {
    Invalidation,  // 推奨間隔（短縮可能）
    Timed,         // 明示的待機（厳守）
    Reload,        // 初期化・リプレイ用
}

// src-tauri/src/youtube/unified_poller.rs
let timeout_ms = match continuation_type {
    ContinuationType::Invalidation => api_timeout.clamp(1000, 5000),
    ContinuationType::Timed => api_timeout,  // APIの指示を厳守
    ContinuationType::Reload => 1000,
};
```

## 参考実装

- [pytchat](https://github.com/taizan-hokuto/pytchat): Python製YouTubeライブチャット取得ライブラリ
- [chat-downloader](https://github.com/xenova/chat-downloader): YouTube/Twitch等のチャットダウンローダー

## 関連PR

- PR#99: InnerTubeポーリング間隔のContinuation型別最適化

## 教訓

1. **APIの値を盲目的に短縮しない**: 間隔を短縮する前に、その値の意味を調査する
2. **フィールドの必須/Optional区別**: 必須フィールドは「指示」、Optionalは「推奨」の可能性が高い
3. **参考実装の確認**: 同じAPIを使う他のOSSプロジェクトの実装を確認する
4. **ドキュメントとの整合性**: 実装を変更したらドキュメントも必ず更新する
