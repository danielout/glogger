<template>
  <AccordionSection :default-open="false">
    <template #title>{{ title }}</template>
    <template #badge>
      <span class="text-text-muted text-xs">{{ count }} rows</span>
    </template>
    <div v-if="count === 0" class="text-text-muted text-xs italic">No data</div>
    <div v-else class="overflow-x-auto max-h-80">
      <DataTable
        :columns="columnDefs"
        :rows="formattedRows"
        :hoverable="true"
        :sticky-header="true"
        compact
        empty-text="No data" />
    </div>
  </AccordionSection>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import AccordionSection from "../Shared/AccordionSection.vue"
import DataTable, { type ColumnDef } from '../Shared/DataTable.vue'

const props = defineProps<{
  title: string
  count: number
  columns: string[]
  rows: Record<string, any>[]
}>()

function formatCell(value: unknown): string {
  if (value === null || value === undefined) return '-'
  if (typeof value === 'boolean') return value ? 'true' : 'false'
  return String(value)
}

const columnDefs = computed<ColumnDef[]>(() =>
  props.columns.map(col => ({ key: col, label: col }))
)

const formattedRows = computed(() =>
  props.rows.map(row => {
    const formatted: Record<string, unknown> = {}
    for (const col of props.columns) {
      formatted[col] = formatCell(row[col])
    }
    return formatted
  })
)
</script>
