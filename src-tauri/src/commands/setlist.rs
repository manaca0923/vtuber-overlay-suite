use crate::db::models::{
    Setlist, SetlistSongWithDetails, SetlistWithSongs, Song, SongStatus,
};
use crate::AppState;
use chrono::Utc;
use std::cmp::Ordering;
use std::collections::HashSet;
use uuid::Uuid;

/// 楽曲一覧を取得
#[tauri::command]
pub async fn get_songs(state: tauri::State<'_, AppState>) -> Result<Vec<Song>, String> {
    let pool = &state.db;
    let songs = sqlx::query_as!(
        Song,
        r#"SELECT id as "id!", title as "title!", artist, category, tags, duration_seconds, created_at as "created_at!", updated_at as "updated_at!" FROM songs ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(songs)
}

/// 楽曲を作成
#[tauri::command]
pub async fn create_song(
    title: String,
    artist: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    duration_seconds: Option<i64>,
    state: tauri::State<'_, AppState>,
) -> Result<Song, String> {
    // 入力バリデーション
    if title.trim().is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if title.len() > 255 {
        return Err("Title is too long (max 255 characters)".to_string());
    }

    let pool = &state.db;
    let mut song = Song::new(title);
    song.artist = artist;
    song.category = category;
    song.duration_seconds = duration_seconds;

    // tagsをJSON文字列に変換
    let tags_json = match tags {
        Some(t) => Some(
            serde_json::to_string(&t)
                .map_err(|e| format!("Failed to serialize tags: {}", e))?
        ),
        None => None,
    };
    song.tags = tags_json.clone();

    sqlx::query!(
        "INSERT INTO songs (id, title, artist, category, tags, duration_seconds, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        song.id,
        song.title,
        song.artist,
        song.category,
        tags_json,
        song.duration_seconds,
        song.created_at,
        song.updated_at
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(song)
}

/// 楽曲を更新
#[tauri::command]
pub async fn update_song(
    id: String,
    title: Option<String>,
    artist: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    duration_seconds: Option<i64>,
    state: tauri::State<'_, AppState>,
) -> Result<Song, String> {
    // 入力バリデーション
    if let Some(ref t) = title {
        if t.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if t.len() > 255 {
            return Err("Title is too long (max 255 characters)".to_string());
        }
    }

    let pool = &state.db;
    let now = Utc::now().to_rfc3339();

    // tagsをJSON文字列に変換
    let tags_json = match tags {
        Some(t) => Some(
            serde_json::to_string(&t)
                .map_err(|e| format!("Failed to serialize tags: {}", e))?
        ),
        None => None,
    };

    sqlx::query!(
        "UPDATE songs
         SET title = COALESCE(?, title),
             artist = COALESCE(?, artist),
             category = COALESCE(?, category),
             tags = COALESCE(?, tags),
             duration_seconds = COALESCE(?, duration_seconds),
             updated_at = ?
         WHERE id = ?",
        title,
        artist,
        category,
        tags_json,
        duration_seconds,
        now,
        id
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 更新後の楽曲を取得
    let song = sqlx::query_as!(
        Song,
        r#"SELECT id as "id!", title as "title!", artist, category, tags, duration_seconds, created_at as "created_at!", updated_at as "updated_at!" FROM songs WHERE id = ?"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Song not found: {}", e))?;

    Ok(song)
}

/// 楽曲を削除
#[tauri::command]
pub async fn delete_song(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pool = &state.db;
    let result = sqlx::query!("DELETE FROM songs WHERE id = ?", id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("Song not found: {}", id));
    }

    Ok(())
}

/// セットリスト一覧を取得
#[tauri::command]
pub async fn get_setlists(state: tauri::State<'_, AppState>) -> Result<Vec<Setlist>, String> {
    let pool = &state.db;
    let setlists = sqlx::query_as!(
        Setlist,
        r#"SELECT id as "id!", name as "name!", description, created_at as "created_at!", updated_at as "updated_at!" FROM setlists ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(setlists)
}

/// セットリストを作成
#[tauri::command]
pub async fn create_setlist(
    name: String,
    description: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Setlist, String> {
    // 入力バリデーション
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.len() > 255 {
        return Err("Name is too long (max 255 characters)".to_string());
    }

    let pool = &state.db;
    let setlist = Setlist::new(name, description);

    sqlx::query!(
        "INSERT INTO setlists (id, name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
        setlist.id,
        setlist.name,
        setlist.description,
        setlist.created_at,
        setlist.updated_at
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(setlist)
}

/// セットリストを削除
#[tauri::command]
pub async fn delete_setlist(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pool = &state.db;
    let result = sqlx::query!("DELETE FROM setlists WHERE id = ?", id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("Setlist not found: {}", id));
    }

    Ok(())
}

/// セットリストに楽曲を追加
#[tauri::command]
pub async fn add_song_to_setlist(
    setlist_id: String,
    song_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;

    // セットリストの存在確認
    let setlist_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM setlists WHERE id = ?")
        .bind(&setlist_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    if setlist_count == 0 {
        return Err(format!("Setlist not found: {}", setlist_id));
    }

    // 楽曲の存在確認
    let song_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM songs WHERE id = ?")
        .bind(&song_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    if song_count == 0 {
        return Err(format!("Song not found: {}", song_id));
    }

    // 現在の最大positionを取得
    let max_position: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(position) FROM setlist_songs WHERE setlist_id = ?"
    )
    .bind(&setlist_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .flatten();

    let new_position = max_position.unwrap_or(-1) + 1;
    let id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO setlist_songs (id, setlist_id, song_id, position)
         VALUES (?, ?, ?, ?)",
        id,
        setlist_id,
        song_id,
        new_position
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// セットリストから楽曲を削除
#[tauri::command]
pub async fn remove_song_from_setlist(
    setlist_id: String,
    setlist_song_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;

    // トランザクション開始
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 削除する曲のpositionを取得
    let deleted_position: i64 = sqlx::query_scalar(
        "SELECT position FROM setlist_songs WHERE id = ? AND setlist_id = ?"
    )
    .bind(&setlist_song_id)
    .bind(&setlist_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("SetlistSong not found: {}", e))?;

    // 曲を削除
    sqlx::query!(
        "DELETE FROM setlist_songs WHERE id = ?",
        setlist_song_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 後続の曲のpositionを詰める
    sqlx::query!(
        "UPDATE setlist_songs
         SET position = position - 1
         WHERE setlist_id = ? AND position > ?",
        setlist_id,
        deleted_position
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // トランザクションをコミット
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// セットリスト（楽曲リスト付き）を取得
#[tauri::command]
pub async fn get_setlist_with_songs(
    id: String,
    state: tauri::State<'_, AppState>,
) -> Result<SetlistWithSongs, String> {
    let pool = &state.db;

    // セットリスト基本情報取得
    let setlist = sqlx::query_as!(
        Setlist,
        r#"SELECT id as "id!", name as "name!", description, created_at as "created_at!", updated_at as "updated_at!" FROM setlists WHERE id = ?"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Setlist not found: {}", e))?;

    // 楽曲リスト取得（JOIN）
    let rows = sqlx::query!(
        r#"
        SELECT
            ss.id as "ss_id!", ss.position as "position!", ss.started_at, ss.ended_at,
            s.id as "song_id!", s.title as "title!", s.artist, s.category, s.tags, s.duration_seconds,
            s.created_at as "song_created_at!", s.updated_at as "song_updated_at!"
        FROM setlist_songs ss
        JOIN songs s ON ss.song_id = s.id
        WHERE ss.setlist_id = ?
        ORDER BY ss.position
        "#,
        id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // 現在の曲のインデックスを計算（started_atがあり、ended_atがない曲）
    let current_index = rows
        .iter()
        .position(|row| row.started_at.is_some() && row.ended_at.is_none())
        .map(|i| i as i64)
        .unwrap_or(-1);

    // SetlistSongWithDetailsに変換
    let songs: Vec<SetlistSongWithDetails> = rows
        .into_iter()
        .map(|row| {
            let song = Song {
                id: row.song_id,
                title: row.title,
                artist: row.artist,
                category: row.category,
                tags: row.tags,
                duration_seconds: row.duration_seconds,
                created_at: row.song_created_at,
                updated_at: row.song_updated_at,
            };

            // ステータスを計算
            let status = if current_index == -1 {
                SongStatus::Pending
            } else {
                match row.position.cmp(&current_index) {
                    Ordering::Less => SongStatus::Done,
                    Ordering::Equal => SongStatus::Current,
                    Ordering::Greater => SongStatus::Pending,
                }
            };

            SetlistSongWithDetails {
                id: row.ss_id,
                position: row.position,
                song,
                started_at: row.started_at,
                ended_at: row.ended_at,
                status,
            }
        })
        .collect();

    Ok(SetlistWithSongs {
        setlist,
        songs,
        current_index,
    })
}

/// 指定位置の曲を現在の曲として設定
#[tauri::command]
pub async fn set_current_song(
    setlist_id: String,
    position: i64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;
    let now = Utc::now().to_rfc3339();

    // トランザクション開始
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 前の曲のended_atを記録
    sqlx::query!(
        "UPDATE setlist_songs
         SET ended_at = ?
         WHERE setlist_id = ? AND started_at IS NOT NULL AND ended_at IS NULL",
        now,
        setlist_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 新しい曲のstarted_atを記録
    sqlx::query!(
        "UPDATE setlist_songs
         SET started_at = ?
         WHERE setlist_id = ? AND position = ? AND started_at IS NULL",
        now,
        setlist_id,
        position
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // トランザクションコミット
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 次の曲へ進む
#[tauri::command]
pub async fn next_song(
    setlist_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;

    // 現在の曲の位置を取得
    let current_position: Option<i64> = sqlx::query_scalar(
        "SELECT position FROM setlist_songs
         WHERE setlist_id = ? AND started_at IS NOT NULL AND ended_at IS NULL"
    )
    .bind(&setlist_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let next_position = match current_position {
        Some(pos) => pos + 1,
        None => 0, // 現在の曲がない場合は最初の曲から開始
    };

    // 単一トランザクション内で全ての更新を実行
    let now = Utc::now().to_rfc3339();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 次の曲が存在するか確認
    let next_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM setlist_songs WHERE setlist_id = ? AND position = ?"
    )
    .bind(&setlist_id)
    .bind(next_position)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if next_exists == 0 {
        return Err("次の曲がありません".to_string());
    }

    // 1. 現在再生中の曲のended_atを記録
    sqlx::query!(
        "UPDATE setlist_songs
         SET ended_at = ?
         WHERE setlist_id = ? AND started_at IS NOT NULL AND ended_at IS NULL",
        now,
        setlist_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 2. 次の曲のstarted_atを記録
    sqlx::query!(
        "UPDATE setlist_songs
         SET started_at = ?
         WHERE setlist_id = ? AND position = ? AND started_at IS NULL",
        now,
        setlist_id,
        next_position
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // 全ての更新を単一トランザクションでコミット
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}

/// 前の曲へ戻る
#[tauri::command]
pub async fn previous_song(
    setlist_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;

    // 現在の曲の位置を取得
    let current_position: Option<i64> = sqlx::query_scalar(
        "SELECT position FROM setlist_songs
         WHERE setlist_id = ? AND started_at IS NOT NULL AND ended_at IS NULL"
    )
    .bind(&setlist_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    match current_position {
        Some(pos) if pos > 0 => {
            // 前の曲が存在する場合
            // 単一トランザクション内で全ての更新を実行
            let now = Utc::now().to_rfc3339();
            let prev_pos = pos - 1;
            let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

            // 1. 現在再生中の曲のended_atを記録
            sqlx::query!(
                "UPDATE setlist_songs
                 SET ended_at = ?
                 WHERE setlist_id = ? AND started_at IS NOT NULL AND ended_at IS NULL",
                now,
                setlist_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            // 2. 前の曲のタイムスタンプをクリア
            sqlx::query!(
                "UPDATE setlist_songs
                 SET started_at = NULL, ended_at = NULL
                 WHERE setlist_id = ? AND position = ?",
                setlist_id,
                prev_pos
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            // 3. 前の曲を現在の曲として設定（started_atを記録）
            sqlx::query!(
                "UPDATE setlist_songs
                 SET started_at = ?
                 WHERE setlist_id = ? AND position = ? AND started_at IS NULL",
                now,
                setlist_id,
                prev_pos
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            // 全ての更新を単一トランザクションでコミット
            tx.commit().await.map_err(|e| e.to_string())?;

            Ok(())
        }
        _ => Err("前の曲がありません".to_string()),
    }
}

/// セットリスト内の曲順を並び替え
#[tauri::command]
pub async fn reorder_setlist_songs(
    setlist_id: String,
    setlist_song_ids: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;

    // 入力バリデーション：空配列チェック
    if setlist_song_ids.is_empty() {
        return Err("曲IDリストが空です".to_string());
    }

    // 入力バリデーション：セットリストの実際の曲IDリストを取得
    let actual_ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM setlist_songs WHERE setlist_id = ? ORDER BY position"
    )
    .bind(&setlist_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    // セットリスト存在確認
    if actual_ids.is_empty() {
        // セットリストが存在しないか、曲がないか判別
        let setlist_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM setlists WHERE id = ?"
        )
        .bind(&setlist_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        if setlist_exists == 0 {
            return Err("セットリストが見つかりません".to_string());
        }
        return Err("セットリストに曲がありません".to_string());
    }

    // 曲数チェック
    if actual_ids.len() != setlist_song_ids.len() {
        return Err(format!(
            "曲数が一致しません（期待: {}, 実際: {}）",
            actual_ids.len(),
            setlist_song_ids.len()
        ));
    }

    // IDの所属確認：渡されたIDがすべてこのセットリストに属しているかチェック
    let passed_set: HashSet<_> = setlist_song_ids.iter().collect();
    let actual_set: HashSet<_> = actual_ids.iter().collect();

    if passed_set != actual_set {
        return Err("無効な曲IDが含まれています".to_string());
    }

    // トランザクション開始
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 2フェーズ更新でユニーク制約違反を回避
    // Phase 1: 一時的なオフセット値に移動（既存のpositionと重複しないように）
    let offset = 10000i64;
    for (index, setlist_song_id) in setlist_song_ids.iter().enumerate() {
        let temp_position = offset + index as i64;

        sqlx::query!(
            "UPDATE setlist_songs
             SET position = ?
             WHERE id = ? AND setlist_id = ?",
            temp_position, setlist_song_id, setlist_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Phase 2: 正しいpositionに設定
    for (index, setlist_song_id) in setlist_song_ids.iter().enumerate() {
        let new_position = index as i64;

        sqlx::query!(
            "UPDATE setlist_songs
             SET position = ?
             WHERE id = ? AND setlist_id = ?",
            new_position, setlist_song_id, setlist_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // セットリストのupdated_atを更新
    let now = Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE setlists SET updated_at = ? WHERE id = ?",
        now, setlist_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    // コミット
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
}
