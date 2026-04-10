<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <div class="flex items-center gap-3">
      <div class="flex-1 min-w-0">
        <BuildHeader />
      </div>
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
      :left-pane="{ title: 'Equipment', defaultWidth: 320, minWidth: 260, maxWidth: 450 }"
      :right-pane="{ title: 'Build Summary', defaultWidth: 380, minWidth: 280, maxWidth: 600, defaultCollapsed: true }">
      <template #left>
        <div class="flex flex-col h-full min-h-0">
          <!-- Tab switcher -->
          <div class="flex border-b border-border-default shrink-0">
            <button
              class="flex-1 px-3 py-1.5 text-xs font-semibold cursor-pointer transition-colors"
              :class="leftTab === 'equipment'
                ? 'text-accent-gold border-b-2 border-accent-gold'
                : 'text-text-muted hover:text-text-secondary'"
              @click="leftTab = 'equipment'">
              Equipment
            </button>
            <button
              class="flex-1 px-3 py-1.5 text-xs font-semibold cursor-pointer transition-colors"
              :class="leftTab === 'abilities'
                ? 'text-accent-gold border-b-2 border-accent-gold'
                : 'text-text-muted hover:text-text-secondary'"
              @click="leftTab = 'abilities'">
              Abilities
            </button>
          </div>

          <!-- Tab content -->
          <div class="flex-1 overflow-y-auto pt-2">
            <SlotGrid v-if="leftTab === 'equipment'" />
            <AbilityBarSummary v-else />
          </div>
        </div>
      </template>

      <!-- Center: mod browser, ability bar editor, or global mod search -->
      <div class="flex flex-col h-full min-h-0">
        <SlotModPicker v-if="store.selectedSlot" />
        <AbilityBarEditor v-else-if="store.activeBar" />
        <GlobalModSearch v-else-if="store.presetMods.length > 0" />
        <div v-else class="flex items-center justify-center h-full text-text-muted text-sm">
          Select an equipment slot or ability bar to start planning
        </div>
      </div>

      <template #right>
        <BuildSummary />
      </template>
    </PaneLayout>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import EmptyState from '../../Shared/EmptyState.vue'
import PaneLayout from '../../Shared/PaneLayout.vue'
import BuildHeader from './BuildHeader.vue'
import SlotGrid from './SlotGrid.vue'
import SlotModPicker from './SlotModPicker.vue'
import AbilityBarEditor from './AbilityBarEditor.vue'
import AbilityBarSummary from './AbilityBarSummary.vue'
import BuildCompleteness from './BuildCompleteness.vue'
import BuildSummary from './BuildSummary.vue'
import GlobalModSearch from './GlobalModSearch.vue'

const store = useBuildPlannerStore()
const leftTab = ref<'equipment' | 'abilities'>('equipment')

// Auto-switch tab when user clicks a slot or ability bar
watch(() => store.activeBar, (bar) => {
  if (bar) leftTab.value = 'abilities'
})
watch(() => store.selectedSlot, (slot) => {
  if (slot) leftTab.value = 'equipment'
})

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
