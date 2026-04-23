<template>
  <div class="flex flex-col gap-4 h-[calc(100vh-200px)] overflow-y-auto">
    <div class="flex items-center justify-between">
      <h3 class="text-text-primary text-sm font-semibold m-0">Crafting History</h3>
      <button
        class="text-text-muted text-xs cursor-pointer bg-transparent border border-border-light rounded px-2 py-0.5 hover:text-text-primary hover:border-border-default"
        :disabled="loading"
        @click="refresh">
        {{ loading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>

    <div v-if="loading" class="text-text-dim text-xs italic">Loading crafting data...</div>

    <EmptyState v-else-if="!hasData" variant="panel" primary="No crafting data available" secondary="Import a character report to see crafting history and stats." />

    <template v-else>
      <!-- Skill progress overview -->
      <SkillCraftingProgress :stats="skillStats" />

      <!-- Top crafted recipes -->
      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between">
          <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">
            Top Crafted Recipes
          </h4>
          <!-- Skill filter -->
          <select v-model="filterSkill" class="input text-xs w-44">
            <option value="">All skills</option>
            <option v-for="skill in availableSkills" :key="skill" :value="skill">
              {{ skill }}
            </option>
          </select>
        </div>

        <table class="w-full text-xs">
          <thead>
            <tr class="text-text-dim border-b border-border-light">
              <th class="text-left py-1 font-medium w-8">#</th>
              <th class="text-left py-1 font-medium">Recipe</th>
              <th class="text-left py-1 font-medium w-28">Skill</th>
              <th class="text-right py-1 font-medium w-16">Level</th>
              <th class="text-right py-1 font-medium w-20">Crafts</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(recipe, idx) in filteredRecipes"
              :key="recipe.recipe_key"
              class="border-b border-surface-dark">
              <td class="py-1 text-text-muted">{{ idx + 1 }}</td>
              <td class="py-1">
                <RecipeInline :reference="recipe.recipe_name" />
              </td>
              <td class="py-1">
                <SkillInline v-if="recipe.reward_skill" :reference="recipe.reward_skill" />
                <span v-else class="text-text-muted">—</span>
              </td>
              <td class="text-right py-1 text-text-muted">
                {{ recipe.skill_level_req ?? '—' }}
              </td>
              <td class="text-right py-1 text-text-primary font-semibold">
                {{ recipe.completions.toLocaleString() }}
              </td>
            </tr>
          </tbody>
        </table>

        <div v-if="filteredRecipes.length === 0" class="text-text-dim text-xs italic">
          No recipes found{{ filterSkill ? ` for ${filterSkill}` : '' }}.
        </div>

        <div v-if="hasMoreRecipes" class="flex justify-center">
          <button
            class="text-text-muted text-xs cursor-pointer bg-transparent border-none hover:text-accent-gold underline"
            @click="showAll = !showAll">
            {{ showAll ? 'Show top 50' : `Show all ${totalFilteredCount} recipes` }}
          </button>
        </div>
      </div>

      <!-- Summary stats -->
      <div class="flex gap-6 text-xs text-text-dim border-t border-border-light pt-3">
        <div>
          Total unique recipes crafted:
          <span class="text-text-primary font-semibold">{{ totalCrafted }}</span>
        </div>
        <div>
          Total completions:
          <span class="text-text-primary font-semibold">{{ totalCompletions.toLocaleString() }}</span>
        </div>
        <div>
          Uncrafted recipes (first-time bonus opportunities):
          <span class="text-accent-gold font-semibold">{{ totalUncrafted }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingHistoryRecipe, SkillCraftingStats } from "../../types/crafting";
import EmptyState from "../Shared/EmptyState.vue";
import SkillCraftingProgress from "./SkillCraftingProgress.vue";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";
import SkillInline from "../Shared/Skill/SkillInline.vue";

const craftingStore = useCraftingStore();

const loading = ref(false);
const history = ref<CraftingHistoryRecipe[]>([]);
const skillStats = ref<SkillCraftingStats[]>([]);
const filterSkill = ref("");
const showAll = ref(false);

const hasData = computed(() => history.value.length > 0 || skillStats.value.length > 0);

const availableSkills = computed(() => {
  const skills = new Set<string>();
  for (const r of history.value) {
    if (r.reward_skill) skills.add(r.reward_skill);
  }
  return Array.from(skills).sort();
});

const allFilteredRecipes = computed(() => {
  if (!filterSkill.value) return history.value;
  return history.value.filter((r) => r.reward_skill === filterSkill.value);
});

const totalFilteredCount = computed(() => allFilteredRecipes.value.length);

const filteredRecipes = computed(() => {
  if (showAll.value) return allFilteredRecipes.value;
  return allFilteredRecipes.value.slice(0, 50);
});

const hasMoreRecipes = computed(() => allFilteredRecipes.value.length > 50);

const totalCrafted = computed(() => history.value.filter((r) => r.completions > 0).length);

const totalCompletions = computed(() =>
  history.value.reduce((sum, r) => sum + r.completions, 0),
);

const totalUncrafted = computed(() =>
  skillStats.value.reduce((sum, s) => sum + s.uncrafted_count, 0),
);

onMounted(() => refresh());

async function refresh() {
  loading.value = true;
  try {
    const [h, s] = await Promise.all([
      craftingStore.getCraftingHistory(),
      craftingStore.getSkillCraftingStats(),
    ]);
    history.value = h;
    skillStats.value = s;
  } catch (e) {
    console.error("[crafting] Failed to load history:", e);
  } finally {
    loading.value = false;
  }
}
</script>
