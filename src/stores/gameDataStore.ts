import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ItemInfo,
  SkillInfo,
  AbilityInfo,
  RecipeInfo,
  QuestInfo,
  NpcInfo,
  EffectInfo,
  PlayerTitleInfo,
  TsysBrowserEntry,
  CacheStatus,
  EntitySources,
  NpcFavorEntry,
} from "../types/gameData";

export type DataStatus = "loading" | "ready" | "error" | "empty";

export const useGameDataStore = defineStore("gameData", () => {
  // ── State ──────────────────────────────────────────────────────────────────
  const status = ref<DataStatus>("loading");
  const errorMessage = ref<string | null>(null);
  const cacheStatus = ref<CacheStatus | null>(null);

  // ── Preloaded NPC map (loaded once on game-data-ready) ────────────────────
  const npcsByKey = ref<Record<string, NpcInfo>>({})
  const npcsByDisplayName = ref<Record<string, NpcInfo>>({})

  async function loadAllNpcsMap(): Promise<void> {
    if (Object.keys(npcsByKey.value).length > 0) return
    const all = await invoke<NpcInfo[]>('get_all_npcs')
    const byKey: Record<string, NpcInfo> = {}
    const byName: Record<string, NpcInfo> = {}
    for (const npc of all) {
      byKey[npc.key] = npc
      byName[npc.name] = npc
    }
    npcsByKey.value = byKey
    npcsByDisplayName.value = byName
  }


  // Icon path cache: icon_id → local filesystem path
  const iconPaths = ref<Record<number, string>>({});

  // ── Startup ────────────────────────────────────────────────────────────────

  // Listen for the Rust-emitted events fired when init_game_data() finishes
  listen<number>("game-data-ready", async (_event) => {
    status.value = "ready";
    errorMessage.value = null;
    await Promise.all([refreshCacheStatus(), loadAllNpcsMap()]);
  });

  listen<string>("game-data-error", (event) => {
    status.value = "error";
    errorMessage.value = event.payload;
  });

  // Kick off an initial status check immediately (covers the case where
  // game-data-ready fires before we're listening)
  refreshCacheStatus().then(() => {
    if (status.value === 'ready') loadAllNpcsMap()
  });

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

  // ── Unified entity resolvers ─────────────────────────────────────────────
  // Each accepts any known reference form (numeric ID, display name, internal
  // name, CDN key) and resolves to the canonical entity.

  async function resolveItem(reference: string | number): Promise<ItemInfo | null> {
    return invoke<ItemInfo | null>("resolve_item", { reference: String(reference) });
  }

  async function resolveItemsBatch(references: string[]): Promise<Record<string, ItemInfo>> {
    return invoke<Record<string, ItemInfo>>("resolve_items_batch", { references });
  }

  async function resolveSkill(reference: string | number): Promise<SkillInfo | null> {
    return invoke<SkillInfo | null>("resolve_skill", { reference: String(reference) });
  }

  async function resolveRecipe(reference: string | number): Promise<RecipeInfo | null> {
    return invoke<RecipeInfo | null>("resolve_recipe", { reference: String(reference) });
  }

  async function resolveQuest(reference: string): Promise<QuestInfo | null> {
    return invoke<QuestInfo | null>("resolve_quest", { reference });
  }

  /** Synchronous NPC resolver using preloaded maps. Tries CDN key, then display name. */
  function resolveNpcSync(reference: string): NpcInfo | null {
    return npcsByKey.value[reference] ?? npcsByDisplayName.value[reference] ?? null
  }

  /** Async NPC resolver via backend (for cases where NPC map isn't loaded yet). */
  async function resolveNpc(reference: string): Promise<NpcInfo | null> {
    return invoke<NpcInfo | null>("resolve_npc", { reference });
  }

  // ── Query/filter commands (not replaced by resolvers) ───────────────────

  async function searchItems(
    query: string,
    filters?: { equipSlot?: string; levelMin?: number; levelMax?: number },
  ): Promise<ItemInfo[]> {
    return invoke<ItemInfo[]>("search_items", {
      query,
      equipSlot: filters?.equipSlot ?? null,
      levelMin: filters?.levelMin ?? null,
      levelMax: filters?.levelMax ?? null,
    });
  }

  async function getItemsByKeyword(keyword: string): Promise<ItemInfo[]> {
    return invoke<ItemInfo[]>("get_items_by_keyword", { keyword });
  }

  async function getAllItemKeywords(): Promise<string[]> {
    return invoke<string[]>("get_all_item_keywords");
  }

  async function getEquipSlots(): Promise<string[]> {
    return invoke<string[]>("get_equip_slots");
  }

  // ── Skill queries ──────────────────────────────────────────────────────────

  async function getAllSkills(): Promise<SkillInfo[]> {
    return invoke<SkillInfo[]>("get_all_skills");
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

  async function searchRecipes(query: string, limit = 50): Promise<RecipeInfo[]> {
    return invoke<RecipeInfo[]>("search_recipes", { query, limit });
  }

  async function getRecipesForSkill(skill: string): Promise<RecipeInfo[]> {
    return invoke<RecipeInfo[]>("get_recipes_for_skill", { skill });
  }

  // ── Quest queries ──────────────────────────────────────────────────────────

  async function getAllQuests(): Promise<QuestInfo[]> {
    return invoke<QuestInfo[]>("get_all_quests");
  }

  async function searchQuests(query: string): Promise<QuestInfo[]> {
    return invoke<QuestInfo[]>("search_quests", { query });
  }

  // ── NPC queries ────────────────────────────────────────────────────────────

  async function getAllNpcs(): Promise<NpcInfo[]> {
    return invoke<NpcInfo[]>("get_all_npcs");
  }

  async function searchNpcs(query: string): Promise<NpcInfo[]> {
    return invoke<NpcInfo[]>("search_npcs", { query });
  }

  async function getNpcsInArea(area: string): Promise<NpcInfo[]> {
    return invoke<NpcInfo[]>("get_npcs_in_area", { area });
  }

  // ── Effect queries ─────────────────────────────────────────────────────────

  async function searchEffects(query: string, limit = 50): Promise<EffectInfo[]> {
    return invoke<EffectInfo[]>("search_effects", { query, limit });
  }

  async function getEffect(id: number): Promise<EffectInfo | null> {
    return invoke<EffectInfo | null>("get_effect", { id });
  }

  // ── TSys browser queries ────────────────────────────────────────────────

  async function getAllTsys(): Promise<TsysBrowserEntry[]> {
    return invoke<TsysBrowserEntry[]>("get_all_tsys");
  }

  async function searchTsys(query: string, limit = 100): Promise<TsysBrowserEntry[]> {
    return invoke<TsysBrowserEntry[]>("search_tsys", { query, limit });
  }

  // ── Player Title queries ──────────────────────────────────────────────────

  async function getAllPlayerTitles(): Promise<PlayerTitleInfo[]> {
    return invoke<PlayerTitleInfo[]>("get_all_player_titles");
  }

  async function searchPlayerTitles(query: string): Promise<PlayerTitleInfo[]> {
    return invoke<PlayerTitleInfo[]>("search_player_titles", { query });
  }

  // ── Source queries ─────────────────────────────────────────────────────────

  async function getAbilitySources(id: number): Promise<EntitySources> {
    return invoke<EntitySources>("get_ability_sources", { id });
  }

  async function getItemSources(id: number): Promise<EntitySources> {
    return invoke<EntitySources>("get_item_sources", { id });
  }

  async function getRecipeSources(id: number): Promise<EntitySources> {
    return invoke<EntitySources>("get_recipe_sources", { id });
  }

  async function getQuestSources(key: string): Promise<EntitySources> {
    return invoke<EntitySources>("get_quest_sources", { key });
  }

  // ── Cross-reference queries ────────────────────────────────────────────────

  async function getNpcsWantingItem(itemId: number): Promise<NpcFavorEntry[]> {
    return invoke<NpcFavorEntry[]>("get_npcs_wanting_item", { itemId });
  }

  async function getNpcsTrainingSkill(skill: string): Promise<NpcInfo[]> {
    return invoke<NpcInfo[]>("get_npcs_training_skill", { skill });
  }

  async function getQuestsForNpc(npcKey: string): Promise<QuestInfo[]> {
    return invoke<QuestInfo[]>("get_quests_for_npc", { npcKey });
  }

  async function getQuestsForSkill(skill: string): Promise<QuestInfo[]> {
    return invoke<QuestInfo[]>("get_quests_for_skill", { skill });
  }

  async function getRecipesForKeyword(keyword: string): Promise<RecipeInfo[]> {
    return invoke<RecipeInfo[]>("get_recipes_for_keyword", { keyword });
  }

  // ── Storage vault queries ──────────────────────────────────────────────────

  interface StorageVaultZoneInfo {
    vault_key: string
    area_key: string | null
    area_name: string | null
    npc_friendly_name: string | null
    num_slots: number | null
  }

  const storageVaultZones = ref<StorageVaultZoneInfo[] | null>(null)

  async function getStorageVaultZones(): Promise<StorageVaultZoneInfo[]> {
    if (storageVaultZones.value) return storageVaultZones.value
    const zones = await invoke<StorageVaultZoneInfo[]>('get_storage_vault_zones')
    storageVaultZones.value = zones
    return zones
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
    // Unified entity resolvers
    resolveItem,
    resolveItemsBatch,
    resolveSkill,
    resolveRecipe,
    resolveQuest,
    resolveNpcSync,
    resolveNpc,
    // Query/filter actions
    refreshCacheStatus,
    forceRefreshCdn,
    searchItems,
    getItemsByKeyword,
    getAllItemKeywords,
    getEquipSlots,
    getAllSkills,
    getAbilitiesForSkill,
    getRecipesForItem,
    getRecipesUsingItem,
    searchRecipes,
    getRecipesForSkill,
    getAllQuests,
    searchQuests,
    getAllNpcs,
    searchNpcs,
    getNpcsInArea,
    npcsByKey,
    npcsByDisplayName,
    loadAllNpcsMap,
    searchEffects,
    getEffect,
    getAllTsys,
    searchTsys,
    getAllPlayerTitles,
    searchPlayerTitles,
    getAbilitySources,
    getItemSources,
    getRecipeSources,
    getQuestSources,
    getStorageVaultZones,
    storageVaultZones,
    getIconPath,
    // Cross-reference queries
    getNpcsWantingItem,
    getNpcsTrainingSkill,
    getQuestsForNpc,
    getQuestsForSkill,
    getRecipesForKeyword,
  };
});
