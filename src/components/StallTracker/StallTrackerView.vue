<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex items-center justify-between">
      <TabBar v-model="activeTab" :tabs="tabs" class="flex-1" />
      <div class="flex items-center gap-2 shrink-0">
        <span
          v-if="store.currentOwner"
          class="text-xs text-text-muted"
          :title="`All data scoped to ${store.currentOwner}`">
          Viewing: <span class="text-entity-player">{{ store.currentOwner }}</span>
        </span>
        <span v-if="importMessage" class="text-xs text-[#7ec87e]">{{ importMessage }}</span>
        <button
          v-if="hasData"
          class="text-xs text-text-dim hover:text-text-primary transition-colors px-2 py-1"
          @click="shopLogOpen = true">
          Shop Log
        </button>
        <button
          class="text-xs text-text-dim hover:text-text-primary transition-colors px-2 py-1"
          :disabled="importing"
          @click="handleImport">
          {{ importing ? 'Importing...' : 'Import' }}
        </button>
        <button
          v-if="hasData && store.currentOwner"
          class="text-xs text-text-dim hover:text-text-primary transition-colors px-2 py-1"
          :disabled="exporting"
          @click="handleExport">
          {{ exporting ? 'Exporting...' : 'Export' }}
        </button>
        <button
          v-if="hasData && store.currentOwner"
          class="text-xs text-text-dim hover:text-accent-red transition-colors px-2 py-1"
          @click="handleClear">
          Clear data
        </button>
      </div>
    </div>
    <div class="flex-1 min-h-0 overflow-auto">
      <StallSalesTab v-if="activeTab === 'sales'" />
      <StallRevenueTab v-else-if="activeTab === 'revenue'" />
      <StallInventoryTab v-else-if="activeTab === 'inventory'" />
    </div>

    <!-- Shop Log modal — first open lazy-mounts the tab; after that the
         modal container stays in the DOM via v-show so the tab's filter
         state and scroll position persist across opens within the session. -->
    <Teleport v-if="shopLogMounted" to="body">
      <Transition name="modal" appear>
        <div
          v-show="shopLogOpen"
          class="fixed inset-0 z-40 flex items-center justify-center p-6">
          <div class="absolute inset-0 bg-black/60" @click="shopLogOpen = false" />
          <div class="relative bg-surface-base border border-border-default rounded-lg shadow-xl flex flex-col w-full max-w-[min(1400px,95vw)] h-full max-h-[90vh]">
            <div class="flex items-center justify-between px-4 py-3 border-b border-border-default">
              <h3 class="text-sm font-semibold text-text-primary">Shop Log</h3>
              <button
                class="text-text-dim hover:text-text-primary transition-colors text-lg leading-none cursor-pointer px-2"
                title="Close (Esc)"
                @click="shopLogOpen = false">
                &times;
              </button>
            </div>
            <div class="flex-1 min-h-0 overflow-auto p-4">
              <StallShopLogTab />
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
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
  { id: 'revenue', label: 'Revenue' },
  { id: 'inventory', label: 'Inventory' },
]

const activeTab = ref('sales')
const store = useStallTrackerStore()
const importing = ref(false)
const importMessage = ref('')
const exporting = ref(false)

const shopLogOpen = ref(false)
// Lazy: the Teleport doesn't mount until the first time the user opens the
// modal. Once mounted it stays in the DOM; `v-show` on the inner overlay
// toggles visibility, so the StallShopLogTab instance — and its filter,
// sort, paging, and scroll state — persists across close/reopen cycles.
const shopLogMounted = ref(false)
watch(shopLogOpen, (open) => {
  if (open) shopLogMounted.value = true
})

function onKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape' || !shopLogOpen.value) return
  // If focus is inside an input (e.g., a SearchableSelect search field),
  // let that input's own Escape handler run — typically to close the
  // dropdown — without also tearing down the whole modal.
  const target = e.target as HTMLElement | null
  if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA')) return
  shopLogOpen.value = false
}

const hasData = computed(() =>
  (store.stats?.total_sales ?? 0) > 0 || store.filterOptions.dates.length > 0
)

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  await Promise.all([store.loadStats(), store.loadFilterOptions()])
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
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
  let filesWithNoEntries = 0
  try {
    for (const path of paths) {
      const result = await invoke<ImportResult>('import_shop_log_file', { path })
      totalNew += result.new_entries
      totalEntries += result.total_entries
      if (result.total_entries === 0) filesWithNoEntries++
    }
    const skipped = totalEntries - totalNew
    if (totalEntries === 0) {
      importMessage.value = filesWithNoEntries === 1
        ? 'No shop log entries found in file. Is it an exported shop log book?'
        : `No shop log entries found in ${filesWithNoEntries} file(s).`
    } else {
      importMessage.value = `Imported ${totalNew} entries` + (skipped > 0 ? `, ${skipped} duplicates skipped` : '')
    }
    store.dataVersion++
    await Promise.all([store.loadStats(), store.loadFilterOptions()])
    setTimeout(() => { importMessage.value = '' }, 5000)
  } catch (e) {
    importMessage.value = `Import failed: ${e}`
  } finally {
    importing.value = false
  }
}

interface ExportResult {
  files_written: number
  entries_written: number
  directory: string
}

async function handleExport() {
  const directory = await open({ directory: true, multiple: false })
  if (!directory || Array.isArray(directory)) return

  exporting.value = true
  importMessage.value = ''
  try {
    const result = await invoke<ExportResult>('export_shop_log_files', {
      directory,
      owner: store.currentOwner,
    })
    importMessage.value = `Exported ${result.entries_written} entries to ${result.files_written} file(s)`
    setTimeout(() => { importMessage.value = '' }, 5000)
  } catch (e) {
    importMessage.value = `Export failed: ${e}`
  } finally {
    exporting.value = false
  }
}

async function handleClear() {
  if (!store.currentOwner) return  // button is gated on this, defensive only
  const character = store.currentOwner
  const ok = await confirm(
    `This will delete all stall tracker data for ${character}. Other characters are not affected.\n\nConsider using Export first — in-game shop log books only hold recent history, so once this data is gone it may not be recoverable from the game.`,
    { title: 'Clear Stall Data', kind: 'warning' },
  )
  if (ok) {
    try {
      await store.clearAll()
    } catch (e) {
      importMessage.value = `Clear failed: ${e}`
    }
  }
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
