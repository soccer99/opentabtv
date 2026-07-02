# Code Style Guide

## TypeScript

### General Rules

- Strict mode enabled (`strict: true`)
- Explicit return types on public functions
- No `any` - use `unknown` and narrow
- Prefer `interface` over `type` for object shapes
- Use `const` assertions for literal types

### Naming Conventions

```typescript
// Variables: camelCase
const channelList = []
const isLoading = false

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT = 3
const API_BASE_URL = 'http://localhost:8885'

// Types/Interfaces: PascalCase
interface TabloDevice {
  id: string
  name: string
  localIp: string
}

// Enums: PascalCase with PascalCase members
enum PlaybackState {
  Idle = 'idle',
  Loading = 'loading',
  Playing = 'playing',
  Paused = 'paused',
  Error = 'error'
}

// Functions: camelCase, verb prefix
function fetchChannels() {}
function handlePlayClick() {}
async function loadDeviceInfo() {}

// Components: PascalCase
// ChannelCard.vue, VideoPlayer.vue

// Composables: use* prefix
function useChannels() {}
function useMediaPlayer() {}
```

### Imports

```typescript
// Order: external → internal → types → styles
import { ref, computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import Hls from 'hls.js'

import { useChannelsStore } from '@/stores/channels'
import { invoke } from '@/services/tauri'

import type { Channel, Device } from '@/types'
```

### Error Handling

```typescript
// Use Result pattern for Tauri commands
interface Result<T, E = Error> {
  ok: boolean
  value?: T
  error?: E
}

// Async functions should handle errors explicitly
async function fetchData(): Promise<Result<Data>> {
  try {
    const data = await invoke<Data>('get_data')
    return { ok: true, value: data }
  } catch (e) {
    console.error('Failed to fetch data:', e)
    return { ok: false, error: e as Error }
  }
}
```

## Vue Components

### Single File Component Structure

```vue
<script setup lang="ts">
// 1. Imports
import { ref, computed, onMounted } from 'vue'
import { useChannelsStore } from '@/stores/channels'

// 2. Props & Emits
interface Props {
  channelId: string
  autoPlay?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  autoPlay: false
})

const emit = defineEmits<{
  play: [channelId: string]
  error: [message: string]
}>()

// 3. Stores
const channelsStore = useChannelsStore()

// 4. Refs & Reactive State
const isPlaying = ref(false)

// 5. Computed
const channel = computed(() =>
  channelsStore.channels.find(c => c.id === props.channelId)
)

// 6. Methods
function handlePlay() {
  isPlaying.value = true
  emit('play', props.channelId)
}

// 7. Lifecycle
onMounted(() => {
  if (props.autoPlay) {
    handlePlay()
  }
})
</script>

<template>
  <div class="channel-card">
    <!-- Template content -->
  </div>
</template>

<style scoped>
/* Component styles - prefer Tailwind utilities */
</style>
```

### Component Guidelines

- Use `<script setup>` syntax exclusively
- Props should have TypeScript interfaces
- Emit events with typed payloads
- Keep templates readable (extract complex logic to computed)
- Prefer composition over inheritance

## Tailwind CSS

### Class Order

Follow Tailwind's recommended order:
1. Layout (flex, grid, position)
2. Sizing (w, h, min, max)
3. Spacing (p, m, gap)
4. Typography (text, font)
5. Visual (bg, border, shadow)
6. Interactive (hover, focus, transition)

```html
<div class="flex items-center gap-4 p-4 text-sm bg-surface-1 rounded-lg hover:bg-surface-2 transition-colors">
```

### Custom Classes

Avoid `@apply` unless creating truly reusable utilities:

```css
/* Good: Component-specific styles in <style scoped> */

/* Acceptable: Truly reusable patterns */
@layer components {
  .btn-primary {
    @apply px-4 py-2 bg-indigo-500 text-white rounded-lg
           hover:bg-indigo-600 focus:ring-2 focus:ring-indigo-400
           transition-colors;
  }
}
```

## Rust (Tauri Backend)

### Naming

```rust
// Variables: snake_case
let channel_count = 0;

// Functions: snake_case
fn fetch_channels() -> Result<Vec<Channel>, Error> {}

// Types: PascalCase
struct TabloDevice {
    id: String,
    name: String,
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Tauri commands: snake_case (maps to camelCase in JS)
#[tauri::command]
async fn get_device_info(ip: String) -> Result<DeviceInfo, String> {}
```

### Error Handling

```rust
// Use thiserror for custom errors
#[derive(Debug, thiserror::Error)]
enum TabloError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),
}

// Commands return Result<T, String> for Tauri
#[tauri::command]
async fn connect_device(ip: String) -> Result<DeviceInfo, String> {
    connect_impl(ip).await.map_err(|e| e.to_string())
}
```

## Git Conventions

### Branch Names

```
feature/channel-guide
fix/playback-buffer-issue
refactor/state-management
docs/api-reference
```

### Commit Messages

```
feat: add channel guide grid view
fix: resolve HLS playback buffer underrun
refactor: extract video player composable
docs: update API documentation
chore: update dependencies
```

## File Organization

```
src/
├── assets/            # Static assets (images, fonts)
├── components/
│   ├── ui/           # Base UI components
│   │   ├── Button.vue
│   │   ├── Card.vue
│   │   └── index.ts  # Barrel export
│   ├── player/
│   └── guide/
├── composables/      # Shared logic
│   ├── useMediaPlayer.ts
│   └── useDeviceDiscovery.ts
├── services/         # External service wrappers
│   ├── tauri.ts      # Tauri invoke wrapper
│   └── tablo.ts      # Tablo API client
├── stores/           # Pinia stores
│   ├── channels.ts
│   ├── devices.ts
│   └── settings.ts
├── types/            # TypeScript definitions
│   ├── channel.ts
│   ├── device.ts
│   └── index.ts
├── views/            # Page components
│   ├── HomeView.vue
│   ├── GuideView.vue
│   └── SettingsView.vue
├── App.vue
├── main.ts
└── router.ts

src-tauri/
├── src/
│   ├── commands/     # Tauri command modules
│   ├── tablo/        # Tablo API implementation
│   ├── lib.rs
│   └── main.rs
├── capabilities/     # Permission definitions
├── icons/
├── Cargo.toml
└── tauri.conf.json
```
