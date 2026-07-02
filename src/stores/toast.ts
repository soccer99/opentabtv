import { ref } from "vue";
import { defineStore } from "pinia";

export type ToastType = "success" | "error" | "warning" | "info";

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration: number;
}

export const useToastStore = defineStore("toast", () => {
  const toasts = ref<Toast[]>([]);

  function generateId(): string {
    return `toast-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
  }

  function addToast(
    message: string,
    type: ToastType = "info",
    duration: number = 4000
  ): string {
    const id = generateId();
    const toast: Toast = { id, message, type, duration };

    toasts.value.push(toast);

    // Auto-dismiss after duration
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, duration);
    }

    return id;
  }

  function removeToast(id: string): void {
    const index = toasts.value.findIndex((t) => t.id === id);
    if (index !== -1) {
      toasts.value.splice(index, 1);
    }
  }

  function clearAll(): void {
    toasts.value = [];
  }

  return {
    toasts,
    addToast,
    removeToast,
    clearAll,
  };
});
