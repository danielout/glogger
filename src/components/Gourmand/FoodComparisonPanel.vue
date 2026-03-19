<template>
  <div v-if="selectedMeal || selectedSnack" class="bg-surface-card border border-border-default rounded p-3">
    <div class="flex items-center gap-3 mb-2">
      <h3 class="text-accent-gold font-bold uppercase tracking-wide text-xs">Food Buff</h3>
      <div class="flex items-center gap-2 text-xs text-text-muted">
        <span v-if="selectedMeal" class="text-accent-green">{{ selectedMeal.name }}</span>
        <span v-if="selectedMeal && selectedSnack">+</span>
        <span v-if="selectedSnack" class="text-accent-blue">{{ selectedSnack.name }}</span>
      </div>
      <button
        class="ml-auto px-2 py-0.5 text-xs bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
        @click="$emit('clear')"
      >
        clear
      </button>
    </div>

    <div class="grid grid-cols-2 gap-x-4 gap-y-0.5 text-xs">
      <div
        v-for="effect in combinedEffects"
        :key="effect.label"
        class="flex justify-between"
      >
        <span class="text-text-secondary">{{ effect.label }}</span>
        <span class="text-accent-green font-medium">+{{ effect.value }}</span>
      </div>
      <div
        v-for="text in textEffects"
        :key="text"
        class="text-text-muted italic"
      >
        {{ text }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { FoodItem } from '../../types/gourmand'

const props = defineProps<{
  selectedMeal: FoodItem | null
  selectedSnack: FoodItem | null
}>()

defineEmits<{
  clear: []
}>()

interface ParsedEffect {
  label: string
  value: number
  suffix: string
}

const effectPattern = /^(.+?)\s+\+(\d+(?:\.\d+)?)\s*(.*)$/

function parseEffects(descs: string[]): { numeric: ParsedEffect[]; text: string[] } {
  const numeric: ParsedEffect[] = []
  const text: string[] = []

  for (const desc of descs) {
    const match = desc.match(effectPattern)
    if (match) {
      const label = match[3] ? `${match[1]} ${match[3]}` : match[1]
      numeric.push({ label, value: parseFloat(match[2]), suffix: match[3] })
    } else {
      text.push(desc)
    }
  }
  return { numeric, text }
}

const combinedEffects = computed(() => {
  const allDescs = [
    ...(props.selectedMeal?.effect_descs ?? []),
    ...(props.selectedSnack?.effect_descs ?? []),
  ]

  const { numeric } = parseEffects(allDescs)

  const merged = new Map<string, number>()
  for (const e of numeric) {
    merged.set(e.label, (merged.get(e.label) ?? 0) + e.value)
  }

  return Array.from(merged.entries()).map(([label, value]) => ({
    label,
    value: Number.isInteger(value) ? value.toString() : value.toFixed(1),
  }))
})

const textEffects = computed(() => {
  const allDescs = [
    ...(props.selectedMeal?.effect_descs ?? []),
    ...(props.selectedSnack?.effect_descs ?? []),
  ]

  const { text } = parseEffects(allDescs)
  // Deduplicate text effects like "Lasts 1 hour (plus Gourmand bonus)"
  return [...new Set(text)]
})
</script>
