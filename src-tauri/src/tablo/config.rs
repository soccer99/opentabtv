//! Configuration and credential storage for Tablo
//!
//! Stores user credentials in the app data directory for auto-login.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{TabloError, TabloResult};

use super::types::Credentials;

const CONFIG_FILENAME: &str = "tablo_config.json";

/// Application configuration stored on disk
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabloConfig {
    /// Saved credentials for 4th Gen cloud login
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Credentials>,
    /// Last connected device IP for quick reconnect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_device_ip: Option<String>,
    /// Last connected device ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_device_id: Option<String>,
}

impl TabloConfig {
    /// Get the config file path in app data directory
    fn config_path() -> TabloResult<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            TabloError::ConfigError("Could not determine config directory".into())
        })?;

        let app_config_dir = config_dir.join("tablo");

        // Create directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir).map_err(|e| {
                TabloError::ConfigError(format!("Failed to create config dir: {}", e))
            })?;
        }

        Ok(app_config_dir.join(CONFIG_FILENAME))
    }

    /// Load config from file
    pub fn load() -> TabloResult<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| TabloError::ConfigError(format!("Failed to read config: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| TabloError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    /// Save config to file
    pub fn save(&self) -> TabloResult<()> {
        let path = Self::config_path()?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| TabloError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, content)
            .map_err(|e| TabloError::ConfigError(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Save credentials
    pub fn save_credentials(email: &str, password: &str) -> TabloResult<()> {
        let mut config = Self::load().unwrap_or_default();
        config.credentials = Some(Credentials {
            email: email.to_string(),
            password: password.to_string(),
        });
        config.save()
    }

    /// Load credentials
    pub fn load_credentials() -> TabloResult<Option<Credentials>> {
        let config = Self::load()?;
        Ok(config.credentials)
    }

    /// Clear credentials
    pub fn clear_credentials() -> TabloResult<()> {
        let mut config = Self::load().unwrap_or_default();
        config.credentials = None;
        config.save()
    }

    /// Save last connected device
    pub fn save_last_device(ip: &str, id: &str) -> TabloResult<()> {
        let mut config = Self::load().unwrap_or_default();
        config.last_device_ip = Some(ip.to_string());
        config.last_device_id = Some(id.to_string());
        config.save()
    }

    /// Get last connected device info
    pub fn get_last_device() -> TabloResult<Option<(String, String)>> {
        let config = Self::load()?;
        match (config.last_device_ip, config.last_device_id) {
            (Some(ip), Some(id)) => Ok(Some((ip, id))),
            _ => Ok(None),
        }
    }

    /// Clear last device
    #[allow(dead_code)]
    pub fn clear_last_device() -> TabloResult<()> {
        let mut config = Self::load().unwrap_or_default();
        config.last_device_ip = None;
        config.last_device_id = None;
        config.save()
    }

    /// Check if credentials are saved
    pub fn has_credentials() -> bool {
        Self::load()
            .map(|c| c.credentials.is_some())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TabloConfig::default();
        assert!(config.credentials.is_none());
        assert!(config.last_device_ip.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = TabloConfig {
            credentials: Some(Credentials {
                email: "test@example.com".to_string(),
                password: "secret".to_string(),
            }),
            last_device_ip: Some("192.168.1.100".to_string()),
            last_device_id: Some("device-123".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: TabloConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.credentials.unwrap().email, "test@example.com");
        assert_eq!(parsed.last_device_ip.unwrap(), "192.168.1.100");
    }
}
