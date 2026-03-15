<template>
  <div class="overflow-auto max-h-[60vh]">
    <table class="w-full text-sm border-collapse">
      <thead class="sticky top-0 bg-surface-base">
        <tr class="text-left text-text-secondary border-b border-border-default">
          <th class="py-1.5 px-2">Currency</th>
          <th class="py-1.5 px-2 text-right">Amount</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="c in currencies"
          :key="c.currency_key"
          class="border-b border-border-default/50 hover:bg-surface-elevated/50">
          <td class="py-1 px-2 text-text-primary">{{ formatKey(c.currency_key) }}</td>
          <td class="py-1 px-2 text-right text-accent-gold">{{ c.amount.toLocaleString() }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { SnapshotCurrency } from '../../types/database'

defineProps<{
  currencies: SnapshotCurrency[]
}>()

function formatKey(key: string): string {
  return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}
</script>
