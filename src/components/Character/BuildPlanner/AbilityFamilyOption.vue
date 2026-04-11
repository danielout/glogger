<template>
  <button
    class="w-full flex items-start gap-2 px-2 py-1.5 rounded text-sm border transition-all text-left"
    :class="rowClass"
    :disabled="!canAdd"
    @click="addSelected">
    <GameIcon :icon-id="family.icon_id" :alt="family.base_name" size="xs" class="mt-0.5 shrink-0" />

    <div class="flex-1 min-w-0">
      <!-- Family name + mod count -->
      <div class="flex items-center gap-1.5">
        <span class="font-medium text-text-primary truncate">{{ family.base_name }}</span>
        <span v-if="modBoostCount > 0" class="text-[10px] text-accent-gold">{{ modBoostCount }} mod{{ modBoostCount > 1 ? 's' : '' }}</span>
      </div>

      <!-- Single tier: show stats inline, no tier selector -->
      <template v-if="tiers.length === 1">
        <div v-if="tiers[0].description" class="text-[10px] text-text-secondary truncate">
          {{ tiers[0].description }}
        </div>
        <div class="flex items-center gap-2 mt-0.5 text-[10px] text-text-dim">
          <span v-if="tiers[0].level">Lv {{ tiers[0].level }}</span>
          <span v-if="tiers[0].reset_time">{{ tiers[0].reset_time }}s cd</span>
          <span v-if="tiers[0].damage_type">{{ tiers[0].damage_type }}</span>
          <span v-if="tiers[0].mana_cost">{{ tiers[0].mana_cost }} mana</span>
          <span v-if="tiers[0].power_cost">{{ tiers[0].power_cost }} power</span>
          <span v-if="tiers[0].range">{{ tiers[0].range }}m</span>
        </div>
      </template>

      <!-- Multi-tier: tier selector buttons + selected tier stats -->
      <template v-else>
        <div class="flex items-center gap-1 mt-1 flex-wrap">
          <span class="text-[10px] text-text-dim mr-0.5">Tier:</span>
          <button
            v-for="(tier, idx) in tiers"
            :key="tier.id"
            class="w-5 h-5 text-[10px] rounded border text-center leading-none cursor-pointer transition-all"
            :class="tierButtonClass(tier, idx)"
            :title="tierTooltip(tier, idx)"
            @click.stop="selectTier(idx)">
            {{ idx + 1 }}
          </button>
        </div>

        <!-- Selected tier preview -->
        <div v-if="selectedTier" class="flex items-center gap-2 mt-0.5 text-[10px] text-text-dim">
          <span>Lv {{ selectedTier.level ?? 0 }}</span>
          <span v-if="selectedTier.reset_time">{{ selectedTier.reset_time }}s cd</span>
          <span v-if="selectedTier.damage_type">{{ selectedTier.damage_type }}</span>
          <span v-if="selectedTier.mana_cost">{{ selectedTier.mana_cost }} mana</span>
          <span v-if="selectedTier.power_cost">{{ selectedTier.power_cost }} power</span>
          <span v-if="selectedTier.pve?.damage != null" class="text-text-secondary">{{ selectedTier.pve.damage }} dmg</span>
          <span v-if="selectedTier.range">{{ selectedTier.range }}m</span>
        </div>
      </template>
    </div>

    <!-- Add/replace indicator -->
    <span
      v-if="canAdd"
      class="text-accent-gold/70 text-xs shrink-0 mt-0.5">
      {{ familyAssigned ? '↻' : '+' }}
    </span>
  </button>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { AbilityInfo, AbilityFamily } from '../../../types/gameData'
import GameIcon from '../../Shared/GameIcon.vue'

const props = withDefaults(defineProps<{
  family: AbilityFamily
  tiers: AbilityInfo[]
  assignedIds: Set<number>
  /** Family base names already on the bar (one tier per family allowed) */
  assignedFamilies?: Set<string>
  modBoostCount?: number
  /** Override the default tier selection (index into tiers array) */
  defaultTierIndex?: number
}>(), {
  modBoostCount: 0,
  defaultTierIndex: -1,
})

const emit = defineEmits<{
  add: [ability: AbilityInfo, familyBaseName: string]
}>()

const selectedTierIndex = ref(0)

const selectedTier = computed(() => props.tiers[selectedTierIndex.value] ?? null)

/** Whether this family already has a tier on the bar */
const familyAssigned = computed(() =>
  props.assignedFamilies?.has(props.family.base_name) ?? false
)

const allAssigned = computed(() =>
  props.tiers.length > 0 && props.tiers.every(t => props.assignedIds.has(t.id))
)

const isSelectedAssigned = computed(() =>
  selectedTier.value ? props.assignedIds.has(selectedTier.value.id) : true
)

/** Can the row be clicked? Yes unless all tiers are assigned or the selected tier is already the one on the bar */
const canAdd = computed(() => {
  if (allAssigned.value) return false
  if (isSelectedAssigned.value) return false
  return true
})

const rowClass = computed(() => {
  if (allAssigned.value) {
    return 'bg-surface-elevated border-border-default opacity-50 cursor-default'
  }
  if (isSelectedAssigned.value) {
    return 'bg-surface-elevated border-border-default opacity-70 cursor-default'
  }
  if (familyAssigned.value) {
    // Family has a different tier on the bar — clicking will replace
    return 'bg-surface-elevated border-border-default hover:border-accent-gold/40 hover:bg-accent-gold/5 cursor-pointer border-l-2 border-l-accent-gold/30'
  }
  return 'bg-surface-elevated border-border-default hover:border-accent-gold/40 hover:bg-accent-gold/5 cursor-pointer'
})

// Default to the highest non-assigned tier (or parent-provided default)
watch(() => [props.tiers, props.assignedIds, props.defaultTierIndex], () => {
  pickDefaultTier()
}, { immediate: true })

function pickDefaultTier() {
  // Use parent-provided default if valid
  if (props.defaultTierIndex >= 0 && props.defaultTierIndex < props.tiers.length) {
    selectedTierIndex.value = props.defaultTierIndex
    return
  }
  // Pick the highest tier that isn't assigned
  for (let i = props.tiers.length - 1; i >= 0; i--) {
    if (!props.assignedIds.has(props.tiers[i].id)) {
      selectedTierIndex.value = i
      return
    }
  }
  // All assigned, just show highest
  selectedTierIndex.value = Math.max(0, props.tiers.length - 1)
}

function selectTier(idx: number) {
  selectedTierIndex.value = idx
}

function tierButtonClass(tier: AbilityInfo, idx: number): string {
  const isAssigned = props.assignedIds.has(tier.id)
  const isSelected = idx === selectedTierIndex.value

  if (isAssigned) {
    return 'bg-surface-dark border-border-default text-text-dim opacity-50 cursor-default'
  }
  if (isSelected) {
    return 'bg-accent-gold/20 border-accent-gold/50 text-accent-gold font-semibold'
  }
  return 'bg-surface-dark border-border-default text-text-secondary hover:border-accent-gold/30 hover:text-text-primary'
}

function tierTooltip(tier: AbilityInfo, _idx: number): string {
  const assigned = props.assignedIds.has(tier.id) ? ' (assigned)' : ''
  return `${tier.name} — Lv ${tier.level ?? 0}${assigned}`
}

function addSelected() {
  if (selectedTier.value && !isSelectedAssigned.value) {
    emit('add', selectedTier.value, props.family.base_name)
  }
}
</script>
