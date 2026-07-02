import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { TabloDevice, ConnectionState, DeviceGeneration } from "@/types";

export const useDevicesStore = defineStore("devices", () => {
  // State
  const devices = ref<TabloDevice[]>([]);
  const activeDevice = ref<TabloDevice | null>(null);
  const connectionState = ref<ConnectionState>("disconnected");
  const error = ref<string | null>(null);
  const isLoading = ref(false);

  // Getters
  const isConnected = computed(() => connectionState.value === "connected");
  const hasDevices = computed(() => devices.value.length > 0);
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

  return {
    // State
    devices,
    activeDevice,
    connectionState,
    error,
    isLoading,
    // Getters
    isConnected,
    hasDevices,
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
  };
});
