<template>
  <div class="h-full flex flex-col">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="p-4 text-sm">
      <span v-if="store.status === 'loading'" class="text-accent-gold"
        >⟳ Loading game data…</span
      >
      <span v-else-if="store.status === 'error'" class="text-accent-red"
        >✕ {{ store.errorMessage }}</span
      >
    </div>

    <div v-else class="flex gap-4 h-full overflow-hidden">
      <!-- Left panel: filters + results -->
      <div class="w-90 shrink-0 flex flex-col gap-2 overflow-hidden">
        <!-- Skill filter dropdown -->
        <div class="flex gap-2">
          <select
            v-model="selectedSkillFilter"
            class="input flex-1 cursor-pointer">
            <option value="All">All Skills</option>
            <option
              v-for="skill in skillsWithAbilities"
              :key="skill.id"
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
          <span v-else-if="filteredAbilities.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredAbilities.length
          }}</span>
        </div>

        <div v-if="selectedSkillFilter === 'All' && !query" class="text-text-dim text-xs italic py-1">
          Select a skill or start typing to search abilities
        </div>

        <div v-else-if="filteredAbilities.length === 0 && !loading && query" class="text-text-dim text-xs italic py-1">
          No abilities found for "{{ query }}"
        </div>

        <div v-else-if="allAbilities.length === 0 && !loading" class="text-text-dim text-xs italic py-1">
          No abilities for {{ selectedSkillFilter }}
        </div>

        <ul ref="listRef" v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(ability, idx) in filteredAbilities"
            :key="ability.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === ability.id, 'bg-surface-elevated': selectedIndex === idx && selected?.id !== ability.id }"
            @click="selectAbility(ability)">
            <span class="text-text-muted text-[0.72rem] min-w-14 shrink-0">[Lv {{ ability.level || 0 }}]</span>
            <span class="text-text-primary/75 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ ability.name }}</span>
          </li>
        </ul>
      </div>

      <!-- Right panel: ability detail -->
      <div
        class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
        :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select an ability to inspect
        </div>

        <template v-else>
          <div class="flex gap-3 items-start">
            <!-- Icon -->
            <div class="shrink-0">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="w-12 h-12 [image-rendering:pixelated] border border-border-default"
                alt="ability icon" />
              <div v-else-if="iconLoading" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-accent-gold animate-spin">
                ⟳
              </div>
              <div v-else-if="selected.icon_id" class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-text-dim">
                {{ selected.icon_id }}
              </div>
              <div v-else class="w-12 h-12 bg-surface-base border border-surface-elevated flex items-center justify-center text-[0.65rem] text-border-default">—</div>
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-accent-gold text-base font-bold mb-1">{{ selected.name }}</div>
              <div class="text-xs text-text-dim mb-1">
                ID: <span class="text-text-secondary font-mono">{{ selected.id }}</span>
                <template v-if="selected.skill">
                  · Skill:
                  <SkillInline :reference="selected.skill" /></template
                >
                <template v-if="selected.level !== null">
                  · Level:
                  <span class="text-text-secondary font-mono">{{ selected.level }}</span></template
                >
                <template v-if="selected.icon_id">
                  · Icon:
                  <span class="text-text-secondary font-mono">{{ selected.icon_id }}</span></template
                >
              </div>
              <div v-if="selected.description" class="text-xs text-text-secondary italic">
                {{ selected.description }}
              </div>
            </div>

            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Combat Details -->
          <div v-if="selected.damage_type || selected.reset_time || selected.target || selected.range" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Combat Details</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.damage_type" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Damage:</span>
                <span class="text-text-secondary">{{ selected.damage_type }}</span>
              </div>
              <div v-if="selected.reset_time" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Cooldown:</span>
                <span class="text-text-secondary">{{ selected.reset_time }}s</span>
              </div>
              <div v-if="selected.target" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Target:</span>
                <span class="text-text-secondary">{{ selected.target }}</span>
              </div>
              <div v-if="selected.range" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Range:</span>
                <span class="text-text-secondary">{{ selected.range }}m</span>
              </div>
              <div v-if="selected.mana_cost" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Mana Cost:</span>
                <span class="text-text-secondary">{{ selected.mana_cost }}</span>
              </div>
              <div v-if="selected.power_cost" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Power Cost:</span>
                <span class="text-text-secondary">{{ selected.power_cost }}</span>
              </div>
              <div v-if="selected.animation" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Animation:</span>
                <span class="text-text-secondary font-mono">{{ selected.animation }}</span>
              </div>
            </div>
          </div>

          <!-- Special Info -->
          <div v-if="selected.special_info || selected.prerequisite" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Info</div>
            <div v-if="selected.prerequisite" class="text-xs text-[#e08060] px-2 py-1 bg-[#151515] border-l-2 border-l-[#4a2a2a]">
              Requires: {{ selected.prerequisite }}
            </div>
            <div v-if="selected.special_info" class="text-xs text-text-secondary italic px-2 py-1">
              {{ selected.special_info }}
            </div>
          </div>

          <!-- Flags from raw_json -->
          <div v-if="abilityFlags.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Flags</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="flag in abilityFlags"
                :key="flag"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a2a1a] border border-[#2a4a2a] text-[#8ab88a]">
                {{ flag }}
              </span>
            </div>
          </div>

          <!-- Upgrade chain from raw_json -->
          <div v-if="selected.raw_json?.UpgradeOf || selected.raw_json?.SharesResetTimerWith" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Related</div>
            <div v-if="selected.raw_json?.UpgradeOf" class="text-xs flex gap-2 px-2 py-1 bg-[#151515]">
              <span class="text-text-muted">Upgrade of:</span>
              <span class="text-text-secondary">{{ selected.raw_json.UpgradeOf }}</span>
            </div>
            <div v-if="selected.raw_json?.SharesResetTimerWith" class="text-xs flex gap-2 px-2 py-1 bg-[#151515]">
              <span class="text-text-muted">Shares cooldown with:</span>
              <span class="text-text-secondary">{{ selected.raw_json.SharesResetTimerWith }}</span>
            </div>
          </div>

          <!-- Sources -->
          <SourcesPanel :sources="sources" :loading="sourcesLoading" />

          <!-- PvE/PvP Details -->
          <div v-if="selected.pve || selected.pvp" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">PvE / PvP</div>
            <div class="flex gap-4">
              <div v-if="selected.pve" class="flex-1">
                <div class="text-[0.65rem] text-text-muted mb-1">PvE</div>
                <pre class="bg-surface-dark border border-surface-card p-2 text-[0.68rem] text-text-muted overflow-x-auto whitespace-pre m-0">{{ JSON.stringify(selected.pve, null, 2) }}</pre>
              </div>
              <div v-if="selected.pvp" class="flex-1">
                <div class="text-[0.65rem] text-text-muted mb-1">PvP</div>
                <pre class="bg-surface-dark border border-surface-card p-2 text-[0.68rem] text-text-muted overflow-x-auto whitespace-pre m-0">{{ JSON.stringify(selected.pvp, null, 2) }}</pre>
              </div>
            </div>
          </div>

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-entity-item"
                :class="{ 'bg-[#1e1a10]! border-[#3a3010]! text-[#887040]!': kw.startsWith('Lint_') }"
                >{{ kw }}</span
              >
            </div>
          </div>

          <!-- Raw JSON -->
          <div v-if="settingsStore.settings.showRawJsonInDataBrowser" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Raw JSON</div>
            <pre class="bg-surface-dark border border-surface-card p-3 text-[0.72rem] text-text-muted overflow-x-auto whitespace-pre m-0 leading-relaxed">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { SkillInfo, AbilityInfo, EntitySources } from "../../types/gameData";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import SourcesPanel from "../Shared/SourcesPanel.vue";

const store = useGameDataStore();
const settingsStore = useSettingsStore();

const allSkills = ref<SkillInfo[]>([]);
const skillAbilityCounts = ref<Record<string, number>>({});
const selectedSkillFilter = ref<string>("All");
const query = ref("");
const allAbilities = ref<AbilityInfo[]>([]);
const selected = ref<AbilityInfo | null>(null);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);
const sources = ref<EntitySources | null>(null);
const sourcesLoading = ref(false);
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const loading = ref(false);

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
    allSkills.value = await store.getAllSkills();
    allSkills.value.sort((a, b) => a.name.localeCompare(b.name));

    // Count abilities per skill to filter out empty skills
    skillAbilityCounts.value = {};
    for (const skill of allSkills.value) {
      const abilities = await store.getAbilitiesForSkill(skill.name);
      skillAbilityCounts.value[skill.name] = abilities.length;
    }
  } finally {
    loading.value = false;
  }
}

watch(selectedSkillFilter, async (skillName) => {
  if (skillName === "All") {
    allAbilities.value = [];
    return;
  }
  loading.value = true;
  try {
    const abilities = await store.getAbilitiesForSkill(skillName);
    allAbilities.value = abilities.sort((a, b) => (a.level || 0) - (b.level || 0));
  } catch (e) {
    console.warn("Failed to load abilities:", e);
    allAbilities.value = [];
  } finally {
    loading.value = false;
  }
});

const skillsWithAbilities = computed(() => {
  return allSkills.value.filter(skill => (skillAbilityCounts.value[skill.name] || 0) > 0);
});

let searchTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (searchTimer) clearTimeout(searchTimer);
  if (!val.trim()) {
    // If a skill is selected, show that skill's abilities; otherwise clear
    if (selectedSkillFilter.value !== "All") return;
    allAbilities.value = [];
    return;
  }
  searchTimer = setTimeout(() => searchAbilities(val.trim()), 250);
});

async function searchAbilities(q: string) {
  if (selectedSkillFilter.value !== "All") return; // client-side filter handles it
  loading.value = true;
  try {
    // Search across all skills
    const results = new Map<number, AbilityInfo>();
    const lower = q.toLowerCase();
    for (const skill of allSkills.value) {
      const abilities = await store.getAbilitiesForSkill(skill.name);
      for (const ability of abilities) {
        if (ability.name.toLowerCase().includes(lower) || ability.description?.toLowerCase().includes(lower)) {
          results.set(ability.id, ability);
        }
      }
      if (results.size >= 50) break;
    }
    allAbilities.value = Array.from(results.values()).sort((a, b) => a.name.localeCompare(b.name));
  } finally {
    loading.value = false;
  }
}

const filteredAbilities = computed(() => {
  if (selectedSkillFilter.value === "All") {
    return allAbilities.value; // already filtered by search
  }
  if (!query.value.trim()) {
    return allAbilities.value;
  }
  const q = query.value.toLowerCase();
  return allAbilities.value.filter(ability =>
    ability.name.toLowerCase().includes(q) ||
    ability.description?.toLowerCase().includes(q)
  );
});

async function selectAbility(ability: AbilityInfo) {
  selected.value = ability;
  iconSrc.value = null;
  sources.value = null;

  // Load sources
  sourcesLoading.value = true;
  store.getAbilitySources(ability.id)
    .then(s => { sources.value = s; })
    .catch(e => { console.warn("Sources fetch failed:", e); })
    .finally(() => { sourcesLoading.value = false; });

  // Load icon if present
  if (ability.icon_id) {
    iconLoading.value = true;
    try {
      const path = await store.getIconPath(ability.icon_id);
      iconSrc.value = convertFileSrc(path);
    } catch (e) {
      console.warn("Icon fetch failed:", e);
    } finally {
      iconLoading.value = false;
    }
  }
}

const abilityFlags = computed(() => {
  if (!selected.value) return [];
  const raw = selected.value.raw_json;
  if (!raw) return [];
  const flags: string[] = [];
  if (selected.value.is_harmless) flags.push("Harmless");
  if (selected.value.works_underwater) flags.push("Works Underwater");
  if (selected.value.works_while_falling) flags.push("Works While Falling");
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
  sources.value = null;
}

useKeyboard({
  listNavigation: {
    items: filteredAbilities,
    selectedIndex,
    onConfirm: (index: number) => {
      const ability = filteredAbilities.value[index];
      if (ability) selectAbility(ability);
    },
    scrollContainerRef: listRef,
  },
});
</script>
