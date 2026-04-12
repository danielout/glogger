<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <div class="flex items-center gap-3">
      <BuildCompleteness v-if="store.activePreset" />
    </div>

    <!-- No preset selected -->
    <div v-if="!store.activePreset" class="flex flex-col items-center justify-center h-full gap-3">
      <EmptyState
        primary="No build selected"
        secondary="Create a new build or select an existing one above." />
      <button
        v-if="store.presets.length === 0"
        class="px-3 py-1.5 text-sm bg-accent-gold/20 border border-accent-gold/40 text-accent-gold rounded cursor-pointer hover:bg-accent-gold/30"
        @click="showCreate = true">
        + New Build
      </button>
    </div>

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

    <ModalDialog
      :show="showCreate"
      title="New Build"
      placeholder="Build name"
      confirm-label="Create"
      @update:show="showCreate = $event"
      @confirm="handleCreate" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import EmptyState from '../../Shared/EmptyState.vue'
import ModalDialog from '../../Shared/ModalDialog.vue'
import PaneLayout from '../../Shared/PaneLayout.vue'
import PaperDollLayout from './PaperDollLayout.vue'
import SlotDetailPanel from './SlotDetailPanel.vue'
import BuildCompleteness from './BuildCompleteness.vue'
import BuildSummary from './BuildSummary.vue'
import GlobalModSearch from './GlobalModSearch.vue'

const store = useBuildPlannerStore()
const showCreate = ref(false)

async function handleCreate(name: string) {
  if (!name) return
  await store.createPreset(name)
}

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
