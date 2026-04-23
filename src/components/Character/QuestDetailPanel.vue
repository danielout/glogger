<template>
  <div
    class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
    :class="{ 'items-center justify-center': !quest }">
    <div v-if="!quest" class="text-border-default italic">
      Select a quest to inspect
    </div>

    <template v-else>
      <!-- Header -->
      <div class="flex flex-col gap-1">
        <div class="flex items-center gap-2">
          <div class="text-accent-gold text-base font-bold">{{ getQuestDisplayName(quest) }}</div>
          <span
            v-if="evaluated.requirements.length"
            class="text-[10px] px-1.5 py-0.5 rounded border shrink-0"
            :class="eligibilityClasses(evaluated.eligibility)">
            {{ eligibilityLabel(evaluated.eligibility) }}
          </span>
        </div>
        <div class="text-xs text-text-dim flex gap-2 flex-wrap">
          <span class="text-text-secondary">{{ quest.internal_name }}</span>
          <span v-if="getQuestLevel(quest)">Lv {{ getQuestLevel(quest) }}</span>
          <span v-if="getQuestArea(quest)">
            <AreaInline :reference="getQuestArea(quest)!" />
          </span>
        </div>
      </div>

      <!-- Description -->
      <div v-if="quest.raw?.Description" class="text-sm text-text-primary/75 italic leading-relaxed">
        {{ quest.raw.Description }}
      </div>

      <!-- Preface Text -->
      <div v-if="quest.raw?.PrefaceText" class="flex flex-col gap-2">
        <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Quest Giver Dialog</div>
        <div class="text-sm text-text-secondary leading-relaxed px-3 py-2 bg-surface-base border-l-3 border-l-[#4a4a2a]">
          {{ quest.raw.PrefaceText }}
        </div>
      </div>

      <!-- Quest Info -->
      <div class="flex flex-col gap-2">
        <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Quest Info</div>
        <div class="grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-2">
          <div v-if="quest.raw?.FavorNpc" class="text-sm flex items-baseline gap-2">
            <span class="text-text-muted min-w-24">Favor NPC:</span>
            <NpcInline :reference="extractNpcKeyFromFavorPath(quest.raw.FavorNpc)" />
          </div>
          <div v-if="quest.raw?.IsCancellable !== undefined" class="text-sm flex gap-2">
            <span class="text-text-muted min-w-24">Cancellable:</span>
            <span class="text-text-secondary">{{ quest.raw.IsCancellable ? 'Yes' : 'No' }}</span>
          </div>
          <div v-if="reuseTime" class="text-sm flex gap-2">
            <span class="text-text-muted min-w-24">Reuse Time:</span>
            <span class="text-text-secondary">{{ reuseTime }}</span>
          </div>
          <div v-if="quest.raw?.WorkOrderSkill" class="text-sm flex items-baseline gap-2">
            <span class="text-text-muted min-w-24">Work Order:</span>
            <SkillInline :reference="quest.raw.WorkOrderSkill" />
          </div>
        </div>
      </div>

      <!-- Requirements -->
      <QuestRequirementsSection
        :requirements="evaluated.requirements"
        :skills-by-name="skillsByName" />

      <!-- Objectives -->
      <QuestObjectivesSection :objectives="Array.isArray(quest.raw?.Objectives) ? quest.raw.Objectives : []" />

      <!-- Rewards -->
      <QuestRewardsSection :quest="quest" />

      <!-- Success Text -->
      <div v-if="quest.raw?.SuccessText" class="flex flex-col gap-2">
        <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Completion Dialog</div>
        <div class="text-sm text-text-secondary leading-relaxed px-3 py-2 bg-surface-base border-l-3 border-l-[#4a4a2a]">
          {{ quest.raw.SuccessText }}
        </div>
      </div>

      <!-- Sources -->
      <SourcesPanel :sources="sources" :loading="sourcesLoading" />

      <!-- Keywords -->
      <div v-if="quest.raw?.Keywords?.length" class="flex flex-col gap-2">
        <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="kw in quest.raw.Keywords"
            :key="kw"
            class="text-xs px-2 py-0.5 bg-[#1a1a2a] text-[#8888bb] rounded-sm">
            {{ kw }}
          </span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { QuestInfo, EntitySources } from '../../types/gameData'
import type { GameStateSkill, GameStateFavor } from '../../types/gameState'
import { useGameDataStore } from '../../stores/gameDataStore'
import { evaluateQuestEligibility, eligibilityLabel, eligibilityClasses } from '../../composables/useQuestRequirements'
import { getQuestDisplayName, getQuestLevel, getQuestArea, formatReuseTime, extractNpcKeyFromFavorPath } from '../../utils/questDisplay'
import QuestRequirementsSection from './QuestDetailSections/QuestRequirementsSection.vue'
import QuestObjectivesSection from './QuestDetailSections/QuestObjectivesSection.vue'
import QuestRewardsSection from './QuestDetailSections/QuestRewardsSection.vue'
import SourcesPanel from '../Shared/SourcesPanel.vue'
import NpcInline from '../Shared/NPC/NpcInline.vue'
import SkillInline from '../Shared/Skill/SkillInline.vue'
import AreaInline from '../Shared/Area/AreaInline.vue'

const props = defineProps<{
  quest: QuestInfo | null
  skillsByName: Record<string, GameStateSkill>
  favorByNpc: Record<string, GameStateFavor>
}>()

const gameData = useGameDataStore()

const sources = ref<EntitySources | null>(null)
const sourcesLoading = ref(false)

const evaluated = computed(() => {
  if (!props.quest) return { eligibility: 'unknown' as const, requirements: [] }
  return evaluateQuestEligibility(props.quest, props.skillsByName, props.favorByNpc)
})

const reuseTime = computed(() => props.quest ? formatReuseTime(props.quest) : null)

watch(() => props.quest, (quest) => {
  sources.value = null
  if (!quest) return

  sourcesLoading.value = true
  gameData.getQuestSources(quest.internal_name)
    .then(s => { sources.value = s })
    .catch(e => { console.warn('Sources fetch failed:', e) })
    .finally(() => { sourcesLoading.value = false })
})
</script>
