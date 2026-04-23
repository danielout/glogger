<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg p-4">
    <div class="text-[10px] text-text-muted uppercase tracking-wide mb-2">System Status</div>
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
        <EntityTooltipWrapper :delay="300" border-class="border-border-default">
          <span class="text-sm text-accent-gold font-bold cursor-help">{{ activeCharacter }}</span>
          <template #tooltip>
            <div class="flex flex-col gap-1 p-1 text-xs">
              <div class="text-text-muted font-semibold mb-0.5">Last Imports</div>
              <div class="flex justify-between gap-4">
                <span class="text-text-secondary">Character:</span>
                <span class="text-text-primary">{{ lastCharacterImport }}</span>
              </div>
              <div class="flex justify-between gap-4">
                <span class="text-text-secondary">Inventory:</span>
                <span class="text-text-primary">{{ lastInventoryImport }}</span>
              </div>
            </div>
          </template>
        </EntityTooltipWrapper>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useCoordinatorStore } from '../../stores/coordinatorStore'
import { useSettingsStore } from '../../stores/settingsStore'
import { useCharacterStore } from '../../stores/characterStore'
import EntityTooltipWrapper from '../Shared/EntityTooltipWrapper.vue'

const coordinator = useCoordinatorStore()
const settingsStore = useSettingsStore()
const characterStore = useCharacterStore()
const activeCharacter = computed(() => settingsStore.settings.activeCharacterName)

function formatDate(dateStr: string | undefined): string {
  if (!dateStr) return 'Never'
  const d = new Date(dateStr)
  if (isNaN(d.getTime())) return dateStr
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit' })
}

const lastCharacterImport = computed(() =>
  formatDate(characterStore.snapshots[0]?.import_date),
)
const lastInventoryImport = computed(() =>
  formatDate(characterStore.inventorySnapshots[0]?.import_date),
)
</script>
