# 二重API呼び出しの回避

## 概要

フロントエンドで事前取得（成功/失敗カウント用）→配信実行（内部で再取得）のパターンは二重取得となり、待機時間が倍増する。

## 問題

```typescript
// NG: 二重取得
const results = await getWeatherMulti(cityTuples);  // 1回目
const successCount = results.length;
await broadcastWeatherMulti(cityTuples, interval);  // 2回目（内部で再取得）
```

- 同じAPIを2回呼び出すためレイテンシが倍増
- キャッシュがあっても不要な処理が発生
- 都市数が多いと待機時間が大幅に伸びる

## 解決策

配信コマンドの戻り値に成功/失敗カウントを含める。

### Rust側

```rust
/// マルチシティ配信結果
#[derive(Debug, Clone, Serialize)]
pub struct BroadcastMultiResult {
    pub success_count: usize,
    pub total_count: usize,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn broadcast_weather_multi(
    state: State<'_, AppState>,
    cities: Vec<(String, String, String)>,
    rotation_interval_sec: u32,
) -> Result<BroadcastMultiResult, String> {
    let total_count = cities.len();
    let weather_data = get_weather_multi(state.clone(), cities).await?;
    let success_count = weather_data.len();

    // ... ブロードキャスト処理 ...

    Ok(BroadcastMultiResult { success_count, total_count })
}
```

### TypeScript側

```typescript
// OK: 1回の呼び出しで完結
const result = await broadcastWeatherMulti(cityTuples, interval);
const { success_count: successCount, total_count: totalCount } = result;
```

## チェックリスト

- [ ] 「取得→配信」のパターンで二重取得になっていないか
- [ ] 配信コマンドが必要な情報を戻り値で返しているか
- [ ] キャッシュがあっても不要な呼び出しを避けているか

## 関連PR

- PR#119: `broadcast_weather_multi`の戻り値改善
