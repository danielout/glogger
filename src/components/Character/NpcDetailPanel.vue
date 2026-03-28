<template>
  <div
    class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
    :class="{ 'items-center justify-center': !npcKey }">
    <div v-if="!npcKey" class="text-border-default italic">
      Select an NPC to inspect
    </div>

    <template v-else-if="cdnData">
      <!-- Header -->
      <div class="flex flex-col gap-1">
        <div class="text-accent-gold text-base font-bold">{{ cdnData.name }}</div>
        <div v-if="cdnData.area_friendly_name" class="text-xs">
          <AreaInline :reference="cdnData.area_name ?? cdnData.area_friendly_name" />
        </div>
        <div v-if="cdnData.desc" class="text-xs text-text-secondary italic mt-1">
          {{ cdnData.desc }}
        </div>
      </div>

      <!-- Favor -->
      <NpcFavorSection
        :snapshot-tier="snapshotTier"
        :gamestate-favor="gamestateFavor" />

      <!-- Services -->
      <NpcServicesSection
        :npc="cdnData"
        :player-tier="effectiveTier" />

      <!-- Preferences -->
      <NpcPreferencesSection :npc="cdnData" />

      <!-- Trained Skills (if not already shown in services) -->
      <div v-if="cdnData.trains_skills?.length && !hasTrainingService" class="flex flex-col gap-1.5">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Training</div>
        <div class="flex flex-wrap gap-1">
          <SkillInline
            v-for="skill in cdnData.trains_skills"
            :key="skill"
            :reference="skill" />
        </div>
      </div>
    </template>

    <!-- Fallback: no CDN data but we have favor info -->
    <template v-else-if="npcKey">
      <div class="flex flex-col gap-2">
        <div class="text-accent-gold text-base font-bold">{{ displayName }}</div>
        <NpcFavorSection
          :snapshot-tier="snapshotTier"
          :gamestate-favor="gamestateFavor" />
        <div class="text-xs text-text-dim italic">No additional data available for this NPC.</div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { NpcInfo } from '../../types/gameData'
import type { GameStateFavor } from '../../types/gameState'
import { parseServices } from '../../types/npcServices'
import NpcFavorSection from './NpcDetailSections/NpcFavorSection.vue'
import NpcServicesSection from './NpcDetailSections/NpcServicesSection.vue'
import NpcPreferencesSection from './NpcDetailSections/NpcPreferencesSection.vue'
import SkillInline from '../Shared/Skill/SkillInline.vue'
import AreaInline from '../Shared/Area/AreaInline.vue'

const props = defineProps<{
  npcKey: string | null
  snapshotTier: string | null
  gamestateFavor: GameStateFavor | null
  cdnData: NpcInfo | null
}>()

const effectiveTier = computed(() =>
  props.gamestateFavor?.favor_tier ?? props.snapshotTier ?? 'Neutral'
)

const displayName = computed(() =>
  props.cdnData?.name ?? props.gamestateFavor?.npc_name ?? props.npcKey?.replace(/^NPC_/, '') ?? ''
)

const hasTrainingService = computed(() => {
  if (!props.cdnData?.services) return false
  return parseServices(props.cdnData.services).some(s => s.type === 'Training')
})
</script>
