-- 設定テーブル
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 楽曲テーブル
CREATE TABLE IF NOT EXISTS songs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    artist TEXT,
    category TEXT,
    tags TEXT,  -- JSON array
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- セットリストテーブル
CREATE TABLE IF NOT EXISTS setlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- セットリスト楽曲（中間テーブル）
CREATE TABLE IF NOT EXISTS setlist_songs (
    id TEXT PRIMARY KEY,
    setlist_id TEXT NOT NULL REFERENCES setlists(id) ON DELETE CASCADE,
    song_id TEXT NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    started_at TEXT,  -- 演奏開始時刻
    ended_at TEXT,    -- 演奏終了時刻
    UNIQUE(setlist_id, position)
);

-- コメントログテーブル
CREATE TABLE IF NOT EXISTS comment_logs (
    id TEXT PRIMARY KEY,
    youtube_id TEXT NOT NULL,
    message TEXT NOT NULL,
    author_name TEXT NOT NULL,
    author_channel_id TEXT NOT NULL,
    author_image_url TEXT,
    is_owner INTEGER NOT NULL DEFAULT 0,
    is_moderator INTEGER NOT NULL DEFAULT 0,
    is_member INTEGER NOT NULL DEFAULT 0,
    message_type TEXT NOT NULL DEFAULT 'text',
    message_data TEXT,  -- JSON (スパチャ金額等)
    published_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_setlist_songs_setlist ON setlist_songs(setlist_id);
CREATE INDEX IF NOT EXISTS idx_setlist_songs_position ON setlist_songs(setlist_id, position);
CREATE INDEX IF NOT EXISTS idx_comment_logs_published ON comment_logs(published_at);
CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
