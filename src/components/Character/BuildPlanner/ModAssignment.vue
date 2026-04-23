<template>
  <div
    class="flex items-start gap-2 px-2 py-1.5 rounded text-sm group"
    :class="mod.is_augment ? 'bg-purple-900/15 border border-purple-700/30' : 'bg-surface-elevated border border-border-default'">
    <GameIcon v-if="resolvedIconId" :icon-id="resolvedIconId" size="xs" class="shrink-0 mt-0.5" />
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span
          v-if="mod.is_augment"
          class="text-[10px] font-semibold text-mod-augment uppercase">
          AUG
        </span>
        <span class="font-medium text-text-primary truncate">{{ resolvedDisplayName }}</span>
        <!-- Tier selector -->
        <TierSelector
          v-if="availableTiers.length > 1"
          :tiers="availableTiers"
          :model-value="currentTierId"
          class="ml-auto"
          @update:model-value="onTierChange" />
      </div>
      <div v-if="resolvedEffects.length > 0" class="mt-0.5">
        <EffectLine v-for="(effect, i) in resolvedEffects" :key="i" :text="effect" />
      </div>
    </div>
    <button
      class="text-red-400/60 hover:text-red-400 text-xs opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer shrink-0 mt-0.5"
      title="Remove mod"
      @click="emit('remove')">
      x
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { BuildPresetMod, TsysTierSummary } from '../../../types/buildPlanner'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import GameIcon from '../../Shared/GameIcon.vue'
import TierSelector from './TierSelector.vue'
import EffectLine from './EffectLine.vue'

const props = defineProps<{
  mod: BuildPresetMod
}>()

const emit = defineEmits<{
  remove: []
}>()

const store = useBuildPlannerStore()
const resolvedEffects = ref<string[]>([])
const resolvedDisplayName = ref(props.mod.power_name)
const resolvedIconId = ref<number | null>(null)

/** Find available tiers from the loaded slot powers */
const availableTiers = computed((): TsysTierSummary[] => {
  const power = store.slotPowers.find(p =>
    (p.internal_name ?? p.key) === props.mod.power_name
  )
  return power?.available_tiers ?? []
})

const currentTierId = computed(() => {
  if (props.mod.tier == null) return ''
  return `id_${props.mod.tier}`
})

async function onTierChange(tierId: string) {
  if (tierId === currentTierId.value) return
  await store.changeModTier(props.mod, tierId)
  await resolveEffects()
}

async function resolveEffects() {
  if (!props.mod.power_name || props.mod.tier == null) return
  try {
    const info = await invoke<{
      internal_name: string
      skill: string | null
      prefix: string | null
      suffix: string | null
      tier_effects: string[]
      icon_id: number | null
    } | null>('get_tsys_power_info', {
      powerName: props.mod.power_name,
      tier: props.mod.tier,
    })
    if (info) {
      if (info.tier_effects) resolvedEffects.value = info.tier_effects
      resolvedDisplayName.value = info.prefix ?? info.suffix ?? props.mod.power_name
      resolvedIconId.value = info.icon_id ?? null
    }
  } catch {
    // Power might not resolve
  }
}

onMounted(resolveEffects)

// Re-resolve when tier changes externally
watch(() => props.mod.tier, resolveEffects)
</script>
