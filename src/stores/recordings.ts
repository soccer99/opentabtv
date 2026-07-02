import { ref, computed, watch } from "vue";
import { defineStore, storeToRefs } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useDevicesStore } from "./devices";
import type { StreamSession } from "./channels";
import type { RecordingType, RecordingState } from "@/types";

// Re-export types for convenience
export type { RecordingType, RecordingState };

// Backend returns ISO string, we convert to Date for display
interface RecordingFromBackend {
  id: string;
  path: string;
  title: string;
  recordingType: RecordingType;
  description?: string;
  recordedAt: string;
  duration: number;
  size: number;
  channelId: string;
  episode?: {
    number: number;
    seasonNumber: number;
    title?: string;
  };
  videoDetails: {
    width: number;
    height: number;
    state: RecordingState;
    hasComskip: boolean;
  };
  userInfo: {
    watched: boolean;
    protected: boolean;
    position: number;
  };
}

export interface RecordingDisplay extends Omit<RecordingFromBackend, "recordedAt"> {
  recordedAt: Date;
}

export const useRecordingsStore = defineStore("recordings", () => {
  const devicesStore = useDevicesStore();
  const { isConnected } = storeToRefs(devicesStore);

  // State
  const recordings = ref<RecordingDisplay[]>([]);
  const selectedRecording = ref<RecordingDisplay | null>(null);
  const currentStream = ref<StreamSession | null>(null);
  const isLoading = ref(false);
  const isStartingPlayback = ref(false);
  const error = ref<string | null>(null);
  const playbackError = ref<string | null>(null);
  const filterType = ref<RecordingType | "all">("all");

  // Getters
  const hasRecordings = computed(() => recordings.value.length > 0);
  const isPlaying = computed(() => currentStream.value !== null);

  const filteredRecordings = computed(() => {
    if (filterType.value === "all") {
      return recordings.value;
    }
    return recordings.value.filter((r) => r.recordingType === filterType.value);
  });

  const recordingsByType = computed(() => {
    const grouped = new Map<RecordingType, RecordingDisplay[]>();
    for (const recording of recordings.value) {
      const type = recording.recordingType;
      if (!grouped.has(type)) {
        grouped.set(type, []);
      }
      grouped.get(type)!.push(recording);
    }
    return grouped;
  });

  const totalCount = computed(() => recordings.value.length);

  const totalSize = computed(() =>
    recordings.value.reduce((sum, r) => sum + r.size, 0)
  );

  const unwatchedCount = computed(() =>
    recordings.value.filter((r) => !r.userInfo.watched).length
  );

  // Actions
  async function fetchRecordings(): Promise<void> {
    if (!isConnected.value) {
      recordings.value = [];
      return;
    }

    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<RecordingFromBackend[]>("get_recordings");

      // Convert ISO strings to Date objects
      recordings.value = result.map((recording) => ({
        ...recording,
        recordedAt: new Date(recording.recordedAt),
      }));
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Failed to fetch recordings:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function playRecording(recording: RecordingDisplay): Promise<StreamSession | null> {
    // Stop existing stream if any
    if (currentStream.value) {
      await stopPlayback();
    }

    selectedRecording.value = recording;
    playbackError.value = null;
    isStartingPlayback.value = true;

    try {
      const session = await invoke<StreamSession>("watch_recording", {
        recordingPath: recording.path,
      });
      currentStream.value = session;
      return session;
    } catch (e) {
      playbackError.value = e instanceof Error ? e.message : String(e);
      console.error("Failed to start recording playback:", e);
      return null;
    } finally {
      isStartingPlayback.value = false;
    }
  }

  async function stopPlayback(): Promise<void> {
    if (currentStream.value) {
      try {
        await invoke("stop_stream", { sessionId: currentStream.value.sessionId });
      } catch (e) {
        console.error("Failed to stop playback:", e);
      }
      currentStream.value = null;
    }
  }

  function selectRecording(recording: RecordingDisplay): void {
    selectedRecording.value = recording;
  }

  function clearSelection(): void {
    selectedRecording.value = null;
    currentStream.value = null;
  }

  function setFilter(type: RecordingType | "all"): void {
    filterType.value = type;
  }

  function clear(): void {
    recordings.value = [];
    selectedRecording.value = null;
    currentStream.value = null;
    error.value = null;
    playbackError.value = null;
    isStartingPlayback.value = false;
  }

  // Watch for connection changes
  watch(isConnected, (connected) => {
    if (connected) {
      fetchRecordings();
    } else {
      clear();
    }
  });

  // Initial fetch if already connected
  if (isConnected.value) {
    fetchRecordings();
  }

  return {
    // State
    recordings,
    selectedRecording,
    currentStream,
    isLoading,
    isStartingPlayback,
    error,
    playbackError,
    filterType,
    // Getters
    hasRecordings,
    isPlaying,
    filteredRecordings,
    recordingsByType,
    totalCount,
    totalSize,
    unwatchedCount,
    // Actions
    fetchRecordings,
    playRecording,
    stopPlayback,
    selectRecording,
    clearSelection,
    setFilter,
    clear,
  };
});
