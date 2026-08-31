mod core;
mod media;
mod plugin;

use core::bass_bridge;
use core::i18n;
use core::paths;
use core::settings;
use core::smtc_bridge;
use media::media_library;
use plugin::manager as plugin_manager;

use serde_json::json;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

const DROPIN_SDK_JS: &[u8] = include_bytes!("../../plugin-sdk/js/dropin-sdk.js");

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn open_devtools(app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is unavailable".to_string())?;
    window.open_devtools();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let protocol_manager: Arc<Mutex<Option<plugin_manager::PluginManager>>> =
        Arc::new(Mutex::new(None));
    let protocol_manager_for_handler = protocol_manager.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_decorum::init())
        .register_uri_scheme_protocol("dropin-plugin", move |_context, request| {
            let mut path = request.uri().path().to_string();
            // WebView implementations differ on whether the custom-scheme host
            // is included in the path. Normalize both forms before sandboxing.
            if let Some(stripped) = path.strip_prefix("/localhost/") {
                path = format!("/{stripped}");
            }
            let response = match path.as_str() {
                "/sdk/dropin-sdk.js" | "/js/dropin-sdk.js" => Some((
                    DROPIN_SDK_JS.to_vec(),
                    "text/javascript; charset=utf-8".to_string(),
                )),
                _ => protocol_manager_for_handler
                    .lock()
                    .ok()
                    .and_then(|manager| {
                        manager
                            .as_ref()
                            .and_then(|manager| manager.serve(&path).ok())
                    }),
            };
            match response {
                Some((body, mime)) => tauri::http::Response::builder()
                    .header("Content-Type", mime)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(body)
                    .unwrap(),
                None => tauri::http::Response::builder()
                    .status(tauri::http::StatusCode::NOT_FOUND)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(b"plugin resource not found".to_vec())
                    .unwrap(),
            }
        })
        .setup(move |app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();

            #[cfg(target_os = "macos")]
            main_window.set_traffic_lights_inset(16.0, 20.0).unwrap();

            let plugin_worker_shutdown = Arc::new(AtomicBool::new(false));
            let bass_service = bass_bridge::BassService::new(app.handle().clone());
            let bass_cleanup_service = bass_service.clone();
            app.manage(bass_service.clone());
            let smtc_service = smtc_bridge::SmtcService::new(app.handle().clone());
            let smtc_cleanup_service = smtc_service.clone();
            let plugin_worker_shutdown_for_event = plugin_worker_shutdown.clone();
            app.manage(smtc_service);
            main_window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                    plugin_worker_shutdown_for_event.store(true, Ordering::Relaxed);
                    let _ = bass_cleanup_service.call_operation("bass_unload", json!({}));
                    let _ = smtc_cleanup_service.call_operation("smtc_close", json!({}));
                }
            });
            let app_paths = paths::app_paths_from_app(app.handle());
            let _ = app_paths.prepare();
            let media_service = media_library::MediaService::new(
                app.handle().clone(),
                paths::app_paths_from_app(app.handle()),
            );
            let plugin_manager = plugin_manager::PluginManager::new(app_paths.clone());
            if let Ok(mut slot) = protocol_manager.lock() {
                *slot = Some(plugin_manager.clone());
            }
            let plugin_worker_manager = plugin_manager.clone();
            let plugin_worker_bass = bass_service.clone();
            let plugin_worker_media = media_service.clone();
            let plugin_worker_app = app.handle().clone();
            std::thread::spawn(move || {
                while !plugin_worker_shutdown.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(1_000));
                    if plugin_worker_shutdown.load(Ordering::Relaxed) {
                        break;
                    }
                    let _ = plugin_worker_manager.tick_background(
                        &plugin_worker_bass,
                        &plugin_worker_media,
                        &plugin_worker_app,
                    );
                }
            });
            app.manage(plugin_manager);
            app.manage(app_paths);
            app.manage(media_service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            open_devtools,
            bass_bridge::bass_call,
            media_library::media_metadata_read_file,
            media_library::media_metadata_read_url,
            media_library::media_lyrics_read,
            media_library::media_library_add_root,
            media_library::media_library_remove_root,
            media_library::media_library_roots,
            media_library::media_library_scan,
            media_library::media_library_cancel_scan,
            media_library::media_library_tracks,
            media_library::media_library_albums,
            media_library::media_library_artists,
            media_library::media_library_refresh_track,
            media_library::media_library_remove_track,
            media_library::media_cover_get,
            media_library::media_cover_path,
            media_library::media_playback_history,
            media_library::media_playback_record,
            media_library::media_pick_folder,
            media_library::media_playback_open,
            media_library::media_playlist_create,
            media_library::media_playlist_remove,
            media_library::media_playlist_rename,
            media_library::media_playlist_list,
            media_library::media_playlist_add_track,
            media_library::media_playlist_remove_track,
            media_library::media_playlist_rule_get,
            media_library::media_playlist_rule_save,
            media_library::media_playlist_rule_evaluate,
            media_library::media_playlist_rule_materialize,
            media_library::media_tag_create,
            media_library::media_tag_remove,
            media_library::media_tag_list,
            media_library::media_track_tag,
            media_library::media_track_untag,
            settings::app_settings_read,
            settings::app_settings_write,
            settings::data_dir_read,
            settings::data_dir_set,
            i18n::i18n_list_custom,
            i18n::i18n_load_custom,
            smtc_bridge::smtc_call,
            plugin_manager::plugin_list,
            plugin_manager::plugin_pick_package,
            plugin_manager::plugin_install,
            plugin_manager::plugin_uninstall,
            plugin_manager::plugin_enable,
            plugin_manager::plugin_disable,
            plugin_manager::plugin_get_permissions,
            plugin_manager::plugin_set_permissions,
            plugin_manager::plugin_call,
            plugin_manager::plugin_update_host_state,
            plugin_manager::plugin_get_ui_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
