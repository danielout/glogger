<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Skills</h3>
      <FilterBar v-model="filter" placeholder="Filter..." :result-count="filtered.length" result-label="" />
    </div>

    <div class="overflow-auto flex-1 min-h-0">
      <DataTable
        :columns="columns"
        :rows="(filtered as unknown as Record<string, unknown>[])"
        :sort-key="sortKey"
        :sort-dir="sortAsc ? 'asc' : 'desc'"
        compact
        empty-text="No skills"
        @sort="onSort">
        <template #cell-skill_name="{ row }">
          <span class="text-text-primary">{{ row.skill_name }}</span>
        </template>
        <template #cell-level="{ row }">
          <span class="text-accent-gold">{{ row.level }}</span>
        </template>
        <template #cell-bonus_levels="{ row }">
          <span class="text-text-muted">{{ (row.bonus_levels as number) > 0 ? `+${row.bonus_levels}` : '' }}</span>
        </template>
        <template #cell-xp_progress="{ row }">
          <span class="text-text-secondary">
            <template v-if="(row.xp_needed_for_next as number) === -1">MAX</template>
            <template v-else>{{ formatNumber(row.xp_toward_next as number) }} / {{ formatNumber(row.xp_needed_for_next as number) }}</template>
          </span>
        </template>
      </DataTable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotSkillLevel } from '../../types/database'
import DataTable from '../Shared/DataTable.vue'
import FilterBar from '../Shared/FilterBar.vue'

const props = defineProps<{
  skills: SnapshotSkillLevel[]
}>()

const filter = ref('')
const sortKey = ref<string>('skill_name')
const sortAsc = ref(true)

const columns = [
  { key: 'skill_name', label: 'Skill', sortable: true },
  { key: 'level', label: 'Level', sortable: true, align: 'right' as const, numeric: true },
  { key: 'bonus_levels', label: 'Bonus', align: 'right' as const, numeric: true },
  { key: 'xp_progress', label: 'XP Progress', align: 'right' as const, numeric: true },
]

const filtered = computed(() => {
  const f = filter.value.toLowerCase()
  const list = f
    ? props.skills.filter(s => s.skill_name.toLowerCase().includes(f))
    : [...props.skills]

  list.sort((a, b) => {
    const dir = sortAsc.value ? 1 : -1
    if (sortKey.value === 'level') return (a.level - b.level) * dir
    return a.skill_name.localeCompare(b.skill_name) * dir
  })

  return list
})

function onSort(payload: { key: string; dir: 'asc' | 'desc' }) {
  sortKey.value = payload.key
  sortAsc.value = payload.dir === 'asc'
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}
</script>
