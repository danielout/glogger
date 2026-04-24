<template>
  <div class="flex flex-col gap-2 text-sm h-full min-h-0">
    <!-- Stats bar -->
    <div class="flex items-center justify-between px-1 shrink-0">
      <div class="flex items-center gap-2">
        <span class="text-xs text-text-muted">Items studied:</span>
        <span class="text-xs font-mono text-accent-gold">{{ stats.total_studied }}</span>
      </div>
      <div v-if="cooldownRemaining > 0" class="flex items-center gap-1">
        <span class="text-[10px] text-text-dim">Cooldown:</span>
        <span class="text-xs font-mono text-amber-400">{{ formatCooldown(cooldownRemaining) }}</span>
      </div>
      <div v-else-if="stats.last_studied_at" class="flex items-center gap-1">
        <span class="text-[10px] text-green-400">Ready to study</span>
      </div>
    </div>

    <!-- Search -->
    <input
      v-model="search"
      type="text"
      placeholder="Search studied items..."
      class="w-full px-2 py-1 rounded bg-surface-2 border border-border text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-blue shrink-0" />

    <!-- Study list -->
    <div class="flex-1 overflow-y-auto min-h-0">
      <div v-if="filteredStudies.length === 0 && studies.length > 0" class="text-xs text-text-dim italic py-2">
        No items match your search.
      </div>
      <div v-else-if="studies.length === 0" class="text-xs text-text-dim italic py-2">
        No items studied yet. Study equipment in-game to start tracking.
      </div>
      <div v-else class="flex flex-col gap-0.5">
        <div
          v-for="study in filteredStudies"
          :key="study.id"
          class="flex items-center justify-between gap-2 py-0.5 px-2 rounded hover:bg-surface-elevated/50">
          <ItemInline :reference="study.item_name" class="text-xs truncate min-w-0" />
          <span class="text-[10px] text-text-dim whitespace-nowrap shrink-0">
            {{ formatStudyDate(study.studied_at) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../../stores/settingsStore'
import ItemInline from '../../Shared/Item/ItemInline.vue'

interface HoplologyStudy {
  id: number
  character_name: string
  server_name: string
  item_name: string
  studied_at: string
  source: string
}

interface HoplologyStats {
  total_studied: number
  last_studied_at: string | null
  last_studied_item: string | null
}

const COOLDOWN_MS = 5 * 60 * 1000 // 5 minutes

const settings = useSettingsStore()
const studies = ref<HoplologyStudy[]>([])
const stats = ref<HoplologyStats>({ total_studied: 0, last_studied_at: null, last_studied_item: null })
const search = ref('')
const now = ref(Date.now())

let refreshInterval: ReturnType<typeof setInterval> | null = null
let unlisten: UnlistenFn | null = null
let unlistenStatus: UnlistenFn | null = null

const cooldownRemaining = computed(() => {
  if (!stats.value.last_studied_at) return 0
  const lastStudied = new Date(stats.value.last_studied_at).getTime()
  const remaining = Math.ceil((lastStudied + COOLDOWN_MS - now.value) / 1000)
  return Math.max(0, remaining)
})

const filteredStudies = computed(() => {
  if (!search.value.trim()) return studies.value
  const q = search.value.toLowerCase()
  return studies.value.filter(s => s.item_name.toLowerCase().includes(q))
})

function formatCooldown(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatStudyDate(dateStr: string): string {
  try {
    const d = new Date(dateStr)
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
  } catch {
    return dateStr
  }
}

async function loadData() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  try {
    const [studyList, studyStats] = await Promise.all([
      invoke<HoplologyStudy[]>('get_hoplology_studies', {
        characterName: char,
        serverName: server,
      }),
      invoke<HoplologyStats>('get_hoplology_stats', {
        characterName: char,
        serverName: server,
      }),
    ])
    studies.value = studyList
    stats.value = studyStats
  } catch (e) {
    console.error('Failed to load hoplology data:', e)
  }
}

onMounted(async () => {
  await loadData()

  // Refresh the cooldown timer every second while active, else every 30s
  refreshInterval = setInterval(() => {
    now.value = Date.now()
  }, cooldownRemaining.value > 0 ? 1000 : 30_000)

  unlisten = await listen<string[]>('game-state-updated', (event) => {
    if (event.payload.includes('hoplology')) {
      loadData()
    }
  })

  // Also listen for real-time study events from chat
  unlistenStatus = await listen('chat-status-event', (event: any) => {
    if (event.payload?.kind === 'ItemStudied') {
      // Refresh data and reset cooldown timer
      loadData()
      now.value = Date.now()
      // Switch to 1-second interval for cooldown display
      if (refreshInterval) clearInterval(refreshInterval)
      refreshInterval = setInterval(() => {
        now.value = Date.now()
        // Switch back to 30s interval once cooldown expires
        if (cooldownRemaining.value <= 0 && refreshInterval) {
          clearInterval(refreshInterval)
          refreshInterval = setInterval(() => { now.value = Date.now() }, 30_000)
        }
      }, 1000)
    }
  })
})

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
  if (unlisten) unlisten()
  if (unlistenStatus) unlistenStatus()
})
</script>
