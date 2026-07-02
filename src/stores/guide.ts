import { ref, computed, watch } from "vue";
import { defineStore, storeToRefs } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useDevicesStore } from "./devices";

export interface GuideAiring {
  id: string;
  path: string;
  channelPath: string;
  title: string;
  description?: string;
  startTime: string; // ISO string from backend
  endTime: string;
  duration: number;
  airingType: "series" | "movie" | "sports" | "program" | "manual" | "unknown";
  episode?: {
    number: number;
    seasonNumber: number;
    title?: string;
  };
  genres?: string[];
}

export interface GuideProgram extends Omit<GuideAiring, "startTime" | "endTime"> {
  startTime: Date;
  endTime: Date;
}

export const useGuideStore = defineStore("guide", () => {
  const devicesStore = useDevicesStore();
  const { isConnected } = storeToRefs(devicesStore);

  // State
  const airings = ref<GuideProgram[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const currentTime = ref(new Date());
  const viewStartTime = ref(getHourStart(new Date()));
  const viewHours = ref(3); // Show 3 hours at a time

  // Timer for realtime updates
  let timeUpdateInterval: ReturnType<typeof setInterval> | null = null;

  // Getters
  const viewEndTime = computed(() => {
    const end = new Date(viewStartTime.value);
    end.setHours(end.getHours() + viewHours.value);
    return end;
  });

  const hasAirings = computed(() => airings.value.length > 0);

  // Get airings grouped by channel path
  const airingsByChannel = computed(() => {
    const grouped = new Map<string, GuideProgram[]>();
    for (const airing of airings.value) {
      const key = airing.channelPath;
      if (!grouped.has(key)) {
        grouped.set(key, []);
      }
      grouped.get(key)!.push(airing);
    }
    // Sort each channel's airings by start time
    for (const programs of grouped.values()) {
      programs.sort((a, b) => a.startTime.getTime() - b.startTime.getTime());
    }
    return grouped;
  });

  // Get visible airings (within view window)
  const visibleAirings = computed(() => {
    const start = viewStartTime.value.getTime();
    const end = viewEndTime.value.getTime();

    return airings.value.filter((airing) => {
      const airingStart = airing.startTime.getTime();
      const airingEnd = airing.endTime.getTime();
      // Airing overlaps with view window
      return airingStart < end && airingEnd > start;
    });
  });

  // Get visible airings grouped by channel
  const visibleAiringsByChannel = computed(() => {
    const start = viewStartTime.value.getTime();
    const end = viewEndTime.value.getTime();
    const grouped = new Map<string, GuideProgram[]>();

    for (const airing of airings.value) {
      const airingStart = airing.startTime.getTime();
      const airingEnd = airing.endTime.getTime();

      // Check if airing overlaps with view window
      if (airingStart < end && airingEnd > start) {
        const key = airing.channelPath;
        if (!grouped.has(key)) {
          grouped.set(key, []);
        }
        grouped.get(key)!.push(airing);
      }
    }

    // Sort each channel's airings by start time
    for (const programs of grouped.values()) {
      programs.sort((a, b) => a.startTime.getTime() - b.startTime.getTime());
    }

    return grouped;
  });

  // Get channels that have airings
  const channelsWithAirings = computed(() => {
    return Array.from(airingsByChannel.value.keys());
  });

  // Get what's on now for a channel
  function getNowPlaying(channelPath: string): GuideProgram | undefined {
    const now = currentTime.value.getTime();
    const channelAirings = airingsByChannel.value.get(channelPath) || [];
    return channelAirings.find(
      (a) => a.startTime.getTime() <= now && a.endTime.getTime() > now
    );
  }

  // Get what's on next for a channel
  function getUpNext(channelPath: string): GuideProgram | undefined {
    const now = currentTime.value.getTime();
    const channelAirings = airingsByChannel.value.get(channelPath) || [];
    return channelAirings.find((a) => a.startTime.getTime() > now);
  }

  // Actions
  async function fetchAirings(): Promise<void> {
    if (!isConnected.value) {
      airings.value = [];
      return;
    }

    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<GuideAiring[]>("get_guide_airings");

      // Convert ISO strings to Date objects
      airings.value = result.map((airing) => ({
        ...airing,
        startTime: new Date(airing.startTime),
        endTime: new Date(airing.endTime),
      }));
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Failed to fetch guide airings:", e);
    } finally {
      isLoading.value = false;
    }
  }

  // Navigation
  function goToNow(): void {
    viewStartTime.value = getHourStart(new Date());
  }

  function goToPrevious(): void {
    const newStart = new Date(viewStartTime.value);
    newStart.setHours(newStart.getHours() - viewHours.value);
    viewStartTime.value = newStart;
  }

  function goToNext(): void {
    const newStart = new Date(viewStartTime.value);
    newStart.setHours(newStart.getHours() + viewHours.value);
    viewStartTime.value = newStart;
  }

  function goToTime(time: Date): void {
    viewStartTime.value = getHourStart(time);
  }

  // Start realtime updates
  function startRealtimeUpdates(): void {
    if (timeUpdateInterval) return;

    // Update current time every minute
    timeUpdateInterval = setInterval(() => {
      currentTime.value = new Date();
    }, 60000); // 1 minute
  }

  // Stop realtime updates
  function stopRealtimeUpdates(): void {
    if (timeUpdateInterval) {
      clearInterval(timeUpdateInterval);
      timeUpdateInterval = null;
    }
  }

  // Clear data
  function clear(): void {
    airings.value = [];
    error.value = null;
  }

  // Helper to get start of hour
  function getHourStart(date: Date): Date {
    const start = new Date(date);
    start.setMinutes(0, 0, 0);
    return start;
  }

  // Watch for connection changes
  watch(isConnected, (connected) => {
    if (connected) {
      fetchAirings();
      startRealtimeUpdates();
    } else {
      clear();
      stopRealtimeUpdates();
    }
  });

  // Initial setup if already connected
  if (isConnected.value) {
    fetchAirings();
    startRealtimeUpdates();
  }

  return {
    // State
    airings,
    isLoading,
    error,
    currentTime,
    viewStartTime,
    viewHours,
    // Getters
    viewEndTime,
    hasAirings,
    airingsByChannel,
    visibleAirings,
    visibleAiringsByChannel,
    channelsWithAirings,
    // Methods
    getNowPlaying,
    getUpNext,
    fetchAirings,
    goToNow,
    goToPrevious,
    goToNext,
    goToTime,
    startRealtimeUpdates,
    stopRealtimeUpdates,
    clear,
  };
});
