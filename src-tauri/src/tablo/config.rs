//! Configuration and credential storage for Tablo
//!
//! Credentials are stored securely using the OS keychain:
//! - macOS: Login Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service (GNOME Keyring, KWallet, etc.)
//!
//! Non-sensitive config (last device IP) is stored in the app data directory.

use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{TabloError, TabloResult};

use super::types::Credentials;

const CONFIG_FILENAME: &str = "tablo_config.json";
const KEYRING_SERVICE: &str = "com.opentabtv.app";
const KEYRING_EMAIL_KEY: &str = "tablo_email";
const KEYRING_PASSWORD_KEY: &str = "tablo_password";

/// Application configuration stored on disk (non-sensitive data only)
/// Credentials are stored securely in the OS keychain, not here.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabloConfig {
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

    /// Save credentials securely to OS keychain
    pub fn save_credentials(email: &str, password: &str) -> TabloResult<()> {
        // Store email in keychain
        let email_entry = Entry::new(KEYRING_SERVICE, KEYRING_EMAIL_KEY)
            .map_err(|e| TabloError::ConfigError(format!("Failed to access keychain: {}", e)))?;
        email_entry
            .set_password(email)
            .map_err(|e| TabloError::ConfigError(format!("Failed to save email to keychain: {}", e)))?;

        // Store password in keychain
        let password_entry = Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_KEY)
            .map_err(|e| TabloError::ConfigError(format!("Failed to access keychain: {}", e)))?;
        password_entry
            .set_password(password)
            .map_err(|e| TabloError::ConfigError(format!("Failed to save password to keychain: {}", e)))?;

        tracing::info!("Credentials saved securely to OS keychain");
        Ok(())
    }

    /// Load credentials from OS keychain
    pub fn load_credentials() -> TabloResult<Option<Credentials>> {
        let email_entry = match Entry::new(KEYRING_SERVICE, KEYRING_EMAIL_KEY) {
            Ok(entry) => entry,
            Err(_) => return Ok(None),
        };

        let password_entry = match Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_KEY) {
            Ok(entry) => entry,
            Err(_) => return Ok(None),
        };

        let email = match email_entry.get_password() {
            Ok(e) => e,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => {
                tracing::warn!("Failed to read email from keychain: {}", e);
                return Ok(None);
            }
        };

        let password = match password_entry.get_password() {
            Ok(p) => p,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => {
                tracing::warn!("Failed to read password from keychain: {}", e);
                return Ok(None);
            }
        };

        Ok(Some(Credentials { email, password }))
    }

    /// Clear credentials from OS keychain
    pub fn clear_credentials() -> TabloResult<()> {
        // Delete email from keychain (ignore errors if not found)
        if let Ok(email_entry) = Entry::new(KEYRING_SERVICE, KEYRING_EMAIL_KEY) {
            let _ = email_entry.delete_credential();
        }

        // Delete password from keychain (ignore errors if not found)
        if let Ok(password_entry) = Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_KEY) {
            let _ = password_entry.delete_credential();
        }

        tracing::info!("Credentials cleared from OS keychain");
        Ok(())
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

    /// Check if credentials are saved in OS keychain
    pub fn has_credentials() -> bool {
        // Check if both email and password exist in keychain
        let email_exists = Entry::new(KEYRING_SERVICE, KEYRING_EMAIL_KEY)
            .and_then(|e| e.get_password())
            .is_ok();

        let password_exists = Entry::new(KEYRING_SERVICE, KEYRING_PASSWORD_KEY)
            .and_then(|e| e.get_password())
            .is_ok();

        email_exists && password_exists
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TabloConfig::default();
        assert!(config.last_device_ip.is_none());
        assert!(config.last_device_id.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = TabloConfig {
            last_device_ip: Some("192.168.1.100".to_string()),
            last_device_id: Some("device-123".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: TabloConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.last_device_ip.unwrap(), "192.168.1.100");
        assert_eq!(parsed.last_device_id.unwrap(), "device-123");
    }

    // Note: Keychain credential tests require a running keychain service
    // and are not suitable for automated unit tests. Manual testing or
    // integration tests with mock keychain should be used instead.
}
