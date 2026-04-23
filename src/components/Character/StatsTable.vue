<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Combat Stats</h3>
      <FilterBar v-model="filter" placeholder="Filter..." :result-count="filtered.length" result-label="" />
    </div>

    <div class="overflow-auto flex-1 min-h-0">
      <DataTable
        :columns="columns"
        :rows="(filtered as unknown as Record<string, unknown>[])"
        compact
        empty-text="No stats">
        <template #cell-stat_key="{ row }">
          <span class="text-text-primary">{{ formatStatKey(row.stat_key as string) }}</span>
        </template>
        <template #cell-value="{ row }">
          <span class="text-accent-gold">{{ formatValue(row.value as number) }}</span>
        </template>
      </DataTable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotStat } from '../../types/database'
import DataTable from '../Shared/DataTable.vue'
import FilterBar from '../Shared/FilterBar.vue'

const props = defineProps<{
  stats: SnapshotStat[]
}>()

const filter = ref('')

const columns = [
  { key: 'stat_key', label: 'Stat' },
  { key: 'value', label: 'Value', align: 'right' as const, numeric: true },
]

const filtered = computed(() => {
  const f = filter.value.toLowerCase()
  return f
    ? props.stats.filter(s => s.stat_key.toLowerCase().includes(f))
    : props.stats
})

function formatStatKey(key: string): string {
  return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function formatValue(v: number): string {
  return Number.isInteger(v) ? v.toLocaleString() : v.toFixed(2)
}
</script>
