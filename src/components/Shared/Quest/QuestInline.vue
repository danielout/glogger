<template>
  <EntityTooltipWrapper
    border-class="border-entity-quest/50"
    @hover="loadData"
  >
    <component
      :is="plain ? 'span' : 'button'"
      :class="plain
        ? 'hover:underline cursor-pointer text-inherit'
        : 'inline-flex items-center gap-1 cursor-pointer hover:underline'"
      @click="handleClick"
    >
      <span :class="plain ? '' : 'text-entity-quest text-xs font-medium'">{{ displayName }}</span>
    </component>
    <template #tooltip>
      <QuestTooltip v-if="questData" :quest="questData" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { QuestInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import QuestTooltip from "./QuestTooltip.vue";

const props = withDefaults(defineProps<{
  reference: string;
  plain?: boolean;
}>(), {
  plain: false,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const questData = ref<QuestInfo | null>(null);

const displayName = computed(() =>
  questData.value?.raw.Name ?? props.reference
);

async function loadData() {
  if (questData.value) return;
  try {
    questData.value = await store.resolveQuest(props.reference);
  } catch (e) {
    console.warn(`Failed to resolve quest: ${props.reference}`, e);
  }
}

function handleClick() {
  navigateToEntity({ type: "quest", id: questData.value?.internal_name ?? props.reference });
}
</script>
