import { useToastStore } from "@/stores/toast";

export function useToast() {
  const store = useToastStore();

  return {
    success: (message: string, duration?: number) =>
      store.addToast(message, "success", duration),

    error: (message: string, duration?: number) =>
      store.addToast(message, "error", duration),

    warning: (message: string, duration?: number) =>
      store.addToast(message, "warning", duration),

    info: (message: string, duration?: number) =>
      store.addToast(message, "info", duration),

    remove: (id: string) => store.removeToast(id),

    clear: () => store.clearAll(),
  };
}
