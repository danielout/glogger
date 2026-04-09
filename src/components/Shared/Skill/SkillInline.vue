<template>
  <EntityTooltipWrapper
    border-class="border-entity-skill/50"
    entity-type="skill"
    :entity-reference="reference"
    :entity-label="skillData?.name ?? reference"
    @hover="loadData"
  >
    <span
      class="inline-flex items-center gap-0.5 cursor-pointer hover:underline text-entity-skill font-medium"
      :class="bordered ? 'bg-entity-skill/5 border border-entity-skill/20 rounded px-1 py-0.5' : ''"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon && skillData?.icon_id" :icon-id="skillData.icon_id" :alt="reference" size="inline" />
      <span>{{ skillData?.name ?? reference }}</span>
    </span>
    <template #tooltip>
      <SkillTooltip v-if="skillData" :skill="skillData" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { SkillInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import SkillTooltip from "./SkillTooltip.vue";

const props = withDefaults(defineProps<{
  reference: string;
  showIcon?: boolean;
  bordered?: boolean;
}>(), {
  showIcon: true,
  bordered: false,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const skillData = ref<SkillInfo | null>(null);
const iconSrc = ref<string | null>(null);

async function loadData() {
  if (skillData.value) return;
  try {
    const skill = await store.resolveSkill(props.reference);
    if (!skill) return;
    skillData.value = skill;
    if (skill.icon_id) {
      const path = await store.getIconPath(skill.icon_id);
      iconSrc.value = convertFileSrc(path);
    }
  } catch (e) {
    console.warn(`Failed to resolve skill: ${props.reference}`, e);
  }
}

onMounted(loadData);

watch(() => props.reference, () => {
  skillData.value = null;
  iconSrc.value = null;
  loadData();
});

function handleClick() {
  navigateToEntity({ type: "skill", id: skillData.value?.name ?? props.reference });
}
</script>
