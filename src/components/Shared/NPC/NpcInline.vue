<template>
  <EntityTooltipWrapper
    border-class="border-entity-npc/50"
    :disabled="!resolvedNpc"
    @hover="() => {}"
  >
    <span
      class="inline-flex items-center gap-0.5 cursor-pointer hover:underline text-entity-npc font-medium"
      :class="bordered ? 'bg-entity-npc/5 border border-entity-npc/20 rounded px-1 py-0.5' : ''"
      @click="handleClick"
    >
      <span>{{ resolvedNpc?.name ?? reference }}</span>
    </span>
    <template #tooltip>
      <NpcTooltip v-if="resolvedNpc" :npc="resolvedNpc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { NpcInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import NpcTooltip from "./NpcTooltip.vue";

const props = withDefaults(defineProps<{
  reference: string;
  npc?: NpcInfo;
  bordered?: boolean;
}>(), {
  bordered: false,
});

const gameData = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

/** Resolve NPC data: explicit prop > key lookup > display name lookup */
const resolvedNpc = computed<NpcInfo | null>(() => {
  if (props.npc) return props.npc;
  return gameData.resolveNpcSync(props.reference);
});

function handleClick() {
  navigateToEntity({ type: "npc", id: resolvedNpc.value?.key ?? props.reference });
}
</script>
