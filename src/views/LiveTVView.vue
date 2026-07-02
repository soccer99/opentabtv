<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useChannelsStore, type Channel } from "@/stores/channels";
import { useDevicesStore } from "@/stores/devices";
import { useSettingsStore } from "@/stores/settings";
import { useMediaPlayer } from "@/composables/useMediaPlayer";
import { useToast } from "@/composables/useToast";

const channelsStore = useChannelsStore();
const devicesStore = useDevicesStore();
const settingsStore = useSettingsStore();
const toast = useToast();

// Video element ref
const videoRef = ref<HTMLVideoElement | null>(null);
const playerContainer = ref<HTMLElement | null>(null);

// Media player
const { state: playerState, controls, loadSource, isFullscreen } = useMediaPlayer(videoRef);

// UI state
const showControls = ref(true);
const controlsTimeout = ref<number | null>(null);

// Computed
const isConnected = computed(() => devicesStore.isConnected);
const channels = computed(() => channelsStore.sortedChannels);
const selectedChannel = computed(() => channelsStore.selectedChannel);
const isStreaming = computed(() => channelsStore.isStreaming);
const channelNumber = computed(() => channelsStore.channelNumber);

// Format channel number for display
function formatChannelNumber(channel: Channel): string {
  return channel.minor > 0 ? `${channel.major}.${channel.minor}` : `${channel.major}`;
}

// Track if playing in VLC
const playingInVlc = ref(false);

// Handle channel selection
async function selectChannel(channel: Channel) {
  const session = await channelsStore.startStream(channel);
  if (session) {
    // Check if user prefers VLC
    if (settingsStore.useVlc) {
      try {
        await invoke("open_in_vlc", {
          url: session.playlistUrl,
          vlcPath: settingsStore.vlcPath || undefined,
        });
        playingInVlc.value = true;
        toast.success(`Opening ${channel.callSign} in VLC`);
      } catch (e) {
        // Fall back to built-in player if VLC fails
        toast.error(`VLC failed: ${e}. Using built-in player.`);
        playingInVlc.value = false;
        loadSource(session.playlistUrl);
      }
    } else {
      playingInVlc.value = false;
      loadSource(session.playlistUrl);
    }
  }
}

// Auto-hide controls
function onMouseMove() {
  showControls.value = true;
  if (controlsTimeout.value) {
    clearTimeout(controlsTimeout.value);
  }
  controlsTimeout.value = window.setTimeout(() => {
    if (playerState.value.isPlaying) {
      showControls.value = false;
    }
  }, 3000);
}

// Load channels when connected
watch(
  () => devicesStore.isConnected,
  async (connected) => {
    if (connected) {
      await channelsStore.fetchChannels();
    }
  },
  { immediate: true }
);

// Cleanup on unmount
onUnmounted(() => {
  channelsStore.stopStream();
  if (controlsTimeout.value) {
    clearTimeout(controlsTimeout.value);
  }
});

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  if (!isStreaming.value) return;

  switch (e.key) {
    case " ":
    case "k":
      e.preventDefault();
      controls.togglePlay();
      break;
    case "f":
      e.preventDefault();
      controls.toggleFullscreen();
      break;
    case "m":
      e.preventDefault();
      controls.toggleMute();
      break;
    case "ArrowUp":
      e.preventDefault();
      controls.setVolume(playerState.value.volume + 0.1);
      break;
    case "ArrowDown":
      e.preventDefault();
      controls.setVolume(playerState.value.volume - 0.1);
      break;
    case "Escape":
      if (isFullscreen.value) {
        controls.exitFullscreen();
      }
      break;
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="h-full">
    <div class="mb-6">
      <h1 class="text-3xl font-bold text-text-primary">Live TV</h1>
      <p class="text-text-secondary mt-2">
        <template v-if="isConnected">
          Select a channel to start watching live television.
        </template>
        <template v-else>
          Connect to a OpenRelay device to watch live TV.
        </template>
      </p>
    </div>

    <!-- Not connected state -->
    <div
      v-if="!isConnected"
      class="flex flex-col items-center justify-center py-16"
    >
      <div class="text-center">
        <svg
          class="w-24 h-24 mx-auto text-text-muted mb-6"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
          />
        </svg>
        <p class="text-text-muted text-lg mb-4">No OpenRelay device connected</p>
        <router-link
          to="/"
          class="inline-flex items-center gap-2 px-6 py-3 bg-accent hover:bg-accent/80 text-white rounded-xl transition-colors"
        >
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          Find Devices
        </router-link>
      </div>
    </div>

    <!-- Connected state -->
    <div
      v-else
      class="grid grid-cols-1 lg:grid-cols-3 gap-6"
    >
      <!-- Video Player Area -->
      <div class="lg:col-span-2">
        <div
          ref="playerContainer"
          class="aspect-video bg-surface-1 rounded-2xl overflow-hidden relative"
          @mousemove="onMouseMove"
          @mouseleave="showControls = false"
        >
          <!-- No channel selected -->
          <div
            v-if="!isStreaming"
            class="absolute inset-0 flex items-center justify-center"
          >
            <div class="text-center">
              <svg
                class="w-16 h-16 mx-auto text-text-muted mb-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                />
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <p class="text-text-muted">Select a channel to begin playback</p>
            </div>
          </div>

          <!-- Playing in VLC indicator -->
          <div
            v-if="isStreaming && playingInVlc"
            class="absolute inset-0 flex items-center justify-center bg-surface-1"
          >
            <div class="text-center">
              <svg
                class="w-16 h-16 mx-auto text-accent mb-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                />
              </svg>
              <p class="text-text-primary font-medium mb-1">Playing in VLC</p>
              <p class="text-text-muted text-sm">{{ selectedChannel?.callSign }} - {{ channelNumber }}</p>
            </div>
          </div>

          <!-- Video element -->
          <video
            v-show="isStreaming && !playingInVlc"
            ref="videoRef"
            class="w-full h-full object-contain bg-black"
            autoplay
            playsinline
          />

          <!-- Buffering indicator -->
          <div
            v-if="playerState.isBuffering && isStreaming && !playingInVlc"
            class="absolute inset-0 flex items-center justify-center bg-black/50"
          >
            <div class="flex flex-col items-center">
              <svg
                class="w-12 h-12 text-white animate-spin"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                />
                <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              <p class="text-white mt-2">Buffering...</p>
            </div>
          </div>

          <!-- Error overlay -->
          <div
            v-if="playerState.error"
            class="absolute inset-0 flex items-center justify-center bg-black/80"
          >
            <div class="text-center">
              <svg
                class="w-12 h-12 mx-auto text-red-500 mb-2"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
              <p class="text-white">{{ playerState.error }}</p>
            </div>
          </div>

          <!-- Video controls overlay -->
          <div
            v-if="isStreaming && !playingInVlc"
            class="absolute bottom-0 left-0 right-0 transition-opacity duration-300"
            :class="showControls ? 'opacity-100' : 'opacity-0'"
          >
            <div class="bg-gradient-to-t from-black/80 to-transparent p-4 pt-12">
              <!-- Channel info -->
              <div class="flex items-center justify-between text-white mb-3">
                <div>
                  <p class="font-medium text-lg">
                    {{ selectedChannel?.callSign }}
                    <span class="text-white/60 ml-2">{{ channelNumber }}</span>
                  </p>
                  <p class="text-sm text-white/60">
                    {{ selectedChannel?.network || "Live TV" }}
                  </p>
                </div>
                <span class="px-2 py-1 bg-red-600 text-xs font-medium rounded">LIVE</span>
              </div>

              <!-- Control buttons -->
              <div class="flex items-center gap-4">
                <!-- Play/Pause -->
                <button
                  @click="controls.togglePlay"
                  class="p-2 hover:bg-white/20 rounded-lg transition-colors"
                >
                  <svg
                    v-if="playerState.isPlaying"
                    class="w-6 h-6 text-white"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
                  </svg>
                  <svg
                    v-else
                    class="w-6 h-6 text-white"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path d="M8 5v14l11-7z" />
                  </svg>
                </button>

                <!-- Volume -->
                <button
                  @click="controls.toggleMute"
                  class="p-2 hover:bg-white/20 rounded-lg transition-colors"
                >
                  <svg
                    v-if="playerState.isMuted || playerState.volume === 0"
                    class="w-6 h-6 text-white"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
                    />
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"
                    />
                  </svg>
                  <svg
                    v-else
                    class="w-6 h-6 text-white"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"
                    />
                  </svg>
                </button>

                <!-- Volume slider -->
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  :value="playerState.volume"
                  @input="controls.setVolume(($event.target as HTMLInputElement).valueAsNumber)"
                  class="w-24 accent-accent"
                />

                <div class="flex-1" />

                <!-- Fullscreen -->
                <button
                  @click="controls.toggleFullscreen"
                  class="p-2 hover:bg-white/20 rounded-lg transition-colors"
                >
                  <svg
                    v-if="!isFullscreen"
                    class="w-6 h-6 text-white"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"
                    />
                  </svg>
                  <svg
                    v-else
                    class="w-6 h-6 text-white"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25"
                    />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Channel List -->
      <div class="lg:col-span-1">
        <div class="glass p-4 max-h-[calc(100vh-12rem)] overflow-hidden flex flex-col">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-text-primary">Channels</h2>
            <span class="text-sm text-text-muted">{{ channels.length }} channels</span>
          </div>

          <!-- Loading state -->
          <div
            v-if="channelsStore.isLoading"
            class="flex items-center justify-center py-8"
          >
            <svg
              class="w-8 h-8 text-accent animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              />
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          </div>

          <!-- Error state -->
          <div
            v-else-if="channelsStore.error"
            class="text-center py-8"
          >
            <p class="text-red-400 mb-2">{{ channelsStore.error }}</p>
            <button
              @click="channelsStore.fetchChannels"
              class="text-accent hover:underline"
            >
              Try again
            </button>
          </div>

          <!-- Empty state -->
          <div
            v-else-if="!channels.length"
            class="text-center py-8"
          >
            <p class="text-text-muted">No channels found</p>
          </div>

          <!-- Channel list -->
          <div
            v-else
            class="space-y-2 overflow-y-auto flex-1"
          >
            <button
              v-for="channel in channels"
              :key="channel.id"
              @click="selectChannel(channel)"
              class="w-full p-3 rounded-xl text-left transition-colors"
              :class="[
                selectedChannel?.id === channel.id
                  ? 'bg-accent text-white'
                  : 'hover:bg-white/5',
              ]"
            >
              <div class="flex items-center gap-3">
                <div
                  class="w-12 h-10 rounded-lg flex items-center justify-center font-bold text-sm shrink-0"
                  :class="[
                    selectedChannel?.id === channel.id
                      ? 'bg-white/20 text-white'
                      : 'bg-surface-2 text-text-primary',
                  ]"
                >
                  {{ formatChannelNumber(channel) }}
                </div>
                <div class="min-w-0">
                  <p
                    class="font-medium truncate"
                    :class="[
                      selectedChannel?.id === channel.id
                        ? 'text-white'
                        : 'text-text-primary',
                    ]"
                  >
                    {{ channel.callSign }}
                  </p>
                  <p
                    class="text-sm truncate"
                    :class="[
                      selectedChannel?.id === channel.id
                        ? 'text-white/70'
                        : 'text-text-muted',
                    ]"
                  >
                    {{ channel.network || "Local" }}
                  </p>
                </div>
                <!-- Resolution badge -->
                <div
                  v-if="channel.resolution === 'hd_1080'"
                  class="ml-auto px-1.5 py-0.5 text-xs font-medium rounded"
                  :class="[
                    selectedChannel?.id === channel.id
                      ? 'bg-white/20 text-white'
                      : 'bg-accent/20 text-accent',
                  ]"
                >
                  HD
                </div>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
