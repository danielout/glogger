<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- Not tailing warning -->
    <div v-if="!coordinator.isPlayerLogTailing"
      class="p-3 bg-yellow-900/20 border border-yellow-700/40 rounded text-sm text-yellow-300">
      Player log tailing is not active. Start tailing to track your inventory in real-time.
    </div>

    <!-- Summary bar -->
    <div class="flex gap-6 text-sm items-center">
      <div class="flex gap-1.5 items-baseline">
        <span class="text-text-muted">Items:</span>
        <span class="text-text-primary font-medium">{{ store.liveItemCount.toLocaleString() }}</span>
      </div>
      <div class="flex gap-1.5 items-baseline">
        <span class="text-text-muted">Total Qty:</span>
        <span class="text-text-primary font-medium">{{ store.liveTotalStacks.toLocaleString() }}</span>
      </div>
      <div v-if="coordinator.isPlayerLogTailing" class="flex gap-1.5 items-center">
        <span class="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
        <span class="text-text-muted text-xs">Live</span>
      </div>
    </div>

    <!-- Search -->
    <div class="flex items-center gap-3">
      <div class="relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search items..."
          class="pl-7 pr-7 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted w-48 focus:outline-none focus:border-accent-gold/50"
        />
        <span class="absolute left-2 top-1/2 -translate-y-1/2 text-text-muted text-xs pointer-events-none">&#x1F50D;</span>
        <button
          v-if="searchQuery"
          class="absolute right-1.5 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary text-xs cursor-pointer"
          @click="searchQuery = ''"
        >&times;</button>
      </div>
      <span v-if="searchQuery" class="text-xs text-text-muted">
        {{ filteredItems.length }} match{{ filteredItems.length !== 1 ? 'es' : '' }}
      </span>
    </div>

    <!-- Empty state -->
    <EmptyState
      v-if="!store.isLivePopulated && coordinator.isPlayerLogTailing"
      primary="Waiting for inventory data."
      secondary="Log into a character in-game to populate." />

    <!-- Two-panel layout: inventory table + activity feed -->
    <div v-if="store.isLivePopulated" class="flex gap-4 flex-1 min-h-0">
      <!-- Item table -->
      <div class="flex-1 overflow-auto">
        <table class="w-full text-sm border-collapse">
          <thead class="sticky top-0 bg-surface-base z-10">
            <tr class="text-left text-text-muted text-xs border-b border-border-default">
              <th class="py-1.5 px-2 w-12">Slot</th>
              <th class="py-1.5 px-2">Item</th>
              <th class="py-1.5 px-2 w-20 text-right">Qty</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in filteredItems"
              :key="item.instance_id"
              class="border-b border-border-default/30 hover:bg-surface-elevated/50 transition-colors"
              :class="{ 'border-l-2 border-l-accent-gold/60': item.is_new }"
            >
              <td class="py-1 px-2 text-text-muted text-xs font-mono">
                {{ item.slot_index >= 0 ? item.slot_index : '-' }}
              </td>
              <td class="py-1 px-2">
                <ItemInline :reference="item.item_name" />
              </td>
              <td class="py-1 px-2 text-right font-mono text-text-primary">
                {{ item.stack_size > 0 ? item.stack_size.toLocaleString() : '-' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Activity feed -->
      <div v-if="store.liveEventLog.length > 0" class="w-64 flex-shrink-0 flex flex-col border-l border-border-default pl-4">
        <h3 class="text-xs text-text-muted font-medium mb-2">Activity</h3>
        <div class="overflow-auto flex-1 space-y-1">
          <div
            v-for="(entry, i) in store.liveEventLog"
            :key="i"
            class="text-xs py-0.5"
          >
            <span
              class="font-medium"
              :class="{
                'text-green-400': entry.kind === 'added',
                'text-red-400': entry.kind === 'removed',
                'text-blue-400': entry.kind === 'stack_changed',
              }"
            >
              {{ entry.kind === 'added' ? '+' : entry.kind === 'removed' ? '-' : '~' }}
            </span>
            <span class="text-text-secondary ml-1">{{ entry.item_name }}</span>
            <span class="text-text-muted ml-1">({{ entry.detail }})</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useCoordinatorStore } from '../../stores/coordinatorStore'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'

const store = useGameStateStore()
const coordinator = useCoordinatorStore()

const searchQuery = ref('')

const filteredItems = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return store.liveItems
  return store.liveItems.filter(item => item.item_name.toLowerCase().includes(q))
})
</script>
