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
      <span class="text-xs uppercase tracking-widest text-text-dim border border-border-light rounded px-2 py-0.5">
        {{ categoryLabel }}
      </span>
    </div>

    <!-- Description -->
    <p v-if="recipe.description" class="text-xs text-text-secondary m-0 leading-relaxed">
      {{ recipe.description }}
    </p>

    <!-- Fixed Ingredients -->
    <div v-if="recipe.fixed_ingredients.length > 0">
      <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Fixed Ingredients
      </div>
      <div class="flex flex-col gap-1">
        <div
          v-for="(ing, i) in recipe.fixed_ingredients"
          :key="i"
          class="flex items-center gap-2 text-xs">
          <span class="text-text-muted w-6 text-right shrink-0">{{ ing.stack_size }}x</span>
          <ItemInline :reference="String(ing.item_id)" />
          <span v-if="getOwnedCount(ing.item_id) > 0" class="text-xs text-accent-green">
            (×{{ getOwnedCount(ing.item_id) }})
          </span>
          <span v-if="ing.chance_to_consume != null && ing.chance_to_consume < 1"
            class="text-text-dim text-xs">
            ({{ Math.round(ing.chance_to_consume * 100) }}% consumed)
          </span>
        </div>
      </div>
    </div>

    <!-- Variable Ingredient Slots -->
    <div v-if="recipe.variable_slots.length > 0">
      <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Variable Ingredient Slots
        <span class="normal-case tracking-normal text-text-dim ml-1">({{ recipe.variable_slots.length }} slots determine the effect)</span>
      </div>
      <div class="flex flex-col gap-3">
        <div v-for="(slot, i) in recipe.variable_slots" :key="i" class="bg-surface-base border border-surface-elevated rounded px-3 py-2">
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-xs text-accent-gold bg-accent-gold/10 rounded px-1.5 py-0.5">
              {{ slot.keyword }}
            </span>
            <span class="text-text-muted text-xs">{{ slot.stack_size }}x needed</span>
          </div>
          <div class="flex flex-wrap gap-x-2 gap-y-1">
            <span
              v-for="itemId in slot.valid_item_ids"
              :key="itemId"
              class="text-xs inline-flex items-center gap-0.5">
              <ItemInline :reference="String(itemId)" />
              <span v-if="getOwnedCount(itemId) > 0" class="text-xs text-accent-green">
                (×{{ getOwnedCount(itemId) }})
              </span>
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
      <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Possible Effect Categories
        <span class="normal-case tracking-normal text-text-dim ml-1">(your brew will get one of these)</span>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="pool in dedupedPools"
          :key="pool"
          :title="getPoolDescription(pool)"
          :class="[
            'text-xs px-2 py-0.5 rounded border cursor-default',
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
      <div class="text-xs text-text-dim mt-1.5">
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
      <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Your Discoveries
        <span class="normal-case tracking-normal text-text-dim ml-1">({{ discoveries.length }} found)</span>
      </div>
      <table class="text-xs">
        <thead>
          <tr class="text-xs uppercase tracking-wider text-text-dim">
            <th
              v-for="(slot, si) in recipe.variable_slots"
              :key="si"
              class="text-left pb-1 font-normal w-36"
              :title="slot.keyword">
              Slot {{ si + 1 }}
            </th>
            <th class="text-left pb-1 font-normal">Effect</th>
            <th class="text-left pb-1 font-normal">Req</th>
            <th class="text-left pb-1 font-normal">Race</th>
            <th class="w-6"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="disc in discoveries"
            :key="disc.id"
            class="border-t border-surface-card align-top group">
            <td
              v-for="(ingId, si) in paddedIngredients(disc)"
              :key="si"
              class="py-1.5 pr-2 w-36">
              <template v-if="ingId !== null">
                <div class="inline-flex items-center gap-0.5">
                  <ItemInline :reference="String(ingId)" />
                  <span v-if="getOwnedCount(ingId) > 0" class="text-xs text-accent-green">
                    ×{{ getOwnedCount(ingId) }}
                  </span>
                </div>
              </template>
              <span v-else class="text-text-dim">—</span>
            </td>
            <td class="py-1.5 pr-3">
              <div class="flex flex-col gap-0.5">
                <span v-if="disc.effect_label" class="text-accent-gold font-semibold">{{ disc.effect_label }}</span>
                <span v-else class="text-text-secondary">{{ disc.power }}</span>
                <template v-if="getPowerInfo(disc)">
                  <div
                    v-for="(effect, ei) in getPowerInfo(disc)!.tier_effects"
                    :key="ei"
                    class="text-xs text-text-secondary leading-snug">
                    {{ effect }}
                  </div>
                </template>
                <div v-else class="text-xs text-text-dim">{{ disc.power }} (T{{ disc.power_tier }})</div>
              </div>
            </td>
            <td class="py-1.5 pr-3">
              <span v-if="getPowerInfo(disc)?.skill" class="text-xs text-text-muted whitespace-nowrap">
                {{ getPowerInfo(disc)!.skill }}
              </span>
              <span v-else class="text-text-dim">—</span>
            </td>
            <td class="py-1.5 pr-1">
              <span
                v-if="disc.race_restriction"
                class="text-xs px-1.5 py-0.5 rounded bg-accent-red/10 text-accent-red border border-accent-red/20 whitespace-nowrap">
                {{ disc.race_restriction }}
              </span>
              <span v-else class="text-text-dim">—</span>
            </td>
            <td class="py-1.5">
              <button
                class="text-text-dim hover:text-accent-red cursor-pointer bg-transparent border-none opacity-0 group-hover:opacity-100 transition-opacity text-xs"
                title="Delete this discovery"
                @click="confirmDelete(disc)">
                ✕
              </button>
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

    <!-- Try Next suggestions -->
    <div v-if="suggestions.length > 0">
      <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-1.5">
        Try Next
        <span class="normal-case tracking-normal text-text-dim ml-1">(untried combos you have ingredients for)</span>
      </div>
      <div class="flex flex-col gap-1.5">
        <div
          v-for="(sug, i) in suggestions"
          :key="i"
          class="flex items-center gap-2 bg-surface-base border rounded px-3 py-1.5"
          :class="sug.ownedCount === sug.totalCount
            ? 'border-accent-green/40'
            : sug.ownedCount > 0
              ? 'border-accent-gold/25'
              : 'border-surface-elevated'">
          <span
            v-if="sug.ownedCount === sug.totalCount"
            class="text-xs text-accent-green font-semibold shrink-0 w-12">
            ✓ Ready
          </span>
          <span
            v-else
            class="text-xs text-text-dim shrink-0 w-12">
            {{ sug.ownedCount }}/{{ sug.totalCount }}
          </span>
          <div class="flex flex-wrap gap-x-2 gap-y-0.5">
            <span
              v-for="ingId in sug.ingredientIds"
              :key="ingId"
              class="text-xs inline-flex items-center gap-0.5">
              <span
                class="w-1.5 h-1.5 rounded-full inline-block shrink-0"
                :class="hasItem(ingId) ? 'bg-accent-green' : 'bg-surface-elevated border border-border-light'" />
              <ItemInline :reference="String(ingId)" />
            </span>
          </div>
        </div>
      </div>
      <div class="text-xs text-text-dim mt-1">
        {{ discoveredCombos.size }} of {{ discoveredCombos.size + suggestions.length }} combos discovered
        <template v-if="totalUntriedCount > suggestions.length">
          · showing top {{ suggestions.length }} of {{ totalUntriedCount }} untried
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { confirm } from "@tauri-apps/plugin-dialog";
import ItemInline from "../Shared/Item/ItemInline.vue";
import type { BrewingRecipe, BrewingIngredient, BrewingDiscovery } from "../../types/gameData/brewing";
import { CATEGORY_LABELS, getPoolLabel, getPoolDescription } from "../../types/gameData/brewing";
import { useBreweryStore } from "../../stores/breweryStore";
import { useGameStateStore } from "../../stores/gameStateStore";

const props = defineProps<{
  recipe: BrewingRecipe;
  ingredientById: Map<number, BrewingIngredient>;
  discoveries: BrewingDiscovery[];
}>();

const store = useBreweryStore();
const gameState = useGameStateStore();

// Session-stable random seed for shuffling suggestions
const sessionSeed = Math.random();

const categoryLabel = computed(() => CATEGORY_LABELS[props.recipe.category]);

const dedupedPools = computed(() => {
  if (!props.recipe.brew_item_effect) return [];
  return [...new Set(props.recipe.brew_item_effect.effect_pools)];
});

function isPlaceholderPool(pool: string): boolean {
  return pool.startsWith("TBD");
}

/** Get the TSys power info from the store's bulk-fetched cache */
function getPowerInfo(disc: BrewingDiscovery) {
  return store.getPowerInfo(disc.power, disc.power_tier);
}

/** Get owned count for an item by type ID */
function getOwnedCount(itemTypeId: number): number {
  const ingredient = props.ingredientById.get(itemTypeId);
  if (!ingredient) return 0;
  return gameState.ownedItemCounts[ingredient.name] ?? 0;
}

function hasItem(itemTypeId: number): boolean {
  return getOwnedCount(itemTypeId) > 0;
}

/** Pad ingredient IDs to match the number of variable slots */
function paddedIngredients(disc: BrewingDiscovery): (number | null)[] {
  const slotCount = props.recipe.variable_slots.length;
  const result: (number | null)[] = [...disc.ingredient_ids];
  while (result.length < slotCount) result.push(null);
  return result;
}

async function confirmDelete(disc: BrewingDiscovery) {
  const label = disc.effect_label ?? disc.power;
  const ok = await confirm(`Delete discovery "${label}"? This cannot be undone.`, {
    title: "Delete Discovery",
    kind: "warning",
  });
  if (ok) {
    store.deleteDiscovery(disc.id);
  }
}

// ── "Try Next" suggestions ──────────────────────────────────────────────────

/** Set of sorted ingredient ID arrays that have already been discovered */
const discoveredCombos = computed(() => {
  const set = new Set<string>();
  for (const d of props.discoveries) {
    const key = [...d.ingredient_ids].sort((a, b) => a - b).join(",");
    set.add(key);
  }
  return set;
});

interface Suggestion {
  ingredientIds: number[];
  ownedCount: number;
  totalCount: number;
}

/** Generate untried combos, prioritized by ingredient availability. Capped to avoid explosion. */
const suggestions = computed((): Suggestion[] => {
  const slots = props.recipe.variable_slots;
  if (slots.length === 0) return [];

  // Get valid item IDs for each slot
  const slotOptions = slots.map((s) => s.valid_item_ids);

  // Guard against combinatorial explosion — if total combos > 500, skip
  const totalCombos = slotOptions.reduce((acc, opts) => acc * Math.max(opts.length, 1), 1);
  if (totalCombos > 500) return [];

  // Generate all combos via cartesian product
  const allCombos: number[][] = cartesian(slotOptions);

  // Filter out already-discovered combos
  const untried: Suggestion[] = [];
  for (const combo of allCombos) {
    const sorted = [...combo].sort((a, b) => a - b);
    const key = sorted.join(",");
    if (discoveredCombos.value.has(key)) continue;

    const ownedCount = combo.filter((id) => hasItem(id)).length;
    untried.push({
      ingredientIds: combo,
      ownedCount,
      totalCount: combo.length,
    });
  }

  // Sort: most owned ingredients first, then shuffle within each tier
  // using a seeded hash so it's stable per session but varies between sessions
  untried.sort((a, b) => {
    if (b.ownedCount !== a.ownedCount) return b.ownedCount - a.ownedCount;
    return seededHash(a.ingredientIds) - seededHash(b.ingredientIds);
  });

  // Return top suggestions
  return untried.slice(0, 5);
});

/** Total count of untried combos (for the "showing X of Y" message) */
const totalUntriedCount = computed(() => {
  const slots = props.recipe.variable_slots;
  if (slots.length === 0) return 0;
  const slotOptions = slots.map((s) => s.valid_item_ids);
  const totalCombos = slotOptions.reduce((acc, opts) => acc * Math.max(opts.length, 1), 1);
  if (totalCombos > 500) return 0;
  return totalCombos - discoveredCombos.value.size;
});

/** Cartesian product of arrays */
function cartesian(arrays: number[][]): number[][] {
  if (arrays.length === 0) return [[]];
  const [first, ...rest] = arrays;
  const restProduct = cartesian(rest);
  const result: number[][] = [];
  for (const item of first) {
    for (const combo of restProduct) {
      result.push([item, ...combo]);
    }
  }
  return result;
}

/** Simple seeded hash for stable-per-session shuffling */
function seededHash(ids: number[]): number {
  let h = sessionSeed * 2147483647;
  for (const id of ids) {
    h = ((h * 31) + id) % 2147483647;
  }
  return h;
}
</script>
