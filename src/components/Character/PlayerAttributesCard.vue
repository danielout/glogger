<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Player Attributes</h3>
      <div class="flex items-center gap-2">
        <input
          v-model="filter"
          type="text"
          placeholder="Filter..."
          class="px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-24" />
        <span class="text-xs text-text-muted">{{ filtered.length }}</span>
      </div>
    </div>

    <div v-if="loading" class="text-xs text-text-muted italic">Loading...</div>

    <div v-else-if="attributes.length === 0" class="text-xs text-text-dim italic">
      No attribute data yet. Attributes populate when you enter a zone in-game.
    </div>

    <div v-else class="flex-1 overflow-y-auto min-h-0">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-elevated">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1 px-2 text-xs">Attribute</th>
            <th class="py-1 px-2 text-right text-xs">Current</th>
            <th class="py-1 px-2 text-right text-xs">Min</th>
            <th class="py-1 px-2 text-right text-xs">Max</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="attr in filtered"
            :key="attr.attribute_name"
            class="border-b border-border-default/30 hover:bg-surface-elevated/50">
            <td class="py-0.5 px-2 text-text-primary">{{ formatAttrName(attr.attribute_name) }}</td>
            <td class="py-0.5 px-2 text-right text-accent-gold tabular-nums">{{ fmtVal(attr.current_value) }}</td>
            <td class="py-0.5 px-2 text-right" :class="attr.min_value < attr.current_value ? 'text-blue-400' : 'text-text-dim'">
              {{ fmtVal(attr.min_value) }}
            </td>
            <td class="py-0.5 px-2 text-right" :class="attr.max_value > attr.current_value ? 'text-green-400' : 'text-text-dim'">
              {{ fmtVal(attr.max_value) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../stores/settingsStore'

interface AttributeExtreme {
  attribute_name: string
  current_value: number
  min_value: number
  max_value: number
}

const settings = useSettingsStore()
const attributes = ref<AttributeExtreme[]>([])
const loading = ref(false)
const filter = ref('')

let unlisten: UnlistenFn | null = null

const filtered = computed(() => {
  const f = filter.value.toLowerCase()
  if (!f) return attributes.value
  return attributes.value.filter(a =>
    a.attribute_name.toLowerCase().includes(f)
  )
})

function formatAttrName(name: string): string {
  return name.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function fmtVal(value: number): string {
  return Number.isInteger(value) ? value.toLocaleString() : value.toLocaleString(undefined, { maximumFractionDigits: 1 })
}

async function loadAttributes() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  loading.value = true
  try {
    attributes.value = await invoke<AttributeExtreme[]>('get_attribute_extremes', {
      characterName: char,
      serverName: server,
    })
  } catch (e) {
    console.error('Failed to load attribute extremes:', e)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await loadAttributes()
  unlisten = await listen<string[]>('game-state-updated', (event) => {
    if (event.payload.includes('attributes')) {
      loadAttributes()
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>
