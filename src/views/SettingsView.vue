<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getVersion, getTauriVersion } from "@tauri-apps/api/app";
import { useAccountStore } from "@/stores/account";
import { useDevicesStore } from "@/stores/devices";
import { useSettingsStore } from "@/stores/settings";
import { useTheme, type ThemeMode } from "@/composables/useTheme";

interface ToolInfo {
  detected: boolean;
  path: string | null;
  version: string | null;
}

interface SystemInfo {
  os: string;
  arch: string;
  osVersion: string;
}

const accountStore = useAccountStore();
const devicesStore = useDevicesStore();
const settingsStore = useSettingsStore();
const { theme, setTheme } = useTheme();

const ffmpegInfo = ref<ToolInfo | null>(null);
const vlcInfo = ref<ToolInfo | null>(null);
const appVersion = ref<string>("");
const tauriVersion = ref<string>("");
const systemInfo = ref<SystemInfo | null>(null);

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
  await Promise.all([
    detectFfmpeg(),
    detectVlc(),
    getVersion().then((v) => (appVersion.value = v)),
    getTauriVersion().then((v) => (tauriVersion.value = v)),
    invoke<SystemInfo>("get_system_info").then((info) => (systemInfo.value = info)),
  ]);
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
        Configure your OpenTabTV preferences.
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

        <!-- VLC Multi-Stream Feature Banner -->
        <div
          v-if="vlcInfo"
          class="mt-4 p-4 rounded-xl"
          :class="settingsStore.useVlc && vlcInfo.detected ? 'bg-accent/10 border border-accent/20' : 'bg-surface-2 border border-white/10'"
        >
          <div class="flex items-start gap-3">
            <svg class="w-5 h-5 text-accent flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
            <div>
              <!-- Enabled state -->
              <template v-if="settingsStore.useVlc && vlcInfo.detected">
                <p class="font-medium text-accent text-sm">Multi-Stream Mode Enabled</p>
                <p class="text-text-muted text-sm mt-1">
                  Watch multiple channels simultaneously! Each channel opens in its own VLC window.
                  Perfect for sports fans or monitoring multiple news stations at once.
                </p>
              </template>
              <!-- Installed but not enabled -->
              <template v-else-if="vlcInfo.detected && !settingsStore.useVlc">
                <p class="font-medium text-text-primary text-sm">Multi-Stream Mode Available</p>
                <p class="text-text-muted text-sm mt-1">
                  Enable VLC playback above to watch multiple channels simultaneously! Each channel opens in its own VLC window -
                  perfect for sports fans or monitoring multiple news stations at once.
                </p>
              </template>
              <!-- Not installed -->
              <template v-else>
                <p class="font-medium text-text-primary text-sm">Unlock Multi-Stream Mode</p>
                <p class="text-text-muted text-sm mt-1">
                  Install VLC to watch multiple channels simultaneously! Each channel opens in its own VLC window -
                  perfect for sports fans or monitoring multiple news stations at once.
                </p>
                <a
                  href="https://www.videolan.org/vlc/"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="inline-flex items-center gap-1.5 mt-3 text-sm text-accent hover:text-accent-hover transition-colors"
                >
                  Download VLC
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </a>
              </template>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- External Tools -->
    <section class="glass p-6 mb-6">
      <h2 class="text-xl font-semibold text-text-primary mb-4">
        External Tools
      </h2>
      <p class="text-sm text-text-muted mb-4">
        These tools are optional. The built-in player handles most streams without them.
      </p>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-secondary mb-2">
            FFmpeg Path
          </label>
          <input
            v-model="settingsStore.ffmpegPath"
            type="text"
            placeholder="/usr/local/bin/ffmpeg"
            class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent"
          />
          <p class="text-xs text-text-muted mt-2">
            For transcoding HEVC streams from 4th Gen devices. Leave empty to use system PATH.
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
            class="w-5 h-5 text-text-muted flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
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
            <div v-else>
              <span class="text-sm text-text-secondary">FFmpeg not installed</span>
              <span class="text-sm text-text-muted"> · </span>
              <a
                href="https://ffmpeg.org/download.html"
                target="_blank"
                rel="noopener noreferrer"
                class="text-sm text-accent hover:text-accent-hover transition-colors"
              >
                Download
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
            VLC Path
          </label>
          <input
            v-model="settingsStore.vlcPath"
            type="text"
            placeholder="/Applications/VLC.app/Contents/MacOS/VLC"
            class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent"
          />
          <p class="text-xs text-text-muted mt-2">
            Alternative external player. Leave empty to auto-detect.
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
            class="w-5 h-5 text-text-muted flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
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
            <div v-else>
              <span class="text-sm text-text-secondary">VLC not installed</span>
              <span class="text-sm text-text-muted"> · </span>
              <a
                href="https://www.videolan.org/vlc/"
                target="_blank"
                rel="noopener noreferrer"
                class="text-sm text-accent hover:text-accent-hover transition-colors"
              >
                Download
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
        <!-- App info -->
        <div class="flex items-start gap-4">
          <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-accent to-accent-hover flex items-center justify-center flex-shrink-0">
            <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          </div>
          <div>
            <h3 class="text-lg font-semibold text-text-primary">OpenTabTV</h3>
            <p class="text-sm text-text-secondary mt-1">
              A modern desktop app for streaming live TV and recordings from your Tablo DVR. Supports both Legacy and 4th Gen devices.
            </p>
          </div>
        </div>

        <!-- Version info -->
        <div class="grid grid-cols-2 gap-4 p-4 bg-surface-2 rounded-xl">
          <div>
            <p class="text-xs text-text-muted uppercase tracking-wide">Version</p>
            <p class="text-sm text-text-primary font-medium mt-1">{{ appVersion || '...' }}</p>
          </div>
          <div>
            <p class="text-xs text-text-muted uppercase tracking-wide">Tauri</p>
            <p class="text-sm text-text-primary font-medium mt-1">{{ tauriVersion || '...' }}</p>
          </div>
        </div>

        <!-- System info -->
        <div v-if="systemInfo" class="p-4 bg-surface-2 rounded-xl">
          <p class="text-xs text-text-muted uppercase tracking-wide mb-2">System</p>
          <p class="text-sm text-text-primary">
            {{ systemInfo.os === 'macos' ? 'macOS' : systemInfo.os === 'windows' ? 'Windows' : systemInfo.os }}
            {{ systemInfo.osVersion }}
            <span class="text-text-muted">({{ systemInfo.arch }})</span>
          </p>
        </div>

        <!-- Links -->
        <div class="flex gap-3">
          <a
            href="https://github.com/soccer99/opentabtv"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center gap-2 px-4 py-2 bg-surface-2 hover:bg-surface-3 rounded-xl text-sm text-text-primary transition-colors"
          >
            <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.17 6.839 9.49.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.604-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.464-1.11-1.464-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.087 2.91.831.092-.646.35-1.086.636-1.336-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0112 6.836c.85.004 1.705.114 2.504.336 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.167 22 16.418 22 12c0-5.523-4.477-10-10-10z" />
            </svg>
            GitHub
            <svg class="w-3 h-3 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </a>
        </div>

        <!-- Credits -->
        <div class="pt-4 border-t border-white/10">
          <p class="text-xs text-text-muted">
            Built with Tauri, Vue, and Tailwind CSS
          </p>
        </div>
      </div>
    </section>
  </div>
</template>
