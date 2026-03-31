<template>
  <div class="bg-surface-base border border-surface-elevated rounded text-xs">
    <!-- Header row (always visible) -->
    <div
      class="flex items-center gap-3 px-3 py-2 cursor-pointer select-none hover:bg-surface-elevated/30"
      @click="expanded = !expanded">
      <span class="text-text-secondary w-3 text-center text-xs">
        {{ expanded ? '&#9662;' : '&#9656;' }}
      </span>
      <RecipeInline :reference="entry.recipe_name" />

      <!-- Quantity / target display -->
      <template v-if="isTargetMode">
        <span class="text-accent-gold font-mono text-[0.65rem]" title="Stock target mode">
          target {{ entry.target_stock }}
        </span>
        <span v-if="stockTarget" class="text-text-muted text-[0.65rem]">
          <template v-if="stockTarget.effectiveQty <= 0">
            <span class="text-green-400">met</span>
          </template>
          <template v-else>
            have {{ stockTarget.currentStock }}, craft {{ stockTarget.effectiveQty }}
          </template>
        </span>
      </template>
      <template v-else>
        <span class="text-text-primary font-mono">&times;{{ entry.quantity }}</span>
      </template>

      <span v-if="estimatedTotalCost" class="text-text-muted text-[0.65rem]">~{{ estimatedTotalCost.toLocaleString() }}g</span>

      <div class="ml-auto flex items-center gap-2" @click.stop>
        <!-- Mode toggle -->
        <button
          class="text-[0.6rem] cursor-pointer bg-transparent border border-border-light rounded px-1.5 py-0.5 hover:border-border-default"
          :class="isTargetMode ? 'text-accent-gold border-accent-gold/30' : 'text-text-muted'"
          :title="isTargetMode ? 'Switch to manual quantity' : 'Switch to stock target mode'"
          @click="toggleTargetMode">
          {{ isTargetMode ? 'target' : 'manual' }}
        </button>

        <!-- Quantity/target input -->
        <input
          v-if="isTargetMode"
          :value="entry.target_stock"
          type="number"
          min="0"
          class="input w-14 text-xs text-center"
          title="Target stock count"
          @change="onTargetStockChange" />
        <input
          v-else
          :value="entry.quantity"
          type="number"
          min="1"
          class="input w-14 text-xs text-center"
          @change="(e: Event) => $emit('update-qty', entry.id, (e.target as HTMLInputElement).valueAsNumber)" />

        <button
          class="text-accent-red/60 text-xs cursor-pointer bg-transparent border-none hover:text-accent-red"
          @click="$emit('remove', entry.id)">
          &#10005;
        </button>
      </div>
    </div>

    <!-- Expanded ingredient list -->
    <div v-if="expanded && recipe" class="border-t border-surface-dark/50 bg-surface-dark/20 px-2 py-1">
      <div v-if="loading" class="text-text-dim italic py-2 px-2">Loading ingredients...</div>
      <template v-else>
        <IngredientRow
          v-for="(ing, i) in resolvedIngredients"
          :key="ing.item_id ?? `dyn-${i}`"
          :ingredient="ing"
          :is-marked-for-crafting="isMarkedForCrafting(ing.item_id)"
          @toggle-intermediate="(itemId) => $emit('toggle-intermediate', entry.id, itemId)" />
        <div v-if="resolvedIngredients.length === 0" class="text-text-dim italic py-2 px-2">
          No ingredients
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useGameDataStore } from "../../stores/gameDataStore";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingProjectEntry, ResolvedIngredient } from "../../types/crafting";
import type { RecipeInfo } from "../../types/gameData/recipes";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";
import IngredientRow from "./IngredientRow.vue";

const props = defineProps<{
  entry: CraftingProjectEntry
  intermediateExpansions: Map<string, boolean>
  stockTarget?: { effectiveQty: number; currentStock: number }
}>();

const emit = defineEmits<{
  'update-qty': [entryId: number, qty: number]
  'remove': [entryId: number]
  'toggle-intermediate': [entryId: number, itemId: number | null]
  'update-target-stock': [entryId: number, targetStock: number | null]
}>();

const gameData = useGameDataStore();
const craftingStore = useCraftingStore();

const expanded = ref(false);
const loading = ref(false);
const estimatedTotalCost = ref<number | null>(null);

defineExpose({ expanded });
const recipe = ref<RecipeInfo | null>(null);
const resolvedIngredients = ref<ResolvedIngredient[]>([]);

const isTargetMode = computed(() => props.entry.target_stock !== null);

// Effective quantity for cost calculation: use stock target result if available
const effectiveQuantity = computed(() => {
  if (isTargetMode.value && props.stockTarget) {
    return props.stockTarget.effectiveQty;
  }
  return props.entry.quantity;
});

function toggleTargetMode() {
  if (isTargetMode.value) {
    // Switch to manual: clear target, keep current quantity
    emit('update-target-stock', props.entry.id, null);
  } else {
    // Switch to target: set target to current quantity
    emit('update-target-stock', props.entry.id, props.entry.quantity);
  }
}

function onTargetStockChange(e: Event) {
  const val = (e.target as HTMLInputElement).valueAsNumber;
  if (isNaN(val) || val < 0) return;
  emit('update-target-stock', props.entry.id, val);
}

async function computeCost() {
  let r = recipe.value;
  if (!r) {
    r = await gameData.resolveRecipe(props.entry.recipe_name);
    if (r) recipe.value = r;
  }
  if (!r) { estimatedTotalCost.value = null; return; }

  const perCraftCost = await craftingStore.estimateRecipeCost(r);
  if (!perCraftCost) { estimatedTotalCost.value = null; return; }

  const qty = effectiveQuantity.value;
  if (qty <= 0) { estimatedTotalCost.value = 0; return; }

  const outputPerCraft = r.result_items[0]?.stack_size ?? 1;
  const primaryChance = (r.result_items[0]?.percent_chance ?? 100) / 100;
  const effectiveOutput = outputPerCraft * primaryChance;
  const craftCount = Math.ceil(qty / effectiveOutput);
  estimatedTotalCost.value = Math.round(perCraftCost * craftCount);
}

onMounted(computeCost);

function isMarkedForCrafting(itemId: number | null): boolean {
  if (itemId === null) return false;
  // Check project-wide: any entry having this item expanded means it's being crafted
  for (const [key, value] of props.intermediateExpansions) {
    if (value && key.endsWith(`:${itemId}`)) return true;
  }
  return false;
}

watch(expanded, async (isExpanded) => {
  if (isExpanded && resolvedIngredients.value.length === 0) {
    loading.value = true;
    try {
      if (!recipe.value) {
        recipe.value = await gameData.resolveRecipe(props.entry.recipe_name);
      }
      if (recipe.value) {
        const resolved = await craftingStore.resolveRecipeIngredients(
          recipe.value,
          effectiveQuantity.value,
          false,
        );
        resolvedIngredients.value = resolved.ingredients;
      }
    } catch (e) {
      console.error("[crafting] Failed to resolve entry ingredients:", e);
    } finally {
      loading.value = false;
    }
  }
});

// Re-resolve if quantity or stock target changes while expanded, and recompute cost
watch([() => props.entry.quantity, () => props.stockTarget], async () => {
  computeCost();
  if (expanded.value && recipe.value) {
    const resolved = await craftingStore.resolveRecipeIngredients(
      recipe.value,
      effectiveQuantity.value,
      false,
    );
    resolvedIngredients.value = resolved.ingredients;
  }
});
</script>
