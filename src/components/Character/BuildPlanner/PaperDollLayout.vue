<template>
  <div class="flex flex-col h-full min-h-0 gap-3 px-2 py-2">
    <!-- Build selector row -->
    <div class="flex items-center gap-2 shrink-0">
      <StyledSelect
        :model-value="String(store.activePreset?.id ?? '')"
        :options="presetOptions"
        placeholder="Select a build..."
        size="sm"
        class="flex-1 min-w-0"
        @update:model-value="onPresetChange" />
      <button
        class="px-2 py-1 text-xs bg-accent-gold/20 border border-accent-gold/40 text-accent-gold rounded cursor-pointer hover:bg-accent-gold/30 shrink-0"
        @click="showCreate = true">
        + New
      </button>
      <button
        v-if="store.activePreset"
        class="px-1.5 py-1 text-xs bg-surface-elevated border border-border-default text-text-secondary rounded cursor-pointer hover:bg-surface-hover shrink-0"
        @click="showRename = true">
        Rename
      </button>
      <button
        v-if="store.activePreset"
        class="px-1.5 py-1 text-xs bg-surface-elevated border border-border-default text-text-secondary rounded cursor-pointer hover:bg-surface-hover shrink-0"
        @click="showClone = true">
        Clone
      </button>
      <button
        v-if="store.activePreset"
        class="px-1.5 py-1 text-xs bg-red-900/20 border border-red-700/40 text-red-400 rounded cursor-pointer hover:bg-red-900/30 shrink-0"
        @click="showDelete = true">
        Del
      </button>
    </div>

    <!-- Import/Export row -->
    <div class="flex items-center gap-2 shrink-0">
      <button
        v-if="store.activePreset"
        class="px-2 py-1 text-xs bg-surface-elevated border border-border-default text-text-secondary rounded cursor-pointer hover:bg-surface-hover"
        @click="handleExport">
        Export
      </button>
      <button
        class="px-2 py-1 text-xs bg-surface-elevated border border-border-default text-text-secondary rounded cursor-pointer hover:bg-surface-hover"
        @click="showImport = true">
        Import
      </button>
      <span v-if="exportMessage" class="text-xs text-accent-gold">{{ exportMessage }}</span>
      <span v-if="importError" class="text-xs text-red-400">{{ importError }}</span>
    </div>

    <!-- Set Defaults (collapsible) -->
    <div v-if="store.activePreset" class="shrink-0">
      <button
        class="flex items-center gap-1 text-[10px] font-semibold text-text-muted uppercase tracking-wider cursor-pointer hover:text-text-secondary w-full"
        @click="showDefaults = !showDefaults">
        <span class="transition-transform" :class="showDefaults ? 'rotate-90' : ''">&#9654;</span>
        Set Defaults
      </button>
      <div v-if="showDefaults" class="flex flex-col gap-1.5 mt-1 pl-3">
        <!-- Rarity + Level -->
        <div class="flex items-center gap-2">
          <label class="text-[10px] text-text-muted">Rarity</label>
          <StyledSelect
            :model-value="store.activePreset.target_rarity"
            :options="rarityOptions"
            size="xs"
            @update:model-value="onRarityChange" />
          <label class="text-[10px] text-text-muted ml-1">Lv</label>
          <input
            type="number"
            :value="store.activePreset.target_level"
            min="1"
            max="125"
            class="bg-surface-elevated border border-border-default rounded px-1.5 py-0.5 text-xs text-text-primary w-12 text-center"
            @change="onLevelChange" />
        </div>
      </div>
    </div>

    <!-- Paper Doll Grid -->
    <div v-if="store.activePreset" class="shrink-0">
      <div class="grid grid-cols-[auto_1fr_auto] gap-x-2 gap-y-1.5 items-start">
        <!-- Left column: Armor slots -->
        <div class="flex flex-col gap-1.5 items-center">
          <PaperDollSlot v-for="slot in leftSlots" :key="slot.id" :slot="slot" />
        </div>

        <!-- Center: Stats -->
        <PaperDollStats />

        <!-- Right column: Non-armor slots -->
        <div class="flex flex-col gap-1.5 items-center">
          <PaperDollSlot v-for="slot in rightSlots" :key="slot.id" :slot="slot" />
        </div>
      </div>
    </div>

    <!-- Ability bars (always visible below equipment) -->
    <div v-if="store.activePreset" class="flex-1 min-h-0 overflow-y-auto border-t border-border-default pt-2">
      <AbilityBarSummary />
    </div>

    <!-- Dialogs -->
    <ModalDialog
      :show="showCreate"
      title="New Build"
      placeholder="Build name"
      confirm-label="Create"
      @update:show="showCreate = $event"
      @confirm="handleCreate" />

    <ModalDialog
      :show="showRename"
      title="Rename Build"
      placeholder="Build name"
      :initial-value="store.activePreset?.name ?? ''"
      confirm-label="Rename"
      @update:show="showRename = $event"
      @confirm="handleRename" />

    <ModalDialog
      :show="showClone"
      title="Clone Build"
      placeholder="Name for the clone"
      :initial-value="store.activePreset ? `${store.activePreset.name} (Copy)` : ''"
      confirm-label="Clone"
      @update:show="showClone = $event"
      @confirm="handleClone" />

    <ModalDialog
      :show="showImport"
      title="Import Build"
      placeholder="Paste build code here..."
      confirm-label="Import"
      @update:show="showImport = $event"
      @confirm="handleImport" />

    <ModalDialog
      :show="showDelete"
      title="Delete Build"
      type="confirm"
      :message="`Are you sure you want to delete &quot;${store.activePreset?.name}&quot;? This cannot be undone.`"
      confirm-label="Delete"
      :danger="true"
      @update:show="showDelete = $event"
      @confirm="handleDelete" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS, RARITY_DEFS } from '../../../types/buildPlanner'
import StyledSelect from '../../Shared/StyledSelect.vue'
import ModalDialog from '../../Shared/ModalDialog.vue'
import PaperDollSlot from './PaperDollSlot.vue'
import PaperDollStats from './PaperDollStats.vue'
import AbilityBarSummary from './AbilityBarSummary.vue'

const store = useBuildPlannerStore()

// Left column: armor slots (Head, Chest, Legs, Feet, Hands)
const leftSlots = computed(() =>
  EQUIPMENT_SLOTS.filter(s => s.group === 'armor')
)

// Right column: non-armor slots (MainHand, OffHand, Necklace, Ring, Belt)
const rightSlots = computed(() =>
  EQUIPMENT_SLOTS.filter(s => s.group !== 'armor')
)

const presetOptions = computed(() =>
  store.presets.map(p => ({ value: String(p.id), label: p.name }))
)

const rarityOptions = RARITY_DEFS.map(r => ({ value: r.id, label: r.label }))

const showDefaults = ref(false)
const showCreate = ref(false)
const showRename = ref(false)
const showClone = ref(false)
const showDelete = ref(false)
const showImport = ref(false)
const exportMessage = ref('')
const importError = ref('')

function onPresetChange(val: string) {
  const id = Number(val)
  const preset = store.presets.find(p => p.id === id)
  if (preset) store.selectPreset(preset)
}

async function handleCreate(name: string) {
  if (!name) return
  await store.createPreset(name)
}

async function handleRename(name: string) {
  if (!name || name === store.activePreset?.name) return
  await store.updatePreset({ name })
}

async function handleClone(name: string) {
  if (!name || !store.activePreset) return
  await store.clonePreset(store.activePreset.id, name)
}

async function handleExport() {
  if (!store.activePreset) return
  try {
    const code = await store.exportPreset(store.activePreset.id)
    await navigator.clipboard.writeText(code)
    exportMessage.value = 'Copied to clipboard!'
    setTimeout(() => { exportMessage.value = '' }, 3000)
  } catch (e) {
    exportMessage.value = `Export failed: ${e}`
    setTimeout(() => { exportMessage.value = '' }, 5000)
  }
}

async function handleImport(code: string) {
  if (!code) return
  importError.value = ''
  try {
    await store.importPreset(code)
  } catch (e) {
    importError.value = `${e}`
    setTimeout(() => { importError.value = '' }, 5000)
  }
}

async function handleDelete() {
  if (!store.activePreset) return
  await store.deletePreset(store.activePreset.id)
}

async function onRarityChange(val: string) {
  await store.updatePreset({ target_rarity: val })
}

async function onLevelChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  if (val >= 1 && val <= 125) {
    await store.updatePreset({ target_level: val })
    await store.onBuildParamsChanged()
  }
}
</script>
