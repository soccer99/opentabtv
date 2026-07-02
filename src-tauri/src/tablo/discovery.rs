//! Tablo device discovery via UDP broadcast

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use crate::error::{TabloError, TabloResult};
use crate::tablo::types::{DeviceGeneration, TabloDevice};

/// Discovery port for Tablo devices
const DISCOVERY_PORT: u16 = 8881;

/// Discovery message
const DISCOVERY_MESSAGE: &[u8] = b"BnGrBDGz";

/// Timeout for UDP discovery
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

/// Discover Tablo devices on the local network via UDP broadcast
pub async fn discover_devices() -> TabloResult<Vec<TabloDevice>> {
    // Run blocking UDP code in a separate thread
    tokio::task::spawn_blocking(discover_devices_blocking)
        .await
        .map_err(|e| TabloError::DiscoveryFailed(e.to_string()))?
}

fn discover_devices_blocking() -> TabloResult<Vec<TabloDevice>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(DISCOVERY_TIMEOUT))?;

    // Send broadcast to discovery port
    let broadcast_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT);
    socket.send_to(DISCOVERY_MESSAGE, broadcast_addr)?;

    let mut devices = Vec::new();
    let mut buf = [0u8; 1024];

    // Collect responses
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Some(device) = parse_discovery_response(&buf[..len], addr) {
                    // Avoid duplicates
                    if !devices.iter().any(|d: &TabloDevice| d.local_ip == device.local_ip) {
                        devices.push(device);
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout, done collecting
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                break;
            }
            Err(e) => {
                tracing::warn!("Discovery recv error: {}", e);
                break;
            }
        }
    }

    Ok(devices)
}

fn parse_discovery_response(data: &[u8], addr: SocketAddr) -> Option<TabloDevice> {
    // Response format varies by Tablo generation
    // Try to parse as JSON first (4th gen), then as simple text (legacy)
    let response_str = std::str::from_utf8(data).ok()?;

    // Try JSON parsing
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_str) {
        let name = json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Tablo DVR")
            .to_string();

        let server_id = json
            .get("serverid")
            .or_else(|| json.get("server_id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let model = json.get("model").and_then(|v| v.as_str()).map(String::from);

        // UDP discovery always finds legacy devices (4th Gen use cloud discovery)
        return Some(TabloDevice {
            id: server_id
                .clone()
                .unwrap_or_else(|| format!("tablo-{}", addr.ip())),
            name,
            local_ip: addr.ip().to_string(),
            model,
            version: json
                .get("version")
                .and_then(|v| v.as_str())
                .map(String::from),
            tuners: json.get("tuners").and_then(|v| v.as_u64()).map(|n| n as u8),
            server_id,
            generation: DeviceGeneration::Legacy,
            sid: None,
            account_token: None,
            lighthouse_token: None,
        });
    }

    // Fallback: assume it's a legacy Tablo if we got a response
    Some(TabloDevice {
        id: format!("tablo-{}", addr.ip()),
        name: "Tablo DVR".to_string(),
        local_ip: addr.ip().to_string(),
        model: None,
        version: None,
        tuners: None,
        server_id: None,
        generation: DeviceGeneration::Legacy,
        sid: None,
        account_token: None,
        lighthouse_token: None,
    })
}

/// Check if a Tablo device is reachable at the given IP
pub async fn check_device_connection(ip: &str) -> TabloResult<bool> {
    let addr = format!("{}:8885", ip);
    match tokio::time::timeout(Duration::from_millis(1500), tokio::net::TcpStream::connect(&addr))
        .await
    {
        Ok(Ok(_)) => Ok(true),
        Ok(Err(_)) => Ok(false),
        Err(_) => Ok(false), // Timeout
    }
}
