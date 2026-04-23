<template>
  <div class="flex gap-2 items-start mb-2">
    <img
      v-if="iconSrc"
      :src="iconSrc"
      :alt="ability.name"
      class="w-8 h-8 object-contain bg-black/30 border border-border-light rounded shrink-0" />
    <div class="flex-1">
      <div class="font-bold text-entity-ability text-sm mb-0.5">{{ ability.name }}</div>
      <div class="flex gap-2 text-xs">
        <span v-if="ability.skill" class="text-entity-skill">{{ ability.skill }}</span>
        <span v-if="ability.level" class="text-text-muted">Lv {{ ability.level }}</span>
      </div>
    </div>
  </div>

  <div v-if="ability.description" class="text-text-secondary text-xs leading-relaxed mb-2 italic">
    {{ ability.description }}
  </div>

  <!-- Combat details -->
  <div v-if="hasCombatDetails" class="flex flex-wrap gap-x-3 gap-y-0.5 text-xs mb-2">
    <span v-if="ability.damage_type" class="text-red-400">{{ ability.damage_type }}</span>
    <span v-if="ability.target" class="text-text-muted">{{ ability.target }}</span>
    <span v-if="ability.range" class="text-text-muted">Range: {{ ability.range }}m</span>
    <span v-if="ability.reset_time" class="text-text-muted">CD: {{ ability.reset_time }}s</span>
  </div>

  <!-- Costs -->
  <div v-if="hasCosts" class="flex flex-wrap gap-x-3 gap-y-0.5 text-xs mb-2">
    <span v-if="ability.mana_cost" class="text-blue-400">{{ ability.mana_cost }} mana</span>
    <span v-if="ability.power_cost" class="text-yellow-400">{{ ability.power_cost }} power</span>
    <span v-if="ability.armor_cost" class="text-text-muted">{{ ability.armor_cost }} armor</span>
    <span v-if="ability.health_cost" class="text-red-400">{{ ability.health_cost }} health</span>
  </div>

  <div v-if="ability.keywords?.length" class="flex flex-wrap gap-1">
    <span
      v-for="keyword in ability.keywords"
      :key="keyword"
      class="bg-entity-ability/10 text-entity-ability px-1.5 py-0.5 rounded-sm text-[10px] uppercase tracking-wide"
    >
      {{ keyword }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { AbilityInfo } from "../../../types/gameData";

const props = defineProps<{
  ability: AbilityInfo;
  iconSrc: string | null;
}>();

const hasCombatDetails = computed(() =>
  props.ability.damage_type || props.ability.target || props.ability.range || props.ability.reset_time
);

const hasCosts = computed(() =>
  props.ability.mana_cost || props.ability.power_cost || props.ability.armor_cost || props.ability.health_cost
);
</script>
