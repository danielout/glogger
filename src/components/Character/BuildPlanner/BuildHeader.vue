<template>
  <div class="flex flex-wrap items-center gap-3 px-1">
    <!-- Build selector -->
    <div class="flex items-center gap-2">
      <label class="text-xs text-text-muted">Build:</label>
      <select
        class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-sm text-text-primary cursor-pointer min-w-40"
        :value="store.activePreset?.id ?? ''"
        @change="onPresetChange">
        <option value="" disabled>Select a build...</option>
        <option v-for="preset in store.presets" :key="preset.id" :value="preset.id">
          {{ preset.name }}
        </option>
      </select>

      <button
        class="px-2 py-1 text-xs bg-accent-gold/20 border border-accent-gold/40 text-accent-gold rounded cursor-pointer hover:bg-accent-gold/30"
        @click="handleCreate">
        + New
      </button>

      <template v-if="store.activePreset">
        <button
          class="px-2 py-1 text-xs bg-surface-elevated border border-border-default text-text-secondary rounded cursor-pointer hover:bg-surface-hover"
          @click="handleRename">
          Rename
        </button>
        <button
          class="px-2 py-1 text-xs bg-red-900/20 border border-red-700/40 text-red-400 rounded cursor-pointer hover:bg-red-900/30"
          @click="handleDelete">
          Delete
        </button>
      </template>
    </div>

    <!-- Skill pickers -->
    <template v-if="store.activePreset">
      <div class="flex items-center gap-2 ml-2">
        <label class="text-xs text-text-muted">Skills:</label>
        <select
          class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-sm text-text-primary cursor-pointer"
          :value="store.activePreset.skill_primary ?? ''"
          @change="onPrimarySkillChange">
          <option value="">Primary...</option>
          <option v-for="skill in store.combatSkills" :key="skill.name" :value="skill.name">
            {{ skill.name }}
          </option>
        </select>
        <span class="text-text-muted text-xs">+</span>
        <select
          class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-sm text-text-primary cursor-pointer"
          :value="store.activePreset.skill_secondary ?? ''"
          @change="onSecondarySkillChange">
          <option value="">Secondary...</option>
          <option v-for="skill in store.combatSkills" :key="skill.name" :value="skill.name">
            {{ skill.name }}
          </option>
        </select>
      </div>

      <!-- Target level + rarity -->
      <div class="flex items-center gap-2 ml-2">
        <label class="text-xs text-text-muted">Level:</label>
        <input
          type="number"
          :value="store.activePreset.target_level"
          min="1"
          max="125"
          class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-sm text-text-primary w-16 text-center"
          @change="onLevelChange" />
        <label class="text-xs text-text-muted ml-1">Rarity:</label>
        <select
          class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-sm text-text-primary cursor-pointer"
          :value="store.activePreset.target_rarity"
          @change="onRarityChange">
          <option v-for="r in rarities" :key="r.id" :value="r.id">{{ r.label }}</option>
        </select>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { RARITY_DEFS } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()
const rarities = RARITY_DEFS

function onPresetChange(e: Event) {
  const id = Number((e.target as HTMLSelectElement).value)
  const preset = store.presets.find(p => p.id === id)
  if (preset) store.selectPreset(preset)
}

async function handleCreate() {
  const name = prompt("Build name:")
  if (!name?.trim()) return
  await store.createPreset(name.trim())
}

async function handleRename() {
  if (!store.activePreset) return
  const name = prompt("New name:", store.activePreset.name)
  if (!name?.trim() || name === store.activePreset.name) return
  await store.updatePreset({ name: name.trim() })
}

async function handleDelete() {
  if (!store.activePreset) return
  if (!confirm(`Delete build "${store.activePreset.name}"?`)) return
  await store.deletePreset(store.activePreset.id)
}

async function onPrimarySkillChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value || null
  await store.updatePreset({ skill_primary: val })
  await store.onBuildParamsChanged()
}

async function onSecondarySkillChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value || null
  await store.updatePreset({ skill_secondary: val })
  await store.onBuildParamsChanged()
}

async function onLevelChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  if (val >= 1 && val <= 125) {
    await store.updatePreset({ target_level: val })
    await store.onBuildParamsChanged()
  }
}

async function onRarityChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value
  await store.updatePreset({ target_rarity: val })
}
</script>
