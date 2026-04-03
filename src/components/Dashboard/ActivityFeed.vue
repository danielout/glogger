<template>
  <div class="flex flex-col h-full">
    <h3 class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-2">{{ title }}</h3>

    <EmptyState
      v-if="entries.length === 0"
      variant="compact"
      :primary="emptyText"
      :secondary="emptyHint" />

    <div v-else class="flex flex-col gap-0.5 overflow-y-auto max-h-52 pr-1">
      <div
        v-for="(entry, i) in entries"
        :key="`${entry.timestamp}-${entry.label}-${i}`"
        class="flex items-center gap-2 py-1 px-2 rounded text-xs hover:bg-surface-elevated/50">
        <!-- Color indicator -->
        <span class="w-1.5 h-1.5 rounded-full shrink-0" :class="dotColor" />

        <!-- Timestamp -->
        <span class="text-text-dim font-mono shrink-0">{{ formatTs(entry.timestamp) }}</span>

        <!-- Label (item name, NPC name, etc.) -->
        <ItemInline v-if="showItemLinks" :reference="entry.label" />
        <NpcInline v-else-if="showNpcLinks" :reference="entry.label" />
        <span v-else class="text-text-primary truncate">{{ entry.label }}</span>

        <!-- Amount -->
        <span class="ml-auto shrink-0 font-mono" :class="amountClass(entry.amount)">
          {{ formatAmount(entry.amount) }}
        </span>

        <!-- Detail context -->
        <span v-if="entry.detail" class="text-text-dim shrink-0">{{ entry.detail }}</span>
      </div>
    </div>

    <!-- Summary line -->
    <div v-if="entries.length > 0" class="mt-auto pt-2 border-t border-border-default text-xs text-text-muted flex justify-between">
      <span>{{ entries.length }} {{ entries.length === 1 ? 'entry' : 'entries' }}</span>
      <span class="font-mono" :class="amountClass(total)">
        {{ signedTotal ? formatSigned(total) : total.toLocaleString() }} {{ unit }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { ActivityEntry } from '../../stores/gameStateStore'
import { formatAnyTimestamp } from '../../composables/useTimestamp'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import NpcInline from '../Shared/NPC/NpcInline.vue'

const props = withDefaults(defineProps<{
  title: string
  entries: ActivityEntry[]
  dotColor?: string
  emptyText?: string
  emptyHint?: string
  showItemLinks?: boolean
  showNpcLinks?: boolean
  unit?: string
  signedTotal?: boolean
}>(), {
  dotColor: 'bg-gray-500',
  emptyText: 'No activity yet.',
  emptyHint: 'Events will appear here as they happen.',
  showItemLinks: false,
  showNpcLinks: false,
  unit: '',
  signedTotal: false,
})

const total = computed(() =>
  props.entries.reduce((sum, e) => sum + e.amount, 0)
)

function formatTs(ts: string): string {
  return formatAnyTimestamp(ts)
}

function formatAmount(amount: number): string {
  if (amount >= 0) return `+${amount.toLocaleString()}`
  return amount.toLocaleString()
}

function formatSigned(amount: number): string {
  if (amount > 0) return `+${amount.toLocaleString()}`
  return amount.toLocaleString()
}

function amountClass(amount: number): string {
  if (amount > 0) return 'text-green-400'
  if (amount < 0) return 'text-red-400'
  return 'text-text-muted'
}
</script>
