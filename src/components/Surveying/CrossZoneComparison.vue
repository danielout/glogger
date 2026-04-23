<template>
  <div class="flex flex-col gap-3">
    <div v-if="zones.length > 1" class="flex items-center gap-2 text-xs">
      <span class="text-[10px] uppercase tracking-wide text-text-secondary font-semibold">Sort by</span>
      <div class="flex gap-0.5 bg-surface-elevated border border-border-default rounded p-0.5">
        <button
          v-for="col in sortColumns"
          :key="col.key"
          class="px-2 py-0.5 text-xs rounded transition-colors"
          :class="
            sortBy === col.key
              ? 'bg-accent-gold/20 text-accent-gold font-semibold'
              : 'text-text-secondary hover:text-text-primary'
          "
          @click="toggleSort(col.key)"
        >
          {{ col.label }}
          <span v-if="sortBy === col.key">{{ sortAsc ? '↑' : '↓' }}</span>
        </button>
      </div>
    </div>

    <table class="w-full text-xs border-collapse">
      <thead>
        <tr class="text-[10px] uppercase tracking-wide text-text-secondary font-semibold border-b border-border-default">
          <th class="text-left py-1.5 px-2">Zone</th>
          <th class="text-right py-1.5 px-2">Surveys</th>
          <th class="text-right py-1.5 px-2">Basic</th>
          <th class="text-right py-1.5 px-2">Motherlode</th>
          <th class="text-right py-1.5 px-2">Multihit</th>
          <th class="text-right py-1.5 px-2">Loot</th>
          <th class="text-right py-1.5 px-2">Bonus Rate</th>
          <th class="text-right py-1.5 px-2">Bonus Items</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="z in sortedZones"
          :key="z.area"
          class="border-b border-border-default/40 hover:bg-surface-elevated/50 cursor-pointer"
          @click="$emit('select-zone', z.area)"
        >
          <td class="py-1 px-2 text-text-primary font-semibold">
            <AreaInline :reference="z.area" />
          </td>
          <td class="text-right py-1 px-2 text-text-primary tabular-nums">{{ z.total_uses }}</td>
          <td class="text-right py-1 px-2 text-text-secondary tabular-nums">{{ z.basic_uses }}</td>
          <td class="text-right py-1 px-2 text-text-secondary tabular-nums">{{ z.motherlode_uses }}</td>
          <td class="text-right py-1 px-2 text-text-secondary tabular-nums">{{ z.multihit_uses }}</td>
          <td class="text-right py-1 px-2 text-accent-gold font-semibold tabular-nums">
            {{ z.total_loot_qty.toLocaleString() }}
          </td>
          <td class="text-right py-1 px-2 tabular-nums" :class="bonusRateClass(z)">
            {{ bonusRate(z) }}
          </td>
          <td class="text-right py-1 px-2 text-text-secondary tabular-nums">
            {{ z.bonus_items_total.toLocaleString() }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
// Cross-zone comparison table used on the Analytics Overview view.
// Each row is clickable to drill into that zone's detail page.
import { computed, ref } from 'vue'
import type { ZoneSummary } from '../../stores/surveyTrackerStore'
import AreaInline from '../Shared/Area/AreaInline.vue'

const props = defineProps<{
  zones: ZoneSummary[]
}>()

defineEmits<{
  'select-zone': [area: string]
}>()

type SortKey = 'total_uses' | 'basic_uses' | 'total_loot_qty' | 'bonus_rate' | 'bonus_items_total'

const sortColumns: { key: SortKey; label: string }[] = [
  { key: 'total_uses', label: 'Surveys' },
  { key: 'total_loot_qty', label: 'Loot' },
  { key: 'bonus_rate', label: 'Bonus Rate' },
  { key: 'bonus_items_total', label: 'Bonus Items' },
]

const sortBy = ref<SortKey>('total_uses')
const sortAsc = ref(false)

function toggleSort(key: SortKey) {
  if (sortBy.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortBy.value = key
    sortAsc.value = false
  }
}

function zoneValue(z: ZoneSummary, key: SortKey): number {
  if (key === 'bonus_rate') {
    return z.basic_uses > 0 ? z.basic_uses_with_bonus / z.basic_uses : 0
  }
  return z[key]
}

const sortedZones = computed(() => {
  const list = [...props.zones]
  list.sort((a, b) => {
    const av = zoneValue(a, sortBy.value)
    const bv = zoneValue(b, sortBy.value)
    return sortAsc.value ? av - bv : bv - av
  })
  return list
})

function bonusRate(z: ZoneSummary): string {
  if (z.basic_uses === 0) return '—'
  const pct = (z.basic_uses_with_bonus / z.basic_uses) * 100
  return `${pct.toFixed(1)}%`
}

function bonusRateClass(z: ZoneSummary): string {
  if (z.basic_uses === 0) return 'text-text-dim'
  return 'text-accent-gold'
}
</script>
