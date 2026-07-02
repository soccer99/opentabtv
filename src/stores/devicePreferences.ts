import { ref } from "vue";
import { defineStore } from "pinia";
import { LazyStore } from "@tauri-apps/plugin-store";

export interface RecentItem {
  id: string;
  watchedAt: string; // ISO timestamp
}

export interface DevicePreferences {
  favoriteChannelIds: string[];
  recentChannels: RecentItem[];
}

// Store instance (lazy loaded)
let store: LazyStore | null = null;

function getStore(): LazyStore {
  if (!store) {
    store = new LazyStore("device-preferences.json");
  }
  return store;
}

const MAX_RECENT_CHANNELS = 3;

export const useDevicePreferencesStore = defineStore("devicePreferences", () => {
  // State: Record<deviceId, DevicePreferences>
  const preferences = ref<Record<string, DevicePreferences>>({});
  const isLoaded = ref(false);

  // Loading promise to prevent race conditions
  let loadingPromise: Promise<void> | null = null;

  // Get or create default preferences for a device
  function getDevicePrefs(deviceId: string): DevicePreferences {
    if (!preferences.value[deviceId]) {
      preferences.value[deviceId] = {
        favoriteChannelIds: [],
        recentChannels: [],
      };
    }
    return preferences.value[deviceId];
  }

  // Load preferences from store (with race condition protection)
  async function load(): Promise<void> {
    // If already loaded, return immediately
    if (isLoaded.value) return;

    // If loading is in progress, wait for it
    if (loadingPromise) {
      return loadingPromise;
    }

    // Start loading
    loadingPromise = (async () => {
      try {
        const s = getStore();
        const saved = await s.get<Record<string, DevicePreferences>>("preferences");
        if (saved) {
          preferences.value = saved;
        }
      } catch (e) {
        console.warn("Failed to load device preferences:", e);
      } finally {
        isLoaded.value = true;
        loadingPromise = null;
      }
    })();

    return loadingPromise;
  }

  // Save preferences to store
  async function save(): Promise<void> {
    try {
      const s = getStore();
      await s.set("preferences", preferences.value);
      await s.save();
    } catch (e) {
      console.warn("Failed to save device preferences:", e);
    }
  }

  // Toggle favorite status for a channel
  async function toggleFavorite(deviceId: string, channelId: string): Promise<void> {
    await load();

    const prefs = getDevicePrefs(deviceId);
    const index = prefs.favoriteChannelIds.indexOf(channelId);

    if (index >= 0) {
      prefs.favoriteChannelIds.splice(index, 1);
    } else {
      prefs.favoriteChannelIds.push(channelId);
    }

    await save();
  }

  // Check if a channel is favorited
  function isFavorite(deviceId: string, channelId: string): boolean {
    const prefs = preferences.value[deviceId];
    if (!prefs) return false;
    return prefs.favoriteChannelIds.includes(channelId);
  }

  // Get favorite channel IDs for a device
  function getFavorites(deviceId: string): string[] {
    const prefs = preferences.value[deviceId];
    return prefs?.favoriteChannelIds || [];
  }

  // Add a channel to recent channels
  async function addRecentChannel(deviceId: string, channelId: string): Promise<void> {
    await load();

    const prefs = getDevicePrefs(deviceId);

    // Remove existing entry for this channel (to move it to front)
    prefs.recentChannels = prefs.recentChannels.filter((r) => r.id !== channelId);

    // Add to front
    prefs.recentChannels.unshift({
      id: channelId,
      watchedAt: new Date().toISOString(),
    });

    // Keep only max items
    if (prefs.recentChannels.length > MAX_RECENT_CHANNELS) {
      prefs.recentChannels = prefs.recentChannels.slice(0, MAX_RECENT_CHANNELS);
    }

    await save();
  }

  // Get recent channels for a device
  function getRecentChannels(deviceId: string): RecentItem[] {
    const prefs = preferences.value[deviceId];
    return prefs?.recentChannels || [];
  }

  // Initialize store (call early in app lifecycle)
  async function initialize(): Promise<void> {
    await load();
  }

  return {
    // State
    preferences,
    isLoaded,
    // Actions
    initialize,
    toggleFavorite,
    isFavorite,
    getFavorites,
    addRecentChannel,
    getRecentChannels,
  };
});
