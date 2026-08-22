mod bass_bridge;
mod i18n;
mod lyrics;
mod media_library;
mod paths;
mod settings;
mod smtc_bridge;

use serde_json::json;
use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_decorum::init())
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window.create_overlay_titlebar().unwrap();

            #[cfg(target_os = "macos")]
            main_window.set_traffic_lights_inset(16.0, 20.0).unwrap();

            let bass_service = bass_bridge::BassService::new(app.handle().clone());
            let bass_cleanup_service = bass_service.clone();
            app.manage(bass_service);
            let smtc_service = smtc_bridge::SmtcService::new(app.handle().clone());
            let smtc_cleanup_service = smtc_service.clone();
            app.manage(smtc_service);
            main_window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                    let _ = bass_cleanup_service.call_operation("bass_unload", json!({}));
                    let _ = smtc_cleanup_service.call_operation("smtc_close", json!({}));
                }
            });
            let app_paths = paths::app_paths_from_app(app.handle());
            let _ = app_paths.prepare();
            app.manage(app_paths);
            app.manage(media_library::MediaService::new(
                app.handle().clone(),
                paths::app_paths_from_app(app.handle()),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
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
            smtc_bridge::smtc_call
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
