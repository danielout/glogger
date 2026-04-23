<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Combat Stats</h3>
      <div class="flex items-center gap-3">
        <input
          v-model="filter"
          type="text"
          placeholder="Filter..."
          class="px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-32" />
        <span class="text-xs text-text-muted">{{ filtered.length }}</span>
      </div>
    </div>

    <div class="overflow-auto flex-1 min-h-0">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-base">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1.5 px-2">Stat</th>
            <th class="py-1.5 px-2 text-right">Value</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="stat in filtered"
            :key="stat.stat_key"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2 text-text-primary">{{ formatStatKey(stat.stat_key) }}</td>
            <td class="py-1 px-2 text-right text-accent-gold tabular-nums">{{ formatValue(stat.value) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotStat } from '../../types/database'

const props = defineProps<{
  stats: SnapshotStat[]
}>()

const filter = ref('')

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
