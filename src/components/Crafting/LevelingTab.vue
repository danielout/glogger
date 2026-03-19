<template>
  <div class="flex gap-4 h-[calc(100vh-200px)]">
    <!-- Left panel: inputs -->
    <div class="w-80 shrink-0 flex flex-col gap-3">
      <h3 class="text-text-primary text-sm font-semibold m-0">XP Leveling Optimizer</h3>

      <!-- Skill selector -->
      <div class="flex flex-col gap-1">
        <label class="text-text-dim text-xs">Skill</label>
        <select v-model="selectedSkill" class="input text-xs" @change="onSkillChange">
          <option value="">Select a skill...</option>
          <option v-for="skill in craftingSkills" :key="skill.name" :value="skill.name">
            {{ skill.name }}
          </option>
        </select>
      </div>

      <!-- Level inputs -->
      <div v-if="selectedSkill" class="flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <label class="text-text-dim text-xs w-24">Current level:</label>
          <input
            v-model.number="currentLevel"
            type="number"
            min="0"
            class="input w-20 text-center text-xs" />
          <button
            v-if="snapshotLevel !== null"
            class="text-text-muted text-[0.65rem] cursor-pointer bg-transparent border-none hover:text-accent-gold underline"
            @click="currentLevel = snapshotLevel">
            (use {{ snapshotLevel }})
          </button>
        </div>
        <div class="flex items-center gap-2">
          <label class="text-text-dim text-xs w-24">Target level:</label>
          <input
            v-model.number="targetLevel"
            type="number"
            min="1"
            class="input w-20 text-center text-xs" />
        </div>
      </div>

      <!-- Strategy -->
      <div v-if="selectedSkill" class="flex flex-col gap-1">
        <label class="text-text-dim text-xs">Strategy</label>
        <select v-model="strategy" class="input text-xs">
          <option value="combined">Combined (recommended)</option>
          <option value="first-time-rush">First-Time Bonus Rush</option>
          <option value="cost-efficient">Cost-Efficient Grinding</option>
        </select>
        <p class="text-text-muted text-[0.65rem] m-0">
          <template v-if="strategy === 'combined'">
            Collect first-time bonuses, then grind the most cost-efficient recipe.
          </template>
          <template v-else-if="strategy === 'first-time-rush'">
            Craft every uncrafted recipe once for bonus XP, then grind highest-XP recipe.
          </template>
          <template v-else>
            Find the recipe with the best XP-per-gold ratio and grind it.
          </template>
        </p>
      </div>

      <!-- Options -->
      <div v-if="selectedSkill" class="flex flex-col gap-1.5">
        <label class="flex items-center gap-2 text-text-dim text-xs cursor-pointer">
          <input
            v-model="includeUnlearnedRecipes"
            type="checkbox"
            class="accent-accent-gold" />
          Include unlearned recipes
        </label>
        <p class="text-text-muted text-[0.65rem] m-0 ml-5">
          Include recipes from the CDN that don't appear in your character export.
          These are recipes you haven't learned yet in-game.
        </p>
      </div>

      <!-- Calculate button -->
      <button
        v-if="selectedSkill"
        class="btn-primary text-xs py-1.5"
        :disabled="calculating || !selectedSkill || targetLevel <= currentLevel"
        @click="calculate">
        {{ calculating ? 'Calculating...' : 'Generate Plan' }}
      </button>

      <!-- Excluded recipes -->
      <div v-if="excludedRecipeIds.size > 0" class="flex flex-col gap-1">
        <div class="flex items-center justify-between">
          <span class="text-text-dim text-[0.65rem] uppercase tracking-wide">
            Excluded ({{ excludedRecipeIds.size }})
          </span>
          <button
            class="text-text-muted text-[0.6rem] cursor-pointer bg-transparent border-none hover:text-accent-gold underline"
            @click="clearExclusions">
            clear all
          </button>
        </div>
        <div
          v-for="[id, name] in excludedRecipeNames"
          :key="id"
          class="flex items-center gap-1.5 text-[0.65rem] text-text-muted">
          <button
            class="text-accent-gold cursor-pointer bg-transparent border-none hover:text-accent-gold/70 text-[0.65rem]"
            @click="restoreRecipe(id)">
            +
          </button>
          <span class="line-through opacity-60">{{ name }}</span>
        </div>
      </div>

      <div v-if="error" class="text-accent-red text-xs">{{ error }}</div>
    </div>

    <!-- Right panel: results -->
    <div class="flex-1 overflow-y-auto border border-surface-elevated rounded p-4">
      <div v-if="!plan" class="flex items-center justify-center h-full text-border-default italic">
        Select a skill and target level to generate a leveling plan
      </div>

      <div v-else class="flex flex-col gap-4">
        <!-- Summary -->
        <div class="bg-surface-elevated rounded p-3 border border-border-light">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <SkillInline :name="plan.skill_name" :show-icon="true" />
              <span class="text-text-primary text-sm font-semibold">
                Lv {{ plan.current_level }} → Lv {{ plan.target_level }}
              </span>
            </div>
            <button
              v-if="plan.steps.length > 0"
              class="text-accent-gold text-[0.65rem] cursor-pointer bg-transparent border border-accent-gold/30 rounded px-2.5 py-1 hover:bg-accent-gold/10 transition-colors"
              :disabled="creatingProject"
              @click="createProjectFromPlan">
              {{ creatingProject ? 'Creating...' : 'Create Crafting Project' }}
            </button>
          </div>
          <div class="grid grid-cols-2 gap-x-6 gap-y-1 text-xs">
            <div>
              <span class="text-text-dim">Total XP needed:</span>
              <span class="text-text-primary font-semibold ml-1">{{ plan.xp_needed.toLocaleString() }}</span>
            </div>
            <div>
              <span class="text-text-dim">Total crafts:</span>
              <span class="text-text-primary font-semibold ml-1">{{ plan.total_crafts.toLocaleString() }}</span>
            </div>
            <div v-if="plan.xp_from_first_time > 0">
              <span class="text-text-dim">XP from bonuses:</span>
              <span class="text-accent-gold font-semibold ml-1">{{ plan.xp_from_first_time.toLocaleString() }}</span>
            </div>
            <div v-if="plan.total_cost > 0">
              <span class="text-text-dim">Est. total cost:</span>
              <span class="text-accent-red font-semibold ml-1">{{ plan.total_cost.toLocaleString() }}g</span>
            </div>
            <div v-if="unlearnedInPlan > 0">
              <span class="text-text-dim">Unlearned recipes:</span>
              <span class="text-purple-400 font-semibold ml-1">{{ unlearnedInPlan }}</span>
            </div>
          </div>
        </div>

        <!-- No steps -->
        <div v-if="plan.levels.length === 0" class="text-text-dim text-xs italic">
          No recipes found for this skill and level range.
        </div>

        <div v-else class="flex flex-col gap-3">
          <!-- Unlearned recipes callout -->
          <div
            v-if="unlearnedInPlan > 0"
            class="bg-purple-900/20 border border-purple-600/30 rounded px-3 py-2 text-xs text-purple-300/80">
            This plan includes {{ unlearnedInPlan }} unlearned recipe{{ unlearnedInPlan !== 1 ? 's' : '' }}
            (marked with <span class="text-purple-400 font-semibold">UNLEARNED</span>).
            You'll need to find and learn {{ unlearnedInPlan !== 1 ? 'them' : 'it' }} before crafting.
          </div>

          <!-- Level-by-level plan -->
          <div v-for="(lvl, lvlIdx) in plan.levels" :key="lvlIdx" class="flex flex-col gap-1">
            <!-- Level header -->
            <div class="flex items-center gap-2 mt-1">
              <span class="text-text-secondary text-xs font-semibold">
                Lv {{ lvl.from_level }} → {{ lvl.to_level }}
              </span>
              <span class="text-text-muted text-[0.65rem]">
                {{ lvl.xp_needed.toLocaleString() }} XP
              </span>
              <span class="text-text-muted text-[0.65rem]">
                · {{ lvl.total_crafts }} craft{{ lvl.total_crafts !== 1 ? 's' : '' }}
              </span>
              <span v-if="lvl.total_cost > 0" class="text-text-muted text-[0.65rem]">
                · {{ lvl.total_cost.toLocaleString() }}g
              </span>
              <div class="flex-1 border-b border-border-default ml-2" />
            </div>

            <!-- Steps in this level -->
            <div
              v-for="(step, idx) in lvl.steps"
              :key="`${lvlIdx}-${idx}`"
              class="flex items-center gap-2 px-3 py-1.5 bg-surface-base border border-surface-elevated rounded text-xs group"
              :class="{ 'border-l-2 border-l-purple-500/50': !step.is_known }">
              <RecipeInline :name="step.recipe_name" />
              <span
                v-if="!step.is_known"
                class="text-purple-400 text-[0.6rem] font-semibold shrink-0"
                title="You haven't learned this recipe yet">
                UNLEARNED
              </span>
              <span class="text-text-primary font-mono shrink-0">×{{ step.craft_count.toLocaleString() }}</span>
              <div class="ml-auto flex items-center gap-2 shrink-0">
                <span v-if="step.xp_first_time > 0" class="text-accent-gold text-[0.65rem]">
                  +{{ (step.xp_per_craft + step.xp_first_time).toLocaleString() }} XP
                  <span class="text-yellow-500/70">({{ step.xp_first_time.toLocaleString() }} bonus)</span>
                </span>
                <span v-else class="text-accent-gold text-[0.65rem]">
                  {{ step.total_xp.toLocaleString() }} XP
                  <span v-if="step.craft_count > 1" class="text-text-muted">({{ step.xp_per_craft.toLocaleString() }}/ea)</span>
                </span>
                <span v-if="step.estimated_cost > 0" class="text-text-muted text-[0.65rem]">
                  {{ step.estimated_cost.toLocaleString() }}g
                </span>
                <button
                  class="text-text-muted/0 group-hover:text-text-muted text-[0.65rem] cursor-pointer bg-transparent border-none hover:text-accent-red ml-1"
                  title="Exclude this recipe"
                  @click="excludeRecipe(step.recipe_id, step.recipe_name)">
                  ✕
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useCraftingStore } from "../../stores/craftingStore";
import type { LevelingPlan, LevelingStrategy } from "../../types/crafting";
import type { SkillInfo } from "../../types/gameData/skills";
import SkillInline from "../Shared/Skill/SkillInline.vue";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";

const gameData = useGameDataStore();
const craftingStore = useCraftingStore();

const craftingSkills = ref<SkillInfo[]>([]);
const selectedSkill = ref("");
const currentLevel = ref(0);
const targetLevel = ref(50);
const strategy = ref<LevelingStrategy>("combined");
const includeUnlearnedRecipes = ref(true);
const excludedRecipeIds = ref<Set<number>>(new Set());
const excludedRecipeNames = ref<Map<number, string>>(new Map());
const calculating = ref(false);
const creatingProject = ref(false);
const plan = ref<LevelingPlan | null>(null);
const error = ref("");
const snapshotLevel = ref<number | null>(null);

const unlearnedInPlan = computed(() =>
  plan.value?.steps.filter((s) => !s.is_known).length ?? 0,
);

onMounted(async () => {
  const allSkills = await gameData.getAllSkills();
  craftingSkills.value = allSkills
    .filter((s) => s.xp_table !== null)
    .sort((a, b) => a.name.localeCompare(b.name));
});

async function onSkillChange() {
  plan.value = null;
  error.value = "";
  snapshotLevel.value = null;
  excludedRecipeIds.value = new Set();
  excludedRecipeNames.value = new Map();

  if (!selectedSkill.value) return;

  const skillData = await craftingStore.getSkillLevel(selectedSkill.value);
  if (skillData) {
    snapshotLevel.value = skillData.level;
    currentLevel.value = skillData.level;
  } else {
    currentLevel.value = 0;
  }
}

async function calculate() {
  if (!selectedSkill.value || targetLevel.value <= currentLevel.value) return;

  calculating.value = true;
  error.value = "";
  plan.value = null;

  try {
    plan.value = await craftingStore.generateLevelingPlan(
      selectedSkill.value,
      currentLevel.value,
      targetLevel.value,
      strategy.value,
      includeUnlearnedRecipes.value,
      excludedRecipeIds.value,
    );
  } catch (e) {
    error.value = String(e);
    console.error("[crafting] Leveling plan failed:", e);
  } finally {
    calculating.value = false;
  }
}

async function createProjectFromPlan() {
  if (!plan.value || plan.value.steps.length === 0) return;
  creatingProject.value = true;

  try {
    const projectName = `${plan.value.skill_name} Lv${plan.value.current_level}→${plan.value.target_level}`;
    const projectId = await craftingStore.createProject(projectName, `Generated by Leveling Optimizer (${plan.value.strategy} strategy)`);

    // Add each unique recipe with its total craft count
    const recipeTotals = new Map<number, { name: string; count: number }>();
    for (const step of plan.value.steps) {
      const existing = recipeTotals.get(step.recipe_id);
      if (existing) {
        existing.count += step.craft_count;
      } else {
        recipeTotals.set(step.recipe_id, { name: step.recipe_name, count: step.craft_count });
      }
    }

    for (const [recipeId, { name, count }] of recipeTotals) {
      await craftingStore.addEntry(projectId, recipeId, name, count);
    }
  } catch (e) {
    error.value = String(e);
    console.error("[crafting] Failed to create project:", e);
  } finally {
    creatingProject.value = false;
  }
}

function excludeRecipe(recipeId: number, recipeName: string) {
  const newSet = new Set(excludedRecipeIds.value);
  newSet.add(recipeId);
  excludedRecipeIds.value = newSet;

  const newNames = new Map(excludedRecipeNames.value);
  newNames.set(recipeId, recipeName);
  excludedRecipeNames.value = newNames;

  calculate();
}

function restoreRecipe(recipeId: number) {
  const newSet = new Set(excludedRecipeIds.value);
  newSet.delete(recipeId);
  excludedRecipeIds.value = newSet;

  const newNames = new Map(excludedRecipeNames.value);
  newNames.delete(recipeId);
  excludedRecipeNames.value = newNames;

  calculate();
}

function clearExclusions() {
  excludedRecipeIds.value = new Set();
  excludedRecipeNames.value = new Map();
  calculate();
}
</script>
