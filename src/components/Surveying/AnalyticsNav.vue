<template>
  <nav class="flex flex-col gap-0.5 p-2 text-xs overflow-y-auto h-full">
    <!-- Overview -->
    <button
      class="text-left px-2 py-1.5 rounded border-l-2 transition-colors"
      :class="
        view.kind === 'overview'
          ? 'border-l-accent-gold bg-surface-elevated text-text-primary font-semibold'
          : 'border-l-transparent text-text-secondary hover:text-text-primary hover:bg-surface-elevated/50'
      "
      @click="$emit('update:view', { kind: 'overview' })"
    >
      Overview
    </button>

    <!-- Zones -->
    <template v-if="zones.length > 0">
      <div class="text-[10px] uppercase tracking-widest text-text-dim mt-3 mb-1 px-2 font-semibold">
        Zones
      </div>
      <button
        v-for="z in zones"
        :key="`zone:${z.area}`"
        class="text-left px-2 py-1.5 rounded border-l-2 transition-colors flex items-center justify-between gap-2"
        :class="
          view.kind === 'zone' && view.area === z.area
            ? 'border-l-accent-gold bg-surface-elevated text-text-primary font-semibold'
            : 'border-l-transparent text-text-secondary hover:text-text-primary hover:bg-surface-elevated/50'
        "
        @click="$emit('update:view', { kind: 'zone', area: z.area })"
      >
        <span class="truncate"><AreaInline :reference="z.area" /></span>
        <span class="text-[10px] text-text-dim tabular-nums shrink-0">{{ z.total_uses }}</span>
      </button>
    </template>

    <!-- Survey Types -->
    <template v-if="surveyTypes.length > 0">
      <div class="text-[10px] uppercase tracking-widest text-text-dim mt-3 mb-1 px-2 font-semibold">
        Survey Types
      </div>
      <button
        v-for="t in surveyTypes"
        :key="`type:${t.map_internal_name}::${t.area ?? ''}`"
        class="text-left px-2 py-1 rounded border-l-2 transition-colors flex items-center justify-between gap-2"
        :class="
          view.kind === 'type' && view.map === t.map_internal_name && view.area === t.area
            ? 'border-l-accent-gold bg-surface-elevated text-text-primary font-semibold'
            : 'border-l-transparent text-text-secondary hover:text-text-primary hover:bg-surface-elevated/50'
        "
        @click="$emit('update:view', { kind: 'type', map: t.map_internal_name, area: t.area })"
      >
        <span class="truncate">{{ t.map_display_name }}</span>
        <span
          class="text-[10px] tabular-nums shrink-0"
          :class="t.kind === 'basic' ? 'text-accent-blue' : 'text-text-dim'"
        >
          {{ t.total_uses }}
        </span>
      </button>
    </template>
  </nav>
</template>

<script setup lang="ts">
// Sidebar navigation for the Analytics tab — Overview / Zones / Survey
// Types. The parent owns the active view as a typed union; this
// component just renders + emits selection events.
import type { ZoneSummary, SurveyTypeSummary } from '../../stores/surveyTrackerStore'
import AreaInline from '../Shared/Area/AreaInline.vue'

export type AnalyticsView =
  | { kind: 'overview' }
  | { kind: 'zone'; area: string }
  | { kind: 'type'; map: string; area: string | null }

defineProps<{
  zones: ZoneSummary[]
  surveyTypes: SurveyTypeSummary[]
  view: AnalyticsView
}>()

defineEmits<{
  'update:view': [view: AnalyticsView]
}>()
</script>
