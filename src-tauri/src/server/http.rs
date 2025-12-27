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
use tower_http::services::ServeDir;

use super::types::{CommentPosition, LayoutPreset, SetlistPosition};

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

    // 静的ファイル配信
    let shared_dir = state.overlays_dir.join("shared");
    let components_dir = state.overlays_dir.join("components");
    let styles_dir = state.overlays_dir.join("styles");
    let serve_shared = ServeDir::new(&shared_dir);
    let serve_components = ServeDir::new(&components_dir);
    let serve_styles = ServeDir::new(&styles_dir);

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/setlist/latest", get(get_latest_setlist_api))
        .route("/api/setlist/{id}", get(get_setlist_api))
        .route("/api/overlay/settings", get(get_overlay_settings_api))
        .route("/overlay/comment", get(overlay_comment))
        .route("/overlay/setlist", get(overlay_setlist))
        .route("/overlay/combined", get(overlay_combined))
        .route("/overlay/combined-v2", get(overlay_combined_v2))
        .nest_service("/overlay/shared", serve_shared)
        .nest_service("/overlay/components", serve_components)
        .nest_service("/overlay/styles", serve_styles)
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

/// 統合オーバーレイHTML（コメント＋セットリスト）
async fn overlay_combined(State(state): State<HttpState>) -> impl IntoResponse {
    let path = state.overlays_dir.join("combined.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Html(content).into_response(),
        Err(e) => {
            log::error!("Failed to read combined.html from {:?}: {}", path, e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load overlay".to_string(),
            ).into_response()
        }
    }
}

/// 3カラム統合オーバーレイHTML v2（22%/56%/22%固定レイアウト）
async fn overlay_combined_v2(State(state): State<HttpState>) -> impl IntoResponse {
    let path = state.overlays_dir.join("combined-v2.html");
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Html(content).into_response(),
        Err(e) => {
            log::error!("Failed to read combined-v2.html from {:?}: {}", path, e);
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
    layout: LayoutPreset,
    primary_color: String,
    font_family: String,
    border_radius: u32,
    comment: CommentSettingsApi,
    setlist: SetlistSettingsApi,
    #[serde(skip_serializing_if = "Option::is_none")]
    weather: Option<WeatherSettingsApi>,
}

/// NOTE: maxCountは画面高さベースの自動調整に統一したため削除
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CommentSettingsApi {
    enabled: bool,
    position: CommentPosition,
    show_avatar: bool,
    font_size: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetlistSettingsApi {
    enabled: bool,
    position: SetlistPosition,
    show_artist: bool,
    font_size: u32,
}

/// 天気ウィジェットの表示位置
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum WeatherPositionApi {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WeatherSettingsApi {
    enabled: bool,
    position: WeatherPositionApi,
}

/// 文字列からWeatherPositionに変換
fn parse_weather_position(s: &str) -> WeatherPositionApi {
    match s {
        "left-bottom" => WeatherPositionApi::LeftBottom,
        "right-top" => WeatherPositionApi::RightTop,
        "right-bottom" => WeatherPositionApi::RightBottom,
        _ => WeatherPositionApi::LeftTop, // デフォルト
    }
}

/// デフォルトのオーバーレイ設定を生成
fn default_overlay_settings() -> OverlaySettingsApiResponse {
    OverlaySettingsApiResponse {
        theme: "default".to_string(),
        layout: LayoutPreset::Streaming,
        primary_color: "#6366f1".to_string(),
        font_family: "'Yu Gothic', 'Meiryo', sans-serif".to_string(),
        border_radius: 8,
        comment: CommentSettingsApi {
            enabled: true,
            position: CommentPosition::BottomRight,
            show_avatar: true,
            font_size: 16,
        },
        setlist: SetlistSettingsApi {
            enabled: true,
            position: SetlistPosition::Bottom,
            show_artist: true,
            font_size: 24,
        },
        weather: Some(WeatherSettingsApi {
            enabled: true,
            position: WeatherPositionApi::LeftTop,
        }),
    }
}

/// 文字列からCommentPositionに変換
fn parse_comment_position(s: &str) -> CommentPosition {
    match s {
        "top-left" => CommentPosition::TopLeft,
        "top-right" => CommentPosition::TopRight,
        "bottom-left" => CommentPosition::BottomLeft,
        _ => CommentPosition::BottomRight, // デフォルト
    }
}

/// 文字列からSetlistPositionに変換
fn parse_setlist_position(s: &str) -> SetlistPosition {
    match s {
        "top" => SetlistPosition::Top,
        "left" => SetlistPosition::Left,
        "right" => SetlistPosition::Right,
        _ => SetlistPosition::Bottom, // デフォルト
    }
}

/// 文字列からLayoutPresetに変換
fn parse_layout_preset(s: &str) -> LayoutPreset {
    match s {
        "streaming" => LayoutPreset::Streaming,
        "talk" => LayoutPreset::Talk,
        "music" => LayoutPreset::Music,
        "gaming" => LayoutPreset::Gaming,
        "custom" => LayoutPreset::Custom,
        "three-column" => LayoutPreset::ThreeColumn,
        _ => LayoutPreset::Streaming, // デフォルト
    }
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
                    // 天気設定をパース（存在する場合のみ）
                    let weather = if settings["weather"].is_object() {
                        Some(WeatherSettingsApi {
                            enabled: settings["weather"]["enabled"].as_bool().unwrap_or(true),
                            position: parse_weather_position(
                                settings["weather"]["position"].as_str().unwrap_or("left-top")
                            ),
                        })
                    } else {
                        // デフォルト値
                        Some(WeatherSettingsApi {
                            enabled: true,
                            position: WeatherPositionApi::LeftTop,
                        })
                    };

                    let response = OverlaySettingsApiResponse {
                        theme: settings["theme"].as_str().unwrap_or("default").to_string(),
                        layout: parse_layout_preset(
                            settings["layout"].as_str().unwrap_or("streaming")
                        ),
                        primary_color: settings["common"]["primaryColor"].as_str().unwrap_or("#6366f1").to_string(),
                        font_family: settings["common"]["fontFamily"].as_str().unwrap_or("'Yu Gothic', 'Meiryo', sans-serif").to_string(),
                        border_radius: settings["common"]["borderRadius"].as_u64().unwrap_or(8) as u32,
                        comment: CommentSettingsApi {
                            enabled: settings["comment"]["enabled"].as_bool().unwrap_or(true),
                            position: parse_comment_position(
                                settings["comment"]["position"].as_str().unwrap_or("bottom-right")
                            ),
                            show_avatar: settings["comment"]["showAvatar"].as_bool().unwrap_or(true),
                            font_size: settings["comment"]["fontSize"].as_u64().unwrap_or(16) as u32,
                        },
                        setlist: SetlistSettingsApi {
                            enabled: settings["setlist"]["enabled"].as_bool().unwrap_or(true),
                            position: parse_setlist_position(
                                settings["setlist"]["position"].as_str().unwrap_or("bottom")
                            ),
                            show_artist: settings["setlist"]["showArtist"].as_bool().unwrap_or(true),
                            font_size: settings["setlist"]["fontSize"].as_u64().unwrap_or(24) as u32,
                        },
                        weather,
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
            // 設定が保存されていない場合はデフォルト値を返す
            let response = default_overlay_settings();
            Json(response).into_response()
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
