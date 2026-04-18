<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Search -->
    <div class="px-2 pt-1 pb-1">
      <input
        v-model="store.effectSearchQuery"
        type="text"
        placeholder="Search effects..."
        class="input text-xs w-full" />
    </div>

    <!-- No discoveries state -->
    <div v-if="store.uniqueEffects.length === 0" class="px-2 py-4 text-center">
      <div class="text-text-dim text-xs italic">
        No effects discovered yet.
      </div>
      <div class="text-text-dim text-[0.6rem] mt-1">
        Scan inventory snapshots or import a CSV to populate.
      </div>
    </div>

    <template v-else>
      <!-- Two-section layout: effect list + detail -->
      <div class="flex-1 min-h-0 flex flex-col">
        <!-- Effect list (scrollable, takes ~40% height) -->
        <div class="overflow-y-auto border-b border-surface-card" :class="store.selectedEffect ? 'max-h-[40%]' : 'flex-1'">
          <button
            v-for="effect in store.filteredEffects"
            :key="effect.power"
            class="flex items-center justify-between px-2 py-1 text-xs text-left cursor-pointer border-none w-full"
            :class="store.selectedEffect === effect.power
              ? 'bg-accent-gold/15 text-accent-gold'
              : effect.raceRestriction
                ? 'bg-transparent text-accent-red/60 hover:bg-surface-base'
                : 'bg-transparent text-text-secondary hover:bg-surface-base'"
            @click="store.selectEffect(effect.power)">
            <div class="truncate">
              <span>{{ effect.effectLabel ?? effect.power }}</span>
              <span v-if="effect.raceRestriction" class="text-[0.55rem] text-accent-red/60 ml-1">
                ({{ effect.raceRestriction }})
              </span>
            </div>
            <span class="text-text-dim font-mono text-[0.55rem] shrink-0 ml-1">
              {{ effect.discoveryCount }}
            </span>
          </button>

          <div v-if="store.filteredEffects.length === 0" class="text-text-dim text-xs italic px-2 py-2 text-center">
            No matching effects.
          </div>
        </div>

        <!-- Effect detail: all recipes that can produce this effect -->
        <div v-if="store.selectedEffect" class="flex-1 min-h-0 overflow-y-auto px-2 py-2">
          <div class="flex items-center justify-between mb-1.5">
            <div class="text-[0.65rem] uppercase tracking-widest text-text-dim">
              Recipes for this effect
            </div>
            <button
              class="text-[0.55rem] text-text-dim hover:text-text-secondary cursor-pointer bg-transparent border-none"
              @click="store.clearEffectSelection()">
              ✕
            </button>
          </div>

          <div class="flex flex-col gap-2">
            <div
              v-for="disc in store.selectedEffectDiscoveries"
              :key="disc.id"
              class="bg-surface-base border rounded px-2 py-1.5"
              :class="hasAllIngredients(disc)
                ? 'border-accent-green/30'
                : hasSomeIngredients(disc)
                  ? 'border-accent-gold/20'
                  : 'border-surface-elevated'">
              <!-- Recipe name -->
              <div class="flex items-center justify-between mb-1">
                <button
                  class="text-[0.6rem] text-text-muted hover:text-accent-gold cursor-pointer bg-transparent border-none p-0 text-left"
                  @click="navigateToRecipe(disc.recipe_id)"
                  :title="'Show recipe: ' + getRecipeName(disc.recipe_id)">
                  {{ getRecipeName(disc.recipe_id) }}
                </button>
                <span v-if="hasAllIngredients(disc)" class="text-[0.55rem] text-accent-green">
                  ✓ have all
                </span>
                <span v-else-if="hasSomeIngredients(disc)" class="text-[0.55rem] text-accent-gold">
                  partial
                </span>
              </div>

              <!-- Ingredients with availability -->
              <div class="flex flex-wrap gap-x-1.5 gap-y-0.5">
                <span
                  v-for="ingId in disc.ingredient_ids"
                  :key="ingId"
                  class="text-[0.6rem] inline-flex items-center gap-0.5"
                  :class="hasItem(ingId) ? 'text-text-primary' : 'text-text-dim'">
                  <span
                    class="w-1.5 h-1.5 rounded-full inline-block shrink-0"
                    :class="hasItem(ingId) ? 'bg-accent-green' : 'bg-surface-elevated'" />
                  {{ getIngredientName(ingId) }}
                </span>
              </div>
            </div>
          </div>

          <div v-if="store.selectedEffectDiscoveries.length === 0" class="text-text-dim text-xs italic text-center py-2">
            No discoveries for this effect.
          </div>
        </div>
      </div>
    </template>

    <!-- Footer -->
    <div class="text-[0.6rem] text-text-muted px-2 py-1 border-t border-surface-card">
      {{ store.uniqueEffects.length }} effects discovered
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useBreweryStore } from "../../stores/breweryStore";
import { useGameStateStore } from "../../stores/gameStateStore";
import type { BrewingDiscovery } from "../../types/gameData/brewing";

const store = useBreweryStore();
const gameState = useGameStateStore();

const emit = defineEmits<{
  (e: "navigate-recipe", recipeId: number): void;
}>();

/** Owned item counts from game state (inventory + storage) */
const ownedCounts = computed(() => gameState.ownedItemCounts);

/** Check if player has an ingredient by type ID */
function hasItem(itemTypeId: number): boolean {
  const ingredient = store.ingredientById.get(itemTypeId);
  if (!ingredient) return false;
  // Check by item name in ownedItemCounts
  return (ownedCounts.value[ingredient.name] ?? 0) > 0;
}

/** Check if player has all ingredients for a discovery */
function hasAllIngredients(disc: BrewingDiscovery): boolean {
  return disc.ingredient_ids.every((id) => hasItem(id));
}

/** Check if player has at least one ingredient */
function hasSomeIngredients(disc: BrewingDiscovery): boolean {
  return disc.ingredient_ids.some((id) => hasItem(id));
}

function getIngredientName(itemId: number): string {
  return store.ingredientById.get(itemId)?.name ?? `#${itemId}`;
}

function getRecipeName(recipeId: number): string {
  return store.recipeById.get(recipeId)?.name ?? `Recipe #${recipeId}`;
}

function navigateToRecipe(recipeId: number) {
  store.selectRecipe(recipeId);
  emit("navigate-recipe", recipeId);
}
</script>
