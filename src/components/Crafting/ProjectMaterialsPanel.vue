<template>
  <div class="shrink-0 overflow-y-auto border border-surface-elevated rounded p-4 flex flex-col gap-4">
    <template v-if="!activeProject">
      <div class="text-text-dim text-xs italic text-center py-8">
        Select a project to see materials
      </div>
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
      <IntermediateCraftsList :intermediates="intermediates" />

      <!-- Materials table -->
      <div v-if="materials.size > 0">
        <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide mb-2">
          Total Materials Needed
        </h4>
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
                <ItemInline v-else-if="mat.item_id !== null" :name="mat.item_name" />
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
      </div>

      <!-- Material availability breakdown -->
      <MaterialSummary v-if="materialNeeds.length > 0" :needs="materialNeeds" />

      <!-- Pickup list -->
      <PickupList v-if="materialNeeds.length > 0" :needs="materialNeeds" />

      <!-- Shopping list -->
      <ShoppingList v-if="materialNeeds.length > 0" :needs="materialNeeds" />

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
      // Dynamic ingredients last
      if (a.is_dynamic !== b.is_dynamic) return a.is_dynamic ? 1 : -1;
      return a.item_name.localeCompare(b.item_name);
    });
});
</script>
