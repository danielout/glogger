<template>
  <div>
    <div v-if="!areaName" class="text-text-dim text-sm italic">
      No area data yet.
    </div>

    <template v-else>
      <div class="mb-3">
        <AreaInline :reference="areaName" />
      </div>

      <!-- NPCs in this area -->
      <div v-if="loading" class="text-text-dim text-xs italic">Loading NPCs…</div>
      <div v-else-if="areaNpcs.length === 0" class="text-text-dim text-xs italic">No friendly NPCs in this area.</div>
      <div v-else class="flex flex-col gap-1.5">
        <div
          v-for="npc in areaNpcs"
          :key="npc.key"
          class="flex items-center justify-between gap-2 text-sm">
          <NpcInline :reference="npc.key" :npc="npc" />
          <span
            v-if="getFavorTier(npc.key)"
            class="text-xs px-1.5 py-0.5 rounded border shrink-0"
            :class="favorBadgeClasses(getFavorTier(npc.key)!)">
            {{ tierDisplayName(getFavorTier(npc.key)!) }}
          </span>
          <span v-else class="text-text-dim text-xs italic shrink-0">No favor data</span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import { favorBadgeClasses, tierDisplayName } from '../../composables/useFavorTiers'
import AreaInline from '../Shared/Area/AreaInline.vue'
import NpcInline from '../Shared/NPC/NpcInline.vue'
import type { NpcInfo } from '../../types/gameData'

const gameState = useGameStateStore()
const gameData = useGameDataStore()

const areaName = computed(() => gameState.world.area?.area_name ?? null)

const areaNpcs = ref<NpcInfo[]>([])
const loading = ref(false)

function getFavorTier(npcKey: string): string | null {
  return gameState.favorByNpc[npcKey]?.favor_tier ?? null
}

watch(areaName, async (name) => {
  if (!name) {
    areaNpcs.value = []
    return
  }
  loading.value = true
  try {
    areaNpcs.value = await gameData.getNpcsInArea(name)
  } catch (e) {
    console.error('[CurrentZone] Failed to load NPCs for area:', e)
    areaNpcs.value = []
  } finally {
    loading.value = false
  }
}, { immediate: true })
</script>
