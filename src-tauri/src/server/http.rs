use axum::{
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use tower_http::cors::CorsLayer;

/// HTTPサーバーを起動
pub async fn start_http_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/overlay/comment", get(overlay_comment))
        .route("/overlay/setlist", get(overlay_setlist))
        .layer(CorsLayer::permissive());

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
async fn overlay_comment() -> impl IntoResponse {
    Html(include_str!("../../overlays/comment.html"))
}

/// セットリストオーバーレイHTML
async fn overlay_setlist() -> impl IntoResponse {
    Html(include_str!("../../overlays/setlist.html"))
}
