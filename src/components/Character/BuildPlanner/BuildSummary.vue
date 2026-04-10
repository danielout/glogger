<template>
  <div class="flex flex-col h-full overflow-y-auto px-4 py-3 space-y-5">
    <!-- Skills & Rarity -->
    <div class="flex items-center gap-4 text-sm">
      <div v-if="store.activePreset?.skill_primary" class="flex items-center gap-1.5">
        <span class="text-[10px] font-semibold text-blue-400 uppercase">Primary:</span>
        <span class="text-text-primary">{{ store.activePreset.skill_primary }}</span>
      </div>
      <div v-if="store.activePreset?.skill_secondary" class="flex items-center gap-1.5">
        <span class="text-[10px] font-semibold text-emerald-400 uppercase">Secondary:</span>
        <span class="text-text-primary">{{ store.activePreset.skill_secondary }}</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span class="text-[10px] font-semibold text-text-muted uppercase">Target:</span>
        <span class="text-text-primary">
          Lv{{ store.activePreset?.target_level }}
          {{ store.activePreset?.target_rarity }}
        </span>
      </div>
    </div>

    <!-- Armor type breakdown -->
    <div v-if="Object.keys(store.armorTypeCounts).length > 0">
      <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-2">Armor Sets</h3>
      <div class="flex items-center gap-2 flex-wrap">
        <span
          v-for="(count, type) in store.armorTypeCounts"
          :key="type"
          class="px-2 py-1 rounded text-xs font-medium"
          :class="[armorBadge(type as string), count >= 3 ? 'ring-1 ring-accent-gold/50' : '']">
          {{ count }}x {{ type }}
          <span v-if="count >= 3" class="text-accent-gold ml-0.5">(3pc bonus)</span>
        </span>
      </div>
    </div>

    <!-- Crafting points overview -->
    <div v-if="totalCPBudget > 0">
      <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-2">Crafting Points</h3>
      <CpProgressBar :used="totalCPUsed" :budget="totalCPBudget" label="Total" size="sm" />
    </div>

    <!-- Per-slot breakdown cards -->
    <div v-if="store.presetMods.length > 0">
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Slot Breakdown</h3>
        <span class="text-[10px] text-text-dim">
          {{ store.presetMods.filter(m => !m.is_augment).length }} mods,
          {{ store.presetMods.filter(m => m.is_augment).length }} augments
        </span>
      </div>
      <div class="space-y-1.5">
        <SummarySlotCard
          v-for="slot in slotsWithMods"
          :key="slot.id"
          :slot="slot"
          :resolved-names="resolvedNames" />
      </div>
    </div>

    <!-- Base item attributes (armor, combat refresh, etc.) -->
    <div v-if="itemAttributes.length > 0">
      <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-2">Item Attributes</h3>
      <div class="space-y-1 px-1">
        <EffectLine
          v-for="attr in itemAttributes"
          :key="attr.label"
          :label="attr.label"
          :formatted-value="attr.formattedValue"
          :numeric-value="attr.value"
          :icon-id="attr.iconId" />
      </div>
    </div>

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
            <span v-if="agg.count > 1" class="text-[10px] text-text-dim shrink-0">
              ({{ agg.count }} sources)
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
import {
  EQUIPMENT_SLOTS,
  getSlotCraftingPoints,
} from '../../../types/buildPlanner'
import type { BuildPresetMod } from '../../../types/buildPlanner'
import EffectLine from './EffectLine.vue'
import CpProgressBar from './CpProgressBar.vue'
import SummarySlotCard from './SummarySlotCard.vue'
import AbilityDamageCard from './AbilityDamageCard.vue'

const VIEW_TABS = [
  { id: 'skill', label: 'By Skill' },
  { id: 'totals', label: 'Effect Totals' },
  { id: 'ability', label: 'By Ability' },
] as const

type ViewTab = typeof VIEW_TABS[number]['id']

const store = useBuildPlannerStore()
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

/** Slots that have at least one mod or augment assigned */
const slotsWithMods = computed(() =>
  EQUIPMENT_SLOTS.filter(s =>
    store.presetMods.some(m => m.equip_slot === s.id)
  )
)

function slotLabel(slotId: string): string {
  return EQUIPMENT_SLOTS.find(s => s.id === slotId)?.label ?? slotId
}

function modKey(mod: BuildPresetMod): string {
  return `${mod.power_name}:${mod.tier ?? 0}`
}

const totalCPBudget = computed(() => {
  let total = 0
  for (const slot of EQUIPMENT_SLOTS) {
    total += getSlotCraftingPoints(store.getSlotItem(slot.id))
  }
  return total
})

const totalCPUsed = computed(() => {
  let used = 0
  for (const slot of EQUIPMENT_SLOTS) {
    used += store.getSlotCpUsed(slot.id)
  }
  return used
})

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
    agg.formattedValue = formatValue(agg.numericValue, agg.displayType)
  }

  // Sort: highest absolute value first
  results.sort((a, b) => Math.abs(b.numericValue) - Math.abs(a.numericValue))
  return results
})

function formatValue(value: number, displayType: string): string {
  switch (displayType) {
    case 'AsPercent':
    case 'AsBuffMod':
      return `+${Math.round(value * 100)}%`
    case 'AsDebuffMod':
      return `-${Math.round(Math.abs(value) * 100)}%`
    case 'AsBuffDelta':
    case 'AsInt':
      return `+${Math.round(value)}`
    case 'AsDebuffDelta':
      return `${Math.round(value)}`
    case 'AsBool':
      return 'Yes'
    default:
      if (value === 0) return ''
      return value > 0 ? `+${Math.round(value)}` : `${Math.round(value)}`
  }
}

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

/** Strip trailing rank number from ability name (e.g. "Pound To Slag 9" -> "Pound To Slag") */
function baseAbilityName(name: string): string {
  return name.replace(/\s+\d+$/, '')
}

const abilityEffectGroups = computed((): AbilityEffectGroup[] => {
  const abilities = store.presetAbilities
  if (abilities.length === 0) return []

  // Deduplicate by base name (e.g. "First Aid 5" and "First Aid 3" both match "First Aid")
  const baseNames = new Map<string, string>()
  for (const a of abilities) {
    if (!a.ability_name) continue
    const base = baseAbilityName(a.ability_name)
    // Keep the full name for display (prefer the highest rank)
    if (!baseNames.has(base) || a.ability_name > baseNames.get(base)!) {
      baseNames.set(base, a.ability_name)
    }
  }

  const groups: AbilityEffectGroup[] = []

  for (const [baseName, displayName] of baseNames) {
    const effects: AbilityEffectEntry[] = []
    const baseNameLower = baseName.toLowerCase()

    for (const mod of store.presetMods) {
      const key = modKey(mod)
      const modEffects = structuredEffects.value[key]
      if (!modEffects) continue

      for (const effect of modEffects) {
        if (effect.label.toLowerCase().includes(baseNameLower)) {
          effects.push({
            label: effect.label,
            numericValue: parseFloat(effect.value) || 0,
            formattedValue: formatValue(parseFloat(effect.value) || 0, effect.displayType),
            iconId: effect.iconId,
            source: `${resolvedNames.value[key] ?? mod.power_name} (${slotLabel(mod.equip_slot)})`,
          })
        }
      }
    }

    if (effects.length > 0) {
      groups.push({ abilityName: displayName, effects })
    }
  }

  // Sort by number of effects (most modified abilities first)
  groups.sort((a, b) => b.effects.length - a.effects.length)
  return groups
})

// ── Item Attributes (base item effects: armor, combat refresh, etc.) ─────────

interface ItemAttribute {
  label: string
  value: number
  formattedValue: string
  iconId: number | null
}

const itemAttributes = ref<ItemAttribute[]>([])

async function resolveItemAttributes() {
  const totals = new Map<string, { value: number; displayType: string; iconId: number | null }>()

  for (const slot of EQUIPMENT_SLOTS) {
    const slotItem = store.getSlotItem(slot.id)
    if (!slotItem || slotItem.item_id === 0) continue

    try {
      const item = await invoke<{
        effect_descs: string[]
      } | null>('resolve_item', { reference: String(slotItem.item_id) })
      if (!item?.effect_descs?.length) continue

      const resolved = await invoke<Array<{
        label: string
        value: string
        display_type: string
        formatted: string
        icon_id: number | null
      }>>('resolve_effect_descs', { descs: item.effect_descs })

      for (const eff of resolved) {
        const numVal = parseFloat(eff.value) || 0
        if (numVal === 0 && eff.display_type !== 'AsBool') continue

        const existing = totals.get(eff.label)
        if (existing) {
          existing.value += numVal
        } else {
          totals.set(eff.label, {
            value: numVal,
            displayType: eff.display_type,
            iconId: eff.icon_id,
          })
        }
      }
    } catch {
      // Item might not resolve
    }
  }

  const results: ItemAttribute[] = []
  for (const [label, data] of totals) {
    results.push({
      label,
      value: data.value,
      formattedValue: formatValue(data.value, data.displayType),
      iconId: data.iconId,
    })
  }
  // Sort by absolute value descending
  results.sort((a, b) => Math.abs(b.value) - Math.abs(a.value))
  itemAttributes.value = results
}

// ── Armor badge styling ──────────────────────────────────────────────────────

function armorBadge(type: string): string {
  switch (type) {
    case 'Cloth': return 'bg-blue-900/30 text-blue-300'
    case 'Leather': return 'bg-amber-900/30 text-amber-300'
    case 'Metal': return 'bg-slate-600/30 text-slate-300'
    case 'Organic': return 'bg-green-900/30 text-green-300'
    default: return 'bg-surface-hover text-text-dim'
  }
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
  }
}

// Load effects eagerly when preset has mods, and when mods change
onMounted(() => {
  if (store.presetMods.length > 0) resolveAllEffects()
  resolveItemAttributes()
})

watch(() => store.activePreset?.id, () => {
  if (store.presetMods.length > 0) resolveAllEffects()
  resolveItemAttributes()
})

watch(() => store.presetMods.length, () => {
  resolveAllEffects()
})
</script>
