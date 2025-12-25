-- youtube_idにUNIQUE制約を追加
-- INSERT OR IGNOREで重複コメントをスキップするために必要

-- 既存データの重複を削除（最初の1件を残す）
DELETE FROM comment_logs
WHERE rowid NOT IN (
    SELECT MIN(rowid)
    FROM comment_logs
    GROUP BY youtube_id
);

-- UNIQUE制約をインデックスとして追加
CREATE UNIQUE INDEX IF NOT EXISTS idx_comment_logs_youtube_id ON comment_logs(youtube_id);
