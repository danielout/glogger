<template>
  <PaneLayout screen-key="db-recipes" :left-pane="{ title: 'Recipes', defaultWidth: 360, minWidth: 280, maxWidth: 500 }">
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
              v-for="skill in skillsWithRecipes"
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
            placeholder="Search recipes…" />
          <span v-if="loading" class="text-accent-gold text-sm animate-spin">⟳</span>
          <span v-else-if="filteredRecipes.length" class="text-text-dim text-xs min-w-6 text-right">{{
            filteredRecipes.length
          }}</span>
        </div>

        <div v-if="selectedSkillFilter === 'All' && !query" class="text-text-dim text-xs italic py-1">
          Select a skill or start typing to search recipes
        </div>

        <div v-else-if="filteredRecipes.length === 0 && !loading && query" class="text-text-dim text-xs italic py-1">
          No recipes found for "{{ query }}"
        </div>

        <div v-else-if="allRecipes.length === 0 && !loading" class="text-text-dim text-xs italic py-1">
          No recipes for {{ selectedSkillFilter }}
        </div>

        <ul ref="listRef" v-else class="list-none m-0 p-0 overflow-y-auto flex-1 border border-surface-elevated">
          <li
            v-for="(recipe, idx) in filteredRecipes"
            :key="recipe.id"
            class="flex items-baseline gap-2 px-2 py-1 cursor-pointer border-b border-surface-dark text-xs hover:bg-[#1e1e1e]"
            :class="{ 'bg-[#1a1a2e] border-l-2 border-l-accent-gold': selected?.id === recipe.id, 'bg-surface-elevated': selectedIndex === idx && selected?.id !== recipe.id }"
            @click="selectRecipe(recipe)">
            <span class="text-text-muted text-[0.72rem] min-w-14 shrink-0">[Lv {{ recipe.skill_level_req || 0 }}]</span>
            <span class="text-text-primary/75 flex-1 overflow-hidden text-ellipsis whitespace-nowrap">{{ recipe.name }}</span>
          </li>
        </ul>
      </div>
      </template>
    </template>

    <!-- Right panel: recipe detail -->
    <div
      class="h-full overflow-y-auto border-l border-surface-elevated p-4 flex flex-col gap-4"
      :class="{ 'items-center justify-center': !selected }">
        <div v-if="!selected" class="text-border-default italic">
          Select a recipe to inspect
        </div>

        <template v-else>
          <div class="flex gap-3 items-start">
            <!-- Icon -->
            <div class="shrink-0">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="w-12 h-12 [image-rendering:pixelated] border border-border-default"
                alt="recipe icon" />
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
                <template v-if="selected.skill_level_req !== null">
                  · Level:
                  <span class="text-text-secondary font-mono">{{ selected.skill_level_req }}</span></template
                >
                <template v-if="selected.internal_name">
                  · Internal:
                  <span class="text-text-secondary font-mono">{{ selected.internal_name }}</span></template
                >
              </div>
              <div v-if="selected.description" class="text-xs text-text-secondary italic">
                {{ selected.description }}
              </div>
            </div>

            <button class="bg-transparent border-none text-text-dim cursor-pointer px-1 py-0 text-sm shrink-0 hover:text-accent-red" @click="clearSelection">✕</button>
          </div>

          <!-- Ingredients -->
          <div v-if="selected.ingredients.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Ingredients</div>
            <div class="flex flex-col gap-1">
              <div
                v-for="(ingredient, idx) in selected.ingredients"
                :key="idx"
                class="flex gap-2 items-center text-sm px-1.5 py-0.5 bg-[#151515] border-l-2"
                :class="ingredient.item_id !== null ? 'border-l-surface-elevated' : 'border-l-[#4a3a1a]'">
                <span class="text-text-muted text-[0.72rem] min-w-10 shrink-0">{{ ingredient.stack_size }}x</span>
                <span class="flex-1">
                  <!-- Specific item ingredient -->
                  <template v-if="ingredient.item_id !== null">
                    <ItemInline
                      v-if="ingredientItems[ingredient.item_id]?.name"
                      :reference="ingredientItems[ingredient.item_id].name" />
                    <span v-else class="text-text-secondary text-xs">Item #{{ ingredient.item_id }}</span>
                  </template>
                  <!-- Wildcard/keyword ingredient -->
                  <template v-else>
                    <span class="text-[#c8a84a]">{{ ingredient.description || 'Any matching item' }}</span>
                    <span v-if="ingredient.item_keys.length" class="text-text-muted text-[0.65rem] ml-1">
                      [{{ ingredient.item_keys.join(', ') }}]
                    </span>
                  </template>
                </span>
                <span v-if="ingredient.chance_to_consume !== null && ingredient.chance_to_consume < 100" class="text-text-muted text-[0.72rem] italic">
                  ({{ ingredient.chance_to_consume }}% consume)
                </span>
              </div>
            </div>
          </div>

          <!-- Results -->
          <div v-if="selected.result_items.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Results</div>
            <div class="flex flex-col gap-1">
              <div
                v-for="(result, idx) in selected.result_items"
                :key="idx"
                class="flex gap-2 items-center text-sm px-1.5 py-0.5 bg-[#151515] border-l-2 border-l-surface-elevated">
                <span class="text-text-muted text-[0.72rem] min-w-10 shrink-0">{{ result.stack_size }}x</span>
                <span class="flex-1">
                  <ItemInline
                    v-if="resultItems[result.item_id]?.name"
                    :reference="resultItems[result.item_id].name" />
                  <span v-else class="text-text-secondary text-xs">Item #{{ result.item_id }}</span>
                </span>
                <span v-if="result.percent_chance !== null && result.percent_chance < 100" class="text-text-muted text-[0.72rem] italic">
                  ({{ result.percent_chance }}% chance)
                </span>
              </div>
            </div>
          </div>

          <!-- XP Rewards -->
          <div v-if="selected.reward_skill" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">XP Rewards</div>
            <div class="flex flex-col gap-1">
              <div class="flex gap-2 text-sm px-2 py-1 bg-[#151515] border-l-2 border-l-[#2a4a2a]">
                <span class="text-[#9a9] font-bold">{{ selected.reward_skill }}:</span>
                <span class="text-[#7a7]">{{ selected.reward_skill_xp || 0 }} XP</span>
                <span v-if="selected.reward_skill_xp_first_time && selected.reward_skill_xp_first_time !== selected.reward_skill_xp" class="text-text-muted text-[0.72rem] italic">
                  ({{ selected.reward_skill_xp_first_time }} XP first time)
                </span>
              </div>
              <div v-if="selected.reward_skill_xp_drop_off_level" class="text-xs text-text-muted px-2 py-0.5">
                XP drops off after level {{ selected.reward_skill_xp_drop_off_level }}
              </div>
            </div>
          </div>

          <!-- Result Effects -->
          <div v-if="selected.result_effects?.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Result Effects</div>
            <ul class="m-0 pl-4 p-0">
              <li
                v-for="(eff, i) in selected.result_effects"
                :key="i"
                class="text-xs text-[#9a9] py-0.5">
                {{ eff }}
              </li>
            </ul>
          </div>

          <!-- Usage Info -->
          <div v-if="selected.usage_delay || selected.action_label" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Usage</div>
            <div class="grid grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-1.5">
              <div v-if="selected.action_label" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Action:</span>
                <span class="text-text-secondary">{{ selected.action_label }}</span>
              </div>
              <div v-if="selected.usage_delay" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Delay:</span>
                <span class="text-text-secondary">{{ selected.usage_delay }}s</span>
              </div>
              <div v-if="selected.sort_skill" class="text-xs flex gap-2">
                <span class="text-text-muted min-w-20">Sort Skill:</span>
                <span class="text-text-secondary">{{ selected.sort_skill }}</span>
              </div>
            </div>
          </div>

          <!-- Prerequisites -->
          <div v-if="selected.prereq_recipe" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Prerequisites</div>
            <div class="text-sm px-2 py-1 bg-[#151515] border-l-2 border-l-[#4a2a2a] text-text-secondary flex items-center gap-1">
              Requires: <RecipeInline :reference="selected.prereq_recipe" />
            </div>
          </div>

          <!-- Sources -->
          <SourcesPanel :sources="sources" :loading="sourcesLoading" />

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="flex flex-col gap-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-[#7ec8e3]"
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
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import { ref, onMounted, watch, computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useKeyboard } from "../../composables/useKeyboard";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import type { SkillInfo, RecipeInfo, ItemInfo, EntitySources } from "../../types/gameData";
import ItemInline from "../Shared/Item/ItemInline.vue";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";
import SourcesPanel from "../Shared/SourcesPanel.vue";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const store = useGameDataStore();
const settingsStore = useSettingsStore();

const allSkills = ref<SkillInfo[]>([]);
const skillRecipeCounts = ref<Record<string, number>>({});
const selectedSkillFilter = ref<string>("All");
const query = ref("");
const allRecipes = ref<RecipeInfo[]>([]);
const selected = ref<RecipeInfo | null>(null);
const sources = ref<EntitySources | null>(null);
const sourcesLoading = ref(false);
const ingredientItems = ref<Record<string, ItemInfo>>({});
const resultItems = ref<Record<string, ItemInfo>>({});
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const loading = ref(false);
const selectedIndex = ref(0);
const listRef = ref<HTMLElement | null>(null);

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

    // Count recipes per skill to filter out empty skills
    skillRecipeCounts.value = {};
    for (const skill of allSkills.value) {
      const recipes = await store.getRecipesForSkill(skill.name);
      skillRecipeCounts.value[skill.name] = recipes.length;
    }
  } finally {
    loading.value = false;
  }
}

watch(selectedSkillFilter, async (skillName) => {
  if (skillName === "All") {
    allRecipes.value = [];
    return;
  }
  loading.value = true;
  try {
    const recipes = await store.getRecipesForSkill(skillName);
    allRecipes.value = recipes.sort((a, b) =>
      (a.skill_level_req || 0) - (b.skill_level_req || 0)
    );
  } catch (e) {
    console.warn("Failed to load recipes:", e);
    allRecipes.value = [];
  } finally {
    loading.value = false;
  }
});

const skillsWithRecipes = computed(() => {
  return allSkills.value.filter(skill => (skillRecipeCounts.value[skill.name] || 0) > 0);
});

let searchTimer: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (searchTimer) clearTimeout(searchTimer);
  if (!val.trim()) {
    if (selectedSkillFilter.value !== "All") return;
    allRecipes.value = [];
    return;
  }
  if (selectedSkillFilter.value !== "All") return; // client-side filter handles it
  searchTimer = setTimeout(() => doSearch(val.trim()), 250);
});

async function doSearch(q: string) {
  loading.value = true;
  try {
    allRecipes.value = await store.searchRecipes(q, 50);
  } finally {
    loading.value = false;
  }
}

const filteredRecipes = computed(() => {
  if (selectedSkillFilter.value === "All") {
    return allRecipes.value; // already filtered by search
  }
  if (!query.value.trim()) {
    return allRecipes.value;
  }
  const q = query.value.toLowerCase();
  return allRecipes.value.filter(recipe =>
    recipe.name.toLowerCase().includes(q) ||
    recipe.description?.toLowerCase().includes(q)
  );
});

watch(filteredRecipes, () => {
  selectedIndex.value = 0;
});

useKeyboard({
  listNavigation: {
    items: filteredRecipes,
    selectedIndex,
    onConfirm: (idx: number) => {
      const recipe = filteredRecipes.value[idx];
      if (recipe) selectRecipe(recipe);
    },
    scrollContainerRef: listRef,
  },
});

async function selectRecipe(recipe: RecipeInfo) {
  selected.value = recipe;
  iconSrc.value = null;
  sources.value = null;
  ingredientItems.value = {};
  resultItems.value = {};

  // Load sources
  sourcesLoading.value = true;
  store.getRecipeSources(recipe.id)
    .then(s => { sources.value = s; })
    .catch(e => { console.warn("Sources fetch failed:", e); })
    .finally(() => { sourcesLoading.value = false; });

  // Load icon if present
  if (recipe.icon_id) {
    iconLoading.value = true;
    try {
      const path = await store.getIconPath(recipe.icon_id);
      iconSrc.value = convertFileSrc(path);
    } catch (e) {
      console.warn("Icon fetch failed:", e);
    } finally {
      iconLoading.value = false;
    }
  }

  // Load ingredient items
  if (recipe.ingredient_item_ids.length > 0) {
    try {
      ingredientItems.value = await store.resolveItemsBatch(recipe.ingredient_item_ids.map(String));
    } catch (e) {
      console.warn("Failed to load ingredient items:", e);
    }
  }

  // Load result items
  if (recipe.result_item_ids.length > 0) {
    try {
      resultItems.value = await store.resolveItemsBatch(recipe.result_item_ids.map(String));
    } catch (e) {
      console.warn("Failed to load result items:", e);
    }
  }
}

function clearSelection() {
  selected.value = null;
  iconSrc.value = null;
  sources.value = null;
  ingredientItems.value = {};
  resultItems.value = {};
}

// Navigate to a specific recipe when navTarget changes
watch(() => props.navTarget, async (target) => {
  if (!target || target.type !== 'recipe') return;
  const name = String(target.id);
  if (selected.value?.name === name) return;

  const recipe = await store.resolveRecipe(name);
  if (recipe) {
    query.value = recipe.name;
    selectRecipe(recipe);
  }
}, { immediate: true });
</script>
