<template>
  <div v-if="powers.length" class="flex flex-col gap-0.5">
    <ModPowerInline
      v-for="(p, i) in powers"
      :key="i"
      :power="p.Power"
      :tier="p.Tier"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ModPowerInline from './ModPowerInline.vue'

const props = defineProps<{
  json: string | null
}>()

interface TsysPower {
  Tier: number
  Power: string
}

const powers = computed<TsysPower[]>(() => {
  if (!props.json) return []
  try {
    const parsed = JSON.parse(props.json)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
})
</script>
