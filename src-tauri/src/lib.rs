mod commands;
mod server;
mod youtube;

use std::sync::{Arc, Mutex};

/// アプリケーション全体の共有状態
pub struct AppState {
    pub poller: Arc<Mutex<Option<youtube::poller::ChatPoller>>>,
    pub server: server::ServerState,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  // サーバー用の共有状態を作成
  let server_state = server::create_server_state();

  // manageに渡す用にcloneしておく
  let server_state_for_manage = Arc::clone(&server_state);

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
      commands::youtube::broadcast_setlist_update,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
