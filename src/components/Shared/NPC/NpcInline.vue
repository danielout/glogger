<template>
  <EntityTooltipWrapper
    border-class="border-entity-npc/50"
    :disabled="!resolvedNpc"
    @hover="() => {}"
  >
    <component
      :is="plain ? 'span' : 'button'"
      :class="plain
        ? 'hover:underline cursor-pointer text-inherit'
        : 'inline-flex items-center gap-1 cursor-pointer hover:underline'"
      @click="handleClick"
    >
      <span :class="plain ? '' : 'text-entity-npc text-xs font-medium'">{{ resolvedNpc?.name ?? reference }}</span>
    </component>
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
  plain?: boolean;
}>(), {
  plain: false,
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
