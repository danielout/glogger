<script setup lang="ts">
import { ref } from "vue";
import ItemSearch from "./ItemSearch.vue";
import SkillBrowser from "./SkillBrowser.vue";
import AbilityBrowser from "./AbilityBrowser.vue";
import RecipeBrowser from "./RecipeBrowser.vue";
import QuestBrowser from "./QuestBrowser.vue";
import NpcBrowser from "./NpcBrowser.vue";

type DataView = "items" | "skills" | "abilities" | "recipes" | "quests" | "npcs";

const currentDataView = ref<DataView>("items");
</script>

<template>
  <div class="data-browser">
    <div class="data-tabs">
      <button
        class="data-tab"
        :class="{ active: currentDataView === 'items' }"
        @click="currentDataView = 'items'">
        Items
      </button>
      <button
        class="data-tab"
        :class="{ active: currentDataView === 'skills' }"
        @click="currentDataView = 'skills'">
        Skills
      </button>
      <button
        class="data-tab"
        :class="{ active: currentDataView === 'abilities' }"
        @click="currentDataView = 'abilities'">
        Abilities
      </button>
      <button
        class="data-tab"
        :class="{ active: currentDataView === 'recipes' }"
        @click="currentDataView = 'recipes'">
        Recipes
      </button>
      <button
        class="data-tab"
        :class="{ active: currentDataView === 'quests' }"
        @click="currentDataView = 'quests'">
        Quests
      </button>
      <button
        class="data-tab"
        :class="{ active: currentDataView === 'npcs' }"
        @click="currentDataView = 'npcs'">
        NPCs
      </button>
    </div>

    <div class="data-content">
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
    </div>
  </div>
</template>

<style scoped>
.data-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.data-tabs {
  display: flex;
  gap: 0;
  margin-bottom: 1rem;
  border-bottom: 1px solid #333;
}

.data-tab {
  padding: 0.4rem 1rem;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: #666;
  cursor: pointer;
  font-family: monospace;
  font-size: 0.9rem;
}

.data-tab:hover {
  color: #aaa;
}

.data-tab.active {
  color: #e0c060;
  border-bottom-color: #e0c060;
}

.data-content {
  flex: 1;
}
</style>
