<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">
        Material Availability
      </h4>
      <div class="flex gap-3 text-[0.65rem] text-text-muted">
        <span>
          <span class="text-green-400">{{ coveredCount }}</span> covered
        </span>
        <span>
          <span class="text-yellow-400">{{ partialCount }}</span> partial
        </span>
        <span>
          <span class="text-accent-red">{{ missingCount }}</span> missing
        </span>
      </div>
    </div>

    <table class="w-full text-xs">
      <thead>
        <tr class="text-text-dim border-b border-border-light">
          <th class="text-left py-1 font-medium">Item</th>
          <th class="text-right py-1 font-medium w-16">Need</th>
          <th class="text-right py-1 font-medium w-16">Inv</th>
          <th class="text-right py-1 font-medium w-16">Storage</th>
          <th class="text-right py-1 font-medium w-20">Shortfall</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="mat in needs"
          :key="mat.item_id"
          class="border-b border-surface-dark"
          :class="rowClass(mat)">
          <td class="py-1">
            <ItemInline :name="mat.item_name" />
          </td>
          <td class="text-right py-1 font-mono text-text-primary">{{ mat.quantity_needed }}</td>
          <td class="text-right py-1 font-mono" :class="mat.inventory_have > 0 ? 'text-green-400' : 'text-text-muted'">
            {{ mat.inventory_have }}
          </td>
          <td class="text-right py-1 font-mono" :class="mat.storage_have > 0 ? 'text-green-400' : 'text-text-muted'">
            {{ mat.storage_have }}
          </td>
          <td class="text-right py-1 font-mono font-semibold" :class="mat.shortfall > 0 ? 'text-accent-red' : 'text-green-400'">
            {{ mat.shortfall > 0 ? mat.shortfall : '✓' }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { MaterialNeed } from "../../types/crafting";
import ItemInline from "../Shared/Item/ItemInline.vue";

const props = defineProps<{
  needs: MaterialNeed[]
}>();

const coveredCount = computed(() => props.needs.filter((m) => m.shortfall === 0).length);
const partialCount = computed(() =>
  props.needs.filter((m) => m.shortfall > 0 && (m.inventory_have > 0 || m.storage_have > 0)).length
);
const missingCount = computed(() =>
  props.needs.filter((m) => m.shortfall > 0 && m.inventory_have === 0 && m.storage_have === 0).length
);

function rowClass(mat: MaterialNeed): string {
  if (mat.shortfall === 0) return "opacity-60";
  return "";
}
</script>
