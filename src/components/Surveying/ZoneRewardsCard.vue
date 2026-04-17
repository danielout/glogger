<template>
  <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2">
    <header class="flex items-baseline gap-2">
      <h4 class="text-xs font-semibold" :class="categoryTextClass">
        {{ mapDisplayName }}
      </h4>
      <span class="text-[0.6rem] text-text-dim">
        {{ totalCompletions }} completed
        <template v-if="(craftingCost ?? 0) > 0">
          · cost {{ formatGold(craftingCost ?? 0) }}
        </template>
      </span>
    </header>

    <table v-if="items.length > 0" class="w-full text-xs border-collapse">
      <thead>
        <tr class="text-[0.6rem] uppercase tracking-wide text-text-secondary font-semibold border-b border-border-default">
          <th class="text-left py-1 px-2">Item</th>
          <th class="text-right py-1 px-2">Total</th>
          <th class="text-right py-1 px-2">Primary</th>
          <th class="text-right py-1 px-2">Bonus</th>
          <th class="text-right py-1 px-2">Drops</th>
          <th class="text-right py-1 px-2">Avg/Drop</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="item in items"
          :key="item.item_name"
          class="border-b border-border-default/40 hover:bg-surface-elevated/50"
        >
          <td class="py-1 px-2">
            <ItemInline :reference="item.item_name" />
          </td>
          <td class="text-right py-1 px-2 text-accent-gold font-semibold tabular-nums">
            {{ item.total_qty.toLocaleString() }}
          </td>
          <td class="text-right py-1 px-2 text-text-primary tabular-nums">
            {{ item.primary_qty.toLocaleString() }}
          </td>
          <td
            class="text-right py-1 px-2 tabular-nums"
            :class="item.bonus_qty > 0 ? 'text-accent-gold' : 'text-text-dim'"
          >
            {{ item.bonus_qty.toLocaleString() }}
          </td>
          <td class="text-right py-1 px-2 text-text-secondary tabular-nums">
            {{ item.times_received.toLocaleString() }}
          </td>
          <td class="text-right py-1 px-2 text-text-muted tabular-nums">
            {{ item.times_received > 0 ? (item.total_qty / item.times_received).toFixed(1) : '—' }}
          </td>
        </tr>
      </tbody>
    </table>

    <div v-else class="text-text-dim italic text-xs">
      No loot data recorded.
    </div>
  </section>
</template>

<script setup lang="ts">
// Per-survey-type or per-zone card listing item drops attributed to this
// scope. Used as the building block for both ZoneView (one card per
// survey type the zone contains) and TypeView (one card for the chosen
// type). Takes a pre-resolved items list so the caller owns the group-by
// logic — this component just renders.
import { computed } from 'vue'
import type { ItemSummary } from '../../stores/surveyTrackerStore'
import ItemInline from '../Shared/Item/ItemInline.vue'
import { formatGold } from '../../composables/useRecipeCost'

const props = defineProps<{
  mapDisplayName: string
  /** `'mineral'` / `'mining'` — tints the header. */
  category?: string
  totalCompletions: number
  craftingCost?: number
  items: ItemSummary[]
}>()

const categoryTextClass = computed(() => {
  if (props.category === 'mineral') return 'text-accent-blue'
  if (props.category === 'mining') return 'text-accent-red'
  return 'text-text-primary'
})
</script>
