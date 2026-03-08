<script setup lang="ts">
import { ref, onMounted, watch, computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../stores/gameDataStore";
import type { SkillInfo, RecipeInfo, ItemInfo } from "../types/gameData";

const store = useGameDataStore();

const allSkills = ref<SkillInfo[]>([]);
const skillRecipeCounts = ref<Record<string, number>>({});
const selectedSkillFilter = ref<string>("All");
const query = ref("");
const allRecipes = ref<RecipeInfo[]>([]);
const selected = ref<RecipeInfo | null>(null);
const ingredientItems = ref<Record<number, ItemInfo>>({});
const resultItems = ref<Record<number, ItemInfo>>({});
const iconSrc = ref<string | null>(null);
const iconLoading = ref(false);
const loading = ref(false);

onMounted(async () => {
  if (store.status === "ready") {
    await loadData();
  }
});

watch(() => store.status, async (newStatus) => {
  if (newStatus === "ready") {
    await loadData();
  }
});

async function loadData() {
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

    // Load all recipes for "All" view
    await loadAllRecipes();
  } finally {
    loading.value = false;
  }
}

async function loadAllRecipes() {
  const allRecipesMap = new Map<number, RecipeInfo>();
  for (const skill of allSkills.value) {
    const recipes = await store.getRecipesForSkill(skill.name);
    recipes.forEach(recipe => allRecipesMap.set(recipe.id, recipe));
  }
  allRecipes.value = Array.from(allRecipesMap.values()).sort((a, b) =>
    a.name.localeCompare(b.name)
  );
}

watch(selectedSkillFilter, async (skillName) => {
  if (skillName === "All") {
    loading.value = true;
    try {
      await loadAllRecipes();
    } catch (e) {
      console.warn("Failed to load all recipes:", e);
      allRecipes.value = [];
    } finally {
      loading.value = false;
    }
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

const filteredRecipes = computed(() => {
  if (!query.value.trim()) {
    return allRecipes.value;
  }
  const q = query.value.toLowerCase();
  return allRecipes.value.filter(recipe =>
    recipe.name.toLowerCase().includes(q) ||
    recipe.description?.toLowerCase().includes(q)
  );
});

async function selectRecipe(recipe: RecipeInfo) {
  selected.value = recipe;
  iconSrc.value = null;
  ingredientItems.value = {};
  resultItems.value = {};

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
      ingredientItems.value = await store.getItemsBatch(recipe.ingredient_item_ids);
    } catch (e) {
      console.warn("Failed to load ingredient items:", e);
    }
  }

  // Load result items
  if (recipe.result_item_ids.length > 0) {
    try {
      resultItems.value = await store.getItemsBatch(recipe.result_item_ids);
    } catch (e) {
      console.warn("Failed to load result items:", e);
    }
  }
}

function clearSelection() {
  selected.value = null;
  iconSrc.value = null;
  ingredientItems.value = {};
  resultItems.value = {};
}
</script>

<template>
  <div class="recipe-browser">
    <!-- Status banner if data not ready -->
    <div v-if="store.status !== 'ready'" class="status-banner">
      <span v-if="store.status === 'loading'" class="status-loading"
        >⟳ Loading game data…</span
      >
      <span v-else-if="store.status === 'error'" class="status-error"
        >✕ {{ store.errorMessage }}</span
      >
    </div>

    <div v-else class="browser-layout">
      <!-- Left panel: filters + results -->
      <div class="search-panel">
        <!-- Skill filter dropdown -->
        <div class="filter-bar">
          <select
            v-model="selectedSkillFilter"
            class="skill-filter">
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
        <div class="search-bar">
          <input
            v-model="query"
            class="search-input"
            placeholder="Filter recipes…" />
          <span v-if="loading" class="search-spinner">⟳</span>
          <span v-else-if="filteredRecipes.length" class="search-count">{{
            filteredRecipes.length
          }}</span>
        </div>

        <div v-else-if="filteredRecipes.length === 0 && !loading && query" class="search-hint">
          No recipes found for "{{ query }}"
        </div>

        <div v-else-if="allRecipes.length === 0 && !loading" class="search-hint">
          No recipes for {{ selectedSkillFilter }}
        </div>

        <ul v-else class="results-list">
          <li
            v-for="recipe in filteredRecipes"
            :key="recipe.id"
            class="result-row"
            :class="{ active: selected?.id === recipe.id }"
            @click="selectRecipe(recipe)">
            <span class="result-level">[Lv {{ recipe.skill_level_req || 0 }}]</span>
            <span class="result-name">{{ recipe.name }}</span>
          </li>
        </ul>
      </div>

      <!-- Right panel: recipe detail -->
      <div class="detail-panel" :class="{ empty: !selected }">
        <div v-if="!selected" class="detail-empty">
          Select a recipe to inspect
        </div>

        <template v-else>
          <div class="detail-header">
            <!-- Icon -->
            <div class="icon-wrap">
              <img
                v-if="iconSrc"
                :src="iconSrc"
                class="item-icon"
                alt="recipe icon" />
              <div v-else-if="iconLoading" class="icon-placeholder loading">
                ⟳
              </div>
              <div v-else-if="selected.icon_id" class="icon-placeholder">
                {{ selected.icon_id }}
              </div>
              <div v-else class="icon-placeholder muted">—</div>
            </div>

            <div class="detail-title-block">
              <div class="detail-name">{{ selected.name }}</div>
              <div class="detail-meta">
                ID: <span class="mono">{{ selected.id }}</span>
                <template v-if="selected.skill">
                  · Skill:
                  <span class="mono">{{ selected.skill }}</span></template
                >
                <template v-if="selected.skill_level_req !== null">
                  · Level:
                  <span class="mono">{{ selected.skill_level_req }}</span></template
                >
                <template v-if="selected.internal_name">
                  · Internal:
                  <span class="mono">{{ selected.internal_name }}</span></template
                >
              </div>
              <div v-if="selected.description" class="detail-desc">
                {{ selected.description }}
              </div>
            </div>

            <button class="close-btn" @click="clearSelection">✕</button>
          </div>

          <!-- Ingredients -->
          <div v-if="selected.ingredients.length" class="detail-section">
            <div class="section-label">Ingredients</div>
            <div class="item-grid">
              <div
                v-for="(ingredient, idx) in selected.ingredients"
                :key="idx"
                class="item-row">
                <span class="item-stack">{{ ingredient.stack_size }}x</span>
                <span class="item-name">
                  {{ ingredientItems[ingredient.item_id]?.name || `Item #${ingredient.item_id}` }}
                </span>
                <span v-if="ingredient.chance_to_consume !== null && ingredient.chance_to_consume < 100" class="item-chance">
                  ({{ ingredient.chance_to_consume }}% consume)
                </span>
              </div>
            </div>
          </div>

          <!-- Results -->
          <div v-if="selected.result_items.length" class="detail-section">
            <div class="section-label">Results</div>
            <div class="item-grid">
              <div
                v-for="(result, idx) in selected.result_items"
                :key="idx"
                class="item-row">
                <span class="item-stack">{{ result.stack_size }}x</span>
                <span class="item-name result">
                  {{ resultItems[result.item_id]?.name || `Item #${result.item_id}` }}
                </span>
                <span v-if="result.percent_chance !== null && result.percent_chance < 100" class="item-chance">
                  ({{ result.percent_chance }}% chance)
                </span>
              </div>
            </div>
          </div>

          <!-- XP Rewards -->
          <div v-if="selected.reward_skill" class="detail-section">
            <div class="section-label">XP Rewards</div>
            <div class="xp-info">
              <span class="xp-skill">{{ selected.reward_skill }}:</span>
              <span class="xp-amount">{{ selected.reward_skill_xp || 0 }} XP</span>
              <span v-if="selected.reward_skill_xp_first_time && selected.reward_skill_xp_first_time !== selected.reward_skill_xp" class="xp-first">
                ({{ selected.reward_skill_xp_first_time }} XP first time)
              </span>
            </div>
          </div>

          <!-- Prerequisites -->
          <div v-if="selected.prereq_recipe" class="detail-section">
            <div class="section-label">Prerequisites</div>
            <div class="prereq-info">
              Requires: <span class="prereq-name">{{ selected.prereq_recipe }}</span>
            </div>
          </div>

          <!-- Keywords -->
          <div v-if="selected.keywords.length" class="detail-section">
            <div class="section-label">Keywords</div>
            <div class="keyword-list">
              <span
                v-for="kw in selected.keywords"
                :key="kw"
                class="keyword"
                :class="{ lint: kw.startsWith('Lint_') }"
                >{{ kw }}</span
              >
            </div>
          </div>

          <!-- Raw JSON -->
          <div class="detail-section">
            <div class="section-label">Raw JSON</div>
            <pre class="json-dump">{{ JSON.stringify(selected, null, 2) }}</pre>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.recipe-browser {
  height: calc(100vh - 130px);
  display: flex;
  flex-direction: column;
}

.status-banner {
  padding: 1rem;
  font-size: 0.9rem;
}
.status-loading {
  color: #e0c060;
}
.status-error {
  color: #f66;
}

.browser-layout {
  display: flex;
  gap: 1rem;
  height: 100%;
  overflow: hidden;
}

/* ── Left panel ── */
.search-panel {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow: hidden;
}

.filter-bar {
  display: flex;
  gap: 0.5rem;
}
.skill-filter {
  flex: 1;
  padding: 0.45rem 0.6rem;
  background: #1a1a1a;
  border: 1px solid #444;
  color: #ccc;
  font-family: monospace;
  font-size: 0.9rem;
  outline: none;
  cursor: pointer;
}
.skill-filter:focus {
  border-color: #e0c060;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
}
.search-input {
  flex: 1;
  padding: 0.45rem 0.6rem;
  background: #1a1a1a;
  border: 1px solid #444;
  color: #ccc;
  font-family: monospace;
  font-size: 0.9rem;
  outline: none;
}
.search-input:focus {
  border-color: #e0c060;
}
.search-spinner {
  color: #e0c060;
  font-size: 0.9rem;
  animation: spin 1s linear infinite;
}
.search-count {
  color: #555;
  font-size: 0.8rem;
  min-width: 1.5rem;
  text-align: right;
}

.search-hint {
  color: #444;
  font-size: 0.8rem;
  font-style: italic;
  padding: 0.25rem 0;
}

.results-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
  border: 1px solid #2a2a2a;
}
.result-row {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.3rem 0.5rem;
  cursor: pointer;
  border-bottom: 1px solid #1a1a1a;
  font-size: 0.82rem;
}
.result-row:hover {
  background: #1e1e1e;
}
.result-row.active {
  background: #1a1a2e;
  border-left: 2px solid #e0c060;
}

.result-level {
  color: #666;
  font-size: 0.72rem;
  min-width: 3.5rem;
  flex-shrink: 0;
}
.result-name {
  color: #bbb;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Right panel ── */
.detail-panel {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #2a2a2a;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.detail-panel.empty {
  align-items: center;
  justify-content: center;
}
.detail-empty {
  color: #333;
  font-style: italic;
}

.detail-header {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
}

.icon-wrap {
  flex-shrink: 0;
}
.item-icon {
  width: 48px;
  height: 48px;
  image-rendering: pixelated;
  border: 1px solid #333;
}
.icon-placeholder {
  width: 48px;
  height: 48px;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.65rem;
  color: #444;
}
.icon-placeholder.loading {
  color: #e0c060;
  animation: spin 1s linear infinite;
}
.icon-placeholder.muted {
  color: #333;
}

.detail-title-block {
  flex: 1;
  min-width: 0;
}
.detail-name {
  color: #e0c060;
  font-size: 1rem;
  font-weight: bold;
  margin-bottom: 0.25rem;
}
.detail-meta {
  font-size: 0.75rem;
  color: #555;
  margin-bottom: 0.3rem;
}
.mono {
  color: #888;
  font-family: monospace;
}
.detail-desc {
  font-size: 0.82rem;
  color: #888;
  font-style: italic;
}

.close-btn {
  background: none;
  border: none;
  color: #444;
  cursor: pointer;
  padding: 0 0.25rem;
  font-size: 0.9rem;
  flex-shrink: 0;
}
.close-btn:hover {
  color: #f66;
  background: none;
  border: none;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.section-label {
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: #555;
  border-bottom: 1px solid #222;
  padding-bottom: 0.2rem;
}

.item-grid {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}
.item-row {
  display: flex;
  gap: 0.5rem;
  align-items: baseline;
  font-size: 0.8rem;
  padding: 0.2rem 0.4rem;
  background: #151515;
  border-left: 2px solid #2a2a2a;
}
.item-stack {
  color: #666;
  font-size: 0.72rem;
  min-width: 2.5rem;
  flex-shrink: 0;
}
.item-name {
  color: #aaa;
  flex: 1;
}
.item-name.result {
  color: #9a9;
}
.item-chance {
  color: #777;
  font-size: 0.72rem;
  font-style: italic;
}

.xp-info {
  display: flex;
  gap: 0.5rem;
  font-size: 0.8rem;
  padding: 0.3rem 0.5rem;
  background: #151515;
  border-left: 2px solid #2a4a2a;
}
.xp-skill {
  color: #9a9;
  font-weight: bold;
}
.xp-amount {
  color: #7a7;
}
.xp-first {
  color: #666;
  font-style: italic;
  font-size: 0.72rem;
}

.prereq-info {
  font-size: 0.8rem;
  padding: 0.3rem 0.5rem;
  background: #151515;
  border-left: 2px solid #4a2a2a;
  color: #aaa;
}
.prereq-name {
  color: #c88;
  font-family: monospace;
}

.keyword-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
}
.keyword {
  font-size: 0.72rem;
  padding: 0.15rem 0.4rem;
  background: #1a1a2e;
  border: 1px solid #2a2a4e;
  color: #7ec8e3;
}
.keyword.lint {
  background: #1e1a10;
  border-color: #3a3010;
  color: #887040;
}

.json-dump {
  background: #0d0d0d;
  border: 1px solid #222;
  padding: 0.75rem;
  font-size: 0.72rem;
  color: #666;
  overflow-x: auto;
  white-space: pre;
  margin: 0;
  line-height: 1.5;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
