use crate::db::models::{
    Setlist, SetlistSongWithDetails, SetlistWithSongs, Song, SongStatus,
};
use crate::AppState;
use chrono::Utc;
use std::cmp::Ordering;
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
    let pool = &state.db;
    let mut song = Song::new(title);
    song.artist = artist;
    song.category = category;
    song.duration_seconds = duration_seconds;

    // tagsをJSON文字列に変換
    let tags_json = tags.and_then(|t| serde_json::to_string(&t).ok());
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
    let pool = &state.db;
    let now = Utc::now().to_rfc3339();

    // tagsをJSON文字列に変換
    let tags_json = tags.and_then(|t| serde_json::to_string(&t).ok());

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

/// セットリストに楽曲を追加
#[tauri::command]
pub async fn add_song_to_setlist(
    setlist_id: String,
    song_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let pool = &state.db;

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
