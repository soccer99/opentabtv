<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useDevicesStore, type DeviceStatus } from "@/stores/devices";
import { useAccountStore } from "@/stores/account";
import { useToast } from "@/composables/useToast";
import type { TabloDevice } from "@/types";

const devicesStore = useDevicesStore();
const accountStore = useAccountStore();
const toast = useToast();

// Scanning state
const isScanning = ref(false);
const connectingDeviceId = ref<string | null>(null);
const checkingDeviceId = ref<string | null>(null);

// Manual IP form
const manualIp = ref("");
const isConnectingManual = ref(false);

// Login form
const showLoginForm = ref(false);
const loginEmail = ref("");
const loginPassword = ref("");
const rememberCredentials = ref(false);
const isLoggingIn = ref(false);
const loginError = ref<string | null>(null);

// Remove confirmation
const deviceToRemove = ref<TabloDevice | null>(null);

const registeredDevices = computed(() => devicesStore.registeredDevices);
const hasRegisteredDevices = computed(() => registeredDevices.value.length > 0);
const activeDevice = computed(() => devicesStore.activeDevice);

function getStatusColor(status: DeviceStatus): string {
  switch (status) {
    case "online":
      return "bg-success";
    case "offline":
      return "bg-error";
    case "checking":
      return "bg-warning animate-pulse";
    default:
      return "bg-text-muted";
  }
}

function getStatusText(status: DeviceStatus): string {
  switch (status) {
    case "online":
      return "Online";
    case "offline":
      return "Offline";
    case "checking":
      return "Checking...";
    default:
      return "Unknown";
  }
}

function isActiveDevice(device: TabloDevice): boolean {
  return activeDevice.value?.id === device.id;
}

function getDeviceBadge(device: TabloDevice): { text: string; class: string } {
  if (device.generation === "gen4") {
    return { text: "4th Gen", class: "bg-accent/20 text-accent" };
  }
  return { text: "Legacy", class: "bg-info/20 text-info" };
}

// Actions
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
    toast.error(e instanceof Error ? e.message : String(e));
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
      toast.success(`Found ${cloudDevices.length} 4th Gen device${cloudDevices.length === 1 ? "" : "s"}`);
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
  toast.info("Signed out from Tablo account");
}

async function checkDevice(device: TabloDevice) {
  checkingDeviceId.value = device.id;
  try {
    const status = await devicesStore.checkDeviceStatus(device);
    if (status === "online") {
      toast.success(`${device.name} is online`);
    } else {
      toast.warning(`${device.name} is offline`);
    }
  } finally {
    checkingDeviceId.value = null;
  }
}

async function checkAllDevices() {
  toast.info("Checking all devices...");
  await devicesStore.checkAllDevices();
  const online = registeredDevices.value.filter(
    (d) => devicesStore.getDeviceStatus(d.id) === "online"
  ).length;
  toast.success(`${online} of ${registeredDevices.value.length} devices online`);
}

async function connectDevice(device: TabloDevice) {
  const status = devicesStore.getDeviceStatus(device.id);
  if (status === "offline") {
    toast.warning(`${device.name} is offline. Try checking connection first.`);
    return;
  }

  connectingDeviceId.value = device.id;
  try {
    if (device.generation === "gen4") {
      await devicesStore.connectGen4Device(device);
    } else {
      await devicesStore.connectToDevice(device);
    }

    if (devicesStore.isConnected) {
      toast.success(`Connected to ${device.name}`);
    } else if (devicesStore.error) {
      toast.error(devicesStore.error);
    }
  } catch (e) {
    toast.error(e instanceof Error ? e.message : String(e));
  } finally {
    connectingDeviceId.value = null;
  }
}

async function disconnectDevice() {
  await devicesStore.disconnect();
  toast.info("Disconnected from device");
}

function confirmRemoveDevice(device: TabloDevice) {
  deviceToRemove.value = device;
}

function cancelRemove() {
  deviceToRemove.value = null;
}

function removeDevice() {
  if (!deviceToRemove.value) return;

  const name = deviceToRemove.value.name;
  const wasActive = isActiveDevice(deviceToRemove.value);

  devicesStore.unregisterDevice(deviceToRemove.value.id);

  if (wasActive) {
    devicesStore.disconnect();
  }

  toast.success(`Removed ${name}`);
  deviceToRemove.value = null;
}

onMounted(async () => {
  // Check for saved credentials and auto-login
  const hasSaved = await accountStore.checkSavedCredentials();
  if (hasSaved && !accountStore.isLoggedIn) {
    const success = await accountStore.autoLogin();
    if (success) {
      const cloudDevices = await accountStore.discoverCloudDevices();
      devicesStore.addCloudDevices(cloudDevices);
      for (const device of cloudDevices) {
        devicesStore.registerDevice(device);
      }
    }
  }

  // Check all registered devices
  if (hasRegisteredDevices.value) {
    await devicesStore.checkAllDevices();
  }
});
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <!-- Header -->
    <div class="mb-8 flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-text-primary">Manage Devices</h1>
        <p class="text-text-secondary mt-2">
          Add, remove, and manage your Tablo DVR devices.
        </p>
      </div>
      <button
        v-if="hasRegisteredDevices"
        @click="checkAllDevices"
        class="px-4 py-2 bg-surface-2 hover:bg-surface-3 text-text-primary rounded-xl font-medium transition-colors flex items-center gap-2"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Check All
      </button>
    </div>

    <!-- Account Status (4th Gen) -->
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

    <!-- Registered Devices -->
    <section class="mb-8">
      <h2 class="text-xl font-semibold text-text-primary mb-4">Your Devices</h2>

      <!-- Empty state -->
      <div v-if="!hasRegisteredDevices" class="glass p-8 text-center">
        <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-surface-2 flex items-center justify-center">
          <svg class="w-8 h-8 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
        </div>
        <p class="text-text-secondary mb-2">No devices registered yet</p>
        <p class="text-sm text-text-muted">Add a device using one of the methods below.</p>
      </div>

      <!-- Device list -->
      <div v-else class="space-y-3">
        <div
          v-for="device in registeredDevices"
          :key="device.id"
          class="glass p-4 flex items-center justify-between"
        >
          <div class="flex items-center gap-4">
            <!-- Status indicator -->
            <div
              class="w-3 h-3 rounded-full shrink-0"
              :class="getStatusColor(devicesStore.getDeviceStatus(device.id))"
              :title="getStatusText(devicesStore.getDeviceStatus(device.id))"
            />
            <!-- Device info -->
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
                  v-if="isActiveDevice(device)"
                  class="px-2 py-0.5 text-xs font-medium rounded-full bg-success/20 text-success"
                >
                  Connected
                </span>
              </div>
              <p class="text-sm text-text-muted">
                {{ device.localIp || "Cloud device" }}
                <span v-if="device.model" class="ml-2">{{ device.model }}</span>
              </p>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2">
            <!-- Check connection -->
            <button
              @click="checkDevice(device)"
              :disabled="checkingDeviceId === device.id"
              class="p-2 text-text-muted hover:text-text-primary hover:bg-white/5 rounded-lg transition-colors disabled:opacity-50"
              title="Check connection"
            >
              <svg
                class="w-5 h-5"
                :class="{ 'animate-spin': checkingDeviceId === device.id }"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>

            <!-- Connect/Disconnect -->
            <button
              v-if="isActiveDevice(device)"
              @click="disconnectDevice"
              class="px-3 py-1.5 text-sm bg-surface-2 hover:bg-surface-3 text-text-primary rounded-lg transition-colors"
            >
              Disconnect
            </button>
            <button
              v-else
              @click="connectDevice(device)"
              :disabled="connectingDeviceId === device.id || devicesStore.getDeviceStatus(device.id) === 'offline'"
              class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
            >
              <svg
                v-if="connectingDeviceId === device.id"
                class="w-4 h-4 animate-spin"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ connectingDeviceId === device.id ? "Connecting..." : "Connect" }}
            </button>

            <!-- Remove -->
            <button
              @click="confirmRemoveDevice(device)"
              class="p-2 text-text-muted hover:text-error hover:bg-error/10 rounded-lg transition-colors"
              title="Remove device"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Add Device Section -->
    <section>
      <h2 class="text-xl font-semibold text-text-primary mb-4">Add Device</h2>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Auto Discovery -->
        <div class="glass p-6">
          <h3 class="text-lg font-semibold text-text-primary mb-3">
            Auto Discovery
          </h3>
          <p class="text-text-secondary text-sm mb-4">
            Scan your local network to find Legacy Tablo devices automatically.
          </p>
          <button
            @click="scanForDevices"
            :disabled="isScanning"
            class="w-full px-4 py-3 bg-accent hover:bg-accent-hover text-white rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            <svg
              v-if="isScanning"
              class="w-5 h-5 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ isScanning ? "Scanning..." : "Scan Network" }}
          </button>
        </div>

        <!-- Manual IP -->
        <div class="glass p-6">
          <h3 class="text-lg font-semibold text-text-primary mb-3">
            Manual Connection
          </h3>
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
          <h3 class="text-lg font-semibold text-text-primary mb-3">
            4th Gen Login
          </h3>
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
            <p class="text-success text-sm flex items-center justify-center gap-2">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              Connected
            </p>
          </div>
        </div>
      </div>
    </section>

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

    <!-- Remove Confirmation Modal -->
    <div
      v-if="deviceToRemove"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      @click.self="cancelRemove"
    >
      <div class="glass p-6 w-full max-w-sm">
        <h2 class="text-xl font-semibold text-text-primary mb-4">
          Remove Device
        </h2>
        <p class="text-text-secondary mb-6">
          Are you sure you want to remove <strong class="text-text-primary">{{ deviceToRemove.name }}</strong>?
          You can add it again later.
        </p>
        <div class="flex gap-3">
          <button
            @click="cancelRemove"
            class="flex-1 px-4 py-3 bg-surface-2 hover:bg-surface-3 text-text-primary rounded-xl font-medium transition-colors"
          >
            Cancel
          </button>
          <button
            @click="removeDevice"
            class="flex-1 px-4 py-3 bg-error hover:bg-error/80 text-white rounded-xl font-medium transition-colors"
          >
            Remove
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
