<template>
  <FoodItemWithTooltip :food-name="food.name">
    <div
      class="w-full px-3 py-2 rounded border text-sm transition-all cursor-default flex items-center gap-2 overflow-hidden"
      :class="cardClasses"
      @click="handleClick"
    >
      <GameIcon :icon-id="food.icon_id" :alt="food.name" size="lg" />
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-1.5">
          <span class="truncate" :class="nameClasses">{{ food.name }}</span>
          <span v-if="eaten" class="text-accent-green text-xs shrink-0">&times;{{ count }}</span>
        </div>
        <div class="flex gap-2 text-xs mt-0.5" :class="metaClasses">
          <span>Lv{{ food.food_level }} {{ food.food_category }}</span>
          <span v-if="food.gourmand_req !== null">Gourm {{ food.gourmand_req }}</span>
        </div>
      </div>
      <div v-if="!canEat" class="text-accent-red text-xs shrink-0" title="Gourmand level too low">
        Req {{ food.gourmand_req }}
      </div>
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

const cardClasses = computed(() => {
  if (props.selected) {
    return 'border-accent-gold bg-surface-elevated'
  }
  if (!props.canEat) {
    return 'border-border-default bg-surface-dark opacity-50'
  }
  if (props.eaten) {
    return 'border-accent-green/30 bg-accent-green/5'
  }
  return 'border-accent-red/30 bg-accent-red/5'
})

const nameClasses = computed(() => {
  if (props.selected) return 'text-accent-gold'
  if (props.eaten) return 'text-accent-green'
  return 'text-text-primary'
})

const metaClasses = computed(() => {
  if (!props.canEat) return 'text-text-dim'
  return 'text-text-muted'
})

function handleClick() {
  if (props.selectable) {
    emit('select', props.food)
  }
}
</script>
