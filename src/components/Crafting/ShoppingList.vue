<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">
        Shopping List
      </h4>
      <span v-if="totalCost > 0" class="text-text-muted text-[0.65rem]">
        Est. vendor cost: <span class="text-accent-red font-semibold">{{ totalCost.toLocaleString() }}g</span>
      </span>
    </div>

    <div v-if="shoppingItems.length === 0" class="text-text-dim text-xs italic">
      You have everything you need!
    </div>

    <table v-else class="w-full text-xs">
      <thead>
        <tr class="text-text-dim border-b border-border-light">
          <th class="text-left py-1 font-medium">Item</th>
          <th class="text-right py-1 font-medium w-16">Need</th>
          <th class="text-right py-1 font-medium w-24">Est. Cost</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="item in shoppingItems"
          :key="item.item_id"
          class="border-b border-surface-dark">
          <td class="py-1">
            <ItemInline :name="item.item_name" />
          </td>
          <td class="text-right py-1 font-mono text-text-primary">{{ item.shortfall }}</td>
          <td class="text-right py-1 font-mono" :class="item.cost ? 'text-accent-red' : 'text-text-muted'">
            {{ item.cost ? `${item.cost.toLocaleString()}g` : '—' }}
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

interface ShoppingItem {
  item_id: number
  item_name: string
  shortfall: number
  cost: number | null
}

const shoppingItems = computed((): ShoppingItem[] => {
  return props.needs
    .filter((m) => m.shortfall > 0)
    .map((m) => ({
      item_id: m.item_id,
      item_name: m.item_name,
      shortfall: m.shortfall,
      cost: m.vendor_price ? Math.round(m.vendor_price * m.shortfall) : null,
    }))
    .sort((a, b) => a.item_name.localeCompare(b.item_name));
});

const totalCost = computed(() => {
  return shoppingItems.value.reduce((sum, item) => sum + (item.cost ?? 0), 0);
});
</script>
