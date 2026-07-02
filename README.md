# OpenTabTV

An open source desktop application for controlling and streaming from [Tablo DVR](https://www.tablotv.com/) devices. Built with Tauri, Vue.js, and Rust.

> **Note:** This is an independent, community-developed project and is not affiliated with or endorsed by Nuvyyo Inc. (the makers of Tablo DVR).

## Features

- **Device Discovery** - Auto-discover Tablo devices on your local network via UDP broadcast or cloud lookup
- **Live TV** - Watch live channels with HLS streaming and channel guide
- **TV Guide** - Browse the program guide with a grid layout showing channels and schedules
- **Recordings** - Browse, play, and manage your DVR recordings library
- **Cross-Platform** - Native apps for macOS, Windows, and Linux
- **Lightweight** - Small bundle size (~10MB) compared to Electron alternatives

## Requirements

### System Requirements

| Platform | Minimum Version | Notes |
|----------|-----------------|-------|
| macOS | 10.15 (Catalina) | Universal binary (Intel + Apple Silicon) |
| Windows | 7+ | WebView2 required (pre-installed on Windows 11) |
| Linux | Ubuntu 22.04+ | webkit2gtk-4.1 required |

### Development Prerequisites

```bash
# Check versions
node --version        # Required: 18+
pnpm --version        # Required: 8+
rustc --version       # Required: 1.77+
```

### External Tools (Optional)

FFmpeg and VLC are **not bundled** due to licensing. Install them separately for full functionality:

```bash
# macOS
brew install ffmpeg
brew install --cask vlc

# Windows
winget install FFmpeg
winget install VideoLAN.VLC

# Linux
sudo apt install ffmpeg vlc
```

## Installation

### From Source

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
   ```

2. **Install Node.js and pnpm**
   ```bash
   # macOS
   brew install node
   npm install -g pnpm

   # Or use your preferred Node version manager
   ```

3. **Install platform dependencies**

   **macOS:**
   ```bash
   xcode-select --install
   ```

   **Linux (Ubuntu/Debian):**
   ```bash
   sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
     libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
   ```

   **Windows:**
   - Install [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Select "C++ build tools" workload with Windows SDK

4. **Clone and install**
   ```bash
   git clone https://github.com/soccer99/opentabtv.git
   cd opentabtv
   pnpm install
   ```

## Development

### Commands

```bash
# Start development server with hot reload
pnpm tauri dev

# Type check without building
pnpm vue-tsc --noEmit

# Build web frontend only
pnpm build

# Preview production build
pnpm preview
```

### Build

```bash
# Build for current platform
pnpm tauri build

# macOS: Universal binary (Intel + ARM)
pnpm tauri build --target universal-apple-darwin

# Windows: 64-bit
pnpm tauri build --target x86_64-pc-windows-msvc
```

See [BUILD.md](BUILD.md) for complete build documentation including CI/CD setup, code signing, and platform-specific notes.

## Usage

### Connecting to Your Tablo

1. Launch the app
2. Your Tablo device will be discovered automatically on the local network
3. Click the device to connect
4. If auto-discovery fails, enter the device IP manually

### Watching Live TV

1. Navigate to **Live TV**
2. Browse the channel list with current program info
3. Click a channel to start watching
4. Use overlay controls for volume, fullscreen, and quality settings

### Browsing Recordings

1. Navigate to **Recordings**
2. View recordings grouped by show, movie, or date
3. Click to play, right-click to delete

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Space | Play/Pause |
| F | Toggle fullscreen |
| M | Mute/Unmute |
| Left/Right | Seek 10 seconds |
| Up/Down | Volume |
| Esc | Exit fullscreen / Close player |
| G | Go to Guide |
| L | Go to Live TV |
| R | Go to Recordings |

## Configuration

Settings are persisted locally and include:

- **Theme** - Dark / Light / System
- **Video Quality** - Auto / 720p / 1080p
- **Default View** - Choose startup screen
- **FFmpeg Path** - Custom path if not in system PATH
- **VLC Integration** - Enable/disable external VLC playback

## Network Requirements

Your Tablo and the app must be on the same local network.

| Port | Protocol | Purpose |
|------|----------|---------|
| 8881 | UDP | Device discovery |
| 8885 | HTTP | Tablo REST API |
| 8887 | WebSocket | Real-time events (WAMP) |
| 80 | HTTP | Video streams |

## Architecture

```
+-------------------------------------------+
|            Vue.js Frontend                |
|  +----------+ +--------+ +-------------+  |
|  |  Views   | | Stores | | Composables |  |
|  +----------+ +--------+ +-------------+  |
|                   | IPC                   |
+-------------------------------------------+
|           Tauri Backend (Rust)            |
|  +----------+ +--------+ +-----------+    |
|  | Commands | | Tablo  | |  FFmpeg   |    |
|  |          | |  API   | |  Wrapper  |    |
|  +----------+ +--------+ +-----------+    |
+-------------------------------------------+
          |                |
          v                v
    +-----------+    +-----------+
    |   Tablo   |    |  FFmpeg   |
    |  Device   |    | (external)|
    +-----------+    +-----------+
```

## Project Structure

```
tablo/
├── src/                    # Vue.js frontend
│   ├── components/         # Reusable UI components
│   ├── composables/        # Vue composables
│   ├── services/           # API service wrappers
│   ├── stores/             # Pinia state stores
│   ├── types/              # TypeScript definitions
│   ├── views/              # Page components
│   └── router.ts           # Vue Router config
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri IPC commands
│   │   └── tablo/          # Tablo API client
│   ├── capabilities/       # Permission definitions
│   └── tauri.conf.json     # Tauri configuration
├── .wiki/                  # Project documentation
└── README.md               # This file
```

## Tech Stack

| Layer | Technology | Version |
|-------|------------|---------|
| Framework | Tauri | 2.x |
| Frontend | Vue.js | 3.x |
| Language | TypeScript | 5.x |
| Build Tool | Vite | 6.x |
| State | Pinia | 3.x |
| Routing | Vue Router | 4.x |
| Styling | Tailwind CSS | 4.x |
| Video | hls.js | 1.x |
| Backend | Rust | 1.77+ |

## Documentation

| Document | Description |
|----------|-------------|
| [BUILD.md](BUILD.md) | Complete build guide for all platforms |
| [SECURITY.md](SECURITY.md) | Security practices and data handling |
| [.wiki/tablo-api.md](.wiki/tablo-api.md) | Tablo device API reference |
| [.wiki/architecture.md](.wiki/architecture.md) | System design and data flow |
| [.wiki/features.md](.wiki/features.md) | Feature specifications |
| [.wiki/tech-stack.md](.wiki/tech-stack.md) | Technology choices |
| [.wiki/code-style.md](.wiki/code-style.md) | Coding conventions |
| [.wiki/design-system.md](.wiki/design-system.md) | UI/UX guidelines |

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Security

OpenTabTV is designed with security as a priority:

- **Credentials stored in OS Keychain** (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- **Strict Content Security Policy** preventing XSS and code injection
- **Minimal Tauri permissions** following principle of least privilege
- **No telemetry or analytics** - your data stays on your device

See [SECURITY.md](SECURITY.md) for complete security documentation.

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
# Verify WebView2 is installed
Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
```

### Linux

```bash
# Verify webkit2gtk
pkg-config --libs webkit2gtk-4.1

# Check for missing libraries
ldd target/release/tablo | grep "not found"
```

## Contributing

Contributions are welcome! Please read the documentation in `.wiki/` before submitting changes.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run type checking: `pnpm vue-tsc --noEmit`
5. Submit a pull request

## License

[MIT License](LICENSE)

## Acknowledgments

- [Tablo API Documentation](https://jessedp.github.io/tablo-api-docs/) - Unofficial API docs
- [Tauri](https://tauri.app/) - Desktop app framework
- [Vue.js](https://vuejs.org/) - Frontend framework
- [hls.js](https://github.com/video-dev/hls.js/) - HLS playback library
