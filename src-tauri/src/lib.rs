mod commands;
mod db;
mod keyring;
mod server;
mod util;
mod youtube;

use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};

/// アプリケーションID（tauri.conf.jsonのidentifierと一致させる）
const APP_IDENTIFIER: &str = "com.vtuber-overlay-suite.desktop";

/// アプリケーション全体の共有状態
pub struct AppState {
    pub poller: Arc<Mutex<Option<youtube::poller::ChatPoller>>>,
    pub server: server::ServerState,
    pub db: SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // サーバー用の共有状態を作成
  let server_state = server::create_server_state();

  // manageに渡す用にcloneしておく
  let server_state_for_manage = Arc::clone(&server_state);

  // データベース初期化（setup前に実行）
  let db_pool = {
    let app_dir = dirs::data_dir()
      .expect("Failed to get data directory");
    let app_dir_path = app_dir.join(APP_IDENTIFIER);
    std::fs::create_dir_all(&app_dir_path).expect("Failed to create app data directory");
    let db_path = app_dir_path.join("app.db");
    tauri::async_runtime::block_on(async {
      db::create_pool(db_path.to_str().unwrap())
        .await
        .expect("Failed to create database pool")
    })
  };

  tauri::Builder::default()
    .setup(move |app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // HTTPサーバーを起動（Tauriのランタイム内で起動）
      tokio::spawn(async move {
        if let Err(e) = server::start_http_server().await {
          log::error!("HTTP server error: {}", e);
        }
      });

      // WebSocketサーバーを起動（Tauriのランタイム内で起動）
      {
        let state_clone = Arc::clone(&server_state);
        tokio::spawn(async move {
          if let Err(e) = server::start_websocket_server(state_clone).await {
            log::error!("WebSocket server error: {}", e);
          }
        });
      }

      Ok(())
    })
    .manage(AppState {
      poller: Arc::new(Mutex::new(None)),
      server: server_state_for_manage,
      db: db_pool,
    })
    .invoke_handler(tauri::generate_handler![
      commands::youtube::validate_api_key,
      commands::youtube::get_live_chat_id,
      commands::youtube::get_chat_messages,
      commands::youtube::start_polling,
      commands::youtube::stop_polling,
      commands::youtube::get_polling_state,
      commands::youtube::get_quota_info,
      commands::youtube::is_polling_running,
      commands::setlist::get_songs,
      commands::setlist::create_song,
      commands::setlist::update_song,
      commands::setlist::delete_song,
      commands::setlist::get_setlists,
      commands::setlist::create_setlist,
      commands::setlist::delete_setlist,
      commands::setlist::add_song_to_setlist,
      commands::setlist::remove_song_from_setlist,
      commands::setlist::get_setlist_with_songs,
      commands::setlist::set_current_song,
      commands::setlist::next_song,
      commands::setlist::previous_song,
      commands::setlist::reorder_setlist_songs,
      commands::keyring::save_api_key,
      commands::keyring::get_api_key,
      commands::keyring::delete_api_key,
      commands::keyring::has_api_key,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
