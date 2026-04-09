<template>
  <EntityTooltipWrapper
    border-class="border-entity-ability/50"
    entity-type="ability"
    :entity-reference="resolvedAbility?.name ?? reference ?? ''"
    :entity-label="displayName"
    @hover="loadData"
  >
    <span
      class="inline-flex items-center gap-0.5 cursor-pointer hover:underline text-entity-ability font-medium"
      :class="bordered ? 'bg-entity-ability/5 border border-entity-ability/20 rounded px-1 py-0.5' : ''"
      @click="handleClick"
    >
      <GameIcon v-if="showIcon && resolvedAbility?.icon_id" :icon-id="resolvedAbility.icon_id" :alt="displayName" size="inline" />
      <span>{{ displayName }}</span>
    </span>
    <template #tooltip>
      <AbilityTooltip v-if="resolvedAbility" :ability="resolvedAbility" :icon-src="iconSrc" />
    </template>
  </EntityTooltipWrapper>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useGameDataStore } from "../../../stores/gameDataStore";
import { useEntityNavigation } from "../../../composables/useEntityNavigation";
import type { AbilityInfo } from "../../../types/gameData";
import EntityTooltipWrapper from "../EntityTooltipWrapper.vue";
import GameIcon from "../GameIcon.vue";
import AbilityTooltip from "./AbilityTooltip.vue";

const props = withDefaults(defineProps<{
  /** Pre-resolved ability object */
  ability?: AbilityInfo;
  /** String reference (display name or numeric ID) — resolved on hover */
  reference?: string;
  showIcon?: boolean;
  bordered?: boolean;
}>(), {
  showIcon: true,
  bordered: false,
});

const store = useGameDataStore();
const { navigateToEntity } = useEntityNavigation();

const resolvedAbility = ref<AbilityInfo | null>(props.ability ?? null);
const iconSrc = ref<string | null>(null);

const displayName = computed(() => resolvedAbility.value?.name ?? props.reference ?? "Unknown");

async function loadData() {
  // If we already have the full ability, just load the icon
  if (resolvedAbility.value) {
    await loadIcon();
    return;
  }
  // Resolve from reference
  if (props.reference) {
    try {
      const ability = await store.resolveAbility(props.reference);
      if (ability) {
        resolvedAbility.value = ability;
        await loadIcon();
      }
    } catch (e) {
      console.warn(`Failed to resolve ability: ${props.reference}`, e);
    }
  }
}

async function loadIcon() {
  if (iconSrc.value || !resolvedAbility.value?.icon_id) return;
  try {
    const path = await store.getIconPath(resolvedAbility.value.icon_id);
    iconSrc.value = convertFileSrc(path);
  } catch (e) {
    console.warn(`Icon load failed for ability: ${displayName.value}`, e);
  }
}

onMounted(() => {
  if (props.ability) loadIcon();
});

watch(() => props.reference, () => {
  if (!props.ability) {
    resolvedAbility.value = null;
    iconSrc.value = null;
  }
});

watch(() => props.ability, (newVal) => {
  if (newVal) {
    resolvedAbility.value = newVal;
    iconSrc.value = null;
    loadIcon();
  }
});

function handleClick() {
  if (resolvedAbility.value) {
    navigateToEntity({ type: "ability", id: resolvedAbility.value.id });
  }
}
</script>
