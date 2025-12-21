# VTuber Overlay Suite

VTuber配信支援ツール - YouTube Liveのコメント表示とセットリスト管理をOBSで簡単に実現するデスクトップアプリケーション

## 特徴

- **コメントオーバーレイ**: YouTube Liveのコメントをリアルタイムで表示
- **セットリスト管理**: 歌配信の曲順管理と表示
- **OBS連携**: ブラウザソースでオーバーレイを簡単に追加
- **認証不要**: APIキーなしで動作（InnerTube API使用）
- **高品質デザイン**: プロレベルのオーバーレイテンプレート

## システム要件

| 項目 | 要件 |
|------|------|
| OS | Windows 10以降 / macOS 12以降 |
| OBS Studio | 28.0以降（推奨） |
| 配信プラットフォーム | YouTube Live |

## インストール

### ダウンロード

[Releases](https://github.com/manaca0923/vtuber-overlay-suite/releases) ページから最新版をダウンロードしてください。

| OS | ファイル |
|----|----------|
| Windows | `vtuber-overlay-suite_x.x.x_x64-setup.exe` |
| macOS | `vtuber-overlay-suite_x.x.x_aarch64.dmg` |

### インストール手順

**Windows**:
1. ダウンロードした `.exe` ファイルを実行
2. インストーラーの指示に従ってインストール

**macOS**:
1. ダウンロードした `.dmg` ファイルを開く
2. アプリを「アプリケーション」フォルダにドラッグ

## 使い方

### クイックスタート

1. **アプリを起動**
2. **YouTube配信のURLまたはVideo IDを入力**
3. **「コメント取得開始」をクリック**
4. **OBSにオーバーレイURLを追加**

### OBS設定

#### コメントオーバーレイ

1. OBSで「ソース」→「+」→「ブラウザ」を選択
2. 以下のURLを入力:
   ```
   http://localhost:19800/overlay/comment
   ```
3. 幅: `400`、高さ: `600` を設定
4. 「カスタムCSS」に以下を入力:
   ```css
   body { background: transparent !important; }
   ```

#### セットリストオーバーレイ

1. OBSで「ソース」→「+」→「ブラウザ」を選択
2. 以下のURLを入力:
   ```
   http://localhost:19800/overlay/setlist
   ```
3. 幅: `350`、高さ: `200` を設定

#### 統合オーバーレイ（推奨）

コメントとセットリストを1つのソースで表示:

```
http://localhost:19800/overlay/combined
```
- 幅: `1920`、高さ: `1080`

### URLパラメータ

オーバーレイはURLパラメータでカスタマイズできます。

```
http://localhost:19800/overlay/comment?fontSize=18&showAvatar=false
```

| パラメータ | 型 | 説明 |
|------------|-----|------|
| `primaryColor` | string | プライマリカラー（例: `#6366f1`） |
| `fontSize` | number | フォントサイズ（8-72） |
| `showAvatar` | boolean | アバター表示（`true`/`false`） |
| `position` | string | 表示位置（`bottom-left`, `top-right`など） |

統合オーバーレイ専用:

| パラメータ | 型 | 説明 |
|------------|-----|------|
| `layout` | string | レイアウト（`streaming`, `talk`, `music`, `gaming`） |
| `commentEnabled` | boolean | コメント表示ON/OFF |
| `setlistEnabled` | boolean | セットリスト表示ON/OFF |

## トラブルシューティング

### コメントが表示されない

1. YouTube配信が「公開」または「限定公開」であることを確認
2. ライブチャットが有効になっていることを確認
3. Video IDが正しいことを確認

### OBSでオーバーレイが表示されない

1. アプリが起動していることを確認
2. URLが `http://localhost:19800/...` であることを確認
3. ブラウザソースの「ローカルファイル」がOFFであることを確認

## 開発

### 必要条件

- Node.js 20+
- Rust 1.86+
- Tauri CLI

### セットアップ

```bash
# 依存関係のインストール
npm install

# 開発サーバー起動
npm run tauri:dev

# ビルド
npm run tauri:build
```

### スクリプト

| コマンド | 説明 |
|---------|------|
| `npm run dev` | Vite開発サーバー起動 |
| `npm run build` | フロントエンドビルド |
| `npm run typecheck` | TypeScript型チェック |
| `npm run lint` | ESLintチェック |
| `npm run tauri:dev` | Tauri開発モード |
| `npm run tauri:build` | Tauriリリースビルド |

### 技術スタック

| レイヤー | 技術 |
|----------|------|
| Desktop | Tauri 2.0 (Rust) |
| Frontend | React + TypeScript + Vite |
| Styling | Tailwind CSS |
| Database | SQLite |
| Realtime | WebSocket |

### ドキュメント

詳細なドキュメントは `docs/` フォルダを参照してください。

| ファイル | 内容 |
|----------|------|
| `001_requirements.md` | 要件サマリー |
| `100_architecture.md` | 技術アーキテクチャ |
| `200_youtube-api.md` | YouTube API仕様 |
| `300_overlay-specs.md` | オーバーレイ仕様 |

## ライセンス

MIT License

## 注意事項

- InnerTube APIは非公式APIのため、YouTube側の仕様変更で動作しなくなる可能性があります
- 配信中に問題が発生した場合は、アプリを再起動してください
