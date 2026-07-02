# Security

OpenTabTV is designed with security as a priority. This document details the security measures implemented to protect your credentials and data.

## Overview

| Aspect | Implementation |
|--------|----------------|
| Credential storage | OS Keychain (macOS/Windows/Linux) |
| Network encryption | HTTPS for cloud API, HMAC-MD5 for local devices |
| Frontend isolation | Sandboxed Tauri WebView |
| Permission model | Principle of least privilege |
| Content Security Policy | Strict CSP preventing XSS/injection |

## Credential Storage

Your Tablo account credentials (for 4th Gen devices) are stored securely using your operating system's native credential manager:

| Platform | Storage Backend | Protection |
|----------|-----------------|------------|
| macOS | Login Keychain | Hardware-backed encryption, user login required |
| Windows | Credential Manager | DPAPI encryption, per-user isolation |
| Linux | Secret Service (GNOME Keyring/KWallet) | Encrypted storage, session-locked |

### How It Works

1. When you save credentials, OpenTabTV calls the OS keychain API directly
2. The OS encrypts your credentials using platform-specific encryption
3. Credentials are protected by your OS login - only you can access them
4. The app never writes credentials to plain text files

### Technical Details

We use the [`keyring`](https://crates.io/crates/keyring) Rust crate (v3.x) which provides:
- Direct FFI bindings to native credential APIs
- No intermediate storage or caching
- Automatic cleanup on credential deletion

**Service identifier**: `com.opentabtv.app`

## Content Security Policy (CSP)

The app enforces a strict Content Security Policy to prevent XSS and code injection attacks:

```
default-src 'self';
script-src 'self' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline';
img-src 'self' data: blob: http: https:;
media-src 'self' blob: http: https:;
connect-src 'self' ipc: http://ipc.localhost https://lighthousetv.ewscloud.com http: https:;
font-src 'self' data:;
object-src 'none';
base-uri 'self';
form-action 'self';
frame-ancestors 'none';
```

### What This Means

| Directive | Protection |
|-----------|------------|
| `script-src 'self'` | Only bundled scripts execute - no external/injected scripts |
| `object-src 'none'` | No plugin content (Flash, Java, etc.) |
| `frame-ancestors 'none'` | Prevents clickjacking attacks |
| `form-action 'self'` | Forms can only submit to the app itself |

### Why Some Directives Are Permissive

- **`media-src http: https:`** - Required because Tablo devices serve HLS video streams over HTTP on your local network
- **`img-src http: https:`** - Required for channel logos and thumbnails from devices
- **`connect-src http: https:`** - Required for device API calls and video segment fetching

These permissive directives only affect media, images, and API connections - **script execution remains locked down**.

## Tauri Permission Model

OpenTabTV follows the principle of least privilege. The capabilities are defined in `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "core:default",
    "store:default",
    "opener:allow-default-urls"
  ]
}
```

### What Each Permission Does

| Permission | Purpose | What It Allows |
|------------|---------|----------------|
| `core:default` | Basic Tauri functionality | IPC communication between frontend/backend |
| `store:default` | Settings persistence | Save app preferences to disk |
| `opener:allow-default-urls` | Open URLs | Open http/https/mailto links in default browser |

### What Is NOT Permitted

- **No filesystem access** - Frontend cannot read/write files
- **No shell access** - Frontend cannot execute system commands
- **No clipboard access** - Frontend cannot access clipboard
- **No notification access** - No system notifications
- **No file dialogs** - No open/save file dialogs from frontend

All privileged operations (like running VLC or FFmpeg) are handled by the Rust backend with explicit commands.

## Data Handling

| Data Type | Storage Location | Encrypted | Persisted |
|-----------|------------------|-----------|-----------|
| Account password | OS Keychain | Yes | Yes |
| Account email | OS Keychain | Yes | Yes |
| Session tokens | Backend memory | N/A | No |
| Device list | localStorage | No | Yes |
| App settings | Tauri store | No | Yes |
| Last device IP | Config file | No | Yes |

### Session Token Handling

**Session tokens are never written to disk.** They exist only in the Rust backend's memory during your session:

1. You log in with email/password
2. Backend receives bearer token from Tablo cloud API
3. Token is stored in `LighthouseClient` struct (memory only)
4. Token is used for subsequent API calls
5. On logout or app close, token is cleared from memory

The frontend receives device information but **tokens are stripped before any localStorage persistence**.

## Network Security

### Cloud API Communication

All communication with `lighthousetv.ewscloud.com` (Tablo's cloud service) uses HTTPS/TLS:

- TLS 1.2+ required
- Certificate validation enabled
- No certificate pinning (relies on system trust store)

### Local Device Communication

Connections to Tablo devices on your LAN use HTTP (required by the device API):

**Legacy Devices**: Direct HTTP, no authentication required
**4th Gen Devices**: HTTP with HMAC-MD5 signed requests

The HMAC authentication ensures requests are valid even over unencrypted HTTP.

### No Telemetry

OpenTabTV does **not**:
- Send usage analytics
- Track user behavior
- Phone home to any server
- Include any third-party analytics SDKs

The only network connections are:
1. To your Tablo device (local network)
2. To Tablo's cloud API for 4th Gen authentication

## HMAC Keys in Source Code

You may notice HMAC keys in `src-tauri/src/tablo/auth.rs`:

```rust
const HASH_KEY: &str = "6l8jU5N43cEilqItmT3U2M2PFM3qPziilXqau9ys";
const DEVICE_KEY: &str = "ljpg6ZkwShVv8aI12E2LP55Ep8vq1uYDPvX0DdTB";
```

**These are not secrets.** They are:
- Public keys from Tablo's official authentication protocol
- Required for any app to communicate with 4th Gen Tablo devices
- Documented in the [tablo-api Python package](https://github.com/trevor-viljoen/tablo-api)
- Equivalent to a public API key, not a private secret

## Build Reproducibility

To verify the security of a build:

1. Clone the repository
2. Compare `Cargo.toml` dependencies against published crates
3. Build locally: `pnpm tauri build`
4. Compare binary hash with release

## Dependency Auditing

We regularly audit dependencies for known vulnerabilities:

```bash
# Rust dependencies
cargo audit

# Node dependencies
pnpm audit
```

## Reporting Security Issues

If you discover a security vulnerability:

1. **Do not** open a public issue
2. Email the maintainers directly or use GitHub's private vulnerability reporting
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to resolve the issue.

## Security Checklist

Before each release, we verify:

- [ ] No credentials in plain text storage
- [ ] CSP properly configured
- [ ] Minimal Tauri permissions
- [ ] No sensitive data in localStorage
- [ ] All dependencies up to date
- [ ] `cargo audit` passes
- [ ] `pnpm audit` passes

## Comparison with Alternatives

| Feature | OpenTabTV | Electron Apps | Browser-based |
|---------|-----------|---------------|---------------|
| Credential storage | OS Keychain | Varies (often plain file) | Browser storage |
| Process isolation | WebView sandbox | Chromium sandbox | Browser sandbox |
| Bundle size | ~10MB | ~150MB+ | N/A |
| Native permissions | Explicit opt-in | Often full access | Limited |
| Code signing | Supported | Supported | N/A |

## Version History

| Version | Security Changes |
|---------|------------------|
| 0.1.0 | Initial release with keyring credential storage, CSP, minimal permissions |
