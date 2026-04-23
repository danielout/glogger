<template>
  <PaneLayout screen-key="db-abilities" :left-pane="{ title: 'Abilities', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
    <template #left>
      <!-- Status banner if data not ready -->
      <div v-if="store.status !== 'ready'" class="p-4 text-sm">
        <span v-if="store.status === 'loading'" class="text-accent-gold"
          >⟳ Loading game data…</span
        >
        <span v-else-if="store.status === 'error'" class="text-accent-red"
          >✕ {{ store.errorMessage }}</span
        >
      </div>

      <template v-else>
      <div class="flex flex-col gap-2 h-full overflow-hidden">
        <!-- Skill filter dropdown -->
        <div class="flex gap-2">
          <select
            v-model="selectedSkillFilter"
            class="input flex-1 cursor-pointer">
            <option value="All">All Skills</option>
            <option
              v-for="skill in skillsWithAbilities"
              :key="skill.name"
              :value="skill.name">
              {{ skill.name }}
            </option>
          </select>
        </div>

        <!-- Search bar -->
        <div class="flex items-center gap-2 relative">
          <input
            v-model="query"
            class="input flex-1"
            placeholder="Search abilities…" />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredFamilies.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredFamilies.length
          }}</span>
        </div>

        <!-- Monster abilities toggle -->
        <label class="flex items-center gap-1.5 text-xs text-text-muted cursor-pointer select-none">
          <input type="checkbox" v-model="showMonsterAbilities" class="accent-accent-gold" />
          Show monster abilities
        </label>

        <div v-if="selectedSkillFilter === 'All' && !query" class="text-text-dim text-xs italic py-1">
          Select a skill or start typing to search abilities
        </div>

        <div v-else-if="filteredFamilies.length === 0 && !loading && query" class="text-text-dim text-xs italic py-1">
          No abilities found for "{{ query }}"
        </div>

        <div v-else-if="allFamilies.length === 0 && !loading" class="text-text-dim text-xs italic py-1">
          No abilities for {{ selectedSkillFilter }}
        </div>

        <ul ref="listRef" v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(family, idx) in filteredFamilies"
            :key="family.base_internal_name"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.base_internal_name === family.base_internal_name, 'bg-surface-elevated': selectedIndex === idx && selected?.base_internal_name !== family.base_internal_name }"
            @click="selectFamily(family)">
            <span class="text-text-muted text-xs min-w-14 shrink-0">[{{ levelRange(family) }}]</span>
            <span class="text-text-primary/75 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ family.base_name }}</span>
            <span v-if="family.tier_ids.length > 1" class="text-text-dim text-[10px] shrink-0">{{ family.tier_ids.length }}T</span>
          </li>
        </ul>
      </div>
      </template>
    </template>

    <!-- Right panel: family detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
      :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select an ability to inspect
        </div>

        <template v-else>
          <!-- Header: icon + shared info -->
          <div class="flex gap-3 items-start">
            <div class="shrink-0">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="w-12 h-12 [image-rendering:pixelated] border border-border-default"
                alt="ability icon" />
              <div v-else-if="iconLoading" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[10px] text-accent-gold animate-spin">
                ⟳
              </div>
              <div v-else-if="selected.icon_id" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[10px] text-text-dim">
                {{ selected.icon_id }}
              </div>
              <div v-else class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[10px] text-border-default">—</div>
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-base font-bold mb-1">{{ selected.base_name }}</div>
              <div class="text-xs text-text-dim mb-1">
                <template v-if="selected.skill">
                  Skill:
                  <SkillInline :reference="selected.skill" /></template>
                <template v-if="selected.damage_type">
                  · Damage:
                  <span class="text-text-secondary">{{ selected.damage_type }}</span></template>
                <template v-if="selected.tier_ids.length > 1">
                  · <span class="text-text-secondary">{{ selected.tier_ids.length }} tiers</span></template>
              </div>
              <!-- Description from base tier -->
              <div v-if="baseTierAbility?.description" class="text-xs text-text-secondary italic">
                {{ baseTierAbility.description }}
              </div>
            </div>

            <button
              class="bg-transparent border-none cursor-pointer px-1 py-0 text-sm shrink-0 transition-colors"
              :class="isFav ? 'text-accent-gold' : 'text-text-dim hover:text-accent-gold'"
              :title="isFav ? 'Remove from favorites' : 'Add to favorites'"
              @click="dataBrowserStore.toggleFavorite({ type: 'ability', reference: selected.base_internal_name, label: selected.base_name })"
            >&#x2605;</button>
            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Loading tiers -->
          <div v-if="tiersLoading" class="text-accent-gold text-xs animate-spin">⟳ Loading tiers…</div>

          <template v-else-if="resolvedTiers.length">
            <!-- Combat Details (shared across tiers) -->
            <div v-if="baseTierAbility && (baseTierAbility.target || sharedCooldown)" class="flex flex-col gap-1.5">
              <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Combat Details</div>
              <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
                <div v-if="baseTierAbility.target" class="text-xs flex gap-2">
                  <span class="text-text-muted min-w-20">Target:</span>
                  <span class="text-text-secondary">{{ baseTierAbility.target }}</span>
                </div>
                <div v-if="sharedCooldown" class="text-xs flex gap-2">
                  <span class="text-text-muted min-w-20">Cooldown:</span>
                  <span class="text-text-secondary">{{ sharedCooldown }}s</span>
                </div>
                <div v-if="baseTierAbility.animation" class="text-xs flex gap-2">
                  <span class="text-text-muted min-w-20">Animation:</span>
                  <span class="text-text-secondary">{{ baseTierAbility.animation }}</span>
                </div>
              </div>
            </div>

            <!-- Tier Progression Table -->
            <AbilityTierTable :tiers="resolvedTiers" v-model:expanded-tier-id="expandedTierId" />

            <!-- Flags (from base tier) -->
            <div v-if="abilityFlags.length" class="flex flex-col gap-1.5">
              <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Flags</div>
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="flag in abilityFlags"
                  :key="flag"
                  class="text-xs px-1.5 py-0.5 bg-[#1a2a1a] border border-[#2a4a2a] text-[#8ab88a]">
                  {{ flag }}
                </span>
              </div>
            </div>

            <!-- Related Treasure Mods (for the base ability) -->
            <div v-if="relatedTsysLoading || relatedTsys.length" class="flex flex-col gap-1.5">
              <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
                Treasure Mods
                <span v-if="relatedTsys.length" class="text-text-muted">({{ relatedTsys.length }})</span>
              </div>
              <div v-if="relatedTsysLoading" class="text-accent-gold text-xs animate-spin">⟳</div>
              <div v-else class="flex flex-col gap-1.5">
                <div
                  v-for="tsys in relatedTsys"
                  :key="tsys.key"
                  class="bg-surface-dark border border-surface-card p-2 text-xs">
                  <div class="flex items-baseline gap-2 mb-0.5">
                    <span class="text-entity-item font-medium">{{ tsys.internal_name || tsys.key }}</span>
                    <span v-if="tsys.skill" class="text-text-muted text-[10px]">{{ tsys.skill }}</span>
                    <span class="text-text-dim text-[10px]">{{ tsys.tier_count }} tiers</span>
                  </div>
                  <div v-if="tsys.slots.length" class="flex gap-1 mb-0.5">
                    <span
                      v-for="slot in tsys.slots"
                      :key="slot"
                      class="text-[10px] px-1 py-0 bg-[#1a1a2e] border border-[#2a2a4e] text-text-muted">
                      {{ slot }}
                    </span>
                  </div>
                  <div v-if="tsys.top_tier_effects.length" class="flex flex-col gap-0.5 text-text-secondary text-xs pl-1">
                    <span v-for="(eff, i) in tsys.top_tier_effects" :key="i">{{ eff }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- PvE/PvP Details (from highest tier) -->
            <div v-if="highestTier && (highestTier.pve || highestTier.pvp)" class="flex flex-col gap-1.5">
              <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
                PvE / PvP
                <span class="text-text-muted">(Tier {{ resolvedTiers.length }})</span>
              </div>
              <div class="flex gap-4">
                <div v-if="highestTier.pve" class="flex-1">
                  <div class="text-[10px] text-text-muted mb-1">PvE</div>
                  <CombatStatsPanel :stats="highestTier.pve" />
                </div>
                <div v-if="highestTier.pvp" class="flex-1">
                  <div class="text-[10px] text-text-muted mb-1">PvP</div>
                  <CombatStatsPanel :stats="highestTier.pvp" />
                </div>
              </div>
            </div>

            <!-- Keywords (from base tier) -->
            <div v-if="baseTierAbility && baseTierAbility.keywords.length" class="flex flex-col gap-1.5">
              <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="kw in baseTierAbility.keywords"
                  :key="kw"
                  class="text-xs px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item"
                  :class="{ 'bg-[#1e1a10]! border-[#3a3010]! text-[#887040]!': kw.startsWith('Lint_') }"
                  >{{ kw }}</span
                >
              </div>
            </div>

            <!-- Raw JSON -->
            <div v-if="settingsStore.settings.showRawJsonInDataBrowser && jsonTier" class="flex flex-col gap-1.5">
              <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
                Raw JSON
                <span class="text-text-muted">({{ jsonTier.name }})</span>
              </div>
              <pre class="bg-surface-dark border border-surface-card p-3 text-xs text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(jsonTier, null, 2) }}</pre>
            </div>
          </template>
        </template>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import { ref, onMounted, watch, computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import { useDataBrowserStore } from "../../stores/dataBrowserStore";
import type { AbilityInfo, AbilityFamily, TsysAbilityXref } from "../../types/gameData";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import CombatStatsPanel from "./CombatStatsPanel.vue";
import AbilityTierTable from "./AbilityTierTable.vue";

const store = useGameDataStore();
const settingsStore = useSettingsStore();
const dataBrowserStore = useDataBrowserStore();

const isFav = computed(() =>
  selected.value ? dataBrowserStore.isFavorite("ability", selected.value.base_internal_name) : false
);

const skillsWithCounts = ref<{ name: string; count: number }[]>([]);
const selectedSkillFilter = ref<string>("All");
const query = ref("");
const allFamilies = ref<AbilityFamily[]>([]);
const selected = ref<AbilityFamily | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
const resolvedTiers = ref<AbilityInfo[]>([]);
const tiersLoading = ref(false);
const relatedTsys = ref<TsysAbilityXref[]>([]);
const relatedTsysLoading = ref(false);
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const loading = ref(false);
const showMonsterAbilities = ref(false);

const expandedTierId = ref<number | null>(null);
const baseTierAbility = computed(() => resolvedTiers.value[0] ?? null);
const highestTier = computed(() => resolvedTiers.value[resolvedTiers.value.length - 1] ?? null);
const jsonTier = computed(() => {
  if (expandedTierId.value != null) {
    return resolvedTiers.value.find(t => t.id === expandedTierId.value) ?? baseTierAbility.value;
  }
  return baseTierAbility.value;
});

const sharedCooldown = computed(() => {
  const base = baseTierAbility.value;
  if (!base?.reset_time) return null;
  // If all tiers share the same cooldown, show it once
  const allSame = resolvedTiers.value.every(t => t.reset_time === base.reset_time);
  return allSame ? base.reset_time : null;
});

onMounted(async () => {
  if (store.status === "ready") {
    await loadSkillList();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadSkillList();
  }
});

async function loadSkillList() {
  loading.value = true;
  try {
    const counts = await store.getSkillsWithAbilityCounts(showMonsterAbilities.value);
    skillsWithCounts.value = counts
      .filter(([, count]) => count > 0)
      .map(([name, count]) => ({ name, count }));
  } finally {
    loading.value = false;
  }
}

async function loadFamiliesForSkill(skillName: string) {
  loading.value = true;
  try {
    allFamilies.value = await store.getAbilityFamiliesForSkill(skillName, showMonsterAbilities.value);
  } catch (e) {
    console.warn("Failed to load ability families:", e);
    allFamilies.value = [];
  } finally {
    loading.value = false;
  }
}

watch(selectedSkillFilter, async (skillName) => {
  if (skillName === "All") {
    // If there's a query active, search across all skills
    if (query.value.trim()) {
      searchFamilies(query.value.trim());
    } else {
      allFamilies.value = [];
    }
    return;
  }
  // If there's a query, do server-side filtered search; otherwise load all for this skill
  if (query.value.trim()) {
    searchFamilies(query.value.trim());
  } else {
    loadFamiliesForSkill(skillName);
  }
});

const skillsWithAbilities = computed(() => skillsWithCounts.value);

// When the monster toggle changes, reload the current view
watch(showMonsterAbilities, async () => {
  await loadSkillList();
  if (query.value.trim()) {
    searchFamilies(query.value.trim());
  } else if (selectedSkillFilter.value !== "All") {
    loadFamiliesForSkill(selectedSkillFilter.value);
  } else {
    allFamilies.value = [];
  }
});

let searchTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (searchTimer) clearTimeout(searchTimer);
  if (!val.trim()) {
    // When clearing search, reload skill families if a skill is selected, otherwise clear
    if (selectedSkillFilter.value !== "All") {
      loadFamiliesForSkill(selectedSkillFilter.value);
    } else {
      allFamilies.value = [];
    }
    return;
  }
  searchTimer = setTimeout(() => searchFamilies(val.trim()), 250);
});

async function searchFamilies(q: string) {
  loading.value = true;
  try {
    const skill = selectedSkillFilter.value !== "All" ? selectedSkillFilter.value : undefined;
    allFamilies.value = await store.searchAbilityFamilies(q, skill, 50, showMonsterAbilities.value);
  } finally {
    loading.value = false;
  }
}

const filteredFamilies = computed(() => allFamilies.value);

function levelRange(family: AbilityFamily): string {
  if (family.tier_ids.length === 1) {
    // Single tier - we'll show the level from resolvedTiers if we have it, otherwise just show "1T"
    return "1T";
  }
  return `${family.tier_ids.length}T`;
}

async function selectFamily(family: AbilityFamily) {
  selected.value = family;
  iconSrc.value = null;
  expandedTierId.value = null;
  dataBrowserStore.addToHistory({ type: "ability", reference: family.base_internal_name, label: family.base_name });
  resolvedTiers.value = [];
  relatedTsys.value = [];

  // Resolve all tier abilities in parallel
  tiersLoading.value = true;
  try {
    const tierPromises = family.tier_ids.map(id => store.resolveAbility(id));
    const tiers = await Promise.all(tierPromises);
    resolvedTiers.value = tiers.filter((t): t is AbilityInfo => t !== null);
  } finally {
    tiersLoading.value = false;
  }

  // Load icon from base tier
  if (family.icon_id) {
    iconLoading.value = true;
    try {
      const path = await store.getIconPath(family.icon_id);
      iconSrc.value = convertFileSrc(path);
    } catch (e) {
      console.warn("Icon fetch failed:", e);
    } finally {
      iconLoading.value = false;
    }
  }

  // Load related treasure mods (from base tier)
  if (family.tier_ids.length > 0) {
    relatedTsysLoading.value = true;
    store.getTsysForAbility(family.tier_ids[0])
      .then(t => { relatedTsys.value = t; })
      .catch(e => { console.warn("TSys xref fetch failed:", e); })
      .finally(() => { relatedTsysLoading.value = false; });
  }
}

const abilityFlags = computed(() => {
  const base = baseTierAbility.value;
  if (!base) return [];
  const raw = base.raw_json;
  if (!raw) return [];
  const flags: string[] = [];
  if (base.is_harmless) flags.push("Harmless");
  if (base.works_underwater) flags.push("Works Underwater");
  if (base.works_while_falling) flags.push("Works While Falling");
  if (raw.WorksInCombat === false) flags.push("Out of Combat Only");
  if (raw.WorksWhileStunned) flags.push("Works While Stunned");
  if (raw.WorksWhileMounted) flags.push("Works While Mounted");
  if (raw.CanSuppressMonsterShout) flags.push("Suppresses Shout");
  if (raw.AoEIsCenteredOnCaster) flags.push("AoE on Self");
  if (raw.InternalAbility) flags.push("Internal");
  if (raw.CanBeOnSidebar === false) flags.push("No Sidebar");
  return flags;
});

function clearSelection() {
  selected.value = null;
  iconSrc.value = null;
  resolvedTiers.value = [];
  relatedTsys.value = [];
}

useKeyboard({
  listNavigation: {
    items: filteredFamilies,
    selectedIndex,
    onConfirm: (index: number) => {
      const family = filteredFamilies.value[index];
      if (family) selectFamily(family);
    },
    scrollContainerRef: listRef,
  },
});
</script>
