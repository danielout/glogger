<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
    <div class="text-[0.65rem] text-text-muted uppercase tracking-wide mb-2">System Status</div>
    <div class="flex flex-col gap-2">
      <div class="flex items-center gap-2">
        <span
          class="w-2 h-2 rounded-full"
          :class="coordinator.isPlayerLogTailing ? 'bg-green-500' : 'bg-red-500'"
        />
        <span class="text-sm text-text-primary">Player.log</span>
        <span class="text-xs text-text-dim ml-auto">{{ coordinator.isPlayerLogTailing ? 'Tailing' : 'Stopped' }}</span>
      </div>
      <div class="flex items-center gap-2">
        <span
          class="w-2 h-2 rounded-full"
          :class="coordinator.isChatLogTailing ? 'bg-green-500' : 'bg-red-500'"
        />
        <span class="text-sm text-text-primary">Chat Log</span>
        <span class="text-xs text-text-dim ml-auto">{{ coordinator.isChatLogTailing ? 'Tailing' : 'Stopped' }}</span>
      </div>
      <div v-if="activeCharacter" class="flex items-center gap-2 mt-1 pt-1 border-t border-border-default">
        <span class="text-sm text-text-secondary">Character:</span>
        <span class="text-sm text-accent-gold font-bold">{{ activeCharacter }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useCoordinatorStore } from '../../stores/coordinatorStore'
import { useSettingsStore } from '../../stores/settingsStore'

const coordinator = useCoordinatorStore()
const settingsStore = useSettingsStore()
const activeCharacter = computed(() => settingsStore.settings.activeCharacterName)
</script>
