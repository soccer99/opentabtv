//! Casting support for Chromecast devices
//!
//! Provides device discovery and media casting to Chromecast-compatible devices.
//!
//! # Security Note
//! This module uses `connect_without_host_verification` for Chromecast connections.
//! This is intentional because Chromecasts on local networks use self-signed certificates
//! and hostname verification would fail. The connection is still encrypted via TLS.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::TabloError;

/// Represents a castable device discovered on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastDevice {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub device_type: CastDeviceType,
    pub model: Option<String>,
}

/// Type of cast device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CastDeviceType {
    Chromecast,
}

// =============================================================================
// Helper Functions (DRY - Don't Repeat Yourself)
// =============================================================================

/// Connect to a Chromecast device. Returns a connection handle.
///
/// # Security
/// Uses TLS without hostname verification because Chromecasts use self-signed
/// certificates on local networks. The connection is still encrypted.
fn connect_to_chromecast(
    device: &CastDevice,
) -> Result<rust_cast::CastDevice<'static>, TabloError> {
    use rust_cast::CastDevice as RustCastDevice;

    // Clone device info for connection and error message
    // rust_cast requires owned strings for 'static lifetime
    let ip = device.ip.clone();
    let port = device.port;
    let name = device.name.clone();

    RustCastDevice::connect_without_host_verification(ip, port).map_err(|e| {
        TabloError::CastError(format!(
            "Failed to connect to Chromecast '{}' at {}:{}: {}",
            name, device.ip, port, e
        ))
    })
}

/// Get the first running application from the Chromecast receiver.
/// Returns an error if no application is running.
fn get_running_app(
    cast_device: &rust_cast::CastDevice,
) -> Result<rust_cast::channels::receiver::Application, TabloError> {
    let status = cast_device
        .receiver
        .get_status()
        .map_err(|e| TabloError::CastError(format!("Failed to get receiver status: {}", e)))?;

    status
        .applications
        .into_iter()
        .next()
        .ok_or_else(|| TabloError::CastError("No application running on Chromecast".to_string()))
}

/// Get the active media session from the Chromecast.
/// Returns an error if no media is playing.
fn get_media_session(
    cast_device: &rust_cast::CastDevice,
    transport_id: &str,
) -> Result<i32, TabloError> {
    let media_status = cast_device
        .media
        .get_status(transport_id, None)
        .map_err(|e| TabloError::CastError(format!("Failed to get media status: {}", e)))?;

    media_status
        .entries
        .first()
        .map(|entry| entry.media_session_id)
        .ok_or_else(|| TabloError::CastError("No active media session on Chromecast".to_string()))
}

// =============================================================================
// Public API
// =============================================================================

/// Discover Chromecast devices on the network using mDNS.
///
/// # Arguments
/// * `timeout_secs` - How long to scan for devices
///
/// # Returns
/// A list of discovered Chromecast devices, deduplicated by IP address.
pub async fn discover_chromecasts(timeout_secs: u64) -> Result<Vec<CastDevice>, TabloError> {
    use mdns_sd::{ServiceDaemon, ServiceEvent};

    let mdns = ServiceDaemon::new()
        .map_err(|e| TabloError::CastError(format!("Failed to create mDNS daemon: {}", e)))?;

    let service_type = "_googlecast._tcp.local.";
    let receiver = mdns
        .browse(service_type)
        .map_err(|e| TabloError::CastError(format!("Failed to browse mDNS: {}", e)))?;

    let mut devices = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);

    tracing::info!("Scanning for Chromecast devices ({}s timeout)...", timeout_secs);

    while std::time::Instant::now() < deadline {
        match receiver.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => {
                if let ServiceEvent::ServiceResolved(info) = event {
                    // Get friendly name from TXT record "fn" (friendly name)
                    let name = info
                        .get_property_val_str("fn")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            info.get_fullname()
                                .split('.')
                                .next()
                                .unwrap_or("Unknown Chromecast")
                                .to_string()
                        });

                    // Get model info from TXT record "md" (model description)
                    let model = info.get_property_val_str("md").map(|s| s.to_string());

                    if let Some(addr) = info.get_addresses().iter().next() {
                        let device = CastDevice {
                            id: info.get_fullname().to_string(),
                            name,
                            ip: addr.to_string(),
                            port: info.get_port(),
                            device_type: CastDeviceType::Chromecast,
                            model,
                        };

                        tracing::debug!(
                            "Found Chromecast: {} ({}) at {}:{}",
                            device.name,
                            device.model.as_deref().unwrap_or("unknown model"),
                            device.ip,
                            device.port
                        );
                        devices.push(device);
                    }
                }
            }
            Err(flume::RecvTimeoutError::Timeout) => continue,
            Err(flume::RecvTimeoutError::Disconnected) => {
                tracing::warn!("mDNS receiver disconnected unexpectedly");
                break;
            }
        }
    }

    // Stop browsing and shutdown daemon
    if let Err(e) = mdns.stop_browse(service_type) {
        tracing::warn!("Failed to stop mDNS browse: {}", e);
    }
    if let Err(e) = mdns.shutdown() {
        tracing::warn!("Failed to shutdown mDNS daemon: {}", e);
    }

    // Deduplicate by IP (same device might be discovered multiple times)
    devices.sort_by(|a, b| a.ip.cmp(&b.ip));
    devices.dedup_by(|a, b| a.ip == b.ip);

    tracing::info!("Discovered {} Chromecast device(s)", devices.len());
    Ok(devices)
}

/// Cast an HLS stream to a Chromecast device.
///
/// # Arguments
/// * `device` - The target Chromecast device
/// * `media_url` - The HLS playlist URL to cast
/// * `title` - Optional title to display (currently unused by DefaultMediaReceiver)
pub fn cast_to_chromecast(
    device: &CastDevice,
    media_url: &str,
    title: Option<&str>,
) -> Result<(), TabloError> {
    use rust_cast::channels::media::{Media, StreamType};
    use rust_cast::channels::receiver::CastDeviceApp;

    tracing::info!(
        "Casting to Chromecast '{}' at {}:{}",
        device.name,
        device.ip,
        device.port
    );

    let cast_device = connect_to_chromecast(device)?;

    // Launch the default media receiver app
    tracing::debug!("Launching DefaultMediaReceiver app...");
    let app = cast_device
        .receiver
        .launch_app(&CastDeviceApp::DefaultMediaReceiver)
        .map_err(|e| TabloError::CastError(format!("Failed to launch media receiver: {}", e)))?;

    // Build the media object
    // Note: DefaultMediaReceiver doesn't display custom metadata well,
    // so we skip metadata for now. Title parameter reserved for future use.
    let _ = title; // Acknowledge unused parameter

    let media = Media {
        content_id: media_url.to_string(),
        content_type: "application/x-mpegurl".to_string(),
        stream_type: StreamType::Live,
        duration: None,
        metadata: None,
    };

    // Load the media (don't log full URL as it may contain tokens)
    tracing::debug!("Loading HLS stream...");
    cast_device
        .media
        .load(&app.transport_id, &app.session_id, &media)
        .map_err(|e| TabloError::CastError(format!("Failed to load media: {}", e)))?;

    tracing::info!("Now casting to '{}'", device.name);
    Ok(())
}

/// Stop the currently running app on a Chromecast.
pub fn stop_chromecast(device: &CastDevice) -> Result<(), TabloError> {
    tracing::info!("Stopping cast on '{}'", device.name);

    let cast_device = connect_to_chromecast(device)?;
    let app = get_running_app(&cast_device)?;

    cast_device
        .receiver
        .stop_app(&app.session_id)
        .map_err(|e| TabloError::CastError(format!("Failed to stop app: {}", e)))?;

    tracing::info!("Stopped casting on '{}'", device.name);
    Ok(())
}

/// Pause media playback on a Chromecast.
pub fn pause_chromecast(device: &CastDevice) -> Result<(), TabloError> {
    tracing::debug!("Pausing playback on '{}'", device.name);

    let cast_device = connect_to_chromecast(device)?;
    let app = get_running_app(&cast_device)?;
    let media_session_id = get_media_session(&cast_device, &app.transport_id)?;

    cast_device
        .media
        .pause(&app.transport_id, media_session_id)
        .map_err(|e| TabloError::CastError(format!("Failed to pause: {}", e)))?;

    Ok(())
}

/// Resume media playback on a Chromecast.
pub fn resume_chromecast(device: &CastDevice) -> Result<(), TabloError> {
    tracing::debug!("Resuming playback on '{}'", device.name);

    let cast_device = connect_to_chromecast(device)?;
    let app = get_running_app(&cast_device)?;
    let media_session_id = get_media_session(&cast_device, &app.transport_id)?;

    cast_device
        .media
        .play(&app.transport_id, media_session_id)
        .map_err(|e| TabloError::CastError(format!("Failed to resume: {}", e)))?;

    Ok(())
}

/// Set volume on a Chromecast (0.0 to 1.0).
///
/// # Arguments
/// * `device` - The target Chromecast
/// * `volume` - Volume level from 0.0 (mute) to 1.0 (max), will be clamped
pub fn set_chromecast_volume(device: &CastDevice, volume: f32) -> Result<(), TabloError> {
    let clamped_volume = volume.clamp(0.0, 1.0);
    tracing::debug!("Setting volume to {:.0}% on '{}'", clamped_volume * 100.0, device.name);

    let cast_device = connect_to_chromecast(device)?;

    cast_device
        .receiver
        .set_volume(clamped_volume)
        .map_err(|e| TabloError::CastError(format!("Failed to set volume: {}", e)))?;

    Ok(())
}
