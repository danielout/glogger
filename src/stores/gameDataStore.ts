import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ItemInfo,
  SkillInfo,
  AbilityInfo,
  RecipeInfo,
  CacheStatus,
} from "../types/gameData";

export type DataStatus = "loading" | "ready" | "error" | "empty";

export const useGameDataStore = defineStore("gameData", () => {
  // ── State ──────────────────────────────────────────────────────────────────
  const status = ref<DataStatus>("loading");
  const errorMessage = ref<string | null>(null);
  const cacheStatus = ref<CacheStatus | null>(null);

  // Icon path cache: icon_id → local filesystem path
  const iconPaths = ref<Record<number, string>>({});

  // ── Startup ────────────────────────────────────────────────────────────────

  // Listen for the Rust-emitted events fired when init_game_data() finishes
  listen<number>("game-data-ready", async (_event) => {
    status.value = "ready";
    errorMessage.value = null;
    await refreshCacheStatus();
  });

  listen<string>("game-data-error", (event) => {
    status.value = "error";
    errorMessage.value = event.payload;
  });

  // Kick off an initial status check immediately (covers the case where
  // game-data-ready fires before we're listening)
  refreshCacheStatus();

  // ── Cache management ───────────────────────────────────────────────────────

  async function refreshCacheStatus() {
    try {
      const s: CacheStatus = await invoke("get_cache_status");
      cacheStatus.value = s;
      if (s.item_count > 0) {
        status.value = "ready";
      }
    } catch (e) {
      console.error("get_cache_status failed:", e);
    }
  }

  async function forceRefreshCdn(): Promise<CacheStatus> {
    status.value = "loading";
    try {
      const s: CacheStatus = await invoke("force_refresh_cdn");
      cacheStatus.value = s;
      status.value = "ready";
      return s;
    } catch (e: any) {
      status.value = "error";
      errorMessage.value = String(e);
      throw e;
    }
  }

  // ── Item queries ───────────────────────────────────────────────────────────

  async function getItem(id: number): Promise<ItemInfo | null> {
    return invoke<ItemInfo | null>("get_item", { id });
  }

  async function getItemByName(name: string): Promise<ItemInfo | null> {
    return invoke<ItemInfo | null>("get_item_by_name", { name });
  }

  async function searchItems(query: string, limit = 20): Promise<ItemInfo[]> {
    return invoke<ItemInfo[]>("search_items", { query, limit });
  }

  // ── Skill queries ──────────────────────────────────────────────────────────

  async function getAllSkills(): Promise<SkillInfo[]> {
    return invoke<SkillInfo[]>("get_all_skills");
  }

  async function getSkillByName(name: string): Promise<SkillInfo | null> {
    return invoke<SkillInfo | null>("get_skill_by_name", { name });
  }

  // ── Ability queries ────────────────────────────────────────────────────────

  async function getAbilitiesForSkill(skill: string): Promise<AbilityInfo[]> {
    return invoke<AbilityInfo[]>("get_abilities_for_skill", { skill });
  }

  // ── Recipe queries ─────────────────────────────────────────────────────────

  async function getRecipesForItem(itemId: number): Promise<RecipeInfo[]> {
    return invoke<RecipeInfo[]>("get_recipes_for_item", { itemId });
  }

  async function getRecipesUsingItem(itemId: number): Promise<RecipeInfo[]> {
    return invoke<RecipeInfo[]>("get_recipes_using_item", { itemId });
  }

  // ── Icon helpers ───────────────────────────────────────────────────────────

  /**
   * Returns a local file path for the given icon ID, fetching from CDN if needed.
   * Results are memoised in iconPaths for the lifetime of the session.
   *
   * Usage in a Vue template:
   *   <img :src="convertFileSrc(iconPath)" />
   * where convertFileSrc is imported from @tauri-apps/api/core.
   */
  async function getIconPath(iconId: number): Promise<string> {
    if (iconPaths.value[iconId]) {
      return iconPaths.value[iconId];
    }
    const path = await invoke<string>("get_icon_path", { iconId });
    iconPaths.value[iconId] = path;
    return path;
  }

  return {
    // State
    status,
    errorMessage,
    cacheStatus,
    iconPaths,
    // Actions
    refreshCacheStatus,
    forceRefreshCdn,
    getItem,
    getItemByName,
    searchItems,
    getAllSkills,
    getSkillByName,
    getAbilitiesForSkill,
    getRecipesForItem,
    getRecipesUsingItem,
    getIconPath,
  };
});
