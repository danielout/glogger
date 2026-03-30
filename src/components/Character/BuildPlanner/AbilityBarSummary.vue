<template>
  <div class="flex flex-col gap-1">
    <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Ability Bars</h3>

    <div v-for="bar in bars" :key="bar.id" class="flex flex-col">
      <!-- Bar header (clickable to select for editing, toggle to expand) -->
      <button
        class="flex items-center justify-between px-2 py-1.5 rounded border text-xs cursor-pointer transition-all"
        :class="barClasses(bar.id)"
        @click="store.selectBar(bar.id)">
        <div class="flex items-center gap-1.5">
          <span
            class="text-[10px] transition-transform"
            :class="expandedBars[bar.id] ? 'rotate-90' : ''"
            @click.stop="toggleExpanded(bar.id)">
            ▶
          </span>
          <span class="font-medium">{{ barLabel(bar.id) }}</span>
        </div>
        <span class="text-[10px]" :class="fillColor(bar.id)">
          {{ store.barAbilityCounts[bar.id] }}/{{ bar.slots }}
        </span>
      </button>

      <!-- Expanded: show assigned abilities with icons -->
      <div
        v-if="expandedBars[bar.id] && barAbilities(bar.id).length > 0"
        class="flex flex-wrap gap-1 px-2 py-1.5 border-x border-b border-border-default/50 rounded-b -mt-px">
        <div
          v-for="ability in barAbilities(bar.id)"
          :key="ability.id"
          class="flex items-center gap-1 text-[10px] text-text-secondary"
          :title="ability.ability_name ?? ''">
          <GameIcon
            v-if="resolvedAbilities[ability.ability_id]"
            :icon-id="resolvedAbilities[ability.ability_id]!.icon_id"
            :alt="ability.ability_name ?? ''"
            size="xs" />
          <span class="truncate max-w-20">{{ ability.ability_name ?? `#${ability.ability_id}` }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { ABILITY_BARS } from '../../../types/buildPlanner'
import type { AbilityInfo } from '../../../types/gameData'
import type { BuildPresetAbility } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'

const store = useBuildPlannerStore()
const gameData = useGameDataStore()
const bars = ABILITY_BARS

const expandedBars = ref<Record<string, boolean>>({
  primary: true,
  secondary: true,
  sidebar: false,
})

const resolvedAbilities = ref<Record<number, AbilityInfo>>({})

function toggleExpanded(barId: string) {
  expandedBars.value[barId] = !expandedBars.value[barId]
}

function barLabel(barId: string): string {
  if (barId === 'primary') return store.activePreset?.skill_primary ?? 'Primary'
  if (barId === 'secondary') return store.activePreset?.skill_secondary ?? 'Secondary'
  return 'Sidebar'
}

function barAbilities(barId: string): BuildPresetAbility[] {
  return store.getBarAbilities(barId)
}

function barClasses(barId: string): string {
  const isSelected = store.activeBar === barId
  if (isSelected) {
    return 'bg-accent-gold/20 border-accent-gold/60 text-accent-gold'
  }
  const count = store.barAbilityCounts[barId as keyof typeof store.barAbilityCounts] ?? 0
  const max = ABILITY_BARS.find(b => b.id === barId)?.slots ?? 6
  if (count >= max) {
    return 'bg-green-900/15 border-green-700/30 text-text-primary hover:bg-green-900/25'
  }
  if (count > 0) {
    return 'bg-yellow-900/15 border-yellow-700/30 text-text-primary hover:bg-yellow-900/25'
  }
  return 'bg-surface-elevated border-border-default text-text-secondary hover:bg-surface-hover'
}

function fillColor(barId: string): string {
  const count = store.barAbilityCounts[barId as keyof typeof store.barAbilityCounts] ?? 0
  const max = ABILITY_BARS.find(b => b.id === barId)?.slots ?? 6
  if (count >= max) return 'text-green-400'
  if (count > 0) return 'text-yellow-400'
  return 'text-text-dim'
}

async function resolveAbilityIcons() {
  const abilityIds = store.presetAbilities.map(a => a.ability_id)
  if (abilityIds.length === 0) return

  // Load abilities for all skills used in the build
  const skills = new Set<string>()
  if (store.activePreset?.skill_primary) skills.add(store.activePreset.skill_primary)
  if (store.activePreset?.skill_secondary) skills.add(store.activePreset.skill_secondary)
  // Sidebar skills
  for (const s of ['FirstAid', 'ArmorPatching', 'SurvivalInstincts']) {
    skills.add(s)
  }

  const map: Record<number, AbilityInfo> = {}
  for (const skill of skills) {
    try {
      const abilities = await gameData.getAbilitiesForSkill(skill)
      for (const a of abilities) {
        if (abilityIds.includes(a.id)) {
          map[a.id] = a
        }
      }
    } catch {
      // Skill might not exist
    }
  }
  resolvedAbilities.value = map
}

watch(() => store.presetAbilities.length, () => {
  resolveAbilityIcons()
})

watch(() => [store.activePreset?.skill_primary, store.activePreset?.skill_secondary], () => {
  resolveAbilityIcons()
})

onMounted(() => {
  if (store.presetAbilities.length > 0) {
    resolveAbilityIcons()
  }
})
</script>
