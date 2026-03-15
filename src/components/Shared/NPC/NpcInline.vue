<template>
  <EntityTooltipWrapper
    border-class="border-entity-npc/50"
    :disabled="!npc"
    @hover="() => {}"
  >
    <button
      class="inline-flex items-center gap-1 cursor-pointer hover:underline"
      @click="handleClick"
    >
      <span class="text-entity-npc text-xs font-medium">{{ npc?.name ?? name }}</span>
    </button>
    <template #tooltip>
      <NpcTooltip v-if="npc" :npc="npc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { NpcInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import NpcTooltip from "./NpcTooltip.vue";

const props = defineProps<{
  name: string;
  npc?: NpcInfo;
}>();

const { navigateToEntity } = useEntityNavigation();

function handleClick() {
  navigateToEntity({ type: "npc", id: props.npc?.key ?? props.name });
}
</script>
