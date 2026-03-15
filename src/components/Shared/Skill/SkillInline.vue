<template>
  <EntityTooltipWrapper
    border-class="border-entity-skill/50"
    @hover="loadData"
  >
    <button
      class="inline-flex items-center gap-1 cursor-pointer hover:underline"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon && skillData?.icon_id" :icon-id="skillData.icon_id" :alt="name" size="xs" />
      <span class="text-entity-skill text-xs font-medium">{{ skillData?.name ?? name }}</span>
    </button>
    <template #tooltip>
      <SkillTooltip v-if="skillData" :skill="skillData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { SkillInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import SkillTooltip from "./SkillTooltip.vue";

const props = withDefaults(defineProps<{
  name: string;
  showIcon?: boolean;
}>(), {
  showIcon: true,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const skillData = ref<SkillInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadData() {
  if (skillData.value) return;
  try {
    const skill = await store.getSkillByName(props.name);
    if (!skill) return;
    skillData.value = skill;
    if (skill.icon_id) {
      const path = await store.getIconPath(skill.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to load skill: ${props.name}`, e);
  }
}

function handleClick() {
  navigateToEntity({ type: "skill", id: props.name });
}
</script>
