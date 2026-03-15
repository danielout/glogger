<template>
  <!-- Skill: "Learned via {SkillInline} skill" -->
  <template v-if="entry.source_type === 'Skill' && entry.skill">
    <span class="text-text-secondary">Learned via</span>
    <SkillInline :name="entry.skill" :show-icon="false" />
    <span class="text-text-secondary">skill</span>
  </template>
  <template v-else-if="entry.source_type === 'Skill'">
    <span class="text-text-secondary">Learned via skill level-up</span>
  </template>

  <!-- Training: "Trained by {NpcInline}" -->
  <template v-else-if="entry.source_type === 'Training' && entry.npc">
    <span class="text-text-secondary">Trained by</span>
    <NpcInline :name="entry.npc" />
  </template>
  <template v-else-if="entry.source_type === 'Training'">
    <span class="text-text-secondary">Trained by NPC</span>
  </template>

  <!-- Vendor: "Sold by {NpcInline}" -->
  <template v-else-if="entry.source_type === 'Vendor' && entry.npc">
    <span class="text-text-secondary">Sold by</span>
    <NpcInline :name="entry.npc" />
  </template>
  <template v-else-if="entry.source_type === 'Vendor'">
    <span class="text-text-secondary">Sold by vendor</span>
  </template>

  <!-- Barter: "Bartered from {NpcInline}" -->
  <template v-else-if="entry.source_type === 'Barter' && entry.npc">
    <span class="text-text-secondary">Bartered from</span>
    <NpcInline :name="entry.npc" />
  </template>
  <template v-else-if="entry.source_type === 'Barter'">
    <span class="text-text-secondary">Available via barter</span>
  </template>

  <!-- NpcGift: "Gift from {NpcInline}" -->
  <template v-else-if="entry.source_type === 'NpcGift' && entry.npc">
    <span class="text-text-secondary">Gift from</span>
    <NpcInline :name="entry.npc" />
  </template>
  <template v-else-if="entry.source_type === 'NpcGift'">
    <span class="text-text-secondary">NPC gift</span>
  </template>

  <!-- HangOut: "Hang out with {NpcInline}" -->
  <template v-else-if="entry.source_type === 'HangOut' && entry.npc">
    <span class="text-text-secondary">Hang out with</span>
    <NpcInline :name="entry.npc" />
  </template>
  <template v-else-if="entry.source_type === 'HangOut'">
    <span class="text-text-secondary">Hang out reward</span>
  </template>

  <!-- Simple text source types -->
  <template v-else-if="entry.source_type === 'Quest'">
    <span class="text-text-secondary">Quest reward</span>
  </template>
  <template v-else-if="entry.source_type === 'QuestObjectiveMacGuffin'">
    <span class="text-text-secondary">Quest objective item</span>
  </template>
  <template v-else-if="entry.source_type === 'Effect'">
    <span class="text-text-secondary">Granted by effect</span>
  </template>
  <template v-else-if="entry.source_type === 'Item'">
    <span class="text-text-secondary">Obtained from item</span>
  </template>
  <template v-else-if="entry.source_type === 'Recipe'">
    <span class="text-text-secondary">Crafted via recipe</span>
  </template>
  <template v-else-if="entry.source_type === 'Monster'">
    <span class="text-text-secondary">Monster drop</span>
  </template>
  <template v-else-if="entry.source_type === 'Angling'">
    <span class="text-text-secondary">Caught via Angling</span>
  </template>
  <template v-else-if="entry.source_type === 'TreasureMap'">
    <span class="text-text-secondary">Treasure map reward</span>
  </template>
  <template v-else-if="entry.source_type === 'CorpseButchering'">
    <span class="text-text-secondary">Corpse butchering</span>
  </template>
  <template v-else-if="entry.source_type === 'CorpseSkinning'">
    <span class="text-text-secondary">Corpse skinning</span>
  </template>
  <template v-else-if="entry.source_type === 'CorpseSkullExtraction'">
    <span class="text-text-secondary">Skull extraction</span>
  </template>
  <template v-else-if="(entry.source_type === 'ResourceInteractor' || entry.source_type === 'CraftedInteractor') && entry.friendly_name">
    <span class="text-text-secondary">Interactor: {{ entry.friendly_name }}</span>
  </template>
  <template v-else-if="entry.source_type === 'ResourceInteractor' || entry.source_type === 'CraftedInteractor'">
    <span class="text-text-secondary">Resource interaction</span>
  </template>

  <!-- Fallback -->
  <template v-else>
    <span class="text-text-secondary">{{ entry.source_type }}</span>
  </template>
</template>

<script setup lang="ts">
import type { SourceEntry } from "../../types/gameData";
import NpcInline from "./NPC/NpcInline.vue";
import SkillInline from "./Skill/SkillInline.vue";

defineProps<{
  entry: SourceEntry
}>();
</script>
