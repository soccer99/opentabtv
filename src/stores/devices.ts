import { ref, computed, watch } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { TabloDevice, ConnectionState, DeviceGeneration } from "@/types";

export type DeviceStatus = "online" | "offline" | "checking";

const REGISTERED_DEVICES_KEY = "tablo_registered_devices";

export const useDevicesStore = defineStore("devices", () => {
  // State
  const devices = ref<TabloDevice[]>([]);
  const activeDevice = ref<TabloDevice | null>(null);
  const connectionState = ref<ConnectionState>("disconnected");
  const error = ref<string | null>(null);
  const isLoading = ref(false);

  // Registered devices (persisted to localStorage)
  const registeredDevices = ref<TabloDevice[]>([]);
  const deviceStatus = ref<Map<string, DeviceStatus>>(new Map());

  // Load registered devices from localStorage on init
  function loadRegisteredDevices() {
    try {
      const stored = localStorage.getItem(REGISTERED_DEVICES_KEY);
      if (stored) {
        registeredDevices.value = JSON.parse(stored);
        // Initialize all as offline until checked
        registeredDevices.value.forEach((d) => {
          deviceStatus.value.set(d.id, "offline");
        });
      }
    } catch (e) {
      console.error("Failed to load registered devices:", e);
    }
  }

  // Save registered devices to localStorage
  function saveRegisteredDevices() {
    try {
      localStorage.setItem(
        REGISTERED_DEVICES_KEY,
        JSON.stringify(registeredDevices.value)
      );
    } catch (e) {
      console.error("Failed to save registered devices:", e);
    }
  }

  // Watch for changes and persist
  watch(registeredDevices, saveRegisteredDevices, { deep: true });

  // Initialize on store creation
  loadRegisteredDevices();

  // Getters
  const isConnected = computed(() => connectionState.value === "connected");
  const hasDevices = computed(() => devices.value.length > 0);
  const hasRegisteredDevices = computed(
    () => registeredDevices.value.length > 0
  );
  const legacyDevices = computed(() =>
    devices.value.filter((d) => d.generation === "legacy")
  );
  const gen4Devices = computed(() =>
    devices.value.filter((d) => d.generation === "gen4")
  );
  const activeDeviceGeneration = computed(() => activeDevice.value?.generation);

  // Actions
  async function discoverDevices(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const found = await invoke<TabloDevice[]>("discover_devices");
      devices.value = found;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Discovery failed:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function connectToDevice(device: TabloDevice): Promise<void> {
    connectionState.value = "connecting";
    error.value = null;

    try {
      const connected = await invoke<TabloDevice>("connect_device", { device });
      activeDevice.value = connected;
      connectionState.value = "connected";
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      connectionState.value = "disconnected";
      console.error("Connection failed:", e);
    }
  }

  async function connectByIp(
    ip: string,
    generation?: DeviceGeneration
  ): Promise<void> {
    connectionState.value = "connecting";
    error.value = null;

    try {
      const device = await invoke<TabloDevice>("connect_by_ip", {
        ip,
        generation,
      });
      activeDevice.value = device;
      connectionState.value = "connected";
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      connectionState.value = "disconnected";
      console.error("Connection failed:", e);
    }
  }

  async function connectGen4Device(device: TabloDevice): Promise<void> {
    if (device.generation !== "gen4") {
      error.value = "Device is not a 4th Gen device";
      return;
    }

    connectionState.value = "connecting";
    error.value = null;

    try {
      const connected = await invoke<TabloDevice>("connect_gen4_device", {
        device,
      });
      activeDevice.value = connected;
      connectionState.value = "connected";
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      connectionState.value = "disconnected";
      console.error("Connection failed:", e);
    }
  }

  function addCloudDevices(cloudDevices: TabloDevice[]): void {
    // Merge cloud devices with existing devices, avoiding duplicates
    const existingIds = new Set(devices.value.map((d) => d.id));
    const newDevices = cloudDevices.filter((d) => !existingIds.has(d.id));
    devices.value = [...devices.value, ...newDevices];
  }

  async function getLastDevice(): Promise<{ ip: string; id: string } | null> {
    try {
      const result = await invoke<[string, string] | null>("get_last_device");
      if (result) {
        return { ip: result[0], id: result[1] };
      }
      return null;
    } catch (e) {
      console.error("Failed to get last device:", e);
      return null;
    }
  }

  async function disconnect(): Promise<void> {
    try {
      await invoke("disconnect_device");
    } catch (e) {
      console.error("Disconnect error:", e);
    }
    activeDevice.value = null;
    connectionState.value = "disconnected";
  }

  async function refreshActiveDevice(): Promise<void> {
    try {
      const device = await invoke<TabloDevice | null>("get_active_device");
      if (device) {
        activeDevice.value = device;
        connectionState.value = "connected";
      }
    } catch (e) {
      console.error("Failed to refresh active device:", e);
    }
  }

  // Register a device (add to persisted list)
  function registerDevice(device: TabloDevice): void {
    const exists = registeredDevices.value.some((d) => d.id === device.id);
    if (!exists) {
      registeredDevices.value.push(device);
      deviceStatus.value.set(device.id, "online");
    } else {
      // Update existing device info
      const index = registeredDevices.value.findIndex(
        (d) => d.id === device.id
      );
      if (index >= 0) {
        registeredDevices.value[index] = device;
      }
    }
  }

  // Unregister a device (remove from persisted list)
  function unregisterDevice(deviceId: string): void {
    registeredDevices.value = registeredDevices.value.filter(
      (d) => d.id !== deviceId
    );
    deviceStatus.value.delete(deviceId);
  }

  // Check if a single device is reachable
  async function checkDeviceStatus(device: TabloDevice): Promise<DeviceStatus> {
    deviceStatus.value.set(device.id, "checking");

    try {
      // Try to ping the device by attempting a basic connection check
      const isReachable = await invoke<boolean>("check_device_reachable", {
        ip: device.localIp,
      });

      const status: DeviceStatus = isReachable ? "online" : "offline";
      deviceStatus.value.set(device.id, status);
      return status;
    } catch (e) {
      console.error(`Failed to check device ${device.name}:`, e);
      deviceStatus.value.set(device.id, "offline");
      return "offline";
    }
  }

  // Check all registered devices
  async function checkAllDevices(): Promise<void> {
    const promises = registeredDevices.value.map((device) =>
      checkDeviceStatus(device)
    );
    await Promise.all(promises);
  }

  // Get status for a device
  function getDeviceStatus(deviceId: string): DeviceStatus {
    return deviceStatus.value.get(deviceId) || "offline";
  }

  return {
    // State
    devices,
    activeDevice,
    connectionState,
    error,
    isLoading,
    registeredDevices,
    deviceStatus,
    // Getters
    isConnected,
    hasDevices,
    hasRegisteredDevices,
    legacyDevices,
    gen4Devices,
    activeDeviceGeneration,
    // Actions
    discoverDevices,
    connectToDevice,
    connectByIp,
    connectGen4Device,
    addCloudDevices,
    getLastDevice,
    disconnect,
    refreshActiveDevice,
    registerDevice,
    unregisterDevice,
    checkDeviceStatus,
    checkAllDevices,
    getDeviceStatus,
  };
});
