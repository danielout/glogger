<template>
  <EntityTooltipWrapper
    class="flex! w-full min-w-0"
    border-class="border-entity-item/50"
    @hover="loadData"
  >
    <slot />
    <template #tooltip>
      <ItemTooltip v-if="itemData" :item="itemData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useGameDataStore } from '../../stores/gameDataStore'
import type { ItemInfo } from '../../types/gameData'
import EntityTooltipWrapper from '../Shared/EntityTooltipWrapper.vue'
import ItemTooltip from '../Shared/Item/ItemTooltip.vue'

const props = defineProps<{
  foodName: string
}>()

const store = useGameDataStore()
const itemData = ref<ItemInfo | null>(null)
const iconSrc = ref<string | null>(null)

async function loadData() {
  if (itemData.value) return
  try {
    const item = await store.resolveItem(props.foodName)
    if (!item) return
    itemData.value = item
    if (item.icon_id) {
      const path = await store.getIconPath(item.icon_id)
      iconSrc.value = convertFileSrc(path)
    }
  } catch (e) {
    console.warn(`Failed to load item: ${props.foodName}`, e)
  }
}
</script>
