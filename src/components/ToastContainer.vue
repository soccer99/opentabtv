<script setup lang="ts">
import { useToastStore, type ToastType } from "@/stores/toast";

const toastStore = useToastStore();

const icons: Record<ToastType, string> = {
  success: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />`,
  error: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />`,
  warning: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />`,
  info: `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />`,
};

const colorClasses: Record<ToastType, { bg: string; icon: string; border: string }> = {
  success: {
    bg: "bg-success/10",
    icon: "text-success",
    border: "border-success/30",
  },
  error: {
    bg: "bg-error/10",
    icon: "text-error",
    border: "border-error/30",
  },
  warning: {
    bg: "bg-warning/10",
    icon: "text-warning",
    border: "border-warning/30",
  },
  info: {
    bg: "bg-info/10",
    icon: "text-info",
    border: "border-info/30",
  },
};
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed bottom-4 right-4 z-[100] flex flex-col gap-3 pointer-events-none"
      aria-live="polite"
    >
      <TransitionGroup name="toast">
        <div
          v-for="toast in toastStore.toasts"
          :key="toast.id"
          class="pointer-events-auto min-w-[280px] max-w-[400px] glass border rounded-xl shadow-xl"
          :class="[colorClasses[toast.type].bg, colorClasses[toast.type].border]"
        >
          <div class="flex items-start gap-3 p-4">
            <!-- Icon -->
            <div
              class="flex-shrink-0 w-5 h-5"
              :class="colorClasses[toast.type].icon"
            >
              <svg
                class="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                v-html="icons[toast.type]"
              />
            </div>

            <!-- Message -->
            <p class="flex-1 text-sm text-text-primary">
              {{ toast.message }}
            </p>

            <!-- Close button -->
            <button
              @click="toastStore.removeToast(toast.id)"
              class="flex-shrink-0 p-1 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/10 transition-colors"
              aria-label="Dismiss notification"
            >
              <svg
                class="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-enter-active {
  transition: all 0.3s ease-out;
}

.toast-leave-active {
  transition: all 0.2s ease-in;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-move {
  transition: transform 0.3s ease;
}
</style>
