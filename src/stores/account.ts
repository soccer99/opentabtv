import { ref, computed } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { TabloAccount, LoginCredentials, TabloDevice } from "@/types";

export const useAccountStore = defineStore("account", () => {
  // State
  const account = ref<TabloAccount | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const hasSavedCredentials = ref(false);

  // Getters
  const isLoggedIn = computed(() => account.value !== null);
  const email = computed(() => account.value?.email ?? null);
  const cloudDevices = computed(() => account.value?.devices ?? []);

  // Actions
  async function login(email: string, password: string): Promise<boolean> {
    isLoading.value = true;
    error.value = null;

    try {
      const result = await invoke<TabloAccount>("login", { email, password });
      account.value = result;
      return true;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Login failed:", e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  async function logout(): Promise<void> {
    try {
      await invoke("logout");
    } catch (e) {
      console.error("Logout error:", e);
    }
    account.value = null;
  }

  async function checkLoginStatus(): Promise<boolean> {
    try {
      const loggedIn = await invoke<boolean>("is_logged_in");
      if (!loggedIn) {
        account.value = null;
      }
      return loggedIn;
    } catch (e) {
      console.error("Failed to check login status:", e);
      return false;
    }
  }

  async function saveCredentials(email: string, password: string): Promise<void> {
    try {
      await invoke("save_credentials", { email, password });
      hasSavedCredentials.value = true;
    } catch (e) {
      console.error("Failed to save credentials:", e);
      throw e;
    }
  }

  async function loadSavedCredentials(): Promise<LoginCredentials | null> {
    try {
      const creds = await invoke<LoginCredentials | null>("load_credentials");
      hasSavedCredentials.value = creds !== null;
      return creds;
    } catch (e) {
      console.error("Failed to load credentials:", e);
      return null;
    }
  }

  async function clearCredentials(): Promise<void> {
    try {
      await invoke("clear_credentials");
      hasSavedCredentials.value = false;
    } catch (e) {
      console.error("Failed to clear credentials:", e);
    }
  }

  async function checkSavedCredentials(): Promise<boolean> {
    try {
      const has = await invoke<boolean>("has_saved_credentials");
      hasSavedCredentials.value = has;
      return has;
    } catch (e) {
      console.error("Failed to check saved credentials:", e);
      return false;
    }
  }

  async function discoverCloudDevices(): Promise<TabloDevice[]> {
    if (!isLoggedIn.value) {
      throw new Error("Must be logged in to discover cloud devices");
    }

    isLoading.value = true;
    error.value = null;

    try {
      const devices = await invoke<TabloDevice[]>("discover_cloud_devices");
      return devices;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      console.error("Cloud discovery failed:", e);
      return [];
    } finally {
      isLoading.value = false;
    }
  }

  async function autoLogin(): Promise<boolean> {
    const creds = await loadSavedCredentials();
    if (creds) {
      return await login(creds.email, creds.password);
    }
    return false;
  }

  return {
    // State
    account,
    isLoading,
    error,
    hasSavedCredentials,
    // Getters
    isLoggedIn,
    email,
    cloudDevices,
    // Actions
    login,
    logout,
    checkLoginStatus,
    saveCredentials,
    loadSavedCredentials,
    clearCredentials,
    checkSavedCredentials,
    discoverCloudDevices,
    autoLogin,
  };
});
