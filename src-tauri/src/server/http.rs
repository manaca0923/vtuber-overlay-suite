use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;
use serde_json::json;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// HTTPサーバー用の共有状態
#[derive(Clone)]
pub struct HttpState {
    pub db: Arc<SqlitePool>,
    pub overlays_dir: PathBuf,
}

/// HTTPサーバーを起動（DB接続付き）
pub async fn start_http_server_with_db(db: SqlitePool, overlays_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let state = HttpState {
        db: Arc::new(db),
        overlays_dir,
    };

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/setlist/latest", get(get_latest_setlist_api))
        .route("/api/setlist/{id}", get(get_setlist_api))
        .route("/api/overlay/settings", get(get_overlay_settings_api))
        .route("/overlay/comment", get(overlay_comment))
        .route("/overlay/setlist", get(overlay_setlist))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "127.0.0.1:19800";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    log::info!("HTTP server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// ヘルスチェックエンドポイント
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "server": "vtuber-overlay-suite",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// コメントオーバーレイHTML
async fn overlay_comment(State(state): State<HttpState>) -> impl IntoResponse {
    let path = state.overlays_dir.join("comment.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Html(content).into_response(),
        Err(e) => {
            // パス情報をログには記録するがレスポンスには含めない
            log::error!("Failed to read comment.html from {:?}: {}", path, e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load overlay".to_string(),
            ).into_response()
        }
    }
}

/// セットリストオーバーレイHTML
async fn overlay_setlist(State(state): State<HttpState>) -> impl IntoResponse {
    let path = state.overlays_dir.join("setlist.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Html(content).into_response(),
        Err(e) => {
            // パス情報をログには記録するがレスポンスには含めない
            log::error!("Failed to read setlist.html from {:?}: {}", path, e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load overlay".to_string(),
            ).into_response()
        }
    }
}

/// セットリストAPIレスポンス
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetlistApiResponse {
    setlist: SetlistInfo,
    songs: Vec<SongInfo>,
    current_index: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetlistInfo {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SongInfo {
    id: String,
    position: i64,
    title: String,
    artist: Option<String>,
    status: String,
}

/// セットリスト詳細を取得する共通関数
async fn fetch_setlist_by_id(
    pool: &SqlitePool,
    setlist_id: &str,
) -> Result<SetlistApiResponse, (axum::http::StatusCode, Json<serde_json::Value>)> {
    // セットリスト基本情報取得
    let setlist_result = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM setlists WHERE id = ?"
    )
    .bind(setlist_id)
    .fetch_optional(pool)
    .await;

    let (setlist_id, setlist_name) = match setlist_result {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({ "error": "Setlist not found" })),
            ));
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            ));
        }
    };

    // 楽曲リスト取得
    let songs_result = sqlx::query_as::<_, (String, i64, String, Option<String>, Option<String>, Option<String>)>(
        r#"
        SELECT
            ss.id, ss.position, s.title, s.artist, ss.started_at, ss.ended_at
        FROM setlist_songs ss
        JOIN songs s ON ss.song_id = s.id
        WHERE ss.setlist_id = ?
        ORDER BY ss.position
        "#
    )
    .bind(&setlist_id)
    .fetch_all(pool)
    .await;

    let rows = match songs_result {
        Ok(rows) => rows,
        Err(e) => {
            log::error!("Database error: {}", e);
            return Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            ));
        }
    };

    // 現在の曲のインデックスを計算
    // Note: current_indexは配列のインデックス（0始まり連続）で、row.1のpositionとは異なる
    let current_index = rows
        .iter()
        .position(|row| row.4.is_some() && row.5.is_none()) // started_at.is_some() && ended_at.is_none()
        .map(|i| i as i64)
        .unwrap_or(-1);

    // レスポンス構築
    let songs: Vec<SongInfo> = rows
        .into_iter()
        .enumerate()
        .map(|(idx, row)| {
            let status = if current_index == -1 {
                "pending".to_string()
            } else if (idx as i64) < current_index {
                "done".to_string()
            } else if (idx as i64) == current_index {
                "current".to_string()
            } else {
                "pending".to_string()
            };

            SongInfo {
                id: row.0,
                position: row.1,
                title: row.2,
                artist: row.3,
                status,
            }
        })
        .collect();

    Ok(SetlistApiResponse {
        setlist: SetlistInfo {
            id: setlist_id,
            name: setlist_name,
        },
        songs,
        current_index,
    })
}

/// セットリストAPI（オーバーレイ初期化用）
async fn get_setlist_api(
    State(state): State<HttpState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.as_ref();
    match fetch_setlist_by_id(pool, &id).await {
        Ok(response) => Json(response).into_response(),
        Err((status, json)) => (status, json).into_response(),
    }
}

/// オーバーレイ設定API（オーバーレイ初期化用）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlaySettingsApiResponse {
    theme: String,
    primary_color: String,
    font_family: String,
    border_radius: u32,
    comment: CommentSettingsApi,
    setlist: SetlistSettingsApi,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommentSettingsApi {
    enabled: bool,
    position: String,
    max_count: u32,
    show_avatar: bool,
    font_size: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetlistSettingsApi {
    enabled: bool,
    position: String,
    show_artist: bool,
    font_size: u32,
}

/// 保存されているオーバーレイ設定を取得
async fn get_overlay_settings_api(
    State(state): State<HttpState>,
) -> impl IntoResponse {
    let pool = state.db.as_ref();

    let result: Result<Option<(String,)>, sqlx::Error> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'overlay_settings'")
            .fetch_optional(pool)
            .await;

    match result {
        Ok(Some((json_str,))) => {
            // JSONをパースして返す
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(settings) => {
                    // SettingsUpdatePayloadと同じ形式に変換
                    let response = OverlaySettingsApiResponse {
                        theme: settings["theme"].as_str().unwrap_or("default").to_string(),
                        primary_color: settings["common"]["primaryColor"].as_str().unwrap_or("#6366f1").to_string(),
                        font_family: settings["common"]["fontFamily"].as_str().unwrap_or("'Yu Gothic', 'Meiryo', sans-serif").to_string(),
                        border_radius: settings["common"]["borderRadius"].as_u64().unwrap_or(8) as u32,
                        comment: CommentSettingsApi {
                            enabled: settings["comment"]["enabled"].as_bool().unwrap_or(true),
                            position: settings["comment"]["position"].as_str().unwrap_or("bottom-right").to_string(),
                            max_count: settings["comment"]["maxCount"].as_u64().unwrap_or(10) as u32,
                            show_avatar: settings["comment"]["showAvatar"].as_bool().unwrap_or(true),
                            font_size: settings["comment"]["fontSize"].as_u64().unwrap_or(16) as u32,
                        },
                        setlist: SetlistSettingsApi {
                            enabled: settings["setlist"]["enabled"].as_bool().unwrap_or(true),
                            position: settings["setlist"]["position"].as_str().unwrap_or("bottom").to_string(),
                            show_artist: settings["setlist"]["showArtist"].as_bool().unwrap_or(true),
                            font_size: settings["setlist"]["fontSize"].as_u64().unwrap_or(24) as u32,
                        },
                    };
                    Json(response).into_response()
                }
                Err(e) => {
                    log::error!("Failed to parse overlay settings: {}", e);
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": "Failed to parse settings" })),
                    ).into_response()
                }
            }
        }
        Ok(None) => {
            // 設定が保存されていない場合は404
            (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({ "error": "No overlay settings found" })),
            ).into_response()
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            ).into_response()
        }
    }
}

/// 最新（最初）のセットリストを取得（オーバーレイ初期化用）
async fn get_latest_setlist_api(
    State(state): State<HttpState>,
) -> impl IntoResponse {
    let pool = state.db.as_ref();

    // 最新のセットリストIDを取得
    let setlist_result = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM setlists ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await;

    let setlist_id = match setlist_result {
        Ok(Some((id,))) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({ "error": "No setlists found" })),
            ).into_response();
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database error" })),
            ).into_response();
        }
    };

    // 共通関数を呼び出し
    match fetch_setlist_by_id(pool, &setlist_id).await {
        Ok(response) => Json(response).into_response(),
        Err((status, json)) => (status, json).into_response(),
    }
}
