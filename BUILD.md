# Building OpenTabTV

This guide covers building OpenTabTV desktop applications for macOS, Windows, and Linux.

## Quick Start

```bash
# Install dependencies
pnpm install

# Development with hot reload
pnpm tauri dev

# Production build for current platform
pnpm tauri build
```

## Supported Platforms

| Platform | Minimum Version | WebView | Architecture |
|----------|-----------------|---------|--------------|
| macOS | 10.15 (Catalina) | WKWebView | Universal (Intel + ARM) |
| Windows | 7+ | WebView2 | x64 |
| Linux | Ubuntu 22.04+ | WebKitGTK 4.1 | x64 |

## Prerequisites

### All Platforms

- **Node.js** 18+
- **pnpm** 8+
- **Rust** 1.77+

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

### Windows

1. **Visual Studio Build Tools 2022**
   - Download from [Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Select "C++ build tools" workload with Windows 10/11 SDK

2. **WebView2** (pre-installed on Windows 11)
   - For Windows 10: [Download WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)

3. **Rust**
   - Download from [rustup](https://www.rust-lang.org/tools/install)

4. **Node.js**
   - Download from [nodejs.org](https://nodejs.org/)

5. **pnpm**
   ```powershell
   npm install -g pnpm
   ```

### Linux (Ubuntu/Debian)

```bash
# Install system dependencies
sudo apt update
sudo apt install \
    libwebkit2gtk-4.1-dev \
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
# Start development server with hot reload
pnpm tauri dev
```

### Production Builds

```bash
# Build for current platform
pnpm tauri build

# macOS: Universal binary (Intel + ARM)
pnpm tauri build --target universal-apple-darwin

# macOS: Apple Silicon only
pnpm tauri build --target aarch64-apple-darwin

# macOS: Intel only
pnpm tauri build --target x86_64-apple-darwin

# Windows: 64-bit
pnpm tauri build --target x86_64-pc-windows-msvc

# Linux: 64-bit
pnpm tauri build --target x86_64-unknown-linux-gnu
```

### Type Checking

```bash
# TypeScript type check (frontend)
pnpm vue-tsc --noEmit

# Rust check (backend)
cd src-tauri && cargo check
```

## Build Outputs

| Platform | Format | Location |
|----------|--------|----------|
| macOS | DMG | `src-tauri/target/release/bundle/dmg/` |
| macOS | APP | `src-tauri/target/release/bundle/macos/` |
| Windows | MSI | `src-tauri/target/release/bundle/msi/` |
| Windows | NSIS | `src-tauri/target/release/bundle/nsis/` |
| Linux | DEB | `src-tauri/target/release/bundle/deb/` |
| Linux | AppImage | `src-tauri/target/release/bundle/appimage/` |
| Linux | RPM | `src-tauri/target/release/bundle/rpm/` |

## External Tools

FFmpeg and VLC are **not bundled** due to GPL licensing. Users install them separately:

### macOS

```bash
brew install ffmpeg
brew install --cask vlc
```

### Windows

```powershell
winget install FFmpeg
winget install VideoLAN.VLC
```

### Linux

```bash
sudo apt install ffmpeg vlc
```

The app detects these tools at runtime and gracefully degrades if not present.

## Code Signing

### macOS

Required for distribution outside the App Store:

1. **Apple Developer Account** ($99/year)
2. **Developer ID Certificate**
3. **Notarization** (required for macOS 10.15+)

Environment variables for CI:

```bash
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

Without signing, Windows SmartScreen will warn users.

## GitHub Actions CI/CD

Example workflow for automated builds:

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
          releaseName: "OpenTabTV v__VERSION__"
          args: ${{ matrix.args }}
```

## Platform-Specific Notes

### macOS

- **Universal Binary**: Build for both Intel and Apple Silicon with `--target universal-apple-darwin`
- **Hardened Runtime**: Required for notarization
- **App Sandbox**: Enable for App Store distribution
- **Gatekeeper**: Users may need to right-click > Open on first launch if unsigned

### Windows

- **WebView2**: Automatically installed on Windows 11, may need manual install on Windows 10
- **Installer Choice**: NSIS (smaller) vs MSI (enterprise-friendly)
- **UAC**: App does not require elevation

### Linux

- **AppImage**: Most portable, runs on most distributions
- **DEB**: Native for Debian/Ubuntu
- **RPM**: Native for Fedora/RHEL
- **Secret Service**: Required for credential storage (GNOME Keyring or KWallet)

## Troubleshooting

### macOS

```bash
# Reset Xcode tools
sudo xcode-select --reset

# Clear Rust build cache
cargo clean

# Verify Xcode license accepted
sudo xcodebuild -license accept
```

### Windows

```powershell
# Verify WebView2 installed
Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'

# Check Visual Studio tools
where cl.exe

# Verify Rust target installed
rustup target list --installed
```

### Linux

```bash
# Verify webkit2gtk
pkg-config --libs webkit2gtk-4.1

# Check for missing libraries
ldd target/release/open-tab-tv | grep "not found"

# Verify D-Bus running (for keyring)
echo $DBUS_SESSION_BUS_ADDRESS
```

### Build Errors

| Error | Solution |
|-------|----------|
| `webkit2gtk-4.1 not found` | Install `libwebkit2gtk-4.1-dev` |
| `WebView2 not found` | Install WebView2 runtime |
| `linker cc not found` | Install build-essential / Visual Studio Build Tools |
| `keyring error` | Ensure Secret Service daemon is running (Linux) |

## Testing Matrix

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Development build | Yes | Yes | Yes |
| Production build | Yes | Yes | Yes |
| HLS playback | Yes | Yes | Yes |
| Device discovery | Yes | Yes | Yes |
| Keychain storage | Yes | Yes | Yes |
| FFmpeg integration | Yes | Yes | Yes |
| VLC integration | Yes | Yes | Yes |

## Bundle Size

| Platform | Installer Size | Installed Size |
|----------|---------------|----------------|
| macOS (Universal) | ~15 MB | ~35 MB |
| Windows (NSIS) | ~8 MB | ~25 MB |
| Linux (AppImage) | ~20 MB | N/A |
| Linux (DEB) | ~8 MB | ~25 MB |

Sizes are approximate and may vary with dependencies.
