<script setup lang="ts">
import { RouterLink } from "vue-router";
import { useDevicesStore } from "@/stores/devices";

const devicesStore = useDevicesStore();

defineProps<{
  title?: string;
  description?: string;
}>();
</script>

<template>
  <div class="flex flex-col items-center justify-center min-h-[60vh] text-center px-4">
    <!-- Icon -->
    <div class="w-20 h-20 mb-6 rounded-2xl bg-surface-2 flex items-center justify-center">
      <svg class="w-10 h-10 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
        />
      </svg>
    </div>

    <!-- Title -->
    <h2 class="text-2xl font-bold text-text-primary mb-2">
      {{ title || "No Device Connected" }}
    </h2>

    <!-- Description -->
    <p class="text-text-secondary max-w-md mb-6">
      {{
        description ||
        "Connect to a Tablo device to view live TV, recordings, and guide data."
      }}
    </p>

    <!-- Actions -->
    <div class="flex flex-col sm:flex-row gap-3">
      <RouterLink
        to="/devices"
        class="px-6 py-3 bg-accent hover:bg-accent-hover text-white rounded-xl font-medium transition-colors flex items-center gap-2"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Add Device
      </RouterLink>

      <RouterLink
        v-if="devicesStore.hasRegisteredDevices"
        to="/devices"
        class="px-6 py-3 bg-surface-2 hover:bg-surface-3 text-text-primary rounded-xl font-medium transition-colors"
      >
        Manage Devices
      </RouterLink>
    </div>

    <!-- Quick tip -->
    <p class="text-sm text-text-muted mt-8 max-w-sm">
      <span class="font-medium">Tip:</span> You can scan your network for Legacy devices
      or sign in with your Tablo account for 4th Gen devices.
    </p>
  </div>
</template>
