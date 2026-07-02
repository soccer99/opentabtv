//! Tablo API client for communicating with Tablo DVR devices

use parking_lot::RwLock;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::error::{TabloError, TabloResult};
use crate::tablo::auth::{make_device_auth, USER_AGENT};
use crate::tablo::cloud::LighthouseClient;
use crate::tablo::types::*;

/// HTTP timeout for API requests
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Tablo API port
const TABLO_PORT: u16 = 8885;

/// Shared state for the active device connection
#[derive(Default)]
pub struct TabloState {
    inner: RwLock<TabloStateInner>,
}

#[derive(Default)]
struct TabloStateInner {
    device: Option<TabloDevice>,
    client: Option<TabloClient>,
    lighthouse: Option<LighthouseClient>,
}

impl TabloState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the active device
    pub fn set_device(&self, device: TabloDevice) -> TabloResult<()> {
        let client = TabloClient::new_with_device(&device)?;
        let mut inner = self.inner.write();
        inner.device = Some(device);
        inner.client = Some(client);
        Ok(())
    }

    /// Get the active device
    pub fn device(&self) -> Option<TabloDevice> {
        self.inner.read().device.clone()
    }

    /// Execute a function with the client
    #[allow(dead_code)]
    pub fn with_client<F, T>(&self, f: F) -> TabloResult<T>
    where
        F: FnOnce(&TabloClient) -> TabloResult<T>,
    {
        let inner = self.inner.read();
        match &inner.client {
            Some(client) => f(client),
            None => Err(TabloError::NoActiveDevice),
        }
    }

    /// Set the lighthouse client for cloud authentication
    pub fn set_lighthouse(&self, client: LighthouseClient) {
        let mut inner = self.inner.write();
        inner.lighthouse = Some(client);
    }

    /// Get the lighthouse client
    pub fn lighthouse(&self) -> Option<LighthouseClient> {
        self.inner.read().lighthouse.clone()
    }

    /// Check if logged in to cloud
    pub fn is_logged_in(&self) -> bool {
        self.inner
            .read()
            .lighthouse
            .as_ref()
            .map(|l| l.is_authenticated())
            .unwrap_or(false)
    }

    /// Clear the active device
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.device = None;
        inner.client = None;
    }

    /// Clear everything including cloud auth
    pub fn clear_all(&self) {
        let mut inner = self.inner.write();
        inner.device = None;
        inner.client = None;
        inner.lighthouse = None;
    }
}

/// HTTP client for Tablo API
#[derive(Clone)]
pub struct TabloClient {
    http: Client,
    base_url: String,
    /// Device generation determines auth behavior
    generation: DeviceGeneration,
}

impl TabloClient {
    /// Create a new client for the given device IP (assumes Legacy device)
    pub fn new(ip: &str) -> TabloResult<Self> {
        Self::new_with_generation(ip, DeviceGeneration::Legacy)
    }

    /// Create a new client with a specific generation
    pub fn new_with_generation(ip: &str, generation: DeviceGeneration) -> TabloResult<Self> {
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map_err(TabloError::NetworkError)?;

        Ok(Self {
            http,
            base_url: format!("http://{}:{}", ip, TABLO_PORT),
            generation,
        })
    }

    /// Create a new client from a TabloDevice
    pub fn new_with_device(device: &TabloDevice) -> TabloResult<Self> {
        Self::new_with_generation(&device.local_ip, device.generation)
    }

    /// Get device generation
    #[allow(dead_code)]
    pub fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    /// Make a GET request to the device (conditional auth based on generation)
    pub async fn get(&self, path: &str) -> TabloResult<serde_json::Value> {
        match self.generation {
            DeviceGeneration::Legacy => self.get_unauthenticated(path).await,
            DeviceGeneration::Gen4 => self.get_authenticated(path).await,
        }
    }

    /// Make a GET request without authentication (Legacy devices)
    async fn get_unauthenticated(&self, path: &str) -> TabloResult<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);

        let resp = self.http.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(TabloError::ApiError(format!(
                "HTTP {} for {}",
                resp.status(),
                path
            )));
        }

        resp.json().await.map_err(TabloError::NetworkError)
    }

    /// Make a GET request with HMAC-MD5 authentication (4th Gen devices)
    async fn get_authenticated(&self, path: &str) -> TabloResult<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let (auth, date) = make_device_auth("GET", path, "");

        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .header("Date", date)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TabloError::ApiError(format!(
                "HTTP {} for {}",
                resp.status(),
                path
            )));
        }

        resp.json().await.map_err(TabloError::NetworkError)
    }

    /// Make a POST request to the device (conditional auth based on generation)
    pub async fn post(&self, path: &str, body: &str) -> TabloResult<serde_json::Value> {
        match self.generation {
            DeviceGeneration::Legacy => self.post_unauthenticated(path, body).await,
            DeviceGeneration::Gen4 => self.post_authenticated(path, body).await,
        }
    }

    /// Make a POST request without authentication (Legacy devices)
    async fn post_unauthenticated(&self, path: &str, body: &str) -> TabloResult<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);

        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TabloError::ApiError(format!(
                "HTTP {} for {}",
                resp.status(),
                path
            )));
        }

        resp.json().await.map_err(TabloError::NetworkError)
    }

    /// Make a POST request with HMAC-MD5 authentication (4th Gen devices)
    async fn post_authenticated(&self, path: &str, body: &str) -> TabloResult<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let (auth, date) = make_device_auth("POST", path, body);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .header("Date", date)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TabloError::ApiError(format!(
                "HTTP {} for {}",
                resp.status(),
                path
            )));
        }

        resp.json().await.map_err(TabloError::NetworkError)
    }

    /// Get server info
    pub async fn server_info(&self) -> TabloResult<ServerInfo> {
        let json = self.get("/server/info").await?;
        serde_json::from_value(json).map_err(TabloError::JsonError)
    }

    /// Get list of channel paths
    pub async fn channel_paths(&self) -> TabloResult<Vec<String>> {
        let json = self.get("/guide/channels").await?;
        serde_json::from_value(json).map_err(TabloError::JsonError)
    }

    /// Get channel details for a list of paths (batch)
    pub async fn batch_get(&self, paths: Vec<String>) -> TabloResult<Vec<serde_json::Value>> {
        // Tablo batch endpoint accepts up to 50 items
        let body = serde_json::to_string(&paths)?;
        let json = self.post("/batch", &body).await?;

        match json {
            serde_json::Value::Array(arr) => Ok(arr),
            serde_json::Value::Object(obj) => {
                // Sometimes returns an object with paths as keys
                Ok(obj.values().cloned().collect())
            }
            _ => Err(TabloError::InvalidResponse("Expected array from batch".into())),
        }
    }

    /// Get full channel list with details
    pub async fn channels(&self) -> TabloResult<Vec<Channel>> {
        let paths = self.channel_paths().await?;
        if paths.is_empty() {
            return Ok(Vec::new());
        }

        // Batch fetch in chunks of 50
        let mut all_channels = Vec::new();
        for chunk in paths.chunks(50) {
            let details = self.batch_get(chunk.to_vec()).await?;

            for (i, detail) in details.into_iter().enumerate() {
                if let Ok(raw) = serde_json::from_value::<RawChannelInfo>(detail.clone()) {
                    if let Some(ch) = raw.channel {
                        let path = chunk.get(i).cloned().unwrap_or_default();
                        let object_id = raw.object_id.unwrap_or(0);

                        let resolution = match ch.resolution.as_deref() {
                            Some("hd_1080") => Resolution::Hd1080,
                            Some("hd_720") | Some("hd") => Resolution::Hd720,
                            Some("sd") => Resolution::Sd,
                            _ => Resolution::Unknown,
                        };

                        all_channels.push(Channel {
                            id: format!("{}", object_id),
                            object_id,
                            path,
                            call_sign: ch.call_sign.or(ch.call_sign_src).unwrap_or_default(),
                            major: ch.major.unwrap_or(0),
                            minor: ch.minor.unwrap_or(0),
                            network: ch.network,
                            resolution,
                            favourite: false,
                        });
                    }
                }
            }
        }

        // Sort by channel number
        all_channels.sort_by(|a, b| {
            a.major.cmp(&b.major).then_with(|| a.minor.cmp(&b.minor))
        });

        Ok(all_channels)
    }

    /// Start watching a channel or recording
    pub async fn watch(&self, path: &str) -> TabloResult<WatchResponse> {
        let watch_path = format!("{}/watch", path.trim_end_matches('/'));
        let json = self.post(&watch_path, "").await?;
        serde_json::from_value(json).map_err(TabloError::JsonError)
    }

    /// Get recording paths
    pub async fn recording_paths(&self) -> TabloResult<Vec<String>> {
        let json = self.get("/recordings/airings").await?;
        serde_json::from_value(json).map_err(TabloError::JsonError)
    }

    /// Get full recording list with details
    pub async fn recordings(&self) -> TabloResult<Vec<Recording>> {
        let paths = self.recording_paths().await?;
        if paths.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_recordings = Vec::new();

        // Batch fetch in chunks of 50
        for chunk in paths.chunks(50) {
            let details = self.batch_get(chunk.to_vec()).await?;

            for (i, detail) in details.into_iter().enumerate() {
                if let Ok(recording) = Self::parse_recording(&detail, chunk.get(i).cloned()) {
                    all_recordings.push(recording);
                }
            }
        }

        // Sort by recorded_at (newest first)
        all_recordings.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));

        Ok(all_recordings)
    }

    /// Parse a raw recording response into a Recording
    fn parse_recording(
        value: &serde_json::Value,
        path: Option<String>,
    ) -> TabloResult<Recording> {
        use chrono::Utc;

        let obj = value
            .as_object()
            .ok_or_else(|| TabloError::InvalidResponse("Expected object".into()))?;

        // Get airing info (recordings use same structure as airings)
        let airing = obj
            .get("airing")
            .and_then(|a| a.as_object());

        // Parse recorded date
        let datetime_str = airing
            .and_then(|a| a.get("datetime"))
            .and_then(|d| d.as_str())
            .unwrap_or("");

        let recorded_at = chrono::DateTime::parse_from_rfc3339(datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let duration = airing
            .and_then(|a| a.get("duration"))
            .and_then(|d| d.as_u64())
            .unwrap_or(0) as u32;

        // Get object ID
        let object_id = obj
            .get("object_id")
            .and_then(|id| id.as_i64())
            .unwrap_or(0);

        // Get title from various fields
        let title = Self::extract_title(obj, airing.unwrap_or(&serde_json::Map::new()));

        // Get description
        let description = obj
            .get("episode")
            .and_then(|ep| ep.get("description"))
            .and_then(|d| d.as_str())
            .or_else(|| {
                obj.get("series")
                    .and_then(|s| s.get("description"))
                    .and_then(|d| d.as_str())
            })
            .or_else(|| {
                obj.get("movie")
                    .and_then(|m| m.get("plot"))
                    .and_then(|d| d.as_str())
            })
            .or_else(|| {
                obj.get("movie")
                    .and_then(|m| m.get("description"))
                    .and_then(|d| d.as_str())
            })
            .map(String::from);

        // Determine recording type
        let recording_type = if obj.contains_key("series") || obj.contains_key("episode") {
            RecordingType::Series
        } else if obj.contains_key("movie") {
            RecordingType::Movie
        } else if obj.contains_key("sport") {
            RecordingType::Sports
        } else {
            RecordingType::Program
        };

        // Get channel path
        let channel_id = airing
            .and_then(|a| a.get("channel_path"))
            .or_else(|| airing.and_then(|a| a.get("channel")))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        // Episode info
        let episode = obj.get("episode").and_then(|ep| {
            let number = ep.get("number").and_then(|n| n.as_u64()).unwrap_or(0) as u32;
            let season = ep.get("season_number").and_then(|n| n.as_u64()).unwrap_or(0) as u32;
            let title = ep.get("title").and_then(|t| t.as_str()).map(String::from);

            if number > 0 || season > 0 {
                Some(EpisodeInfo {
                    number,
                    season_number: season,
                    title,
                })
            } else {
                None
            }
        });

        // Video details
        let video = obj.get("video_details").and_then(|v| v.as_object());
        let video_details = RecordingVideoDetails {
            width: video
                .and_then(|v| v.get("width"))
                .and_then(|w| w.as_u64())
                .unwrap_or(1920) as u32,
            height: video
                .and_then(|v| v.get("height"))
                .and_then(|h| h.as_u64())
                .unwrap_or(1080) as u32,
            state: match video.and_then(|v| v.get("state")).and_then(|s| s.as_str()) {
                Some("finished") => RecordingState::Finished,
                Some("recording") => RecordingState::Recording,
                Some("failed") => RecordingState::Failed,
                _ => RecordingState::Unknown,
            },
            has_comskip: video
                .and_then(|v| v.get("comskip"))
                .and_then(|c| c.get("state"))
                .and_then(|s| s.as_str())
                == Some("ready"),
        };

        // User info
        let user = obj.get("user_info").and_then(|u| u.as_object());
        let user_info = UserInfo {
            watched: user
                .and_then(|u| u.get("watched"))
                .and_then(|w| w.as_bool())
                .unwrap_or(false),
            protected: user
                .and_then(|u| u.get("protected"))
                .and_then(|p| p.as_bool())
                .unwrap_or(false),
            position: user
                .and_then(|u| u.get("position"))
                .and_then(|p| p.as_u64())
                .unwrap_or(0) as u32,
        };

        // Size from video details
        let size = video
            .and_then(|v| v.get("size"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0);

        Ok(Recording {
            id: format!("{}", object_id),
            path: path.unwrap_or_default(),
            title,
            recording_type,
            description,
            recorded_at,
            duration,
            size,
            channel_id,
            episode,
            video_details,
            user_info,
        })
    }

    /// Get current airing paths (what's on now)
    pub async fn airing_paths(&self) -> TabloResult<Vec<String>> {
        let json = self.get("/guide/airings").await?;
        serde_json::from_value(json).map_err(TabloError::JsonError)
    }

    /// Get full guide airings with details for a time window
    pub async fn guide_airings(&self) -> TabloResult<Vec<GuideAiring>> {
        let paths = self.airing_paths().await?;
        if paths.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_airings = Vec::new();

        // Batch fetch in chunks of 50
        for chunk in paths.chunks(50) {
            let details = self.batch_get(chunk.to_vec()).await?;

            for (i, detail) in details.into_iter().enumerate() {
                if let Ok(airing) = Self::parse_airing(&detail, chunk.get(i).cloned()) {
                    all_airings.push(airing);
                }
            }
        }

        // Sort by start time
        all_airings.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(all_airings)
    }

    /// Parse a raw airing response into a GuideAiring
    fn parse_airing(
        value: &serde_json::Value,
        path: Option<String>,
    ) -> TabloResult<GuideAiring> {
        use chrono::Utc;

        let obj = value
            .as_object()
            .ok_or_else(|| TabloError::InvalidResponse("Expected object".into()))?;

        // Get airing info
        let airing = obj
            .get("airing")
            .and_then(|a| a.as_object())
            .ok_or_else(|| TabloError::InvalidResponse("Missing airing".into()))?;

        // Parse dates
        let datetime_str = airing
            .get("datetime")
            .and_then(|d| d.as_str())
            .unwrap_or("");

        let start_time = chrono::DateTime::parse_from_rfc3339(datetime_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let duration = airing
            .get("duration")
            .and_then(|d| d.as_u64())
            .unwrap_or(1800) as u32;

        let end_time = start_time + chrono::Duration::seconds(duration as i64);

        // Get channel path
        let channel_path = airing
            .get("channel_path")
            .or_else(|| airing.get("channel"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        // Get object ID
        let object_id = obj
            .get("object_id")
            .and_then(|id| id.as_i64())
            .unwrap_or(0);

        // Get title from show_path metadata or airing
        let title = Self::extract_title(obj, airing);

        // Get description
        let description = obj
            .get("episode")
            .and_then(|ep| ep.get("description"))
            .and_then(|d| d.as_str())
            .or_else(|| {
                obj.get("series")
                    .and_then(|s| s.get("description"))
                    .and_then(|d| d.as_str())
            })
            .or_else(|| {
                obj.get("movie")
                    .and_then(|m| m.get("description"))
                    .and_then(|d| d.as_str())
            })
            .map(String::from);

        // Determine airing type
        let airing_type = if obj.contains_key("series") || obj.contains_key("episode") {
            RecordingType::Series
        } else if obj.contains_key("movie") {
            RecordingType::Movie
        } else if obj.contains_key("sport") {
            RecordingType::Sports
        } else {
            RecordingType::Program
        };

        // Episode info
        let episode = obj.get("episode").and_then(|ep| {
            let number = ep.get("number").and_then(|n| n.as_u64()).unwrap_or(0) as u32;
            let season = ep.get("season_number").and_then(|n| n.as_u64()).unwrap_or(0) as u32;
            let title = ep.get("title").and_then(|t| t.as_str()).map(String::from);

            if number > 0 || season > 0 {
                Some(EpisodeInfo {
                    number,
                    season_number: season,
                    title,
                })
            } else {
                None
            }
        });

        // Genres
        let genres = obj
            .get("series")
            .and_then(|s| s.get("genres"))
            .or_else(|| obj.get("movie").and_then(|m| m.get("genres")))
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        Ok(GuideAiring {
            id: format!("{}", object_id),
            path: path.unwrap_or_default(),
            channel_path,
            title,
            description,
            start_time,
            end_time,
            duration,
            airing_type,
            episode,
            genres,
        })
    }

    /// Extract title from various fields in the airing response
    fn extract_title(obj: &serde_json::Map<String, serde_json::Value>, airing: &serde_json::Map<String, serde_json::Value>) -> String {
        // Try series title
        if let Some(series) = obj.get("series").and_then(|s| s.as_object()) {
            if let Some(title) = series.get("title").and_then(|t| t.as_str()) {
                return title.to_string();
            }
        }

        // Try movie title
        if let Some(movie) = obj.get("movie").and_then(|m| m.as_object()) {
            if let Some(title) = movie.get("title").and_then(|t| t.as_str()) {
                return title.to_string();
            }
        }

        // Try sport event
        if let Some(sport) = obj.get("sport").and_then(|s| s.as_object()) {
            if let Some(title) = sport.get("title").and_then(|t| t.as_str()) {
                return title.to_string();
            }
        }

        // Try program
        if let Some(program) = obj.get("program").and_then(|p| p.as_object()) {
            if let Some(title) = program.get("title").and_then(|t| t.as_str()) {
                return title.to_string();
            }
        }

        // Fallback to airing title or "Unknown"
        airing
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown Program")
            .to_string()
    }
}

/// Active streaming sessions
#[derive(Default)]
pub struct StreamSessions {
    sessions: RwLock<std::collections::HashMap<String, StreamSession>>,
}

impl StreamSessions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new stream session
    pub fn create(&self, channel_id: &str, playlist_url: String) -> StreamSession {
        let session = StreamSession {
            session_id: Uuid::new_v4().to_string(),
            playlist_url,
            channel_id: channel_id.to_string(),
            transcoded: Some(false),
        };

        self.sessions
            .write()
            .insert(session.session_id.clone(), session.clone());

        session
    }

    /// Get a session by ID
    #[allow(dead_code)]
    pub fn get(&self, session_id: &str) -> Option<StreamSession> {
        self.sessions.read().get(session_id).cloned()
    }

    /// Remove a session
    pub fn remove(&self, session_id: &str) {
        self.sessions.write().remove(session_id);
    }

    /// Clear all sessions
    pub fn clear(&self) {
        self.sessions.write().clear();
    }
}

/// Global application state
pub struct AppState {
    pub tablo: Arc<TabloState>,
    pub streams: Arc<StreamSessions>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tablo: Arc::new(TabloState::new()),
            streams: Arc::new(StreamSessions::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
