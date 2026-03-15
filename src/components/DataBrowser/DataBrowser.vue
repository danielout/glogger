<template>
  <div class="flex flex-col h-full">
    <div class="flex mb-4 border-b border-border-default">
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'items' }"
        @click="currentDataView = 'items'">
        Items
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'skills' }"
        @click="currentDataView = 'skills'">
        Skills
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'abilities' }"
        @click="currentDataView = 'abilities'">
        Abilities
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'recipes' }"
        @click="currentDataView = 'recipes'">
        Recipes
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'quests' }"
        @click="currentDataView = 'quests'">
        Quests
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'npcs' }"
        @click="currentDataView = 'npcs'">
        NPCs
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'effects' }"
        @click="currentDataView = 'effects'">
        Effects
      </button>
      <button
        class="px-4 py-1.5 bg-transparent border-none border-b-2 border-transparent text-text-muted cursor-pointer font-mono text-sm hover:text-text-secondary"
        :class="{ 'text-accent-gold! border-b-accent-gold!': currentDataView === 'titles' }"
        @click="currentDataView = 'titles'">
        Titles
      </button>
    </div>

    <div class="flex-1">
      <template v-if="currentDataView === 'items'">
        <ItemSearch />
      </template>
      <template v-else-if="currentDataView === 'skills'">
        <SkillBrowser />
      </template>
      <template v-else-if="currentDataView === 'abilities'">
        <AbilityBrowser />
      </template>
      <template v-else-if="currentDataView === 'recipes'">
        <RecipeBrowser />
      </template>
      <template v-else-if="currentDataView === 'quests'">
        <QuestBrowser />
      </template>
      <template v-else-if="currentDataView === 'npcs'">
        <NpcBrowser />
      </template>
      <template v-else-if="currentDataView === 'effects'">
        <EffectBrowser />
      </template>
      <template v-else-if="currentDataView === 'titles'">
        <TitleBrowser />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import ItemSearch from "./ItemSearch.vue";
import SkillBrowser from "./SkillBrowser.vue";
import AbilityBrowser from "./AbilityBrowser.vue";
import RecipeBrowser from "./RecipeBrowser.vue";
import QuestBrowser from "./QuestBrowser.vue";
import NpcBrowser from "./NpcBrowser.vue";
import EffectBrowser from "./EffectBrowser.vue";
import TitleBrowser from "./TitleBrowser.vue";

type DataView = "items" | "skills" | "abilities" | "recipes" | "quests" | "npcs" | "effects" | "titles";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
}>();

const currentDataView = ref<DataView>("items");

const entityTypeToTab: Record<string, DataView> = {
  item: "items",
  skill: "skills",
  ability: "abilities",
  recipe: "recipes",
  quest: "quests",
  npc: "npcs",
  effect: "effects",
  title: "titles",
};

// When an inline component is clicked, switch to the right browser tab
watch(() => props.navTarget, (target) => {
  if (!target) return;
  const tab = entityTypeToTab[target.type];
  if (tab) {
    currentDataView.value = tab;
  }
});
</script>
