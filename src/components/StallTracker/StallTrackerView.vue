<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex items-center justify-between">
      <TabBar v-model="activeTab" :tabs="tabs" class="flex-1" />
      <div class="flex items-center gap-2 shrink-0">
        <span v-if="importMessage" class="text-xs text-[#7ec87e]">{{ importMessage }}</span>
        <button
          class="text-xs text-text-dim hover:text-text-primary transition-colors px-2 py-1"
          :disabled="importing"
          @click="handleImport">
          {{ importing ? 'Importing...' : 'Import' }}
        </button>
        <button
          v-if="store.sales.length > 0 || store.shopLog.length > 0"
          class="text-xs text-text-dim hover:text-accent-red transition-colors px-2 py-1"
          @click="handleClear">
          Clear data
        </button>
      </div>
    </div>
    <div class="flex-1 min-h-0 overflow-auto">
      <StallSalesTab v-if="activeTab === 'sales'" />
      <StallShopLogTab v-else-if="activeTab === 'shop-log'" />
      <StallRevenueTab v-else-if="activeTab === 'revenue'" />
      <StallInventoryTab v-else-if="activeTab === 'inventory'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { confirm, open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import TabBar from '../Shared/TabBar.vue'
import StallSalesTab from './StallSalesTab.vue'
import StallShopLogTab from './StallShopLogTab.vue'
import StallRevenueTab from './StallRevenueTab.vue'
import StallInventoryTab from './StallInventoryTab.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const tabs = [
  { id: 'sales', label: 'Sales' },
  { id: 'shop-log', label: 'Shop Log' },
  { id: 'revenue', label: 'Revenue' },
  { id: 'inventory', label: 'Inventory' },
]

const activeTab = ref('sales')
const store = useStallTrackerStore()
const importing = ref(false)
const importMessage = ref('')

onMounted(() => {
  store.loadAll()
})

interface ImportResult {
  total_entries: number
  new_entries: number
}

async function handleImport() {
  const files = await open({
    multiple: true,
    filters: [{ name: 'Shop Log', extensions: ['txt'] }],
  })
  if (!files) return

  const paths = Array.isArray(files) ? files : [files]
  importing.value = true
  importMessage.value = ''

  let totalNew = 0
  let totalEntries = 0
  try {
    for (const path of paths) {
      const result = await invoke<ImportResult>('import_shop_log_file', { path })
      totalNew += result.new_entries
      totalEntries += result.total_entries
    }
    const skipped = totalEntries - totalNew
    importMessage.value = `Imported ${totalNew} entries` + (skipped > 0 ? `, ${skipped} duplicates skipped` : '')
    await store.loadAll()
    setTimeout(() => { importMessage.value = '' }, 5000)
  } catch (e) {
    importMessage.value = `Import failed: ${e}`
  } finally {
    importing.value = false
  }
}

async function handleClear() {
  const ok = await confirm(
    'This will delete all stall tracker data (sales and shop log entries). You can re-import by opening your shop log books in-game.',
    { title: 'Clear Stall Data', kind: 'warning' },
  )
  if (ok) {
    await store.clearAll()
  }
}
</script>
