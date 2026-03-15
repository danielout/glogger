<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center gap-3">
      <input
        v-model="filter"
        type="text"
        placeholder="Filter NPCs..."
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-48" />
      <span class="text-xs text-text-muted">{{ filtered.length }} NPCs</span>
    </div>

    <div class="overflow-auto max-h-[60vh]">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-base">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1.5 px-2">NPC</th>
            <th class="py-1.5 px-2">Favor Level</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="npc in filtered"
            :key="npc.npc_key"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2 text-text-primary">{{ formatNpcKey(npc.npc_key) }}</td>
            <td class="py-1 px-2" :class="favorColor(npc.favor_level)">{{ npc.favor_level }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotNpcFavor } from '../../types/database'

const props = defineProps<{
  favor: SnapshotNpcFavor[]
}>()

const filter = ref('')

const filtered = computed(() => {
  const f = filter.value.toLowerCase()
  return f
    ? props.favor.filter(n => n.npc_key.toLowerCase().includes(f) || n.favor_level.toLowerCase().includes(f))
    : props.favor
})

function formatNpcKey(key: string): string {
  return key.replace(/^NPC_/, '').replace(/^Cow_/, '').replace(/_/g, ' ')
}

const favorColors: Record<string, string> = {
  SoulMates: 'text-purple-400',
  LikeFamily: 'text-pink-400',
  BestFriends: 'text-blue-400',
  CloseFriends: 'text-cyan-400',
  Friends: 'text-green-400',
  Comfortable: 'text-yellow-400',
  Neutral: 'text-text-muted',
}

function favorColor(level: string): string {
  return favorColors[level] || 'text-text-secondary'
}
</script>
