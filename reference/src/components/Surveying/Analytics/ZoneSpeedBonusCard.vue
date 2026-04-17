<template>
  <div class="bg-surface-card border border-border-default rounded p-3">
    <div class="flex items-center gap-2 mb-2">
      <span class="text-[0.65rem] uppercase tracking-widest font-bold"
            :class="category.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
        {{ category.category === 'mineral' ? 'Mineral' : 'Mining' }} Speed Bonus Items
      </span>
      <span class="text-[0.6rem] text-text-dim">
        {{ category.speed_bonus_count }}/{{ category.total_surveys }} surveys
        ({{ category.speed_bonus_rate.toFixed(1) }}%)
      </span>
      <span v-if="category.avg_bonus_value > 0" class="text-[0.6rem] text-[#c8b47e]">
        avg {{ formatGold(category.avg_bonus_value) }}/proc
      </span>
    </div>

    <table v-if="category.item_stats.length > 0" class="text-xs w-full">
      <thead>
        <tr class="text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
          <th class="text-left py-0.5 px-2 font-bold">Item</th>
          <th class="text-right py-0.5 px-2 font-bold">Total</th>
          <th class="text-right py-0.5 px-2 font-bold">Seen</th>
          <th class="text-right py-0.5 px-2 font-bold">Min</th>
          <th class="text-right py-0.5 px-2 font-bold">Max</th>
          <th class="text-right py-0.5 px-2 font-bold">Avg</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="item in category.item_stats"
          :key="item.item_name"
          class="bg-black/10 border-b border-border-default hover:bg-black/20"
        >
          <td class="py-0.5 px-2"><ItemInline :reference="item.item_name" /></td>
          <td class="text-right py-0.5 px-2 font-mono text-[#c8b47e]">{{ item.total_quantity }}</td>
          <td class="text-right py-0.5 px-2 font-mono text-text-secondary">{{ item.times_seen }}/{{ item.total_procs }}</td>
          <td class="text-right py-0.5 px-2 font-mono">{{ item.min_per_proc }}</td>
          <td class="text-right py-0.5 px-2 font-mono">{{ item.max_per_proc }}</td>
          <td class="text-right py-0.5 px-2 font-mono text-text-primary">{{ item.avg_per_proc.toFixed(1) }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else class="text-text-dim italic text-xs">No speed bonus data recorded.</div>
  </div>
</template>

<script setup lang="ts">
import type { CategorySpeedBonusStats } from "../../../types/database";
import ItemInline from "../../Shared/Item/ItemInline.vue";

defineProps<{
  category: CategorySpeedBonusStats;
}>();

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) return rounded.toLocaleString() + "g";
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}
</script>
