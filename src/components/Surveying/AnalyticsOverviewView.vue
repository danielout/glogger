<template>
  <div class="flex flex-col gap-4">
    <!-- Top stat cards -->
    <div class="grid grid-cols-2 xl:grid-cols-4 gap-2">
      <StatCard
        label="Total Surveys"
        :value="analytics.total_uses.toLocaleString()"
        :sub="`${analytics.total_basic_uses.toLocaleString()} basic · ${motherlodeCount} motherlode · ${multihitCount} multihit`"
      />
      <StatCard
        label="Speed Bonus Rate"
        :value="bonusRateDisplay"
        :sub="bonusRateSub"
      />
      <StatCard
        label="Bonus Items"
        :value="analytics.bonus_items_total.toLocaleString()"
        sub="total bonus drops"
      />
      <StatCard
        label="Zones"
        :value="String(analytics.zones.length)"
        :sub="`${analytics.survey_types.length} survey types`"
      />
    </div>

    <!-- Cross-zone comparison -->
    <section
      v-if="analytics.zones.length > 0"
      class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2"
    >
      <header class="flex items-baseline justify-between">
        <h3 class="text-xs uppercase tracking-widest text-accent-blue font-semibold">
          Cross-Zone Comparison
        </h3>
        <span class="text-[10px] text-text-dim">click a zone to drill in</span>
      </header>
      <CrossZoneComparison :zones="analytics.zones" @select-zone="emitZone" />
    </section>

    <!-- All survey types -->
    <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2">
      <header class="flex items-baseline justify-between">
        <h3 class="text-xs uppercase tracking-widest text-accent-blue font-semibold">
          All Survey Types
        </h3>
        <span class="text-[10px] text-text-dim">sorted by most-used · click to drill in</span>
      </header>
      <table class="w-full text-xs border-collapse">
        <thead>
          <tr class="text-[10px] uppercase tracking-wide text-text-secondary font-semibold border-b border-border-default">
            <th class="text-left py-1.5 px-2">Survey Type</th>
            <th class="text-left py-1.5 px-2">Zone</th>
            <th class="text-left py-1.5 px-2">Kind</th>
            <th class="text-right py-1.5 px-2">Done</th>
            <th class="text-right py-1.5 px-2">Items</th>
            <th class="text-right py-1.5 px-2">Avg/Use</th>
            <th class="text-right py-1.5 px-2">Bonus Rate</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="t in analytics.survey_types"
            :key="`${t.map_internal_name}::${t.area ?? ''}`"
            class="border-b border-border-default/40 hover:bg-surface-elevated/50 cursor-pointer"
            @click="$emit('select-type', { map: t.map_internal_name, area: t.area })"
          >
            <td class="py-1 px-2 text-text-primary font-semibold">{{ t.map_display_name }}</td>
            <td class="py-1 px-2 text-text-secondary">
              <AreaInline v-if="t.area" :reference="t.area" />
              <span v-else class="text-text-dim">—</span>
            </td>
            <td class="py-1 px-2 text-text-secondary">{{ t.kind }}</td>
            <td class="text-right py-1 px-2 text-text-primary tabular-nums">{{ t.total_uses }}</td>
            <td class="text-right py-1 px-2 text-accent-gold font-semibold tabular-nums">
              {{ t.total_loot_qty.toLocaleString() }}
            </td>
            <td class="text-right py-1 px-2 text-text-muted tabular-nums">
              {{ t.avg_loot_per_use !== null ? t.avg_loot_per_use.toFixed(1) : '—' }}
            </td>
            <td class="text-right py-1 px-2 tabular-nums" :class="typeBonusRateClass(t)">
              {{ typeBonusRate(t) }}
            </td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<script setup lang="ts">
// Overview view shown when no zone or type is selected. Top metric
// cards + cross-zone comparison + all-survey-types table. Rows in
// both tables are clickable to drill into a zone or type detail view.
import { computed } from 'vue'
import type { SurveyAnalytics, SurveyTypeSummary } from '../../stores/surveyTrackerStore'
import StatCard from './StatCard.vue'
import CrossZoneComparison from './CrossZoneComparison.vue'
import AreaInline from '../Shared/Area/AreaInline.vue'

const props = defineProps<{
  analytics: SurveyAnalytics
}>()

const emit = defineEmits<{
  'select-zone': [area: string]
  'select-type': [payload: { map: string; area: string | null }]
}>()

function emitZone(area: string) {
  emit('select-zone', area)
}

const motherlodeCount = computed(() =>
  props.analytics.zones.reduce((sum, z) => sum + z.motherlode_uses, 0).toLocaleString(),
)
const multihitCount = computed(() =>
  props.analytics.zones.reduce((sum, z) => sum + z.multihit_uses, 0).toLocaleString(),
)

const bonusRateDisplay = computed(() => {
  const a = props.analytics
  if (a.total_basic_uses === 0) return '—'
  return `${((a.basic_uses_with_bonus / a.total_basic_uses) * 100).toFixed(1)}%`
})
const bonusRateSub = computed(() => {
  const a = props.analytics
  if (a.total_basic_uses === 0) return 'no basic surveys yet'
  return `${a.basic_uses_with_bonus} of ${a.total_basic_uses} basic`
})

function typeBonusRate(t: SurveyTypeSummary): string {
  if (t.kind !== 'basic' || t.total_uses === 0) return '—'
  return `${((t.uses_with_bonus / t.total_uses) * 100).toFixed(1)}%`
}
function typeBonusRateClass(t: SurveyTypeSummary): string {
  if (t.kind !== 'basic' || t.total_uses === 0) return 'text-text-dim'
  return 'text-accent-gold'
}
</script>
