import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../stores/gameDataStore";

export function useGameIcon() {
  const store = useGameDataStore();
  const iconSrc = ref<string | null>(null);
  const iconLoading = ref(false);
  let loadedIconId: number | null = null;

  async function loadIcon(iconId: number | null | undefined) {
    if (iconId == null) return;
    if (iconId === loadedIconId) return;
    loadedIconId = iconId;

    iconLoading.value = true;
    try {
      const path = await store.getIconPath(iconId);
      iconSrc.value = convertFileSrc(path);
    } catch (e) {
      console.warn(`Icon load failed for id ${iconId}:`, e);
      iconSrc.value = null;
    } finally {
      iconLoading.value = false;
    }
  }

  return { iconSrc, iconLoading, loadIcon };
}
