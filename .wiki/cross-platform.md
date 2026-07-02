# Cross-Platform Build Guide

## Supported Platforms

| Platform | Minimum Version | WebView | Notes |
|----------|-----------------|---------|-------|
| macOS | 10.15 (Catalina) | WKWebView | Universal binary (Intel + ARM) |
| Windows | 7+ | WebView2 | Pre-installed on Windows 11 |
| Linux | Ubuntu 22.04+ | WebKitGTK 4.1 | webkit2gtk-4.1 required |

## Prerequisites by Platform

### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# Install Node.js (via Homebrew)
brew install node

# Install pnpm
npm install -g pnpm
```

**macOS 15 (Sequoia) Notes:**
- Full compatibility with Tauri 2.x
- WKWebView is native and always available
- Code signing requires Apple Developer account for distribution
- Notarization required for distribution outside App Store

### Windows

```powershell
# 1. Install Visual Studio Build Tools 2022
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Select "C++ build tools" workload and Windows 10/11 SDK

# 2. WebView2 (pre-installed on Windows 11)
# For Windows 10: https://developer.microsoft.com/microsoft-edge/webview2/

# 3. Install Rust
# Download rustup-init.exe from https://www.rust-lang.org/tools/install

# 4. Install Node.js
# Download from https://nodejs.org/

# 5. Install pnpm
npm install -g pnpm
```

**Windows 11 Notes:**
- WebView2 is pre-installed
- NSIS or MSI installer formats available
- Code signing via Azure Key Vault or local certificate

### Linux

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt install nodejs

# Install pnpm
npm install -g pnpm
```

## Build Commands

### Development

```bash
# All platforms
pnpm tauri dev
```

### Production Build

```bash
# Build for current platform
pnpm tauri build

# macOS: Universal binary (Intel + ARM)
pnpm tauri build --target universal-apple-darwin

# Windows: 64-bit
pnpm tauri build --target x86_64-pc-windows-msvc

# Linux: 64-bit
pnpm tauri build --target x86_64-unknown-linux-gnu
```

## Build Outputs

| Platform | Format | Location |
|----------|--------|----------|
| macOS | DMG, APP | `src-tauri/target/release/bundle/dmg/` |
| Windows | MSI, NSIS | `src-tauri/target/release/bundle/msi/` |
| Linux | DEB, AppImage, RPM | `src-tauri/target/release/bundle/` |

## GitHub Actions CI/CD

```yaml
name: Build and Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
            args: "--target aarch64-apple-darwin"
            rust_target: aarch64-apple-darwin
          - platform: macos-latest
            args: "--target x86_64-apple-darwin"
            rust_target: x86_64-apple-darwin
          - platform: windows-latest
            args: "--target x86_64-pc-windows-msvc"
            rust_target: x86_64-pc-windows-msvc
          - platform: ubuntu-22.04
            args: ""
            rust_target: x86_64-unknown-linux-gnu

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install pnpm
        run: npm install -g pnpm

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.rust_target }}

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev \
            build-essential libssl-dev libgtk-3-dev \
            libayatana-appindicator3-dev librsvg2-dev

      - name: Install frontend dependencies
        run: pnpm install

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: "Tablo v__VERSION__"
          args: ${{ matrix.args }}
```

## Code Signing

### macOS

Required for distribution outside development:

1. **Apple Developer Account** ($99/year)
2. **Developer ID Certificate**
3. **Notarization** (required for macOS 10.15+)

```bash
# Environment variables for CI
APPLE_CERTIFICATE=<base64-encoded-p12>
APPLE_CERTIFICATE_PASSWORD=<password>
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
APPLE_TEAM_ID=<team-id>
APPLE_ID=<apple-id-email>
APPLE_PASSWORD=<app-specific-password>
```

### Windows

Options for code signing:

1. **Azure Key Vault** (recommended for CI)
2. **Local Certificate** (PFX/P12)
3. **Hardware Token** (USB)

```bash
# Azure Key Vault for CI
AZURE_CLIENT_ID=<client-id>
AZURE_TENANT_ID=<tenant-id>
AZURE_CLIENT_SECRET=<secret>
```

## Platform-Specific Considerations

### macOS

- **App Sandbox**: Enable for App Store distribution
- **Hardened Runtime**: Required for notarization
- **Entitlements**: Camera, microphone, etc. require explicit permissions
- **Universal Binary**: Build for both Intel and Apple Silicon

### Windows

- **UAC**: Consider manifest settings for elevation
- **Windows Defender**: Sign to avoid SmartScreen warnings
- **Installer Choice**: NSIS (smaller) vs MSI (enterprise-friendly)

### Linux

- **AppImage**: Most portable, runs on most distros
- **DEB**: Debian/Ubuntu native package
- **RPM**: Fedora/RHEL native package
- **Flatpak**: Future consideration for sandboxed distribution

## FFmpeg and VLC Notes

Both are **NOT bundled** due to licensing (GPL/LGPL):

### macOS

```bash
# User installation
brew install ffmpeg
brew install --cask vlc
```

### Windows

```powershell
# User installation via winget
winget install FFmpeg
winget install VideoLAN.VLC
```

### Detection at Runtime

```rust
#[tauri::command]
fn check_ffmpeg() -> bool {
    which::which("ffmpeg").is_ok()
}

#[tauri::command]
fn check_vlc() -> bool {
    #[cfg(target_os = "macos")]
    return std::path::Path::new("/Applications/VLC.app").exists();

    #[cfg(target_os = "windows")]
    return which::which("vlc").is_ok() ||
        std::path::Path::new("C:\\Program Files\\VideoLAN\\VLC\\vlc.exe").exists();

    #[cfg(target_os = "linux")]
    return which::which("vlc").is_ok();
}
```

## Testing Matrix

| Test | macOS 15 | Windows 11 | Ubuntu 24.04 |
|------|----------|------------|--------------|
| Dev build | ✅ | ✅ | ✅ |
| Production build | ✅ | ✅ | ✅ |
| FFmpeg detection | ✅ | ✅ | ✅ |
| VLC integration | ✅ | ✅ | ✅ |
| HLS playback | ✅ | ✅ | ✅ |
| Device discovery | ✅ | ✅ | ✅ |

## Troubleshooting

### macOS

```bash
# Reset Xcode tools
sudo xcode-select --reset

# Clear Rust cache
cargo clean
```

### Windows

```powershell
# Verify WebView2
Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'

# Check Visual Studio tools
where cl.exe
```

### Linux

```bash
# Verify webkit2gtk
pkg-config --libs webkit2gtk-4.1

# Missing libraries
ldd target/release/tablo | grep "not found"
```
