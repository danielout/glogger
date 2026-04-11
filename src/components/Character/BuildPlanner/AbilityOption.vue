<template>
  <div
    class="flex items-center gap-2 px-2 py-1.5 rounded text-sm border transition-all"
    :class="isAssigned
      ? 'bg-surface-elevated border-border-default opacity-50'
      : 'bg-surface-elevated border-border-default hover:bg-surface-hover hover:border-accent-gold/30 cursor-pointer'"
    @click="!isAssigned && emit('add')">
    <GameIcon :icon-id="ability.icon_id" :alt="ability.name" size="xs" />
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span class="font-medium text-text-primary truncate">{{ ability.name }}</span>
        <span v-if="ability.level" class="text-[10px] text-text-dim">Lv {{ ability.level }}</span>
      </div>
      <div v-if="ability.description" class="text-[10px] text-text-secondary truncate">
        {{ ability.description }}
      </div>
      <div class="flex items-center gap-2 mt-0.5 text-[10px] text-text-dim">
        <span v-if="ability.reset_time">{{ ability.reset_time }}s cd</span>
        <span v-if="ability.damage_type">{{ ability.damage_type }}</span>
        <span v-if="ability.mana_cost">{{ ability.mana_cost }} mana</span>
        <span v-if="ability.power_cost">{{ ability.power_cost }} power</span>
        <span v-if="ability.range">{{ ability.range }}m</span>
        <span v-if="modBoostCount > 0" class="text-accent-gold">{{ modBoostCount }} mod{{ modBoostCount > 1 ? 's' : '' }} boost this</span>
      </div>
    </div>
    <button
      v-if="!isAssigned"
      class="text-accent-gold/70 hover:text-accent-gold text-xs shrink-0 cursor-pointer"
      title="Add to bar"
      @click.stop="emit('add')">
      +
    </button>
  </div>
</template>

<script setup lang="ts">
import type { AbilityInfo } from '../../../types/gameData'
import GameIcon from '../../Shared/GameIcon.vue'

withDefaults(defineProps<{
  ability: AbilityInfo
  isAssigned: boolean
  modBoostCount?: number
}>(), {
  modBoostCount: 0,
})

const emit = defineEmits<{
  add: []
}>()
</script>
