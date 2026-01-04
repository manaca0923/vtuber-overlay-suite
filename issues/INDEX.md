# Issues ノウハウインデックス

このファイルは `/issues` 配下のノウハウを効率的に参照するためのインデックスです。

## 必須ノウハウ（実装前に確認）

実装を始める前に、該当カテゴリのファイルを確認してください。

| カテゴリ | 参照ファイル | 要点 |
|---------|-------------|------|
| **Tauri invoke** | [007](007_tauri-invoke-snake-case.md) | パラメータ名は `snake_case` |
| **serde** | [021](021_serde-field-naming.md) | `rename_all` の適用範囲、戻り値の型定義 |
| **入力検証** | [013](013_pr68-accessibility-defensive-coding.md) | 型ガード、上下限チェック |
| **セキュリティ** | [002](002_overlay-security.md) | URL検証、XSS、深層防御 |
| **定数** | [020](020_magic-number-constants.md) | マジックナンバー禁止 |
| **WebSocket** | [010](010_pr62-websocket-manager-bfcache.md) | bfcache対応 |
| **アニメーション** | [022](022_animation-callback-patterns.md) | 二重実行防止 |

---

## 実装タイプ別チェックリスト

### Tauriコマンド追加
- [007](007_tauri-invoke-snake-case.md): `rename_all = "snake_case"`
- [021](021_serde-field-naming.md): variant内フィールド注意
- [013](013_pr68-accessibility-defensive-coding.md): 入力検証

### WebSocket/postMessage
- [010](010_pr62-websocket-manager-bfcache.md): bfcache、タイマークリア
- [002](002_overlay-security.md): origin検証
- [021](021_serde-field-naming.md): フィールド命名

### 設定項目追加
- [016](016_pr93-widget-visibility-review.md): オプショナル設計
- [025](025_pr106-theme-font-settings.md): マイグレーション
- [020](020_magic-number-constants.md): 定数化

### UI/オーバーレイ
- [001](001_component-system-review.md): コンポーネント管理
- [022](022_animation-callback-patterns.md): アニメーション
- [013](013_pr68-accessibility-defensive-coding.md): a11y

---

## カテゴリ別詳細

<details>
<summary><strong>Rust/Tauri</strong></summary>

| ID | 要点 |
|----|------|
| [003](003_tauri-rust-patterns.md) | Tauri 2.0 基本パターン |
| [005](005_pr56-sqlite-retry-review.md) | SQLite `busy_timeout` |
| [007](007_tauri-invoke-snake-case.md) | invoke snake_case |
| [019](019_innertube-polling-interval.md) | ポーリング間隔 |
| [021](021_serde-field-naming.md) | serde命名規則 |
| [028](028_pr109-log-level-trace.md) | 定期実行ログは`trace` |

</details>

<details>
<summary><strong>セキュリティ・入力検証</strong></summary>

| ID | 要点 |
|----|------|
| [002](002_overlay-security.md) | URL検証、XSS、postMessage |
| [013](013_pr68-accessibility-defensive-coding.md) | 型ガード、フォールバック |
| [025](025_pr106-theme-font-settings.md) | 深層防御 |
| [031](031_pr116-promo-validation-review.md) | 公開APIのバリデーション、境界チェック |

</details>

<details>
<summary><strong>JavaScript/TypeScript/React</strong></summary>

| ID | 要点 |
|----|------|
| [001](001_component-system-review.md) | コンポーネント管理 |
| [010](010_pr62-websocket-manager-bfcache.md) | bfcache対応 |
| [018](018_eslint-overlay-config.md) | ESLint設定 |
| [020](020_magic-number-constants.md) | 定数化パターン |
| [022](022_animation-callback-patterns.md) | アニメーション |
| [027](027_pr108-multi-city-weather-review.md) | setInterval/visibilitychange |
| [029](029_react-compiler-dependency-inference.md) | React Compiler依存配列推論 |
| [030](030_pr115-queue-management-review.md) | 非同期競合対策、旧データ互換性 |

</details>

<details>
<summary><strong>UI/オーバーレイ</strong></summary>

| ID | 要点 |
|----|------|
| [009](009_pr60-useBundledKey-persistence.md) | 設定永続化 |
| [016](016_pr93-widget-visibility-review.md) | ウィジェット表示 |
| [023](023_pr104-superchat-widget-review.md) | スパチャウィジェット |
| [024](024_pr105-superchat-settings-review.md) | スパチャ設定 |
| [025](025_pr106-theme-font-settings.md) | テーマ・フォント |

</details>

<details>
<summary><strong>テスト・コードスタイル</strong></summary>

| ID | 要点 |
|----|------|
| [012](012_pr66-test-patterns.md) | JSDOM、Object.defineProperty |
| [014](014_pr84-http-mock-patterns.md) | mockitoパターン |
| [015](015_pr88-code-style.md) | 連続空行 |
| [017](017_dead-code-warnings.md) | dead_code警告 |
| [026](026_pr107-from-trait-duplication.md) | Fromトレイトで重複削減 |
| [027](027_pr108-multi-city-weather-review.md) | マルチシティ天気、setInterval/visibility |

</details>

<details>
<summary><strong>PRレビュー記録（履歴）</strong></summary>

詳細なレビュー履歴。通常参照不要。

| ID | 関連PR |
|----|--------|
| [004](004_pr55-codex-review.md) | PR#55 |
| [006](006_pr57-weather-api-review.md) | PR#57 |
| [008](008_pr59-codex-review.md) | PR#59 |
| [011](011_pr64-technical-improvements.md) | PR#64 |

</details>

---

## 新機能実装時のチェックリスト

### Tauriコマンド追加時
- [ ] [007](007_tauri-invoke-snake-case.md): パラメータ名は `snake_case`
- [ ] [021](021_serde-field-naming.md): serde設定を確認
- [ ] [013](013_pr68-accessibility-defensive-coding.md): 入力値の検証
- [ ] [003](003_tauri-rust-patterns.md)#8: RwLockガードをawait境界をまたいで保持しない

### WebSocket/postMessage追加時
- [ ] [010](010_pr62-websocket-manager-bfcache.md): bfcache対応
- [ ] [002](002_overlay-security.md): origin検証、ペイロード検証
- [ ] [021](021_serde-field-naming.md): フィールド命名規則

### 設定項目追加時
- [ ] [016](016_pr93-widget-visibility-review.md): オプショナルフィールド設計
- [ ] [025](025_pr106-theme-font-settings.md): マイグレーション処理
- [ ] [020](020_magic-number-constants.md): デフォルト値の定数化

### UI/オーバーレイ追加時
- [ ] [001](001_component-system-review.md): コンポーネント管理
- [ ] [022](022_animation-callback-patterns.md): アニメーション二重実行防止
- [ ] [013](013_pr68-accessibility-defensive-coding.md): アクセシビリティ

---

## 更新履歴

- 2024-12: 初版作成（issues 001-025）
