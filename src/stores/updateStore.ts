import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";

interface UpdateInfo {
  available: boolean;
  latest_version: string;
  download_url: string;
  release_notes: string | null;
}

const CHECK_INTERVAL_MS = 60 * 60 * 1000; // 1 hour
const RESURFACE_MS = 5 * 60 * 60 * 1000; // 5 hours

export const useUpdateStore = defineStore("update", () => {
  const updateAvailable = ref(false);
  const latestVersion = ref("");
  const downloadUrl = ref("");
  const dismissed = ref(false);

  let intervalId: ReturnType<typeof setInterval> | null = null;
  let resurfaceId: ReturnType<typeof setTimeout> | null = null;

  async function checkForUpdate(showToast = true) {
    try {
      const info = await invoke<UpdateInfo>("check_for_update");
      if (info.available) {
        const isNew = !updateAvailable.value;
        updateAvailable.value = true;
        latestVersion.value = info.latest_version;
        downloadUrl.value = info.download_url;

        if (isNew && showToast) {
          const { info: toastInfo } = useToast();
          toastInfo(`Glogger v${info.latest_version} is available!`);
        }
      }
    } catch {
      // Silently ignore — no network, rate-limited, etc.
    }
  }

  function startPolling() {
    // Check once shortly after startup (give the app a moment to settle)
    setTimeout(() => checkForUpdate(), 5000);
    intervalId = setInterval(() => checkForUpdate(), CHECK_INTERVAL_MS);
  }

  function stopPolling() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
  }

  function dismiss() {
    dismissed.value = true;
    if (resurfaceId) clearTimeout(resurfaceId);
    resurfaceId = setTimeout(() => {
      dismissed.value = false;
    }, RESURFACE_MS);
  }

  return {
    updateAvailable,
    latestVersion,
    downloadUrl,
    dismissed,
    checkForUpdate,
    startPolling,
    stopPolling,
    dismiss,
  };
});
