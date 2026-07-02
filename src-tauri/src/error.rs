//! Error types for the Tablo application

use thiserror::Error;

/// Application-wide error type
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum TabloError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("No active device connection")]
    NoActiveDevice,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Timeout")]
    Timeout,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Not logged in")]
    NotLoggedIn,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Cast error: {0}")]
    CastError(String),
}

/// Result type alias for Tablo operations
pub type TabloResult<T> = Result<T, TabloError>;

// Implement serialization for Tauri command returns
impl serde::Serialize for TabloError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
