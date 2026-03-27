<template>
  <div class="flex flex-col gap-4">
    <div class="flex-1">
      <QuickCalcTab v-if="activeTab === 'quick-calc'" />
      <ProjectsTab v-else-if="activeTab === 'projects'" />
      <LevelingTab v-else-if="activeTab === 'leveling'" />
      <HistoryTab v-else-if="activeTab === 'history'" />
      <WorkOrdersTab v-else-if="activeTab === 'work-orders'" />
      <CooksHelperTab v-else-if="activeTab === 'cooks-helper'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import QuickCalcTab from "./QuickCalcTab.vue";
import ProjectsTab from "./ProjectsTab.vue";
import LevelingTab from "./LevelingTab.vue";
import HistoryTab from "./HistoryTab.vue";
import WorkOrdersTab from "./WorkOrdersTab.vue";
import CooksHelperTab from "./CooksHelperTab.vue";

defineProps<{
  activeTab: string;
}>();
</script>
