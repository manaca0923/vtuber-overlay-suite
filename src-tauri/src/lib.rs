mod commands;
mod youtube;

use commands::youtube::PollerState;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .manage(PollerState {
      poller: Arc::new(Mutex::new(None)),
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
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
