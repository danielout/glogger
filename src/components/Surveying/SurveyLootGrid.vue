<template>
  <section v-if="rows.length > 0" class="flex flex-col gap-2">
    <div v-if="title" class="flex items-baseline justify-between">
      <h4 class="text-[0.65rem] uppercase tracking-widest font-semibold" :class="titleClass ?? 'text-text-secondary'">
        {{ title }}
      </h4>
      <span v-if="subtitle" class="text-[0.6rem] text-text-dim">{{ subtitle }}</span>
    </div>

    <div class="flex flex-wrap gap-2">
      <div v-for="row in rows" :key="row.item_name" class="flex flex-col">
        <ItemMinicard
          :reference="row.item_name"
          width="max"
        />
        <!-- Count / bonus breakdown / value row, right-aligned under the card -->
        <div class="flex items-center gap-2 px-2 pt-1 text-[0.65rem] tabular-nums">
          <span class="text-text-primary font-semibold">&times;{{ row.total_qty.toLocaleString() }}</span>
          <span
            v-if="row.bonus_qty > 0"
            class="text-text-dim"
            :title="`${row.primary_qty} primary + ${row.bonus_qty} bonus`"
          >
            ({{ row.primary_qty }}<span class="text-text-muted"> + </span><span class="text-accent-gold">{{ row.bonus_qty }}</span>)
          </span>
          <span
            v-if="row.total_value !== null"
            class="text-accent-gold ml-auto"
          >
            {{ formatGold(row.total_value) }}
          </span>
          <span
            v-else-if="showUnpricedHint"
            class="text-text-dim italic ml-auto"
          >
            no price
          </span>
        </div>
      </div>
    </div>
  </section>

  <div v-else-if="emptyMessage" class="text-text-dim italic text-xs">
    {{ emptyMessage }}
  </div>
</template>

<script setup lang="ts">
// Loot summary grid used by the Session, History, and Analytics surfaces.
// Wraps the standard ItemMinicard (icon + name + market/vendor pricing +
// owned count in tooltip) and adds a count/bonus breakdown line below
// each card.
//
// The "77 (68 + 9)" style breakdown only appears when at least one bonus
// drop is recorded for that item — the "+ 9" half is rendered in the
// accent-gold token so the bonus stands out at a glance.
import type { LootSummaryRow } from '../../stores/surveyTrackerStore'
import ItemMinicard from '../Shared/Item/ItemMinicard.vue'
import { formatGold } from '../../composables/useRecipeCost'

defineProps<{
  rows: LootSummaryRow[]
  /** Section heading above the grid. Omit for a bare grid (e.g. embedded in a panel). */
  title?: string
  /** Tailwind class(es) for the title color — lets callers tint the section header. */
  titleClass?: string
  /** Small right-aligned caption next to the title (e.g. "12 unique items"). */
  subtitle?: string
  /** When true, unpriced items show "no price" in-line; otherwise they show nothing. */
  showUnpricedHint?: boolean
  /** Rendered in place of the grid when rows is empty. Omit to render nothing. */
  emptyMessage?: string
}>()
</script>
