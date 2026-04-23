<template>
  <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2 h-full overflow-hidden">
    <header class="flex items-baseline justify-between shrink-0">
      <h3 class="text-[10px] uppercase tracking-widest text-text-secondary font-semibold">
        Loot Found
      </h3>
      <span class="text-[10px] text-text-dim">{{ rows.length }} unique</span>
    </header>

    <div v-if="rows.length === 0" class="text-text-dim text-xs italic py-2">
      No loot attributed yet.
    </div>

    <div v-else class="flex-1 min-h-0 overflow-y-auto -mr-1 pr-1">
      <table class="w-full text-xs border-collapse">
        <thead class="sticky top-0 bg-surface-card">
          <tr class="text-[10px] uppercase tracking-wide text-text-secondary font-semibold border-b border-border-default">
            <th class="text-left py-1 px-1">Item</th>
            <th class="text-right py-1 px-1">Qty</th>
            <th v-if="hasBonus" class="text-right py-1 px-1">Split</th>
            <th class="text-right py-1 px-1">Value</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="row in rows"
            :key="row.item_name"
            class="border-b border-border-default/40 hover:bg-surface-elevated/50"
          >
            <td class="py-1 px-1">
              <ItemMinicard :reference="row.item_name" width="min" :bordered="false" />
            </td>
            <td class="text-right py-1 px-1 text-accent-gold font-semibold tabular-nums align-top pt-2">
              {{ row.total_qty.toLocaleString() }}
            </td>
            <td v-if="hasBonus" class="text-right py-1 px-1 tabular-nums align-top pt-2">
              <span v-if="row.bonus_qty > 0" class="text-text-dim">
                {{ row.primary_qty }}<span class="text-text-muted"> + </span><span class="text-accent-gold">{{ row.bonus_qty }}</span>
              </span>
              <span v-else class="text-text-dim">—</span>
            </td>
            <td class="text-right py-1 px-1 tabular-nums align-top pt-2" :class="row.unit_value !== null ? 'text-text-secondary' : 'text-text-dim italic'">
              {{ row.total_value !== null ? formatGold(row.total_value) : '—' }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
// Loot overview table for the center panel. Shows each item as an
// ItemMinicard row (with inline market-value editing via the ??? button)
// plus quantity, primary/bonus split, and total value.
import { computed } from 'vue'
import type { LootSummaryRow } from '../../stores/surveyTrackerStore'
import ItemMinicard from '../Shared/Item/ItemMinicard.vue'
import { formatGold } from '../../composables/useRecipeCost'

const props = defineProps<{
  rows: LootSummaryRow[]
}>()

const hasBonus = computed(() => props.rows.some(r => r.bonus_qty > 0))
</script>
