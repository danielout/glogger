<template>
  <div class="flex flex-col gap-3">
    <div v-if="loading" class="text-text-dim text-sm italic">Loading gift data...</div>

    <template v-else>
      <!-- Progress header -->
      <div class="flex items-center justify-between text-sm">
        <span>
          <span class="text-accent-gold font-bold">{{ totalGiftsGiven }}</span>
          <span class="text-text-muted"> / {{ totalGiftsMax }} gifts</span>
        </span>
        <span class="text-xs text-text-dim">{{ resetLabel }}</span>
      </div>

      <!-- Progress bar -->
      <div class="h-1.5 bg-surface-elevated rounded-full overflow-hidden">
        <div
          class="h-full bg-accent-gold rounded-full transition-all duration-300"
          :style="{ width: progressPct + '%' }" />
      </div>

      <!-- Summary counts -->
      <div class="text-xs text-text-muted">
        <span class="text-accent-green">{{ maxedCount }} maxed</span>
        <span class="mx-1">·</span>
        <span>{{ remainingCount }} remaining</span>
      </div>

      <!-- NPCs still needing gifts -->
      <div v-if="needsGifts.length > 0" class="flex flex-col gap-1.5">
        <div
          v-for="status in needsGifts"
          :key="status.npc.key"
          class="flex items-center justify-between gap-2 text-sm">
          <NpcInline :reference="status.npc.key" :npc="status.npc" />
          <span class="text-xs font-mono shrink-0 tracking-wide">
            <span
              v-for="i in status.maxGifts"
              :key="i"
              :class="i <= status.giftsThisWeek ? 'text-accent-gold' : 'text-text-dim'">●</span>
          </span>
        </div>
      </div>

      <div v-else-if="totalGiftsMax > 0" class="text-xs text-accent-green italic">
        All NPCs maxed this week!
      </div>

      <div v-else class="text-xs text-text-dim italic">
        No Statehelm NPCs tracked yet.
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useStatehelmTracker } from '../../../composables/useStatehelmTracker'
import NpcInline from '../../Shared/NPC/NpcInline.vue'

const {
  npcStatuses,
  totalGiftsGiven,
  totalGiftsMax,
  loading,
  loadGiftLog,
  weekStart,
} = useStatehelmTracker()

onMounted(() => loadGiftLog())

const progressPct = computed(() => {
  if (totalGiftsMax.value === 0) return 0
  return Math.round((totalGiftsGiven.value / totalGiftsMax.value) * 100)
})

const maxedCount = computed(() =>
  npcStatuses.value.filter(s => s.giftsThisWeek >= s.maxGifts).length
)

const remainingCount = computed(() =>
  npcStatuses.value.filter(s => s.giftsThisWeek < s.maxGifts).length
)

/** NPCs that still need gifts, sorted by fewest gifts first, capped at 5 */
const needsGifts = computed(() =>
  npcStatuses.value
    .filter(s => s.giftsThisWeek < s.maxGifts)
    .sort((a, b) => a.giftsThisWeek - b.giftsThisWeek)
    .slice(0, 5)
)

/** Time until weekly reset (Monday 00:00 UTC) */
const resetLabel = computed(() => {
  const resetTime = new Date(weekStart.value.getTime() + 7 * 24 * 60 * 60 * 1000)
  const now = new Date()
  const diff = resetTime.getTime() - now.getTime()
  if (diff <= 0) return 'Resetting...'

  const days = Math.floor(diff / (24 * 60 * 60 * 1000))
  const hours = Math.floor((diff % (24 * 60 * 60 * 1000)) / (60 * 60 * 1000))

  if (days > 0) return `Resets in ${days}d ${hours}h`
  return `Resets in ${hours}h`
})
</script>
