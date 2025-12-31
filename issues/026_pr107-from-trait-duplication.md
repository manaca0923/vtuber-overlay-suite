# From トレイトによる型変換コードの重複削減

## 概要

データ型間の変換コードが複数箇所で重複している場合、`From` トレイトを実装することで一元化できる。

## 問題パターン

複数のコマンドで同じ型への変換を手動で行っている：

```rust
// コマンドA
let payload = SomePayload {
    field1: data.field1.clone(),
    field2: data.field2,
    field3: Some(data.field3),
};

// コマンドB (同じ変換コード)
let payload = SomePayload {
    field1: data.field1.clone(),
    field2: data.field2,
    field3: Some(data.field3),
};
```

## 解決策

`From` トレイトを実装して変換ロジックを一元化：

```rust
impl From<&SourceData> for TargetPayload {
    fn from(data: &SourceData) -> Self {
        Self {
            field1: data.field1.clone(),
            field2: data.field2,
            field3: Some(data.field3),
        }
    }
}

// 使用側
let payload = TargetPayload::from(&data);
// または
let payload: TargetPayload = (&data).into();
```

## メリット

1. **DRY原則**: 変換ロジックが1箇所に集約
2. **保守性**: フィールド追加時に修正箇所が1つ
3. **Rustらしい**: 標準トレイトを活用したイディオマティックなコード
4. **型安全**: コンパイル時に変換の整合性を保証

## 適用例（PR#107）

`WeatherData` から `WeatherUpdatePayload` への変換を `From` トレイトで実装：

- 対象ファイル: `src-tauri/src/server/types.rs`
- 変更前: 4箇所で同じ変換コードが重複
- 変更後: `From<&WeatherData> for WeatherUpdatePayload` を1箇所で定義

## チェックリスト

新しいペイロード型やデータ型を追加する際：

- [ ] 他の型からの変換が必要か確認
- [ ] 変換ロジックが2箇所以上で使われる場合は `From` トレイト実装を検討
- [ ] 所有権を消費する場合は `From<T>`、参照の場合は `From<&T>` を選択

## 関連PR

- PR#107: 天気ウィジェット自動更新機能
