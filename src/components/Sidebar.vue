<script setup lang="ts">
import { computed } from "vue";
import { useRoute, RouterLink } from "vue-router";
import { useDevicesStore } from "@/stores/devices";

defineProps<{
  isCollapsed: boolean;
  isMobileOpen: boolean;
}>();

const emit = defineEmits<{
  toggleCollapse: [];
  closeMobile: [];
}>();

const route = useRoute();
const devicesStore = useDevicesStore();

const navItems = [
  { path: "/", name: "Home", icon: "home" },
  { path: "/live", name: "Live TV", icon: "tv" },
  { path: "/guide", name: "Guide", icon: "calendar" },
  { path: "/recordings", name: "Recordings", icon: "film" },
  { path: "/settings", name: "Settings", icon: "settings" },
];

const isActive = (path: string) => {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
};

// Connection status
const connectionStatus = computed(() => {
  if (devicesStore.connectionState === "connecting") {
    return {
      color: "bg-warning animate-pulse",
      text: "Connecting",
      subtext: "Please wait...",
    };
  }
  if (devicesStore.isConnected) {
    return {
      color: "bg-success",
      text: devicesStore.activeDevice?.name || "Connected",
      subtext: devicesStore.activeDevice?.localIp || "Device connected",
    };
  }
  return {
    color: "bg-text-muted",
    text: "No Device",
    subtext: "Not connected",
  };
});

function handleNavClick() {
  // Close mobile menu when navigating
  emit("closeMobile");
}
</script>

<template>
  <!-- Sidebar Container -->
  <aside
    class="fixed top-0 left-0 h-screen z-50 flex flex-col
           bg-surface-1/95 backdrop-blur-xl border-r border-white/5
           transition-all duration-300 ease-out"
    :class="[
      // Desktop behavior
      isCollapsed ? 'lg:w-[72px]' : 'lg:w-60',
      // Mobile behavior: slide in from left
      isMobileOpen
        ? 'translate-x-0 w-72'
        : '-translate-x-full lg:translate-x-0',
    ]"
  >
    <!-- Logo Area -->
    <div
      class="h-16 flex items-center border-b border-white/5 px-4"
      :class="isCollapsed ? 'lg:justify-center' : 'justify-between'"
    >
      <RouterLink
        to="/"
        class="flex items-center gap-3"
        @click="handleNavClick"
      >
        <!-- Logo icon -->
        <div class="w-10 h-10 rounded-xl bg-accent flex items-center justify-center shrink-0">
          <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.14 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
          </svg>
        </div>
        <!-- Logo text - hidden when collapsed -->
        <Transition name="fade-slide">
          <div v-if="!isCollapsed" class="lg:block hidden">
            <h1 class="text-lg font-bold text-text-primary leading-tight">OpenTabTV</h1>
            <p class="text-xs text-text-muted">DVR Manager</p>
          </div>
        </Transition>
        <!-- Always show on mobile -->
        <div class="lg:hidden">
          <h1 class="text-lg font-bold text-text-primary leading-tight">OpenTabTV</h1>
          <p class="text-xs text-text-muted">DVR Manager</p>
        </div>
      </RouterLink>

      <!-- Collapse toggle (desktop only) -->
      <button
        @click="emit('toggleCollapse')"
        class="hidden lg:flex p-2 rounded-lg hover:bg-white/5 text-text-muted hover:text-text-primary transition-colors"
        :class="isCollapsed ? 'lg:hidden' : ''"
        aria-label="Collapse sidebar"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
        </svg>
      </button>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 p-3 space-y-1 overflow-y-auto">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        @click="handleNavClick"
        class="flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-200 group"
        :class="[
          isActive(item.path)
            ? 'bg-accent text-white'
            : 'text-text-secondary hover:bg-white/5 hover:text-text-primary',
          isCollapsed ? 'lg:justify-center lg:px-0' : ''
        ]"
        :title="isCollapsed ? item.name : undefined"
      >
        <!-- Icons -->
        <span
          class="w-6 h-6 flex items-center justify-center shrink-0"
          :class="isActive(item.path) ? 'text-white' : 'text-text-muted group-hover:text-text-primary'"
        >
          <!-- Home -->
          <svg v-if="item.icon === 'home'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
          <!-- TV -->
          <svg v-else-if="item.icon === 'tv'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
          <!-- Calendar -->
          <svg v-else-if="item.icon === 'calendar'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <!-- Film -->
          <svg v-else-if="item.icon === 'film'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z" />
          </svg>
          <!-- Settings -->
          <svg v-else-if="item.icon === 'settings'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </span>

        <!-- Label - hidden when collapsed on desktop -->
        <Transition name="fade-slide">
          <span
            v-if="!isCollapsed"
            class="font-medium text-sm whitespace-nowrap hidden lg:block"
          >
            {{ item.name }}
          </span>
        </Transition>
        <!-- Always show label on mobile -->
        <span class="font-medium text-sm lg:hidden">{{ item.name }}</span>
      </RouterLink>
    </nav>

    <!-- Expand button (when collapsed) -->
    <button
      v-if="isCollapsed"
      @click="emit('toggleCollapse')"
      class="hidden lg:flex mx-3 mb-3 p-2.5 rounded-lg bg-surface-2 hover:bg-surface-3 text-text-muted hover:text-text-primary justify-center transition-colors"
      aria-label="Expand sidebar"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
      </svg>
    </button>

    <!-- Device Status -->
    <div class="p-3 border-t border-white/5">
      <div
        class="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-surface-2/50"
        :class="isCollapsed ? 'lg:justify-center lg:px-0' : ''"
      >
        <!-- Status indicator -->
        <div
          class="w-2.5 h-2.5 rounded-full shrink-0"
          :class="connectionStatus.color"
        />
        <!-- Status text - hidden when collapsed -->
        <Transition name="fade-slide">
          <div v-if="!isCollapsed" class="min-w-0 hidden lg:block">
            <p class="text-sm font-medium text-text-primary truncate">
              {{ connectionStatus.text }}
            </p>
            <p class="text-xs text-text-muted truncate">
              {{ connectionStatus.subtext }}
            </p>
          </div>
        </Transition>
        <!-- Always show on mobile -->
        <div class="min-w-0 lg:hidden">
          <p class="text-sm font-medium text-text-primary truncate">
            {{ connectionStatus.text }}
          </p>
          <p class="text-xs text-text-muted truncate">
            {{ connectionStatus.subtext }}
          </p>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
/* Fade slide transition for collapsing elements */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}
</style>
