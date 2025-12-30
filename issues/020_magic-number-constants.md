# マジックナンバーの定数化

## 概要

同じ意味を持つ数値が複数箇所に散らばっている場合、定数として一元化すべき。
変更時の漏れを防ぎ、コードの意図を明確にする。

## 問題例

```rust
// 悪い例: 同じ値が複数箇所にハードコード
// unified_poller.rs
buffer_interval_ms: Some(1000), // InnerTubeは1秒バッファ

// youtube.rs
buffer_interval_ms: Some(1000), // InnerTubeは1秒バッファ
```

## 解決方法

```rust
// 良い例: 定数として一元化
// innertube/types.rs
/// InnerTubeモードのバッファ間隔（1秒）
/// オーバーレイ側で1秒間に取得したコメントを等間隔で表示するための設定
pub const INNERTUBE_BUFFER_INTERVAL_MS: u32 = 1000;

// 使用箇所
use crate::youtube::innertube::INNERTUBE_BUFFER_INTERVAL_MS;
buffer_interval_ms: Some(INNERTUBE_BUFFER_INTERVAL_MS),
```

## 定数化の判断基準

| 状況 | 定数化すべきか |
|------|--------------|
| 同じ値が2箇所以上で使用 | ✅ 必須 |
| 値の意味を説明するコメントが必要 | ✅ 推奨 |
| 将来変更される可能性がある | ✅ 推奨 |
| 1箇所のみで使用、変更の可能性低い | ❌ 不要 |

## 定数の命名規則

```rust
// モジュール/機能 + 用途 + 単位
const INNERTUBE_BUFFER_INTERVAL_MS: u32 = 1000;
const MAX_POLLING_INTERVAL_MS: u64 = 30000;
const MIN_DISPLAY_INTERVAL: u32 = 100;
```

## 関連ノウハウ

- **issues/013**: 防御的プログラミング（上下限ガード）
- **issues/019**: InnerTubeポーリング間隔

## 関連PR

- PR#100: InnerTubeコメント表示のリアルタイム体験改善（レビューで指摘）

## 教訓

1. **2箇所以上で使う値は定数化**: コピペミスや変更漏れを防ぐ
2. **コメントで根拠を明記**: なぜこの値を選んだか後からわかるように
3. **適切なモジュールに配置**: 関連する型や定数と同じ場所に置く
4. **publicにしてエクスポート**: 複数ファイルから参照できるように
