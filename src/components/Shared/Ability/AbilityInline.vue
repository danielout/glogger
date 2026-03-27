<template>
  <EntityTooltipWrapper
    border-class="border-entity-ability/50"
    @hover="loadIcon"
  >
    <component
      :is="plain ? 'span' : 'button'"
      :class="plain
        ? 'hover:underline cursor-pointer text-inherit'
        : 'inline-flex items-center gap-1 cursor-pointer hover:underline'"
      @click="handleClick"
    >
      <GameIcon v-if="!plain && ability.icon_id" :icon-id="ability.icon_id" :alt="ability.name" size="xs" />
      <span :class="plain ? '' : 'text-entity-ability text-xs font-medium'">{{ ability.name }}</span>
    </component>
    <template #tooltip>
      <AbilityTooltip :ability="ability" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { AbilityInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import AbilityTooltip from "./AbilityTooltip.vue";

const props = withDefaults(defineProps<{
  ability: AbilityInfo;
  plain?: boolean;
}>(), {
  plain: false,
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

function handleClick() {
  navigateToEntity({ type: "ability", id: props.ability.id });
}
</script>
