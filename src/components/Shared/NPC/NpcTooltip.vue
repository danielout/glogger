<template>
  <div class="mb-2">
    <div class="font-bold text-entity-npc text-sm mb-0.5">{{ npc.name }}</div>
    <div v-if="npc.area_friendly_name" class="text-entity-area text-xs">{{ npc.area_friendly_name }}</div>
  </div>

  <div v-if="npc.desc" class="text-text-secondary text-xs leading-relaxed mb-2 italic">
    {{ npc.desc }}
  </div>

  <div v-if="npc.trains_skills?.length" class="mb-2">
    <div class="text-text-muted text-[0.65rem] uppercase tracking-wide mb-1">Trains</div>
    <div class="flex flex-wrap gap-1">
      <span
        v-for="skill in npc.trains_skills"
        :key="skill"
        class="bg-entity-skill/10 text-entity-skill px-1.5 py-0.5 rounded-sm text-[0.65rem]"
      >
        {{ skill }}
      </span>
    </div>
  </div>

  <div v-if="topPrefs.length" class="mb-1">
    <div class="text-text-muted text-[0.65rem] uppercase tracking-wide mb-1">Preferences</div>
    <div v-for="pref in topPrefs" :key="pref.name ?? pref.desire" class="text-xs flex gap-1.5 py-0.5">
      <span :class="desireColor(pref.desire)">{{ pref.desire }}</span>
      <span class="text-text-secondary">{{ pref.name ?? pref.keywords?.join(', ') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { NpcInfo } from "../../../types/gameData";

const props = defineProps<{
  npc: NpcInfo;
}>();

const topPrefs = computed(() =>
  props.npc.preferences?.slice(0, 5) ?? []
);

function desireColor(desire: string): string {
  switch (desire.toLowerCase()) {
    case "love": return "text-accent-red";
    case "like": return "text-accent-green";
    case "hate": return "text-accent-red";
    default: return "text-text-muted";
  }
}
</script>
