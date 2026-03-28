<template>
  <div v-if="objectives.length" class="flex flex-col gap-1.5">
    <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Objectives
    </div>
    <div
      v-for="(obj, idx) in objectives"
      :key="idx"
      class="text-[0.82rem] text-text-secondary flex gap-2 items-baseline">
      <span class="text-[#6a9fb5] font-bold min-w-20 text-xs shrink-0">
        {{ getObjectiveTypeDisplay(obj.Type) }}
      </span>
      <span class="flex-1">
        {{ obj.Description }}
        <ItemInline v-if="obj.ItemName" :reference="obj.ItemName" class="ml-1" />
      </span>
      <span v-if="obj.Number" class="text-text-muted text-xs shrink-0">({{ obj.Number }})</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { QuestObjective } from '../../../types/gameData'
import { getObjectiveTypeDisplay } from '../../../utils/questDisplay'
import ItemInline from '../../Shared/Item/ItemInline.vue'

defineProps<{
  objectives: QuestObjective[]
}>()
</script>
