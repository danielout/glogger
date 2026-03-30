<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <div class="flex items-center gap-3">
      <div class="flex-1 min-w-0">
        <BuildHeader />
      </div>
      <button
        v-if="store.activePreset"
        class="shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded text-xs font-semibold cursor-pointer transition-colors"
        :class="showSummary
          ? 'bg-accent-gold/20 text-accent-gold border border-accent-gold/30'
          : 'bg-surface-elevated text-text-muted border border-border-default hover:text-text-primary hover:border-accent-gold/30'"
        @click="showSummary = !showSummary">
        Build Summary
      </button>
    </div>

    <!-- No preset selected -->
    <EmptyState
      v-if="!store.activePreset"
      primary="No build selected"
      secondary="Create a new build or select an existing one above." />

    <!-- Two-panel layout -->
    <div v-else class="flex gap-3 flex-1 min-h-0">
      <!-- Left panel: slot list + ability bars -->
      <div class="w-80 shrink-0 flex flex-col gap-3 min-h-0 overflow-y-auto">
        <SlotGrid />
        <AbilityBarSummary />
      </div>

      <!-- Right panel: mod browser or ability bar editor -->
      <div class="flex-1 min-h-0">
        <SlotModPicker v-if="store.selectedSlot" />
        <AbilityBarEditor v-else-if="store.activeBar" />
        <div v-else class="flex items-center justify-center h-full text-text-muted text-sm">
          Select an equipment slot or ability bar to start planning
        </div>
      </div>
    </div>

    <!-- Build Summary slide-out panel -->
    <BuildSummary v-model="showSummary" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import EmptyState from '../../Shared/EmptyState.vue'
import BuildHeader from './BuildHeader.vue'
import SlotGrid from './SlotGrid.vue'
import SlotModPicker from './SlotModPicker.vue'
import AbilityBarEditor from './AbilityBarEditor.vue'
import AbilityBarSummary from './AbilityBarSummary.vue'
import BuildSummary from './BuildSummary.vue'

const store = useBuildPlannerStore()
const showSummary = ref(false)

onMounted(async () => {
  await Promise.all([
    store.loadCombatSkills(),
    store.loadPresets(),
  ])
  // Auto-select the first preset if available
  if (!store.activePreset && store.presets.length > 0) {
    await store.selectPreset(store.presets[0])
  }
})
</script>
