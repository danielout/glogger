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
        <button
          v-if="store.hasData"
          type="button"
          class="px-3 py-1 text-xs bg-surface-elevated border border-border-default rounded hover:border-accent-gold/50 text-text-primary"
          @click="openShopLog">
          Shop Log
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
import TabBar, { type Tab } from '../Shared/TabBar.vue'
import EmptyState from '../Shared/EmptyState.vue'
import StallSalesTab from './StallSalesTab.vue'
import StallRevenueTab from './StallRevenueTab.vue'
import StallInventoryTab from './StallInventoryTab.vue'
import StallShopLogTab from './StallShopLogTab.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const store = useStallTrackerStore()

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

// Native <input type="date"> popups are controlled by the webview and
// Chromium's outside-click behavior is inconsistent — sometimes the popup
// stays open until the input itself loses focus. Force-blur any focused
// date input on a mousedown outside it so the picker closes the way users
// expect. One global listener at the parent covers every date input under
// the Stall Tracker (Sales / Revenue / Shop Log modal).
function onDocumentMousedown(e: MouseEvent) {
  const active = document.activeElement as HTMLInputElement | null
  if (!active || active.tagName !== 'INPUT' || active.type !== 'date') return
  if (e.target instanceof Node && active.contains(e.target)) return
  active.blur()
}
document.addEventListener('mousedown', onDocumentMousedown)

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  document.removeEventListener('mousedown', onDocumentMousedown)
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
