<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex gap-2 border-b-2 border-border-default pb-2">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="[
          'px-5 py-2 bg-[#1a1a2e] border border-border-light border-b-0 rounded-t text-text-secondary cursor-pointer text-sm font-medium transition-all relative hover:bg-[#2a2a3e] hover:text-text-secondary',
          activeTab === tab.id && 'bg-[#2a2a3e]! border-[#7ec8e3]! text-[#7ec8e3]! font-semibold after:content-[\'\'] after:absolute after:-bottom-0.5 after:left-0 after:right-0 after:h-0.5 after:bg-[#7ec8e3]'
        ]"
        @click="activeTab = tab.id">
        {{ tab.label }}
      </button>
    </div>

    <div class="flex-1 min-h-0">
      <LiveInventoryTab v-if="activeTab === 'live'" />
      <InventoryView v-else-if="activeTab === 'storage'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import LiveInventoryTab from './LiveInventoryTab.vue'
import InventoryView from '../Character/InventoryView.vue'

type TabId = 'live' | 'storage'

const tabs: { id: TabId; label: string }[] = [
  { id: 'live', label: 'Inventory' },
  { id: 'storage', label: 'Storage' },
]

const activeTab = ref<TabId>('live')
</script>
