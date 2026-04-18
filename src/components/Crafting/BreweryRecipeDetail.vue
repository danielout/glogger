<template>
  <div class="p-4 flex flex-col gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-sm font-bold text-text-primary m-0">{{ recipe.name }}</h2>
        <div class="text-xs text-text-muted mt-0.5">
          <span>Level {{ recipe.skill_level_req }}</span>
          <span class="mx-1.5 opacity-30">·</span>
          <span>{{ recipe.xp }} XP</span>
          <span v-if="recipe.usage_delay_message" class="mx-1.5 opacity-30">·</span>
          <span v-if="recipe.usage_delay_message" class="text-text-dim">{{ recipe.usage_delay_message }}</span>
        </div>
      </div>
      <span class="text-[0.6rem] uppercase tracking-widest text-text-dim border border-border-light rounded px-2 py-0.5">
        {{ categoryLabel }}
      </span>
    </div>

    <!-- Description -->
    <p v-if="recipe.description" class="text-xs text-text-secondary m-0 leading-relaxed">
      {{ recipe.description }}
    </p>

    <!-- Fixed Ingredients -->
    <div v-if="recipe.fixed_ingredients.length > 0">
      <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Fixed Ingredients
      </div>
      <div class="flex flex-col gap-1">
        <div
          v-for="(ing, i) in recipe.fixed_ingredients"
          :key="i"
          class="flex items-center gap-2 text-xs">
          <span class="font-mono text-text-muted w-6 text-right shrink-0">{{ ing.stack_size }}x</span>
          <ItemInline :reference="String(ing.item_id)" />
          <span v-if="ing.chance_to_consume != null && ing.chance_to_consume < 1"
            class="text-text-dim text-[0.6rem]">
            ({{ Math.round(ing.chance_to_consume * 100) }}% consumed)
          </span>
        </div>
      </div>
    </div>

    <!-- Variable Ingredient Slots -->
    <div v-if="recipe.variable_slots.length > 0">
      <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Variable Ingredient Slots
        <span class="normal-case tracking-normal text-text-dim ml-1">({{ recipe.variable_slots.length }} slots determine the effect)</span>
      </div>
      <div class="flex flex-col gap-3">
        <div v-for="(slot, i) in recipe.variable_slots" :key="i" class="bg-surface-base border border-surface-elevated rounded px-3 py-2">
          <div class="flex items-center gap-2 mb-1">
            <span class="text-[0.6rem] font-mono text-accent-gold bg-accent-gold/10 rounded px-1.5 py-0.5">
              {{ slot.keyword }}
            </span>
            <span class="text-text-muted text-[0.6rem]">{{ slot.stack_size }}x needed</span>
          </div>
          <div v-if="slot.description" class="text-xs text-text-secondary mb-1.5">
            {{ slot.description }}
          </div>
          <div class="flex flex-wrap gap-1.5">
            <span
              v-for="itemId in slot.valid_item_ids"
              :key="itemId"
              class="text-xs">
              <ItemInline :reference="String(itemId)" />
            </span>
          </div>
          <div v-if="slot.valid_item_ids.length === 0" class="text-xs text-text-dim italic">
            No matching items found in CDN data
          </div>
        </div>
      </div>
    </div>

    <!-- Effect Pool Info -->
    <div v-if="recipe.brew_item_effect">
      <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Effect Pools
        <span class="normal-case tracking-normal text-text-dim ml-1">(possible effect categories for this recipe)</span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="pool in recipe.brew_item_effect.effect_pools"
          :key="pool"
          :class="[
            'text-[0.6rem] px-2 py-0.5 rounded border',
            isPlaceholderPool(pool)
              ? 'border-accent-warning/30 text-accent-warning bg-accent-warning/5'
              : 'border-border-light text-text-secondary bg-surface-base',
          ]">
          {{ pool }}
          <span v-if="isPlaceholderPool(pool)" class="ml-1 opacity-60">(placeholder)</span>
        </span>
      </div>
      <div class="text-[0.6rem] text-text-dim mt-1.5">
        Tier {{ recipe.brew_item_effect.tier }}
        <span class="mx-1 opacity-30">·</span>
        {{ recipe.brew_item_effect.ingredient_slots.length }} variable slot{{ recipe.brew_item_effect.ingredient_slots.length === 1 ? '' : 's' }} determine the effect
        <span class="mx-1 opacity-30">·</span>
        {{ dedupedPools.length }} unique pool{{ dedupedPools.length === 1 ? '' : 's' }}
      </div>
    </div>

    <!-- No variable slots message for simple recipes -->
    <div v-if="recipe.variable_slots.length === 0 && !recipe.brew_item_effect"
      class="text-xs text-text-dim italic bg-surface-base border border-surface-elevated rounded px-3 py-2">
      This recipe has no variable ingredient slots — the output is always the same.
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import type { BrewingRecipe, BrewingIngredient } from "../../types/gameData/brewing";
import { CATEGORY_LABELS } from "../../types/gameData/brewing";

const props = defineProps<{
  recipe: BrewingRecipe;
  ingredientById: Map<number, BrewingIngredient>;
}>();

const categoryLabel = computed(() => CATEGORY_LABELS[props.recipe.category]);

const dedupedPools = computed(() => {
  if (!props.recipe.brew_item_effect) return [];
  return [...new Set(props.recipe.brew_item_effect.effect_pools)];
});

function isPlaceholderPool(pool: string): boolean {
  return pool.startsWith("TBD");
}
</script>
