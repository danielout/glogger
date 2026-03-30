<template>
  <div class="flex flex-col gap-1.5">
    <div class="flex items-center gap-2">
      <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Base Item</h4>
    </div>

    <!-- Currently selected item -->
    <div v-if="currentItem" class="flex items-center gap-2 px-2 py-1.5 bg-surface-elevated border border-border-default rounded text-sm group">
      <ItemInline :reference="String(currentItem.item_id)" :show-icon="true" />
      <span class="flex-1" />
      <button
        class="text-red-400/60 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
        title="Remove item"
        @click="store.clearSlotItem()">
        x
      </button>
    </div>

    <!-- Search input (shown when no item or when searching) -->
    <div v-if="!currentItem || searching" class="relative">
      <input
        ref="searchInput"
        v-model="query"
        type="text"
        :placeholder="currentItem ? 'Search for a different item...' : 'Search for an item...'"
        class="w-full bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary"
        @focus="searching = true"
        @input="onSearch" />

      <!-- Search results dropdown -->
      <div
        v-if="searching && (results.length > 0 || query.length > 0)"
        class="absolute z-20 top-full left-0 right-0 mt-1 bg-surface-elevated border border-border-default rounded shadow-lg max-h-48 overflow-y-auto">
        <div v-if="loading" class="px-2 py-2 text-xs text-text-muted text-center">
          Searching...
        </div>
        <div v-else-if="results.length === 0 && query.length > 0" class="px-2 py-2 text-xs text-text-dim text-center">
          No items found
        </div>
        <button
          v-for="item in results"
          :key="item.id"
          class="w-full text-left px-2 py-1.5 text-xs hover:bg-surface-hover cursor-pointer flex items-center gap-2 border-b border-border-default last:border-b-0"
          @click="selectItem(item)">
          <GameIcon :icon-id="item.icon_id" :alt="item.name" size="xs" />
          <div class="flex-1 min-w-0">
            <div class="text-text-primary truncate">{{ item.name }}</div>
            <div v-if="item.skill_reqs" class="text-[10px] text-text-dim">
              <span v-for="(level, skill) in item.skill_reqs" :key="String(skill)" class="mr-2">
                {{ skill }} {{ level }}
              </span>
            </div>
          </div>
          <span v-if="item.craft_points" class="text-[10px] text-text-dim shrink-0">
            {{ item.craft_points }}cp
          </span>
        </button>
      </div>
    </div>

    <!-- Toggle search when item is set -->
    <button
      v-if="currentItem && !searching"
      class="text-[10px] text-text-dim hover:text-text-secondary cursor-pointer text-left"
      @click="searching = true">
      Change item...
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import type { ItemInfo } from '../../../types/gameData'
import ItemInline from '../../Shared/Item/ItemInline.vue'
import GameIcon from '../../Shared/GameIcon.vue'

const store = useBuildPlannerStore()
const gameData = useGameDataStore()

const query = ref('')
const results = ref<ItemInfo[]>([])
const loading = ref(false)
const searching = ref(false)
let searchTimeout: ReturnType<typeof setTimeout> | null = null

const currentItem = computed(() => {
  if (!store.selectedSlot) return undefined
  return store.getSlotItem(store.selectedSlot)
})

// Close search when slot changes
watch(() => store.selectedSlot, () => {
  searching.value = false
  query.value = ''
  results.value = []
})

function onSearch() {
  if (searchTimeout) clearTimeout(searchTimeout)
  if (query.value.length < 2) {
    results.value = []
    return
  }
  loading.value = true
  searchTimeout = setTimeout(async () => {
    try {
      results.value = await gameData.searchItems(query.value, 20, {
        equipSlot: store.selectedSlot ?? undefined,
      })
    } catch {
      results.value = []
    } finally {
      loading.value = false
    }
  }, 200)
}

async function selectItem(item: ItemInfo) {
  await store.setSlotItem(item.id, item.name)
  searching.value = false
  query.value = ''
  results.value = []
}
</script>
