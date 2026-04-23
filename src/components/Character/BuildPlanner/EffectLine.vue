<template>
  <!-- Structured mode: label + formatted value + optional icon -->
  <div v-if="label" class="flex items-center gap-2 text-xs">
    <GameIcon v-if="iconId" :icon-id="iconId" size="xs" class="shrink-0" />
    <span class="flex-1 text-text-secondary">{{ label }}</span>
    <span v-if="formattedValue" class="font-semibold shrink-0" :class="valueColor">
      {{ formattedValue }}
    </span>
  </div>

  <!-- Raw string mode: parse and colorize inline numbers -->
  <div v-else class="text-xs text-text-secondary">
    <template v-for="(segment, i) in parsedSegments" :key="i">
      <span v-if="segment.type === 'positive'" class="font-semibold text-value-positive">{{ segment.text }}</span>
      <span v-else-if="segment.type === 'negative'" class="font-semibold text-value-negative">{{ segment.text }}</span>
      <span v-else>{{ segment.text }}</span>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GameIcon from '../../Shared/GameIcon.vue'

const props = defineProps<{
  /** Raw effect string (e.g. "Nice Attack Damage +14%") -- used in string mode */
  text?: string
  /** Structured mode fields */
  label?: string
  formattedValue?: string
  numericValue?: number
  iconId?: number | null
}>()

interface Segment {
  text: string
  type: 'text' | 'positive' | 'negative'
}

const valueColor = computed(() => {
  if (props.numericValue == null) return 'text-text-secondary'
  if (props.numericValue > 0) return 'text-value-positive'
  if (props.numericValue < 0) return 'text-value-negative'
  return 'text-text-secondary'
})

/** Parse raw effect string into segments with positive/negative coloring */
const parsedSegments = computed((): Segment[] => {
  if (!props.text) return []

  const segments: Segment[] = []
  // Match patterns like +14%, -5, +28, +0.12, -3.5%, etc.
  const regex = /([+-]\d+(?:\.\d+)?%?)/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = regex.exec(props.text)) !== null) {
    // Text before the match
    if (match.index > lastIndex) {
      segments.push({ text: props.text.slice(lastIndex, match.index), type: 'text' })
    }
    // The numeric value
    const numStr = match[1]
    const isNegative = numStr.startsWith('-')
    segments.push({ text: numStr, type: isNegative ? 'negative' : 'positive' })
    lastIndex = regex.lastIndex
  }

  // Remaining text
  if (lastIndex < props.text.length) {
    segments.push({ text: props.text.slice(lastIndex), type: 'text' })
  }

  // If no matches found, return the whole string as text
  if (segments.length === 0) {
    segments.push({ text: props.text, type: 'text' })
  }

  return segments
})
</script>
