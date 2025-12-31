mod commands;
mod config;
mod db;
mod keyring;
mod server;
mod superchat;
pub mod util; // doctestのためpubにする
mod weather;
mod youtube;

use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// アプリケーションID（tauri.conf.jsonのidentifierと一致させる）
const APP_IDENTIFIER: &str = "com.vtuber-overlay-suite.desktop";

/// アプリケーション全体の共有状態
pub struct AppState {
    pub poller: Arc<Mutex<Option<youtube::poller::ChatPoller>>>,
    pub server: server::ServerState,
    pub db: SqlitePool,
    pub weather: Arc<weather::WeatherClient>,
    pub weather_updater: Arc<weather::WeatherAutoUpdater>,
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

  // HTTPサーバー用にdb_poolをclone
  let db_pool_for_http = db_pool.clone();
  // WebSocketサーバー用にdb_poolをclone
  let db_pool_for_ws = db_pool.clone();

  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
    .setup(move |app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // HTTPサーバーを起動（DB接続付き）
      let http_db = db_pool_for_http.clone();
      
      // オーバーレイディレクトリのパスを取得
      // 開発中はsrc-tauri/overlays、本番ではリソースディレクトリを使用
      let overlays_dir = if cfg!(debug_assertions) {
        // 開発環境：src-tauri/overlaysを直接参照
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("overlays")
      } else {
        // 本番環境：アプリバンドル内のリソースを使用
        app.path().resource_dir()
          .expect("Failed to get resource directory")
          .join("overlays")
      };
      
      log::info!("Overlays directory: {:?}", overlays_dir);
      
      tauri::async_runtime::spawn(async move {
        if let Err(e) = server::start_http_server_with_db(http_db, overlays_dir).await {
          log::error!("HTTP server error: {}", e);
        }
      });

      // WebSocketサーバーを起動（Tauriのランタイム内で起動）
      {
        let state_clone = Arc::clone(&server_state);
        let ws_db = db_pool_for_ws.clone();
        tauri::async_runtime::spawn(async move {
          if let Err(e) = server::start_websocket_server(state_clone, ws_db).await {
            log::error!("WebSocket server error: {}", e);
          }
        });
      }

      Ok(())
    })
    .manage({
      // 天気クライアントを作成（Open-Meteo APIはAPIキー不要）
      let weather_client = Arc::new(weather::WeatherClient::new());

      // 天気自動更新タスクを開始（15分ごとにブロードキャスト）
      let weather_updater = Arc::new(weather::WeatherAutoUpdater::start(
        Arc::clone(&weather_client),
        Arc::clone(&server_state_for_manage),
      ));

      AppState {
        poller: Arc::new(Mutex::new(None)),
        server: server_state_for_manage,
        db: db_pool,
        weather: weather_client,
        weather_updater,
      }
    })
    .invoke_handler({
      // デバッグビルドではtest_innertube_connectionを含む
      #[cfg(debug_assertions)]
      {
        tauri::generate_handler![
          commands::youtube::validate_api_key,
          commands::youtube::get_live_chat_id,
          commands::youtube::get_chat_messages,
          commands::youtube::start_polling,
          commands::youtube::stop_polling,
          commands::youtube::get_polling_state,
          commands::youtube::get_quota_info,
          commands::youtube::is_polling_running,
          commands::youtube::send_test_comment,
          commands::youtube::save_polling_state,
          commands::youtube::load_polling_state,
          commands::youtube::save_wizard_settings,
          commands::youtube::load_wizard_settings,
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
          commands::setlist::broadcast_setlist_update,
          commands::keyring::save_api_key,
          commands::keyring::get_api_key,
          commands::keyring::delete_api_key,
          commands::keyring::has_api_key,
          commands::overlay::save_overlay_settings,
          commands::overlay::load_overlay_settings,
          commands::overlay::broadcast_settings_update,
          commands::template::validate_template,
          commands::template::get_default_template,
          commands::youtube::save_api_mode,
          commands::youtube::load_api_mode,
          commands::youtube::test_innertube_connection,
          commands::youtube::start_polling_innertube,
          commands::youtube::stop_polling_innertube,
          commands::youtube::is_polling_innertube_running,
          commands::youtube::get_api_key_status,
          commands::youtube::has_bundled_api_key,
          commands::youtube::set_byok_key,
          commands::youtube::get_active_api_key,
          commands::youtube::switch_to_secondary_key,
          commands::youtube::reset_to_primary_key,
          commands::youtube::start_unified_polling,
          commands::youtube::stop_unified_polling,
          commands::youtube::is_unified_polling_running,
          commands::youtube::get_unified_polling_mode,
          commands::youtube::get_live_stream_stats,
          commands::youtube::broadcast_kpi_update,
          commands::weather::set_weather_city,
          commands::weather::get_weather_city,
          commands::weather::get_weather,
          commands::weather::fetch_weather,
          commands::weather::broadcast_weather_update,
          commands::weather::clear_weather_cache,
          commands::weather::get_weather_cache_ttl,
          commands::weather::refresh_weather,
          commands::weather::broadcast_weather,
          commands::weather::set_weather_city_and_broadcast,
          commands::weather::get_weather_multi,
          commands::weather::broadcast_weather_multi,
          commands::weather::set_multi_city_mode,
          commands::system::get_system_fonts,
        ]
      }
      // リリースビルドではtest_innertube_connectionを除外
      #[cfg(not(debug_assertions))]
      {
        tauri::generate_handler![
          commands::youtube::validate_api_key,
          commands::youtube::get_live_chat_id,
          commands::youtube::get_chat_messages,
          commands::youtube::start_polling,
          commands::youtube::stop_polling,
          commands::youtube::get_polling_state,
          commands::youtube::get_quota_info,
          commands::youtube::is_polling_running,
          commands::youtube::send_test_comment,
          commands::youtube::save_polling_state,
          commands::youtube::load_polling_state,
          commands::youtube::save_wizard_settings,
          commands::youtube::load_wizard_settings,
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
          commands::setlist::broadcast_setlist_update,
          commands::keyring::save_api_key,
          commands::keyring::get_api_key,
          commands::keyring::delete_api_key,
          commands::keyring::has_api_key,
          commands::overlay::save_overlay_settings,
          commands::overlay::load_overlay_settings,
          commands::overlay::broadcast_settings_update,
          commands::template::validate_template,
          commands::template::get_default_template,
          commands::youtube::save_api_mode,
          commands::youtube::load_api_mode,
          commands::youtube::start_polling_innertube,
          commands::youtube::stop_polling_innertube,
          commands::youtube::is_polling_innertube_running,
          commands::youtube::get_api_key_status,
          commands::youtube::has_bundled_api_key,
          commands::youtube::set_byok_key,
          commands::youtube::get_active_api_key,
          commands::youtube::switch_to_secondary_key,
          commands::youtube::reset_to_primary_key,
          commands::youtube::start_unified_polling,
          commands::youtube::stop_unified_polling,
          commands::youtube::is_unified_polling_running,
          commands::youtube::get_unified_polling_mode,
          commands::youtube::get_live_stream_stats,
          commands::youtube::broadcast_kpi_update,
          commands::weather::set_weather_city,
          commands::weather::get_weather_city,
          commands::weather::get_weather,
          commands::weather::fetch_weather,
          commands::weather::broadcast_weather_update,
          commands::weather::clear_weather_cache,
          commands::weather::get_weather_cache_ttl,
          commands::weather::refresh_weather,
          commands::weather::broadcast_weather,
          commands::weather::set_weather_city_and_broadcast,
          commands::weather::get_weather_multi,
          commands::weather::broadcast_weather_multi,
          commands::weather::set_multi_city_mode,
          commands::system::get_system_fonts,
        ]
      }
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
