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
| HTTP Server | Actix-web / Axum | Rust製 |
| YouTube API | InnerTube API (メイン) | 公式REST APIはデバッグ用 |
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
│  │ - 設定画面          │◄──►│ - InnerTube API通信         │ │
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
            │ InnerTube │   │  Studio   │   │ (Preview) │
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
│   │   │   └── settings.rs   # 設定管理
│   │   ├── youtube/          # YouTube API実装
│   │   │   ├── mod.rs
│   │   │   ├── client.rs     # APIクライアント
│   │   │   ├── types.rs      # 型定義
│   │   │   └── poller.rs     # ポーリング制御
│   │   ├── server/           # WebSocket/HTTP
│   │   │   ├── mod.rs
│   │   │   ├── websocket.rs
│   │   │   └── http.rs
│   │   ├── db/               # SQLite
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── queries.rs
│   │   └── secure/           # セキュアストレージ
│   │       └── mod.rs
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
│   ├── App.tsx
│   └── main.tsx
├── overlay/                   # OBS用オーバーレイ
│   ├── comment/              # コメント表示
│   │   ├── index.html
│   │   ├── style.css
│   │   └── script.js
│   ├── setlist/              # セットリスト表示
│   │   ├── index.html
│   │   ├── style.css
│   │   └── script.js
│   └── templates/            # テンプレートアセット
│       └── default/
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## 通信フロー

### 1. コメント取得フロー

```
[YouTube API] ──(REST)──► [Rust Poller] ──(Channel)──► [WebSocket Server]
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
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
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
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-updater": "^2",
    "react": "^18",
    "react-dom": "^18",
    "@dnd-kit/core": "^6",      // ドラッグ&ドロップ
    "@dnd-kit/sortable": "^8",
    "zustand": "^5"             // 状態管理
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "typescript": "^5",
    "vite": "^6",
    "@vitejs/plugin-react": "^4",
    "tailwindcss": "^3"
  }
}
```
