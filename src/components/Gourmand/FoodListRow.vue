<template>
  <FoodItemWithTooltip :food-name="food.name">
    <div
      class="flex items-center gap-1.5 px-2 py-0.5 rounded text-sm transition-all cursor-default"
      :class="rowClasses"
      @click="handleClick"
    >
      <button
        class="shrink-0 w-3.5 h-3.5 rounded border flex items-center justify-center text-[10px] leading-none transition-all"
        :class="toggleClasses"
        :title="eaten ? (manuallyMarked ? 'Remove manual mark' : 'Mark as not eaten') : 'Mark as eaten'"
        @click.stop="emit('toggle', food)"
      >
        <span v-if="eaten">&#10003;</span>
      </button>
      <GameIcon :icon-id="food.icon_id" :alt="food.name" size="xs" />
      <span class="truncate" :class="nameClasses">{{ food.name }}</span>
      <span class="text-text-muted text-xs shrink-0">
        (Lv{{ food.food_level }}<template v-if="food.gourmand_req !== null">, G{{ food.gourmand_req }}</template>)
      </span>
      <span v-if="eaten && !manuallyMarked" class="text-accent-green text-xs shrink-0">&times;{{ count }}</span>
      <span v-if="manuallyMarked" class="text-accent-blue text-xs shrink-0" title="Manually marked">manual</span>
      <span v-if="!canEat" class="text-accent-red text-xs shrink-0">Req {{ food.gourmand_req }}</span>
    </div>
  </FoodItemWithTooltip>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { FoodItem } from '../../types/gourmand'
import GameIcon from '../Shared/GameIcon.vue'
import FoodItemWithTooltip from './FoodItemWithTooltip.vue'

const props = defineProps<{
  food: FoodItem
  eaten: boolean
  count: number
  manuallyMarked: boolean
  selected: boolean
  selectable: boolean
  canEat: boolean
}>()

const emit = defineEmits<{
  select: [food: FoodItem]
  toggle: [food: FoodItem]
}>()

const toggleClasses = computed(() => {
  if (props.manuallyMarked) return 'border-accent-blue bg-accent-blue/20 text-accent-blue hover:bg-accent-blue/30'
  if (props.eaten) return 'border-accent-green bg-accent-green/20 text-accent-green hover:bg-accent-green/30'
  return 'border-border-default hover:border-text-muted hover:bg-surface-elevated'
})

const rowClasses = computed(() => {
  if (props.selected) return 'bg-surface-elevated'
  if (!props.canEat) return 'opacity-50'
  if (props.manuallyMarked) return 'hover:bg-accent-blue/5'
  if (props.eaten) return 'hover:bg-accent-green/5'
  return 'hover:bg-accent-red/5'
})

const nameClasses = computed(() => {
  if (props.selected) return 'text-accent-gold'
  if (props.manuallyMarked) return 'text-accent-blue'
  if (props.eaten) return 'text-accent-green'
  if (!props.canEat) return 'text-text-dim'
  return 'text-text-primary'
})

function handleClick() {
  if (props.selectable) {
    emit('select', props.food)
  }
}
</script>
