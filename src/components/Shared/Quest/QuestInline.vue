<template>
  <EntityTooltipWrapper
    border-class="border-entity-quest/50"
    @hover="loadData"
  >
    <button
      class="inline-flex items-center gap-1 cursor-pointer hover:underline"
      @click="handleClick"
    >
      <span class="text-entity-quest text-xs font-medium">{{ displayName }}</span>
    </button>
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

const props = defineProps<{
  questKey: string;
}>();

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const questData = ref<QuestInfo | null>(null);

const displayName = computed(() =>
  questData.value?.raw.Name ?? props.questKey
);

async function loadData() {
  if (questData.value) return;
  try {
    questData.value = await store.getQuestByKey(props.questKey);
  } catch (e) {
    console.warn(`Failed to load quest: ${props.questKey}`, e);
  }
}

function handleClick() {
  navigateToEntity({ type: "quest", id: props.questKey });
}
</script>
