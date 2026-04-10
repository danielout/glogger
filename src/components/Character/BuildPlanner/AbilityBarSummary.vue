<template>
  <div class="flex flex-col gap-2">
    <div v-for="bar in bars" :key="bar.id">
      <!-- Bar card -->
      <button
        class="w-full flex flex-col rounded border cursor-pointer transition-all"
        :class="barClasses(bar.id)"
        @click="store.selectBar(bar.id)">

        <!-- Header row: skill name + fill count -->
        <div class="flex items-center justify-between px-2.5 py-1.5">
          <span class="text-xs font-semibold">{{ barLabel(bar.id) }}</span>
          <span class="text-[10px]" :class="fillColor(bar.id)">
            {{ store.barAbilityCounts[bar.id] }}/{{ bar.slots }}
          </span>
        </div>

        <!-- Ability icon strip (always visible when abilities assigned) -->
        <div
          v-if="barAbilities(bar.id).length > 0"
          class="flex flex-wrap gap-1 px-2.5 pb-2 -mt-0.5">
          <div
            v-for="ability in barAbilities(bar.id)"
            :key="ability.id"
            class="flex items-center gap-1"
            :title="ability.ability_name ?? ''">
            <GameIcon
              v-if="resolvedAbilities[ability.ability_id]"
              :icon-id="resolvedAbilities[ability.ability_id]!.icon_id"
              :alt="ability.ability_name ?? ''"
              size="xs" />
            <span class="text-[10px] text-text-secondary truncate max-w-16">
              {{ ability.ability_name ?? `#${ability.ability_id}` }}
            </span>
          </div>
        </div>

        <!-- Empty state -->
        <div
          v-else
          class="px-2.5 pb-2 -mt-0.5 text-[10px] text-text-dim italic">
          No abilities assigned
        </div>
      </button>
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

const resolvedAbilities = ref<Record<number, AbilityInfo>>({})

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
    return 'bg-accent-gold/15 border-accent-gold/50 text-text-primary'
  }
  const count = store.barAbilityCounts[barId as keyof typeof store.barAbilityCounts] ?? 0
  const max = ABILITY_BARS.find(b => b.id === barId)?.slots ?? 6
  if (count >= max) {
    return 'bg-surface-elevated border-border-default text-text-primary hover:bg-surface-hover border-l-2 border-l-green-500/50'
  }
  if (count > 0) {
    return 'bg-surface-elevated border-border-default text-text-primary hover:bg-surface-hover border-l-2 border-l-yellow-500/40'
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

  const skills = new Set<string>()
  if (store.activePreset?.skill_primary) skills.add(store.activePreset.skill_primary)
  if (store.activePreset?.skill_secondary) skills.add(store.activePreset.skill_secondary)
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
