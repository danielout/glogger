import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "./settingsStore";

export interface CharacterDeath {
  id: number;
  character_name: string;
  server_name: string;
  died_at: string;
  killer_name: string;
  killer_entity_id: string | null;
  killing_ability: string;
  health_damage: number;
  armor_damage: number;
  area: string | null;
  damage_type: string | null;
}

export interface DeathDamageSource {
  id: number;
  death_id: number;
  event_order: number;
  timestamp: string;
  attacker_name: string;
  attacker_entity_id: string | null;
  ability_name: string;
  health_damage: number;
  armor_damage: number;
  is_crit: boolean;
}

export const useDeathStore = defineStore("deaths", () => {
  const deaths = ref<CharacterDeath[]>([]);
  const loaded = ref(false);
  /** Cache of damage sources per death ID, loaded on demand. */
  const damageSourcesCache = ref<Record<number, DeathDamageSource[]>>({});

  async function loadDeaths() {
    const settings = useSettingsStore();
    const characterName = settings.settings.activeCharacterName;
    const serverName = settings.settings.activeServerName;
    if (!characterName || !serverName) return;

    try {
      deaths.value = await invoke("get_character_deaths", {
        characterName,
        serverName,
      });
      loaded.value = true;
    } catch (e) {
      console.error("[deaths] Failed to load deaths:", e);
    }
  }

  function handleDeathEvent(payload: {
    kind: string;
    timestamp: string;
    killer_name: string;
    killer_entity_id: string;
    killing_ability: string;
    health_damage: number;
    armor_damage: number;
  }) {
    const settings = useSettingsStore();
    // Prepend the new death (most recent first)
    deaths.value.unshift({
      id: Date.now(), // Temporary ID until next DB load
      character_name: settings.settings.activeCharacterName ?? "",
      server_name: settings.settings.activeServerName ?? "",
      died_at: payload.timestamp,
      killer_name: payload.killer_name,
      killer_entity_id: payload.killer_entity_id,
      killing_ability: payload.killing_ability,
      health_damage: payload.health_damage,
      armor_damage: payload.armor_damage,
      area: null, // Area comes from DB record; live events don't include it yet
      damage_type: null, // Damage type comes from DB record
    });
  }

  async function loadDamageSources(deathId: number): Promise<DeathDamageSource[]> {
    if (damageSourcesCache.value[deathId]) {
      return damageSourcesCache.value[deathId];
    }
    try {
      const sources = await invoke<DeathDamageSource[]>("get_death_damage_sources", { deathId });
      damageSourcesCache.value[deathId] = sources;
      return sources;
    } catch (e) {
      console.error("[deaths] Failed to load damage sources:", e);
      return [];
    }
  }

  // ── Computed summaries ──────────────────────────────────────────────────

  const totalDeaths = computed(() => deaths.value.length);

  const deathsByKiller = computed(() => {
    const counts = new Map<string, number>();
    for (const d of deaths.value) {
      counts.set(d.killer_name, (counts.get(d.killer_name) ?? 0) + 1);
    }
    return [...counts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count);
  });

  const deathsByAbility = computed(() => {
    const counts = new Map<string, number>();
    for (const d of deaths.value) {
      counts.set(d.killing_ability, (counts.get(d.killing_ability) ?? 0) + 1);
    }
    return [...counts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count);
  });

  const deathsByArea = computed(() => {
    const counts = new Map<string, number>();
    for (const d of deaths.value) {
      const area = d.area ?? "Unknown";
      counts.set(area, (counts.get(area) ?? 0) + 1);
    }
    return [...counts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count);
  });

  const deathsByDamageType = computed(() => {
    const counts = new Map<string, number>();
    for (const d of deaths.value) {
      const dtype = d.damage_type ?? "Unknown";
      counts.set(dtype, (counts.get(dtype) ?? 0) + 1);
    }
    return [...counts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count);
  });

  return {
    deaths,
    loaded,
    damageSourcesCache,
    loadDeaths,
    loadDamageSources,
    handleDeathEvent,
    totalDeaths,
    deathsByKiller,
    deathsByAbility,
    deathsByArea,
    deathsByDamageType,
  };
});
