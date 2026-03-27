<template>
  <EntityTooltipWrapper
    border-class="border-entity-item/50"
    @hover="loadItemData"
  >
    <div class="bg-black/20 border border-border-default rounded-md p-3 cursor-help transition-all duration-200 hover:bg-black/35 hover:border-border-hover hover:-translate-y-0.5 hover:shadow-lg">
      <div class="flex items-center gap-3">
        <GameIcon :icon-id="itemData?.icon_id" :alt="itemName" size="lg" />
        <div class="flex-1 min-w-0 flex flex-col gap-1">
          <div class="text-text-primary text-sm font-medium overflow-hidden text-ellipsis whitespace-nowrap">{{ itemName }}</div>
          <div class="flex gap-2 text-xs">
            <span class="text-text-secondary font-semibold">&times;{{ count }}</span>
            <span class="text-entity-item">{{ percentage }}%</span>
          </div>
        </div>
      </div>
    </div>
    <template #tooltip>
      <ItemTooltip v-if="itemData" :item="itemData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../stores/gameDataStore";
import type { ItemInfo } from "../../types/gameData";
import EntityTooltipWrapper from "./EntityTooltipWrapper.vue";
import GameIcon from "./GameIcon.vue";
import ItemTooltip from "./Item/ItemTooltip.vue";

const props = defineProps<{
  itemName: string;
  count: number;
  percentage: number;
}>();

const store = useGameDataStore();

const itemData = ref<ItemInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadItemData() {
  if (itemData.value) return;
  try {
    const item = await store.resolveItem(props.itemName);
    if (!item) return;
    itemData.value = item;
    if (item.icon_id) {
      const path = await store.getIconPath(item.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to load item: ${props.itemName}`, e);
  }
}

onMounted(loadItemData);
</script>
