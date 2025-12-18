# 110_development-environment.md

開発環境のセットアップ手順と要件

## 現在の開発環境（macOS）

### ハードウェア・OS

- **OS**: macOS 15.6.1 (Sequoia)
- **CPU**: Apple M3
- **アーキテクチャ**: ARM64 (Apple Silicon)

### 開発ツール

| ツール | バージョン | 備考 |
|--------|-----------|------|
| **Homebrew** | 5.0.6 | パッケージマネージャ |
| **Git** | 2.39.3 | バージョン管理 |
| **Node.js** | 20.19.2 | JavaScript/TypeScriptランタイム |
| **npm** | 10.8.2 | Node.jsパッケージマネージャ |
| **Rust** | 1.86.0 | Rustコンパイラ（Homebrew版） |
| **Cargo** | 1.86.0 | Rustパッケージマネージャ |

### フロントエンド

| ツール | バージョン |
|--------|-----------|
| **TypeScript** | 5.9.3 |
| **Vite** | 7.2.4 |
| **React** | 19.2.0 |
| **Tailwind CSS** | 4.1.18 |

### Tauri

| パッケージ | バージョン |
|-----------|-----------|
| **@tauri-apps/cli** | 2.9.6 |
| **@tauri-apps/api** | 2.9.1 |
| **tauri (Rust)** | 2.9.5 |
| **tauri-build** | 2.5.3 |

### Rust依存関係

主要なクレート：

| クレート | バージョン | 用途 |
|---------|-----------|------|
| **tokio** | 1.x | 非同期ランタイム |
| **reqwest** | 0.12 | HTTP クライアント |
| **serde** | 1.0 | シリアライゼーション |
| **chrono** | 0.4 | 日時処理 |
| **thiserror** | 2.0 | エラー型定義 |
| **log** | 0.4 | ロギング |
| **sqlx** | 0.7 | SQLiteデータベースクライアント |
| **uuid** | 1.0 | UUID生成 |
| **dirs** | 5.0 | クロスプラットフォームディレクトリ取得 |

### SQLx（データベース）

プロジェクトではSQLxのオフラインモードを使用しています：

- **`.sqlx/`ディレクトリ**: コンパイル時のクエリ検証用メタデータ（Gitにコミット済み）
- **`.cargo/config.toml`**: `SQLX_OFFLINE=true` を設定済み
- **DATABASE_URL**: ビルド時に不要（オフラインモード使用）

#### 開発時の注意事項

**通常のビルド**（推奨）:
```bash
cargo build
```
`.sqlx/`メタデータを使用してコンパイルされるため、データベース接続は不要です。

**クエリ変更時の手順**:
SQLクエリを変更した場合は、以下の手順でメタデータを更新してください：

```bash
# 1. 開発用データベースURLを設定（一時的）
echo "DATABASE_URL=sqlite:./dev.db" > .env

# 2. sqlx-cliのインストール（初回のみ）
cargo install sqlx-cli --no-default-features --features sqlite

# 3. データベースとマイグレーションの実行
sqlx database create
sqlx migrate run

# 4. メタデータの再生成（SQLX_OFFLINEを無効化）
SQLX_OFFLINE=false cargo sqlx prepare

# 5. .envファイルを削除（.gitignoreに含まれているため不要）
rm .env
```

**重要**:
- `.sqlx/`ディレクトリの変更は必ずGitにコミットしてください
- `cargo sqlx prepare`実行時は**必ず`SQLX_OFFLINE=false`を指定**してください（`.cargo/config.toml`の設定を上書き）

## セットアップ手順（macOS）

### 1. 前提条件のインストール

```bash
# Homebrewのインストール（未インストールの場合）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Node.jsとRustのインストール
brew install node
brew install rust

# バージョン確認
node --version  # v20.19.2 推奨
npm --version   # 10.8.2 推奨
rustc --version # 1.86.0 推奨
```

### 2. プロジェクトのクローン

```bash
git clone https://github.com/manaca0923/vtuber-overlay-suite.git
cd vtuber-overlay-suite
```

### 3. 依存関係のインストール

```bash
# フロントエンド依存関係
npm install

# Rust依存関係（自動）
cd src-tauri
cargo build
cd ..
```

### 4. 開発サーバーの起動

```bash
# Tauriアプリの起動（推奨）
npm run tauri:dev

# または、フロントエンドのみ
npm run dev
```

### 5. ビルド

```bash
# 型チェック
npm run typecheck

# リント
npm run lint

# 本番ビルド
npm run tauri:build
```

## Windows環境対応

### 現在の方針

- **開発**: macOS環境で実施
- **ビルド検証**: GitHub ActionsのWindowsランナーで自動実行
- **本格対応**: 必要になったタイミングでWindows環境をセットアップ

### GitHub Actionsによる自動ビルド

`.github/workflows/build-windows.yml`で以下を実施：

1. Windowsランナーでのビルド実行
2. 成果物（.exe, .msi）の生成
3. Artifactsとして保存

これにより、macOS開発環境でも**Windowsでのビルド可能性を常に担保**

### 将来のWindows環境要件

Windowsで開発する場合、以下が必要：

- **OS**: Windows 10/11
- **ツール**:
  - Visual Studio Build Tools（C++コンパイラ）
  - WebView2（Tauri要件）
  - Node.js 20.x
  - Rust（rustup経由）
  - Git for Windows

詳細なセットアップ手順は、Windows環境での開発開始時に追記予定。

## トラブルシューティング

### Apple Silicon（M1/M2/M3）特有の問題

Rust依存関係のネイティブビルドで問題が発生する場合：

```bash
# Rosetta 2のインストール（Intel互換）
softwareupdate --install-rosetta
```

### Tauri Dev起動時のエラー

```bash
# キャッシュクリア
rm -rf node_modules
rm -rf src-tauri/target
npm install
cd src-tauri && cargo clean && cd ..
```

### TypeScript型エラー

```bash
# 型定義の再生成
npm run typecheck
```

## 参考リンク

- [Tauri 2.0 ドキュメント](https://v2.tauri.app/)
- [Rust公式サイト](https://www.rust-lang.org/)
- [Vite公式サイト](https://vite.dev/)
- [React公式サイト](https://react.dev/)
