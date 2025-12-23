# リリース手順

## 概要

本アプリは Tauri 2.0 の自動更新機能を使用しています。リリースは GitHub Actions で自動化されており、GitHub Releases を更新サーバーとして使用します。

## 初回セットアップ（署名キー生成）

### 1. 署名キーを生成

```bash
# ローカルで署名キーを生成
npx tauri signer generate -w ~/.tauri/vtuber-overlay-suite.key
```

パスワードを入力すると、以下のファイルが生成されます：
- `~/.tauri/vtuber-overlay-suite.key` - 秘密鍵
- `~/.tauri/vtuber-overlay-suite.key.pub` - 公開鍵

### 2. 公開鍵を設定

生成された公開鍵（`.key.pub`）の内容を `src-tauri/tauri.conf.json` の `plugins.updater.pubkey` に設定します：

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk..."
    }
  }
}
```

### 3. GitHub Secrets に秘密鍵を登録

リポジトリの Settings → Secrets and variables → Actions で以下を追加：

| Secret名 | 値 |
|----------|-----|
| `TAURI_SIGNING_PRIVATE_KEY` | `~/.tauri/vtuber-overlay-suite.key` の内容 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | キー生成時に設定したパスワード |

## リリース手順

### 方法1: タグプッシュ（推奨）

```bash
# バージョンを更新
# src-tauri/tauri.conf.json の "version" を更新
# package.json の "version" を更新

# コミット
git add -A
git commit -m "chore: bump version to 0.2.0"

# タグ作成・プッシュ
git tag v0.2.0
git push origin main --tags
```

GitHub Actions が自動的にビルド・リリースを作成します。

### 方法2: 手動トリガー

GitHub Actions → Release → Run workflow から手動実行も可能です。

## リリース成果物

| ファイル | 説明 |
|----------|------|
| `VTuber Overlay Suite_x.x.x_aarch64.dmg` | macOS (Apple Silicon) |
| `VTuber Overlay Suite_x.x.x_x64.dmg` | macOS (Intel) |
| `VTuber Overlay Suite_x.x.x_x64-setup.exe` | Windows インストーラー |
| `VTuber Overlay Suite_x.x.x_x64_en-US.msi` | Windows MSI |
| `latest.json` | 自動更新用マニフェスト |

## 自動更新の仕組み

1. アプリ起動時に `latest.json` をチェック
2. 新しいバージョンがあれば通知
3. ユーザーが「更新する」をクリック
4. ダウンロード・インストール・再起動

### エンドポイント

```
https://github.com/manaca0923/vtuber-overlay-suite/releases/latest/download/latest.json
```

### latest.json の構造

```json
{
  "version": "0.2.0",
  "notes": "リリースノート",
  "pub_date": "2025-01-01T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "url": "https://github.com/.../VTuber Overlay Suite_0.2.0_aarch64.app.tar.gz",
      "signature": "..."
    },
    "darwin-x86_64": {
      "url": "https://github.com/.../VTuber Overlay Suite_0.2.0_x64.app.tar.gz",
      "signature": "..."
    },
    "windows-x86_64": {
      "url": "https://github.com/.../VTuber Overlay Suite_0.2.0_x64-setup.nsis.zip",
      "signature": "..."
    }
  }
}
```

## トラブルシューティング

### 更新チェックが失敗する

- `pubkey` が正しく設定されているか確認
- GitHub Releases が公開されているか確認
- `latest.json` が正しく生成されているか確認

### 署名エラー

- GitHub Secrets の `TAURI_SIGNING_PRIVATE_KEY` が正しいか確認
- パスワードが一致しているか確認

### ビルドエラー（Windows）

- Protobuf がインストールされているか確認（choco install protoc）

### ビルドエラー（macOS）

- Protobuf がインストールされているか確認（brew install protobuf）

## 参考リンク

- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [tauri-plugin-updater](https://github.com/tauri-apps/tauri-plugin-updater)
- [Tauri GitHub Action](https://github.com/tauri-apps/tauri-action)
