<template>
  <EntityTooltipWrapper
    border-class="border-entity-ability/50"
    @hover="loadIcon"
  >
    <span
      class="inline-flex items-center gap-0.5 cursor-pointer hover:underline text-entity-ability font-medium"
      :class="bordered ? 'bg-entity-ability/5 border border-entity-ability/20 rounded px-1 py-0.5' : ''"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon && ability.icon_id" :icon-id="ability.icon_id" :alt="ability.name" size="inline" />
      <span>{{ ability.name }}</span>
    </span>
    <template #tooltip>
      <AbilityTooltip :ability="ability" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { AbilityInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import AbilityTooltip from "./AbilityTooltip.vue";

const props = withDefaults(defineProps<{
  ability: AbilityInfo;
  showIcon?: boolean;
  bordered?: boolean;
}>(), {
  showIcon: true,
  bordered: false,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const iconSrc = ref<string | null>(null);

async function loadIcon() {
  if (iconSrc.value || !props.ability.icon_id) return;
  try {
    const path = await store.getIconPath(props.ability.icon_id);
    iconSrc.value = convertFileSrc(path);
  } catch (e) {
    console.warn(`Icon load failed for ability: ${props.ability.name}`, e);
  }
}

onMounted(loadIcon);

function handleClick() {
  navigateToEntity({ type: "ability", id: props.ability.id });
}
</script>
