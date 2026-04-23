<template>
  <div v-if="requirements.length" class="flex flex-col gap-1.5">
    <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Requirements
    </div>
    <div
      v-for="(evReq, idx) in requirements"
      :key="idx"
      class="flex items-baseline gap-2 text-xs">
      <span
        class="w-4 text-center shrink-0 text-xs font-bold"
        :class="requirementStatusColor(evReq.status)">
        {{ requirementStatusIcon(evReq.status) }}
      </span>

      <!-- Skill requirement with inline -->
      <template v-if="evReq.requirement.T === 'MinSkillLevel' && evReq.requirement.Skill">
        <SkillInline :reference="evReq.requirement.Skill" />
        <span class="text-text-muted text-xs">
          level
          <span :class="evReq.status === 'met' ? 'text-green-400' : 'text-red-400'">
            {{ playerSkillLevel(evReq.requirement.Skill) }}
          </span>
          / {{ evReq.requirement.MinSkillLevel }}
        </span>
      </template>

      <!-- Favor requirement with NPC inline -->
      <template v-else-if="evReq.requirement.T === 'MinFavorLevel' && evReq.requirement.Npc">
        <NpcInline :reference="extractNpcKeyFromFavorPath(evReq.requirement.Npc)" />
        <span class="text-text-muted text-xs">
          need {{ tierDisplayName(String(evReq.requirement.Level)) }} favor
        </span>
      </template>

      <!-- Quest prerequisite with inline -->
      <template v-else-if="evReq.requirement.T === 'QuestCompleted' && evReq.requirement.Quest">
        <span class="text-text-muted text-xs">Complete:</span>
        <QuestInline :reference="evReq.requirement.Quest" />
      </template>

      <!-- Active combat skill -->
      <template v-else-if="evReq.requirement.T === 'ActiveCombatSkill' && evReq.requirement.Skill">
        <span class="text-text-muted text-xs">Active skill:</span>
        <SkillInline :reference="evReq.requirement.Skill" />
      </template>

      <!-- Fallback -->
      <template v-else>
        <span class="text-text-secondary">{{ evReq.detail }}</span>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { skillTotalLevel, type GameStateSkill } from '../../../types/gameState'
import { requirementStatusIcon, requirementStatusColor, type EvaluatedRequirement } from '../../../composables/useQuestRequirements'
import { extractNpcKeyFromFavorPath } from '../../../utils/questDisplay'
import { tierDisplayName } from '../../../composables/useFavorTiers'
import SkillInline from '../../Shared/Skill/SkillInline.vue'
import NpcInline from '../../Shared/NPC/NpcInline.vue'
import QuestInline from '../../Shared/Quest/QuestInline.vue'

const props = defineProps<{
  requirements: EvaluatedRequirement[]
  skillsByName: Record<string, GameStateSkill>
}>()

function playerSkillLevel(skillName: string): number {
  const skill = props.skillsByName[skillName]
  return skill ? skillTotalLevel(skill) : 0
}
</script>
