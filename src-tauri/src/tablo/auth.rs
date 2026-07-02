//! HMAC-MD5 authentication for 4th gen Tablo devices
//!
//! Based on tablo-api Python package: https://github.com/trevor-viljoen/tablo-api

use chrono::Utc;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};

type HmacMd5 = Hmac<Md5>;

/// HMAC signing key for device auth (from tablo-api package)
const HASH_KEY: &str = "6l8jU5N43cEilqItmT3U2M2PFM3qPziilXqau9ys";

/// Device key included in auth header (from tablo-api package)
const DEVICE_KEY: &str = "ljpg6ZkwShVv8aI12E2LP55Ep8vq1uYDPvX0DdTB";

/// Generate authentication headers for local device requests
///
/// Canonical format: {METHOD}\n{PATH}\n{BODY_MD5_HASH}\n{DATE}
/// Auth header: tablo:{DEVICE_KEY}:{SIGNATURE_HEX}
pub fn make_device_auth(method: &str, path: &str, body: &str) -> (String, String) {
    // RFC 1123 date format
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

    // MD5 hash of body (empty string if no body)
    let body_hash = if body.is_empty() {
        String::new()
    } else {
        let mut hasher = Md5::new();
        hasher.update(body.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    // Canonical request string
    let canonical = format!(
        "{}\n{}\n{}\n{}",
        method.to_uppercase(),
        path,
        body_hash,
        date
    );

    // HMAC-MD5 signature (hex encoded, not base64)
    let mut mac =
        HmacMd5::new_from_slice(HASH_KEY.as_bytes()).expect("HMAC can take key of any size");
    mac.update(canonical.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let auth_header = format!("tablo:{}:{}", DEVICE_KEY, signature);

    (auth_header, date)
}

/// User agent string for Tablo API requests
pub const USER_AGENT: &str = "Tablo-FAST/2.0.0 (Desktop; Tauri)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_generation() {
        let (auth, date) = make_device_auth("GET", "/server/info", "");
        // Format: tablo:{DEVICE_KEY}:{SIGNATURE_HEX}
        assert!(auth.starts_with("tablo:ljpg6ZkwShVv8aI12E2LP55Ep8vq1uYDPvX0DdTB:"));
        assert!(!date.is_empty());
        // Signature should be 32 hex chars (MD5 output)
        let sig = auth.split(':').last().unwrap();
        assert_eq!(sig.len(), 32);
    }

    #[test]
    fn test_auth_with_body() {
        let body = r#"{"test": "data"}"#;
        let (auth, _) = make_device_auth("POST", "/batch", body);
        assert!(auth.starts_with("tablo:ljpg6ZkwShVv8aI12E2LP55Ep8vq1uYDPvX0DdTB:"));
    }
}
