<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">
        Computed Stats
      </h3>
      <button
        class="px-3 py-1 text-xs bg-accent-gold/15 border border-accent-gold/30 text-accent-gold rounded cursor-pointer transition-all hover:bg-accent-gold/25 disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="loading"
        @click="refresh">
        {{ loading ? 'Computing...' : 'Refresh' }}
      </button>
    </div>

    <div v-if="loading" class="text-xs text-text-muted italic">Computing stats...</div>

    <div v-else-if="!stats" class="text-xs text-text-dim italic">
      No game state data yet. Skills, recipes, or report data will appear here.
    </div>

    <div v-else class="flex-1 overflow-y-auto min-h-0">
      <table class="w-full text-sm border-collapse">
        <tbody>
          <!-- Skill Totals -->
          <tr class="border-b border-border-default/30">
            <td colspan="2" class="py-1 px-2 text-xs font-semibold text-text-muted uppercase tracking-wider">Skill Totals</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Total Level</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.total_level.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Base Levels</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.total_base_level.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Bonus Levels</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.total_bonus_levels.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Skills Known</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.skill_count.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Total XP Earned</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.total_xp_earned.toLocaleString() }}</td>
          </tr>

          <!-- Rate Stats (only if time_played available) -->
          <template v-if="stats.hours_played != null">
            <tr class="border-b border-border-default/30">
              <td colspan="2" class="py-1 px-2 text-xs font-semibold text-text-muted uppercase tracking-wider pt-2">Rates</td>
            </tr>
            <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
              <td class="py-0.5 px-2 text-text-primary">Time Played</td>
              <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ formatHoursPlayed(stats.hours_played) }}</td>
            </tr>
            <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
              <td class="py-0.5 px-2 text-text-primary">XP / Hour</td>
              <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ formatRate(stats.xp_per_hour) }}</td>
            </tr>
            <tr v-if="stats.kills_per_hour != null" class="border-b border-border-default/30 hover:bg-surface-elevated/50">
              <td class="py-0.5 px-2 text-text-primary">Kills / Hour</td>
              <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ formatRate(stats.kills_per_hour) }}</td>
            </tr>
            <tr v-if="stats.deaths_per_hour != null" class="border-b border-border-default/30 hover:bg-surface-elevated/50">
              <td class="py-0.5 px-2 text-text-primary">Deaths / Hour</td>
              <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ formatRate(stats.deaths_per_hour) }}</td>
            </tr>
          </template>
          <template v-else>
            <tr class="border-b border-border-default/30">
              <td colspan="2" class="py-1 px-2 text-xs text-text-dim italic">Open /age in-game for rate stats</td>
            </tr>
          </template>

          <!-- Crafting Activity -->
          <tr class="border-b border-border-default/30">
            <td colspan="2" class="py-1 px-2 text-xs font-semibold text-text-muted uppercase tracking-wider pt-2">Crafting Activity</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Items Crafted</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.items_crafted.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Items Distilled</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.items_distilled.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Items Deconstructed</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.items_deconstructed.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Times Teleported</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.times_teleported.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Items Dyed</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ stats.items_dyed.toLocaleString() }}</td>
          </tr>
          <tr class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">Time Watching Bars Fill</td>
            <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ formatDuration(stats.total_crafting_seconds) }}</td>
          </tr>

          <!-- Per-skill Crafting Breakdown -->
          <template v-if="stats.crafting_by_skill.length > 0">
            <tr class="border-b border-border-default/30">
              <td colspan="2" class="py-1 px-2 text-xs font-semibold text-text-muted uppercase tracking-wider pt-2">Crafting By Skill</td>
            </tr>
            <tr
              v-for="entry in stats.crafting_by_skill"
              :key="entry.skill_name"
              class="border-b border-border-default/30 hover:bg-surface-elevated/50">
              <td class="py-0.5 px-2 text-text-primary">{{ entry.skill_name }}</td>
              <td class="py-0.5 px-2 text-right text-accent-gold font-mono">
                {{ entry.total_crafted.toLocaleString() }}
                <span v-if="entry.crafting_seconds > 0" class="text-text-dim text-xs ml-1">({{ formatDuration(entry.crafting_seconds) }})</span>
              </td>
            </tr>
          </template>

        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../stores/settingsStore'

interface SkillCraftingBreakdown {
  skill_name: string
  total_crafted: number
  crafting_seconds: number
}

interface ComputedStats {
  total_level: number
  total_base_level: number
  total_bonus_levels: number
  skill_count: number
  total_xp_earned: number
  items_crafted: number
  items_distilled: number
  items_deconstructed: number
  times_teleported: number
  items_dyed: number
  total_crafting_seconds: number
  hours_played: number | null
  xp_per_hour: number | null
  kills_per_hour: number | null
  deaths_per_hour: number | null
  crafting_by_skill: SkillCraftingBreakdown[]
}

const settings = useSettingsStore()
const stats = ref<ComputedStats | null>(null)
const loading = ref(false)

let unlisten: UnlistenFn | null = null

function formatDuration(totalSeconds: number): string {
  if (totalSeconds <= 0) return '0s'
  const days = Math.floor(totalSeconds / 86400)
  const hours = Math.floor((totalSeconds % 86400) / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = Math.floor(totalSeconds % 60)

  const parts: string[] = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0) parts.push(`${minutes}m`)
  if (seconds > 0 || parts.length === 0) parts.push(`${seconds}s`)
  return parts.join(' ')
}

function formatHoursPlayed(hours: number | null): string {
  if (hours == null) return '—'
  const days = Math.floor(hours / 24)
  const h = Math.floor(hours % 24)
  if (days > 0) return `${days}d ${h}h`
  return `${h}h`
}

function formatRate(value: number | null): string {
  if (value == null) return '—'
  return value.toLocaleString(undefined, { maximumFractionDigits: 1 })
}

async function refresh() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  loading.value = true
  try {
    stats.value = await invoke<ComputedStats>('get_computed_stats', {
      characterName: char,
      serverName: server,
    })
  } catch (e) {
    console.error('Failed to compute stats:', e)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await refresh()
  unlisten = await listen<string[]>('game-state-updated', (event) => {
    if (event.payload.includes('skills') || event.payload.includes('recipes') || event.payload.includes('report_stats')) {
      refresh()
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>
