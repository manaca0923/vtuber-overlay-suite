# 技術アーキテクチャ

## 技術スタック

| レイヤー | 技術 | 備考 |
|----------|------|------|
| Desktop | Tauri 2.0 (Rust) | クロスプラットフォーム |
| Frontend | React + TypeScript | Vite推奨 |
| Styling | Tailwind CSS | 高速開発 |
| Overlay | HTML/CSS + Canvas/WebGL | OBSブラウザソース用 |
| Database | SQLite | ローカル永続化 |
| Realtime | WebSocket | localhost通信 |
| HTTP Server | Axum | Rust製 |
| YouTube API | gRPC Streaming (メイン) | InnerTube=バックアップ, REST=互換 |
| Updater | Tauri Updater | 起動時チェック |

---

## システム構成図

```
┌─────────────────────────────────────────────────────────────┐
│                    Tauri 2.0 Application                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────┐    ┌─────────────────────────────┐ │
│  │    React Frontend   │    │       Rust Backend          │ │
│  ├─────────────────────┤    ├─────────────────────────────┤ │
│  │ - 設定画面          │◄──►│ - YouTube API通信 (gRPC/InnerTube) │ │
│  │ - プレビュー        │    │ - ポーリング制御            │ │
│  │ - セットリスト編集  │    │ - WebSocket Server          │ │
│  │ - ウィザード        │    │ - HTTP Server               │ │
│  └─────────────────────┘    │ - SQLite管理                │ │
│                              │ - セキュアストレージ        │ │
│                              └─────────────────────────────┘ │
│                                          │                   │
│                              ┌───────────┴───────────┐       │
│                              │       SQLite DB       │       │
│                              │ - 設定                │       │
│                              │ - 楽曲データ          │       │
│                              │ - コメントログ        │       │
│                              └───────────────────────┘       │
└─────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────┐
            │  YouTube  │   │    OBS    │   │  Browser  │
            │ API(gRPC) │   │  Studio   │   │ (Preview) │
            └───────────┘   └───────────┘   └───────────┘
```

---

## ディレクトリ構成（推奨）

```
vtuber-overlay-suite/
├── docs/                      # ドキュメント
├── src-tauri/                 # Rust Backend
│   ├── src/
│   │   ├── main.rs           # エントリーポイント
│   │   ├── lib.rs            # ライブラリルート
│   │   ├── commands/         # Tauriコマンド
│   │   │   ├── mod.rs
│   │   │   ├── youtube.rs    # YouTube API関連
│   │   │   ├── setlist.rs    # セットリスト操作
│   │   │   ├── overlay.rs    # オーバーレイ設定
│   │   │   └── settings.rs   # 設定管理
│   │   ├── youtube/          # YouTube API実装
│   │   │   ├── mod.rs
│   │   │   ├── client.rs     # APIクライアント
│   │   │   ├── types.rs      # 型定義
│   │   │   └── poller.rs     # ポーリング制御
│   │   ├── server/           # WebSocket/HTTP
│   │   │   ├── mod.rs
│   │   │   ├── websocket.rs
│   │   │   ├── http.rs
│   │   │   └── types.rs      # メッセージ型定義
│   │   ├── db/               # SQLite
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── queries.rs
│   │   └── secure/           # セキュアストレージ
│   │       └── mod.rs
│   ├── overlays/             # オーバーレイHTML（実際の配置）
│   │   ├── comment.html
│   │   ├── setlist.html
│   │   ├── combined.html     # 統合オーバーレイ
│   │   └── shared/           # 共通リソース
│   │       ├── overlay-common.css
│   │       └── comment-renderer.js
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # React Frontend
│   ├── components/
│   │   ├── settings/         # 設定画面
│   │   ├── setlist/          # セットリスト
│   │   ├── preview/          # プレビュー
│   │   └── wizard/           # 初回設定
│   ├── hooks/
│   ├── stores/               # 状態管理
│   ├── types/
│   │   ├── overlaySettings.ts
│   │   └── template.ts       # 3カラムテンプレート型（将来）
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## オーバーレイレイアウト層（将来実装予定）

> **ステータス**: 設計完了、実装予定

### 概要

3カラム固定レイアウト（22%/56%/22%）をベースとした、slot配置システム。

```
┌─────────────────────────────────────────────────────────────┐
│                    OBS Browser Source (1920x1080)           │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌────────────────────────┐  ┌──────────┐    │
│  │ left.top │  │                        │  │right.top │    │
│  ├──────────┤  │                        │  ├──────────┤    │
│  │left.     │  │                        │  │right.    │    │
│  │topBelow  │  │      center.full       │  │upper     │    │
│  ├──────────┤  │      (主役ステージ)    │  │(セトリ)  │    │
│  │left.     │  │                        │  ├──────────┤    │
│  │middle    │  │                        │  │right.    │    │
│  │(コメント)│  │                        │  │lowerL/R  │    │
│  ├──────────┤  └────────────────────────┘  ├──────────┤    │
│  │left.lower│                              │right.    │    │
│  ├──────────┤                              │bottom    │    │
│  │left.     │                              │(告知)    │    │
│  │bottom    │                              └──────────┘    │
│  └──────────┘                                              │
│      22%              56%                     22%          │
└─────────────────────────────────────────────────────────────┘
```

### テンプレート管理

テンプレート設定はJSONで定義し、SQLiteのsettingsテーブルに保存。

```
テンプレートJSON
    ↓
JSON Schema検証（src-tauri/schemas/）
    ↓
クランプ処理（Rust）
    ↓
SQLite保存
    ↓
WebSocket配信 → OBSオーバーレイ
```

詳細は `docs/300_overlay-specs.md`（3カラムレイアウト）および `docs/400_data-models.md`（型定義）を参照。

---

## 通信フロー

### 1. コメント取得フロー

```
[YouTube InnerTube] ──(HTTP)──► [Rust Poller] ──(Channel)──► [WebSocket Server]
                               │                              │
                               ▼                              ▼
                          [SQLite]                    [OBS Browser Source]
                        (ログ保存)                      (リアルタイム表示)
```

### 2. セットリスト更新フロー

```
[React UI] ──(Tauri Command)──► [Rust Backend] ──► [SQLite]
                                      │
                                      ▼
                              [WebSocket Server]
                                      │
                                      ▼
                              [OBS Browser Source]
```

---

## ポート設計

| 用途 | ポート | 備考 |
|------|--------|------|
| HTTP Server | 19800 | オーバーレイ配信用 |
| WebSocket | 19801 | リアルタイム更新用 |

**エンドポイント設計**:
```
GET  http://localhost:19800/overlay/comment    # コメントオーバーレイ
GET  http://localhost:19800/overlay/setlist    # セットリストオーバーレイ
GET  http://localhost:19800/api/health         # ヘルスチェック
WS   ws://localhost:19801/ws                   # WebSocket接続
```

---

## セキュリティ要件

### APIキー保護
- **保存**: OSセキュアストレージ使用
  - Windows: DPAPI（Credential Manager）
  - macOS: Keychain
- **メモリ**: 使用後はゼロクリア
- **ログ**: マスキング必須（`AIza***...***`）

### Tauri権限設定（capabilities）

```json
{
  "permissions": [
    "core:default",
    "shell:allow-open",
    "http:default",
    "websocket:default",
    "fs:allow-app-read",
    "fs:allow-app-write"
  ]
}
```

---

## 依存クレート（Rust）

```toml
[dependencies]
tauri = { version = "2", features = ["updater"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }
axum = "0.7"
tokio-tungstenite = "0.24"
keyring = "3"              # セキュアストレージ
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## 依存パッケージ（Frontend）

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.9.1",
    "@tauri-apps/plugin-updater": "^2.9.0",
    "react": "^19.2.0",
    "react-dom": "^19.2.0",
    "@dnd-kit/core": "^6.3.1",      // ドラッグ&ドロップ
    "@dnd-kit/sortable": "^10.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.9.6",
    "typescript": "~5.9.3",
    "vite": "^7.2.4",
    "@vitejs/plugin-react": "^5.1.1",
    "tailwindcss": "^4.1.18"
  }
}
```
