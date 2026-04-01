<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Bar header -->
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-2">
        <h3 class="text-sm font-semibold text-text-primary">{{ barLabel }} Abilities</h3>
        <span class="text-xs text-text-muted">
          {{ barAbilities.length }}/{{ maxSlots }} slots
        </span>
      </div>
    </div>

    <div class="flex-1 flex gap-3 min-h-0">
      <!-- Assigned abilities -->
      <div class="w-72 shrink-0 flex flex-col gap-1.5 min-h-0 overflow-y-auto">
        <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Ability Bar</h4>
        <div v-if="barAbilities.length === 0" class="text-xs text-text-dim py-2">
          No abilities assigned yet. Browse and add from the right.
        </div>
        <div
          v-for="ability in barAbilities"
          :key="ability.id"
          class="flex items-center gap-2 px-2 py-1.5 rounded text-sm bg-surface-elevated border border-border-default group">
          <span class="text-[10px] text-text-dim w-4 shrink-0 text-center">{{ ability.slot_position + 1 }}</span>
          <AbilityInline
            v-if="resolvedAbilities[ability.ability_id]"
            :ability="resolvedAbilities[ability.ability_id]!" />
          <span v-else class="text-text-secondary text-xs">{{ ability.ability_name ?? `#${ability.ability_id}` }}</span>
          <span class="flex-1" />
          <button
            class="text-red-400/60 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer shrink-0"
            title="Remove ability"
            @click="store.removeAbility(ability)">
            x
          </button>
        </div>
      </div>

      <!-- Available abilities browser -->
      <div class="flex-1 flex flex-col gap-2 min-h-0">
        <div class="flex items-center gap-2">
          <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Available Abilities</h4>
          <input
            v-model="abilityFilter"
            type="text"
            placeholder="Filter abilities..."
            class="bg-surface-elevated border border-border-default rounded px-2 py-0.5 text-xs text-text-primary flex-1 max-w-60" />
        </div>

        <div v-if="loading" class="text-xs text-text-muted py-4 text-center">
          Loading abilities...
        </div>

        <div v-else class="flex-1 overflow-y-auto flex flex-col gap-1">
          <!-- Skill abilities group -->
          <template v-if="skillAbilities.length > 0">
            <div class="sticky top-0 bg-surface-base py-1 z-10">
              <h5 class="text-[10px] font-semibold uppercase tracking-wider text-blue-400">
                {{ skillName }} ({{ filteredSkillAbilities.length }})
              </h5>
            </div>
            <AbilityOption
              v-for="ability in filteredSkillAbilities"
              :key="ability.id"
              :ability="ability"
              :is-assigned="isAssigned(ability.id)"
              @add="handleAdd(ability)" />
          </template>

          <!-- Sidebar abilities from other skills -->
          <template v-if="store.activeBar === 'sidebar' && sidebarAbilities.length > 0">
            <div class="sticky top-0 bg-surface-base py-1 z-10">
              <h5 class="text-[10px] font-semibold uppercase tracking-wider text-text-muted">
                Sidebar Skills ({{ filteredSidebarAbilities.length }})
              </h5>
            </div>
            <AbilityOption
              v-for="ability in filteredSidebarAbilities"
              :key="ability.id"
              :ability="ability"
              :is-assigned="isAssigned(ability.id)"
              @add="handleAdd(ability)" />
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { useSettingsStore } from '../../../stores/settingsStore'
import type { AbilityInfo } from '../../../types/gameData'
import AbilityInline from '../../Shared/Ability/AbilityInline.vue'
import AbilityOption from './AbilityOption.vue'

const store = useBuildPlannerStore()
const gameData = useGameDataStore()
const settingsStore = useSettingsStore()

function filterObtainable(abilities: AbilityInfo[]): AbilityInfo[] {
  if (settingsStore.settings.showUnobtainableItems) return abilities
  return abilities.filter(a => !a.keywords.includes('Lint_NotObtainable'))
}

const skillAbilities = ref<AbilityInfo[]>([])
const sidebarAbilities = ref<AbilityInfo[]>([])
const resolvedAbilities = ref<Record<number, AbilityInfo>>({})
const loading = ref(false)
const abilityFilter = ref('')

const SIDEBAR_SKILLS = ['FirstAid', 'ArmorPatching', 'SurvivalInstincts']

const barLabel = computed(() => {
  if (store.activeBar === 'primary') return store.activePreset?.skill_primary ?? 'Primary'
  if (store.activeBar === 'secondary') return store.activePreset?.skill_secondary ?? 'Secondary'
  return 'Sidebar'
})

const skillName = computed(() => {
  if (store.activeBar === 'primary') return store.activePreset?.skill_primary ?? 'Primary'
  if (store.activeBar === 'secondary') return store.activePreset?.skill_secondary ?? 'Secondary'
  return 'Sidebar'
})

const maxSlots = computed(() => store.activeBar === 'sidebar' ? 10 : 6)

const barAbilities = computed(() => {
  if (!store.activeBar) return []
  return store.getBarAbilities(store.activeBar)
})

const filteredSkillAbilities = computed(() => {
  if (!abilityFilter.value) return skillAbilities.value
  const q = abilityFilter.value.toLowerCase()
  return skillAbilities.value.filter(a =>
    a.name.toLowerCase().includes(q) ||
    (a.description?.toLowerCase().includes(q) ?? false)
  )
})

const filteredSidebarAbilities = computed(() => {
  if (!abilityFilter.value) return sidebarAbilities.value
  const q = abilityFilter.value.toLowerCase()
  return sidebarAbilities.value.filter(a =>
    a.name.toLowerCase().includes(q) ||
    (a.description?.toLowerCase().includes(q) ?? false)
  )
})

function isAssigned(abilityId: number): boolean {
  return barAbilities.value.some(a => a.ability_id === abilityId)
}

async function handleAdd(ability: AbilityInfo) {
  await store.addAbility(ability.id, ability.name)
}

async function loadAbilities() {
  loading.value = true
  try {
    // Load abilities for the bar's skill
    const skill = store.activeBar === 'primary'
      ? store.activePreset?.skill_primary
      : store.activeBar === 'secondary'
        ? store.activePreset?.skill_secondary
        : null

    if (skill) {
      skillAbilities.value = filterObtainable(await gameData.getAbilitiesForSkill(skill))
    } else {
      skillAbilities.value = []
    }

    // For sidebar bar, also load sidebar-eligible abilities
    if (store.activeBar === 'sidebar') {
      const sidebarResults: AbilityInfo[] = []
      for (const sideSkill of SIDEBAR_SKILLS) {
        try {
          const abilities = await gameData.getAbilitiesForSkill(sideSkill)
          sidebarResults.push(...abilities.filter(a =>
            (a.raw_json as Record<string, unknown>)?.CanBeOnSidebar !== false
          ))
        } catch {
          // Skill might not exist
        }
      }
      sidebarAbilities.value = filterObtainable(sidebarResults)
    } else {
      sidebarAbilities.value = []
    }

    // Build resolution map for assigned abilities
    await resolveAssignedAbilities()
  } finally {
    loading.value = false
  }
}

async function resolveAssignedAbilities() {
  const allLoaded = [...skillAbilities.value, ...sidebarAbilities.value]
  const map: Record<number, AbilityInfo> = {}
  for (const a of allLoaded) {
    map[a.id] = a
  }
  resolvedAbilities.value = map
}

watch(() => store.activeBar, () => {
  abilityFilter.value = ''
  if (store.activeBar) loadAbilities()
})

watch(() => [store.activePreset?.skill_primary, store.activePreset?.skill_secondary], () => {
  if (store.activeBar) loadAbilities()
})

onMounted(() => {
  if (store.activeBar) loadAbilities()
})
</script>
