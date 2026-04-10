<template>
  <div class="flex flex-wrap items-center gap-3 px-1">
    <!-- Build selector -->
    <div class="flex items-center gap-2">
      <label class="text-xs text-text-muted">Build:</label>
      <StyledSelect
        :model-value="String(store.activePreset?.id ?? '')"
        :options="presetOptions"
        placeholder="Select a build..."
        size="sm"
        @update:model-value="onPresetChange" />

      <button
        class="px-2 py-1 text-xs bg-accent-gold/20 border border-accent-gold/40 text-accent-gold rounded cursor-pointer hover:bg-accent-gold/30"
        @click="showCreate = true">
        + New
      </button>

      <template v-if="store.activePreset">
        <button
          class="px-2 py-1 text-xs bg-surface-elevated border border-border-default text-text-secondary rounded cursor-pointer hover:bg-surface-hover"
          @click="showRename = true">
          Rename
        </button>
        <button
          class="px-2 py-1 text-xs bg-red-900/20 border border-red-700/40 text-red-400 rounded cursor-pointer hover:bg-red-900/30"
          @click="showDelete = true">
          Delete
        </button>
      </template>
    </div>

    <!-- Skill pickers -->
    <template v-if="store.activePreset">
      <div class="flex items-center gap-2 ml-2">
        <label class="text-xs text-text-muted">Skills:</label>
        <StyledSelect
          :model-value="store.activePreset.skill_primary ?? ''"
          :options="skillOptions('Primary...')"
          placeholder="Primary..."
          size="sm"
          color-class="text-blue-400"
          @update:model-value="onPrimarySkillChange" />
        <span class="text-text-muted text-xs">+</span>
        <StyledSelect
          :model-value="store.activePreset.skill_secondary ?? ''"
          :options="skillOptions('Secondary...')"
          placeholder="Secondary..."
          size="sm"
          color-class="text-emerald-400"
          @update:model-value="onSecondarySkillChange" />
      </div>

      <!-- Default level + rarity (used as starting values for new slots) -->
      <div class="flex items-center gap-2 ml-2">
        <label class="text-xs text-text-dim" title="Default values for new equipment slots">Defaults:</label>
        <label class="text-xs text-text-muted">Lv</label>
        <input
          type="number"
          :value="store.activePreset.target_level"
          min="1"
          max="125"
          class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-sm text-text-primary w-14 text-center"
          @change="onLevelChange" />
        <StyledSelect
          :model-value="store.activePreset.target_rarity"
          :options="rarityOptions"
          size="sm"
          @update:model-value="onRarityChange" />
      </div>
    </template>

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
import { RARITY_DEFS } from '../../../types/buildPlanner'
import ModalDialog from '../../Shared/ModalDialog.vue'
import StyledSelect from '../../Shared/StyledSelect.vue'

const store = useBuildPlannerStore()

const presetOptions = computed(() =>
  store.presets.map(p => ({ value: String(p.id), label: p.name }))
)

function skillOptions(placeholder: string) {
  return [
    { value: '', label: placeholder },
    ...store.combatSkills.map(s => ({ value: s.name, label: s.name })),
  ]
}

const rarityOptions = RARITY_DEFS.map(r => ({ value: r.id, label: r.label }))

const showCreate = ref(false)
const showRename = ref(false)
const showDelete = ref(false)

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

async function handleDelete() {
  if (!store.activePreset) return
  await store.deletePreset(store.activePreset.id)
}

async function onPrimarySkillChange(val: string) {
  await store.updatePreset({ skill_primary: val || null })
  await store.onBuildParamsChanged()
}

async function onSecondarySkillChange(val: string) {
  await store.updatePreset({ skill_secondary: val || null })
  await store.onBuildParamsChanged()
}

async function onLevelChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  if (val >= 1 && val <= 125) {
    await store.updatePreset({ target_level: val })
    await store.onBuildParamsChanged()
  }
}

async function onRarityChange(val: string) {
  await store.updatePreset({ target_rarity: val })
}
</script>
