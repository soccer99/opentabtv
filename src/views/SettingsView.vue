<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAccountStore } from "@/stores/account";
import { useDevicesStore } from "@/stores/devices";
import { useSettingsStore } from "@/stores/settings";
import { useTheme, type ThemeMode } from "@/composables/useTheme";

interface ToolInfo {
  detected: boolean;
  path: string | null;
  version: string | null;
}

const accountStore = useAccountStore();
const devicesStore = useDevicesStore();
const settingsStore = useSettingsStore();
const { theme, setTheme } = useTheme();

const ffmpegInfo = ref<ToolInfo | null>(null);
const vlcInfo = ref<ToolInfo | null>(null);

async function detectFfmpeg() {
  const customPath = settingsStore.ffmpegPath.trim() || undefined;
  ffmpegInfo.value = await invoke<ToolInfo>("detect_ffmpeg", {
    customPath,
  });
}

async function detectVlc() {
  const customPath = settingsStore.vlcPath.trim() || undefined;
  vlcInfo.value = await invoke<ToolInfo>("detect_vlc", {
    customPath,
  });
}

// Re-detect when custom paths change (debounced)
let ffmpegDebounce: ReturnType<typeof setTimeout> | null = null;
let vlcDebounce: ReturnType<typeof setTimeout> | null = null;

watch(() => settingsStore.ffmpegPath, () => {
  if (ffmpegDebounce) clearTimeout(ffmpegDebounce);
  ffmpegDebounce = setTimeout(detectFfmpeg, 500);
});

watch(() => settingsStore.vlcPath, () => {
  if (vlcDebounce) clearTimeout(vlcDebounce);
  vlcDebounce = setTimeout(detectVlc, 500);
});

// Disable VLC playback option when VLC is not detected
watch(vlcInfo, (info) => {
  if (info && !info.detected) {
    settingsStore.useVlc = false;
  }
});

function handleThemeChange(newTheme: ThemeMode) {
  setTheme(newTheme);
}

async function handleClearCredentials() {
  await accountStore.clearCredentials();
  await accountStore.logout();
}

async function handleDisconnect() {
  await devicesStore.disconnect();
}

onMounted(async () => {
  await accountStore.checkSavedCredentials();
  await Promise.all([detectFfmpeg(), detectVlc()]);
});

onUnmounted(() => {
  if (ffmpegDebounce) clearTimeout(ffmpegDebounce);
  if (vlcDebounce) clearTimeout(vlcDebounce);
});
</script>

<template>
  <div class="max-w-2xl">
    <div class="mb-6">
      <h1 class="text-3xl font-bold text-text-primary">Settings</h1>
      <p class="text-text-secondary mt-2">
        Configure your Tablo app preferences.
      </p>
    </div>

    <!-- Appearance -->
    <section class="glass p-6 mb-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">Appearance</h2>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-2">
            Theme
          </label>
          <div class="flex gap-2">
            <button
              v-for="option in (['dark', 'light', 'system'] as const)"
              :key="option"
              @click="handleThemeChange(option)"
              class="px-4 py-2 rounded-xl capitalize transition-colors"
              :class="[
                theme === option
                  ? 'bg-accent text-white'
                  : 'bg-surface-2 text-text-secondary hover:text-text-primary',
              ]"
            >
              {{ option }}
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Playback -->
    <section class="glass p-6 mb-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">Playback</h2>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-2">
            Video Quality
          </label>
          <select
            v-model="settingsStore.preferredQuality"
            class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary focus:border-accent focus:ring-1 focus:ring-accent"
          >
            <option value="auto">Auto (Recommended)</option>
            <option value="720p">720p</option>
            <option value="1080p">1080p</option>
          </select>
        </div>

        <div class="flex items-center justify-between">
          <div>
            <p
              class="font-medium"
              :class="vlcInfo?.detected ? 'text-text-primary' : 'text-text-muted'"
            >
              Use VLC for playback
            </p>
            <p class="text-sm text-text-muted">
              <template v-if="vlcInfo?.detected">
                Open streams in VLC instead of built-in player
              </template>
              <template v-else>
                VLC not installed
              </template>
            </p>
          </div>
          <button
            @click="vlcInfo?.detected && (settingsStore.useVlc = !settingsStore.useVlc)"
            :disabled="!vlcInfo?.detected"
            class="relative w-12 h-6 rounded-full transition-colors"
            :class="[
              !vlcInfo?.detected
                ? 'bg-surface-3 opacity-50 cursor-not-allowed'
                : settingsStore.useVlc
                  ? 'bg-accent'
                  : 'bg-surface-3'
            ]"
          >
            <span
              class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform"
              :class="[settingsStore.useVlc && vlcInfo?.detected ? 'translate-x-6' : '']"
            />
          </button>
        </div>
      </div>
    </section>

    <!-- External Tools -->
    <section class="glass p-6 mb-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">
        External Tools
      </h2>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-2">
            FFmpeg Path (optional)
          </label>
          <input
            v-model="settingsStore.ffmpegPath"
            type="text"
            placeholder="/usr/local/bin/ffmpeg"
            class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent"
          />
          <p class="text-xs text-text-muted mt-2">
            Leave empty to use system PATH
          </p>
        </div>

        <!-- FFmpeg status -->
        <div
          v-if="ffmpegInfo"
          class="flex items-center gap-2 p-3 bg-surface-2 rounded-xl"
        >
          <svg
            v-if="ffmpegInfo.detected"
            class="w-5 h-5 text-success flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
          <svg
            v-else
            class="w-5 h-5 text-warning flex-shrink-0"
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
          <div class="min-w-0 flex-1">
            <span
              v-if="ffmpegInfo.detected"
              class="text-sm text-text-primary"
            >
              FFmpeg detected
              <span v-if="ffmpegInfo.version" class="text-text-muted">
                ({{ ffmpegInfo.version }})
              </span>
            </span>
            <div v-else class="flex items-center gap-2 flex-wrap">
              <span class="text-sm text-warning">FFmpeg not found</span>
              <a
                href="https://ffmpeg.org/download.html"
                target="_blank"
                rel="noopener noreferrer"
                class="text-sm text-accent hover:text-accent-hover transition-colors inline-flex items-center gap-1"
              >
                Download FFmpeg
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
              </a>
            </div>
            <p
              v-if="ffmpegInfo.detected && ffmpegInfo.path"
              class="text-xs text-text-muted truncate"
              :title="ffmpegInfo.path"
            >
              {{ ffmpegInfo.path }}
            </p>
          </div>
        </div>
        <div v-else class="flex items-center gap-2 p-3 bg-surface-2 rounded-xl">
          <div class="w-5 h-5 border-2 border-text-muted border-t-transparent rounded-full animate-spin" />
          <span class="text-sm text-text-muted">Detecting FFmpeg...</span>
        </div>

        <!-- Divider -->
        <div class="border-t border-white/10 my-4" />

        <!-- VLC -->
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-2">
            VLC Path (optional)
          </label>
          <input
            v-model="settingsStore.vlcPath"
            type="text"
            placeholder="/Applications/VLC.app/Contents/MacOS/VLC"
            class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent"
          />
          <p class="text-xs text-text-muted mt-2">
            Leave empty to auto-detect
          </p>
        </div>

        <!-- VLC status -->
        <div
          v-if="vlcInfo"
          class="flex items-center gap-2 p-3 bg-surface-2 rounded-xl"
        >
          <svg
            v-if="vlcInfo.detected"
            class="w-5 h-5 text-success flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
          <svg
            v-else
            class="w-5 h-5 text-warning flex-shrink-0"
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
          <div class="min-w-0 flex-1">
            <span
              v-if="vlcInfo.detected"
              class="text-sm text-text-primary"
            >
              VLC detected
              <span v-if="vlcInfo.version" class="text-text-muted">
                ({{ vlcInfo.version }})
              </span>
            </span>
            <div v-else class="flex items-center gap-2 flex-wrap">
              <span class="text-sm text-warning">VLC not found</span>
              <a
                href="https://www.videolan.org/vlc/"
                target="_blank"
                rel="noopener noreferrer"
                class="text-sm text-accent hover:text-accent-hover transition-colors inline-flex items-center gap-1"
              >
                Download VLC
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
              </a>
            </div>
            <p
              v-if="vlcInfo.detected && vlcInfo.path"
              class="text-xs text-text-muted truncate"
              :title="vlcInfo.path"
            >
              {{ vlcInfo.path }}
            </p>
          </div>
        </div>
        <div v-else class="flex items-center gap-2 p-3 bg-surface-2 rounded-xl">
          <div class="w-5 h-5 border-2 border-text-muted border-t-transparent rounded-full animate-spin" />
          <span class="text-sm text-text-muted">Detecting VLC...</span>
        </div>
      </div>
    </section>

    <!-- Account -->
    <section class="glass p-6 mb-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">Account</h2>

      <div class="space-y-4">
        <!-- Logged in state -->
        <div v-if="accountStore.isLoggedIn" class="p-4 bg-surface-2 rounded-xl">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full bg-accent/20 flex items-center justify-center">
                <svg class="w-5 h-5 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
              </div>
              <div>
                <p class="font-medium text-text-primary">{{ accountStore.email }}</p>
                <p class="text-sm text-text-muted">4th Gen account</p>
              </div>
            </div>
            <span class="px-2 py-1 text-xs font-medium bg-success/20 text-success rounded-full">
              Connected
            </span>
          </div>
        </div>

        <!-- Not logged in state -->
        <div v-else class="p-4 bg-surface-2 rounded-xl">
          <p class="text-text-secondary text-sm">
            Sign in with your Tablo account to access 4th Gen devices.
          </p>
          <router-link
            to="/"
            class="inline-block mt-3 px-4 py-2 bg-accent hover:bg-accent-hover text-white text-sm rounded-xl font-medium transition-colors"
          >
            Sign In
          </router-link>
        </div>

        <!-- Saved credentials -->
        <div v-if="accountStore.hasSavedCredentials" class="flex items-center justify-between p-3 bg-surface-2 rounded-xl">
          <div>
            <p class="font-medium text-text-primary">Saved Credentials</p>
            <p class="text-sm text-text-muted">Auto-login enabled</p>
          </div>
          <button
            @click="handleClearCredentials"
            class="px-3 py-1.5 text-sm text-error hover:bg-error/10 rounded-lg transition-colors"
          >
            Clear
          </button>
        </div>
      </div>
    </section>

    <!-- Device -->
    <section class="glass p-6 mb-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">Device</h2>

      <div class="space-y-4">
        <!-- Active device -->
        <div v-if="devicesStore.activeDevice" class="p-4 bg-surface-2 rounded-xl">
          <div class="flex items-center justify-between">
            <div>
              <div class="flex items-center gap-2">
                <p class="font-medium text-text-primary">{{ devicesStore.activeDevice.name }}</p>
                <span
                  class="px-2 py-0.5 text-xs font-medium rounded-full"
                  :class="devicesStore.activeDevice.generation === 'gen4' ? 'bg-accent/20 text-accent' : 'bg-info/20 text-info'"
                >
                  {{ devicesStore.activeDevice.generation === 'gen4' ? '4th Gen' : 'Legacy' }}
                </span>
              </div>
              <p class="text-sm text-text-muted">{{ devicesStore.activeDevice.localIp }}</p>
            </div>
            <button
              @click="handleDisconnect"
              class="px-3 py-1.5 text-sm text-error hover:bg-error/10 rounded-lg transition-colors"
            >
              Disconnect
            </button>
          </div>
        </div>

        <!-- No active device -->
        <div v-else class="p-4 bg-surface-2 rounded-xl">
          <p class="text-text-secondary text-sm">No device connected</p>
          <router-link
            to="/"
            class="inline-block mt-3 px-4 py-2 bg-surface-3 hover:bg-surface-2 text-text-primary text-sm rounded-xl font-medium transition-colors"
          >
            Connect Device
          </router-link>
        </div>
      </div>
    </section>

    <!-- About -->
    <section class="glass p-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">About</h2>

      <div class="space-y-4">
        <div class="pt-4 border-t border-white/10">
          <p class="text-sm text-text-muted">
            Tablo App v0.1.0 • Built with Tauri + Vue
          </p>
          <p class="text-xs text-text-muted mt-2">
            Supports Legacy (pre-4th Gen) and 4th Gen Tablo devices
          </p>
        </div>
      </div>
    </section>
  </div>
</template>
