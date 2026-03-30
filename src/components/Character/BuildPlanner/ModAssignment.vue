<template>
  <div
    class="flex items-start gap-2 px-2 py-1.5 rounded text-sm group"
    :class="mod.is_augment ? 'bg-purple-900/15 border border-purple-700/30' : 'bg-surface-elevated border border-border-default'">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span
          v-if="mod.is_augment"
          class="text-[10px] font-semibold text-purple-400 uppercase">
          AUG
        </span>
        <span class="font-medium text-text-primary truncate">{{ mod.power_name }}</span>
      </div>
      <div v-if="resolvedEffects.length > 0" class="text-xs text-text-secondary mt-0.5">
        <div v-for="(effect, i) in resolvedEffects" :key="i">{{ effect }}</div>
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
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { BuildPresetMod } from '../../../types/buildPlanner'

const props = defineProps<{
  mod: BuildPresetMod
}>()

const emit = defineEmits<{
  remove: []
}>()

const resolvedEffects = ref<string[]>([])

onMounted(async () => {
  if (!props.mod.power_name || props.mod.tier == null) return
  try {
    const info = await invoke<{
      internal_name: string
      skill: string | null
      tier_effects: string[]
    } | null>('get_tsys_power_info', {
      powerName: props.mod.power_name,
      tier: props.mod.tier,
    })
    if (info?.tier_effects) {
      resolvedEffects.value = info.tier_effects
    }
  } catch {
    // Power might not resolve — that's fine
  }
})
</script>
