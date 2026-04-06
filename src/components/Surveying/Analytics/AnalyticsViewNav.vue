<template>
  <div class="flex flex-col gap-0.5 p-2">
    <!-- Overview -->
    <button
      @click="$emit('update:selectedView', 'overview')"
      :class="[
        'text-left px-2 py-1.5 text-xs rounded transition-all',
        selectedView === 'overview'
          ? 'bg-[#1a1a2e] border-l-2 border-l-accent-gold text-text-primary font-semibold'
          : 'border-l-2 border-l-transparent text-text-secondary hover:text-text-primary hover:bg-[#1e1e1e]'
      ]"
    >Overview</button>

    <!-- Zones section -->
    <template v-if="zones.length > 0">
      <div class="text-[0.6rem] uppercase tracking-widest text-text-dim mt-3 mb-1 px-2 font-bold">
        Zones
      </div>
      <button
        v-for="zone in zones"
        :key="zone.zone"
        @click="$emit('update:selectedView', `zone:${zone.zone}`)"
        :class="[
          'text-left px-2 py-1.5 text-xs rounded transition-all flex items-center justify-between gap-2',
          selectedView === `zone:${zone.zone}`
            ? 'bg-[#1a1a2e] border-l-2 border-l-accent-gold text-text-primary font-semibold'
            : 'border-l-2 border-l-transparent text-text-secondary hover:text-text-primary hover:bg-[#1e1e1e]'
        ]"
      >
        <span class="truncate">{{ formatZone(zone.zone) }}</span>
        <span class="text-[0.6rem] text-text-dim font-mono shrink-0">{{ zoneSurveyCount(zone) }}</span>
      </button>
    </template>

    <!-- Survey Types section -->
    <template v-if="uniqueSurveyTypes.length > 0">
      <div class="text-[0.6rem] uppercase tracking-widest text-text-dim mt-3 mb-1 px-2 font-bold">
        Survey Types
      </div>
      <button
        v-for="st in uniqueSurveyTypes"
        :key="st.name"
        @click="$emit('update:selectedView', `surveytype:${st.name}`)"
        :class="[
          'text-left px-2 py-1 text-xs rounded transition-all flex items-center justify-between gap-2',
          selectedView === `surveytype:${st.name}`
            ? 'bg-[#1a1a2e] border-l-2 border-l-accent-gold text-text-primary font-semibold'
            : 'border-l-2 border-l-transparent text-text-secondary hover:text-text-primary hover:bg-[#1e1e1e]'
        ]"
      >
        <span class="truncate">{{ st.name }}</span>
        <span class="text-[0.6rem] font-mono shrink-0"
              :class="st.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
          {{ st.totalCompleted }}
        </span>
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { ZoneAnalytics } from "../../../types/database";

const props = defineProps<{
  zones: ZoneAnalytics[];
  selectedView: string;
}>();

defineEmits<{
  "update:selectedView": [value: string];
}>();

interface UniqueSurveyType {
  name: string;
  category: string;
  totalCompleted: number;
}

const uniqueSurveyTypes = computed<UniqueSurveyType[]>(() => {
  const map = new Map<string, UniqueSurveyType>();
  for (const zone of props.zones) {
    for (const st of zone.survey_type_stats) {
      const existing = map.get(st.survey_type);
      if (existing) {
        existing.totalCompleted += st.total_completed;
      } else {
        map.set(st.survey_type, {
          name: st.survey_type,
          category: st.category,
          totalCompleted: st.total_completed,
        });
      }
    }
  }
  return [...map.values()].sort((a, b) => b.totalCompleted - a.totalCompleted);
});

function zoneSurveyCount(zone: ZoneAnalytics): number {
  return zone.speed_bonus_stats.reduce((sum, c) => sum + c.total_surveys, 0);
}

function formatZone(zone: string): string {
  return zone.replace(/([a-z])([A-Z])/g, "$1 $2");
}
</script>
