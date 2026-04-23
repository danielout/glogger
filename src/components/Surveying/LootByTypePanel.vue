<template>
  <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-3">
    <header class="flex items-baseline justify-between">
      <h3 class="text-[10px] uppercase tracking-widest text-text-secondary font-semibold">
        Breakdown by Survey Type
      </h3>
      <span v-if="bonusSummary" class="text-[10px] text-accent-gold">
        {{ bonusSummary }}
      </span>
    </header>

    <div v-if="typeGroups.length === 0" class="text-text-dim text-xs italic">
      No uses recorded yet.
    </div>

    <!-- Per-type accordion -->
    <div v-for="group in typeGroups" :key="group.displayName" class="flex flex-col gap-1">
      <button
        class="w-full flex items-center justify-between px-2 py-1.5 rounded bg-surface-elevated border border-border-default hover:bg-surface-elevated/80 transition-colors text-xs"
        @click="toggleType(group.displayName)"
      >
        <div class="flex items-center gap-2 min-w-0">
          <span class="text-text-dim text-[10px]">{{ expanded[group.displayName] ? '▼' : '▶' }}</span>
          <span class="text-text-primary font-semibold truncate">{{ group.displayName }}</span>
          <span class="text-[10px] text-text-dim">{{ group.kind }}</span>
        </div>
        <div class="flex items-center gap-3 shrink-0 tabular-nums">
          <span>
            <span class="text-text-primary font-semibold">{{ group.useCount }}</span>
            <span class="text-text-dim"> uses</span>
          </span>
          <span class="text-accent-gold font-semibold">{{ group.totalLoot }}</span>
          <span v-if="group.avgLoot > 0" class="text-text-dim">
            ~{{ group.avgLoot.toFixed(1) }}/use
          </span>
        </div>
      </button>

      <!-- Expanded: item breakdown -->
      <div v-if="expanded[group.displayName]" class="ml-4 flex flex-col gap-1">
        <!-- Primary items -->
        <div v-if="group.primaryItems.length > 0" class="flex flex-wrap gap-1.5 py-1">
          <div
            v-for="item in group.primaryItems"
            :key="`primary:${item.item_name}`"
            class="flex items-center gap-1 text-xs bg-surface-elevated rounded px-1.5 py-0.5 border border-border-default/50"
          >
            <ItemInline :reference="item.item_name" :show-icon="false" />
            <span class="text-text-primary font-semibold tabular-nums">&times;{{ item.total_qty }}</span>
          </div>
        </div>

        <!-- Bonus items -->
        <div v-if="group.bonusItems.length > 0" class="flex flex-wrap gap-1.5 py-1">
          <span class="text-[10px] text-accent-gold uppercase tracking-wider font-semibold self-center mr-1">
            Bonus
          </span>
          <div
            v-for="item in group.bonusItems"
            :key="`bonus:${item.item_name}`"
            class="flex items-center gap-1 text-xs bg-accent-gold/5 rounded px-1.5 py-0.5 border border-accent-gold/20"
          >
            <ItemInline :reference="item.item_name" :show-icon="false" />
            <span class="text-accent-gold font-semibold tabular-nums">&times;{{ item.bonus_qty }}</span>
          </div>
        </div>

        <div v-if="group.primaryItems.length === 0 && group.bonusItems.length === 0" class="text-text-dim italic text-xs py-1">
          No loot recorded.
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
// Loot breakdown by survey type for the center panel. Groups the
// session's uses by map_display_name, shows per-type use count +
// total loot + avg, and expands to per-item lists split by primary
// vs speed-bonus drops.
import { computed, reactive } from 'vue'
import type { SurveySessionDetail, LootSummaryRow, SurveyUse } from '../../stores/surveyTrackerStore'
import ItemInline from '../Shared/Item/ItemInline.vue'

const props = defineProps<{
  detail: SurveySessionDetail
}>()

interface TypeGroup {
  displayName: string
  kind: string
  useCount: number
  totalLoot: number
  avgLoot: number
  primaryItems: LootSummaryRow[]
  bonusItems: LootSummaryRow[]
}

const expanded = reactive<Record<string, boolean>>({})

function toggleType(name: string) {
  expanded[name] = !expanded[name]
}

const typeGroups = computed<TypeGroup[]>(() => {
  // Group uses by display name.
  const usesByType = new Map<string, { uses: SurveyUse[]; kind: string }>()
  for (const u of props.detail.uses) {
    const key = u.map_display_name
    if (!usesByType.has(key)) {
      usesByType.set(key, { uses: [], kind: u.kind })
    }
    usesByType.get(key)!.uses.push(u)
  }

  // For each type, partition the loot summary into primary vs bonus.
  // We don't have per-type loot breakdown from the backend (loot_summary
  // is session-wide), so we show the global items and note that the
  // breakdown is session-wide. Future: backend could return per-type items.
  return [...usesByType.entries()]
    .map(([displayName, { uses, kind }]) => {
      const useCount = uses.length
      const totalLoot = uses.reduce((sum, u) => sum + u.loot_qty, 0)
      const avgLoot = useCount > 0 ? totalLoot / useCount : 0
      return {
        displayName,
        kind,
        useCount,
        totalLoot,
        avgLoot,
        // Show the session-wide primary/bonus items since we don't
        // have per-type-attributed loot breakdown (yet).
        primaryItems: props.detail.loot_summary.filter(r => r.primary_qty > 0),
        bonusItems: props.detail.loot_summary.filter(r => r.bonus_qty > 0),
      }
    })
    .sort((a, b) => b.useCount - a.useCount)
})

const bonusSummary = computed(() => {
  const totalBonus = props.detail.loot_summary.reduce((sum, r) => sum + r.bonus_qty, 0)
  const totalPrimary = props.detail.loot_summary.reduce((sum, r) => sum + r.primary_qty, 0)
  if (totalBonus === 0) return null
  const total = totalPrimary + totalBonus
  const pct = ((totalBonus / total) * 100).toFixed(1)
  return `${totalBonus} bonus items (${pct}%)`
})
</script>
