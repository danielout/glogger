<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider shrink-0">Currencies</h3>
    <div class="overflow-auto flex-1 min-h-0">
      <DataTable
        :columns="columns"
        :rows="(currencies as unknown as Record<string, unknown>[])"
        compact
        empty-text="No currencies">
        <template #cell-currency_key="{ row }">
          <span class="text-text-primary">{{ formatKey(row.currency_key as string) }}</span>
        </template>
        <template #cell-amount="{ row }">
          <span class="text-accent-gold">{{ (row.amount as number).toLocaleString() }}</span>
        </template>
      </DataTable>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SnapshotCurrency } from '../../types/database'
import DataTable from '../Shared/DataTable.vue'

defineProps<{
  currencies: SnapshotCurrency[]
}>()

const columns = [
  { key: 'currency_key', label: 'Currency' },
  { key: 'amount', label: 'Amount', align: 'right' as const, numeric: true },
]

function formatKey(key: string): string {
  return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}
</script>
