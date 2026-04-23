<template>
  <AccordionSection :default-open="false">
    <template #title>{{ title }}</template>
    <template #badge>
      <span class="text-text-muted text-xs">{{ count }} rows</span>
    </template>
    <div v-if="count === 0" class="text-text-muted text-xs italic">No data</div>
    <div v-else class="overflow-x-auto max-h-80 overflow-y-auto">
      <table class="w-full text-xs border-collapse">
        <thead class="sticky top-0 bg-surface-base">
          <tr class="text-text-muted text-left">
            <th
              v-for="col in columns"
              :key="col"
              class="px-2 py-1 border-b border-border-default whitespace-nowrap">
              {{ col }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, i) in rows" :key="i" class="hover:bg-surface-elevated/30">
            <td
              v-for="col in columns"
              :key="col"
              class="px-2 py-0.5 whitespace-nowrap"
              :class="col === 'last_confirmed_at' || col === 'source' ? 'text-text-muted' : 'text-text-primary'">
              {{ formatCell(row[col]) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </AccordionSection>
</template>

<script setup lang="ts">
import AccordionSection from "../Shared/AccordionSection.vue";

defineProps<{
  title: string
  count: number
  columns: string[]
  rows: Record<string, any>[]
}>();

function formatCell(value: unknown): string {
  if (value === null || value === undefined) return '-'
  if (typeof value === 'boolean') return value ? 'true' : 'false'
  return String(value)
}
</script>
