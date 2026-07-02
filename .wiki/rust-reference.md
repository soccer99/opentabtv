# Rust Reference Implementation

Translated from tablo-web (Python) and tablo-tools-electron (TypeScript).

## Core Types

```rust
// src-tauri/src/tablo/types.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Tablo device discovered via cloud or UDP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabloDevice {
    pub sid: String,
    pub name: String,
    pub private_ip: String,
    pub local_url: String,
    pub server_id: String,
    pub server_version: Option<String>,
    pub board: Option<String>,
    pub model: Option<DeviceModel>,

    // Auth tokens (4th gen)
    pub account_token: Option<String>,
    pub lighthouse_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceModel {
    pub name: String,
    pub tuners: Option<u8>,
}

/// Server info from /server/info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub private_ip: String,
    pub server_id: String,
    pub server_version: String,
    pub timezone: String,
    pub model: DeviceModel,
}

/// Tuner status from /server/tuners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunerStatus {
    pub number: u8,
    pub in_use: bool,
    pub recording: bool,
}

/// Hard drive info from /server/harddrives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardDrive {
    pub name: String,
    pub free_mib: u64,
    pub size_mib: u64,
}

/// Channel from /guide/channels/{id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub path: String,
    pub object_id: String,
    pub channel: ChannelInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub major: u16,
    pub minor: u16,
    pub network: Option<String>,
    pub call_sign: String,
    pub resolution: Option<String>,
    pub channel_identifier: Option<String>,
    pub logos: Option<Vec<ChannelLogo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelLogo {
    pub kind: String,
    pub url: String,
}

/// Recording airing from /recordings/airings/{id}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub path: String,
    pub object_id: u64,
    pub airing_details: AiringDetails,
    pub video_details: Option<VideoDetails>,
    pub user_info: Option<UserInfo>,
    pub episode: Option<EpisodeInfo>,
    pub event: Option<EventInfo>,
    pub series_path: Option<String>,
    pub movie_path: Option<String>,
    pub sport_path: Option<String>,
    pub snapshot_image: Option<SnapshotImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiringDetails {
    pub datetime: String,
    pub show_title: String,
    pub duration: u32,
    pub channel_path: Option<String>,
    pub genres: Option<Vec<String>>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDetails {
    pub duration: u32,
    pub state: VideoState,
    pub size: u64,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub clean: bool,
    pub audio: Option<String>,
    pub comskip: Option<ComskipInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoState {
    Finished,
    Recording,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComskipInfo {
    pub state: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub watched: bool,
    pub protected: bool,
    pub position: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub season_number: u16,
    pub number: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotImage {
    pub image_id: u64,
}

/// Watch session response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchSession {
    pub playlist_url: String,
    pub token: Option<String>,
    pub expires: Option<String>,
    pub keepalive: Option<u32>,
    pub bif_url_sd: Option<String>,
    pub bif_url_hd: Option<String>,
    pub cutlist: Option<Vec<CutlistEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutlistEntry {
    pub start: f64,
    pub end: f64,
}

/// Schedule recording request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub schedule: ScheduleRule,
    pub schedule_rule: String,
    pub config: ScheduleConfig,
    pub program: ProgramInfo,
    pub show_counts: ShowCounts,
    pub keep: KeepRule,
    pub recordings_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRule {
    pub rule: String,
    pub offsets: ScheduleOffsets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleOffsets {
    pub start: i32,
    pub end: i32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub title: String,
    pub channel_path: String,
    pub duration: u32,
    pub kind: String,
    pub once: Option<OnceSchedule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnceSchedule {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowCounts {
    pub airing_count: u32,
    pub conflicted_count: u32,
    pub scheduled_count: u32,
    pub unwatched_count: Option<u32>,
    pub protected_count: Option<u32>,
    pub failed_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeepRule {
    pub rule: String,
    pub count: Option<u32>,
}

/// Settings from /settings/info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSettings {
    pub led: Option<String>,
    pub recording_quality: String,
    pub live_tv_quality: Option<String>,
    pub commercial_skip: Option<String>,
    pub auto_delete: Option<bool>,
}

/// Subscription from /account/subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub title: String,
    pub status: String,
    pub description: Option<String>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabloError {
    pub error: TabloErrorInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabloErrorInfo {
    pub code: String,
    pub description: String,
    pub details: Option<String>,
}
```

## API Client

```rust
// src-tauri/src/tablo/client.rs

use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::sync::Semaphore;
use std::sync::Arc;

use super::types::*;

const API_PORT: u16 = 8885;
const BATCH_LIMIT: usize = 50;

pub struct TabloClient {
    http: Client,
    base_url: String,
    semaphore: Arc<Semaphore>,
}

impl TabloClient {
    pub fn new(device_ip: &str) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url: format!("http://{}:{}", device_ip, API_PORT),
            semaphore: Arc::new(Semaphore::new(30)), // Concurrent request limit
        }
    }

    /// GET request to Tablo device
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, TabloClientError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = format!("{}{}", self.base_url, path);

        let resp = self.http.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            if let Ok(err) = resp.json::<TabloError>().await {
                return Err(TabloClientError::Api(err.error.description));
            }
            return Err(TabloClientError::Http(status));
        }

        Ok(resp.json().await?)
    }

    /// POST request to Tablo device
    async fn post<T, B>(&self, path: &str, body: Option<B>) -> Result<T, TabloClientError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = format!("{}{}", self.base_url, path);

        let mut req = self.http.post(&url);
        if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            if let Ok(err) = resp.json::<TabloError>().await {
                return Err(TabloClientError::Api(err.error.description));
            }
            return Err(TabloClientError::Http(status));
        }

        Ok(resp.json().await?)
    }

    /// PATCH request to Tablo device
    async fn patch<T, B>(&self, path: &str, body: B) -> Result<T, TabloClientError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = format!("{}{}", self.base_url, path);

        let resp = self.http.patch(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            if let Ok(err) = resp.json::<TabloError>().await {
                return Err(TabloClientError::Api(err.error.description));
            }
            return Err(TabloClientError::Http(status));
        }

        Ok(resp.json().await?)
    }

    /// DELETE request to Tablo device
    async fn delete(&self, path: &str) -> Result<(), TabloClientError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let url = format!("{}{}", self.base_url, path);

        let resp = self.http.delete(&url).send().await?;

        if !resp.status().is_success() && resp.status() != StatusCode::NO_CONTENT {
            let status = resp.status();
            if let Ok(err) = resp.json::<TabloError>().await {
                return Err(TabloClientError::Api(err.error.description));
            }
            return Err(TabloClientError::Http(status));
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────
    // Server Info
    // ─────────────────────────────────────────────────────────────────────

    pub async fn get_server_info(&self) -> Result<ServerInfo, TabloClientError> {
        self.get("/server/info").await
    }

    pub async fn get_tuners(&self) -> Result<Vec<TunerStatus>, TabloClientError> {
        self.get("/server/tuners").await
    }

    pub async fn get_harddrives(&self) -> Result<Vec<HardDrive>, TabloClientError> {
        self.get("/server/harddrives").await
    }

    pub async fn get_settings(&self) -> Result<DeviceSettings, TabloClientError> {
        self.get("/settings/info").await
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<Subscription>, TabloClientError> {
        self.get("/account/subscription").await
    }

    // ─────────────────────────────────────────────────────────────────────
    // Channels
    // ─────────────────────────────────────────────────────────────────────

    /// Get list of channel paths
    pub async fn get_channel_paths(&self) -> Result<Vec<String>, TabloClientError> {
        self.get("/guide/channels").await
    }

    /// Get channel details by path
    pub async fn get_channel(&self, path: &str) -> Result<Channel, TabloClientError> {
        self.get(path).await
    }

    /// Batch fetch channel details (max 50 at a time)
    pub async fn batch_get_channels(&self, paths: &[String]) -> Result<Vec<Channel>, TabloClientError> {
        let mut all_channels = Vec::new();

        for chunk in paths.chunks(BATCH_LIMIT) {
            let batch: std::collections::HashMap<String, Option<Channel>> =
                self.post("/batch", Some(chunk)).await?;

            for (_path, channel) in batch {
                if let Some(c) = channel {
                    all_channels.push(c);
                }
            }
        }

        Ok(all_channels)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Recordings
    // ─────────────────────────────────────────────────────────────────────

    /// Get list of recording paths
    pub async fn get_recording_paths(&self) -> Result<Vec<String>, TabloClientError> {
        self.get("/recordings/airings").await
    }

    /// Get recording details by path
    pub async fn get_recording(&self, path: &str) -> Result<Recording, TabloClientError> {
        self.get(path).await
    }

    /// Batch fetch recording details
    pub async fn batch_get_recordings(&self, paths: &[String]) -> Result<Vec<Recording>, TabloClientError> {
        let mut all_recordings = Vec::new();

        for chunk in paths.chunks(BATCH_LIMIT) {
            let batch: std::collections::HashMap<String, Option<Recording>> =
                self.post("/batch", Some(chunk)).await?;

            for (_path, recording) in batch {
                if let Some(r) = recording {
                    all_recordings.push(r);
                }
            }
        }

        Ok(all_recordings)
    }

    /// Delete a recording
    pub async fn delete_recording(&self, path: &str) -> Result<(), TabloClientError> {
        self.delete(path).await
    }

    /// Mark recording as watched/unwatched
    pub async fn set_watched(&self, path: &str, watched: bool) -> Result<(), TabloClientError> {
        self.patch::<serde_json::Value, _>(path, serde_json::json!({ "watched": watched }))
            .await?;
        Ok(())
    }

    /// Mark recording as protected/unprotected
    pub async fn set_protected(&self, path: &str, protected: bool) -> Result<(), TabloClientError> {
        self.patch::<serde_json::Value, _>(path, serde_json::json!({ "protected": protected }))
            .await?;
        Ok(())
    }

    /// Set playback position
    pub async fn set_position(&self, path: &str, position: u32) -> Result<(), TabloClientError> {
        self.patch::<serde_json::Value, _>(path, serde_json::json!({ "position": position }))
            .await?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────
    // Streaming
    // ─────────────────────────────────────────────────────────────────────

    /// Start watch session for channel or recording
    pub async fn watch(&self, path: &str) -> Result<WatchSession, TabloClientError> {
        self.post(&format!("{}/watch", path), None::<()>).await
    }

    // ─────────────────────────────────────────────────────────────────────
    // Guide / Airings
    // ─────────────────────────────────────────────────────────────────────

    /// Get list of airing paths (scheduled/available programs)
    pub async fn get_airing_paths(&self) -> Result<Vec<String>, TabloClientError> {
        self.get("/guide/airings").await
    }

    /// Schedule a recording for an airing
    pub async fn schedule_airing(&self, path: &str) -> Result<serde_json::Value, TabloClientError> {
        self.patch(path, serde_json::json!({ "scheduled": true })).await
    }

    /// Cancel a scheduled recording
    pub async fn unschedule_airing(&self, path: &str) -> Result<serde_json::Value, TabloClientError> {
        self.patch(path, serde_json::json!({ "scheduled": false })).await
    }

    /// Schedule a manual recording
    pub async fn schedule_manual(
        &self,
        title: &str,
        channel_path: &str,
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        duration_minutes: u32,
        timezone: &str,
    ) -> Result<serde_json::Value, TabloClientError> {
        let request = ScheduleRequest {
            schedule: ScheduleRule {
                rule: "all".to_string(),
                offsets: ScheduleOffsets {
                    start: 0,
                    end: 0,
                    source: "none".to_string(),
                },
            },
            schedule_rule: "all".to_string(),
            config: ScheduleConfig {
                title: title.to_string(),
                channel_path: channel_path.to_string(),
                duration: duration_minutes * 60,
                kind: "once".to_string(),
                once: Some(OnceSchedule {
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    timezone: timezone.to_string(),
                }),
            },
            program: ProgramInfo {
                title: title.to_string(),
            },
            show_counts: ShowCounts {
                airing_count: 1,
                conflicted_count: 0,
                scheduled_count: 1,
                unwatched_count: None,
                protected_count: None,
                failed_count: None,
            },
            keep: KeepRule {
                rule: "none".to_string(),
                count: None,
            },
            recordings_path: None,
        };

        self.post("/guide/programs", Some(request)).await
    }

    // ─────────────────────────────────────────────────────────────────────
    // Series Scheduling
    // ─────────────────────────────────────────────────────────────────────

    /// Schedule all episodes of a series
    pub async fn schedule_series_all(&self, series_path: &str) -> Result<serde_json::Value, TabloClientError> {
        self.patch(series_path, serde_json::json!({ "schedule": "all" })).await
    }

    /// Schedule only new episodes of a series
    pub async fn schedule_series_new(&self, series_path: &str) -> Result<serde_json::Value, TabloClientError> {
        self.patch(series_path, serde_json::json!({ "schedule": "new" })).await
    }

    /// Cancel series recording
    pub async fn unschedule_series(&self, series_path: &str) -> Result<serde_json::Value, TabloClientError> {
        self.patch(series_path, serde_json::json!({ "schedule": "none" })).await
    }

    /// Set keep rule for series (1, 3, 5, 10, 20 or all)
    pub async fn set_series_keep(&self, series_path: &str, count: Option<u32>) -> Result<serde_json::Value, TabloClientError> {
        let keep = match count {
            Some(n) => serde_json::json!({ "keep": { "rule": "count", "count": n } }),
            None => serde_json::json!({ "keep": { "rule": "all", "count": null } }),
        };
        self.patch(series_path, keep).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TabloClientError {
    #[error("HTTP error: {0}")]
    Http(StatusCode),

    #[error("API error: {0}")]
    Api(String),

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
}
```

## Device Discovery

```rust
// src-tauri/src/tablo/discovery.rs

use std::net::UdpSocket;
use std::time::Duration;
use tokio::time::timeout;

const DISCOVERY_PORT: u16 = 8881;
const CLOUD_API: &str = "https://api.tablotv.com/assocserver/getipinfo/";

/// Discover Tablo devices via cloud API
pub async fn discover_cloud() -> Result<Vec<TabloDevice>, DiscoveryError> {
    let client = reqwest::Client::new();
    let resp = client.get(CLOUD_API).send().await?;

    if !resp.status().is_success() {
        return Err(DiscoveryError::CloudApi(resp.status().to_string()));
    }

    let devices: Vec<CloudDevice> = resp.json().await?;

    Ok(devices
        .into_iter()
        .map(|d| TabloDevice {
            sid: d.server_id.clone(),
            name: d.name,
            private_ip: d.private_ip,
            local_url: format!("http://{}:{}", d.private_ip, 8885),
            server_id: d.server_id,
            server_version: None,
            board: None,
            model: None,
            account_token: None,
            lighthouse_token: None,
        })
        .collect())
}

/// Discover Tablo devices via UDP broadcast (legacy/fallback)
pub async fn discover_udp() -> Result<Vec<TabloDevice>, DiscoveryError> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(3)))?;

    // Send discovery packet
    socket.send_to(b"", ("255.255.255.255", DISCOVERY_PORT))?;

    let mut devices = Vec::new();
    let mut buf = [0u8; 1024];

    // Collect responses for 3 seconds
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Ok(info) = serde_json::from_slice::<UdpDiscoveryResponse>(&buf[..len]) {
                    devices.push(TabloDevice {
                        sid: info.server_id.clone(),
                        name: info.name,
                        private_ip: addr.ip().to_string(),
                        local_url: format!("http://{}:{}", addr.ip(), 8885),
                        server_id: info.server_id,
                        server_version: Some(info.server_version),
                        board: info.board,
                        model: None,
                        account_token: None,
                        lighthouse_token: None,
                    });
                }
            }
            Err(_) => break,
        }
    }

    Ok(devices)
}

/// Check if device is reachable
pub async fn check_connection(ip: &str) -> bool {
    let addr = format!("{}:{}", ip, 8885);
    match timeout(Duration::from_millis(750), tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

#[derive(Debug, serde::Deserialize)]
struct CloudDevice {
    server_id: String,
    name: String,
    private_ip: String,
}

#[derive(Debug, serde::Deserialize)]
struct UdpDiscoveryResponse {
    server_id: String,
    name: String,
    server_version: String,
    board: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Cloud API error: {0}")]
    CloudApi(String),

    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}
```

## HLS Proxy

```rust
// src-tauri/src/tablo/hls.rs

use reqwest::Client;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct HlsProxyState {
    sessions: RwLock<HashMap<String, StreamSession>>,
    http: Client,
}

pub struct StreamSession {
    pub playlist_url: String,
    pub base_url: String,
}

impl HlsProxyState {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            http: Client::new(),
        }
    }

    /// Start a new stream session
    pub async fn start_session(&self, playlist_url: &str) -> String {
        let session_id = Uuid::new_v4().to_string().replace("-", "");

        // Extract base URL from playlist URL
        let base_url = playlist_url
            .rsplit_once('/')
            .map(|(base, _)| base.to_string())
            .unwrap_or_else(|| playlist_url.to_string());

        let session = StreamSession {
            playlist_url: playlist_url.to_string(),
            base_url,
        };

        self.sessions.write().await.insert(session_id.clone(), session);
        session_id
    }

    /// Stop a stream session
    pub async fn stop_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }

    /// Get session info
    pub async fn get_session(&self, session_id: &str) -> Option<(String, String)> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| (s.playlist_url.clone(), s.base_url.clone()))
    }

    /// Rewrite HLS manifest to use proxy URLs
    pub fn rewrite_manifest(&self, manifest: &str, session_id: &str, base_url: &str) -> String {
        manifest
            .lines()
            .map(|line| {
                let trimmed = line.trim();

                // Skip comments and empty lines
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    return line.to_string();
                }

                // Rewrite segment URLs
                let full_url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    trimmed.to_string()
                } else {
                    format!("{}/{}", base_url, trimmed)
                };

                // Extract path from URL
                if let Ok(url) = url::Url::parse(&full_url) {
                    let path = url.path().trim_start_matches('/');
                    let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                    format!("/api/hls/{}/{}{}", session_id, path, query)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

## Tauri Commands

```rust
// src-tauri/src/commands/tablo.rs

use tauri::State;
use crate::tablo::{TabloClient, TabloDevice, ServerInfo, Recording, Channel, WatchSession};

#[tauri::command]
pub async fn discover_devices() -> Result<Vec<TabloDevice>, String> {
    // Try cloud discovery first, fall back to UDP
    match crate::tablo::discovery::discover_cloud().await {
        Ok(devices) if !devices.is_empty() => Ok(devices),
        _ => crate::tablo::discovery::discover_udp()
            .await
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub async fn get_server_info(device_ip: String) -> Result<ServerInfo, String> {
    let client = TabloClient::new(&device_ip);
    client.get_server_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_channels(device_ip: String) -> Result<Vec<Channel>, String> {
    let client = TabloClient::new(&device_ip);
    let paths = client.get_channel_paths().await.map_err(|e| e.to_string())?;
    client.batch_get_channels(&paths).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recordings(device_ip: String) -> Result<Vec<Recording>, String> {
    let client = TabloClient::new(&device_ip);
    let paths = client.get_recording_paths().await.map_err(|e| e.to_string())?;
    client.batch_get_recordings(&paths).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watch_channel(device_ip: String, channel_path: String) -> Result<WatchSession, String> {
    let client = TabloClient::new(&device_ip);
    client.watch(&channel_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watch_recording(device_ip: String, recording_path: String) -> Result<WatchSession, String> {
    let client = TabloClient::new(&device_ip);
    client.watch(&recording_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_recording(device_ip: String, recording_path: String) -> Result<(), String> {
    let client = TabloClient::new(&device_ip);
    client.delete_recording(&recording_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_recording_watched(device_ip: String, path: String, watched: bool) -> Result<(), String> {
    let client = TabloClient::new(&device_ip);
    client.set_watched(&path, watched).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_recording_protected(device_ip: String, path: String, protected: bool) -> Result<(), String> {
    let client = TabloClient::new(&device_ip);
    client.set_protected(&path, protected).await.map_err(|e| e.to_string())
}
```
