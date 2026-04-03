<template>
  <time :datetime="isoValue" :title="tooltip" class="timestamp whitespace-nowrap">{{ display }}</time>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  formatTimeShort,
  formatTimeFull,
  formatDateShort,
  formatDate,
  formatDateTimeShort,
  formatDateTimeFull,
  formatRelative,
  formatSmart,
  getTimezoneSuffix,
  parseUtc,
} from '../../composables/useTimestamp'

export type TimestampGranularity =
  | 'time-short'      // 14:30
  | 'time-full'       // 14:30:00
  | 'date-short'      // Mar 26
  | 'date-full'       // 2026-03-26
  | 'datetime-short'  // Mar 26, 14:30
  | 'datetime-full'   // 2026-03-26 14:30:00
  | 'relative'        // 2m ago
  | 'smart'           // time if today, datetime-short otherwise

const props = withDefaults(defineProps<{
  /** UTC timestamp string from the database */
  value: string
  /** Display granularity */
  granularity?: TimestampGranularity
}>(), {
  granularity: 'smart',
})

const formatters: Record<TimestampGranularity, (ts: string) => string> = {
  'time-short': formatTimeShort,
  'time-full': formatTimeFull,
  'date-short': formatDateShort,
  'date-full': formatDate,
  'datetime-short': formatDateTimeShort,
  'datetime-full': formatDateTimeFull,
  'relative': formatRelative,
  'smart': formatSmart,
}

const display = computed(() => {
  if (!props.value) return ''
  const fn = formatters[props.granularity]
  return fn(props.value)
})

const tooltip = computed(() => {
  if (!props.value) return ''
  return formatDateTimeFull(props.value) + getTimezoneSuffix()
})

const isoValue = computed(() => {
  if (!props.value) return ''
  const d = parseUtc(props.value)
  return isNaN(d.getTime()) ? '' : d.toISOString()
})
</script>
