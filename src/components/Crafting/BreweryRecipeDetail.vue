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
        Possible Effect Categories
        <span class="normal-case tracking-normal text-text-dim ml-1">(your brew will get one of these)</span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="pool in dedupedPools"
          :key="pool"
          :title="getPoolDescription(pool)"
          :class="[
            'text-[0.6rem] px-2 py-0.5 rounded border cursor-default',
            isPlaceholderPool(pool)
              ? 'border-accent-warning/30 text-accent-warning bg-accent-warning/5'
              : pool.startsWith('RacialBonuses')
                ? 'border-accent-red/30 text-accent-red/80 bg-accent-red/5'
                : 'border-border-light text-text-secondary bg-surface-base',
          ]">
          {{ getPoolLabel(pool) }}
          <span v-if="isPlaceholderPool(pool)" class="ml-1 opacity-60">(not yet implemented)</span>
          <span v-if="pool.startsWith('RacialBonuses')" class="ml-1 opacity-60">(may be race-locked)</span>
        </span>
      </div>
      <div class="text-[0.6rem] text-text-dim mt-1.5">
        Tier {{ recipe.brew_item_effect.tier }}
        <span class="mx-1 opacity-30">·</span>
        {{ recipe.brew_item_effect.ingredient_slots.length }} variable slot{{ recipe.brew_item_effect.ingredient_slots.length === 1 ? '' : 's' }} determine which effect you get
      </div>
    </div>

    <!-- No variable slots message for simple recipes -->
    <div v-if="recipe.variable_slots.length === 0 && !recipe.brew_item_effect"
      class="text-xs text-text-dim italic bg-surface-base border border-surface-elevated rounded px-3 py-2">
      This recipe has no variable ingredient slots — the output is always the same.
    </div>

    <!-- Discoveries -->
    <div v-if="discoveries.length > 0">
      <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Your Discoveries
        <span class="normal-case tracking-normal text-text-dim ml-1">({{ discoveries.length }} found)</span>
      </div>
      <table class="text-xs">
        <thead>
          <tr class="text-[0.6rem] uppercase tracking-wider text-text-dim">
            <th class="text-left pb-1 font-normal">Ingredients</th>
            <th class="text-left pb-1 font-normal">Effect</th>
            <th class="text-left pb-1 font-normal">Req</th>
            <th class="text-left pb-1 font-normal">Race</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="disc in discoveries"
            :key="disc.id"
            class="border-t border-surface-card align-top">
            <td class="py-1.5 pr-3">
              <div class="flex flex-wrap gap-1">
                <ItemInline
                  v-for="ingId in disc.ingredient_ids"
                  :key="ingId"
                  :reference="String(ingId)" />
              </div>
            </td>
            <td class="py-1.5 pr-3">
              <div class="flex flex-col gap-0.5">
                <span v-if="disc.effect_label" class="text-accent-gold font-semibold">{{ disc.effect_label }}</span>
                <span v-else class="text-text-secondary">{{ disc.power }}</span>
                <!-- Resolved effect descriptions from TSys -->
                <template v-if="getPowerInfo(disc)">
                  <div
                    v-for="(effect, ei) in getPowerInfo(disc)!.tier_effects"
                    :key="ei"
                    class="text-[0.6rem] text-text-secondary leading-snug">
                    {{ effect }}
                  </div>
                </template>
                <div v-else class="text-[0.6rem] text-text-dim">{{ disc.power }} (T{{ disc.power_tier }})</div>
              </div>
            </td>
            <td class="py-1.5 pr-3">
              <span v-if="getPowerInfo(disc)?.skill" class="text-[0.6rem] text-text-muted whitespace-nowrap">
                {{ getPowerInfo(disc)!.skill }}
              </span>
              <span v-else class="text-text-dim">—</span>
            </td>
            <td class="py-1.5">
              <span
                v-if="disc.race_restriction"
                class="text-[0.6rem] px-1.5 py-0.5 rounded bg-accent-red/10 text-accent-red border border-accent-red/20 whitespace-nowrap">
                {{ disc.race_restriction }} only
              </span>
              <span v-else class="text-text-dim">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- No discoveries yet prompt -->
    <div v-else-if="recipe.variable_slots.length > 0"
      class="text-xs text-text-dim italic bg-surface-base border border-surface-elevated rounded px-3 py-2">
      No discoveries for this recipe yet. Click "Scan Snapshots" to extract brewing data from your inventory exports.
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ItemInline from "../Shared/Item/ItemInline.vue";
import type { BrewingRecipe, BrewingIngredient, BrewingDiscovery } from "../../types/gameData/brewing";
import { CATEGORY_LABELS, getPoolLabel, getPoolDescription } from "../../types/gameData/brewing";

interface TsysPowerInfo {
  internal_name: string;
  skill: string | null;
  prefix: string | null;
  suffix: string | null;
  tier_effects: string[];
  icon_id: number | null;
}

const props = defineProps<{
  recipe: BrewingRecipe;
  ingredientById: Map<number, BrewingIngredient>;
  discoveries: BrewingDiscovery[];
}>();

const categoryLabel = computed(() => CATEGORY_LABELS[props.recipe.category]);

const dedupedPools = computed(() => {
  if (!props.recipe.brew_item_effect) return [];
  return [...new Set(props.recipe.brew_item_effect.effect_pools)];
});

function isPlaceholderPool(pool: string): boolean {
  return pool.startsWith("TBD");
}

// ── TSys power info lookups ─────────────────────────────────────────────────

const powerInfoCache = ref<Map<string, TsysPowerInfo>>(new Map());

/** Unique power+tier combos from current discoveries */
const uniquePowerKeys = computed(() => {
  const keys = new Set<string>();
  for (const d of props.discoveries) {
    keys.add(`${d.power}:${d.power_tier}`);
  }
  return keys;
});

/** Look up cached power info for a discovery */
function getPowerInfo(disc: BrewingDiscovery): TsysPowerInfo | undefined {
  return powerInfoCache.value.get(`${disc.power}:${disc.power_tier}`);
}

/** Fetch TSys power info for all unique powers in discoveries */
async function fetchPowerInfos() {
  for (const key of uniquePowerKeys.value) {
    if (powerInfoCache.value.has(key)) continue;
    const [powerName, tierStr] = key.split(":");
    const tier = parseInt(tierStr);
    try {
      const info = await invoke<TsysPowerInfo | null>("get_tsys_power_info", {
        powerName,
        tier,
      });
      if (info) {
        powerInfoCache.value.set(key, info);
      }
    } catch {
      // silently skip — CDN may not have this power
    }
  }
}

// Fetch when discoveries change
watch(
  () => props.discoveries,
  () => { fetchPowerInfos(); },
  { immediate: true }
);
</script>
