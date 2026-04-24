<template>
  <div class="flex flex-col gap-3 min-h-0 h-full">
    <div class="flex items-center justify-between">
      <h3 class="section-heading">
        Character Report Stats
      </h3>
      <span v-if="lastUpdated" class="text-xs text-text-dim font-mono">
        Last updated {{ formatTimestamp(lastUpdated) }}
      </span>
    </div>

    <div v-if="loading" class="text-xs text-text-muted italic">Loading...</div>

    <div v-else-if="stats.length === 0" class="text-xs text-text-dim italic">
      No report stats yet. Open your behavior report or age report in-game to populate.
    </div>

    <template v-else>
      <div class="flex items-center gap-3 mb-1 shrink-0">
        <input
          v-model="filter"
          type="text"
          placeholder="Filter stats..."
          class="px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-32" />
        <span class="text-xs text-text-muted">{{ filteredStats.length }} stats</span>
      </div>

      <div class="flex-1 overflow-y-auto min-h-0">
      <div v-for="group in groupedFiltered" :key="group.category" class="mb-3">
        <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-1 px-2">
          {{ formatCategory(group.category) }}
        </h4>
        <table class="w-full text-sm border-collapse">
          <tbody>
            <tr
              v-for="stat in group.stats"
              :key="stat.stat_name"
              class="border-b border-border-default/30 hover:bg-surface-elevated/50">
              <td class="py-0.5 px-2 text-text-primary">{{ formatStatName(stat.stat_name) }}</td>
              <td class="py-0.5 px-2 text-right text-accent-gold font-mono">{{ formatValue(stat.stat_value) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../stores/settingsStore'
import { formatDateTimeFull as formatTimestamp } from '../../composables/useTimestamp'

interface ReportStat {
  category: string
  stat_name: string
  stat_value: string
  updated_at: string
}

interface StatGroup {
  category: string
  stats: ReportStat[]
}

const CATEGORY_ORDER = ['age', 'killing_stats', 'challenges', 'food_stats', 'misc_stats', 'badges']

const settings = useSettingsStore()
const stats = ref<ReportStat[]>([])
const loading = ref(false)
const filter = ref('')

let unlisten: UnlistenFn | null = null

const lastUpdated = computed(() => {
  if (stats.value.length === 0) return null
  return stats.value.reduce((latest, s) =>
    s.updated_at > latest ? s.updated_at : latest, stats.value[0].updated_at)
})

const filteredStats = computed(() => {
  const f = filter.value.toLowerCase()
  if (!f) return stats.value
  return stats.value.filter(s =>
    s.stat_name.toLowerCase().includes(f) ||
    s.category.toLowerCase().includes(f) ||
    s.stat_value.toLowerCase().includes(f)
  )
})

const groupedFiltered = computed<StatGroup[]>(() => {
  const groups = new Map<string, ReportStat[]>()
  for (const s of filteredStats.value) {
    const list = groups.get(s.category) || []
    list.push(s)
    groups.set(s.category, list)
  }

  const result: StatGroup[] = []
  // Ordered categories first
  for (const cat of CATEGORY_ORDER) {
    const g = groups.get(cat)
    if (g) {
      result.push({ category: cat, stats: g })
      groups.delete(cat)
    }
  }
  // Any remaining categories
  for (const [cat, g] of groups) {
    result.push({ category: cat, stats: g })
  }
  return result
})

function formatCategory(cat: string): string {
  return cat.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function formatStatName(name: string): string {
  return name.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function formatValue(value: string): string {
  // If it's a pure number, format with locale separators
  const num = Number(value)
  if (!isNaN(num) && value.trim() !== '') {
    return num.toLocaleString()
  }
  // Booleans
  if (value === 'yes') return 'Yes'
  if (value === 'no') return 'No'
  if (value === 'true') return '\u2713' // checkmark for badges
  return value
}

async function loadStats() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  loading.value = true
  try {
    stats.value = await invoke<ReportStat[]>('get_character_report_stats', {
      characterName: char,
      serverName: server,
    })
  } catch (e) {
    console.error('Failed to load report stats:', e)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await loadStats()
  unlisten = await listen<string[]>('game-state-updated', (event) => {
    if (event.payload.includes('report_stats')) {
      loadStats()
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>
