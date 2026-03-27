<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center gap-3">
      <input
        v-model="filter"
        type="text"
        placeholder="Filter NPCs..."
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-48" />
      <label class="flex items-center gap-1.5 text-xs text-text-muted cursor-pointer select-none">
        <input type="checkbox" v-model="hideNeutral" class="cursor-pointer" />
        Hide Neutral
      </label>
      <span class="text-xs text-text-muted">{{ filtered.length }} NPCs</span>
      <span v-if="gameStateFavorCount > 0" class="text-xs text-text-secondary ml-2">
        {{ gameStateFavorCount }} in game state
      </span>
    </div>

    <div class="overflow-auto max-h-[60vh]">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-base">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1.5 px-2">NPC</th>
            <th class="py-1.5 px-2">Snapshot</th>
            <th class="py-1.5 px-2">Game State</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="npc in filtered"
            :key="npc.npc_key"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2">
              <NpcInline :reference="npc.npc_key" />
            </td>
            <td class="py-1 px-2" :class="favorColor(npc.favor_level)">{{ npc.favor_level }}</td>
            <td class="py-1 px-2">
              <template v-if="gameStateFavorMap[npc.npc_key]?.favor_tier">
                <span :class="favorColor(gameStateFavorMap[npc.npc_key].favor_tier!)">
                  {{ gameStateFavorMap[npc.npc_key].favor_tier }}
                </span>
                <span v-if="gameStateFavorMap[npc.npc_key].favor_tier !== npc.favor_level" class="text-accent-gold text-[0.65rem] ml-1">
                  mismatch
                </span>
              </template>
              <span v-else class="text-text-muted">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import type { SnapshotNpcFavor } from '../../types/database'
import NpcInline from '../Shared/NPC/NpcInline.vue'

const props = defineProps<{
  favor: SnapshotNpcFavor[]
}>()

const gameState = useGameStateStore()
const filter = ref('')
const hideNeutral = ref(false)

const gameStateFavorMap = computed(() => gameState.favorByNpc)
const gameStateFavorCount = computed(() => gameState.favor.length)

const filtered = computed(() => {
  let list = props.favor
  if (hideNeutral.value) {
    list = list.filter(n => n.favor_level !== 'Neutral')
  }
  const f = filter.value.toLowerCase()
  if (f) {
    list = list.filter(n => n.npc_key.toLowerCase().includes(f) || n.favor_level.toLowerCase().includes(f))
  }
  return list
})

const favorColors: Record<string, string> = {
  SoulMates: 'text-purple-400',
  LikeFamily: 'text-pink-400',
  BestFriends: 'text-blue-400',
  CloseFriends: 'text-cyan-400',
  Friends: 'text-green-400',
  Comfortable: 'text-yellow-400',
  Neutral: 'text-text-muted',
  Despised: 'text-red-400',
}

function favorColor(level: string): string {
  return favorColors[level] || 'text-text-secondary'
}
</script>
