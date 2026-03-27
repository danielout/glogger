<template>
  <div class="flex-1 min-w-0 overflow-y-auto border border-surface-elevated rounded p-4 flex flex-col gap-3">
    <template v-if="!activeProject">
      <EmptyState variant="compact" primary="No project selected" secondary="Select a project to see materials." />
    </template>

    <template v-else>
      <!-- Calculate button -->
      <div v-if="activeProject.entries.length > 0" class="flex gap-2">
        <button
          class="btn-primary text-xs py-1.5 flex-1"
          :disabled="resolving"
          @click="$emit('resolve')">
          {{ resolving ? 'Calculating...' : 'Calculate All Materials' }}
        </button>
      </div>

      <!-- Intermediate crafts -->
      <AccordionSection v-if="intermediates.length > 0">
        <template #title>Intermediate Crafts</template>
        <template #badge>
          <span class="text-text-muted text-[0.65rem]">{{ intermediates.length }}</span>
        </template>
        <IntermediateCraftsList :intermediates="intermediates" :bare="true" />
      </AccordionSection>

      <!-- Materials table -->
      <AccordionSection v-if="materials.size > 0">
        <template #title>Total Materials Needed</template>
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

        <!-- Check inventory -->
        <button
          class="btn-secondary text-xs py-1.5 mt-3 w-full"
          :disabled="checkingAvailability"
          @click="$emit('check-availability')">
          {{ checkingAvailability ? 'Checking...' : 'Check Inventory' }}
        </button>
      </AccordionSection>

      <!-- Material availability breakdown -->
      <AccordionSection v-if="materialNeeds.length > 0">
        <template #title>Material Availability</template>
        <template #badge>
          <div class="flex gap-3 text-[0.65rem] text-text-muted">
            <span><span class="text-green-400">{{ coveredCount }}</span> covered</span>
            <span><span class="text-yellow-400">{{ partialCount }}</span> partial</span>
            <span><span class="text-accent-red">{{ missingCount }}</span> missing</span>
          </div>
        </template>
        <MaterialSummary :needs="materialNeeds" :bare="true" />
      </AccordionSection>

      <!-- Pickup list -->
      <AccordionSection v-if="materialNeeds.length > 0 && hasPickupItems">
        <template #title>Pickup List</template>
        <template #badge>
          <span class="text-text-muted text-[0.65rem]">{{ pickupAreaCount }} area{{ pickupAreaCount !== 1 ? 's' : '' }}</span>
        </template>
        <PickupList :needs="materialNeeds" :bare="true" />
      </AccordionSection>

      <!-- Shopping / gathering list -->
      <AccordionSection v-if="materialNeeds.length > 0 && hasShortfalls">
        <template #title>Shopping / Gathering List</template>
        <template #badge>
          <span v-if="vendorCost > 0" class="text-text-muted text-[0.65rem]">
            Est. cost: <span class="text-accent-red font-semibold">{{ vendorCost.toLocaleString() }}g</span>
          </span>
        </template>
        <ShoppingList :needs="materialNeeds" :bare="true" />
      </AccordionSection>

      <!-- Export button -->
      <button
        v-if="materialNeeds.length > 0 && activeProject"
        class="btn-secondary text-xs py-1.5 w-full"
        @click="craftingStore.exportMaterialList(activeProject.name, materialNeeds)">
        Export Material List
      </button>

      <!-- Start tracking button -->
      <div v-if="activeProject.entries.length > 0 && materials.size > 0">
        <button
          v-if="!tracker?.active"
          class="btn-secondary text-xs py-1.5 w-full"
          @click="craftingStore.startProjectTracking()">
          Start Tracking
        </button>
      </div>

      <!-- Live crafting panel -->
      <LiveCraftingPanel />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useCraftingStore } from "../../stores/craftingStore";
import type { CraftingProject, FlattenedMaterial, IntermediateCraft, MaterialNeed } from "../../types/crafting";
import EmptyState from "../Shared/EmptyState.vue";
import AccordionSection from "../Shared/AccordionSection.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import MaterialSummary from "./MaterialSummary.vue";
import PickupList from "./PickupList.vue";
import ShoppingList from "./ShoppingList.vue";
import LiveCraftingPanel from "./LiveCraftingPanel.vue";
import IntermediateCraftsList from "./IntermediateCraftsList.vue";

const props = defineProps<{
  activeProject: CraftingProject | null
  materials: Map<string, FlattenedMaterial>
  intermediates: IntermediateCraft[]
  materialNeeds: MaterialNeed[]
  resolving: boolean
  checkingAvailability: boolean
}>();

defineEmits<{
  resolve: []
  'check-availability': []
}>();

const craftingStore = useCraftingStore();
const tracker = computed(() => craftingStore.tracker);

const sortedMaterials = computed(() => {
  return Array.from(props.materials.values())
    .sort((a, b) => {
      if (a.is_dynamic !== b.is_dynamic) return a.is_dynamic ? 1 : -1;
      return a.item_name.localeCompare(b.item_name);
    });
});

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
