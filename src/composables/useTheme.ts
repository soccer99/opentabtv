import { ref, computed, watch } from "vue";
import { LazyStore } from "@tauri-apps/plugin-store";

export type ThemeMode = "dark" | "light" | "system";

// Singleton state to share across components
const themeMode = ref<ThemeMode>("system");
const resolvedTheme = ref<"dark" | "light">("dark");
const isInitialized = ref(false);

// Store instance (lazy loaded)
let store: LazyStore | null = null;

function getStore(): LazyStore {
  if (!store) {
    store = new LazyStore("settings.json");
  }
  return store;
}

function getSystemTheme(): "dark" | "light" {
  if (typeof window === "undefined") return "dark";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme(theme: "dark" | "light") {
  const root = document.documentElement;

  // Remove both classes first
  root.classList.remove("dark", "light");

  // Add the current theme class
  root.classList.add(theme);

  // Update color-scheme for native elements
  root.style.colorScheme = theme;

  resolvedTheme.value = theme;
}

function computeResolvedTheme(mode: ThemeMode): "dark" | "light" {
  if (mode === "system") {
    return getSystemTheme();
  }
  return mode;
}

export function useTheme() {
  // Initialize theme from store and set up listeners
  async function initialize() {
    if (isInitialized.value) return;

    try {
      const s = getStore();
      const savedTheme = await s.get<ThemeMode>("theme");

      if (savedTheme && ["dark", "light", "system"].includes(savedTheme)) {
        themeMode.value = savedTheme;
      }
    } catch (e) {
      console.warn("Failed to load theme from store:", e);
    }

    // Apply initial theme
    applyTheme(computeResolvedTheme(themeMode.value));

    // Listen for system theme changes
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", () => {
      if (themeMode.value === "system") {
        applyTheme(getSystemTheme());
      }
    });

    isInitialized.value = true;
  }

  // Set theme and persist to store
  async function setTheme(mode: ThemeMode) {
    themeMode.value = mode;
    applyTheme(computeResolvedTheme(mode));

    try {
      const s = getStore();
      await s.set("theme", mode);
      await s.save();
    } catch (e) {
      console.warn("Failed to save theme to store:", e);
    }
  }

  // Watch for external changes to themeMode
  watch(themeMode, (newMode) => {
    applyTheme(computeResolvedTheme(newMode));
  });

  // Computed property for checking if dark mode is active
  const isDark = computed(() => resolvedTheme.value === "dark");

  return {
    /** Current theme mode setting ('dark', 'light', or 'system') */
    theme: themeMode,
    /** The actual resolved theme ('dark' or 'light') */
    resolvedTheme,
    /** Whether dark mode is currently active */
    isDark,
    /** Set the theme mode */
    setTheme,
    /** Initialize the theme system (call once in App.vue) */
    initialize,
  };
}
