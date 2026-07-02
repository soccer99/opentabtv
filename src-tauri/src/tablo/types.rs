//! Tablo API data types
//!
//! Many types are defined for future features (recordings, guide, etc.)
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Device generation - Legacy (pre-4th Gen) or Gen4 (2024+)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeviceGeneration {
    /// Legacy devices (pre-4th Gen): DUAL LITE, DUAL 128GB, QUAD, etc.
    /// No authentication required for local API access.
    #[default]
    Legacy,
    /// 4th Gen devices (2024+): Requires cloud auth + HMAC-MD5 signed local requests.
    Gen4,
}

/// A discovered Tablo device
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabloDevice {
    pub id: String,
    pub name: String,
    pub local_ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuners: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    /// Device generation (legacy or 4th gen)
    #[serde(default)]
    pub generation: DeviceGeneration,
    /// 4th Gen: Device SID from cloud API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    /// 4th Gen: Account bearer token from Lighthouse login
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_token: Option<String>,
    /// 4th Gen: Device-scoped token from Lighthouse select
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lighthouse_token: Option<String>,
}

/// Tablo account information from cloud login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabloAccount {
    pub email: String,
    pub token: String,
    pub devices: Vec<CloudDevice>,
}

/// Device info from cloud API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDevice {
    pub sid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Login credentials for 4th Gen cloud auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

/// Login response from Lighthouse API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Server info response from /server/info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "totalTuners")]
    pub total_tuners: Option<u8>,
    #[serde(default, rename = "availableTuners")]
    pub available_tuners: Option<u8>,
    #[serde(default, rename = "serverID")]
    pub server_id: Option<String>,
}

/// Channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub object_id: i64,
    pub path: String,
    pub call_sign: String,
    pub major: u16,
    pub minor: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    pub resolution: Resolution,
    #[serde(default)]
    pub favourite: bool,
}

/// Video resolution
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    Hd1080,
    Hd720,
    Sd,
    #[default]
    #[serde(other)]
    Unknown,
}

/// Raw channel response from Tablo API
#[derive(Debug, Clone, Deserialize)]
pub struct RawChannelInfo {
    #[serde(default)]
    pub channel: Option<RawChannelDetails>,
    #[serde(default)]
    pub object_id: Option<i64>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawChannelDetails {
    #[serde(default)]
    pub call_sign: Option<String>,
    #[serde(default)]
    pub call_sign_src: Option<String>,
    #[serde(default)]
    pub major: Option<u16>,
    #[serde(default)]
    pub minor: Option<u16>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
}

/// Current program/airing information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentProgram {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<EpisodeInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}

/// Episode information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeInfo {
    pub number: u32,
    pub season_number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Stream session information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamSession {
    pub session_id: String,
    pub playlist_url: String,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcoded: Option<bool>,
}

/// Watch response from Tablo API
#[derive(Debug, Clone, Deserialize)]
pub struct WatchResponse {
    pub playlist_url: String,
    #[serde(default)]
    pub video_details: Option<VideoDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoDetails {
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub state: Option<String>,
}

/// Recording information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recording {
    pub id: String,
    pub path: String,
    pub title: String,
    pub recording_type: RecordingType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub duration: u32,
    pub size: u64,
    pub channel_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<EpisodeInfo>,
    pub video_details: RecordingVideoDetails,
    pub user_info: UserInfo,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingType {
    Series,
    Movie,
    Sports,
    Program,
    Manual,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingVideoDetails {
    pub width: u32,
    pub height: u32,
    pub state: RecordingState,
    pub has_comskip: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingState {
    Finished,
    Recording,
    Failed,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub watched: bool,
    pub protected: bool,
    pub position: u32,
}

/// Guide airing (program in the schedule)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuideAiring {
    pub id: String,
    pub path: String,
    pub channel_path: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: u32,
    pub airing_type: RecordingType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<EpisodeInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
}

/// Channel with its current program
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelWithProgram {
    #[serde(flatten)]
    pub channel: Channel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_program: Option<CurrentProgram>,
}
