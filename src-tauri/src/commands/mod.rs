//! Tauri IPC commands for frontend communication

use tauri::State;

use crate::error::TabloError;
use crate::tablo::cloud::LighthouseClient;
use crate::tablo::config::TabloConfig;
use crate::tablo::discovery;
use crate::tablo::{
    AppState, Channel, Credentials, DeviceGeneration, ServerInfo, StreamSession, TabloAccount,
    TabloDevice,
};

/// Discover Tablo devices on the local network
#[tauri::command]
pub async fn discover_devices() -> Result<Vec<TabloDevice>, TabloError> {
    tracing::info!("Starting device discovery...");
    let devices = discovery::discover_devices().await?;
    tracing::info!("Found {} devices", devices.len());
    Ok(devices)
}

/// Connect to a Tablo device by IP address
#[tauri::command]
pub async fn connect_by_ip(
    ip: String,
    generation: Option<String>,
    state: State<'_, AppState>,
) -> Result<TabloDevice, TabloError> {
    tracing::info!("Connecting to device at {}", ip);

    // Verify device is reachable
    if !discovery::check_device_connection(&ip).await? {
        return Err(TabloError::ConnectionFailed(format!(
            "Cannot connect to {} on port 8885",
            ip
        )));
    }

    // Determine generation - if not specified, try to auto-detect
    let device_generation = match generation.as_deref() {
        Some("gen4") => DeviceGeneration::Gen4,
        Some("legacy") => DeviceGeneration::Legacy,
        _ => {
            // Try to detect: Legacy devices respond without auth, 4th Gen require it
            // First try without auth (legacy)
            let legacy_client = crate::tablo::TabloClient::new(&ip)?;
            match legacy_client.server_info().await {
                Ok(_) => DeviceGeneration::Legacy,
                Err(e) => {
                    // Check if it's a 401 Unauthorized - indicates 4th Gen device
                    let error_str = e.to_string();
                    if error_str.contains("401") || error_str.contains("Unauthorized") {
                        tracing::info!("Device at {} requires auth, trying as 4th Gen", ip);
                        DeviceGeneration::Gen4
                    } else {
                        // Other errors - could be network issues, timeout, etc.
                        return Err(TabloError::ConnectionFailed(format!(
                            "Failed to connect to device at {}: {}",
                            ip, e
                        )));
                    }
                }
            }
        }
    };

    // Create client with appropriate generation
    let client = crate::tablo::TabloClient::new_with_generation(&ip, device_generation)?;
    let info = match client.server_info().await {
        Ok(info) => info,
        Err(e) => {
            // If 4th Gen auth failed, give a helpful error
            if device_generation == DeviceGeneration::Gen4 {
                let error_str = e.to_string();
                if error_str.contains("401") || error_str.contains("Unauthorized") {
                    return Err(TabloError::ApiError(
                        "4th Gen device authentication failed. The HMAC authentication may not be \
                        compatible with this device. Please try using '4th Gen Login' to connect \
                        via cloud authentication instead.".into()
                    ));
                }
            }
            return Err(e);
        }
    };

    let device = TabloDevice {
        id: info
            .server_id
            .clone()
            .unwrap_or_else(|| format!("tablo-{}", ip)),
        name: info.name,
        local_ip: ip.clone(),
        model: info.model,
        version: Some(info.version),
        tuners: info.total_tuners,
        server_id: info.server_id,
        generation: device_generation,
        sid: None,
        account_token: None,
        lighthouse_token: None,
    };

    // Set as active device
    state.tablo.set_device(device.clone())?;

    // Save last device
    let _ = TabloConfig::save_last_device(&ip, &device.id);

    tracing::info!("Connected to {} ({:?})", device.name, device_generation);
    Ok(device)
}

/// Connect to a discovered device
#[tauri::command]
pub async fn connect_device(
    device: TabloDevice,
    state: State<'_, AppState>,
) -> Result<TabloDevice, TabloError> {
    // Pass the device's generation to connect_by_ip
    let generation = match device.generation {
        DeviceGeneration::Legacy => Some("legacy".to_string()),
        DeviceGeneration::Gen4 => Some("gen4".to_string()),
    };
    connect_by_ip(device.local_ip.clone(), generation, state).await
}

/// Get the currently connected device
#[tauri::command]
pub fn get_active_device(state: State<'_, AppState>) -> Option<TabloDevice> {
    state.tablo.device()
}

/// Disconnect from the current device
#[tauri::command]
pub fn disconnect_device(state: State<'_, AppState>) {
    state.tablo.clear();
    state.streams.clear();
    tracing::info!("Disconnected from device");
}

/// Get server info from the connected device
#[tauri::command]
pub async fn get_server_info(state: State<'_, AppState>) -> Result<ServerInfo, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;
    client.server_info().await
}

/// Get list of channels from the connected device
#[tauri::command]
pub async fn get_channels(state: State<'_, AppState>) -> Result<Vec<Channel>, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;
    client.channels().await
}

/// Start a live TV stream for a channel
#[tauri::command]
pub async fn start_live_stream(
    channel_path: String,
    state: State<'_, AppState>,
) -> Result<StreamSession, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;

    tracing::info!("Starting stream for {}", channel_path);

    let watch_response = client.watch(&channel_path).await?;

    let session = state
        .streams
        .create(&channel_path, watch_response.playlist_url);

    tracing::info!("Stream session created: {}", session.session_id);
    Ok(session)
}

/// Stop a stream session
#[tauri::command]
pub fn stop_stream(session_id: String, state: State<'_, AppState>) {
    state.streams.remove(&session_id);
    tracing::info!("Stream session stopped: {}", session_id);
}

/// Get all active stream sessions
#[tauri::command]
pub fn get_active_streams(_state: State<'_, AppState>) -> Vec<StreamSession> {
    // Sessions are tracked client-side
    Vec::new()
}

/// Get recording paths from the connected device
#[tauri::command]
pub async fn get_recording_paths(state: State<'_, AppState>) -> Result<Vec<String>, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;
    client.recording_paths().await
}

/// Get all recordings with full details from the connected device
#[tauri::command]
pub async fn get_recordings(
    state: State<'_, AppState>,
) -> Result<Vec<crate::tablo::Recording>, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;
    tracing::info!("Fetching recordings...");
    let recordings = client.recordings().await?;
    tracing::info!("Found {} recordings", recordings.len());
    Ok(recordings)
}

/// Start watching a recording
#[tauri::command]
pub async fn watch_recording(
    recording_path: String,
    state: State<'_, AppState>,
) -> Result<StreamSession, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;

    tracing::info!("Starting recording playback for {}", recording_path);

    let watch_response = client.watch(&recording_path).await?;

    let session = state
        .streams
        .create(&recording_path, watch_response.playlist_url);

    tracing::info!("Recording session created: {}", session.session_id);
    Ok(session)
}

/// Batch fetch details for multiple paths
#[tauri::command]
pub async fn batch_get(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;
    client.batch_get(paths).await
}

/// Get guide airings (programs) from the connected device
#[tauri::command]
pub async fn get_guide_airings(
    state: State<'_, AppState>,
) -> Result<Vec<crate::tablo::GuideAiring>, TabloError> {
    let device = state.tablo.device().ok_or(TabloError::NoActiveDevice)?;
    let client = crate::tablo::TabloClient::new_with_device(&device)?;
    tracing::info!("Fetching guide airings...");
    let airings = client.guide_airings().await?;
    tracing::info!("Found {} guide airings", airings.len());
    Ok(airings)
}

// ============================================================================
// Authentication Commands (4th Gen)
// ============================================================================

/// Login to Tablo cloud (4th Gen devices)
#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<TabloAccount, TabloError> {
    tracing::info!("Logging in as {}", email);

    let mut lighthouse = LighthouseClient::new();
    let account = lighthouse
        .login_and_get_account(&Credentials {
            email: email.clone(),
            password: password.clone(),
        })
        .await?;

    // Store the lighthouse client in state
    state.tablo.set_lighthouse(lighthouse);

    tracing::info!(
        "Logged in successfully, found {} devices",
        account.devices.len()
    );
    Ok(account)
}

/// Logout from Tablo cloud
#[tauri::command]
pub fn logout(state: State<'_, AppState>) {
    state.tablo.clear_all();
    tracing::info!("Logged out");
}

/// Check if logged in to Tablo cloud
#[tauri::command]
pub fn is_logged_in(state: State<'_, AppState>) -> bool {
    state.tablo.is_logged_in()
}

/// Save credentials for auto-login
#[tauri::command]
pub fn save_credentials(email: String, password: String) -> Result<(), TabloError> {
    TabloConfig::save_credentials(&email, &password)?;
    tracing::info!("Credentials saved");
    Ok(())
}

/// Load saved credentials
#[tauri::command]
pub fn load_credentials() -> Result<Option<Credentials>, TabloError> {
    TabloConfig::load_credentials()
}

/// Clear saved credentials
#[tauri::command]
pub fn clear_credentials() -> Result<(), TabloError> {
    TabloConfig::clear_credentials()?;
    tracing::info!("Credentials cleared");
    Ok(())
}

/// Check if credentials are saved
#[tauri::command]
pub fn has_saved_credentials() -> bool {
    TabloConfig::has_credentials()
}

/// Discover 4th Gen devices via cloud API
#[tauri::command]
pub async fn discover_cloud_devices(
    state: State<'_, AppState>,
) -> Result<Vec<TabloDevice>, TabloError> {
    let mut lighthouse = state
        .tablo
        .lighthouse()
        .ok_or(TabloError::NotLoggedIn)?;

    tracing::info!("Discovering 4th Gen devices via cloud...");
    let devices = lighthouse.discover_devices().await?;
    tracing::info!("Found {} cloud devices", devices.len());

    Ok(devices)
}

/// Connect to a 4th Gen device (already has tokens from cloud discovery)
#[tauri::command]
pub async fn connect_gen4_device(
    device: TabloDevice,
    state: State<'_, AppState>,
) -> Result<TabloDevice, TabloError> {
    if device.generation != DeviceGeneration::Gen4 {
        return Err(TabloError::ApiError(
            "Device is not a 4th Gen device".into(),
        ));
    }

    // Verify we can reach the device locally
    if !device.local_ip.is_empty()
        && !discovery::check_device_connection(&device.local_ip).await?
    {
        return Err(TabloError::ConnectionFailed(format!(
            "Cannot connect to {} on port 8885",
            device.local_ip
        )));
    }

    // Set as active device
    state.tablo.set_device(device.clone())?;

    // Save last device
    let _ = TabloConfig::save_last_device(&device.local_ip, &device.id);

    tracing::info!("Connected to 4th Gen device: {}", device.name);
    Ok(device)
}

/// Get last connected device info
#[tauri::command]
pub fn get_last_device() -> Result<Option<(String, String)>, TabloError> {
    TabloConfig::get_last_device()
}

// ============================================================================
// External Tools Detection
// ============================================================================

/// External tool detection result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolInfo {
    pub detected: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// Detect FFmpeg installation
#[tauri::command]
pub fn detect_ffmpeg(custom_path: Option<String>) -> ToolInfo {
    use std::process::Command;

    // Try custom path first, then system PATH
    let ffmpeg_cmd = custom_path.as_deref().unwrap_or("ffmpeg");

    let result = Command::new(ffmpeg_cmd).arg("-version").output();

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout
                .lines()
                .next()
                .and_then(|line| {
                    // Parse version from "ffmpeg version X.Y.Z ..."
                    line.split_whitespace()
                        .nth(2)
                        .map(|v| v.to_string())
                })
                .or_else(|| Some("unknown".to_string()));

            // Get the actual path using `which` (Unix) or `where` (Windows)
            let path = if custom_path.is_some() {
                custom_path
            } else {
                #[cfg(unix)]
                {
                    Command::new("which")
                        .arg("ffmpeg")
                        .output()
                        .ok()
                        .filter(|o| o.status.success())
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                }
                #[cfg(windows)]
                {
                    Command::new("where")
                        .arg("ffmpeg")
                        .output()
                        .ok()
                        .filter(|o| o.status.success())
                        .map(|o| {
                            String::from_utf8_lossy(&o.stdout)
                                .lines()
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string()
                        })
                }
            };

            ToolInfo {
                detected: true,
                path,
                version,
            }
        }
        _ => ToolInfo {
            detected: false,
            path: None,
            version: None,
        },
    }
}

/// Detect VLC installation
#[tauri::command]
pub fn detect_vlc(custom_path: Option<String>) -> ToolInfo {
    // Platform-specific VLC binary name and common paths
    #[cfg(target_os = "macos")]
    let default_paths = [
        "/Applications/VLC.app/Contents/MacOS/VLC",
        "/usr/local/bin/vlc",
    ];

    #[cfg(target_os = "windows")]
    let default_paths = [
        r"C:\Program Files\VideoLAN\VLC\vlc.exe",
        r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe",
    ];

    #[cfg(target_os = "linux")]
    let default_paths = ["/usr/bin/vlc", "/usr/local/bin/vlc", "/snap/bin/vlc"];

    // Try custom path first
    if let Some(ref path) = custom_path {
        if let Some(info) = try_vlc_at_path(path) {
            return info;
        }
    }

    // Try default paths
    for path in default_paths {
        if let Some(info) = try_vlc_at_path(path) {
            return info;
        }
    }

    // Try system PATH
    let vlc_cmd = if cfg!(windows) { "vlc.exe" } else { "vlc" };
    if let Some(info) = try_vlc_at_path(vlc_cmd) {
        return info;
    }

    ToolInfo {
        detected: false,
        path: None,
        version: None,
    }
}

/// Open a URL in VLC
#[tauri::command]
pub fn open_in_vlc(url: String, vlc_path: Option<String>) -> Result<(), String> {
    use std::process::Command;

    // Get VLC path
    let vlc_info = detect_vlc(vlc_path);
    if !vlc_info.detected {
        return Err("VLC is not installed".to_string());
    }

    let vlc_path = vlc_info.path.ok_or("VLC path not found")?;

    // Launch VLC with the URL
    #[cfg(target_os = "macos")]
    {
        // On macOS, use 'open' with the .app bundle or app name
        // If vlc_path is the binary inside .app, extract the .app path
        let app_path = if vlc_path.contains(".app/Contents/MacOS/") {
            vlc_path
                .split(".app/Contents/MacOS/")
                .next()
                .map(|s| format!("{}.app", s))
                .unwrap_or_else(|| "VLC".to_string())
        } else {
            "VLC".to_string()
        };

        Command::new("open")
            .args(["-a", &app_path, &url])
            .spawn()
            .map_err(|e| format!("Failed to launch VLC: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new(&vlc_path)
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to launch VLC: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new(&vlc_path)
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to launch VLC: {}", e))?;
    }

    Ok(())
}

fn try_vlc_at_path(path: &str) -> Option<ToolInfo> {
    use std::process::Command;

    let result = Command::new(path).arg("--version").output();

    match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout
                .lines()
                .next()
                .and_then(|line| {
                    // Parse version from "VLC media player X.Y.Z ..."
                    line.split_whitespace()
                        .find(|word| word.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                        .map(|v| v.to_string())
                });

            // Resolve the actual path
            let resolved_path = if path.contains('/') || path.contains('\\') {
                Some(path.to_string())
            } else {
                #[cfg(unix)]
                {
                    Command::new("which")
                        .arg(path)
                        .output()
                        .ok()
                        .filter(|o| o.status.success())
                        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                }
                #[cfg(windows)]
                {
                    Command::new("where")
                        .arg(path)
                        .output()
                        .ok()
                        .filter(|o| o.status.success())
                        .map(|o| {
                            String::from_utf8_lossy(&o.stdout)
                                .lines()
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string()
                        })
                }
            };

            Some(ToolInfo {
                detected: true,
                path: resolved_path,
                version,
            })
        }
        _ => None,
    }
}
