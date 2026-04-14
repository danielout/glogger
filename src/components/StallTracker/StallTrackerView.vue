<template>
  <div class="flex flex-col gap-4 h-full min-h-0">
    <!-- Tab bar + action row -->
    <div class="flex items-end justify-between gap-4 flex-shrink-0">
      <TabBar
        v-model="activeTab"
        :tabs="tabs" />

      <div class="flex items-center gap-3 pb-2">
        <span
          v-if="store.currentOwner"
          class="text-sm text-text-secondary"
          :title="`All Stall Tracker data is scoped to the active character (${store.currentOwner}). Switch character to view another stall.`">
          Viewing: <span class="text-entity-player">{{ store.currentOwner }}</span>
        </span>
        <span
          v-if="actionMessage"
          class="text-xs text-text-secondary italic max-w-md truncate"
          :title="actionMessage">
          {{ actionMessage }}
        </span>
        <button
          v-if="store.hasData"
          type="button"
          class="px-3 py-1 text-xs bg-surface-elevated border border-border-default rounded hover:border-accent-gold/50 text-text-primary"
          @click="openShopLog">
          Shop Log
        </button>
        <button
          type="button"
          class="px-3 py-1 text-xs bg-surface-elevated border border-border-default rounded hover:border-accent-gold/50 text-text-primary disabled:opacity-50"
          :disabled="actionInProgress"
          @click="handleImport">
          Import
        </button>
        <button
          v-if="store.hasData && store.currentOwner"
          type="button"
          class="px-3 py-1 text-xs bg-surface-elevated border border-border-default rounded hover:border-accent-gold/50 text-text-primary disabled:opacity-50"
          :disabled="actionInProgress"
          @click="handleExport">
          Export
        </button>
        <button
          v-if="store.hasData && store.currentOwner"
          type="button"
          class="px-3 py-1 text-xs bg-surface-elevated border border-border-default rounded hover:border-red-500/50 hover:text-red-400 text-text-primary disabled:opacity-50"
          :disabled="actionInProgress"
          @click="handleClear">
          Clear data
        </button>
      </div>
    </div>

    <!-- Active tab content -->
    <div class="flex-1 min-h-0 overflow-hidden">
      <EmptyState
        v-if="!store.currentOwner"
        primary="No active character"
        secondary="Set an active character in Settings to view Stall Tracker data." />
      <template v-else>
        <StallSalesTab v-if="activeTab === 'sales'" />
        <StallRevenueTab v-else-if="activeTab === 'revenue'" />
        <StallInventoryTab v-else-if="activeTab === 'inventory'" />
      </template>
    </div>

    <!-- Shop Log modal — lazy mounted (v-if on Teleport) so the inner tab's
         filter/sort/pagination state doesn't exist until the user opens it
         the first time. After mount, subsequent toggles use v-show on the
         inner overlay so the StallShopLogTab instance — and its scroll
         position, filters, page offset — persists across open/close cycles
         within the session. z-[60] is required: the app-wide MenuBar is
         z-50 and would paint over the modal title at z-50 or below
         (verified by plan §11.5). -->
    <Teleport
      v-if="shopLogMounted"
      to="body">
      <div
        v-show="shopLogOpen"
        class="fixed inset-0 z-[60] flex items-center justify-center p-4">
        <div
          class="absolute inset-0 bg-black/60"
          @click="shopLogOpen = false" />
        <div class="relative bg-surface-base border border-border-default rounded shadow-2xl flex flex-col w-full max-w-6xl h-full max-h-[90vh]">
          <div class="flex items-center justify-between px-4 py-3 border-b border-border-default">
            <h3 class="text-text-primary text-sm font-medium m-0">Shop Log</h3>
            <button
              type="button"
              class="text-text-secondary hover:text-text-primary text-xl leading-none px-2"
              aria-label="Close"
              @click="shopLogOpen = false">
              ×
            </button>
          </div>
          <div class="flex-1 min-h-0 overflow-hidden p-4">
            <StallShopLogTab />
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog, confirm } from '@tauri-apps/plugin-dialog'
import TabBar, { type Tab } from '../Shared/TabBar.vue'
import EmptyState from '../Shared/EmptyState.vue'
import StallSalesTab from './StallSalesTab.vue'
import StallRevenueTab from './StallRevenueTab.vue'
import StallInventoryTab from './StallInventoryTab.vue'
import StallShopLogTab from './StallShopLogTab.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import type { ImportResult, ExportResult } from '../../types/stallTracker'

const store = useStallTrackerStore()

// Ephemeral status text shown in the action row (auto-clears after 5s).
const actionMessage = ref<string>('')
const actionInProgress = ref<boolean>(false)
let messageTimer: ReturnType<typeof setTimeout> | null = null

function flashMessage(text: string) {
  actionMessage.value = text
  if (messageTimer) clearTimeout(messageTimer)
  messageTimer = setTimeout(() => {
    actionMessage.value = ''
    messageTimer = null
  }, 5000)
}

async function handleImport() {
  if (actionInProgress.value) return
  const selected = await openDialog({
    multiple: true,
    filters: [{ name: 'Shop log book', extensions: ['txt'] }],
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  if (paths.length === 0) return

  actionInProgress.value = true
  // Tally imported entries per owner. Continue on per-file errors so that
  // a single malformed file in a 5-file import doesn't lose the other 4
  // (each invoke runs its own transaction — partial success is real and
  // the user deserves an accurate summary).
  const entriesByOwner = new Map<string, number>()
  const failedFiles: { path: string; error: string }[] = []
  let totalNew = 0
  let totalRows = 0
  let anyClaimed = false

  for (const path of paths) {
    try {
      const result = await invoke<ImportResult>('import_shop_log_file', {
        path,
        currentOwner: store.currentOwner,
      })
      const owner = result.effective_owner ?? '(unknown)'
      entriesByOwner.set(owner, (entriesByOwner.get(owner) ?? 0) + result.total_entries)
      totalNew += result.new_entries
      totalRows += result.total_entries
      if (result.owner_claimed) anyClaimed = true
    } catch (e) {
      const fileName = path.split(/[/\\]/).pop() ?? path
      failedFiles.push({ path: fileName, error: String(e) })
      console.error(`[StallTracker] Import failed for ${path}:`, e)
    }
  }

  await store.refresh()
  actionInProgress.value = false

  // Build the summary. Successes and failures are reported together so the
  // user always knows the partial-success state.
  const successCount = paths.length - failedFiles.length
  const parts: string[] = []

  if (totalRows === 0 && failedFiles.length === 0) {
    flashMessage('No shop log entries found in the selected file(s).')
    return
  }

  if (totalRows > 0) {
    const ownersList = Array.from(entriesByOwner.entries())
      .map(([o, n]) => `${n.toLocaleString()} for ${o}`)
      .join(', ')
    const otherOwners = Array.from(entriesByOwner.keys()).filter(
      (o) => o !== store.currentOwner,
    )
    if (otherOwners.length > 0) {
      parts.push(`Imported ${ownersList}. Switch character to view entries for other owners.`)
    } else if (anyClaimed) {
      parts.push(
        `Imported ${totalRows.toLocaleString()} entries (claimed for ${store.currentOwner ?? 'active character'} — book did not identify an owner).`,
      )
    } else {
      const dupes = totalRows - totalNew
      parts.push(
        `Imported ${totalNew.toLocaleString()} entries${dupes > 0 ? `, ${dupes.toLocaleString()} duplicates skipped` : ''}.`,
      )
    }
  }

  if (failedFiles.length > 0) {
    if (successCount > 0) {
      parts.push(
        `${failedFiles.length} of ${paths.length} files failed: ${failedFiles
          .map((f) => f.path)
          .join(', ')}`,
      )
    } else {
      parts.push(`Import failed: ${failedFiles[0].error}`)
    }
  }

  flashMessage(parts.join(' '))
}

async function handleExport() {
  if (actionInProgress.value || !store.currentOwner) return
  const dir = await openDialog({ directory: true })
  if (!dir || Array.isArray(dir)) return

  actionInProgress.value = true
  try {
    const result = await invoke<ExportResult>('export_shop_log_files', {
      directory: dir,
      owner: store.currentOwner,
    })
    flashMessage(
      `Exported ${result.events_exported.toLocaleString()} events to ${result.files_written.toLocaleString()} file(s).`,
    )
  } catch (e) {
    flashMessage(`Export failed: ${e}`)
    console.error('[StallTracker] Export failed:', e)
  } finally {
    actionInProgress.value = false
  }
}

async function handleClear() {
  if (actionInProgress.value || !store.currentOwner) return
  const ok = await confirm(
    `This will delete all stall tracker data for ${store.currentOwner}. Other characters are not affected.\n\n` +
      `Consider using Export first — in-game shop log books only hold recent history, so once this data is gone it may not be recoverable from the game.`,
    { title: 'Clear Stall Data', kind: 'warning' },
  )
  if (!ok) return

  actionInProgress.value = true
  try {
    const deleted = await store.clearAll()
    flashMessage(`Cleared ${deleted.toLocaleString()} stall events for ${store.currentOwner}.`)
  } catch (e) {
    flashMessage(`Clear failed: ${e}`)
    console.error('[StallTracker] Clear failed:', e)
  } finally {
    actionInProgress.value = false
  }
}

const tabs: Tab[] = [
  { id: 'sales', label: 'Sales' },
  { id: 'revenue', label: 'Revenue' },
  { id: 'inventory', label: 'Inventory' },
]
const activeTab = ref<string>('sales')

// Shop Log modal state — `shopLogMounted` is one-way: once true, never reset
// within a session. `shopLogOpen` is the visibility toggle that the v-show
// inside the Teleport reads.
const shopLogMounted = ref(false)
const shopLogOpen = ref(false)

function openShopLog() {
  shopLogMounted.value = true
  shopLogOpen.value = true
}

function onKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape' || !shopLogOpen.value) return
  // CRITICAL: don't swallow Escape inside text inputs or SearchableSelect
  // dropdowns. The dropdowns handle their own Escape-to-close; intercepting
  // here would tear down the whole modal instead of just closing the dropdown.
  // Date inputs are exempted — Escape from a date input should close the
  // modal, since native date pickers don't have their own Escape behavior
  // worth preserving.
  const target = e.target as HTMLElement | null
  if (target && target.tagName === 'TEXTAREA') return
  if (target && target.tagName === 'INPUT' && (target as HTMLInputElement).type !== 'date') return
  shopLogOpen.value = false
}
window.addEventListener('keydown', onKeydown)
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  if (messageTimer) clearTimeout(messageTimer)
})

// Close the modal on character switch — the persisted Shop Log filters
// would otherwise apply against a different character's data on next open.
watch(
  () => store.currentOwner,
  () => {
    shopLogOpen.value = false
  },
)

// Lazy: stats and filter options were preloaded at startup, but if the user
// somehow lands here before that's done, fire one refresh.
onMounted(() => {
  if (store.stats === null && store.currentOwner) {
    void store.loadStats()
    void store.loadFilterOptions()
  }
})
</script>
