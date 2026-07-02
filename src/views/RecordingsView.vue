<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRecordingsStore, type RecordingType, type RecordingDisplay } from "@/stores/recordings";
import { useDevicesStore } from "@/stores/devices";
import { useMediaPlayer } from "@/composables/useMediaPlayer";
import NoDeviceConnected from "@/components/NoDeviceConnected.vue";

const recordingsStore = useRecordingsStore();
const devicesStore = useDevicesStore();

// Video element ref
const videoRef = ref<HTMLVideoElement | null>(null);

// Media player
const { state: playerState, controls, loadSource, isFullscreen } = useMediaPlayer(videoRef);

// UI state
const showControls = ref(true);
const controlsTimeout = ref<number | null>(null);
const showPlayer = ref(false);

// Computed
const isConnected = computed(() => devicesStore.isConnected);
const recordings = computed(() => recordingsStore.filteredRecordings);
const selectedRecording = computed(() => recordingsStore.selectedRecording);
const activeTab = computed(() => recordingsStore.filterType);

// Progress percentage (safe from division by zero)
const progressPercent = computed(() => {
  if (!playerState.value.duration || playerState.value.duration === 0) {
    return 0;
  }
  return (playerState.value.currentTime / playerState.value.duration) * 100;
});

// Filter tabs
const tabs: Array<{ key: RecordingType | "all"; label: string }> = [
  { key: "all", label: "All" },
  { key: "movie", label: "Movies" },
  { key: "series", label: "Series" },
  { key: "sports", label: "Sports" },
];

// Format duration for display (seconds to readable format)
function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes} min`;
}

// Format date for display
function formatDate(date: Date): string {
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

// Format episode info
function formatEpisode(recording: RecordingDisplay): string {
  if (recording.episode) {
    const { seasonNumber, number, title } = recording.episode;
    let text = `S${seasonNumber}E${number}`;
    if (title) {
      text += ` - ${title}`;
    }
    return text;
  }
  return formatDuration(recording.duration);
}

// Format file size
function formatSize(bytes: number): string {
  if (bytes === 0) return "";
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) {
    return `${gb.toFixed(1)} GB`;
  }
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
}

// Handle tab change
function setActiveTab(tab: RecordingType | "all") {
  recordingsStore.setFilter(tab);
}

// Handle recording selection and playback
async function playRecording(recording: RecordingDisplay) {
  showPlayer.value = true;
  const session = await recordingsStore.playRecording(recording);
  if (session) {
    loadSource(session.playlistUrl);
  }
}

// Close player
function closePlayer() {
  recordingsStore.stopPlayback();
  showPlayer.value = false;
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

// Format time for display (seconds to mm:ss or hh:mm:ss)
function formatPlaybackTime(seconds: number): string {
  if (!seconds || !isFinite(seconds)) return "0:00";
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  if (hrs > 0) {
    return `${hrs}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  if (!showPlayer.value) return;

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
    case "ArrowLeft":
      e.preventDefault();
      controls.seek(playerState.value.currentTime - 10);
      break;
    case "ArrowRight":
      e.preventDefault();
      controls.seek(playerState.value.currentTime + 10);
      break;
    case "Escape":
      e.preventDefault();
      if (isFullscreen.value) {
        controls.exitFullscreen();
      } else {
        closePlayer();
      }
      break;
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

// Cleanup on unmount
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  recordingsStore.stopPlayback();
  if (controlsTimeout.value) {
    clearTimeout(controlsTimeout.value);
  }
});
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-3xl font-bold text-text-primary">Recordings</h1>
      <p class="text-text-secondary mt-2">
        <template v-if="isConnected">
          Browse and manage your recorded content.
          <span v-if="recordingsStore.totalCount > 0" class="text-text-muted">
            {{ recordingsStore.totalCount }} recordings
            <template v-if="recordingsStore.unwatchedCount > 0">
              ({{ recordingsStore.unwatchedCount }} unwatched)
            </template>
          </span>
        </template>
        <template v-else>
          Connect to a Tablo device to view your recordings.
        </template>
      </p>
    </div>

    <!-- Not connected state -->
    <NoDeviceConnected
      v-if="!isConnected"
      title="No Device Connected"
      description="Connect to a Tablo device to browse and watch your recordings."
    />

    <!-- Connected state -->
    <template v-else>
      <!-- Video Player Modal -->
      <div
        v-if="showPlayer"
        class="fixed inset-0 z-50 bg-black/90 flex items-center justify-center"
      >
        <!-- Close button (positioned relative to viewport) -->
        <button
          @click="closePlayer"
          class="absolute top-4 right-4 z-10 p-2 text-white/60 hover:text-white transition-colors"
        >
          <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        <div class="w-full max-w-6xl mx-4">
          <div
            class="aspect-video bg-black rounded-2xl overflow-hidden relative"
            @mousemove="onMouseMove"
            @mouseleave="showControls = false"
          >
            <!-- Video element -->
            <video
              ref="videoRef"
              class="w-full h-full object-contain"
              autoplay
              playsinline
            />

            <!-- Buffering indicator -->
            <div
              v-if="playerState.isBuffering"
              class="absolute inset-0 flex items-center justify-center bg-black/50"
            >
              <div class="flex flex-col items-center">
                <svg class="w-12 h-12 text-white animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
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
                <svg class="w-12 h-12 mx-auto text-red-500 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <p class="text-white">{{ playerState.error }}</p>
              </div>
            </div>

            <!-- Video controls overlay -->
            <div
              class="absolute bottom-0 left-0 right-0 transition-opacity duration-300"
              :class="showControls ? 'opacity-100' : 'opacity-0'"
            >
              <div class="bg-gradient-to-t from-black/80 to-transparent p-4 pt-12">
                <!-- Recording info -->
                <div class="flex items-center justify-between text-white mb-3">
                  <div>
                    <p class="font-medium text-lg">{{ selectedRecording?.title }}</p>
                    <p class="text-sm text-white/60">
                      {{ selectedRecording?.episode ? formatEpisode(selectedRecording) : formatDuration(selectedRecording?.duration || 0) }}
                    </p>
                  </div>
                  <span class="px-2 py-1 bg-accent text-xs font-medium rounded uppercase">
                    {{ selectedRecording?.recordingType }}
                  </span>
                </div>

                <!-- Progress bar -->
                <div class="mb-3">
                  <div class="flex items-center gap-3 text-sm text-white/60 mb-1">
                    <span>{{ formatPlaybackTime(playerState.currentTime) }}</span>
                    <div class="flex-1 h-1 bg-white/20 rounded-full overflow-hidden">
                      <div
                        class="h-full bg-accent transition-all"
                        :style="{ width: `${progressPercent}%` }"
                      />
                    </div>
                    <span>{{ formatPlaybackTime(playerState.duration) }}</span>
                  </div>
                </div>

                <!-- Control buttons -->
                <div class="flex items-center gap-4">
                  <!-- Play/Pause -->
                  <button @click="controls.togglePlay" class="p-2 hover:bg-white/20 rounded-lg transition-colors">
                    <svg v-if="playerState.isPlaying" class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
                    </svg>
                    <svg v-else class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M8 5v14l11-7z" />
                    </svg>
                  </button>

                  <!-- Volume -->
                  <button @click="controls.toggleMute" class="p-2 hover:bg-white/20 rounded-lg transition-colors">
                    <svg v-if="playerState.isMuted || playerState.volume === 0" class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
                    </svg>
                    <svg v-else class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
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
                  <button @click="controls.toggleFullscreen" class="p-2 hover:bg-white/20 rounded-lg transition-colors">
                    <svg v-if="!isFullscreen" class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                    </svg>
                    <svg v-else class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4.5M9 9H4.5M9 9L3.75 3.75M9 15v4.5M9 15H4.5M9 15l-5.25 5.25M15 9h4.5M15 9V4.5M15 9l5.25-5.25M15 15h4.5M15 15v4.5m0-4.5l5.25 5.25" />
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Keyboard shortcuts hint -->
        <div class="absolute bottom-4 left-4 text-white/40 text-xs hidden md:block">
          Space: Play/Pause | F: Fullscreen | M: Mute | Arrows: Seek/Volume | Esc: Close
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex gap-2 mb-6">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          @click="setActiveTab(tab.key)"
          class="px-4 py-2 rounded-xl font-medium transition-colors"
          :class="[
            activeTab === tab.key
              ? 'bg-accent text-white'
              : 'glass text-text-secondary hover:text-text-primary',
          ]"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- Loading state -->
      <div
        v-if="recordingsStore.isLoading"
        class="flex items-center justify-center py-16"
      >
        <svg class="w-12 h-12 text-accent animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      </div>

      <!-- Error state -->
      <div
        v-else-if="recordingsStore.error"
        class="text-center py-12"
      >
        <svg class="w-16 h-16 mx-auto text-red-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="text-red-400 mb-2">{{ recordingsStore.error }}</p>
        <button
          @click="recordingsStore.fetchRecordings"
          class="text-accent hover:underline"
        >
          Try again
        </button>
      </div>

      <!-- Recordings Grid -->
      <div
        v-else-if="recordings.length > 0"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
      >
        <div
          v-for="recording in recordings"
          :key="recording.id"
          @click="playRecording(recording)"
          class="glass overflow-hidden group cursor-pointer"
        >
          <!-- Thumbnail -->
          <div class="aspect-video bg-surface-2 relative">
            <div
              class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity bg-black/50"
            >
              <button class="p-4 rounded-full bg-accent hover:bg-accent-hover transition-colors">
                <svg
                  class="w-8 h-8 text-white"
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
                </svg>
              </button>
            </div>

            <!-- Type badge -->
            <div
              class="absolute top-2 right-2 px-2 py-1 rounded text-xs font-medium capitalize"
              :class="{
                'bg-category-movie': recording.recordingType === 'movie',
                'bg-category-series': recording.recordingType === 'series',
                'bg-category-sports': recording.recordingType === 'sports',
                'bg-surface-1': !['movie', 'series', 'sports'].includes(recording.recordingType),
              }"
            >
              {{ recording.recordingType }}
            </div>

            <!-- Recording state badge -->
            <div
              v-if="recording.videoDetails.state === 'recording'"
              class="absolute top-2 left-2 px-2 py-1 bg-red-600 rounded text-xs font-medium flex items-center gap-1"
            >
              <span class="w-2 h-2 bg-white rounded-full animate-pulse" />
              REC
            </div>

            <!-- Watched indicator -->
            <div
              v-if="recording.userInfo.watched"
              class="absolute bottom-2 left-2 px-2 py-1 bg-black/60 rounded text-xs text-white/80"
            >
              Watched
            </div>

            <!-- Protected indicator -->
            <div
              v-if="recording.userInfo.protected"
              class="absolute bottom-2 right-2"
            >
              <svg class="w-4 h-4 text-white/80" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" />
              </svg>
            </div>
          </div>

          <!-- Info -->
          <div class="p-4">
            <h3 class="font-medium text-text-primary truncate">
              {{ recording.title }}
            </h3>
            <div class="flex items-center justify-between mt-2">
              <span class="text-sm text-text-muted truncate flex-1">
                {{ formatEpisode(recording) }}
              </span>
              <span class="text-sm text-text-muted ml-2 whitespace-nowrap">
                {{ formatDate(recording.recordedAt) }}
              </span>
            </div>
            <div
              v-if="recording.size > 0"
              class="text-xs text-text-muted mt-1"
            >
              {{ formatSize(recording.size) }}
            </div>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div
        v-else
        class="text-center py-12"
      >
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
            d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
          />
        </svg>
        <p class="text-text-muted">
          <template v-if="activeTab === 'all'">
            No recordings found
          </template>
          <template v-else>
            No {{ activeTab }} recordings found
          </template>
        </p>
      </div>
    </template>
  </div>
</template>
