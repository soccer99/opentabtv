import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";

export interface Channel {
  id: string;
  objectId: number;
  path: string;
  callSign: string;
  major: number;
  minor: number;
  network: string | null;
  resolution: "hd_1080" | "hd_720" | "sd" | "unknown";
  favourite: boolean;
}

export interface StreamSession {
  sessionId: string;
  playlistUrl: string;
  channelId: string;
  transcoded?: boolean;
}

export const useChannelsStore = defineStore("channels", () => {
  // State
  const channels = ref<Channel[]>([]);
  const selectedChannel = ref<Channel | null>(null);
  const currentStream = ref<StreamSession | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const hasChannels = computed(() => channels.value.length > 0);
  const isStreaming = computed(() => currentStream.value !== null);

  const channelNumber = computed(() => {
    if (!selectedChannel.value) return "";
    const { major, minor } = selectedChannel.value;
    return minor > 0 ? `${major}.${minor}` : `${major}`;
  });

  const sortedChannels = computed(() => {
    return [...channels.value].sort((a, b) => {
      if (a.major !== b.major) return a.major - b.major;
      return a.minor - b.minor;
    });
  });

  // Actions
  async function fetchChannels(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<Channel[]>("get_channels");
      channels.value = result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Failed to fetch channels:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function startStream(channel: Channel): Promise<StreamSession | null> {
    // Stop existing stream if any
    if (currentStream.value) {
      await stopStream();
    }

    selectedChannel.value = channel;
    error.value = null;

    try {
      const session = await invoke<StreamSession>("start_live_stream", {
        channelPath: channel.path,
      });
      currentStream.value = session;
      return session;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Failed to start stream:", e);
      return null;
    }
  }

  async function stopStream(): Promise<void> {
    if (currentStream.value) {
      try {
        await invoke("stop_stream", { sessionId: currentStream.value.sessionId });
      } catch (e) {
        console.error("Failed to stop stream:", e);
      }
      currentStream.value = null;
    }
  }

  function selectChannel(channel: Channel): void {
    selectedChannel.value = channel;
  }

  function clearSelection(): void {
    selectedChannel.value = null;
    currentStream.value = null;
  }

  return {
    // State
    channels,
    selectedChannel,
    currentStream,
    isLoading,
    error,
    // Getters
    hasChannels,
    isStreaming,
    channelNumber,
    sortedChannels,
    // Actions
    fetchChannels,
    startStream,
    stopStream,
    selectChannel,
    clearSelection,
  };
});
