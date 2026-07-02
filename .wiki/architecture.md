# Application Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tablo Desktop App                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Vue.js Frontend                       │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │    │
│  │  │  Views   │ │Components│ │  Stores  │ │Composables│   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │    │
│  │                      ↓ invoke()                          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                           │ IPC                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Tauri Backend (Rust)                  │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │    │
│  │  │ Commands │ │  Tablo   │ │  FFmpeg  │ │  Config  │   │    │
│  │  │          │ │   API    │ │ Wrapper  │ │  Store   │   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│                      Tauri Runtime (WebView)                     │
└─────────────────────────────────────────────────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
           ▼                  ▼                  ▼
    ┌──────────┐      ┌──────────────┐    ┌──────────┐
    │  Tablo   │      │    FFmpeg    │    │   VLC    │
    │  Device  │      │  (external)  │    │(optional)│
    └──────────┘      └──────────────┘    └──────────┘
```

## Frontend Architecture

### View Hierarchy

```
App.vue
├── MainLayout.vue
│   ├── Sidebar.vue (navigation)
│   └── <router-view>
│       ├── HomeView.vue (device selection, quick access)
│       ├── LiveTVView.vue (channel list, now playing)
│       ├── GuideView.vue (TV guide grid)
│       ├── RecordingsView.vue (library browser)
│       └── SettingsView.vue (preferences)
```

### Store Architecture

```typescript
// Centralized state management with Pinia

// devices.ts - Device discovery and connection
export const useDevicesStore = defineStore('devices', () => {
  const devices = ref<TabloDevice[]>([])
  const activeDevice = ref<TabloDevice | null>(null)
  const connectionState = ref<ConnectionState>('disconnected')

  async function discoverDevices() { /* ... */ }
  async function connectToDevice(device: TabloDevice) { /* ... */ }
  async function connectByIp(ip: string) { /* ... */ }
})

// channels.ts - Channel data and live TV
export const useChannelsStore = defineStore('channels', () => {
  const channels = ref<Channel[]>([])
  const currentChannel = ref<Channel | null>(null)
  const streamUrl = ref<string | null>(null)

  async function fetchChannels() { /* ... */ }
  async function watchChannel(channelId: string) { /* ... */ }
})

// guide.ts - TV guide data
export const useGuideStore = defineStore('guide', () => {
  const programs = ref<Map<string, Program[]>>(new Map())
  const timeRange = ref({ start: Date, end: Date })

  async function fetchGuide(date: Date) { /* ... */ }
})

// recordings.ts - DVR recordings
export const useRecordingsStore = defineStore('recordings', () => {
  const recordings = ref<Recording[]>([])
  const categories = computed(() => /* group by show/movie */)

  async function fetchRecordings() { /* ... */ }
  async function playRecording(id: string) { /* ... */ }
  async function deleteRecording(id: string) { /* ... */ }
})

// settings.ts - User preferences (persisted)
export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light' | 'system'>('dark')
  const preferredQuality = ref<'auto' | '720p' | '1080p'>('auto')
  const savedDevices = ref<SavedDevice[]>([])
}, { persist: true })
```

### Composables

```typescript
// composables/useMediaPlayer.ts
export function useMediaPlayer(videoRef: Ref<HTMLVideoElement | null>) {
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)
  const buffered = ref(0)
  const volume = ref(1)
  const isMuted = ref(false)
  const error = ref<string | null>(null)

  let hls: Hls | null = null

  function loadSource(url: string) { /* ... */ }
  function play() { /* ... */ }
  function pause() { /* ... */ }
  function seek(time: number) { /* ... */ }
  function setVolume(level: number) { /* ... */ }
  function destroy() { /* ... */ }

  return {
    isPlaying,
    currentTime,
    duration,
    buffered,
    volume,
    isMuted,
    error,
    loadSource,
    play,
    pause,
    seek,
    setVolume,
    destroy
  }
}

// composables/useDeviceDiscovery.ts
export function useDeviceDiscovery() {
  const isScanning = ref(false)
  const devices = ref<TabloDevice[]>([])

  async function scanNetwork() { /* ... */ }
  async function checkManualIp(ip: string) { /* ... */ }

  return { isScanning, devices, scanNetwork, checkManualIp }
}
```

## Backend Architecture (Rust)

### Module Structure

```rust
// src/lib.rs
mod commands;
mod tablo;
mod ffmpeg;
mod config;
mod error;

pub use commands::*;

// src/commands/mod.rs
mod device;
mod channel;
mod recording;
mod playback;

pub use device::*;
pub use channel::*;
pub use recording::*;
pub use playback::*;

// src/tablo/mod.rs
mod auth;
mod api;
mod discovery;
mod types;

pub struct TabloClient {
    auth: TabloAuth,
    device: Option<TabloDevice>,
}

impl TabloClient {
    pub async fn discover() -> Result<Vec<TabloDevice>, TabloError> { }
    pub async fn connect(&mut self, device: TabloDevice) -> Result<(), TabloError> { }
    pub async fn get_channels(&self) -> Result<Vec<Channel>, TabloError> { }
    pub async fn watch_channel(&self, id: &str) -> Result<StreamInfo, TabloError> { }
}

// src/ffmpeg/mod.rs
pub struct FfmpegWrapper {
    process: Option<Child>,
}

impl FfmpegWrapper {
    pub fn is_available() -> bool { }
    pub async fn transcode_hls(&mut self, input: &str, output: &str) -> Result<(), FfmpegError> { }
    pub fn stop(&mut self) { }
}
```

### Tauri Commands

```rust
// src/commands/device.rs
#[tauri::command]
pub async fn discover_devices() -> Result<Vec<TabloDevice>, String> {
    TabloClient::discover().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_device(
    state: State<'_, AppState>,
    device: TabloDevice
) -> Result<DeviceInfo, String> {
    let mut client = state.tablo_client.lock().await;
    client.connect(device).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_by_ip(
    state: State<'_, AppState>,
    ip: String
) -> Result<DeviceInfo, String> {
    // Validate IP, create device, connect
}

// src/commands/playback.rs
#[tauri::command]
pub async fn start_live_stream(
    state: State<'_, AppState>,
    channel_id: String
) -> Result<StreamUrl, String> {
    let client = state.tablo_client.lock().await;
    let stream = client.watch_channel(&channel_id).await?;

    // Check if transcoding needed
    if needs_transcoding(&stream) {
        let output_url = state.ffmpeg.transcode(&stream.url).await?;
        Ok(StreamUrl { url: output_url, transcoded: true })
    } else {
        Ok(StreamUrl { url: stream.url, transcoded: false })
    }
}
```

### State Management

```rust
// src/main.rs
struct AppState {
    tablo_client: Mutex<TabloClient>,
    ffmpeg: FfmpegWrapper,
    config: RwLock<AppConfig>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            tablo_client: Mutex::new(TabloClient::new()),
            ffmpeg: FfmpegWrapper::new(),
            config: RwLock::new(AppConfig::load()),
        })
        .invoke_handler(tauri::generate_handler![
            discover_devices,
            connect_device,
            connect_by_ip,
            get_channels,
            start_live_stream,
            stop_stream,
            get_recordings,
            // ... more commands
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

## Data Flow

### Device Connection Flow

```
User enters IP / clicks device
        │
        ▼
┌───────────────────┐
│ connectDevice()   │  (Vue)
└─────────┬─────────┘
          │ invoke
          ▼
┌───────────────────┐
│ connect_device    │  (Rust command)
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ TabloClient       │  (Rust)
│ - authenticate()  │
│ - get_device_info │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ Tablo Cloud API   │  (HTTPS)
│ + Local Device    │  (HTTP)
└───────────────────┘
```

### Live TV Playback Flow

```
User clicks channel
        │
        ▼
┌───────────────────┐
│ watchChannel()    │  (Vue store)
└─────────┬─────────┘
          │ invoke
          ▼
┌───────────────────┐
│ start_live_stream │  (Rust command)
├───────────────────┤
│ 1. Request stream │
│ 2. Check codec    │
│ 3. Transcode?     │
└─────────┬─────────┘
          │
    ┌─────┴─────┐
    │           │
    ▼           ▼
 Direct     FFmpeg
  HLS      Transcode
    │           │
    └─────┬─────┘
          │
          ▼
┌───────────────────┐
│ Return stream URL │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ hls.js player     │  (Vue component)
│ loads & plays     │
└───────────────────┘
```

## Security Model

### Tauri Capabilities

```json
// src-tauri/capabilities/main.json
{
  "identifier": "main",
  "description": "Main window capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "dialog:allow-open",
    "fs:allow-read-app-config",
    "fs:allow-write-app-config",
    "http:allow-fetch"
  ]
}
```

### Network Security

- Tablo cloud auth over HTTPS
- Local device communication on LAN only (HTTP)
- Credentials stored in OS keychain (via Tauri plugin)
- No credentials sent to external servers except Tablo cloud

## Error Handling Strategy

```typescript
// Frontend: Use typed error responses
interface ApiError {
  code: string
  message: string
  details?: unknown
}

interface ApiResult<T> {
  data?: T
  error?: ApiError
}

// Display user-friendly errors
function handleError(error: ApiError) {
  switch (error.code) {
    case 'DEVICE_NOT_FOUND':
      toast.error('Tablo device not found. Check network connection.')
      break
    case 'AUTH_FAILED':
      toast.error('Authentication failed. Please check credentials.')
      break
    case 'FFMPEG_NOT_FOUND':
      toast.error('FFmpeg not installed. Some streams may not play.')
      break
    default:
      toast.error(error.message)
  }
}
```

## Performance Considerations

1. **IPC Overhead**: Minimize invoke() calls, batch data where possible
2. **Video Rendering**: Use native WebView video element, avoid React/Vue re-renders
3. **State Updates**: Debounce rapid state changes (e.g., playback position)
4. **Memory**: Clean up HLS instances and FFmpeg processes on navigation
5. **Startup**: Lazy load views, persist device cache for quick reconnection
