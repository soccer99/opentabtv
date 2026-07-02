# Technology Stack

## Overview

Desktop + Web application for Tablo DVR using modern cross-platform technologies.

## Core Stack

| Layer      | Technology                  | Version  | Notes                           |
|------------|-----------------------------|---------|---------------------------------|
| Framework  | Tauri                       | 2.x     | Lightweight desktop runtime     |
| Frontend   | Vue.js                      | 3.x     | Composition API + `<script setup>` |
| Language   | TypeScript                  | 5.x     | Strict mode enabled             |
| Build      | Vite                        | 5.x     | Fast dev server & bundler       |
| State      | Pinia                       | 2.x     | Vue's official state management |
| Routing    | Vue Router                  | 4.x     | Client-side navigation          |
| Styling    | Tailwind CSS                | 4.x     | Utility-first CSS               |
| Video      | hls.js                      | 1.x     | HLS playback in browser         |
| Backend    | Rust (Tauri)                | 1.77+   | System operations               |

## External Dependencies

### Required (User Must Install)

| Tool   | Purpose                     | License        | Notes                    |
|--------|-----------------------------|----------------|--------------------------|
| FFmpeg | Video transcoding           | LGPL/GPL       | Not bundled (licensing)  |
| VLC    | Alternative playback        | GPL            | Optional, if installed   |

### Detection Strategy
```rust
// Check for FFmpeg
which ffmpeg || where ffmpeg

// Check for VLC on macOS
/Applications/VLC.app/Contents/MacOS/VLC --version
```

## Why Tauri over Electron?

| Aspect         | Tauri           | Electron        |
|----------------|-----------------|-----------------|
| Bundle Size    | ~5-10 MB        | ~150+ MB        |
| Memory Usage   | Lower           | Higher          |
| Security       | Rust + Sandbox  | Node.js         |
| WebView        | System native   | Bundled Chromium|
| Build Time     | Faster          | Slower          |

## Tauri 2.0 Features Used

- **Capability-based permissions** - deny by default
- **Plugin system** - fs, dialog, shell, clipboard, notification
- **IPC via Commands** - Rust ↔ TypeScript communication
- **Auto-updater** - Built-in update mechanism
- **macOS code signing** - Developer ID support

## Vue 3 Architecture

### Composition API Pattern
```typescript
// stores/channels.ts
export const useChannelsStore = defineStore('channels', () => {
  const channels = ref<Channel[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const sortedChannels = computed(() =>
    [...channels.value].sort((a, b) => a.number - b.number)
  )

  async function fetchChannels() {
    loading.value = true
    try {
      channels.value = await invoke('get_channels')
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error'
    } finally {
      loading.value = false
    }
  }

  return { channels, loading, error, sortedChannels, fetchChannels }
})
```

### Component Structure
```
src/
├── components/        # Reusable UI components
│   ├── ui/           # Base components (Button, Card, etc.)
│   ├── player/       # Video player components
│   └── guide/        # TV guide components
├── views/            # Page-level components
├── stores/           # Pinia stores
├── composables/      # Shared composition functions
├── services/         # API/Tauri service wrappers
├── types/            # TypeScript type definitions
└── utils/            # Utility functions
```

## Video Playback Strategy

### Primary: hls.js
```typescript
import Hls from 'hls.js'

if (Hls.isSupported()) {
  const hls = new Hls({
    lowLatencyMode: true,
    liveSyncDuration: 1.5,
    liveMaxLatencyDuration: 3.5
  })
  hls.loadSource(streamUrl)
  hls.attachMedia(videoElement)
}
```

### Fallback: FFmpeg Transcoding
For streams with codecs not supported by browser:
```rust
// Tauri command to start FFmpeg transcoding
#[tauri::command]
async fn transcode_stream(input_url: String) -> Result<String, String> {
    // FFmpeg: input HLS → re-encode to H.264/AAC → output HLS
    // Return local HLS URL for hls.js playback
}
```

### VLC Integration (Optional)
```rust
#[tauri::command]
async fn open_in_vlc(stream_url: String) -> Result<(), String> {
    // Check if VLC exists, launch with stream URL
    Command::new("/Applications/VLC.app/Contents/MacOS/VLC")
        .arg(stream_url)
        .spawn()
}
```

## State Persistence

Using `pinia-plugin-persistedstate`:
```typescript
// Persist user preferences, device cache
export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light'>('dark')
  const savedDevices = ref<TabloDevice[]>([])

  return { theme, savedDevices }
}, {
  persist: true
})
```

## Build Targets

| Platform | Format    | Notes                          |
|----------|-----------|--------------------------------|
| macOS    | DMG, APP  | Universal binary (Intel + ARM) |
| Windows  | MSI, NSIS | Future consideration           |
| Linux    | AppImage  | Future consideration           |
| Web      | Static    | Hosted version without Tauri   |

## Development Tools

- **VS Code** with Volar, Tauri, rust-analyzer extensions
- **Vue Devtools** for state inspection
- **CrabNebula DevTools** for Tauri debugging
