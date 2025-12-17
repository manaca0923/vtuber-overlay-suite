# VTuber Overlay Suite

VTuber配信支援ツール - コメントオーバーレイとセットリスト管理を提供するデスクトップアプリケーション

## 技術スタック

- **Desktop**: Tauri 2.0 (Rust)
- **Frontend**: React + TypeScript + Vite
- **Styling**: Tailwind CSS

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

## ドキュメント

詳細なドキュメントは `docs/` フォルダを参照してください。
