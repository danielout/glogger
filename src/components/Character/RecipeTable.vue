<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center gap-3">
      <input
        v-model="filter"
        type="text"
        placeholder="Filter recipes..."
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-48" />
      <span class="text-xs text-text-muted">{{ filtered.length }} recipes</span>
    </div>

    <div class="overflow-auto max-h-[60vh]">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-base">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary" @click="toggleSort('recipe_key')">
              Recipe {{ sortIcon('recipe_key') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary text-right" @click="toggleSort('completions')">
              Completions {{ sortIcon('completions') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="recipe in filtered"
            :key="recipe.recipe_key"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2 text-text-primary">{{ recipe.recipe_key }}</td>
            <td class="py-1 px-2 text-right text-text-secondary">{{ recipe.completions.toLocaleString() }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotRecipeCompletion } from '../../types/database'

const props = defineProps<{
  recipes: SnapshotRecipeCompletion[]
}>()

const filter = ref('')
const sortKey = ref<'recipe_key' | 'completions'>('recipe_key')
const sortAsc = ref(true)

const filtered = computed(() => {
  const f = filter.value.toLowerCase()
  const list = f
    ? props.recipes.filter(r => r.recipe_key.toLowerCase().includes(f))
    : [...props.recipes]

  list.sort((a, b) => {
    const dir = sortAsc.value ? 1 : -1
    if (sortKey.value === 'completions') return (a.completions - b.completions) * dir
    return a.recipe_key.localeCompare(b.recipe_key) * dir
  })

  return list
})

function toggleSort(key: 'recipe_key' | 'completions') {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = key === 'recipe_key'
  }
}

function sortIcon(key: string): string {
  if (sortKey.value !== key) return ''
  return sortAsc.value ? '▲' : '▼'
}
</script>
