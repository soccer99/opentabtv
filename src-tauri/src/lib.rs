//! Tablo Desktop Application
//!
//! A Tauri application for streaming live TV and recordings from Tablo DVR devices.

mod commands;
mod error;
mod tablo;

use tablo::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create shared application state
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Device commands
            commands::discover_devices,
            commands::connect_by_ip,
            commands::connect_device,
            commands::get_active_device,
            commands::disconnect_device,
            commands::get_server_info,
            commands::get_last_device,
            // Channel commands
            commands::get_channels,
            // Guide commands
            commands::get_guide_airings,
            // Streaming commands
            commands::start_live_stream,
            commands::stop_stream,
            commands::get_active_streams,
            // Recording commands
            commands::get_recordings,
            commands::get_recording_paths,
            commands::watch_recording,
            commands::batch_get,
            // Authentication commands (4th Gen)
            commands::login,
            commands::logout,
            commands::is_logged_in,
            commands::save_credentials,
            commands::load_credentials,
            commands::clear_credentials,
            commands::has_saved_credentials,
            commands::discover_cloud_devices,
            commands::connect_gen4_device,
            // External tools detection
            commands::detect_ffmpeg,
            commands::detect_vlc,
            commands::open_in_vlc,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
