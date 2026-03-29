<template>
  <div
    class="flex-1 overflow-y-auto border border-surface-elevated p-4 flex flex-col gap-4"
    :class="{ 'items-center justify-center': !skill }">
    <div v-if="!skill" class="text-border-default italic">
      Select a skill to inspect
    </div>

    <template v-else>
      <!-- Header -->
      <div class="flex gap-3 items-start">
        <div class="flex-1 min-w-0">
          <div class="text-accent-gold text-base font-bold mb-0.5">{{ skill.skill_name }}</div>
          <div class="text-xs text-text-secondary mb-1">
            Level <SkillLevelDisplay :skill="skill"><span class="text-white font-bold">{{ skillTotalLevel(skill) }}</span></SkillLevelDisplay>
          </div>
          <div class="flex items-center gap-2">
            <span v-if="cdnData?.combat === true" class="text-[0.65rem] px-1.5 py-0.5 bg-red-900/30 border border-red-700/40 text-red-300 rounded">Combat</span>
            <span v-else-if="cdnData?.combat === false" class="text-[0.65rem] px-1.5 py-0.5 bg-blue-900/30 border border-blue-700/40 text-blue-300 rounded">Non-Combat</span>
            <span v-if="isActive" class="text-[0.65rem] px-1.5 py-0.5 bg-accent-gold/20 border border-accent-gold/40 text-accent-gold rounded">Active</span>
          </div>
        </div>

        <button
          class="px-2 py-1 text-xs rounded cursor-pointer transition-all border shrink-0"
          :class="isTracked
            ? 'bg-accent-gold/20 border-accent-gold/40 text-accent-gold hover:bg-accent-gold/30'
            : 'bg-surface-base border-border-default text-text-muted hover:text-accent-gold hover:border-accent-gold/40'"
          @click="toggleTrack">
          {{ isTracked ? 'Untrack' : 'Track' }}
        </button>
      </div>

      <!-- XP Progress -->
      <div class="flex flex-col gap-1">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">XP Progress</div>
        <div class="h-2 bg-border-default rounded-sm overflow-hidden">
          <div class="h-full bg-accent-gold rounded-sm transition-all duration-300" :style="{ width: xpPercent + '%' }"></div>
        </div>
        <div class="text-xs text-text-secondary">
          <template v-if="skill.tnl <= 0">MAX</template>
          <template v-else>{{ skill.xp.toLocaleString() }} / {{ (skill.xp + skill.tnl).toLocaleString() }} XP <span class="text-text-dim">({{ skill.tnl.toLocaleString() }} to next level)</span></template>
        </div>
      </div>

      <!-- Session Stats -->
      <div v-if="session" class="flex flex-col gap-1.5">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Session</div>
        <div class="grid grid-cols-2 gap-1.5">
          <div class="text-xs">
            <span class="text-text-muted">XP Gained:</span>
            <span class="text-text-primary font-bold ml-1">{{ session.xpGained.toLocaleString() }}</span>
          </div>
          <div class="text-xs">
            <span class="text-text-muted">XP/Hour:</span>
            <span class="text-text-primary font-bold ml-1">{{ xphr }}</span>
          </div>
          <div class="text-xs">
            <span class="text-text-muted">Levels Gained:</span>
            <span class="text-text-primary font-bold ml-1">{{ session.levelsGained }}</span>
          </div>
          <div class="text-xs">
            <span class="text-text-muted">Next Level:</span>
            <span class="text-text-primary font-bold ml-1">{{ ttl }}</span>
          </div>
        </div>
      </div>

      <!-- Description -->
      <div v-if="cdnData?.description" class="flex flex-col gap-1">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Description</div>
        <div class="text-xs text-text-secondary italic">{{ cdnData.description }}</div>
      </div>

      <!-- Advancement Hints -->
      <div v-if="relevantHints.length" class="flex flex-col gap-1.5">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Advancement Hints</div>
        <div class="flex flex-col gap-1">
          <div
            v-for="hint in relevantHints"
            :key="hint.level"
            class="text-xs flex gap-2 px-2 py-0.5 bg-[#151515]">
            <span class="text-text-muted min-w-14 shrink-0">Lv {{ hint.level }}:</span>
            <span class="text-text-secondary">{{ hint.text }}</span>
          </div>
        </div>
      </div>

      <!-- Upcoming Rewards -->
      <div v-if="upcomingRewards.length" class="flex flex-col gap-1.5">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Upcoming Rewards</div>
        <div class="flex flex-col gap-1">
          <div
            v-for="reward in upcomingRewards"
            :key="reward.level"
            class="text-xs flex items-center gap-2 px-2 py-0.5 bg-[#151515]">
            <span class="text-text-muted min-w-14 shrink-0">Lv {{ reward.level }}:</span>
            <!-- Ability reward: show as inline with tooltip -->
            <template v-if="reward.ability">
              <AbilityInline :ability="reward.ability" />
            </template>
            <!-- Ability reward we couldn't resolve: show internal name -->
            <template v-else-if="reward.abilityKey">
              <span class="text-entity-ability">{{ reward.abilityKey }}</span>
            </template>
            <!-- Bonus to skill reward -->
            <template v-if="reward.bonusSkill">
              <span class="text-text-secondary">Bonus to</span>
              <SkillInline :reference="reward.bonusSkill" />
            </template>
            <!-- Recipe reward -->
            <template v-if="reward.recipeKey">
              <span class="text-text-secondary">Recipe: {{ reward.recipeKey }}</span>
            </template>
            <!-- Fallback -->
            <span v-if="!reward.ability && !reward.abilityKey && !reward.bonusSkill && !reward.recipeKey" class="text-text-secondary">
              {{ reward.fallback }}
            </span>
          </div>
        </div>
      </div>

      <!-- Related Abilities (with inline tooltips and sources) -->
      <div v-if="abilities.length" class="flex flex-col gap-1.5">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
          Related Abilities ({{ abilities.length }})
        </div>
        <div class="flex flex-col gap-0.5">
          <div
            v-for="ability in abilities"
            :key="ability.id"
            class="text-xs flex items-center gap-2 px-2 py-0.5 border-b border-[#151515] hover:bg-surface-base">
            <span class="text-text-muted text-[0.72rem] min-w-14 shrink-0">[Lv {{ ability.level || 0 }}]</span>
            <AbilityInline :ability="ability" class="flex-1" />
            <span v-if="abilitySourceLabels[ability.id]" class="text-text-dim text-[0.6rem] shrink-0">
              {{ abilitySourceLabels[ability.id] }}
            </span>
          </div>
        </div>
      </div>

      <!-- Keywords -->
      <div v-if="cdnData?.keywords?.length" class="flex flex-col gap-1.5">
        <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">Keywords</div>
        <div class="flex flex-wrap gap-1">
          <span
            v-for="kw in cdnData.keywords"
            :key="kw"
            class="text-[0.72rem] px-1.5 py-0.5 bg-[#1a1a2e] border border-[#2a2a4e] text-[#7ec8e3]">{{ kw }}</span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import { skillTotalLevel, type GameStateSkill } from '../../types/gameState'
import type { SkillInfo, AbilityInfo, EntitySources } from '../../types/gameData'
import SkillLevelDisplay from '../Shared/SkillLevelDisplay.vue'
import AbilityInline from '../Shared/Ability/AbilityInline.vue'
import SkillInline from '../Shared/Skill/SkillInline.vue'

interface ParsedReward {
  level: number
  ability: AbilityInfo | null
  abilityKey: string | null
  bonusSkill: string | null
  recipeKey: string | null
  fallback: string
}

const props = defineProps<{
  skill: GameStateSkill | null
}>()

const store = useGameStateStore()
const gameData = useGameDataStore()

const cdnData = ref<SkillInfo | null>(null)
const abilities = ref<AbilityInfo[]>([])
const abilitySourceLabels = ref<Record<number, string>>({})

const session = computed(() =>
  props.skill ? store.sessionSkills[props.skill.skill_name] ?? null : null
)

const isActive = computed(() => {
  if (!props.skill || !store.activeSkills) return false
  return store.activeSkills.skill1_name === props.skill.skill_name
    || store.activeSkills.skill2_name === props.skill.skill_name
})

const isTracked = computed(() =>
  props.skill ? store.isSkillTracked(props.skill.skill_name) : false
)

async function toggleTrack() {
  if (!props.skill) return
  if (isTracked.value) {
    await store.untrackSkill(props.skill.skill_name)
  } else {
    await store.trackSkill(props.skill.skill_name)
  }
}

const xphr = computed(() => {
  if (!session.value) return '0'
  return store.xpPerHour(session.value).toLocaleString()
})

const ttl = computed(() => {
  if (!session.value) return '—'
  return store.timeToNextLevel(session.value)
})

const xpPercent = computed(() => {
  if (!props.skill || props.skill.tnl <= 0) return 100
  const total = props.skill.xp + props.skill.tnl
  if (total <= 0) return 0
  return Math.min(100, Math.round((props.skill.xp / total) * 100))
})

// Parse advancement hints from CDN data, show levels >= current
const relevantHints = computed(() => {
  if (!cdnData.value?.advancement_hints || !props.skill) return []
  const hints = cdnData.value.advancement_hints as Record<string, string>
  return Object.entries(hints)
    .map(([level, text]) => ({ level: Number(level), text }))
    .filter(h => h.level >= props.skill!.level)
    .sort((a, b) => a.level - b.level)
    .slice(0, 5)
})

// Parse upcoming rewards from CDN data, resolving ability names via loaded abilities
const upcomingRewards = computed<ParsedReward[]>(() => {
  if (!cdnData.value?.rewards || !props.skill) return []
  const rewards = cdnData.value.rewards as Record<string, Record<string, unknown>>

  // Build a lookup from internal ability key to AbilityInfo
  const abilityByInternalName: Record<string, AbilityInfo> = {}
  for (const ab of abilities.value) {
    // The raw_json often has an InternalName or the key format
    const internalName = ab.raw_json?.InternalName as string | undefined
    if (internalName) abilityByInternalName[internalName] = ab
    // Also index by the display name for fallback matching
    abilityByInternalName[ab.name] = ab
  }

  return Object.entries(rewards)
    .map(([level, data]): ParsedReward => {
      const abilityKey = data.Ability as string | undefined
      const bonusSkill = data.BonusToSkill as string | undefined
      const recipeKey = data.Recipe as string | undefined

      let resolvedAbility: AbilityInfo | null = null
      if (abilityKey) {
        resolvedAbility = abilityByInternalName[abilityKey] ?? null
      }

      return {
        level: Number(level),
        ability: resolvedAbility,
        abilityKey: resolvedAbility ? null : (abilityKey ?? null),
        bonusSkill: bonusSkill ?? null,
        recipeKey: recipeKey ?? null,
        fallback: !abilityKey && !bonusSkill && !recipeKey ? JSON.stringify(data) : '',
      }
    })
    .filter(r => r.level > props.skill!.level)
    .sort((a, b) => a.level - b.level)
    .slice(0, 10)
})

/** Summarize the primary source type for an ability as a short label */
function summarizeSource(sources: EntitySources): string {
  // Check CDN sources first
  for (const s of sources.cdn_sources) {
    switch (s.source_type) {
      case 'Skill': return 'Skill level-up'
      case 'Training': return s.npc ? `Trainer: ${s.npc}` : 'Trainer'
      case 'Quest': return 'Quest reward'
      case 'HangOut': return 'Hang out'
      case 'Effect': return 'Effect'
      case 'NpcGift': return 'NPC gift'
      case 'Monster': return 'Monster drop'
    }
  }
  // Check items
  if (sources.bestowed_by_items.length > 0) {
    return 'Skill book'
  }
  // Check quests
  if (sources.rewarded_by_quests.length > 0) {
    return 'Quest reward'
  }
  return ''
}

// Load CDN data + abilities + sources when skill changes
watch(() => props.skill?.skill_name, async (name) => {
  cdnData.value = null
  abilities.value = []
  abilitySourceLabels.value = {}
  if (!name) return

  try {
    const [info, abs] = await Promise.all([
      gameData.resolveSkill(name),
      gameData.getAbilitiesForSkill(name),
    ])
    cdnData.value = info
    abilities.value = abs.sort((a, b) => (a.level || 0) - (b.level || 0))

    // Load sources for each ability in the background
    const labels: Record<number, string> = {}
    const sourcePromises = abs.map(async (ab) => {
      try {
        const sources = await gameData.getAbilitySources(ab.id)
        const label = summarizeSource(sources)
        if (label) labels[ab.id] = label
      } catch {
        // silently skip failed source lookups
      }
    })
    await Promise.all(sourcePromises)
    abilitySourceLabels.value = labels
  } catch (e) {
    console.warn('Failed to load skill detail:', e)
  }
}, { immediate: true })
</script>
