<template>
  <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2 h-full">
    <header class="shrink-0">
      <h3 class="text-[0.65rem] uppercase tracking-widest text-text-secondary font-semibold">
        Loot Distribution
      </h3>
    </header>

    <div v-if="dataset.length === 0" class="flex-1 flex items-center justify-center text-text-dim text-xs italic">
      No loot data.
    </div>

    <div v-else class="flex-1 min-h-0">
      <VueUiDonut :dataset="dataset" :config="config" />
    </div>
  </section>
</template>

<script setup lang="ts">
// Donut chart of loot distribution for the session center panel.
// Items under 3% of total share are bucketed into "Other" to keep
// the chart readable. Config matches the app's existing SkillsTab
// donut pattern.
import { computed } from 'vue'
import { VueUiDonut } from 'vue-data-ui'
import type { VueUiDonutConfig, VueUiDonutDatasetItem } from 'vue-data-ui'
import type { LootSummaryRow } from '../../stores/surveyTrackerStore'

const props = defineProps<{
  rows: LootSummaryRow[]
}>()

const chartPalette = [
  '#7ec8e3', '#c87e7e', '#6366f1', '#f59e0b', '#10b981',
  '#ef4444', '#8b5cf6', '#ec4899', '#14b8a6', '#f97316',
]

const dataset = computed<VueUiDonutDatasetItem[]>(() => {
  if (props.rows.length === 0) return []
  const total = props.rows.reduce((sum, r) => sum + r.total_qty, 0)
  if (total === 0) return []
  const threshold = total * 0.03

  const result: VueUiDonutDatasetItem[] = []
  let otherQty = 0
  let colorIdx = 0

  // Sort by quantity descending so the biggest slices get the first colors.
  const sorted = [...props.rows].sort((a, b) => b.total_qty - a.total_qty)

  for (const row of sorted) {
    if (row.total_qty >= threshold) {
      result.push({
        name: row.item_name,
        color: chartPalette[colorIdx % chartPalette.length],
        values: [row.total_qty],
      })
      colorIdx++
    } else {
      otherQty += row.total_qty
    }
  }

  if (otherQty > 0) {
    result.push({
      name: 'Other',
      color: '#52525b',
      values: [otherQty],
    })
  }

  return result
})

const config = computed<VueUiDonutConfig>(() => ({
  responsive: true,
  useCssAnimation: true,
  useBlurOnHover: false,
  style: {
    fontFamily: 'inherit',
    chart: {
      backgroundColor: 'transparent',
      color: '#a1a1aa',
      layout: {
        labels: {
          dataLabels: {
            show: true,
            hideUnderValue: 3,
          },
          percentage: {
            show: true,
            color: '#a1a1aa',
            bold: true,
            fontSize: 10,
            rounding: 1,
          },
          name: {
            show: true,
            color: '#d4d4d8',
            bold: false,
            fontSize: 10,
          },
          value: {
            show: false,
          },
          hollow: {
            show: true,
            total: {
              show: true,
              bold: true,
              fontSize: 14,
              color: '#d4d4d8',
              text: 'Items',
              value: {
                color: '#e4e4e7',
                fontSize: 16,
                bold: true,
                rounding: 0,
              },
            },
            average: { show: false },
          },
        },
        donut: {
          strokeWidth: 48,
          borderWidth: 1,
          useShadow: false,
        },
      },
      legend: {
        show: false,
      },
      title: {
        text: '',
      },
      tooltip: {
        show: true,
        showValue: true,
        showPercentage: true,
        roundingValue: 0,
        roundingPercentage: 1,
        backgroundColor: '#27272a',
        color: '#d4d4d8',
        borderColor: '#3f3f46',
        borderWidth: 1,
        borderRadius: 4,
        fontSize: 12,
      },
    },
  },
  userOptions: { show: false },
  table: { show: false },
}))
</script>
