<template>
  <div class="flex flex-wrap gap-1">
    <EntityTooltipWrapper
      v-for="item in items"
      :key="item.id"
      border-class="border-entity-item/50"
      @hover="() => loadItem(item)"
    >
      <button
        class="w-8 h-8 rounded-sm bg-black/30 border border-border-light hover:border-entity-item/50 cursor-pointer relative overflow-hidden p-0"
        @click="handleClick(item)"
      >
        <img
          v-if="iconCache[item.type_id]"
          :src="iconCache[item.type_id]"
          :alt="item.item_name"
          class="absolute inset-0 w-full h-full object-cover" />
        <span v-else class="text-[10px] text-text-muted">?</span>
        <span
          v-if="item.stack_size > 1"
          class="absolute bottom-0 right-0 text-[10px] leading-none bg-black/70 text-text-primary px-0.5 rounded-tl-sm"
        >{{ item.stack_size }}</span>
      </button>
      <template #tooltip>
        <ItemTooltip v-if="itemDataCache[item.type_id]" :item="itemDataCache[item.type_id]!" :icon-src="iconCache[item.type_id] ?? null" />
        <div v-else class="text-text-primary text-xs">{{ item.item_name }}</div>
      </template>
    </EntityTooltipWrapper>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useGameDataStore } from '../../stores/gameDataStore'
import { useEntityNavigation } from '../../composables/useEntityNavigation'
import type { SnapshotItem } from '../../types/database'
import type { ItemInfo } from '../../types/gameData'
import EntityTooltipWrapper from '../Shared/EntityTooltipWrapper.vue'
import ItemTooltip from '../Shared/Item/ItemTooltip.vue'

const props = defineProps<{ items: SnapshotItem[] }>()

const store = useGameDataStore()
const { navigateToEntity } = useEntityNavigation()

const iconCache = reactive<Record<number, string>>({})
const itemDataCache = reactive<Record<number, ItemInfo>>({})

async function loadItem(item: SnapshotItem) {
  if (itemDataCache[item.type_id]) return
  try {
    const data = await store.resolveItem(item.type_id)
    if (!data) return
    itemDataCache[item.type_id] = data
    if (data.icon_id) {
      const path = await store.getIconPath(data.icon_id)
      iconCache[item.type_id] = convertFileSrc(path)
    }
  } catch { /* skip */ }
}

function handleClick(item: SnapshotItem) {
  navigateToEntity({ type: 'item', id: item.item_name })
}

// Eagerly load icons for visible items (grid views need icons before hover)
async function preloadIcons(items: SnapshotItem[]) {
  const seen = new Set<number>()
  for (const item of items) {
    if (seen.has(item.type_id) || iconCache[item.type_id]) continue
    seen.add(item.type_id)
    loadItem(item)
  }
}

watch(() => props.items, (items) => preloadIcons(items), { immediate: true })
</script>
