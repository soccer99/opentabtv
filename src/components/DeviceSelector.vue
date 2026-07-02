<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { useDevicesStore, type DeviceStatus } from "@/stores/devices";
import { useToast } from "@/composables/useToast";
import type { TabloDevice } from "@/types";

defineProps<{
  isCollapsed: boolean;
}>();

const router = useRouter();
const devicesStore = useDevicesStore();
const toast = useToast();

const isOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);

const registeredDevices = computed(() => devicesStore.registeredDevices);
const activeDevice = computed(() => devicesStore.activeDevice);
const hasDevices = computed(() => registeredDevices.value.length > 0);

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

async function selectDevice(device: TabloDevice) {
  const status = devicesStore.getDeviceStatus(device.id);

  if (status === "offline") {
    toast.warning(`${device.name} is offline. Try checking connection first.`);
    return;
  }

  if (status === "checking") {
    toast.info("Please wait while checking device status...");
    return;
  }

  try {
    if (device.generation === "gen4") {
      await devicesStore.connectGen4Device(device);
    } else {
      await devicesStore.connectToDevice(device);
    }

    if (devicesStore.isConnected) {
      toast.success(`Connected to ${device.name}`);
      isOpen.value = false;
    } else if (devicesStore.error) {
      toast.error(devicesStore.error);
    }
  } catch (e) {
    toast.error(e instanceof Error ? e.message : String(e));
  }
}

function navigateToDevices() {
  isOpen.value = false;
  router.push("/devices");
}

function toggleDropdown() {
  isOpen.value = !isOpen.value;
}

// Close dropdown when clicking outside
function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
}

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  // Check all devices on mount
  if (hasDevices.value) {
    devicesStore.checkAllDevices();
  }
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
});

// Current connection status
const connectionStatus = computed(() => {
  if (devicesStore.connectionState === "connecting") {
    return {
      color: "bg-warning animate-pulse",
      text: "Connecting",
      subtext: "Please wait...",
    };
  }
  if (devicesStore.isConnected && activeDevice.value) {
    return {
      color: "bg-success",
      text: activeDevice.value.name,
      subtext: activeDevice.value.localIp || "Connected",
    };
  }
  return {
    color: "bg-text-muted",
    text: "No Device",
    subtext: hasDevices.value ? "Select a device" : "Add a device",
  };
});
</script>

<template>
  <div ref="dropdownRef" class="relative p-3 border-t border-white/5">
    <!-- Collapsed state: just show status dot -->
    <div
      v-if="isCollapsed"
      class="lg:flex hidden items-center justify-center"
    >
      <button
        @click="toggleDropdown"
        class="p-2.5 rounded-lg bg-surface-2/50 hover:bg-surface-2 transition-colors"
        :title="connectionStatus.text"
      >
        <div
          class="w-2.5 h-2.5 rounded-full"
          :class="connectionStatus.color"
        />
      </button>
    </div>

    <!-- Expanded state: show full selector -->
    <div :class="isCollapsed ? 'lg:hidden' : ''">
      <button
        @click="toggleDropdown"
        class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg bg-surface-2/50 hover:bg-surface-2 transition-colors"
      >
        <!-- Status indicator -->
        <div
          class="w-2.5 h-2.5 rounded-full shrink-0"
          :class="connectionStatus.color"
        />
        <!-- Status text -->
        <div class="min-w-0 flex-1 text-left">
          <p class="text-sm font-medium text-text-primary truncate">
            {{ connectionStatus.text }}
          </p>
          <p class="text-xs text-text-muted truncate">
            {{ connectionStatus.subtext }}
          </p>
        </div>
        <!-- Chevron -->
        <svg
          class="w-4 h-4 text-text-muted shrink-0 transition-transform"
          :class="{ 'rotate-180': isOpen }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
        </svg>
      </button>
    </div>

    <!-- Dropdown menu -->
    <Transition
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <div
        v-if="isOpen"
        class="absolute bottom-full left-3 right-3 mb-2 bg-surface-1 border border-white/10 rounded-xl shadow-xl overflow-hidden z-50"
      >
        <!-- Device list -->
        <div v-if="hasDevices" class="max-h-64 overflow-y-auto">
          <button
            v-for="device in registeredDevices"
            :key="device.id"
            @click="selectDevice(device)"
            class="w-full px-4 py-3 flex items-center gap-3 hover:bg-white/5 transition-colors text-left"
            :class="{
              'bg-accent/10': isActiveDevice(device),
              'opacity-50': devicesStore.getDeviceStatus(device.id) === 'offline'
            }"
          >
            <!-- Status dot -->
            <div
              class="w-2 h-2 rounded-full shrink-0"
              :class="getStatusColor(devicesStore.getDeviceStatus(device.id))"
            />
            <!-- Device info -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <p class="text-sm font-medium text-text-primary truncate">
                  {{ device.name }}
                </p>
                <span
                  v-if="isActiveDevice(device)"
                  class="px-1.5 py-0.5 text-[10px] font-medium rounded bg-accent/20 text-accent"
                >
                  Active
                </span>
              </div>
              <div class="flex items-center gap-2 text-xs text-text-muted">
                <span>{{ device.localIp }}</span>
                <span
                  class="px-1.5 py-0.5 text-[10px] rounded"
                  :class="device.generation === 'gen4' ? 'bg-accent/20 text-accent' : 'bg-info/20 text-info'"
                >
                  {{ device.generation === 'gen4' ? '4th Gen' : 'Legacy' }}
                </span>
              </div>
            </div>
            <!-- Status text -->
            <span class="text-xs text-text-muted shrink-0">
              {{ getStatusText(devicesStore.getDeviceStatus(device.id)) }}
            </span>
          </button>
        </div>

        <!-- Empty state -->
        <div v-else class="px-4 py-6 text-center">
          <p class="text-sm text-text-muted">No devices registered</p>
        </div>

        <!-- Actions -->
        <div class="border-t border-white/5 p-2 space-y-1">
          <button
            @click="navigateToDevices"
            class="w-full px-3 py-2 flex items-center gap-2 text-sm text-text-secondary hover:text-text-primary hover:bg-white/5 rounded-lg transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add / Manage Devices
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>
