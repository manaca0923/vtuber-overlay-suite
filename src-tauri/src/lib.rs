mod commands;
mod youtube;

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
    .invoke_handler(tauri::generate_handler![
      commands::youtube::validate_api_key,
      commands::youtube::get_live_chat_id,
      commands::youtube::get_chat_messages,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
