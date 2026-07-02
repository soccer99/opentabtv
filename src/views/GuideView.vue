<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { useDevicesStore } from "@/stores/devices";
import { useChannelsStore, type Channel } from "@/stores/channels";
import { useGuideStore, type GuideProgram } from "@/stores/guide";

const devicesStore = useDevicesStore();
const channelsStore = useChannelsStore();
const guideStore = useGuideStore();

const { isConnected } = storeToRefs(devicesStore);
const { sortedChannels } = storeToRefs(channelsStore);
const {
  isLoading,
  error,
  currentTime,
  viewStartTime,
  viewEndTime,
  viewHours,
  visibleAiringsByChannel,
  hasAirings,
} = storeToRefs(guideStore);

// Selected program for details panel
const selectedProgram = ref<GuideProgram | null>(null);

// Generate time slots for the header
const timeSlots = computed(() => {
  const slots: Date[] = [];
  const start = new Date(viewStartTime.value);
  // 30-minute slots
  for (let i = 0; i < viewHours.value * 2; i++) {
    const slot = new Date(start);
    slot.setMinutes(slot.getMinutes() + i * 30);
    slots.push(slot);
  }
  return slots;
});

// Format time for display
function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}

// Format date for header
function formatDate(date: Date): string {
  const today = new Date();
  const tomorrow = new Date(today);
  tomorrow.setDate(tomorrow.getDate() + 1);

  if (date.toDateString() === today.toDateString()) {
    return "Today";
  } else if (date.toDateString() === tomorrow.toDateString()) {
    return "Tomorrow";
  }
  return date.toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" });
}

// Calculate program position and width in the grid
function getProgramStyle(program: GuideProgram): Record<string, string> {
  const viewStart = viewStartTime.value.getTime();
  const viewEnd = viewEndTime.value.getTime();
  const viewDuration = viewEnd - viewStart;

  const progStart = Math.max(program.startTime.getTime(), viewStart);
  const progEnd = Math.min(program.endTime.getTime(), viewEnd);

  const leftPercent = ((progStart - viewStart) / viewDuration) * 100;
  const widthPercent = ((progEnd - progStart) / viewDuration) * 100;

  return {
    left: `${leftPercent}%`,
    width: `${Math.max(widthPercent, 1)}%`,
  };
}

// Calculate "now" indicator position
const nowIndicatorStyle = computed(() => {
  const viewStart = viewStartTime.value.getTime();
  const viewEnd = viewEndTime.value.getTime();
  const now = currentTime.value.getTime();

  if (now < viewStart || now > viewEnd) {
    return { display: "none" };
  }

  const viewDuration = viewEnd - viewStart;
  const leftPercent = ((now - viewStart) / viewDuration) * 100;

  return {
    left: `${leftPercent}%`,
  };
});

// Check if we can go to previous time
const canGoPrevious = computed(() => {
  const now = new Date();
  now.setHours(now.getHours() - 24); // Allow going back 24 hours
  return viewStartTime.value > now;
});

// Check if we can go to next time
const canGoNext = computed(() => {
  const maxFuture = new Date();
  maxFuture.setDate(maxFuture.getDate() + 14); // 2 weeks ahead
  return viewStartTime.value < maxFuture;
});

// Check if "now" is in the current view
const isNowInView = computed(() => {
  const now = currentTime.value.getTime();
  return now >= viewStartTime.value.getTime() && now <= viewEndTime.value.getTime();
});

// Get channel number display
function getChannelNumber(channel: Channel): string {
  return channel.minor > 0 ? `${channel.major}.${channel.minor}` : `${channel.major}`;
}

// Get programs for a channel
function getChannelPrograms(channel: Channel): GuideProgram[] {
  return visibleAiringsByChannel.value.get(channel.path) || [];
}

// Select a program
function selectProgram(program: GuideProgram): void {
  selectedProgram.value = program;
}

// Close program details
function closeDetails(): void {
  selectedProgram.value = null;
}

// Format episode info
function formatEpisode(program: GuideProgram): string {
  if (!program.episode) return "";
  const { seasonNumber, number, title } = program.episode;
  let text = "";
  if (seasonNumber && number) {
    text = `S${seasonNumber} E${number}`;
  } else if (number) {
    text = `Episode ${number}`;
  }
  if (title) {
    text += text ? ` - ${title}` : title;
  }
  return text;
}

// Fetch data on mount
onMounted(() => {
  if (isConnected.value) {
    channelsStore.fetchChannels();
    guideStore.fetchAirings();
  }
});

// Cleanup on unmount
onUnmounted(() => {
  guideStore.stopRealtimeUpdates();
});
</script>

<template>
  <div>
    <!-- Header -->
    <div class="mb-6 flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-text-primary">TV Guide</h1>
        <p class="text-text-secondary mt-2">
          <template v-if="isConnected">
            Browse upcoming programs and schedule recordings.
          </template>
          <template v-else>
            Connect to a Tablo device to view the program guide.
          </template>
        </p>
      </div>
      <div v-if="isConnected" class="flex items-center gap-2">
        <!-- Date display -->
        <span class="text-text-primary font-medium mr-2">
          {{ formatDate(viewStartTime) }}
        </span>

        <!-- Today button -->
        <button
          class="px-4 py-2 glass hover:bg-white/10 rounded-lg transition-colors"
          :class="{ 'bg-accent/20 text-accent': isNowInView }"
          @click="guideStore.goToNow()"
        >
          <span class="text-text-primary">Now</span>
        </button>

        <!-- Previous button -->
        <button
          class="p-2 glass hover:bg-white/10 rounded-lg transition-colors disabled:opacity-50"
          :disabled="!canGoPrevious"
          @click="guideStore.goToPrevious()"
        >
          <svg
            class="w-5 h-5 text-text-primary"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </button>

        <!-- Next button -->
        <button
          class="p-2 glass hover:bg-white/10 rounded-lg transition-colors disabled:opacity-50"
          :disabled="!canGoNext"
          @click="guideStore.goToNext()"
        >
          <svg
            class="w-5 h-5 text-text-primary"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 5l7 7-7 7"
            />
          </svg>
        </button>

        <!-- Refresh button -->
        <button
          class="p-2 glass hover:bg-white/10 rounded-lg transition-colors"
          :class="{ 'animate-spin': isLoading }"
          :disabled="isLoading"
          @click="guideStore.fetchAirings()"
        >
          <svg
            class="w-5 h-5 text-text-primary"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
        </button>
      </div>
    </div>

    <!-- Not Connected State -->
    <div
      v-if="!isConnected"
      class="glass rounded-xl p-12 text-center"
    >
      <svg
        class="w-16 h-16 mx-auto text-text-muted mb-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
        />
      </svg>
      <h2 class="text-xl font-semibold text-text-primary mb-2">No Device Connected</h2>
      <p class="text-text-muted max-w-md mx-auto">
        Connect to a Tablo DVR device from the Home page to view the TV guide
        and see what's on now and upcoming.
      </p>
    </div>

    <!-- Loading State -->
    <div
      v-else-if="isLoading && !hasAirings"
      class="glass rounded-xl p-12 text-center"
    >
      <div class="w-12 h-12 border-4 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
      <p class="text-text-muted">Loading guide data...</p>
    </div>

    <!-- Error State -->
    <div
      v-else-if="error"
      class="glass rounded-xl p-12 text-center"
    >
      <svg
        class="w-16 h-16 mx-auto text-red-400 mb-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        />
      </svg>
      <h2 class="text-xl font-semibold text-text-primary mb-2">Error Loading Guide</h2>
      <p class="text-text-muted max-w-md mx-auto mb-4">{{ error }}</p>
      <button
        class="px-4 py-2 bg-accent hover:bg-accent/80 rounded-lg transition-colors text-white"
        @click="guideStore.fetchAirings()"
      >
        Try Again
      </button>
    </div>

    <!-- Guide Grid -->
    <div v-else class="glass overflow-hidden rounded-xl">
      <!-- Time Header -->
      <div class="flex border-b border-white/10 sticky top-0 z-10 bg-surface-1/95 backdrop-blur">
        <div class="w-32 flex-shrink-0 p-3 bg-surface-2">
          <span class="text-text-muted text-sm">Channel</span>
        </div>
        <div class="flex-1 relative">
          <div class="flex">
            <div
              v-for="slot in timeSlots"
              :key="slot.getTime()"
              class="flex-1 p-3 border-l border-white/10 text-center min-w-[80px]"
            >
              <span class="text-text-muted text-sm">{{ formatTime(slot) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty Guide State -->
      <div
        v-if="sortedChannels.length === 0"
        class="p-12 text-center"
      >
        <p class="text-text-muted">No channels available.</p>
      </div>

      <!-- Channel Rows -->
      <div v-else class="relative">
        <div
          v-for="channel in sortedChannels"
          :key="channel.id"
          class="flex border-b border-white/10 last:border-b-0 min-h-[60px]"
        >
          <!-- Channel Info -->
          <div class="w-32 flex-shrink-0 p-3 bg-surface-1 flex flex-col justify-center">
            <p class="font-medium text-text-primary">{{ getChannelNumber(channel) }}</p>
            <p class="text-xs text-text-muted truncate">{{ channel.callSign }}</p>
          </div>

          <!-- Programs Grid -->
          <div class="flex-1 relative h-[60px]">
            <!-- Program blocks -->
            <div
              v-for="program in getChannelPrograms(channel)"
              :key="program.id"
              class="absolute top-0 bottom-0 p-2 border-l border-white/10 hover:bg-white/10 cursor-pointer transition-colors overflow-hidden group"
              :class="{
                'bg-accent/10': program.startTime <= currentTime && program.endTime > currentTime,
              }"
              :style="getProgramStyle(program)"
              @click="selectProgram(program)"
            >
              <p class="text-sm text-text-primary truncate font-medium">
                {{ program.title }}
              </p>
              <p class="text-xs text-text-muted truncate">
                {{ formatTime(program.startTime) }} - {{ formatTime(program.endTime) }}
              </p>
            </div>

            <!-- No programs placeholder -->
            <div
              v-if="getChannelPrograms(channel).length === 0"
              class="absolute inset-0 flex items-center justify-center text-text-muted text-sm"
            >
              No guide data
            </div>
          </div>
        </div>

        <!-- Now indicator line -->
        <div
          v-if="isNowInView"
          class="absolute top-0 bottom-0 w-0.5 bg-accent z-20 pointer-events-none"
          :style="{ ...nowIndicatorStyle, left: `calc(128px + ${nowIndicatorStyle.left})` }"
        >
          <div class="absolute -top-1 -left-1.5 w-3 h-3 bg-accent rounded-full"></div>
        </div>
      </div>
    </div>

    <!-- Program Details Modal -->
    <Teleport to="body">
      <div
        v-if="selectedProgram"
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4"
        @click.self="closeDetails"
      >
        <div class="glass rounded-xl max-w-lg w-full p-6 shadow-2xl">
          <!-- Header -->
          <div class="flex items-start justify-between mb-4">
            <div>
              <h2 class="text-xl font-bold text-text-primary">
                {{ selectedProgram.title }}
              </h2>
              <p v-if="formatEpisode(selectedProgram)" class="text-text-secondary mt-1">
                {{ formatEpisode(selectedProgram) }}
              </p>
            </div>
            <button
              class="p-1 hover:bg-white/10 rounded-lg transition-colors"
              @click="closeDetails"
            >
              <svg class="w-6 h-6 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Time info -->
          <div class="flex items-center gap-4 mb-4 text-sm text-text-muted">
            <span>{{ formatTime(selectedProgram.startTime) }} - {{ formatTime(selectedProgram.endTime) }}</span>
            <span class="px-2 py-0.5 bg-white/10 rounded text-xs capitalize">
              {{ selectedProgram.airingType }}
            </span>
          </div>

          <!-- Description -->
          <p v-if="selectedProgram.description" class="text-text-secondary mb-4">
            {{ selectedProgram.description }}
          </p>
          <p v-else class="text-text-muted italic mb-4">
            No description available.
          </p>

          <!-- Genres -->
          <div v-if="selectedProgram.genres?.length" class="flex flex-wrap gap-2 mb-4">
            <span
              v-for="genre in selectedProgram.genres"
              :key="genre"
              class="px-2 py-1 bg-white/5 rounded text-xs text-text-muted"
            >
              {{ genre }}
            </span>
          </div>

          <!-- Actions (placeholder for future record button) -->
          <div class="flex justify-end gap-2 pt-4 border-t border-white/10">
            <button
              class="px-4 py-2 glass hover:bg-white/10 rounded-lg transition-colors"
              @click="closeDetails"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
