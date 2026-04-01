<template>
  <EntityTooltipWrapper
    border-class="border-entity-quest/50"
    @hover="loadData"
  >
    <span
      class="inline-flex items-center gap-0.5 cursor-pointer hover:underline text-entity-quest font-medium"
      :class="bordered ? 'bg-entity-quest/5 border border-entity-quest/20 rounded px-1 py-0.5' : ''"
      @click="handleClick"
    >
      <span>{{ displayName }}</span>
    </span>
    <template #tooltip>
      <QuestTooltip v-if="questData" :quest="questData" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { QuestInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import QuestTooltip from "./QuestTooltip.vue";

const props = withDefaults(defineProps<{
  reference: string;
  bordered?: boolean;
}>(), {
  bordered: false,
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

onMounted(loadData);

watch(() => props.reference, () => {
  questData.value = null;
  loadData();
});

function handleClick() {
  navigateToEntity({ type: "quest", id: questData.value?.internal_name ?? props.reference });
}
</script>
