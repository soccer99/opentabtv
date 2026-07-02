<script setup lang="ts">
import { ref, provide, onMounted } from "vue";
import { RouterView } from "vue-router";
import Sidebar from "@/components/Sidebar.vue";
import ToastContainer from "@/components/ToastContainer.vue";
import { useTheme } from "@/composables/useTheme";

// Initialize theme system
const { initialize: initializeTheme } = useTheme();
onMounted(() => {
  initializeTheme();
});

// Sidebar state - shared with sidebar component
const isSidebarCollapsed = ref(false);
const isMobileMenuOpen = ref(false);

// Provide sidebar state to child components
provide("sidebarCollapsed", isSidebarCollapsed);
provide("mobileMenuOpen", isMobileMenuOpen);

function toggleSidebar() {
  isSidebarCollapsed.value = !isSidebarCollapsed.value;
}

function toggleMobileMenu() {
  isMobileMenuOpen.value = !isMobileMenuOpen.value;
}

function closeMobileMenu() {
  isMobileMenuOpen.value = false;
}
</script>

<template>
  <div class="min-h-screen bg-surface-0">
    <!-- Mobile Header (visible on small screens) -->
    <header
      class="lg:hidden fixed top-0 left-0 right-0 h-16 bg-surface-1/95 backdrop-blur-lg border-b border-white/5 z-40 flex items-center justify-between px-4"
    >
      <!-- Mobile menu button -->
      <button
        @click="toggleMobileMenu"
        class="p-2 rounded-lg hover:bg-white/5 transition-colors"
        aria-label="Toggle menu"
      >
        <svg
          class="w-6 h-6 text-text-primary"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            v-if="!isMobileMenuOpen"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 6h16M4 12h16M4 18h16"
          />
          <path
            v-else
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>

      <!-- Logo -->
      <div class="flex items-center gap-2">
        <div class="w-8 h-8 rounded-lg bg-accent flex items-center justify-center">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.14 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
          </svg>
        </div>
        <span class="text-xl font-bold text-text-primary">OpenTabTV</span>
      </div>

      <!-- Spacer for centering -->
      <div class="w-10"></div>
    </header>

    <!-- Mobile menu overlay -->
    <Transition name="fade">
      <div
        v-if="isMobileMenuOpen"
        class="lg:hidden fixed inset-0 bg-black/60 backdrop-blur-sm z-40"
        @click="closeMobileMenu"
      />
    </Transition>

    <!-- Sidebar Navigation -->
    <Sidebar
      :is-collapsed="isSidebarCollapsed"
      :is-mobile-open="isMobileMenuOpen"
      @toggle-collapse="toggleSidebar"
      @close-mobile="closeMobileMenu"
    />

    <!-- Main Content Area -->
    <main
      class="min-h-screen transition-all duration-300 ease-out"
      :class="[
        // Desktop: offset by sidebar width
        isSidebarCollapsed ? 'lg:pl-[72px]' : 'lg:pl-60',
        // Mobile: no offset (full width), add top padding for header
        'pt-16 lg:pt-0'
      ]"
    >
      <div class="p-4 lg:p-6 xl:p-8 max-w-[1400px] mx-auto">
        <RouterView v-slot="{ Component }">
          <Transition name="page" mode="out-in">
            <component :is="Component" />
          </Transition>
        </RouterView>
      </div>
    </main>

    <!-- Toast Notifications -->
    <ToastContainer />
  </div>
</template>

<style scoped>
/* Page transition */
.page-enter-active,
.page-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

/* Fade transition for overlay */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
