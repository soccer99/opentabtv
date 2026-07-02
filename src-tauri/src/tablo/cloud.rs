//! Lighthouse Cloud API client for 4th Gen Tablo devices
//!
//! Handles authentication and cloud-based operations:
//! - Login with email/password
//! - Device discovery via cloud
//! - Cloud guide data for OTT channels

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{TabloError, TabloResult};

use super::types::{CloudDevice, Credentials, DeviceGeneration, TabloAccount, TabloDevice};

const LIGHTHOUSE_BASE_URL: &str = "https://lighthousetv.ewscloud.com";
const USER_AGENT: &str = "Tablo-FAST/2.0.0 (Desktop; Tauri)";

/// Lighthouse Cloud API client for 4th Gen Tablo devices
#[derive(Clone)]
pub struct LighthouseClient {
    client: Client,
    token: Option<String>,
}

impl LighthouseClient {
    /// Create HTTP client with standard configuration
    fn create_http_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client")
    }

    /// Create a new Lighthouse client without authentication
    pub fn new() -> Self {
        Self {
            client: Self::create_http_client(),
            token: None,
        }
    }

    /// Create a new Lighthouse client with an existing token
    #[allow(dead_code)]
    pub fn with_token(token: String) -> Self {
        Self {
            client: Self::create_http_client(),
            token: Some(token),
        }
    }

    /// Check if client has a valid token
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get the current token
    #[allow(dead_code)]
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Login to Lighthouse and get account token
    pub async fn login(&mut self, email: &str, password: &str) -> TabloResult<String> {
        #[derive(Serialize)]
        struct LoginBody {
            email: String,
            password: String,
        }

        #[derive(Deserialize)]
        struct LoginResponseBody {
            token: String,
        }

        let response = self
            .client
            .post(format!("{}/api/v2/login/", LIGHTHOUSE_BASE_URL))
            .json(&LoginBody {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .map_err(|e| TabloError::ConnectionFailed(format!("Login request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TabloError::AuthenticationFailed(format!(
                "Login failed ({}): {}",
                status, body
            )));
        }

        let login_response: LoginResponseBody = response
            .json()
            .await
            .map_err(|e| TabloError::ApiError(format!("Failed to parse login response: {}", e)))?;

        self.token = Some(login_response.token.clone());
        Ok(login_response.token)
    }

    /// Get list of devices associated with account
    pub async fn get_devices(&self) -> TabloResult<Vec<CloudDevice>> {
        let token = self
            .token
            .as_ref()
            .ok_or(TabloError::NotLoggedIn)?;

        #[derive(Deserialize)]
        struct AccountResponse {
            #[serde(default)]
            devices: Vec<CloudDeviceResponse>,
        }

        #[derive(Deserialize)]
        struct CloudDeviceResponse {
            sid: String,
            #[serde(default)]
            name: Option<String>,
            #[serde(default, rename = "localIP")]
            local_ip: Option<String>,
            #[serde(default)]
            model: Option<String>,
        }

        let response = self
            .client
            .get(format!("{}/api/v2/account/", LIGHTHOUSE_BASE_URL))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| TabloError::ConnectionFailed(format!("Get devices failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(TabloError::ApiError(format!(
                "Failed to get devices: {}",
                status
            )));
        }

        let account: AccountResponse = response
            .json()
            .await
            .map_err(|e| TabloError::ApiError(format!("Failed to parse account response: {}", e)))?;

        Ok(account
            .devices
            .into_iter()
            .map(|d| CloudDevice {
                sid: d.sid.clone(),
                name: d.name.unwrap_or_else(|| format!("Tablo ({})", &d.sid[..8.min(d.sid.len())])),
                local_ip: d.local_ip,
                model: d.model,
            })
            .collect())
    }

    /// Select a device and get device-scoped Lighthouse token
    pub async fn select_device(&self, device_sid: &str) -> TabloResult<String> {
        let token = self
            .token
            .as_ref()
            .ok_or(TabloError::NotLoggedIn)?;

        #[derive(Serialize)]
        struct SelectBody {
            #[serde(rename = "deviceSid")]
            device_sid: String,
        }

        #[derive(Deserialize)]
        struct SelectResponse {
            token: String,
        }

        let response = self
            .client
            .post(format!("{}/api/v2/account/select/", LIGHTHOUSE_BASE_URL))
            .bearer_auth(token)
            .json(&SelectBody {
                device_sid: device_sid.to_string(),
            })
            .send()
            .await
            .map_err(|e| TabloError::ConnectionFailed(format!("Select device failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(TabloError::ApiError(format!(
                "Failed to select device: {}",
                status
            )));
        }

        let select_response: SelectResponse = response
            .json()
            .await
            .map_err(|e| TabloError::ApiError(format!("Failed to parse select response: {}", e)))?;

        Ok(select_response.token)
    }

    /// Convert cloud device to TabloDevice
    pub fn cloud_device_to_tablo(
        cloud: CloudDevice,
        account_token: &str,
        lighthouse_token: &str,
    ) -> TabloDevice {
        TabloDevice {
            id: cloud.sid.clone(),
            name: cloud.name,
            local_ip: cloud.local_ip.unwrap_or_default(),
            model: cloud.model,
            version: None,
            tuners: None,
            server_id: None,
            generation: DeviceGeneration::Gen4,
            sid: Some(cloud.sid),
            account_token: Some(account_token.to_string()),
            lighthouse_token: Some(lighthouse_token.to_string()),
        }
    }

    /// Get cloud guide channels for a device
    #[allow(dead_code)]
    pub async fn get_cloud_channels(
        &self,
        lighthouse_token: &str,
    ) -> TabloResult<serde_json::Value> {
        let response = self
            .client
            .get(format!(
                "{}/api/v2/account/{}/guide/channels/",
                LIGHTHOUSE_BASE_URL, lighthouse_token
            ))
            .send()
            .await
            .map_err(|e| TabloError::ConnectionFailed(format!("Get channels failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(TabloError::ApiError(format!(
                "Failed to get cloud channels: {}",
                status
            )));
        }

        response
            .json()
            .await
            .map_err(|e| TabloError::ApiError(format!("Failed to parse channels: {}", e)))
    }

    /// Login and get full account information including devices
    pub async fn login_and_get_account(
        &mut self,
        credentials: &Credentials,
    ) -> TabloResult<TabloAccount> {
        let token = self.login(&credentials.email, &credentials.password).await?;
        let devices = self.get_devices().await?;

        Ok(TabloAccount {
            email: credentials.email.clone(),
            token,
            devices,
        })
    }

    /// Discover 4th Gen devices via cloud and convert to TabloDevice list
    pub async fn discover_devices(&mut self) -> TabloResult<Vec<TabloDevice>> {
        let token = self
            .token
            .as_ref()
            .ok_or(TabloError::NotLoggedIn)?
            .clone();

        let cloud_devices = self.get_devices().await?;
        let mut devices = Vec::new();

        for cloud_device in cloud_devices {
            // Get device-scoped token for each device
            match self.select_device(&cloud_device.sid).await {
                Ok(lighthouse_token) => {
                    devices.push(Self::cloud_device_to_tablo(
                        cloud_device,
                        &token,
                        &lighthouse_token,
                    ));
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to select device {}: {}",
                        cloud_device.sid,
                        e
                    );
                }
            }
        }

        Ok(devices)
    }
}

impl Default for LighthouseClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighthouse_client_creation() {
        let client = LighthouseClient::new();
        assert!(!client.is_authenticated());
    }

    #[test]
    fn test_lighthouse_client_with_token() {
        let client = LighthouseClient::with_token("test_token".to_string());
        assert!(client.is_authenticated());
        assert_eq!(client.token(), Some("test_token"));
    }
}
