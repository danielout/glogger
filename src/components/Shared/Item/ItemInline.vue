<template>
  <EntityTooltipWrapper
    border-class="border-entity-item/50"
    :interactive="true"
    @hover="loadData"
  >
    <component
      :is="plain ? 'span' : 'button'"
      :class="plain
        ? 'hover:underline cursor-pointer text-inherit'
        : 'inline-flex items-center gap-1 cursor-pointer hover:underline'"
      @click="handleClick"
    >
      <GameIcon v-if="!plain && showIcon" :icon-id="itemData?.icon_id" :alt="reference" size="xs" />
      <span :class="plain ? '' : 'text-entity-item text-xs font-medium'">{{ itemData?.name ?? reference }}</span>
    </component>
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
  reference: string;
  showIcon?: boolean;
  plain?: boolean;
}>(), {
  showIcon: true,
  plain: false,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const itemData = ref<ItemInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadData() {
  if (itemData.value) return;
  try {
    const item = await store.resolveItem(props.reference);
    if (!item) return;
    itemData.value = item;
    if (item.icon_id) {
      const path = await store.getIconPath(item.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to resolve item: ${props.reference}`, e);
  }
}

onMounted(loadData);

function handleClick() {
  const id = itemData.value?.name ?? props.reference;
  navigateToEntity({ type: "item", id });
}
</script>
