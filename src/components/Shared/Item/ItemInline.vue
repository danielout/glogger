<template>
  <EntityTooltipWrapper
    border-class="border-entity-item/50"
    @hover="loadData"
  >
    <button
      class="inline-flex items-center gap-1 cursor-pointer hover:underline"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon" :icon-id="itemData?.icon_id" :alt="name" size="xs" />
      <span class="text-entity-item text-xs font-medium">{{ itemData?.name ?? name }}</span>
    </button>
    <template #tooltip>
      <ItemTooltip v-if="itemData" :item="itemData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { ItemInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import ItemTooltip from "./ItemTooltip.vue";

const props = withDefaults(defineProps<{
  name: string;
  itemId?: number;
  showIcon?: boolean;
}>(), {
  showIcon: true,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const itemData = ref<ItemInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadData() {
  if (itemData.value) return;
  try {
    let item: ItemInfo | null = null;
    if (props.itemId) {
      item = await store.getItem(props.itemId);
    } else {
      // Try display name first, then fall back to internal name
      item = await store.getItemByName(props.name);
      if (!item) {
        item = await store.getItemByInternalName(props.name);
      }
    }
    if (!item) return;
    itemData.value = item;
    if (item.icon_id) {
      const path = await store.getIconPath(item.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to load item: ${props.name}`, e);
  }
}

onMounted(loadData);

function handleClick() {
  const id = itemData.value?.name ?? props.name;
  navigateToEntity({ type: "item", id });
}
</script>
