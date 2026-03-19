<template>
  <FoodItemWithTooltip :food-name="food.name">
    <div
      class="flex items-center gap-1.5 px-2 py-0.5 rounded text-sm transition-all cursor-default"
      :class="rowClasses"
      @click="handleClick"
    >
      <GameIcon :icon-id="food.icon_id" :alt="food.name" size="xs" />
      <span class="truncate" :class="nameClasses">{{ food.name }}</span>
      <span class="text-text-muted text-xs shrink-0">
        (Lv{{ food.food_level }}<template v-if="food.gourmand_req !== null">, G{{ food.gourmand_req }}</template>)
      </span>
      <span v-if="eaten" class="text-accent-green text-xs shrink-0">&times;{{ count }}</span>
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
  selected: boolean
  selectable: boolean
  canEat: boolean
}>()

const emit = defineEmits<{
  select: [food: FoodItem]
}>()

const rowClasses = computed(() => {
  if (props.selected) return 'bg-surface-elevated'
  if (!props.canEat) return 'opacity-50'
  if (props.eaten) return 'hover:bg-accent-green/5'
  return 'hover:bg-accent-red/5'
})

const nameClasses = computed(() => {
  if (props.selected) return 'text-accent-gold'
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
