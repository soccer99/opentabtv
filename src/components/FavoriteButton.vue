<script setup lang="ts">
import { computed } from "vue";
import { useDevicePreferencesStore } from "@/stores/devicePreferences";
import { useDevicesStore } from "@/stores/devices";

const props = withDefaults(
  defineProps<{
    channelId: string;
    size?: "sm" | "md" | "lg";
  }>(),
  {
    size: "md",
  }
);

const devicePreferencesStore = useDevicePreferencesStore();
const devicesStore = useDevicesStore();

const isFavorited = computed(() => {
  const deviceId = devicesStore.activeDevice?.id;
  if (!deviceId) return false;
  return devicePreferencesStore.isFavorite(deviceId, props.channelId);
});

const sizeClasses = computed(() => {
  switch (props.size) {
    case "sm":
      return "w-4 h-4";
    case "lg":
      return "w-6 h-6";
    default:
      return "w-5 h-5";
  }
});

async function handleClick() {
  const deviceId = devicesStore.activeDevice?.id;
  if (!deviceId) return;
  await devicePreferencesStore.toggleFavorite(deviceId, props.channelId);
}
</script>

<template>
  <button
    @click.stop="handleClick"
    class="p-1 rounded-lg transition-colors hover:bg-white/10"
    :class="isFavorited ? 'text-accent' : 'text-text-muted hover:text-accent'"
    :aria-label="isFavorited ? 'Remove from favorites' : 'Add to favorites'"
  >
    <!-- Filled heart -->
    <svg
      v-if="isFavorited"
      :class="sizeClasses"
      viewBox="0 0 24 24"
    >
      <path
        fill="currentColor"
        d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
      />
    </svg>
    <!-- Outline heart -->
    <svg
      v-else
      :class="sizeClasses"
      viewBox="0 0 24 24"
    >
      <path
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
      />
    </svg>
  </button>
</template>
