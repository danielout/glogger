<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <div class="flex items-center gap-3">
      <BuildCompleteness v-if="store.activePreset" />
    </div>

    <!-- No preset selected -->
    <EmptyState
      v-if="!store.activePreset"
      primary="No build selected"
      secondary="Create a new build or select an existing one above." />

    <!-- Three-panel PaneLayout -->
    <PaneLayout
      v-else
      screen-key="build-planner"
      :left-pane="{ title: 'Equipment', fixedWidth: 280 }"
      :right-pane="{ title: 'Build Summary', defaultWidth: 550, minWidth: 300, maxWidth: 900 }">
      <template #left>
        <PaperDollLayout />
      </template>

      <!-- Center: mod browser or global mod search -->
      <div class="flex flex-col h-full min-h-0">
        <SlotDetailPanel v-if="store.selectedSlot" />
        <GlobalModSearch v-else-if="store.presetMods.length > 0" />
        <div v-else class="flex items-center justify-center h-full text-text-muted text-sm">
          Select an equipment slot to start planning
        </div>
      </div>

      <template #right>
        <BuildSummary />
      </template>
    </PaneLayout>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import EmptyState from '../../Shared/EmptyState.vue'
import PaneLayout from '../../Shared/PaneLayout.vue'
import PaperDollLayout from './PaperDollLayout.vue'
import SlotDetailPanel from './SlotDetailPanel.vue'
import BuildCompleteness from './BuildCompleteness.vue'
import BuildSummary from './BuildSummary.vue'
import GlobalModSearch from './GlobalModSearch.vue'

const store = useBuildPlannerStore()

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
