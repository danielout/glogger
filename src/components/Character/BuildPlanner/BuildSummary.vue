<template>
  <div class="flex flex-col h-full overflow-y-auto px-4 py-3 space-y-5">
    <!-- Effects section with view tabs -->
    <div v-if="store.presetMods.length > 0">
      <!-- Tab bar -->
      <div class="flex items-center gap-1 mb-3">
        <button
          v-for="tab in VIEW_TABS"
          :key="tab.id"
          class="px-2.5 py-1 rounded text-xs font-semibold cursor-pointer transition-colors"
          :class="activeTab === tab.id
            ? 'bg-accent-gold/20 text-accent-gold'
            : 'text-text-muted hover:text-text-secondary'"
          @click="activeTab = tab.id">
          {{ tab.label }}
        </button>
      </div>

      <div v-if="loadingEffects" class="text-sm text-text-muted py-4 text-center">
        Loading effects...
      </div>

      <!-- By Skill view (original) -->
      <div v-else-if="activeTab === 'skill'" class="space-y-4">
        <div v-for="group in effectGroups" :key="group.label">
          <h4 class="text-sm font-semibold mb-2" :class="group.labelClass">
            {{ group.label }} ({{ group.mods.length }})
          </h4>
          <div class="space-y-2">
            <div
              v-for="mod in group.mods"
              :key="mod.id"
              class="flex items-start gap-3 text-sm pl-2 py-1 rounded"
              :class="mod.is_augment ? 'bg-purple-900/10' : ''">
              <span class="text-text-dim shrink-0 w-24 text-xs pt-0.5">{{ slotLabel(mod.equip_slot) }}</span>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="font-medium text-text-primary">{{ resolvedNames[modKey(mod)] ?? mod.power_name }}</span>
                  <span v-if="mod.is_augment" class="text-[10px] font-semibold text-purple-400 uppercase">AUG</span>
                </div>
                <div v-if="resolvedEffects[modKey(mod)]" class="mt-0.5">
                  <EffectLine v-for="(effect, i) in resolvedEffects[modKey(mod)]" :key="i" :text="effect" />
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="effectGroups.length === 0" class="text-sm text-text-dim text-center py-4">
          No mods assigned yet
        </div>
      </div>

      <!-- Effect Totals view -->
      <div v-else-if="activeTab === 'totals'" class="space-y-3">
        <div v-if="aggregatedEffects.length === 0" class="text-sm text-text-dim text-center py-4">
          No effects resolved yet
        </div>
        <div v-else class="space-y-1">
          <div
            v-for="agg in aggregatedEffects"
            :key="agg.label"
            class="flex items-center gap-2 py-1 px-2 rounded"
            :class="agg.count > 1 ? 'bg-surface-elevated' : ''">
            <EffectLine
              :label="agg.label"
              :formatted-value="agg.formattedValue"
              :numeric-value="agg.numericValue"
              :icon-id="agg.iconId"
              class="flex-1" />
            <span v-if="agg.count > 1" class="text-[10px] text-text-muted shrink-0">
              {{ agg.count }} sources
            </span>
          </div>
        </div>
      </div>

      <!-- By Ability view -->
      <div v-else-if="activeTab === 'ability'" class="space-y-2">
        <div v-if="abilityEffectGroups.length === 0" class="text-sm text-text-dim text-center py-4">
          {{ store.presetAbilities.length === 0 ? 'No abilities assigned to ability bars' : 'No mod effects reference your abilities' }}
        </div>
        <AbilityDamageCard
          v-for="group in abilityEffectGroups"
          :key="group.abilityName"
          :ability-name="group.abilityName"
          :effects="group.effects" />
      </div>
    </div>

    <div v-else class="text-sm text-text-dim text-center py-8">
      No mods assigned. Select equipment slots and add mods to see your build summary.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { EQUIPMENT_SLOTS } from '../../../types/buildPlanner'
import type { BuildPresetMod } from '../../../types/buildPlanner'
import { formatStatValue } from '../../../composables/useBuildStats'
import EffectLine from './EffectLine.vue'
import AbilityDamageCard from './AbilityDamageCard.vue'

const VIEW_TABS = [
  { id: 'skill', label: 'By Skill' },
  { id: 'totals', label: 'Effect Totals' },
  { id: 'ability', label: 'By Ability' },
] as const

type ViewTab = typeof VIEW_TABS[number]['id']

const store = useBuildPlannerStore()
const gameData = useGameDataStore()
const loadingEffects = ref(false)
const activeTab = ref<ViewTab>('skill')
const resolvedEffects = ref<Record<string, string[]>>({})
const resolvedNames = ref<Record<string, string>>({})
/** Tracks which mods belong to which skill (from resolved power info) */
const modSkills = ref<Record<string, string | null>>({})

/** Structured effect data per mod key */
interface StructuredEffect {
  label: string
  value: string
  displayType: string
  formatted: string
  iconId: number | null
}
const structuredEffects = ref<Record<string, StructuredEffect[]>>({})

function slotLabel(slotId: string): string {
  return EQUIPMENT_SLOTS.find(s => s.id === slotId)?.label ?? slotId
}

function modKey(mod: BuildPresetMod): string {
  return `${mod.power_name}:${mod.tier ?? 0}`
}

interface EffectGroup {
  label: string
  labelClass: string
  mods: BuildPresetMod[]
}

const effectGroups = computed((): EffectGroup[] => {
  const primary = store.activePreset?.skill_primary
  const secondary = store.activePreset?.skill_secondary
  const groups: EffectGroup[] = []

  if (primary) {
    const primaryMods = store.presetMods.filter(m => modSkills.value[modKey(m)] === primary)
    if (primaryMods.length > 0) {
      groups.push({ label: primary, labelClass: 'text-blue-400', mods: primaryMods })
    }
  }

  if (secondary) {
    const secondaryMods = store.presetMods.filter(m => modSkills.value[modKey(m)] === secondary)
    if (secondaryMods.length > 0) {
      groups.push({ label: secondary, labelClass: 'text-emerald-400', mods: secondaryMods })
    }
  }

  const enduranceMods = store.presetMods.filter(m => {
    const skill = modSkills.value[modKey(m)]
    return skill === 'Endurance' && skill !== primary && skill !== secondary
  })
  if (enduranceMods.length > 0) {
    groups.push({ label: 'Endurance', labelClass: 'text-amber-400', mods: enduranceMods })
  }

  const genericMods = store.presetMods.filter(m => {
    const skill = modSkills.value[modKey(m)]
    return !skill || skill === 'AnySkill'
  })
  if (genericMods.length > 0) {
    groups.push({ label: 'Generic', labelClass: 'text-text-muted', mods: genericMods })
  }

  const grouped = new Set(groups.flatMap(g => g.mods.map(m => m.id)))
  const remaining = store.presetMods.filter(m => !grouped.has(m.id))
  if (remaining.length > 0) {
    groups.push({ label: 'Other', labelClass: 'text-text-dim', mods: remaining })
  }

  return groups
})

// ── Effect Totals ────────────────────────────────────────────────────────────

interface AggregatedEffect {
  label: string
  numericValue: number
  formattedValue: string
  displayType: string
  iconId: number | null
  count: number
}

const aggregatedEffects = computed((): AggregatedEffect[] => {
  const totals = new Map<string, AggregatedEffect>()

  for (const mod of store.presetMods) {
    const key = modKey(mod)
    const effects = structuredEffects.value[key]
    if (!effects) continue

    for (const effect of effects) {
      const numVal = parseFloat(effect.value) || 0
      const existing = totals.get(effect.label)
      if (existing) {
        existing.numericValue += numVal
        existing.count += 1
      } else {
        totals.set(effect.label, {
          label: effect.label,
          numericValue: numVal,
          formattedValue: '',
          displayType: effect.displayType,
          iconId: effect.iconId,
          count: 1,
        })
      }
    }
  }

  // Format aggregated values
  const results = Array.from(totals.values())
  for (const agg of results) {
    agg.formattedValue = formatStatValue(agg.numericValue, agg.displayType)
  }

  // Sort: highest absolute value first
  results.sort((a, b) => Math.abs(b.numericValue) - Math.abs(a.numericValue))
  return results
})

// ── By Ability view ──────────────────────────────────────────────────────────

interface AbilityEffectEntry {
  label: string
  numericValue: number
  formattedValue: string
  iconId: number | null
  source: string
}

interface AbilityEffectGroup {
  abilityName: string
  effects: AbilityEffectEntry[]
}

/**
 * Ability effect groups built using the precomputed TSys↔Ability index.
 * Uses a single batch call to the backend, then groups effects client-side.
 */
const abilityEffectGroups = ref<AbilityEffectGroup[]>([])

async function refreshAbilityEffectGroups() {
  if (store.presetAbilities.length === 0 || store.presetMods.length === 0) {
    abilityEffectGroups.value = []
    return
  }

  const assignedAbilityIds = new Set(store.presetAbilities.map(a => a.ability_id))
  const abilityNames = new Map<number, string>()
  for (const a of store.presetAbilities) {
    if (a.ability_name) abilityNames.set(a.ability_id, a.ability_name)
  }

  // Collect unique power names from all assigned mods
  // power_name is the internal_name — backend resolves to CDN key via index
  const powerNames = [...new Set(store.presetMods.map(m => m.power_name))]

  // Single batch call — backend accepts internal names or CDN keys, O(1) per key
  let tsysAbilityMap: Record<string, number[]> = {}
  if (powerNames.length > 0) {
    try {
      tsysAbilityMap = await gameData.getTsysAbilityMap(powerNames)
    } catch { /* ignore */ }
  }

  // Group effects by ability
  const groups = new Map<number, AbilityEffectEntry[]>()

  for (const mod of store.presetMods) {
    const key = modKey(mod)

    const abilityIds = tsysAbilityMap[mod.power_name]
    if (!abilityIds) continue

    const matchedIds = abilityIds.filter(id => assignedAbilityIds.has(id))
    if (matchedIds.length === 0) continue

    const modEffects = structuredEffects.value[key]
    if (!modEffects) continue

    for (const abilityId of matchedIds) {
      if (!groups.has(abilityId)) groups.set(abilityId, [])
      for (const effect of modEffects) {
        groups.get(abilityId)!.push({
          label: effect.label,
          numericValue: parseFloat(effect.value) || 0,
          formattedValue: formatStatValue(parseFloat(effect.value) || 0, effect.displayType),
          iconId: effect.iconId,
          source: `${resolvedNames.value[key] ?? mod.power_name} (${slotLabel(mod.equip_slot)})`,
        })
      }
    }
  }

  const result: AbilityEffectGroup[] = []
  for (const [abilityId, effects] of groups) {
    const name = abilityNames.get(abilityId) ?? `Ability #${abilityId}`
    result.push({ abilityName: name, effects })
  }
  result.sort((a, b) => b.effects.length - a.effects.length)
  abilityEffectGroups.value = result
}

// ── Data loading ─────────────────────────────────────────────────────────────

async function resolveAllEffects() {
  loadingEffects.value = true
  const effects: Record<string, string[]> = {}
  const skills: Record<string, string | null> = {}
  const names: Record<string, string> = {}
  const structured: Record<string, StructuredEffect[]> = {}
  try {
    for (const mod of store.presetMods) {
      if (mod.tier == null) continue
      const key = modKey(mod)
      if (effects[key]) continue
      try {
        const info = await invoke<{
          internal_name: string
          skill: string | null
          prefix: string | null
          suffix: string | null
          tier_effects: string[]
          tier_effects_structured: Array<{
            label: string
            value: string
            display_type: string
            formatted: string
            icon_id: number | null
          }>
        } | null>('get_tsys_power_info', {
          powerName: mod.power_name,
          tier: mod.tier,
        })
        if (info) {
          if (info.tier_effects) effects[key] = info.tier_effects
          skills[key] = info.skill
          const displayName = info.prefix ?? info.suffix ?? mod.power_name
          if (displayName !== mod.power_name) names[key] = displayName
          if (info.tier_effects_structured) {
            structured[key] = info.tier_effects_structured.map(e => ({
              label: e.label,
              value: e.value,
              displayType: e.display_type,
              formatted: e.formatted,
              iconId: e.icon_id,
            }))
          }
        }
      } catch {
        // Power might not resolve
      }
    }
  } finally {
    resolvedEffects.value = effects
    modSkills.value = skills
    resolvedNames.value = names
    structuredEffects.value = structured
    loadingEffects.value = false
    // Refresh ability cross-reference after effects are resolved
    refreshAbilityEffectGroups()
  }
}

// Load effects when mods change (array reference changes on preset load)
onMounted(() => {
  if (store.presetMods.length > 0) resolveAllEffects()
})

watch(() => store.presetMods, () => {
  resolveAllEffects()
})
</script>
