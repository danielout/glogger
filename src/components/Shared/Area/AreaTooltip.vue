<template>
  <div>
    <div class="mb-1.5">
      <div class="font-bold text-entity-area text-sm">{{ areaName }}</div>
      <div v-if="shortName && shortName !== areaName" class="text-text-muted text-xs">
        {{ shortName }}
      </div>
    </div>

    <!-- NPCs in this area -->
    <div v-if="areaNpcs.length" class="mb-1.5">
      <div class="text-text-muted text-[10px] uppercase tracking-wide mb-1">Notable NPCs</div>
      <div class="flex flex-wrap gap-x-1.5 gap-y-0.5 text-xs">
        <template v-for="(npc, idx) in displayedNpcs" :key="npc.key">
          <NpcInline :reference="npc.key" :npc="npc" /><span v-if="idx < displayedNpcs.length - 1" class="text-text-dim">,</span>
        </template>
      </div>
      <div v-if="areaNpcs.length > maxNpcs" class="text-text-dim text-[10px] mt-0.5">
        +{{ areaNpcs.length - maxNpcs }} more
      </div>
    </div>

    <!-- Monsters in this area -->
    <div v-if="areaMonsters.length" class="mb-1.5">
      <div class="text-text-muted text-[10px] uppercase tracking-wide mb-1">Monsters</div>
      <div class="flex flex-wrap gap-x-1.5 gap-y-0.5 text-xs">
        <template v-for="(name, idx) in displayedMonsters" :key="name">
          <EnemyInline :reference="name" /><span v-if="idx < displayedMonsters.length - 1" class="text-text-dim">,</span>
        </template>
      </div>
      <div v-if="areaMonsters.length > maxMonsters" class="text-text-dim text-[10px] mt-0.5">
        +{{ areaMonsters.length - maxMonsters }} more
      </div>
    </div>

    <!-- Area key (subtle) -->
    <div class="text-text-dim text-[10px] mt-1">
      {{ areaKey }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGameDataStore } from '../../../stores/gameDataStore'
import type { NpcInfo } from '../../../types/gameData'
import NpcInline from '../NPC/NpcInline.vue'
import EnemyInline from '../Enemy/EnemyInline.vue'

const props = defineProps<{
  areaKey: string
  areaName: string
  shortName: string | null
}>()

const maxNpcs = 8
const maxMonsters = 12

const gameData = useGameDataStore()

// Find NPCs located in this area by matching area_name (CDN key)
const areaNpcs = computed<NpcInfo[]>(() => {
  const allNpcs = Object.values(gameData.npcsByKey)
  return allNpcs
    .filter(npc => npc.area_name === props.areaKey)
    .sort((a, b) => a.name.localeCompare(b.name))
})

const displayedNpcs = computed(() => areaNpcs.value.slice(0, maxNpcs))

// Monsters in this area (loaded from translation strings)
const areaMonsters = ref<string[]>([])

watchEffect(async () => {
  try {
    areaMonsters.value = await invoke<string[]>('get_monsters_in_area', { area: props.areaKey })
  } catch {
    areaMonsters.value = []
  }
})

const displayedMonsters = computed(() => areaMonsters.value.slice(0, maxMonsters))
</script>
