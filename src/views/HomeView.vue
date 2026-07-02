<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useDevicesStore } from "@/stores/devices";
import { useAccountStore } from "@/stores/account";
import { useRecordingsStore } from "@/stores/recordings";
import { useChannelsStore, type Channel } from "@/stores/channels";
import { useGuideStore } from "@/stores/guide";
import { useDevicePreferencesStore } from "@/stores/devicePreferences";
import { useToast } from "@/composables/useToast";
import { formatChannelNumber, formatRelativeDate } from "@/utils/format";
import type { TabloDevice } from "@/types";

const router = useRouter();
const devicesStore = useDevicesStore();
const accountStore = useAccountStore();
const recordingsStore = useRecordingsStore();
const channelsStore = useChannelsStore();
const guideStore = useGuideStore();
const devicePreferencesStore = useDevicePreferencesStore();
const toast = useToast();

const manualIp = ref("");
const isScanning = ref(false);
const connectingDeviceId = ref<string | null>(null);
const isConnectingManual = ref(false);

// Login form
const showLoginForm = ref(false);
const loginEmail = ref("");
const loginPassword = ref("");
const rememberCredentials = ref(false);
const isLoggingIn = ref(false);
const loginError = ref<string | null>(null);

// Computed
const isConnected = computed(() => devicesStore.isConnected);
const activeDevice = computed(() => devicesStore.activeDevice);

// Favorite channels with current program info
const favoriteChannels = computed(() => {
  const deviceId = activeDevice.value?.id;
  if (!deviceId) return [];
  const favoriteIds = devicePreferencesStore.getFavorites(deviceId);
  return channelsStore.sortedChannels
    .filter((c) => favoriteIds.includes(c.id))
    .map((channel) => ({
      channel,
      nowPlaying: guideStore.getNowPlaying(channel.path),
    }));
});

const hasFavorites = computed(() => favoriteChannels.value.length > 0);

// Recent channels
const recentChannels = computed(() => {
  const deviceId = activeDevice.value?.id;
  if (!deviceId) return [];
  const recentItems = devicePreferencesStore.getRecentChannels(deviceId);
  return recentItems
    .map((item) => channelsStore.sortedChannels.find((c) => c.id === item.id))
    .filter((c): c is Channel => c !== undefined);
});

const hasRecentChannels = computed(() => recentChannels.value.length > 0);

// Recent recordings (sorted by recordedAt, most recent first)
const recentRecordings = computed(() => {
  return [...recordingsStore.recordings]
    .sort((a, b) => b.recordedAt.getTime() - a.recordedAt.getTime())
    .slice(0, 3);
});

const hasRecentRecordings = computed(() => recentRecordings.value.length > 0);

// Navigate to Live TV with specific channel
async function goToChannel(channel: Channel) {
  await channelsStore.startStream(channel);
  router.push("/live");
}

// Navigate to Recordings and play specific recording
async function goToRecording(recording: typeof recentRecordings.value[0]) {
  await recordingsStore.playRecording(recording);
  router.push("/recordings");
}

async function scanForDevices() {
  isScanning.value = true;
  try {
    await devicesStore.discoverDevices();

    if (devicesStore.error) {
      toast.error(devicesStore.error);
    } else if (devicesStore.devices.length === 0) {
      toast.warning("No Tablo devices found on network");
    } else {
      // Register discovered devices
      for (const device of devicesStore.devices) {
        devicesStore.registerDevice(device);
      }
      const count = devicesStore.devices.length;
      toast.success(`Found ${count} device${count === 1 ? "" : "s"}`);
    }
  } finally {
    isScanning.value = false;
  }
}

async function connectManual() {
  if (!manualIp.value) return;

  isConnectingManual.value = true;
  try {
    await devicesStore.connectByIp(manualIp.value);
    if (devicesStore.isConnected && devicesStore.activeDevice) {
      // Register the device
      devicesStore.registerDevice(devicesStore.activeDevice);
      toast.success(`Connected to ${devicesStore.activeDevice.name}`);
      manualIp.value = "";
    } else if (devicesStore.error) {
      toast.error(devicesStore.error);
    }
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    toast.error(errorMsg);
  } finally {
    isConnectingManual.value = false;
  }
}

async function handleLogin() {
  if (!loginEmail.value || !loginPassword.value) return;

  isLoggingIn.value = true;
  loginError.value = null;

  try {
    const success = await accountStore.login(loginEmail.value, loginPassword.value);
    if (success) {
      if (rememberCredentials.value) {
        await accountStore.saveCredentials(loginEmail.value, loginPassword.value);
      }
      showLoginForm.value = false;
      // Discover and register cloud devices
      const cloudDevices = await accountStore.discoverCloudDevices();
      devicesStore.addCloudDevices(cloudDevices);
      for (const device of cloudDevices) {
        devicesStore.registerDevice(device);
      }
    } else {
      loginError.value = accountStore.error || "Login failed";
    }
  } catch (e) {
    loginError.value = e instanceof Error ? e.message : String(e);
  } finally {
    isLoggingIn.value = false;
  }
}

async function handleLogout() {
  await accountStore.logout();
}

function getDeviceBadge(device: TabloDevice): { text: string; class: string } {
  if (device.generation === "gen4") {
    return { text: "4th Gen", class: "bg-accent/20 text-accent" };
  }
  return { text: "Legacy", class: "bg-info/20 text-info" };
}

async function handleDeviceConnect(device: TabloDevice) {
  connectingDeviceId.value = device.id;
  try {
    if (device.generation === "gen4") {
      await devicesStore.connectGen4Device(device);
    } else {
      await devicesStore.connectToDevice(device);
    }
    if (devicesStore.isConnected) {
      // Register the device
      devicesStore.registerDevice(device);
      toast.success(`Connected to ${device.name}`);
    } else if (devicesStore.error) {
      toast.error(devicesStore.error);
    }
  } catch (e) {
    const errorMsg = e instanceof Error ? e.message : String(e);
    toast.error(errorMsg);
  } finally {
    connectingDeviceId.value = null;
  }
}

async function disconnect() {
  await devicesStore.disconnect();
  toast.info("Disconnected from device");
}

onMounted(async () => {
  // Check for saved credentials and auto-login
  const hasSaved = await accountStore.checkSavedCredentials();
  if (hasSaved) {
    const success = await accountStore.autoLogin();
    if (success) {
      const cloudDevices = await accountStore.discoverCloudDevices();
      devicesStore.addCloudDevices(cloudDevices);
      for (const device of cloudDevices) {
        devicesStore.registerDevice(device);
      }
    }
  }

  // If connected, load data for dashboard
  if (devicesStore.isConnected) {
    await Promise.all([
      recordingsStore.fetchRecordings(),
      channelsStore.fetchChannels(),
      guideStore.fetchAirings(),
      devicePreferencesStore.initialize(),
    ]);
  }
});
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <!-- ============================================ -->
    <!-- CONNECTED STATE: Dashboard -->
    <!-- ============================================ -->
    <template v-if="isConnected && activeDevice">
      <!-- Header -->
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-text-primary">Dashboard</h1>
        <p class="text-text-secondary mt-2">
          Welcome back. Here's what's happening with your Tablo.
        </p>
      </div>

      <!-- Connected Device Card -->
      <div class="glass p-6 mb-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <!-- Device icon -->
            <div class="w-14 h-14 rounded-2xl bg-success/20 flex items-center justify-center">
              <svg class="w-7 h-7 text-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <!-- Device info -->
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl font-semibold text-text-primary">{{ activeDevice.name }}</h2>
                <span
                  class="px-2 py-0.5 text-xs font-medium rounded-full"
                  :class="getDeviceBadge(activeDevice).class"
                >
                  {{ getDeviceBadge(activeDevice).text }}
                </span>
                <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-success/20 text-success">
                  Connected
                </span>
              </div>
              <p class="text-text-muted mt-1">
                {{ activeDevice.localIp }}
                <span v-if="activeDevice.model" class="ml-2">{{ activeDevice.model }}</span>
                <span v-if="activeDevice.tuners" class="ml-2">{{ activeDevice.tuners }} tuners</span>
              </p>
            </div>
          </div>
          <!-- Actions -->
          <div class="flex items-center gap-2">
            <router-link
              to="/devices"
              class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-white/5 rounded-lg transition-colors"
            >
              Switch Device
            </router-link>
            <button
              @click="disconnect"
              class="px-4 py-2 text-sm bg-surface-2 hover:bg-surface-3 text-text-primary rounded-lg transition-colors"
            >
              Disconnect
            </button>
          </div>
        </div>
      </div>

      <!-- Favorite Channels -->
      <div v-if="hasFavorites" class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-text-primary">Favorite Channels</h2>
          <router-link
            to="/live"
            class="text-sm text-accent hover:text-accent-hover transition-colors"
          >
            View All
          </router-link>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <button
            v-for="{ channel, nowPlaying } in favoriteChannels"
            :key="channel.id"
            @click="goToChannel(channel)"
            class="glass p-4 text-left hover:bg-white/10 transition-colors group"
          >
            <div class="flex items-center gap-3 mb-2">
              <div class="w-12 h-10 rounded-lg bg-accent/20 flex items-center justify-center font-bold text-sm text-accent group-hover:bg-accent/30 transition-colors">
                {{ formatChannelNumber(channel) }}
              </div>
              <div class="min-w-0 flex-1">
                <p class="font-medium text-text-primary truncate">{{ channel.callSign }}</p>
                <p class="text-xs text-text-muted truncate">{{ channel.network || "Local" }}</p>
              </div>
              <!-- Heart icon -->
              <svg class="w-4 h-4 text-accent" viewBox="0 0 24 24">
                <path
                  fill="currentColor"
                  d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
                />
              </svg>
            </div>
            <!-- Now Playing -->
            <div v-if="nowPlaying" class="mt-2 pt-2 border-t border-white/10">
              <p class="text-xs text-text-muted mb-1">Now Playing</p>
              <p class="text-sm text-text-primary truncate">{{ nowPlaying.title }}</p>
            </div>
            <div v-else class="mt-2 pt-2 border-t border-white/10">
              <p class="text-xs text-text-muted">No guide data</p>
            </div>
          </button>
        </div>
      </div>

      <!-- Recent Channels -->
      <div v-if="hasRecentChannels" class="mb-8">
        <h2 class="text-xl font-semibold text-text-primary mb-4">Recently Watched</h2>
        <div class="flex gap-4 overflow-x-auto pb-2">
          <button
            v-for="channel in recentChannels"
            :key="channel.id"
            @click="goToChannel(channel)"
            class="glass p-4 flex items-center gap-3 hover:bg-white/10 transition-colors shrink-0"
          >
            <div class="w-12 h-10 rounded-lg bg-surface-2 flex items-center justify-center font-bold text-sm text-text-primary">
              {{ formatChannelNumber(channel) }}
            </div>
            <div class="min-w-0">
              <p class="font-medium text-text-primary">{{ channel.callSign }}</p>
              <p class="text-xs text-text-muted">{{ channel.network || "Local" }}</p>
            </div>
          </button>
        </div>
      </div>

      <!-- Recent Recordings -->
      <div v-if="hasRecentRecordings" class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-text-primary">Recent Recordings</h2>
          <router-link
            to="/recordings"
            class="text-sm text-accent hover:text-accent-hover transition-colors"
          >
            View All
          </router-link>
        </div>
        <div class="flex gap-4 overflow-x-auto pb-2">
          <button
            v-for="recording in recentRecordings"
            :key="recording.id"
            @click="goToRecording(recording)"
            class="glass p-4 w-64 shrink-0 text-left hover:bg-white/10 transition-colors"
          >
            <div class="aspect-video bg-surface-2 rounded-lg mb-3 flex items-center justify-center">
              <svg class="w-10 h-10 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <p class="font-medium text-text-primary truncate">{{ recording.title }}</p>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-text-muted">{{ formatRelativeDate(recording.recordedAt) }}</span>
              <span
                v-if="!recording.userInfo.watched"
                class="px-1.5 py-0.5 text-xs font-medium rounded bg-accent/20 text-accent"
              >
                New
              </span>
            </div>
          </button>
        </div>
      </div>

      <!-- Quick Actions -->
      <h2 class="text-xl font-semibold text-text-primary mb-4">Quick Actions</h2>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <router-link
          to="/live"
          class="glass p-6 text-center hover:bg-white/10 transition-colors group"
        >
          <div class="w-14 h-14 mx-auto mb-3 rounded-2xl bg-accent/20 flex items-center justify-center group-hover:bg-accent/30 transition-colors">
            <svg class="w-7 h-7 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <p class="font-medium text-text-primary">Live TV</p>
          <p class="text-xs text-text-muted mt-1">Watch now</p>
        </router-link>

        <router-link
          to="/guide"
          class="glass p-6 text-center hover:bg-white/10 transition-colors group"
        >
          <div class="w-14 h-14 mx-auto mb-3 rounded-2xl bg-info/20 flex items-center justify-center group-hover:bg-info/30 transition-colors">
            <svg class="w-7 h-7 text-info" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </div>
          <p class="font-medium text-text-primary">Guide</p>
          <p class="text-xs text-text-muted mt-1">What's on</p>
        </router-link>

        <router-link
          to="/recordings"
          class="glass p-6 text-center hover:bg-white/10 transition-colors group"
        >
          <div class="w-14 h-14 mx-auto mb-3 rounded-2xl bg-success/20 flex items-center justify-center group-hover:bg-success/30 transition-colors">
            <svg class="w-7 h-7 text-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
            </svg>
          </div>
          <p class="font-medium text-text-primary">Recordings</p>
          <p class="text-xs text-text-muted mt-1">
            <template v-if="recordingsStore.totalCount > 0">
              {{ recordingsStore.totalCount }} saved
            </template>
            <template v-else>
              Your library
            </template>
          </p>
        </router-link>

        <router-link
          to="/settings"
          class="glass p-6 text-center hover:bg-white/10 transition-colors group"
        >
          <div class="w-14 h-14 mx-auto mb-3 rounded-2xl bg-warning/20 flex items-center justify-center group-hover:bg-warning/30 transition-colors">
            <svg class="w-7 h-7 text-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <p class="font-medium text-text-primary">Settings</p>
          <p class="text-xs text-text-muted mt-1">Preferences</p>
        </router-link>
      </div>

      <!-- Stats Row -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Recordings Stats -->
        <div class="glass p-5">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-success/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
              </svg>
            </div>
            <div>
              <p class="text-2xl font-bold text-text-primary">{{ recordingsStore.totalCount || 0 }}</p>
              <p class="text-xs text-text-muted">Recordings</p>
            </div>
          </div>
        </div>

        <!-- Unwatched -->
        <div class="glass p-5">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-accent/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              </svg>
            </div>
            <div>
              <p class="text-2xl font-bold text-text-primary">{{ recordingsStore.unwatchedCount || 0 }}</p>
              <p class="text-xs text-text-muted">Unwatched</p>
            </div>
          </div>
        </div>

        <!-- Device Info -->
        <div class="glass p-5">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl bg-info/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-info" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
            </div>
            <div>
              <p class="text-2xl font-bold text-text-primary">{{ activeDevice.tuners || '?' }}</p>
              <p class="text-xs text-text-muted">Tuners</p>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- ============================================ -->
    <!-- DISCONNECTED STATE: Welcome / Setup -->
    <!-- ============================================ -->
    <template v-else>
      <!-- Header -->
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-text-primary">Welcome to OpenTabTV</h1>
        <p class="text-text-secondary mt-2">
          Connect to your Tablo DVR to start watching live TV and recordings.
        </p>
      </div>

      <!-- Account Status -->
      <div v-if="accountStore.isLoggedIn" class="glass p-4 mb-6 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-full bg-accent/20 flex items-center justify-center">
            <svg class="w-5 h-5 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </div>
          <div>
            <p class="font-medium text-text-primary">{{ accountStore.email }}</p>
            <p class="text-sm text-text-muted">4th Gen account connected</p>
          </div>
        </div>
        <button
          @click="handleLogout"
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          Sign Out
        </button>
      </div>

      <!-- Connection Options -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Auto Discovery -->
        <div class="glass p-6">
          <h2 class="text-xl font-semibold text-text-primary mb-4">
            Auto Discovery
          </h2>
          <p class="text-text-secondary text-sm mb-4">
            Scan your local network to find Legacy Tablo devices automatically.
          </p>
          <button
            @click="scanForDevices"
            :disabled="isScanning"
            class="w-full px-4 py-3 bg-accent hover:bg-accent-hover text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {{ isScanning ? "Scanning..." : "Scan Network" }}
          </button>
        </div>

        <!-- Manual IP -->
        <div class="glass p-6">
          <h2 class="text-xl font-semibold text-text-primary mb-4">
            Manual Connection
          </h2>
          <p class="text-text-secondary text-sm mb-4">
            Enter your Tablo's IP address directly if auto-discovery doesn't work.
          </p>
          <div class="space-y-2">
            <input
              v-model="manualIp"
              type="text"
              placeholder="192.168.1.100"
              :disabled="isConnectingManual"
              class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent disabled:opacity-50"
            />
            <button
              @click="connectManual"
              :disabled="!manualIp || isConnectingManual"
              class="w-full px-4 py-3 bg-surface-3 hover:bg-surface-2 text-text-primary rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              <svg
                v-if="isConnectingManual"
                class="w-5 h-5 animate-spin"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isConnectingManual ? "Connecting..." : "Connect" }}
            </button>
          </div>
        </div>

        <!-- 4th Gen Login -->
        <div class="glass p-6">
          <h2 class="text-xl font-semibold text-text-primary mb-4">
            4th Gen Login
          </h2>
          <p class="text-text-secondary text-sm mb-4">
            Sign in with your Tablo account to access 4th Gen devices.
          </p>
          <button
            v-if="!accountStore.isLoggedIn && !showLoginForm"
            @click="showLoginForm = true"
            class="w-full px-4 py-3 bg-surface-3 hover:bg-surface-2 text-text-primary rounded-xl font-medium transition-colors"
          >
            Sign In
          </button>
          <div v-else-if="accountStore.isLoggedIn" class="text-center">
            <p class="text-success text-sm">Connected</p>
          </div>
        </div>
      </div>

      <!-- Login Form Modal -->
      <div
        v-if="showLoginForm && !accountStore.isLoggedIn"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
        @click.self="showLoginForm = false"
      >
        <div class="glass p-6 w-full max-w-md">
          <h2 class="text-xl font-semibold text-text-primary mb-4">
            Sign in to Tablo
          </h2>
          <p class="text-text-secondary text-sm mb-6">
            Enter your Tablo account credentials to access 4th Gen devices.
          </p>

          <form @submit.prevent="handleLogin" class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-2">
                Email
              </label>
              <input
                v-model="loginEmail"
                type="email"
                placeholder="your@email.com"
                class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent"
                required
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-text-secondary mb-2">
                Password
              </label>
              <input
                v-model="loginPassword"
                type="password"
                placeholder="Your password"
                class="w-full px-4 py-3 bg-surface-2 border border-white/10 rounded-xl text-text-primary placeholder-text-muted focus:border-accent focus:ring-1 focus:ring-accent"
                required
              />
            </div>

            <label class="flex items-center gap-2 cursor-pointer">
              <input
                v-model="rememberCredentials"
                type="checkbox"
                class="w-4 h-4 rounded border-white/10 bg-surface-2 text-accent focus:ring-accent"
              />
              <span class="text-sm text-text-secondary">Remember me</span>
            </label>

            <div v-if="loginError" class="p-3 bg-error/20 text-error text-sm rounded-xl">
              {{ loginError }}
            </div>

            <div class="flex gap-3 pt-2">
              <button
                type="button"
                @click="showLoginForm = false"
                class="flex-1 px-4 py-3 bg-surface-2 hover:bg-surface-3 text-text-primary rounded-xl font-medium transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                :disabled="isLoggingIn"
                class="flex-1 px-4 py-3 bg-accent hover:bg-accent-hover text-white rounded-xl font-medium transition-colors disabled:opacity-50"
              >
                {{ isLoggingIn ? "Signing in..." : "Sign In" }}
              </button>
            </div>
          </form>
        </div>
      </div>

      <!-- Found Devices -->
      <div
        v-if="devicesStore.devices.length > 0"
        class="mt-8"
      >
        <h2 class="text-xl font-semibold text-text-primary mb-4">
          Found Devices
        </h2>
        <div class="space-y-3">
          <div
            v-for="device in devicesStore.devices"
            :key="device.id"
            class="glass p-4 flex items-center justify-between transition-colors"
            :class="[
              connectingDeviceId && connectingDeviceId !== device.id
                ? 'opacity-50 cursor-not-allowed'
                : 'cursor-pointer hover:bg-white/10'
            ]"
            @click="!connectingDeviceId && handleDeviceConnect(device)"
          >
            <div class="flex items-center gap-3">
              <!-- Connecting spinner -->
              <div
                v-if="connectingDeviceId === device.id"
                class="w-8 h-8 flex items-center justify-center"
              >
                <svg class="w-5 h-5 text-accent animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              </div>
              <div>
                <div class="flex items-center gap-2">
                  <p class="font-medium text-text-primary">{{ device.name }}</p>
                  <span
                    class="px-2 py-0.5 text-xs font-medium rounded-full"
                    :class="getDeviceBadge(device).class"
                  >
                    {{ getDeviceBadge(device).text }}
                  </span>
                  <span
                    v-if="connectingDeviceId === device.id"
                    class="text-xs text-accent"
                  >
                    Connecting...
                  </span>
                </div>
                <p class="text-sm text-text-muted">
                  {{ device.localIp || "Cloud device" }}
                  <span v-if="device.model" class="ml-2">{{ device.model }}</span>
                </p>
              </div>
            </div>
            <div class="text-text-muted">
              <svg
                v-if="connectingDeviceId !== device.id"
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 5l7 7-7 7"
                />
              </svg>
            </div>
          </div>
        </div>
      </div>

      <!-- Registered Devices (if any but not discovered) -->
      <div
        v-else-if="devicesStore.hasRegisteredDevices"
        class="mt-8"
      >
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-text-primary">Your Devices</h2>
          <router-link
            to="/devices"
            class="text-sm text-accent hover:text-accent-hover transition-colors"
          >
            Manage
          </router-link>
        </div>
        <p class="text-text-muted text-sm mb-4">
          Select a device to connect, or scan to find new devices.
        </p>
        <div class="space-y-3">
          <div
            v-for="device in devicesStore.registeredDevices"
            :key="device.id"
            class="glass p-4 flex items-center justify-between cursor-pointer hover:bg-white/10 transition-colors"
            @click="handleDeviceConnect(device)"
          >
            <div class="flex items-center gap-3">
              <div
                class="w-2.5 h-2.5 rounded-full"
                :class="devicesStore.getDeviceStatus(device.id) === 'online' ? 'bg-success' : 'bg-text-muted'"
              />
              <div>
                <div class="flex items-center gap-2">
                  <p class="font-medium text-text-primary">{{ device.name }}</p>
                  <span
                    class="px-2 py-0.5 text-xs font-medium rounded-full"
                    :class="getDeviceBadge(device).class"
                  >
                    {{ getDeviceBadge(device).text }}
                  </span>
                </div>
                <p class="text-sm text-text-muted">{{ device.localIp }}</p>
              </div>
            </div>
            <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
