<template>
  <EntityTooltipWrapper
    border-class="border-entity-item/50"
    @hover="loadData"
  >
    <button class="cursor-pointer" @click="handleClick">
      <GameIcon :icon-id="itemData?.icon_id" :alt="name" :size="size" />
    </button>
    <template #tooltip>
      <ItemTooltip v-if="itemData" :item="itemData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { ItemInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import ItemTooltip from "./ItemTooltip.vue";

const props = withDefaults(defineProps<{
  name: string;
  size?: "xs" | "sm" | "md" | "lg";
}>(), {
  size: "sm",
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const itemData = ref<ItemInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadData() {
  if (itemData.value) return;
  try {
    const item = await store.getItemByName(props.name);
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

function handleClick() {
  navigateToEntity({ type: "item", id: props.name });
}
</script>
