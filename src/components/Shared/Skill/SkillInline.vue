<template>
  <EntityTooltipWrapper
    border-class="border-entity-skill/50"
    @hover="loadData"
  >
    <component
      :is="plain ? 'span' : 'button'"
      :class="plain
        ? 'hover:underline cursor-pointer text-inherit'
        : 'inline-flex items-center gap-1 cursor-pointer hover:underline'"
      @click="handleClick"
    >
      <GameIcon v-if="!plain && showIcon && skillData?.icon_id" :icon-id="skillData.icon_id" :alt="reference" size="xs" />
      <span :class="plain ? '' : 'text-entity-skill text-xs font-medium'">{{ skillData?.name ?? reference }}</span>
    </component>
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
  reference: string;
  showIcon?: boolean;
  plain?: boolean;
}>(), {
  showIcon: true,
  plain: false,
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

function handleClick() {
  navigateToEntity({ type: "skill", id: skillData.value?.name ?? props.reference });
}
</script>
