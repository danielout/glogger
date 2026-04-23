<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <div class="flex items-center justify-between shrink-0">
      <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Player Attributes</h3>
      <FilterBar
        v-model="filter"
        placeholder="Filter..."
        :result-count="filtered.length" />
    </div>

    <div v-if="loading" class="text-xs text-text-muted italic">Loading...</div>

    <div v-else-if="attributes.length === 0" class="text-xs text-text-dim italic">
      No attribute data yet. Attributes populate when you enter a zone in-game.
    </div>

    <div v-else class="flex-1 overflow-y-auto min-h-0">
      <DataTable
        :columns="attrColumns"
        :rows="filtered as unknown as Record<string, unknown>[]"
        :sticky-header="true"
        compact
        empty-text="No matching attributes">
        <template #cell-attribute_name="{ row }">
          {{ formatAttrName((row as unknown as AttributeExtreme).attribute_name) }}
        </template>
        <template #cell-current_value="{ row }">
          <span class="text-accent-gold font-mono">{{ fmtVal((row as unknown as AttributeExtreme).current_value) }}</span>
        </template>
        <template #cell-min_value="{ row }">
          <span class="font-mono" :class="(row as unknown as AttributeExtreme).min_value < (row as unknown as AttributeExtreme).current_value ? 'text-blue-400' : 'text-text-dim'">
            {{ fmtVal((row as unknown as AttributeExtreme).min_value) }}
          </span>
        </template>
        <template #cell-max_value="{ row }">
          <span class="font-mono" :class="(row as unknown as AttributeExtreme).max_value > (row as unknown as AttributeExtreme).current_value ? 'text-value-positive' : 'text-text-dim'">
            {{ fmtVal((row as unknown as AttributeExtreme).max_value) }}
          </span>
        </template>
      </DataTable>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettingsStore } from '../../stores/settingsStore'
import DataTable, { type ColumnDef } from '../Shared/DataTable.vue'
import FilterBar from '../Shared/FilterBar.vue'

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

const attrColumns: ColumnDef[] = [
  { key: 'attribute_name', label: 'Attribute' },
  { key: 'current_value', label: 'Current', numeric: true },
  { key: 'min_value', label: 'Min', numeric: true },
  { key: 'max_value', label: 'Max', numeric: true },
]

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
