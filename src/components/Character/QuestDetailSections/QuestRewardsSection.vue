<template>
  <div v-if="hasRewards" class="flex flex-col gap-1.5">
    <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Rewards
    </div>

    <!-- Favor -->
    <div v-if="quest.raw?.Reward_Favor && quest.raw?.FavorNpc" class="flex items-baseline gap-2 text-xs">
      <span class="text-[#c0a0e0] font-bold">+{{ quest.raw.Reward_Favor }} Favor</span>
      <NpcInline :reference="extractNpcKeyFromFavorPath(quest.raw.FavorNpc)" />
    </div>

    <!-- XP / Currency rewards -->
    <div
      v-for="(reward, idx) in quest.raw?.Rewards ?? []"
      :key="'r' + idx"
      class="flex items-baseline gap-2 text-xs text-[#60e090]">
      <span class="text-xs">&#x2726;</span>
      <template v-if="reward.T === 'SkillXp' && reward.Skill">
        <SkillInline :reference="reward.Skill" />
        <span class="text-text-muted text-xs">{{ reward.Xp }} XP</span>
      </template>
      <template v-else>
        <span>{{ getRewardTypeDisplay(reward) }}</span>
      </template>
    </div>

    <!-- Item rewards -->
    <div
      v-for="(item, idx) in quest.raw?.Rewards_Items ?? []"
      :key="'i' + idx"
      class="flex items-baseline gap-2 text-xs text-[#60e090]">
      <span class="text-xs">&#x2726;</span>
      <ItemInline :reference="item.Item" />
      <span v-if="item.StackSize > 1" class="text-text-muted text-xs">&times; {{ item.StackSize }}</span>
    </div>

    <!-- Loot profile -->
    <div v-if="quest.raw?.Rewards_NamedLootProfile" class="text-sm text-text-secondary italic">
      Loot Profile: {{ quest.raw.Rewards_NamedLootProfile }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { QuestInfo } from '../../../types/gameData'
import { getRewardTypeDisplay, extractNpcKeyFromFavorPath } from '../../../utils/questDisplay'
import NpcInline from '../../Shared/NPC/NpcInline.vue'
import SkillInline from '../../Shared/Skill/SkillInline.vue'
import ItemInline from '../../Shared/Item/ItemInline.vue'

const props = defineProps<{
  quest: QuestInfo
}>()

const hasRewards = computed(() =>
  (props.quest.raw?.Rewards?.length ?? 0) > 0
  || (props.quest.raw?.Rewards_Items?.length ?? 0) > 0
  || props.quest.raw?.Reward_Favor
  || props.quest.raw?.Rewards_NamedLootProfile
)
</script>
