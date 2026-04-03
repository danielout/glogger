<template>
  <div class="flex-1 min-w-0 overflow-y-auto border border-surface-elevated rounded p-4 flex flex-col gap-3">
    <!-- Empty state -->
    <template v-if="!activeProject && !activeGroupName">
      <EmptyState variant="compact" primary="No project selected" secondary="Select a project to see materials." />
    </template>

    <template v-else>
      <!-- Header: group or project -->
      <div v-if="activeGroupName" class="flex flex-col gap-1">
        <h3 class="text-accent-gold text-base font-semibold m-0">{{ activeGroupName }}</h3>
        <div class="text-text-muted text-[0.65rem]">
          Group summary &mdash; {{ groupProjectNames.length }} project{{ groupProjectNames.length !== 1 ? 's' : '' }}
        </div>
      </div>

      <!-- Recipe summary: what you're crafting -->
      <div v-if="recipeEntries.length > 0" class="flex items-start gap-2">
        <div class="flex flex-wrap gap-x-4 gap-y-0.5 text-xs flex-1 min-w-0">
          <div
            v-for="entry in recipeEntries"
            :key="entry.id"
            class="flex items-center gap-1.5 text-text-muted">
            <RecipeInline :reference="entry.recipe_name" />
            <template v-if="entry.target_stock !== null">
              <span
                v-if="getStockTarget(entry.id)"
                class="text-[0.65rem]"
                :class="getStockTarget(entry.id)!.effectiveQty <= 0 ? 'text-green-400' : 'text-accent-gold'">
                {{ getStockTarget(entry.id)!.effectiveQty <= 0 ? 'met' : `×${getStockTarget(entry.id)!.effectiveQty}` }}
              </span>
              <span v-else class="font-mono text-[0.65rem] text-accent-gold">target {{ entry.target_stock }}</span>
            </template>
            <span v-else class="font-mono text-text-primary/70">&times;{{ entry.quantity }}</span>
          </div>
        </div>
        <button
          v-if="hasContent"
          class="text-[0.65rem] text-text-muted hover:text-text-primary cursor-pointer bg-transparent border border-border-light rounded px-1.5 py-0.5 shrink-0 transition-colors"
          :disabled="resolving"
          @click="$emit('resolve')">
          {{ resolving ? 'Refreshing...' : 'Recheck Inventory' }}
        </button>
      </div>

      <!-- Two-column layout for materials + actionable lists -->
      <div v-if="hasContent" class="grid grid-cols-1 lg:grid-cols-2 gap-3 items-start">
        <!-- Left: Materials overview -->
        <div class="flex flex-col gap-3">
          <!-- Unified materials table (with availability) -->
          <AccordionSection v-if="materialNeeds.length > 0">
            <template #title>Materials</template>
            <template #badge>
              <div class="flex gap-3 text-[0.65rem] text-text-muted">
                <span>{{ materialNeeds.length }}</span>
                <span><span class="text-green-400">{{ coveredCount }}</span> ok</span>
                <span v-if="partialCount > 0"><span class="text-yellow-400">{{ partialCount }}</span> partial</span>
                <span v-if="missingCount > 0"><span class="text-accent-red">{{ missingCount }}</span> missing</span>
              </div>
            </template>
            <MaterialSummary :needs="materialNeeds" :bare="true" />
          </AccordionSection>

          <!-- Fallback: raw materials table when availability hasn't been checked yet -->
          <AccordionSection v-else-if="materials.size > 0">
            <template #title>Materials</template>
            <template #badge>
              <span class="text-text-muted text-[0.65rem]">{{ materials.size }} items</span>
            </template>
            <table class="w-full text-xs">
              <thead>
                <tr class="text-text-dim border-b border-border-light">
                  <th class="text-left py-1 font-medium">Item</th>
                  <th class="text-right py-1 font-medium w-20">Qty</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="mat in sortedMaterials"
                  :key="mat.key"
                  class="border-b border-surface-dark">
                  <td class="py-1">
                    <template v-if="mat.is_dynamic">
                      <span class="text-accent-gold/60 text-[0.65rem] mr-1">&#9670;</span>
                      <span class="text-text-secondary">{{ mat.item_name }}</span>
                    </template>
                    <ItemInline v-else-if="mat.item_id !== null" :reference="mat.item_name" />
                    <span v-else class="text-text-muted italic">{{ mat.item_name }}</span>
                  </td>
                  <td class="text-right py-1 text-text-primary font-mono whitespace-nowrap">
                    {{ mat.expected_quantity }}
                    <span
                      v-if="mat.chance_to_consume < 1"
                      class="text-accent-gold cursor-help"
                      :title="`~${Math.round(mat.chance_to_consume * 100)}% chance to consume per use. Raw quantity: ${mat.quantity}`">
                      *
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </AccordionSection>

          <!-- Craft or Buy: unified intermediate management -->
          <AccordionSection v-if="craftableItems.length > 0">
            <template #title>Craft or Buy?</template>
            <template #badge>
              <span class="text-text-muted text-[0.65rem]">
                {{ expandedItemIds.size }} crafting, {{ craftableButNotExpanded.length }} buying
              </span>
            </template>
            <div class="flex flex-col gap-1">
              <div
                v-for="item in craftableItems"
                :key="item.item_id"
                class="flex items-center gap-2 px-2 py-1.5 rounded text-xs"
                :class="item.isExpanded
                  ? 'bg-accent-gold/5 border border-accent-gold/20'
                  : 'bg-surface-dark/30 border border-surface-elevated/50'">
                <ItemInline :reference="item.item_name" />
                <div class="flex flex-col text-[0.65rem] text-text-muted leading-tight">
                  <span>need {{ item.quantity }}</span>
                  <template v-if="item.have > 0">
                    <span v-if="item.toCraft > 0">
                      have <span class="text-green-400">{{ item.have }}</span>
                      → {{ item.isExpanded ? 'craft' : 'buy' }} <span class="text-text-primary">{{ item.toCraft }}</span>
                    </span>
                    <span v-else class="text-green-400">have {{ item.have }} ✓</span>
                  </template>
                </div>
                <button
                  v-if="!activeGroupName"
                  class="ml-auto text-[0.65rem] cursor-pointer bg-transparent border rounded px-1.5 py-0.5 transition-colors shrink-0"
                  :class="item.isExpanded
                    ? 'text-accent-gold border-accent-gold/40 bg-accent-gold/10 hover:bg-transparent hover:text-text-muted'
                    : 'text-text-muted border-border-light hover:text-accent-gold hover:border-accent-gold/30'"
                  @click="$emit('toggle-intermediate', item.item_id)">
                  {{ item.isExpanded ? '✓ crafting' : '+ craft' }}
                </button>
              </div>
            </div>
          </AccordionSection>
        </div>

        <!-- Right: Actionable lists -->
        <div class="flex flex-col gap-3">
          <!-- Shopping / gathering list -->
          <AccordionSection v-if="materialNeeds.length > 0 && hasShortfalls">
            <template #title>Shopping / Gathering</template>
            <template #badge>
              <span v-if="vendorCost > 0" class="text-text-muted text-[0.65rem]">
                ~<span class="text-accent-red font-semibold">{{ vendorCost.toLocaleString() }}g</span>
              </span>
            </template>
            <ShoppingList :needs="materialNeeds" :bare="true" />
          </AccordionSection>

          <!-- Pickup list -->
          <AccordionSection v-if="materialNeeds.length > 0 && hasPickupItems">
            <template #title>Pickup List</template>
            <template #badge>
              <span class="text-text-muted text-[0.65rem]">{{ pickupAreaCount }} area{{ pickupAreaCount !== 1 ? 's' : '' }}</span>
            </template>
            <PickupList :needs="materialNeeds" :bare="true" />
          </AccordionSection>

          <!-- "All covered" state when nothing to shop or pick up -->
          <div
            v-if="materialNeeds.length > 0 && !hasShortfalls && !hasPickupItems"
            class="border border-surface-elevated rounded px-3 py-4 text-center text-xs text-green-400">
            All materials in inventory — ready to craft!
          </div>
        </div>
      </div>

      <!-- Footer actions -->
      <div v-if="materialNeeds.length > 0 || (activeProject && !activeGroupName && materials.size > 0)" class="flex gap-2">
        <button
          v-if="materialNeeds.length > 0 && displayName"
          class="btn-secondary text-xs py-1.5 flex-1"
          @click="craftingStore.exportMaterialList(displayName, materialNeeds)">
          Export
        </button>

        <!-- Start tracking (project only) -->
        <button
          v-if="activeProject && !activeGroupName && materials.size > 0 && !tracker?.active"
          class="btn-secondary text-xs py-1.5 flex-1"
          @click="craftingStore.startProjectTracking()">
          Start Tracking
        </button>
      </div>

      <!-- Live crafting panel (project only) -->
      <LiveCraftingPanel v-if="activeProject && !activeGroupName" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingProject, CraftingProjectEntry, FlattenedMaterial, IntermediateCraft, MaterialNeed } from "../../types/crafting";
import EmptyState from "../Shared/EmptyState.vue";
import AccordionSection from "../Shared/AccordionSection.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import RecipeInline from "../Shared/Recipe/RecipeInline.vue";
import MaterialSummary from "./MaterialSummary.vue";
import PickupList from "./PickupList.vue";
import ShoppingList from "./ShoppingList.vue";
import LiveCraftingPanel from "./LiveCraftingPanel.vue";

const props = defineProps<{
  activeProject: CraftingProject | null
  activeGroupName: string | null
  groupProjectNames: string[]
  groupEntries: CraftingProjectEntry[]
  stockTargets: Map<number, { effectiveQty: number; currentStock: number }>
  materials: Map<string, FlattenedMaterial>
  intermediates: IntermediateCraft[]
  expandedItemIds: Set<number>
  intermediateStock: Map<number, number>
  materialNeeds: MaterialNeed[]
  resolving: boolean
  checkingAvailability: boolean
}>();

defineEmits<{
  'resolve': []
  'toggle-intermediate': [itemId: number]
}>();

const craftingStore = useCraftingStore();
const tracker = computed(() => craftingStore.tracker);

const displayName = computed(() =>
  props.activeGroupName ?? props.activeProject?.name ?? null,
);

/** The recipe entries to display in the summary */
const recipeEntries = computed(() => {
  if (props.activeGroupName) return props.groupEntries;
  return props.activeProject?.entries ?? [];
});

const hasEntries = computed(() => recipeEntries.value.length > 0);
const hasContent = computed(() => props.materials.size > 0 || props.materialNeeds.length > 0 || props.intermediates.length > 0);

// Shorthands for template readability
const materials = computed(() => props.materials);

function getStockTarget(entryId: number) {
  return props.stockTargets.get(entryId) ?? null;
}

const sortedMaterials = computed(() => {
  return Array.from(props.materials.values())
    .sort((a, b) => {
      if (a.is_dynamic !== b.is_dynamic) return a.is_dynamic ? 1 : -1;
      return a.item_name.localeCompare(b.item_name);
    });
});

interface CraftableItem {
  item_id: number
  item_name: string
  quantity: number
  have: number
  toCraft: number
  isExpanded: boolean
  intermediate: IntermediateCraft | null
}

/**
 * All craftable items across the project — both currently expanded (being crafted)
 * and not expanded (being bought). Includes availability info so users can see
 * "need 10, have 5, craft 5".
 */
const craftableItems = computed((): CraftableItem[] => {
  const items = new Map<number, CraftableItem>();

  // Build availability lookup from materialNeeds
  const availLookup = new Map<number, { have: number }>();
  for (const need of props.materialNeeds) {
    availLookup.set(need.item_id, { have: need.inventory_have + need.storage_have });
  }

  // Craftable items from flat materials (not currently expanded)
  for (const mat of props.materials.values()) {
    if (mat.item_id !== null && mat.is_craftable && !props.expandedItemIds.has(mat.item_id)) {
      const avail = availLookup.get(mat.item_id);
      const have = avail?.have ?? 0;
      items.set(mat.item_id, {
        item_id: mat.item_id,
        item_name: mat.item_name,
        quantity: mat.expected_quantity,
        have,
        toCraft: Math.max(0, mat.expected_quantity - have),
        isExpanded: false,
        intermediate: null,
      });
    }
  }

  // Currently expanded intermediates — use intermediateStock for have/toCraft
  for (const inter of props.intermediates) {
    const onHand = props.intermediateStock.get(inter.item_id) ?? 0;
    if (!items.has(inter.item_id)) {
      // quantity_produced is the shortfall amount (already stock-adjusted by resolver)
      // The total needed is shortfall + what we already have
      const totalNeeded = inter.quantity_produced + onHand;
      items.set(inter.item_id, {
        item_id: inter.item_id,
        item_name: inter.item_name,
        quantity: totalNeeded,
        have: onHand,
        toCraft: inter.quantity_produced,
        isExpanded: true,
        intermediate: inter,
      });
    } else {
      const existing = items.get(inter.item_id)!;
      existing.isExpanded = true;
      existing.intermediate = inter;
      existing.have = onHand;
      existing.toCraft = Math.max(0, existing.quantity - onHand);
    }
  }

  // Sort: expanded first, then alphabetical
  return Array.from(items.values()).sort((a, b) => {
    if (a.isExpanded !== b.isExpanded) return a.isExpanded ? -1 : 1;
    return a.item_name.localeCompare(b.item_name);
  });
});

const craftableButNotExpanded = computed(() =>
  craftableItems.value.filter((i) => !i.isExpanded),
);

const coveredCount = computed(() => props.materialNeeds.filter((m) => m.shortfall === 0).length);
const partialCount = computed(() =>
  props.materialNeeds.filter((m) => m.shortfall > 0 && (m.inventory_have > 0 || m.storage_have > 0)).length
);
const missingCount = computed(() =>
  props.materialNeeds.filter((m) => m.shortfall > 0 && m.inventory_have === 0 && m.storage_have === 0).length
);

const hasPickupItems = computed(() =>
  props.materialNeeds.some((m) => m.storage_have > 0)
);

const pickupAreaCount = computed(() => {
  const areas = new Set<string>();
  for (const mat of props.materialNeeds) {
    if (mat.storage_have === 0) continue;
    for (const vs of mat.vault_breakdown) {
      if (vs.quantity > 0) areas.add(vs.vault_name);
    }
  }
  return areas.size;
});

const hasShortfalls = computed(() =>
  props.materialNeeds.some((m) => m.shortfall > 0)
);

const vendorCost = computed(() =>
  props.materialNeeds
    .filter((m) => m.shortfall > 0 && m.vendor_price)
    .reduce((sum, m) => sum + (m.vendor_price! * m.shortfall), 0)
);
</script>
