<template>
  <div class="flex flex-col gap-3 min-w-64">
    <!-- Currently tracked items (scrollable, capped height) -->
    <div class="flex flex-col min-h-0">
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1 shrink-0">Tracked Items</div>
      <div v-if="prefs.trackedItems.length === 0" class="text-xs text-text-dim italic">None</div>
      <div v-else class="overflow-y-auto max-h-32">
        <div v-for="name in prefs.trackedItems" :key="name" class="flex items-center justify-between gap-2 py-0.5">
          <ItemInline :reference="name" />
          <button
            class="text-xs text-red-400 hover:text-red-300 cursor-pointer shrink-0"
            @click="removeItem(name)">
            Remove
          </button>
        </div>
      </div>
    </div>

    <!-- Search to add -->
    <div class="shrink-0">
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">Add Item</div>
      <input
        v-model="search"
        type="text"
        placeholder="Search items..."
        class="w-full px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50" />
    </div>

    <!-- Search results -->
    <div class="overflow-y-auto min-h-30 max-h-40">
      <div v-if="isSearching" class="text-xs text-text-dim italic py-1">Searching...</div>
      <template v-else>
        <div
          v-for="item in searchResults"
          :key="item.id"
          class="flex items-center justify-between gap-2 py-0.5 cursor-pointer hover:bg-surface-elevated/50 px-1 rounded"
          @click="addItem(item.name)">
          <ItemInline :reference="item.name" />
        </div>
        <div v-if="search.length > 0 && !isSearching && searchResults.length === 0" class="text-xs text-text-dim italic py-1">
          No matching items found.
        </div>
        <div v-if="search.length === 0" class="text-xs text-text-dim italic py-1">
          Type to search for items.
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { useViewPrefs } from '../../../composables/useViewPrefs'
import { CRITICAL_RESOURCES_DEFAULTS, type CriticalResourcesPrefs } from './criticalResourcesPrefs'
import ItemInline from '../../Shared/Item/ItemInline.vue'
import type { ItemInfo } from '../../../types/gameData/items'

const gameData = useGameDataStore()
const { prefs, update } = useViewPrefs<CriticalResourcesPrefs>('widget.critical-resources', CRITICAL_RESOURCES_DEFAULTS)
const search = ref('')
const searchResults = ref<ItemInfo[]>([])
const isSearching = ref(false)

let searchTimer: ReturnType<typeof setTimeout> | null = null

watch(search, (query) => {
  if (searchTimer) clearTimeout(searchTimer)
  if (!query.trim()) {
    searchResults.value = []
    isSearching.value = false
    return
  }
  isSearching.value = true
  searchTimer = setTimeout(async () => {
    const results = await gameData.searchItems(query.trim())
    const tracked = new Set(prefs.value.trackedItems)
    searchResults.value = results.filter(item => !tracked.has(item.name)).slice(0, 20)
    isSearching.value = false
  }, 250)
})

function addItem(name: string) {
  if (!prefs.value.trackedItems.includes(name)) {
    update({ trackedItems: [...prefs.value.trackedItems, name] })
  }
  search.value = ''
}

function removeItem(name: string) {
  update({ trackedItems: prefs.value.trackedItems.filter(n => n !== name) })
}
</script>
