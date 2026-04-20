<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Skills</h3>
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
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary" @click="toggleSort('skill_name')">
              Skill {{ sortIcon('skill_name') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary text-right" @click="toggleSort('level')">
              Level {{ sortIcon('level') }}
            </th>
            <th class="py-1.5 px-2 text-right">Bonus</th>
            <th class="py-1.5 px-2 text-right">XP Progress</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="skill in filtered"
            :key="skill.skill_name"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2 text-text-primary">{{ skill.skill_name }}</td>
            <td class="py-1 px-2 text-right text-accent-gold">{{ skill.level }}</td>
            <td class="py-1 px-2 text-right text-text-muted">
              {{ skill.bonus_levels > 0 ? `+${skill.bonus_levels}` : '' }}
            </td>
            <td class="py-1 px-2 text-right text-text-secondary">
              <template v-if="skill.xp_needed_for_next === -1">MAX</template>
              <template v-else>{{ formatNumber(skill.xp_toward_next) }} / {{ formatNumber(skill.xp_needed_for_next) }}</template>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotSkillLevel } from '../../types/database'

const props = defineProps<{
  skills: SnapshotSkillLevel[]
}>()

const filter = ref('')
const sortKey = ref<'skill_name' | 'level'>('skill_name')
const sortAsc = ref(true)

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

function toggleSort(key: 'skill_name' | 'level') {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = key === 'skill_name'
  }
}

function sortIcon(key: string): string {
  if (sortKey.value !== key) return ''
  return sortAsc.value ? '▲' : '▼'
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}
</script>
