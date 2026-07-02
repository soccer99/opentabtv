import { ref } from "vue";
import { defineStore } from "pinia";

export const useSettingsStore = defineStore(
  "settings",
  () => {
    // Theme
    const theme = ref<"dark" | "light" | "system">("dark");

    // Playback
    const preferredQuality = ref<"auto" | "720p" | "1080p">("auto");
    const useVlc = ref(false);

    // External tools
    const ffmpegPath = ref<string>("");
    const vlcPath = ref<string>("");

    // Saved devices for quick reconnection
    const savedDevices = ref<
      Array<{
        id: string;
        name: string;
        localIp: string;
        lastConnected: string;
      }>
    >([]);

    // Actions
    function setTheme(newTheme: "dark" | "light" | "system") {
      theme.value = newTheme;
    }

    function setQuality(quality: "auto" | "720p" | "1080p") {
      preferredQuality.value = quality;
    }

    function addSavedDevice(device: {
      id: string;
      name: string;
      localIp: string;
    }) {
      const existing = savedDevices.value.findIndex((d) => d.id === device.id);
      if (existing >= 0) {
        savedDevices.value[existing].lastConnected = new Date().toISOString();
      } else {
        savedDevices.value.push({
          ...device,
          lastConnected: new Date().toISOString(),
        });
      }
    }

    function removeSavedDevice(deviceId: string) {
      savedDevices.value = savedDevices.value.filter((d) => d.id !== deviceId);
    }

    function clearSavedDevices() {
      savedDevices.value = [];
    }

    return {
      // State
      theme,
      preferredQuality,
      useVlc,
      ffmpegPath,
      vlcPath,
      savedDevices,
      // Actions
      setTheme,
      setQuality,
      addSavedDevice,
      removeSavedDevice,
      clearSavedDevices,
    };
  },
  {
    // Enable persistence
    persist: true,
  }
);
