<template>
  <div class="flex flex-col h-full">
    <EmptyState
      v-if="entries.length === 0"
      variant="compact"
      :primary="emptyText"
      :secondary="emptyHint" />

    <div v-else class="flex flex-col gap-0.5 overflow-y-auto max-h-52 pr-1">
      <div
        v-for="entry in entries"
        :key="`${entry.timestamp}-${entry.label}-${entry.amount}-${entry.detail}`"
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
    <div v-if="entries.length > 0" class="mt-auto pt-2 border-t border-border-default text-xs text-text-muted flex items-center justify-between">
      <span class="flex items-center gap-1.5">
        {{ entries.length }} {{ entries.length === 1 ? 'entry' : 'entries' }}
        <span
          v-if="warningTooltip"
          class="relative group cursor-help text-yellow-500/70 hover:text-yellow-400">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-3 h-3">
            <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.168-.168 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.457-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
          </svg>
          <span class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-64 p-2 rounded bg-surface-elevated border border-border-default text-xs text-text-secondary leading-snug opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto transition-opacity z-50 shadow-lg">
            {{ warningTooltip }}
          </span>
        </span>
      </span>
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
  entries: ActivityEntry[]
  dotColor?: string
  emptyText?: string
  emptyHint?: string
  showItemLinks?: boolean
  showNpcLinks?: boolean
  unit?: string
  signedTotal?: boolean
  /** Use '×' prefix instead of '+' for amounts (e.g. outgoing items) */
  quantityPrefix?: boolean
  /** Optional warning text shown as a hoverable icon next to the title */
  warningTooltip?: string
}>(), {
  dotColor: 'bg-gray-500',
  emptyText: 'No activity yet.',
  emptyHint: 'Events will appear here as they happen.',
  showItemLinks: false,
  showNpcLinks: false,
  unit: '',
  signedTotal: false,
  quantityPrefix: false,
  warningTooltip: undefined,
})

const total = computed(() =>
  props.entries.reduce((sum, e) => sum + e.amount, 0)
)

function formatTs(ts: string): string {
  return formatAnyTimestamp(ts)
}

function formatAmount(amount: number): string {
  if (props.quantityPrefix) return `×${amount.toLocaleString()}`
  if (amount >= 0) return `+${amount.toLocaleString()}`
  return amount.toLocaleString()
}

function formatSigned(amount: number): string {
  if (amount > 0) return `+${amount.toLocaleString()}`
  return amount.toLocaleString()
}

function amountClass(amount: number): string {
  if (props.quantityPrefix) return 'text-text-secondary'
  if (amount > 0) return 'text-value-positive'
  if (amount < 0) return 'text-value-negative'
  return 'text-text-muted'
}
</script>
