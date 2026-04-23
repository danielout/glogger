<template>
  <EntityTooltipWrapper
    border-class="border-entity-item/50"
    @hover="loadData"
  >
    <button
      class="flex items-center gap-3 bg-surface-card border border-border-default rounded-md px-3 py-2 cursor-pointer hover:bg-surface-elevated hover:border-border-hover transition-colors"
      @click="handleClick"
    >
      <GameIcon :icon-id="itemData?.icon_id" :alt="name" size="lg" />
      <div class="flex flex-col items-start gap-0.5 min-w-0">
        <span class="text-entity-item text-sm font-bold truncate max-w-48">{{ itemData?.name ?? name }}</span>
        <span v-if="itemData?.value" class="text-accent-gold text-xs">{{ itemData.value }}g</span>
        <span v-if="typeline" class="text-text-muted text-[10px] uppercase tracking-wide">{{ typeline }}</span>
      </div>
    </button>
    <template #tooltip>
      <ItemTooltip v-if="itemData" :item="itemData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { ItemInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import ItemTooltip from "./ItemTooltip.vue";

const props = defineProps<{
  name: string;
}>();

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const itemData = ref<ItemInfo | null>(null);
const iconSrc = ref<string | null>(null);

const typeline = computed(() => itemData.value?.keywords?.[0] ?? null);

async function loadData() {
  if (itemData.value) return;
  try {
    const item = await store.resolveItem(props.name);
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
