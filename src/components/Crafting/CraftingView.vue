<template>
  <PaneLayout screen-key="crafting">
    <div class="h-full min-h-0 flex flex-col">
      <div class="flex-1 min-h-0 h-full">
        <QuickCalcTab v-if="activeTab === 'quick-calc'" />
        <ProjectsTab v-else-if="activeTab === 'projects'" />
        <LevelingTab v-else-if="activeTab === 'leveling'" />
        <HistoryTab v-else-if="activeTab === 'history'" />
        <WorkOrdersTab v-else-if="activeTab === 'work-orders'" />
        <CooksHelperTab v-else-if="activeTab === 'cooks-helper'" />
        <BreweryTab v-else-if="activeTab === 'brewery'" />
        <SkillsTab v-else-if="activeTab === 'skills'" />
        <DynamicItemsTab v-else-if="activeTab === 'dynamic-items'" />
      </div>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from "../Shared/PaneLayout.vue";
import QuickCalcTab from "./QuickCalcTab.vue";
import ProjectsTab from "./ProjectsTab.vue";
import LevelingTab from "./LevelingTab.vue";
import HistoryTab from "./HistoryTab.vue";
import WorkOrdersTab from "./WorkOrdersTab.vue";
import CooksHelperTab from "./CooksHelperTab.vue";
import BreweryTab from "./BreweryTab.vue";
import SkillsTab from "./SkillsTab.vue";
import DynamicItemsTab from "./DynamicItemsTab.vue";

defineProps<{
  activeTab: string;
}>();
</script>
