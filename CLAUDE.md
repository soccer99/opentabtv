# Tablo - Desktop & Web App for Tablo DVR

A modern Tauri + Vue.js application for controlling and streaming from Tablo DVR devices.

## Quick Reference

### Development Commands

```bash
# Install dependencies
pnpm install

# Run development server with hot reload
pnpm tauri dev

# Build web frontend only (for testing)
pnpm build

# Type check without building
pnpm vue-tsc --noEmit
```

### Build Commands

```bash
# Build production app for current platform
pnpm tauri build

# macOS: Build universal binary (Intel + ARM)
pnpm tauri build --target universal-apple-darwin

# macOS: Build for Apple Silicon only
pnpm tauri build --target aarch64-apple-darwin

# macOS: Build for Intel only
pnpm tauri build --target x86_64-apple-darwin

# Windows: Build 64-bit installer
pnpm tauri build --target x86_64-pc-windows-msvc
```

### Tauri CLI Utilities

```bash
# Show Tauri environment info
pnpm tauri info

# Generate app icons from source image
pnpm tauri icon path/to/icon.png

# Update Tauri dependencies
pnpm tauri update
```

### Code Quality

```bash
# Type checking
pnpm vue-tsc --noEmit

# Format with Prettier (if configured)
pnpm prettier --write src/

# Lint with ESLint (if configured)
pnpm eslint src/ --fix
```

## Prerequisites

```bash
# Check installed versions
node --version        # Required: 18+
pnpm --version        # Required: 8+
rustc --version       # Required: 1.77+
cargo --version       # Comes with Rust

# External tools (user-installed, not bundled)
ffmpeg -version       # FFmpeg for video transcoding
vlc --version         # VLC (optional) for external playback

# macOS: Install external tools
brew install ffmpeg
brew install --cask vlc

# Windows: Install external tools
winget install FFmpeg
winget install VideoLAN.VLC
```

## Project Structure

```
tablo/
├── src/                    # Vue.js frontend
│   ├── assets/             # CSS, images
│   ├── components/         # Reusable UI components
│   ├── composables/        # Vue composables
│   ├── services/           # API service wrappers
│   ├── stores/             # Pinia state stores
│   ├── types/              # TypeScript definitions
│   ├── views/              # Page components
│   ├── App.vue             # Root component
│   ├── main.ts             # Entry point
│   └── router.ts           # Vue Router config
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri IPC commands
│   │   ├── tablo/          # Tablo API client
│   │   ├── lib.rs          # Library root
│   │   └── main.rs         # Entry point
│   ├── capabilities/       # Permission definitions
│   ├── icons/              # App icons
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── .wiki/                  # Project documentation
├── dist/                   # Built frontend (generated)
└── CLAUDE.md               # This file
```

## Wiki Documentation

Detailed documentation in `.wiki/`:

| File | Contents |
|------|----------|
| [tablo-api.md](.wiki/tablo-api.md) | Tablo device API, ports, auth flow |
| [third-party-dev.md](.wiki/third-party-dev.md) | 3rd party app development guide |
| [design-system.md](.wiki/design-system.md) | 2026 UI trends, colors, typography |
| [tech-stack.md](.wiki/tech-stack.md) | Technology choices and rationale |
| [code-style.md](.wiki/code-style.md) | TypeScript, Vue, Rust conventions |
| [architecture.md](.wiki/architecture.md) | System architecture, data flow |
| [features.md](.wiki/features.md) | Feature specifications |
| [cross-platform.md](.wiki/cross-platform.md) | Windows/macOS/Linux build guide |

## Key Conventions

### Vue Components
- Use `<script setup lang="ts">` exclusively
- Props with TypeScript interfaces
- Pinia stores with Composition API (setup style)
- Tailwind CSS for styling

### Tauri Commands
```typescript
// Frontend: invoke Rust commands
import { invoke } from '@tauri-apps/api/core'
const devices = await invoke<TabloDevice[]>('discover_devices')
```

```rust
// Backend: define commands
#[tauri::command]
async fn discover_devices() -> Result<Vec<TabloDevice>, String> { }
```

### State Management
- `useDevicesStore` - Device discovery/connection
- `useChannelsStore` - Channel list and live TV
- `useGuideStore` - TV guide data
- `useRecordingsStore` - DVR recordings
- `useSettingsStore` - User preferences (persisted)

## External Dependencies

FFmpeg and VLC are **NOT bundled** due to licensing:
- FFmpeg: LGPL/GPL - user must install separately
- VLC: GPL - optional, detected if installed

## Network Requirements

| Port | Protocol | Purpose |
|------|----------|---------|
| 8881 | UDP | Device discovery |
| 8885 | HTTP | Tablo API |
| 8887 | WebSocket | WAMP API |
| 80 | HTTP | Video streams |

## Design Principles

1. **Dark mode first** - Light theme as option
2. **Glassmorphism** - Subtle frosted glass panels
3. **Bento grid** - Card-based content layout
4. **Minimal controls** - Auto-hide video controls
5. **Keyboard first** - Full keyboard navigation

## Cross-Platform Notes

| Platform | Installer | Notes |
|----------|-----------|-------|
| macOS | DMG, APP | Universal binary, needs code signing |
| Windows | MSI, NSIS | WebView2 pre-installed on Win11 |
| Linux | AppImage, DEB | webkit2gtk-4.1 required |

See [cross-platform.md](.wiki/cross-platform.md) for detailed build instructions.

## Tablo Device Generations

This app targets **ALL Tablo device generations** with unified API support.

### Device Generation Support

| Generation | Models | Auth Required | Status |
|------------|--------|---------------|--------|
| Legacy (Pre-4th Gen) | DUAL LITE, DUAL 128GB, QUAD, QUAD 1TB, Original 1/2 Tuner | None | Supported |
| 4th Gen (2024+) | All current models with 128GB built-in storage | Cloud + HMAC-MD5 | Supported |

### API Differences by Generation

| Aspect | Legacy | 4th Gen |
|--------|--------|---------|
| **Authentication** | None required | Cloud auth + HMAC-MD5 signed local requests |
| **Discovery** | UDP broadcast (port 8881) | Cloud API (`api.tablotv.com`) |
| **Guide Data** | Local device API (port 8885) | Cloud API (`lighthousetv.ewscloud.com`) |
| **Local API** | Direct HTTP to :8885 | HMAC-MD5 signed requests to :8885 |
| **Video Codec** | H.264 (browser compatible) | May use H.265/HEVC (needs FFmpeg) |
| **Subscription** | Required for guide data | Free (no subscription needed) |

### 4th Gen Authentication Flow

```
1. POST lighthousetv.ewscloud.com/api/v2/login/  → Bearer token
2. GET  /api/v2/account/                          → Device list with local IPs
3. POST /api/v2/account/select/                   → Device-scoped Lighthouse token
4. GET  /api/v2/account/{token}/guide/channels/   → Channel guide data (cloud)
5. POST http://{local_ip}:8885/.../watch          → HLS playlist (HMAC-MD5 signed)
```

### Legacy Sunset Notice

**September 1, 2026**: Tablo is discontinuing support for all legacy Tablo apps. Legacy devices can opt into the "Legacy Transition Program" to use 4th Gen apps with firmware 2.2.58+.

## Reference Implementations (.ref/)

Local copies of reference projects for API understanding:

### tablo-web (4th Gen ONLY)
- **Path**: `.ref/tablo-web/`
- **Source**: [github.com/trevor-viljoen/tablo-web](https://github.com/trevor-viljoen/tablo-web)
- **Stack**: Python (FastAPI) + React + FFmpeg
- **Auth**: Uses `tablo-api` Python package for cloud auth
- **Key Files**:
  - `backend/app/state.py` - 4th Gen auth flow, cloud API integration
  - `backend/app/routes/auth.py` - Login/device selection endpoints
  - `backend/app/routes/stream.py` - HLS proxy with FFmpeg transcoding

### Tablo_Lite (Legacy ONLY)
- **Path**: `.ref/Tablo_Lite/`
- **Source**: [github.com/Epchk/Tablo_Lite](https://github.com/Epchk/Tablo_Lite)
- **Stack**: Vanilla JavaScript (no framework)
- **Auth**: None (unauthenticated local API)
- **Key Files**:
  - `tablo_ota.js` - Complete legacy API wrapper (fetch-based)
  - `index.html` - Single-page UI

### tablo-tools-electron (Legacy ONLY)
- **Path**: `.ref/tablo-tools-electron/`
- **Source**: [github.com/jessedp/tablo-tools-electron](https://github.com/jessedp/tablo-tools-electron)
- **Stack**: TypeScript/Electron + React + Redux
- **Auth**: None
- **Status**: Archived, explicitly states "4th Gen NOT supported"
- **Key Files**:
  - Uses `tablo-api-js` npm package for legacy API

### Other Reference Projects (not cloned)

| Project | URL | Generation | Notes |
|---------|-----|------------|-------|
| tablo2plex | [hearhellacopters/tablo2plex](https://github.com/hearhellacopters/tablo2plex) | 4th Gen | HDHomeRun emulator, HMAC-MD5 auth reverse-engineered |
| TabloHARemote | [dgshue/TabloHARemote](https://github.com/dgshue/TabloHARemote) | 4th Gen | Home Assistant integration |
| tablo-api (Python) | [trevor-viljoen/tablo-api](https://github.com/trevor-viljoen/tablo-api) | 4th Gen | Cloud auth wrapper (`TabloAuth`, `TabloClient`) |
| tablo-api-js | [jessedp/tablo-api-js](https://github.com/jessedp/tablo-api-js) | Legacy | Node.js API wrapper |
| @gibme/tablo.tv | [gibme-npm/tablo.tv](https://github.com/gibme-npm/tablo.tv) | Both | TypeScript, requires HMAC keys for 4th Gen |

## References

- [Tauri v2 Docs](https://v2.tauri.app/)
- [Vue 3 Docs](https://vuejs.org/)
- [Pinia Docs](https://pinia.vuejs.org/)
- [Tailwind CSS](https://tailwindcss.com/)
- [hls.js](https://github.com/video-dev/hls.js/)
- [Tablo API Docs (Legacy)](https://jessedp.github.io/tablo-api-docs/)
- [Tablo Support - 4th Gen Guide](https://support.tablotv.com/hc/en-us/articles/18689595623188)
- [Tablo Legacy Transition Program](https://www.tablotv.com/legacy-transition/)
