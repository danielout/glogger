<template>
  <div class="flex flex-col gap-1.5">
    <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Tier Progression
      <span class="text-text-muted">({{ tiers.length }} tiers)</span>
    </div>

    <!-- Compact table -->
    <div class="overflow-x-auto">
      <table class="w-full text-xs border-collapse">
        <thead>
          <tr class="text-text-muted text-[0.65rem]">
            <th class="text-left px-1.5 py-1 border-b border-surface-card font-normal">Tier</th>
            <th class="text-left px-1.5 py-1 border-b border-surface-card font-normal">Level</th>
            <th v-if="showDamage" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Damage</th>
            <th v-if="showPowerCost" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Power</th>
            <th v-if="showManaCost" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Mana</th>
            <th v-if="showArmorCost" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Armor</th>
            <th v-if="showHealthCost" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Health</th>
            <th v-if="showRange" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Range</th>
            <th v-if="showCooldown" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">CD</th>
            <th v-if="showRageCost" class="text-right px-1.5 py-1 border-b border-surface-card font-normal">Rage</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(tier, idx) in tiers"
            :key="tier.id"
            class="hover:bg-[#1e1e1e] cursor-pointer"
            :class="{ 'bg-[#1a1a2e]': expandedTier === tier.id }"
            @click="toggleExpand(tier.id)">
            <td class="px-1.5 py-1 border-b border-surface-dark text-text-primary">
              {{ idx + 1 }}
            </td>
            <td class="px-1.5 py-1 border-b border-surface-dark text-text-secondary font-mono">
              {{ tier.level ?? 0 }}
            </td>
            <td v-if="showDamage" class="text-right px-1.5 py-1 border-b border-surface-dark text-text-secondary font-mono">
              <span v-if="tier.pve?.damage != null">{{ tier.pve.damage }}</span>
            </td>
            <td v-if="showPowerCost" class="text-right px-1.5 py-1 border-b border-surface-dark font-mono text-[#c0a040]">
              <span v-if="tier.power_cost != null">{{ tier.power_cost }}</span>
              <span v-else-if="tier.pve?.power_cost != null">{{ tier.pve.power_cost }}</span>
            </td>
            <td v-if="showManaCost" class="text-right px-1.5 py-1 border-b border-surface-dark font-mono text-[#6090c0]">
              <span v-if="tier.mana_cost != null">{{ tier.mana_cost }}</span>
            </td>
            <td v-if="showArmorCost" class="text-right px-1.5 py-1 border-b border-surface-dark font-mono text-text-muted">
              <span v-if="tier.armor_cost != null">{{ tier.armor_cost }}</span>
            </td>
            <td v-if="showHealthCost" class="text-right px-1.5 py-1 border-b border-surface-dark font-mono text-accent-red">
              <span v-if="tier.health_cost != null">{{ tier.health_cost }}</span>
            </td>
            <td v-if="showRange" class="text-right px-1.5 py-1 border-b border-surface-dark text-text-secondary font-mono">
              <span v-if="tier.range != null">{{ tier.range }}m</span>
            </td>
            <td v-if="showCooldown" class="text-right px-1.5 py-1 border-b border-surface-dark text-text-secondary font-mono">
              <span v-if="tier.reset_time != null">{{ tier.reset_time }}s</span>
            </td>
            <td v-if="showRageCost" class="text-right px-1.5 py-1 border-b border-surface-dark font-mono text-[#c06060]">
              <span v-if="tier.pve?.rage_cost != null">{{ tier.pve.rage_cost }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Expanded tier detail -->
    <div v-if="expandedTier != null && expandedAbility" class="bg-surface-dark border border-surface-card p-3 flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <span class="text-text-primary text-xs font-medium">{{ expandedAbility.name }}</span>
        <span class="text-text-dim text-[0.65rem]">ID: {{ expandedAbility.id }}</span>
      </div>

      <div v-if="expandedAbility.description" class="text-xs text-text-secondary italic">
        {{ expandedAbility.description }}
      </div>

      <div v-if="expandedAbility.prerequisite" class="text-xs text-[#e08060] px-2 py-1 bg-[#151515] border-l-2 border-l-[#4a2a2a]">
        Requires: {{ expandedAbility.prerequisite }}
      </div>

      <div v-if="expandedAbility.special_info" class="text-xs text-text-secondary italic">
        {{ expandedAbility.special_info }}
      </div>

      <!-- Per-tier sources -->
      <SourcesPanel :sources="tierSources" :loading="tierSourcesLoading" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useGameDataStore } from '../../stores/gameDataStore'
import type { AbilityInfo, EntitySources } from '../../types/gameData'
import SourcesPanel from '../Shared/SourcesPanel.vue'

const props = defineProps<{
  tiers: AbilityInfo[]
}>()

const store = useGameDataStore()
const expandedTier = defineModel<number | null>('expandedTierId', { default: null })
const tierSources = ref<EntitySources | null>(null)
const tierSourcesLoading = ref(false)

const expandedAbility = computed(() =>
  props.tiers.find(t => t.id === expandedTier.value) ?? null
)

// Column visibility: only show columns that have data in at least one tier
const showDamage = computed(() => props.tiers.some(t => t.pve?.damage != null))
const showPowerCost = computed(() => props.tiers.some(t => t.power_cost != null || t.pve?.power_cost != null))
const showManaCost = computed(() => props.tiers.some(t => t.mana_cost != null))
const showArmorCost = computed(() => props.tiers.some(t => t.armor_cost != null))
const showHealthCost = computed(() => props.tiers.some(t => t.health_cost != null))
const showRange = computed(() => props.tiers.some(t => t.range != null))
const showCooldown = computed(() => props.tiers.some(t => t.reset_time != null))
const showRageCost = computed(() => props.tiers.some(t => t.pve?.rage_cost != null))

function toggleExpand(id: number) {
  expandedTier.value = expandedTier.value === id ? null : id
}

// Load sources when a tier is expanded
watch(expandedTier, async (id) => {
  if (id == null) {
    tierSources.value = null
    return
  }
  tierSourcesLoading.value = true
  try {
    tierSources.value = await store.getAbilitySources(id)
  } finally {
    tierSourcesLoading.value = false
  }
})
</script>
