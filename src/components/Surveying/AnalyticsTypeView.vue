<template>
  <div class="flex flex-col gap-4">
    <!-- Top stats -->
    <div class="grid grid-cols-2 xl:grid-cols-4 gap-2">
      <StatCard label="Survey Type" :value="type.map_display_name" />
      <StatCard
        label="Completed"
        :value="type.total_uses.toLocaleString()"
        :sub="`${type.total_loot_qty.toLocaleString()} items attributed`"
      />
      <StatCard
        label="Avg Yield / Use"
        :value="type.avg_loot_per_use !== null ? type.avg_loot_per_use.toFixed(2) : '—'"
        :sub="type.kind"
      />
      <StatCard
        label="Bonus Rate"
        :value="bonusRate"
        :sub="bonusRateSub"
      />
    </div>

    <!-- Items table -->
    <ZoneRewardsCard
      :map-display-name="type.map_display_name"
      :category="category"
      :total-completions="type.total_uses"
      :items="type.items"
    />

    <!-- Same map in other zones, if any -->
    <section v-if="otherZoneMatches.length > 0" class="flex flex-col gap-2">
      <header class="flex items-baseline gap-2">
        <h3 class="text-xs uppercase tracking-widest text-accent-blue font-semibold">
          Also in Other Zones
        </h3>
        <span class="text-[0.6rem] text-text-dim">same map, different area</span>
      </header>
      <ZoneRewardsCard
        v-for="t in otherZoneMatches"
        :key="`other:${t.map_internal_name}::${t.area ?? ''}`"
        :map-display-name="`${t.map_display_name} — ${t.area ?? '(unknown)'}`"
        :category="category"
        :total-completions="t.total_uses"
        :items="t.items"
      />
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SurveyAnalytics, SurveyTypeSummary } from '../../stores/surveyTrackerStore'
import StatCard from './StatCard.vue'
import ZoneRewardsCard from './ZoneRewardsCard.vue'

const props = defineProps<{
  type: SurveyTypeSummary
  analytics: SurveyAnalytics
}>()

const category = computed(() =>
  props.type.map_internal_name.startsWith('MiningSurvey') ? 'mining' : 'mineral',
)

const bonusRate = computed(() => {
  if (props.type.kind !== 'basic' || props.type.total_uses === 0) return '—'
  return `${((props.type.uses_with_bonus / props.type.total_uses) * 100).toFixed(1)}%`
})
const bonusRateSub = computed(() => {
  if (props.type.kind !== 'basic' || props.type.total_uses === 0) return 'basic-only stat'
  return `${props.type.uses_with_bonus} of ${props.type.total_uses} basic`
})

// Same map_internal_name in a different area — shown so users can
// compare, e.g. the same motherlode map in Ilmari vs Gazluk.
const otherZoneMatches = computed(() =>
  props.analytics.survey_types.filter(
    t => t.map_internal_name === props.type.map_internal_name && t.area !== props.type.area,
  ),
)
</script>
