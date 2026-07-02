import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { CastDevice, CastState, CastSession } from "@/types";

/**
 * Extract error message from unknown error type.
 * Utility function to avoid repeating this pattern.
 */
function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export const useCastingStore = defineStore("casting", () => {
  // State - using more descriptive names to avoid conflicts
  const castState = ref<CastState>("idle");
  const availableDevices = ref<CastDevice[]>([]);
  const activeSession = ref<CastSession | null>(null);
  const lastError = ref<string | null>(null);

  // Operation locks to prevent race conditions
  const isDiscoveryInProgress = ref(false);
  const isCastOperationInProgress = ref(false);

  // Computed
  const isCasting = computed(() => castState.value === "casting");
  const isDiscovering = computed(() => isDiscoveryInProgress.value);
  const hasDevices = computed(() => availableDevices.value.length > 0);
  const castDeviceName = computed(() => activeSession.value?.device.name ?? null);

  // Actions

  /**
   * Discover available cast devices on the network.
   * Protected against concurrent calls.
   */
  async function discoverDevices(timeoutSecs = 5): Promise<CastDevice[]> {
    // Prevent concurrent discovery operations
    if (isDiscoveryInProgress.value) {
      return availableDevices.value;
    }

    isDiscoveryInProgress.value = true;
    const previousState = castState.value;
    castState.value = "discovering";
    lastError.value = null;

    try {
      const devices = await invoke<CastDevice[]>("discover_cast_devices", {
        timeoutSecs,
      });

      availableDevices.value = devices;
      // Restore previous state (casting or idle)
      castState.value = activeSession.value ? "casting" : "idle";

      return devices;
    } catch (error) {
      lastError.value = `Discovery failed: ${extractErrorMessage(error)}`;
      // Restore previous state on error
      castState.value = previousState === "casting" ? "casting" : "idle";
      return [];
    } finally {
      isDiscoveryInProgress.value = false;
    }
  }

  /**
   * Cast media to a device.
   * Protected against concurrent operations.
   */
  async function castToDevice(
    device: CastDevice,
    mediaUrl: string,
    title?: string
  ): Promise<boolean> {
    // Prevent concurrent cast operations
    if (isCastOperationInProgress.value) {
      lastError.value = "Another cast operation is in progress";
      return false;
    }

    isCastOperationInProgress.value = true;

    try {
      // Stop current cast first if active
      if (castState.value === "casting" && activeSession.value) {
        await stopCastingInternal();
      }

      castState.value = "connecting";
      lastError.value = null;

      await invoke("cast_to_device", {
        device,
        mediaUrl,
        title: title ?? null,
      });

      activeSession.value = {
        device,
        mediaUrl,
        title,
        isPlaying: true,
      };
      castState.value = "casting";

      return true;
    } catch (error) {
      lastError.value = `Cast failed: ${extractErrorMessage(error)}`;
      castState.value = "idle";
      activeSession.value = null;
      return false;
    } finally {
      isCastOperationInProgress.value = false;
    }
  }

  /**
   * Internal stop casting without lock (used by castToDevice).
   */
  async function stopCastingInternal(): Promise<boolean> {
    if (!activeSession.value) {
      return true;
    }

    const device = activeSession.value.device;

    try {
      await invoke("stop_cast", { device });
      activeSession.value = null;
      castState.value = "idle";
      return true;
    } catch (error) {
      // Even if stop fails, clear local state - device may be disconnected
      activeSession.value = null;
      castState.value = "idle";
      // Don't set lastError here as this is an internal operation
      return false;
    }
  }

  /**
   * Stop current casting session.
   * Protected against concurrent operations.
   */
  async function stopCasting(): Promise<boolean> {
    if (!activeSession.value) {
      return true;
    }

    // Prevent concurrent operations
    if (isCastOperationInProgress.value) {
      lastError.value = "Another cast operation is in progress";
      return false;
    }

    isCastOperationInProgress.value = true;
    lastError.value = null;

    try {
      const device = activeSession.value.device;
      await invoke("stop_cast", { device });
      activeSession.value = null;
      castState.value = "idle";
      return true;
    } catch (error) {
      // Even if stop fails, clear local state - device may be disconnected
      lastError.value = `Stop failed: ${extractErrorMessage(error)}`;
      activeSession.value = null;
      castState.value = "idle";
      return false;
    } finally {
      isCastOperationInProgress.value = false;
    }
  }

  /**
   * Pause current casting playback.
   */
  async function pause(): Promise<boolean> {
    if (!activeSession.value) {
      return false;
    }

    try {
      await invoke("pause_cast", { device: activeSession.value.device });
      activeSession.value.isPlaying = false;
      return true;
    } catch (error) {
      lastError.value = `Pause failed: ${extractErrorMessage(error)}`;
      return false;
    }
  }

  /**
   * Resume current casting playback.
   */
  async function resume(): Promise<boolean> {
    if (!activeSession.value) {
      return false;
    }

    try {
      await invoke("resume_cast", { device: activeSession.value.device });
      activeSession.value.isPlaying = true;
      return true;
    } catch (error) {
      lastError.value = `Resume failed: ${extractErrorMessage(error)}`;
      return false;
    }
  }

  /**
   * Toggle play/pause on cast device.
   */
  async function togglePlayPause(): Promise<boolean> {
    if (!activeSession.value) {
      return false;
    }
    return activeSession.value.isPlaying ? pause() : resume();
  }

  /**
   * Set volume on cast device (0.0 to 1.0).
   */
  async function setVolume(volume: number): Promise<boolean> {
    if (!activeSession.value) {
      return false;
    }

    const clampedVolume = Math.max(0, Math.min(1, volume));

    try {
      await invoke("set_cast_volume", {
        device: activeSession.value.device,
        volume: clampedVolume,
      });
      return true;
    } catch (error) {
      lastError.value = `Volume failed: ${extractErrorMessage(error)}`;
      return false;
    }
  }

  /**
   * Clear any error state.
   */
  function clearError(): void {
    lastError.value = null;
  }

  /**
   * Reset all state (e.g., on app disconnect).
   */
  function reset(): void {
    castState.value = "idle";
    availableDevices.value = [];
    activeSession.value = null;
    lastError.value = null;
    isDiscoveryInProgress.value = false;
    isCastOperationInProgress.value = false;
  }

  return {
    // State (exposed as readonly where possible)
    state: castState, // Keep 'state' for backward compatibility
    availableDevices,
    activeSession,
    lastError,
    isDiscovering,

    // Computed
    isCasting,
    hasDevices,
    castDeviceName,

    // Actions
    discoverDevices,
    castToDevice,
    stopCasting,
    pause,
    resume,
    togglePlayPause,
    setVolume,
    clearError,
    reset,
  };
});
