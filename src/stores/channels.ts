import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useDevicesStore } from "./devices";
import { useDevicePreferencesStore } from "./devicePreferences";

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

export interface VlcStreamSession extends StreamSession {
  channelName: string;
  channelNumber: string;
  startedAt: Date;
}

export const useChannelsStore = defineStore("channels", () => {
  // State
  const channels = ref<Channel[]>([]);
  const selectedChannel = ref<Channel | null>(null);
  const currentStream = ref<StreamSession | null>(null);
  const vlcStreams = ref<Map<string, VlcStreamSession>>(new Map());
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const hasChannels = computed(() => channels.value.length > 0);
  const isStreaming = computed(() => currentStream.value !== null);
  const activeVlcStreams = computed(() => Array.from(vlcStreams.value.values()));
  const vlcStreamCount = computed(() => vlcStreams.value.size);

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

      // Track recent channel after successful stream start
      const devicesStore = useDevicesStore();
      const devicePreferencesStore = useDevicePreferencesStore();
      const deviceId = devicesStore.activeDevice?.id;
      if (deviceId) {
        devicePreferencesStore.addRecentChannel(deviceId, channel.id);
      }

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

  /**
   * Start a stream for VLC playback without stopping existing streams.
   * Allows multiple simultaneous VLC streams.
   */
  async function startVlcStream(channel: Channel): Promise<VlcStreamSession | null> {
    error.value = null;

    try {
      const session = await invoke<StreamSession>("start_live_stream", {
        channelPath: channel.path,
      });

      const channelNum =
        channel.minor > 0 ? `${channel.major}.${channel.minor}` : `${channel.major}`;

      const vlcSession: VlcStreamSession = {
        ...session,
        channelName: channel.callSign,
        channelNumber: channelNum,
        startedAt: new Date(),
      };

      // Add to VLC streams map
      vlcStreams.value.set(session.sessionId, vlcSession);

      // Track recent channel
      const devicesStore = useDevicesStore();
      const devicePreferencesStore = useDevicePreferencesStore();
      const deviceId = devicesStore.activeDevice?.id;
      if (deviceId) {
        devicePreferencesStore.addRecentChannel(deviceId, channel.id);
      }

      return vlcSession;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Failed to start VLC stream:", e);
      return null;
    }
  }

  /**
   * Stop a specific VLC stream by session ID
   */
  async function stopVlcStream(sessionId: string): Promise<void> {
    try {
      await invoke("stop_stream", { sessionId });
    } catch (e) {
      console.error("Failed to stop VLC stream:", e);
    }
    vlcStreams.value.delete(sessionId);
  }

  /**
   * Stop all active VLC streams
   */
  async function stopAllVlcStreams(): Promise<void> {
    const sessions = Array.from(vlcStreams.value.keys());
    await Promise.all(sessions.map((id) => stopVlcStream(id)));
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
    vlcStreams,
    isLoading,
    error,
    // Getters
    hasChannels,
    isStreaming,
    activeVlcStreams,
    vlcStreamCount,
    channelNumber,
    sortedChannels,
    // Actions
    fetchChannels,
    startStream,
    stopStream,
    startVlcStream,
    stopVlcStream,
    stopAllVlcStreams,
    selectChannel,
    clearSelection,
  };
});
