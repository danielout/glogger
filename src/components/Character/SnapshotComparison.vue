<template>
  <div class="flex flex-col gap-2">
    <div v-if="diffs.length === 0" class="text-sm text-text-muted p-2">
      Select two snapshots above and click "Compare Selected" to see skill changes.
    </div>

    <div v-else class="overflow-auto max-h-[60vh]">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-base">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1.5 px-2">Skill</th>
            <th class="py-1.5 px-2 text-right">Old Level</th>
            <th class="py-1.5 px-2 text-right">New Level</th>
            <th class="py-1.5 px-2 text-right">Change</th>
            <th class="py-1.5 px-2 text-right">XP Change</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="diff in sortedDiffs"
            :key="diff.skill_name"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2 text-text-primary">{{ diff.skill_name }}</td>
            <td class="py-1 px-2 text-right text-text-muted">{{ diff.old_level }}</td>
            <td class="py-1 px-2 text-right text-text-primary">{{ diff.new_level }}</td>
            <td class="py-1 px-2 text-right" :class="changeColor(diff.level_change)">
              {{ diff.level_change > 0 ? `+${diff.level_change}` : diff.level_change || '-' }}
            </td>
            <td class="py-1 px-2 text-right text-text-secondary">
              {{ formatXpChange(diff) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SkillDiff } from '../../types/database'

const props = defineProps<{
  diffs: SkillDiff[]
}>()

const sortedDiffs = computed(() => {
  return [...props.diffs].sort((a, b) => {
    // Changed skills first, then alphabetical
    const aChanged = a.level_change !== 0 || a.old_xp !== a.new_xp
    const bChanged = b.level_change !== 0 || b.old_xp !== b.new_xp
    if (aChanged !== bChanged) return aChanged ? -1 : 1
    return a.skill_name.localeCompare(b.skill_name)
  })
})

function changeColor(change: number): string {
  if (change > 0) return 'text-value-positive'
  if (change < 0) return 'text-value-negative'
  return 'text-text-muted'
}

function formatXpChange(diff: SkillDiff): string {
  const change = diff.new_xp - diff.old_xp
  if (change === 0 && diff.level_change === 0) return '-'
  if (change > 0) return `+${change.toLocaleString()}`
  return change.toLocaleString()
}
</script>
