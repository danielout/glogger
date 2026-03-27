<template>
  <div class="flex flex-col h-full">
    <div class="flex-1 min-h-0">
      <template v-if="activeTab === 'items'">
        <ItemSearch :nav-target="props.navTarget" />
      </template>
      <template v-else-if="activeTab === 'skills'">
        <SkillBrowser :nav-target="props.navTarget" />
      </template>
      <template v-else-if="activeTab === 'abilities'">
        <AbilityBrowser />
      </template>
      <template v-else-if="activeTab === 'recipes'">
        <RecipeBrowser :nav-target="props.navTarget" />
      </template>
      <template v-else-if="activeTab === 'quests'">
        <QuestBrowser :nav-target="props.navTarget" />
      </template>
      <template v-else-if="activeTab === 'npcs'">
        <NpcBrowser :nav-target="props.navTarget" />
      </template>
      <template v-else-if="activeTab === 'effects'">
        <EffectBrowser />
      </template>
      <template v-else-if="activeTab === 'titles'">
        <TitleBrowser />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { EntityNavigationTarget } from "../../composables/useEntityNavigation";
import ItemSearch from "./ItemSearch.vue";
import SkillBrowser from "./SkillBrowser.vue";
import AbilityBrowser from "./AbilityBrowser.vue";
import RecipeBrowser from "./RecipeBrowser.vue";
import QuestBrowser from "./QuestBrowser.vue";
import NpcBrowser from "./NpcBrowser.vue";
import EffectBrowser from "./EffectBrowser.vue";
import TitleBrowser from "./TitleBrowser.vue";

const props = defineProps<{
  navTarget?: EntityNavigationTarget | null;
  activeTab: string;
}>();
</script>
